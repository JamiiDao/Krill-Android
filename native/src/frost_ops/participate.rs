use bitcode::{Decode, Encode};
use krill_common::{
    round1, ActivityStoreKey, AsymmetricKeypairBytes, Blake3HashBytes, FrostCredentialSeed,
    FrostParticipateMessage, MinMaxParticipants, Tai64NTimestamp,
};

use crate::{FrostEd25519, RustFfiError, RustFfiResult};

pub struct FrostParticipateMessageWrapper(pub(crate) FrostParticipateMessage);

impl FrostParticipateMessageWrapper {
    pub async fn new(
        domain_or_ip: &str,
        activity_id: ActivityStoreKey,
        min_max: MinMaxParticipants,
    ) -> RustFfiResult<Self> {
        let ecdk = AsymmetricKeypairBytes::new()?;

        crate::ClientUtils::log_to_logcat("ECDK GENERATED!");

        let store = crate::app_storage()?;

        crate::ClientUtils::log_to_logcat("-> APP STORAGE ACCESSED!");

        let credential: FrostCredentialSeed;

        if let Some(org_info) = store.get_org_info(domain_or_ip).await? {
            credential = org_info.identity;
        } else {
            crate::ClientUtils::log_to_logcat("ORG NOT FOUND, CANNOT PARTICIPATE!");

            return Err(RustFfiError::OrgNotFound);
        }

        crate::ClientUtils::log_to_logcat("-> ORG & CREDENTIAL FOUND!");

        let identifier = credential.frost_identifier::<FrostEd25519>()?;

        let (secret, public) =
            frost_core::keys::dkg::part1(identifier, min_max.max, min_max.min, rand::rngs::OsRng)
                .map_err(|error| RustFfiError::Frost(error.to_string()))?;

        crate::ClientUtils::log_to_logcat("-> round1 secret/public generated!");

        let round1_secret = round1::Round1SecretBytes::new::<FrostEd25519>(secret)?;
        crate::ClientUtils::log_to_logcat("-> round1 secret parsed!");
        let round1_public = round1::Round1PackageBytes::parse::<FrostEd25519>(&public)?;
        crate::ClientUtils::log_to_logcat("-> round1 public parsed!");

        crate::ClientUtils::log_to_logcat("-> round1 generated & parsed!");

        let outcome = FrostParticipateMessage {
            domain_or_ip: domain_or_ip.to_string(),
            timestamp: Tai64NTimestamp::now(),
            activity_id,
            participant: credential.seed().to_string(),
            min_max,
            round1_dkg: round1_public.clone(),
            ..Default::default()
        };

        let to_storage = StoredFrostParticipateMessage {
            domain_or_ip: outcome.domain_or_ip.to_string(),
            timestamp: outcome.timestamp,
            activity_id,
            participant: outcome.participant.clone(),
            min_max,
            round1_secret,
            round1_public,
            binding_hash: outcome.binding_hash,
            ecdk: ecdk.clone(),
        };

        let mut wrapped = FrostParticipateMessageWrapper(outcome);
        wrapped.compute_signature(ecdk)?;

        store.set_activity(to_storage).await?;

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
pub struct StoredFrostParticipateMessage {
    pub domain_or_ip: String,
    pub timestamp: Tai64NTimestamp,
    pub activity_id: ActivityStoreKey,
    pub participant: String,
    pub min_max: MinMaxParticipants,
    pub round1_secret: round1::Round1SecretBytes,
    pub round1_public: round1::Round1PackageBytes,
    pub binding_hash: Blake3HashBytes,
    pub ecdk: AsymmetricKeypairBytes,
}
