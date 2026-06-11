use std::collections::{BTreeMap, HashMap};

use bitcode::{Decode, Encode};
use krill_common::{
    finalized::{FrostKeyPackageBytes, FrostPublicKeyPackage},
    round1, round2, ActivityMetadata, ActivityState, ActivityStoreKey, AsymmetricKeypairBytes,
    AsymmetricSignatureBytes, AsymmetricVerifyingKeyBytes, Blake3HashBytes,
    EphemeralClientDeviceKeypair, EphemeralClientDeviceVerifyingKey, FrostCredentialSeed,
    FrostOpsError, FrostOpsResult, FrostParticipateMessage, FrostRound2ParticipantEncryptedPayload,
    MinMaxParticipants, QuicProtocolOp, Tai64NTimestamp,
};
use smol::channel::Sender;
use zeroize::Zeroize;

use crate::{
    api::{
        ActivityListener, ActivityListenerOutcome, NextChannelOp, RustTypeActivitySubscriberChannel,
    },
    AppStorage, ClientUtils, FrostEd25519, RustFfiError, RustFfiResult, StoredOrgInfo,
};

pub struct FrostParticipateMessageWrapper(pub(crate) FrostParticipateMessage);

impl FrostParticipateMessageWrapper {
    pub(crate) async fn set(
        domain_or_ip: &str,
        activity_metadata: ActivityMetadata,
    ) -> RustFfiResult<FrostParticipantInternalData> {
        let store_key = activity_metadata.store_key();

        let store = crate::app_storage()?;

        let mut stored_org_info = if let Some(org_info) = store.get_org_info(domain_or_ip).await? {
            org_info
        } else {
            crate::ClientUtils::log_to_logcat("ORG NOT FOUND, CANNOT PARTICIPATE!");

            return Err(RustFfiError::OrgNotFound);
        };

        let credential = &stored_org_info.identity;
        let identity = credential.seed().to_string();
        crate::ClientUtils::log_to_logcat("-> ORG & CREDENTIAL FOUND!");

        let min_max = activity_metadata.threshold;
        let store_key_hex = activity_metadata.as_hex();

        if stored_org_info
            .activities
            .get_mut(&activity_metadata.as_hex())
            .is_some()
        {
            return Err(RustFfiError::ActivityAlreadyExists);
        };

        let identifier = credential.frost_identifier::<FrostEd25519>()?;

        let (secret, public) =
            frost_core::keys::dkg::part1(identifier, min_max.max, min_max.min, rand::rngs::OsRng)
                .map_err(|error| RustFfiError::Frost(error.to_string()))?;
        crate::ClientUtils::log_to_logcat("-> round1 secret/public generated!");

        let round1_secret = round1::Round1SecretBytes::new::<FrostEd25519>(secret)?;
        crate::ClientUtils::log_to_logcat("-> round1 secret parsed!");
        let round1_public = round1::Round1PackageBytes::parse::<FrostEd25519>(&public)?;
        crate::ClientUtils::log_to_logcat("-> round1 public parsed!");

        let ecdk = AsymmetricKeypairBytes::new()?;
        crate::ClientUtils::log_to_logcat("ECDK GENERATED!");

        let new_internal = FrostParticipantInternalData {
            metadata: activity_metadata,
            identity: identity.clone(),
            identity_seed: credential.clone(),
            domain_or_ip: domain_or_ip.to_string(),
            timestamp: store_key.timestamp()?,
            activity_id: store_key,
            min_max,
            ecdk,
            hpke_kp: EphemeralClientDeviceKeypair::new()?,
            round1_secret,
            round1_public,
            round1_participants: Vec::default(),
            round2_secret: Option::default(),
            round2_public: HashMap::default(),
            key_package: Option::default(),
            public_package: Option::default(),
            round2_received_public: HashMap::default(),
        };

        stored_org_info
            .activities
            .insert(store_key_hex.clone(), new_internal.clone());
        stored_org_info.active.replace(store_key_hex);

        store.set_org_info(domain_or_ip, stored_org_info).await?;

        let new_org = store
            .get_org_info(domain_or_ip)
            .await?
            .ok_or(RustFfiError::OrgNotFound)?;

        crate::ClientUtils::log_to_logcat(&format!(
            "ADDED ID {:?}",
            new_org.activities.keys().collect::<Vec<&String>>()
        ));

        Ok(new_internal)
    }

    pub async fn get(
        domain_or_ip: &str,
        activity_metadata: &ActivityMetadata,
    ) -> RustFfiResult<Self> {
        let store = crate::app_storage()?;

        let mut stored_org_info = if let Some(org_info) = store.get_org_info(domain_or_ip).await? {
            org_info
        } else {
            crate::ClientUtils::log_to_logcat("ORG NOT FOUND, CANNOT PARTICIPATE!");

            return Err(RustFfiError::OrgNotFound);
        };

        let participant_local_info = stored_org_info
            .activities
            .get_mut(&activity_metadata.as_hex())
            .cloned()
            .ok_or(RustFfiError::ActivityNotFound)?;

        let ecdk = participant_local_info.ecdk.clone();

        let mut wrapped = Self(participant_local_info.into());
        wrapped.compute_signature(ecdk)?;

        Ok(wrapped)
    }

    /// Allows the target participants to ensure that the entire message was meant
    /// for the organization with the intended timestamp.
    /// ### Packing
    /// domain_or_ip.as_bytes || timestamp 12 bytes || activity_id 48 bytes || participant.as_bytes
    /// || min_max.min.to_le_bytes || min_max.max.to_le_bytes || Round1PackageBytes dkg encoded
    pub fn compute_binding_hash(&self) -> Blake3HashBytes {
        let mut binding_hash = blake3::Hasher::new();
        binding_hash
            .update(self.0.domain_or_ip.as_bytes())
            .update(self.0.timestamp.as_slice())
            .update(self.0.participant.as_bytes())
            .update(&self.0.min_max.min.to_le_bytes())
            .update(&self.0.min_max.max.to_le_bytes())
            .update(&self.0.round1_dkg.encode());

        Blake3HashBytes::pre_hashed(binding_hash.finalize())
    }

    pub fn compute_signature(&mut self, akp: AsymmetricKeypairBytes) -> RustFfiResult<&mut Self> {
        let binding_hash = self.compute_binding_hash();

        let (edvk, ecds) = akp.sign_and_return_encodable_and_verifying_key(binding_hash)?;

        self.0.binding_hash = binding_hash;
        self.0.ecdvk = edvk;
        self.0.ecds = ecds;

        Ok(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub(crate) struct FrostParticipantInternalData {
    pub(crate) metadata: ActivityMetadata,
    pub(crate) identity: String,
    pub(crate) identity_seed: FrostCredentialSeed,
    pub(crate) domain_or_ip: String,
    pub(crate) timestamp: Tai64NTimestamp,
    pub(crate) activity_id: ActivityStoreKey,
    pub(crate) min_max: MinMaxParticipants,
    pub(crate) ecdk: AsymmetricKeypairBytes,
    pub(crate) hpke_kp: EphemeralClientDeviceKeypair,
    pub(crate) round1_secret: round1::Round1SecretBytes,
    pub(crate) round1_public: round1::Round1PackageBytes,
    pub(crate) round2_secret: Option<round2::Round2SecretBytes>,
    pub(crate) key_package: Option<FrostKeyPackageBytes>,
    pub(crate) public_package: Option<FrostPublicKeyPackage>,
    pub(crate) round2_public: HashMap<String, FrostRound2ParticipantEncryptedPayload>,
    pub(crate) round1_participants: Vec<FrostParticipateMessage>,
    pub(crate) round2_received_public: HashMap<String, FrostRound2ParticipantEncryptedPayload>,
}

impl From<FrostParticipantInternalData> for FrostParticipateMessage {
    fn from(value: FrostParticipantInternalData) -> Self {
        Self {
            domain_or_ip: value.domain_or_ip,
            timestamp: value.timestamp,
            activity_id: value.activity_id,
            participant: value.identity,
            min_max: value.min_max,
            round1_dkg: value.round1_public,
            binding_hash: Blake3HashBytes::default(),
            ecdvk: value.ecdk.verifying_key_encodable(),
            hpke_vk: value.hpke_kp.verifying_key_encodable(),
            participant_seed: value.identity_seed,
            ecds: AsymmetricSignatureBytes::default(),
        }
    }
}

pub struct DkgRound2Payload;

impl DkgRound2Payload {
    pub(crate) async fn set_if_exists(
        sld_tld: &str,
        activity_store_hex: String,
    ) -> RustFfiResult<Vec<u8>> {
        let storage = crate::app_storage()?;

        let org_info = storage
            .get_org_info(sld_tld)
            .await?
            .ok_or(RustFfiError::OrgNotFound)?;

        let activity = org_info
            .activities
            .get(&activity_store_hex)
            .ok_or(RustFfiError::ActivityNotFound)?
            .clone();

        let store_key = activity.metadata.store_key();

        let round2_public = if activity.round2_secret.is_some() {
            activity.round2_public
        } else {
            Self::new_data(storage, sld_tld, activity_store_hex, org_info, activity).await?
        };

        let op = QuicProtocolOp::ParticipantRound2 {
            activity_id: store_key,
            payload: round2_public.values().cloned().collect(),
        };

        Ok(op.encode())
    }

    pub(crate) async fn new_data(
        storage: &'static AppStorage,
        sld_tld: &str,
        activity_store_hex: String,
        mut org_info: StoredOrgInfo,
        mut activity: FrostParticipantInternalData,
    ) -> RustFfiResult<HashMap<String, FrostRound2ParticipantEncryptedPayload>> {
        let my_hpke_kp: EphemeralClientDeviceKeypair = activity.hpke_kp.clone();
        let my_frost_credential: FrostCredentialSeed = activity.identity_seed.clone();
        let my_ecdk: AsymmetricKeypairBytes = activity.ecdk.clone();

        let mut prepared_round1_packages = BTreeMap::<
            frost_core::Identifier<FrostEd25519>,
            frost_core::keys::dkg::round1::Package<FrostEd25519>,
        >::default();

        let mut identifier_mapping = HashMap::<
            frost_core::Identifier<FrostEd25519>,
            (FrostCredentialSeed, EphemeralClientDeviceVerifyingKey),
        >::default();

        activity
            .round1_participants
            .iter()
            .try_for_each(|package| {
                let identifier = package
                    .participant_seed
                    .frost_identifier::<FrostEd25519>()?;

                identifier_mapping.insert(
                    identifier,
                    (package.participant_seed.clone(), package.hpke_vk.clone()),
                );

                let round1_package_inner = package.round1_dkg.to_frost_package::<FrostEd25519>()?;

                prepared_round1_packages.insert(identifier, round1_package_inner);

                Ok::<_, FrostOpsError>(())
            })?;
        let store_key = activity.metadata.store_key();

        let round1_secret = activity.round1_secret.deserialize::<FrostEd25519>()?;

        let (mut part2_secret, part2_packages) =
            frost_core::keys::dkg::part2::<FrostEd25519>(round1_secret, &prepared_round1_packages)
                .map_err(|error| {
                    let error: frost_dkg_types::FrostOpsError = error.into();

                    error
                })?;

        let part2_secret_bytes = round2::Round2SecretBytes::serialize(&part2_secret)?;
        part2_secret.zeroize();
        activity.round2_secret.replace(part2_secret_bytes);

        part2_packages
            .into_iter()
            .try_for_each(|(identifier, package)| {
                let (frost_credential, hpke_vk) = identifier_mapping
                    .remove(&identifier)
                    .ok_or(FrostOpsError::FrostCredentialNotSet)?;
                let credential_str = frost_credential.seed().to_string();

                let public_as_bytes = round2::Round2PackageBytes::parse(&package)?;

                let timestamp = Tai64NTimestamp::now();

                let hpke_vk_decoded = EphemeralClientDeviceVerifyingKey::from_bytes(hpke_vk);

                let round2_encrypted_public = my_hpke_kp
                    .clone()
                    .generate_he_outputs(public_as_bytes.encode(), &hpke_vk_decoded)?;

                let round2_enc_payload = Self::generate_ecds(
                    FrostRound2ParticipantEncryptedPayload {
                        store_key,
                        sender_seed: my_frost_credential.clone(),
                        recipient_seed: frost_credential,
                        round2_encrypted_public,
                        timestamp,
                        binding_hash: Blake3HashBytes::default(),
                        ecds: AsymmetricSignatureBytes::default(),
                    },
                    my_ecdk.clone(),
                )?;

                activity
                    .round2_public
                    .insert(credential_str, round2_enc_payload);

                Ok::<_, FrostOpsError>(())
            })?;

        let to_send = activity.round2_public.clone();

        org_info.activities.insert(activity_store_hex, activity);

        storage.set_org_info(sld_tld, org_info).await?;

        Ok(to_send)
    }

    pub fn compute_binding_hash(
        payload: &FrostRound2ParticipantEncryptedPayload,
    ) -> Blake3HashBytes {
        let mut hasher = blake3::Hasher::new();
        hasher
            .update(&payload.sender_seed.encode())
            .update(&payload.recipient_seed.encode())
            .update(&payload.round2_encrypted_public.ciphertext)
            .update(
                &payload
                    .round2_encrypted_public
                    .sender_static_verifying_key
                    .0,
            )
            .update(payload.timestamp.as_slice());

        Blake3HashBytes::pre_hashed(hasher.finalize())
    }

    pub fn generate_ecds(
        mut payload: FrostRound2ParticipantEncryptedPayload,
        my_eckp: AsymmetricKeypairBytes,
    ) -> FrostOpsResult<FrostRound2ParticipantEncryptedPayload> {
        payload.binding_hash = Self::compute_binding_hash(&payload);

        payload.ecds = my_eckp.sign_and_return_encodable(payload.binding_hash)?;

        Ok(payload)
    }

    pub fn verify_binding_hash(
        payload: &FrostRound2ParticipantEncryptedPayload,
        other_ecdvk: AsymmetricVerifyingKeyBytes,
    ) -> FrostOpsResult<bool> {
        if payload.binding_hash != Self::compute_binding_hash(payload) {
            return Err(FrostOpsError::BindingHashMismatch);
        }

        let to_ed25519 = other_ecdvk.from_bytes()?;
        let signature = payload.ecds.from_bytes();

        Ok(to_ed25519
            .verify_strict(payload.binding_hash.as_ref(), &signature)
            .is_ok())
    }
}

pub struct FinalizeDkgOp;

impl FinalizeDkgOp {
    pub async fn fetch_round2(
        sender: Sender<NextChannelOp>,
        listener: std::sync::Arc<dyn ActivityListener>,
        domain_or_ip: String,
        activity_id: String,
    ) -> RustFfiResult<Vec<u8>> {
        let storage = crate::app_storage()?;
        let mut org_info = storage
            .get_org_info(&domain_or_ip)
            .await?
            .ok_or(RustFfiError::OrgNotFound)?;
        let identity = org_info.identity.clone();

        let activity = org_info
            .activities
            .get(&activity_id)
            .cloned()
            .ok_or(RustFfiError::ActivityNotFound)?;

        let activity_store_key = activity.metadata.store_key();

        let op = QuicProtocolOp::FetchRound2 {
            participant: identity,
            activity_id: activity_store_key,
        };

        Ok(op.encode())
    }

    pub async fn finalize(
        sender: Sender<NextChannelOp>,
        listener: std::sync::Arc<dyn ActivityListener>,
        domain_or_ip: String,
        activity_id: String,
    ) -> RustFfiResult<Vec<u8>> {
        let storage = crate::app_storage()?;
        let mut org_info = storage
            .get_org_info(&domain_or_ip)
            .await?
            .ok_or(RustFfiError::OrgNotFound)?;
        let identity = org_info.identity.clone();

        let mut activity = org_info
            .activities
            .get(&activity_id)
            .cloned()
            .ok_or(RustFfiError::ActivityNotFound)?;

        let activity_store_key = activity.metadata.store_key();

        let mut round2_secret = activity
            .round2_secret
            .clone()
            .ok_or(RustFfiError::InvalidActivityState)?
            .deserialize::<FrostEd25519>()?;

        let my_hpke_kp = activity.hpke_kp.clone();

        let mut round1_received_packages = BTreeMap::<
            frost_core::Identifier<FrostEd25519>,
            frost_core::keys::dkg::round1::Package<FrostEd25519>,
        >::default();
        let mut round2_received_packages = BTreeMap::<
            frost_core::Identifier<FrostEd25519>,
            frost_core::keys::dkg::round2::Package<FrostEd25519>,
        >::default();

        activity.round1_participants.iter().try_for_each(|value| {
            let package = value.round1_dkg.to_frost_package::<FrostEd25519>()?;
            let identifier = value.participant_seed.frost_identifier::<FrostEd25519>()?;
            round1_received_packages.insert(identifier, package);

            Ok::<_, FrostOpsError>(())
        })?;

        if (activity.round2_received_public.values().len() + 1)
            < activity.metadata.threshold.min as usize
        {
            ClientUtils::log_to_logcat("DKG Part3 fetching round2 packages since len is 0");

            if sender
                .send(NextChannelOp::FetchRound2Packages {
                    domain_or_ip: domain_or_ip.clone(),
                    activity_id,
                })
                .await
                .is_err()
            {
                return Err(RustFfiError::DkgChannelError);
            }
        }

        for round2_participant_info in activity.round2_received_public.values() {
            let ecdvk = activity
                .round1_participants
                .iter()
                .find(|round1_participant_info| {
                    round1_participant_info.participant_seed == round2_participant_info.sender_seed
                })
                .ok_or(FrostOpsError::InvalidParticipant)?
                .ecdvk;

            ClientUtils::log_to_logcat("DKG Part3 collecting round2 packages. FOUND ecdvk");

            if !DkgRound2Payload::verify_binding_hash(round2_participant_info, ecdvk)? {
                ClientUtils::log_to_logcat("DKG Part3 collecting round2 packages. FOUND ecdvk");

                break;
            }

            let hpke = EphemeralClientDeviceKeypair::hpke();

            let (secret_key, _) = my_hpke_kp.clone().into_secret_key()?;
            ClientUtils::log_to_logcat("DKG Part3 my HPKE success");

            let sender_static_verifying_key = round2_participant_info
                .round2_encrypted_public
                .sender_static_verifying_key
                .clone()
                .from_bytes();
            ClientUtils::log_to_logcat("DKG Part3  sender static success");

            let encapsulated_key = round2_participant_info
                .round2_encrypted_public
                .sender_ephemeral_verifying_key
                .clone()
                .from_bytes();
            ClientUtils::log_to_logcat("DKG Part3  sender HE success");

            let mut receiver_ctx = hpke
                .setup_receiver(
                    encapsulated_key.as_slice(),
                    &secret_key,
                    EphemeralClientDeviceKeypair::INFO_BYTES,
                    None,
                    None,
                    Some(&sender_static_verifying_key), // <-- sender static public key
                )
                .map_err(|error| {
                    let error: FrostOpsError = error.into();

                    error
                })?;
            ClientUtils::log_to_logcat("DKG Part3  decoded HPKE success");

            let decoded_payload = receiver_ctx
                .open(
                    EphemeralClientDeviceKeypair::AAD_BYTES,
                    round2_participant_info.round2_encrypted_public.ciphertext(),
                )
                .map_err(|error| {
                    let error: FrostOpsError = error.into();

                    error
                })?;
            ClientUtils::log_to_logcat("DKG Part3  decoded round2 public success");

            let round2_decoded = round2::Round2PackageBytes::decode(&decoded_payload)?
                .to_frost_package::<FrostEd25519>()?;
            ClientUtils::log_to_logcat("DKG Part3 round2 to FROST success");

            let identifier = round2_participant_info
                .sender_seed
                .frost_identifier::<FrostEd25519>()?;
            ClientUtils::log_to_logcat("DKG Part3 round2 to FROST identifier success");

            round2_received_packages.insert(identifier, round2_decoded);
        }

        ClientUtils::log_to_logcat(&format!(
            "DKG Part3 round1 packages len:{:?}",
            round1_received_packages.len()
        ));

        ClientUtils::log_to_logcat(&format!(
            "DKG Part3 round2 packages len:{:?}",
            round2_received_packages.len()
        ));

        let (mut key_package, public_package) = frost_core::keys::dkg::part3(
            &round2_secret,
            &round1_received_packages,
            &round2_received_packages,
        )
        .map_err(|error| {
            ClientUtils::log_to_logcat(&format!("DKG Part3 error:{:?}", error));
            let error: FrostOpsError = error.into();

            error
        })?;

        round2_secret.zeroize();

        let encoded_key_package = FrostKeyPackageBytes::encode(&key_package)?;
        let encoded_public_package = FrostPublicKeyPackage::encode(&public_package)?;
        key_package.zeroize();
        let base58_public_key = encoded_key_package.verifying_key_base58::<FrostEd25519>()?;
        activity
            .metadata
            .group_key
            .replace(base58_public_key.clone());

        activity.metadata.state = ActivityState::DkgFinalized;
        activity.key_package.replace(encoded_key_package);
        activity.public_package.replace(encoded_public_package);

        org_info
            .activities
            .insert(activity.metadata.as_hex(), activity);

        storage.set_org_info(&domain_or_ip, org_info).await?;

        listener.on_recv(ActivityListenerOutcome {
            data: RustTypeActivitySubscriberChannel::DkgFinalized,
        });

        let op = QuicProtocolOp::DkgFinalized {
            participant: identity,
            activity_id: activity_store_key,
            base58_public_key,
        };

        Ok(op.encode())
    }
}
