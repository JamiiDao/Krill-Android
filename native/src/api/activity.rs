use async_io::Timer;
use async_lock::RwLock;
use krill_common::{
    ActivityMetadata, ActivityStoreKey, Byte48Array, QuicProtocolOp, QuicTransmissionEncoder,
    QuicTransmissionError,
};

use crate::{
    ClientUtils, FrostParticipateMessageWrapper, QuicClient, RustFfiError, RustTypeActivityMetadata,
};

use std::{sync::Arc, time::Duration};

#[uniffi::export(with_foreign)]
pub trait QuicBidirectionalListener: Send + Sync {
    fn on_tick(&self, value: String);
}

#[derive(uniffi::Object)]
pub struct QuicBidirectionalEmitter;

#[uniffi::export]
impl QuicBidirectionalEmitter {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }

    pub async fn start(
        &self,
        listener: Arc<dyn QuicBidirectionalListener>,
        domain_or_ip: String,
        activity_id: String,
    ) -> Result<(), RustFfiError> {
        let counter = async_dup::Arc::new(RwLock::new(0u32));

        let mut store_key: Byte48Array = [0u8; 48];
        faster_hex::hex_decode(activity_id.as_bytes(), &mut store_key)
            .or(Err(RustFfiError::InvalidActivityId))?;

        let (mut send, mut recv) = QuicClient::setup_connect(&domain_or_ip)
            .await?
            .open_bi()
            .await
            .map_err(|error| {
                let error: QuicTransmissionError = error.into();

                error
            })?;

        // FrostParticipateMessageWrapper::init(
        //     &activity_data.domain_or_ip,
        //     ActivityStoreKey(store_key),
        //     activity_data.threshold.into(),
        // )
        // .await?;

        // spawn receiver task (important for persistence)

        let cloned_counter = counter.clone();
        smol::spawn(async move {
            loop {
                // read length prefix
                let mut len_buf = [0u8; 4];

                if recv.read_exact(&mut len_buf).await.is_err() {
                    listener.on_tick(format!("Terminated"));

                    break; // server closed stream
                }

                let len = u32::from_be_bytes(len_buf) as usize;

                let mut data = vec![0u8; len];

                if recv.read_exact(&mut data).await.is_err() {
                    break;
                }

                let response = std::mem::take(&mut data);

                if response.is_empty() {
                    listener.on_tick(format!("Received empty response the relay"));
                }

                if response[0] == 0 {
                    match QuicTransmissionEncoder::decode_success::<QuicProtocolOp>(&response[1..])
                    {
                        Ok(value) => {
                            listener.on_tick(format!(
                                "Received from the relay:{}-{value:?}",
                                *cloned_counter.read().await
                            ));
                        }
                        Err(error) => {
                            listener.on_tick(format!("Received invalid data-{error}"));
                        }
                    }
                } else {
                    let failure = QuicTransmissionEncoder::decode_failure(&response[1..]);

                    listener.on_tick(format!("Received from the relay:{failure}"));
                }
            }
        })
        .detach();

        // send loop (persistent)
        loop {
            let payload = bitcode::encode(&QuicProtocolOp::Hello);

            let len = (payload.len() as u32).to_be_bytes();

            send.write_all(&len).await.map_err(|error| {
                let error: QuicTransmissionError = error.into();

                error
            })?;
            send.write_all(&payload).await.map_err(|error| {
                let error: QuicTransmissionError = error.into();

                error
            })?;

            Timer::after(Duration::from_secs(1)).await;

            if *counter.read().await == 10 {
                // only now close stream
                send.finish().map_err(|error| {
                    let error: QuicTransmissionError = error.into();

                    error
                })?;

                break;
            }

            *counter.write().await += 1;

            ClientUtils::log_to_logcat(&format!("{}", *counter.read().await));
        }

        Ok(())
    }
}

#[uniffi::export]
pub async fn rust_fn_get_activity(
    parsed: RustTypeParsedActivityDeeplink,
    offset: i32,
) -> Result<Option<RustTypeActivityMetadata>, RustFfiError> {
    let mut identifier: Byte48Array = [0u8; 48];
    faster_hex::hex_decode(parsed.identifier_hex.as_bytes(), &mut identifier)
        .or(Err(RustFfiError::InvalidActivityDeeplink))?;

    let payload = QuicProtocolOp::GetActivityMetadata(ActivityStoreKey(identifier));

    crate::ClientUtils::log_to_logcat(&format!("TARGET DOMAIN QUIC: {}", &parsed.domain));

    Ok(
        QuicClient::connect::<Option<ActivityMetadata>>(&parsed.domain, &payload)
            .await?
            .map(|activity| (offset, activity, parsed.domain).into()),
    )
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

    let mut identifier: Byte48Array = [0u8; 48];
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

    let mut store_key: Byte48Array = [0u8; 48];
    faster_hex::hex_decode(activity_data.activity_id.as_bytes(), &mut store_key)
        .or(Err(RustFfiError::InvalidActivityId))?;

    FrostParticipateMessageWrapper::init(
        &activity_data.domain_or_ip,
        ActivityStoreKey(store_key),
        activity_data.threshold.into(),
    )
    .await?;

    Ok(domain_or_ip)
}
