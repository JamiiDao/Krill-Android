use std::sync::{Arc, LazyLock};

use async_lock::RwLock;
use krill_common::{
    ActivityMetadata, ActivityState, ActivityStoreKey, Byte28Array, FrostParticipateMessage,
    FrostRound2ParticipantEncryptedPayload, QuicDkgRelayResponse, QuicProtocolOp,
    QuicTransmissionEncoder, QuicTransmissionError, QuicTransmissionResult, MAX_LEN_10_MIB,
};
use smol::channel::{bounded, Sender};

use crate::{
    AppStorage, ClientUtils, DkgRound2Payload, FinalizeDkgOp, FrostParticipateMessageWrapper,
    QuicClient, RustFfiError, RustFfiResult, RustTypeActivityMetadata,
};

pub(crate) static SCANNED_ACTIVITY: LazyLock<async_dup::Arc<RwLock<ActivityMetadata>>> =
    LazyLock::new(|| async_dup::Arc::new(RwLock::new(ActivityMetadata::default())));

async fn initial_payload(domain_or_ip: &str, activity_id: &str) -> RustFfiResult<Vec<u8>> {
    let storage = crate::app_storage()?;

    let stored_org_info = storage
        .get_org_info(domain_or_ip)
        .await?
        .ok_or(RustFfiError::OrgNotFound)?;

    let activity =
        if let Some(activity_found) = stored_org_info.activities.get(activity_id).cloned() {
            activity_found
        } else {
            let metadata = SCANNED_ACTIVITY.read().await.clone();

            FrostParticipateMessageWrapper::set(domain_or_ip, metadata).await?
        };
    let activity_store_hex = activity.metadata.as_hex();

    match activity.metadata.state {
        ActivityState::DkgRound1 => {
            let participante_payload: FrostParticipateMessage = activity.into();

            Ok(QuicProtocolOp::Participate(Box::new(participante_payload)).encode())
        }
        ActivityState::DkgRound2 => {
            Ok(DkgRound2Payload::set_if_exists(domain_or_ip, activity_store_hex).await?)
        }
        _ => Ok(bitcode::encode(&QuicProtocolOp::Hello)),
    }
}

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
        timezone: i32,
    ) -> Result<(), RustFfiError> {
        let activity_id = SCANNED_ACTIVITY.read().await.as_hex();
        let encoded_op = initial_payload(&domain_or_ip, &activity_id).await?;

        let (mut send_stream, recv_stream) = QuicClient::setup_connect(&domain_or_ip)
            .await?
            .open_bi()
            .await
            .map_err(|error| {
                let error: QuicTransmissionError = error.into();

                error
            })?;

        let len = (encoded_op.len() as u32).to_be_bytes();

        send_stream.write_all(&len).await.map_err(|error| {
            let error: QuicTransmissionError = error.into();

            error
        })?;
        send_stream.write_all(&encoded_op).await.map_err(|error| {
            let error: QuicTransmissionError = error.into();

            error
        })?;

        ClientUtils::log_to_logcat("SENT TO RELAY....REACHED");

        Ok(activate_listener(listener, send_stream, recv_stream).await?)
    }
}

async fn activate_listener(
    listener: Arc<dyn ActivityListener>,
    mut send_stream: quinn::SendStream,
    mut recv_stream: quinn::RecvStream,
) -> QuicTransmissionResult<()> {
    let (sender, receiver) = bounded::<NextChannelOp>(32);

    let listener_inner = listener.clone();

    let sender_cloned = sender.clone();

    smol::spawn(async move {
        loop {
            // read length prefix
            let mut len_buf = [0u8; 4];

            if let Err(error) = recv_stream.read_exact(&mut len_buf).await {
                listener_inner.on_recv(ActivityListenerOutcome {
                    data: RustTypeActivitySubscriberChannel::Terminated(error.to_string()),
                });

                break; // server closed stream
            }

            let len = u32::from_be_bytes(len_buf) as usize;

            let mut data = vec![0u8; len];

            if let Err(error) = recv_stream.read_exact(&mut data).await {
                ClientUtils::log_to_logcat("READ EXACT ERR REACHED");

                listener_inner.on_recv(ActivityListenerOutcome {
                    data: RustTypeActivitySubscriberChannel::Terminated(error.to_string()),
                });

                break;
            }

            if data.len() > MAX_LEN_10_MIB {
                ClientUtils::log_to_logcat("DATA CAPACITY EXCEEDED");

                listener_inner.on_recv(ActivityListenerOutcome {
                    data: RustTypeActivitySubscriberChannel::Terminated(
                        "DATA CAPACITY EXCEEDED".to_string(),
                    ),
                });

                break;
            }

            if let Err(error) = handle_response(listener_inner.clone(), sender.clone(), data).await
            {
                ClientUtils::log_to_logcat(&format!("Relay response error: {error:?}"));

                listener_inner.on_recv(ActivityListenerOutcome {
                    data: RustTypeActivitySubscriberChannel::Terminated(error.to_string()),
                });

                break;
            }
        }
    })
    .detach();

    // send loop (persistent)
    while let Ok(message) = receiver.recv().await {
        ClientUtils::log_to_logcat(&format!("Received Message via channel {message:?}"));

        let payload: Vec<u8> = match message {
            NextChannelOp::PerformDkgRound2 {
                sld_tld,
                activity_id,
            } => match DkgRound2Payload::set_if_exists(&sld_tld, activity_id).await {
                Ok(value) => value,
                Err(error) => {
                    listener.on_recv(ActivityListenerOutcome {
                        data: RustTypeActivitySubscriberChannel::Terminated(error.to_string()),
                    });

                    break;
                }
            },
            NextChannelOp::FetchRound2Packages {
                domain_or_ip,
                activity_id,
            } => {
                match FinalizeDkgOp::fetch_round2(
                    sender_cloned.clone(),
                    listener.clone(),
                    domain_or_ip,
                    activity_id,
                )
                .await
                {
                    Ok(value) => value,
                    Err(error) => {
                        listener.on_recv(ActivityListenerOutcome {
                            data: RustTypeActivitySubscriberChannel::Terminated(error.to_string()),
                        });

                        break;
                    }
                }
            }
            NextChannelOp::FinalizeDkg {
                sld_tld,
                activity_id,
            } => {
                match FinalizeDkgOp::finalize(
                    sender_cloned.clone(),
                    listener.clone(),
                    sld_tld,
                    activity_id,
                )
                .await
                {
                    Ok(value) => value,
                    Err(error) => {
                        listener.on_recv(ActivityListenerOutcome {
                            data: RustTypeActivitySubscriberChannel::Terminated(error.to_string()),
                        });

                        break;
                    }
                }
            }
        };

        let len = (payload.len() as u32).to_be_bytes();

        send_stream.write_all(&len).await?;
        send_stream.write_all(&payload).await?;
    }

    send_stream.finish()?;

    Ok(())
}

async fn handle_response(
    ui_listener: Arc<dyn ActivityListener>,
    sender: Sender<NextChannelOp>,
    data: Vec<u8>,
) -> RustFfiResult<()> {
    let storage = crate::app_storage()?;

    let op_flag = if let Some(op_flag) = data.first() {
        *op_flag
    } else {
        return Err(QuicTransmissionError::InvalidResponsePayload.into());
    };

    let decoded: QuicDkgRelayResponse = if op_flag == 0 {
        return Err(QuicTransmissionEncoder::decode_failure(&data[1..]).into());
    } else if op_flag == 1 {
        match QuicTransmissionEncoder::decode_success::<QuicDkgRelayResponse>(&data[1..]) {
            Err(error) => return Err(error.into()),
            Ok(value) => value,
        }
    } else {
        return Err(QuicTransmissionError::InvalidResponsePayload.into());
    };

    match decoded {
        QuicDkgRelayResponse::NoActiveActivity => {
            crate::ClientUtils::log_to_logcat("NO ACTIVE ACTIVITY");
        }
        QuicDkgRelayResponse::InvalidRequest => {
            crate::ClientUtils::log_to_logcat("INVALID REQUEST");
        }
        QuicDkgRelayResponse::Ack(mut received_ack) => {
            // TODO Add verification for ed25519 signing the data
            let received_ack_id = received_ack.metadata.as_hex();

            crate::ClientUtils::log_to_logcat(&format!("ACK RECEIVED {}", received_ack_id));

            let mut info = storage
                .get_all_orgs()
                .await?
                .first()
                .cloned()
                .ok_or(RustFfiError::OrgNotFound)?;

            crate::ClientUtils::log_to_logcat(&format!(
                "ORG FOUND ACTIVITIES {:?}",
                info.activities.keys().collect::<Vec<&String>>()
            ));

            let identity = info.identity.seed().to_string();

            let mut found_activity = info
                .activities
                .get(&received_ack.metadata.as_hex())
                .cloned()
                .ok_or(RustFfiError::ActivityNotFound)?;

            if let Some(position) = received_ack
                .round1
                .iter()
                .position(|message| message.participant == identity)
            {
                received_ack.round1.remove(position);
            }

            received_ack.round1.into_iter().for_each(|message| {
                crate::ClientUtils::log_to_logcat(&format!("ADDED DATA: {}", message.participant));
                found_activity.round1_participants.push(message);
            });

            let mut transition = false;

            if (found_activity.round1_participants.len() + 1) == found_activity.min_max.max as usize
            {
                found_activity.metadata.state = ActivityState::DkgRound2;
                crate::ClientUtils::log_to_logcat("TRANSITION TO ROUND2 ");
                transition = true;
            }

            info.activities
                .insert(found_activity.metadata.as_hex(), found_activity);
            let sld_tld = info.sld_tld.clone();

            storage.set_org_info(&sld_tld, info).await?;

            crate::ClientUtils::log_to_logcat(&format!("DECODED RESPONSE: {received_ack_id}",));

            ui_listener.on_recv(ActivityListenerOutcome {
                data: RustTypeActivitySubscriberChannel::Ack,
            });

            if transition {
                ui_listener.on_recv(ActivityListenerOutcome {
                    data: RustTypeActivitySubscriberChannel::DkgRound2,
                });

                if sender
                    .send(NextChannelOp::PerformDkgRound2 {
                        sld_tld,
                        activity_id: received_ack_id,
                    })
                    .await
                    .is_err()
                {
                    return Err(RustFfiError::DkgChannelError);
                }
            }
        }
        QuicDkgRelayResponse::DkgNewParticipant(participant_round1) => {
            crate::ClientUtils::log_to_logcat(&format!(
                "NEW PARTICIPANT: {}",
                participant_round1.participant
            ));

            let sld_tld = participant_round1.domain_or_ip.clone();
            let participant = participant_round1.participant.as_str();

            let added_participant = format!(
                "{}...{} has joined",
                &participant[..4],
                &participant[participant.len() - 8..]
            );

            let mut org_info = storage
                .get_all_orgs()
                .await?
                .first()
                .cloned()
                .ok_or(RustFfiError::OrgNotFound)?;

            if org_info.identity.seed() == participant_round1.participant {
                return Ok(());
            }

            let active_activity = org_info
                .active
                .clone()
                .ok_or(RustFfiError::NoActiveActivity)?;

            let mut activity = org_info
                .activities
                .get(&active_activity)
                .cloned()
                .ok_or(RustFfiError::ActivityNotFound)?;
            let activity_id = activity.metadata.as_hex();

            activity.round1_participants.push(participant_round1);
            activity.round1_participants.dedup();

            let mut transition = false;

            if activity.metadata.threshold.max as usize == (activity.round1_participants.len() + 1)
            {
                activity.metadata.state = ActivityState::DkgRound2;
                crate::ClientUtils::log_to_logcat("TRANSITIONED TO ROUND2");

                transition = true;
            }

            org_info
                .activities
                .insert(activity.metadata.as_hex(), activity);

            storage
                .set_org_info(&org_info.sld_tld.clone(), org_info)
                .await?;

            ui_listener.on_recv(ActivityListenerOutcome {
                data: RustTypeActivitySubscriberChannel::NewSubscriber(added_participant),
            });

            if transition {
                ui_listener.on_recv(ActivityListenerOutcome {
                    data: RustTypeActivitySubscriberChannel::DkgRound2,
                });

                if sender
                    .send(NextChannelOp::PerformDkgRound2 {
                        activity_id,
                        sld_tld,
                    })
                    .await
                    .is_err()
                {
                    return Err(RustFfiError::DkgChannelError);
                }
            }
        }

        QuicDkgRelayResponse::DkgRound2Ack => {
            crate::ClientUtils::log_to_logcat("DKG ROUND2 ACK");
            ui_listener.on_recv(ActivityListenerOutcome {
                data: RustTypeActivitySubscriberChannel::DkgRound2,
            });
        }

        QuicDkgRelayResponse::DkgRound2PublicPackages {
            domain_or_ip,
            activity_id,
            round2_packages,
        } => {
            let round2_len = round2_packages.len();
            crate::ClientUtils::log_to_logcat(&format!(
                "Received DkgRound2PublicPackages from relay: {activity_id}-round2 len {}",
                round2_len
            ));

            let mut org_info = storage
                .get_org_info(&domain_or_ip)
                .await?
                .ok_or(RustFfiError::OrgNotFound)?;
            let mut activity = org_info
                .activities
                .get(&activity_id)
                .cloned()
                .ok_or(RustFfiError::OrgNotFound)?;

            if activity.metadata.state != ActivityState::DkgRound2 {
                return Err(RustFfiError::InvalidActivityState);
            }

            for package in round2_packages {
                if package.sender_seed == org_info.identity {
                    ClientUtils::log_to_logcat("Invalid Sender. Received My own Round2 package");
                }

                activity
                    .round2_received_public
                    .insert(package.sender_seed.seed().to_string(), package);
            }

            let transition = (activity.round2_received_public.values().len() + 1)
                >= activity.metadata.threshold.min as usize;

            org_info
                .activities
                .insert(activity.metadata.as_hex(), activity);

            storage.set_org_info(&domain_or_ip, org_info).await?;

            if transition
                && sender
                    .send(NextChannelOp::FinalizeDkg {
                        sld_tld: domain_or_ip,
                        activity_id,
                    })
                    .await
                    .is_err()
            {
                return Err(RustFfiError::DkgChannelError);
            }
        }

        QuicDkgRelayResponse::FinalizeDkg {
            domain_or_ip,
            activity_id,
            round2_packages,
        } => {
            let round2_len = round2_packages.len();

            set_recieved_dkg_round2_packages(storage, &domain_or_ip, &activity_id, round2_packages)
                .await?;

            crate::ClientUtils::log_to_logcat(&format!(
                "Finalize DKG FINALIZED: {activity_id}-round2 len {}",
                round2_len
            ));
            ui_listener.on_recv(ActivityListenerOutcome {
                data: RustTypeActivitySubscriberChannel::FinalizeDkg,
            });

            if sender
                .send(NextChannelOp::FinalizeDkg {
                    sld_tld: domain_or_ip,
                    activity_id,
                })
                .await
                .is_err()
            {
                return Err(RustFfiError::DkgChannelError);
            }
        }

        QuicDkgRelayResponse::DkgRound2AckAndFinalize {
            domain_or_ip,
            activity_id,
            round2_packages,
        } => {
            let round2_len = round2_packages.len();

            set_recieved_dkg_round2_packages(storage, &domain_or_ip, &activity_id, round2_packages)
                .await?;

            crate::ClientUtils::log_to_logcat(&format!(
                "ACK and Finalize DKG FINALIZED: {activity_id}-round2 len {round2_len}"
            ));

            ui_listener.on_recv(ActivityListenerOutcome {
                data: RustTypeActivitySubscriberChannel::FinalizeDkg,
            });

            if sender
                .send(NextChannelOp::FinalizeDkg {
                    activity_id,
                    sld_tld: domain_or_ip,
                })
                .await
                .is_err()
            {
                return Err(RustFfiError::DkgChannelError);
            }
        }

        QuicDkgRelayResponse::DkgFinalized => {
            crate::ClientUtils::log_to_logcat("NEW DKG FINALIZED");

            ui_listener.on_recv(ActivityListenerOutcome {
                data: RustTypeActivitySubscriberChannel::DkgFinalized,
            });
        }
    }

    Ok(())
}

async fn set_recieved_dkg_round2_packages(
    storage: &'static AppStorage,
    domain_or_ip: &str,
    activity_id: &str,
    round2_packages: Vec<FrostRound2ParticipantEncryptedPayload>,
) -> RustFfiResult<()> {
    let mut org_info = storage
        .get_org_info(domain_or_ip)
        .await?
        .ok_or(RustFfiError::OrgNotFound)?;
    let mut activity = org_info
        .activities
        .get(activity_id)
        .cloned()
        .ok_or(RustFfiError::OrgNotFound)?;

    if activity.metadata.state != ActivityState::DkgRound2 {
        return Err(RustFfiError::InvalidActivityState);
    }

    for package in round2_packages {
        if package.sender_seed == org_info.identity {
            ClientUtils::log_to_logcat("Invalid Sender. Received My own Round2 package");
        }

        activity
            .round2_received_public
            .insert(package.sender_seed.seed().to_string(), package);
    }

    org_info
        .activities
        .insert(activity.metadata.as_hex(), activity);

    storage.set_org_info(domain_or_ip, org_info).await
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

    if let Some(activity_metadata) =
        QuicClient::connect::<Option<ActivityMetadata>>(&parsed.domain, &payload).await?
    {
        *SCANNED_ACTIVITY.write().await = activity_metadata.clone();

        let outcome: Result<RustTypeActivityMetadata, RustFfiError> =
            (offset, activity_metadata, parsed.domain).try_into();

        Ok(Some(outcome?))
    } else {
        Ok(None)
    }
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

    Ok(domain_or_ip)
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum RustTypeActivitySubscriberChannel {
    Processing,
    Ack,
    NewSubscriber(String),
    DkgRound2,
    FinalizeDkg,
    DkgFinalized,
    NoActive,
    Terminated(String),
}

#[uniffi::export]
impl RustTypeActivitySubscriberChannel {
    pub fn to_ui_message(&self) -> String {
        match self {
            Self::Processing => "Processing".to_string(),
            Self::Ack => "Round 1 Key Agreement Started...".to_string(),
            Self::DkgRound2 => "Round 2 Key Agreement Started...".to_string(),
            Self::FinalizeDkg => "Consolidating the group key".to_string(),
            Self::DkgFinalized => "Participants have agreed on a Ed25519 public key".to_string(),
            Self::NewSubscriber(id) => format!("{id} joined the activity"),
            Self::NoActive => "No active activity".to_string(),
            Self::Terminated(value) => format!("Terminated: {value}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NextChannelOp {
    PerformDkgRound2 {
        sld_tld: String,
        activity_id: String,
    },
    FetchRound2Packages {
        domain_or_ip: String,
        activity_id: String,
    },
    FinalizeDkg {
        sld_tld: String,
        activity_id: String,
    },
}
