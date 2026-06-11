use std::sync::Arc;

use async_lock::RwLock;
use krill_common::{
    ActivityInfo, ActivityMetadata, ActivityState, ActivityStoreKey, ActivitySubscriberChannel,
    Byte28Array, QuicProtocolOp, QuicTransmissionEncoder, QuicTransmissionError,
};

use crate::{
    ClientUtils, FrostParticipateMessageWrapper, QuicClient, RustFfiError, RustTypeActivityInfo,
    RustTypeActivityMetadata,
};

#[derive(Debug, uniffi::Record)]
pub struct ActivityListenerOutcome {
    pub data: RustTypeActivitySubscriberChannel,
}

#[uniffi::export(with_foreign)]
pub trait ActivityListener: Send + Sync {
    fn on_recv(&self, value: ActivityListenerOutcome);
}

#[derive(uniffi::Object)]
pub struct ActivityEmitter;

#[uniffi::export]
impl ActivityEmitter {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }

    pub async fn start(
        &self,
        listener: Arc<dyn ActivityListener>,
        domain_or_ip: String,
        activity_id: String,
        timezone: i32,
    ) -> Result<(), RustFfiError> {
        let mut store_key: Byte28Array = [0u8; 28];
        faster_hex::hex_decode(activity_id.as_bytes(), &mut store_key)
            .or(Err(RustFfiError::InvalidActivityId))?;

        let storage = crate::app_storage()?;

        let stored_org_info = storage
            .get_org_info(&domain_or_ip)
            .await?
            .ok_or(RustFfiError::OrgNotFound)?;

        let active = if let Some(active) = stored_org_info.activities.get(&activity_id) {
            active
        } else {
            return Err(RustFfiError::InvalidActivityId);
        };

        let wrapped = FrostParticipateMessageWrapper::new(
            &domain_or_ip,
            ActivityStoreKey(store_key),
            active.metadata.threshold,
        )
        .await?;

        let encoded_op = bitcode::encode(&QuicProtocolOp::Participate(Box::new(wrapped.0)));

        let (send, mut recv) = QuicClient::setup_connect(&domain_or_ip)
            .await?
            .open_bi()
            .await
            .map_err(|error| {
                let error: QuicTransmissionError = error.into();

                error
            })?;

        let send = async_dup::Arc::new(RwLock::new(send));

        let send_outer = send.clone();

        let len = (encoded_op.len() as u32).to_be_bytes();

        send_outer
            .write()
            .await
            .write_all(&len)
            .await
            .map_err(|error| {
                let error: QuicTransmissionError = error.into();

                error
            })?;
        send_outer
            .write()
            .await
            .write_all(&encoded_op)
            .await
            .map_err(|error| {
                let error: QuicTransmissionError = error.into();

                error
            })?;

        smol::spawn(async move {
            loop {
                // read length prefix
                let mut len_buf = [0u8; 4];

                if recv.read_exact(&mut len_buf).await.is_err() {
                    ClientUtils::log_to_logcat("Terminated....REACHED");

                    listener.on_recv(ActivityListenerOutcome {
                        data: RustTypeActivitySubscriberChannel::Terminated,
                    });

                    break; // server closed stream
                }

                let len = u32::from_be_bytes(len_buf) as usize;

                let mut data = vec![0u8; len];

                if recv.read_exact(&mut data).await.is_err() {
                    break;
                }

                let response = std::mem::take(&mut data);

                if response.is_empty() {
                    ClientUtils::log_to_logcat("Empty response....REACHED");

                    // listener.on_tick(ActivityListenerOutcome {
                    //     listen: false,
                    //     data: "Received empty response the relay".to_string(),
                    // });
                }

                if response[0] == 0 {
                    match QuicTransmissionEncoder::decode_success::<ActivitySubscriberChannel>(
                        &response[1..],
                    ) {
                        Ok(decoded_response) => {
                            listener.on_recv(ActivityListenerOutcome {
                                data: decoded_response.into(),
                            });
                        }
                        Err(error) => {
                            ClientUtils::log_to_logcat(&format!(
                                "QUIC TRAMISSION ERROR....REACHED: {error:?}"
                            ));

                            listener.on_recv(ActivityListenerOutcome {
                                data: RustTypeActivitySubscriberChannel::Terminated,
                            });
                        }
                    }
                } else {
                    let failure = QuicTransmissionEncoder::decode_failure(&response[1..]);
                    ClientUtils::log_to_logcat(&format!("RELAY FAILURE....REACHED: {failure:?}"));

                    listener.on_recv(ActivityListenerOutcome {
                        data: RustTypeActivitySubscriberChannel::Terminated,
                    });
                }
            }
        })
        .await;

        Ok(())

        // send loop (persistent)
        // loop {
        //     let payload = bitcode::encode(&QuicProtocolOp::Hello);

        // let len = (payload.len() as u32).to_be_bytes();

        // send_outer
        //     .write()
        //     .await
        //     .write_all(&len)
        //     .await
        //     .map_err(|error| {
        //         let error: QuicTransmissionError = error.into();

        //         error
        //     })?;
        // send_outer
        //     .write()
        //     .await
        //     .write_all(&payload)
        //     .await
        //     .map_err(|error| {
        //         let error: QuicTransmissionError = error.into();

        //         error
        //     })?;

        // Timer::after(Duration::from_secs(1)).await;

        // if *counter.read().await == 10 {
        //     // only now close stream
        //     send.finish().map_err(|error| {
        //         let error: QuicTransmissionError = error.into();

        //         error
        //     })?;

        //     break;
        // }

        // *counter.write().await += 1;
        // }
    }
}

#[uniffi::export]
pub async fn rust_fn_get_activity(
    parsed: RustTypeParsedActivityDeeplink,
    offset: i32,
) -> Result<Option<RustTypeActivityMetadata>, RustFfiError> {
    let mut identifier: Byte28Array = [0u8; 28];
    faster_hex::hex_decode(parsed.identifier_hex.as_bytes(), &mut identifier)
        .or(Err(RustFfiError::InvalidActivityDeeplink))?;

    let payload = QuicProtocolOp::GetActivityMetadata(ActivityStoreKey(identifier));

    crate::ClientUtils::log_to_logcat(&format!("TARGET DOMAIN QUIC: {}", &parsed.domain));

    QuicClient::connect::<Option<ActivityMetadata>>(&parsed.domain, &payload)
        .await?
        .map(|activity| {
            let outcome: Result<RustTypeActivityMetadata, RustFfiError> =
                (offset, activity, parsed.domain).try_into();

            outcome
        })
        .transpose()
}

#[uniffi::export]
pub fn rust_fn_parse_activity_deeplink(
    activity_data: String,
) -> Result<RustTypeParsedActivityDeeplink, RustFfiError> {
    let [domain, identifier_hex] = activity_data
        .split(":")
        .collect::<Vec<&str>>()
        .try_into()
        .or(Err(RustFfiError::InvalidActivityDeeplink))?;

    let mut identifier: Byte28Array = [0u8; 28];
    faster_hex::hex_decode(identifier_hex.as_bytes(), &mut identifier)
        .or(Err(RustFfiError::InvalidActivityDeeplink))?;

    crate::ClientUtils::log_to_logcat(&format!("PARSED DOMAIN SCANNED QUIC: {}", &domain));

    Ok(RustTypeParsedActivityDeeplink {
        domain: domain.to_string(),
        identifier_hex: identifier_hex.to_string(),
    })
}

#[derive(Debug, uniffi::Record)]
pub struct RustTypeParsedActivityDeeplink {
    pub domain: String,
    pub identifier_hex: String,
}

#[uniffi::export]
pub async fn rust_fn_participate_in_activity(
    activity_data: RustTypeActivityMetadata,
) -> Result<String, RustFfiError> {
    let domain_or_ip = activity_data.domain_or_ip.clone();

    let storage = crate::app_storage()?;

    let mut org_info = storage
        .get_org_info(&activity_data.domain_or_ip)
        .await?
        .ok_or(RustFfiError::OrgNotFound)?;

    let already_exists = org_info.activities.get(&activity_data.activity_id);

    if already_exists.is_none() {
        org_info.active.replace(activity_data.activity_id.clone());
        org_info
            .activities
            .insert(activity_data.activity_id, ActivityInfo::default());
    }

    crate::ClientUtils::log_to_logcat(&format!("Org Info PARTICIPANTE: {org_info:?}"));

    storage.set_org_info(&domain_or_ip, org_info).await?;

    Ok(domain_or_ip)
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum RustTypeActivitySubscriberChannel {
    Ack,
    NewSubscriber,
    Terminated,
}

#[uniffi::export]
impl RustTypeActivitySubscriberChannel {
    pub fn to_ui_message(&self) -> String {
        match self {
            Self::Ack => ActivitySubscriberChannel::Ack.to_ui_str().to_string(),
            Self::NewSubscriber => ActivitySubscriberChannel::NewSubscriber
                .to_ui_str()
                .to_string(),
            Self::Terminated => "Connection terminated by relay".to_string(),
        }
    }
}

impl From<ActivitySubscriberChannel> for RustTypeActivitySubscriberChannel {
    fn from(value: ActivitySubscriberChannel) -> Self {
        match value {
            ActivitySubscriberChannel::Ack => Self::Ack,
            ActivitySubscriberChannel::NewSubscriber => Self::NewSubscriber,
        }
    }
}
