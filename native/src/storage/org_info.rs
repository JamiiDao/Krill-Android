use bitcode::{Decode, Encode};
use frost_dkg_types::{AsymmetricKeypairBytes, EphemeralClientDeviceKeypair, FrostCredentialSeed};
use krill_common::OrganizationInfo;
use redb::TableDefinition;

use crate::{AppStorage, RustFfiError, RustFfiResult};

impl AppStorage {
    pub(crate) const ORG_INFO_TABLE: TableDefinition<'static, &[u8], Vec<u8>> =
        TableDefinition::new("Organizations");

    pub(crate) async fn get_org_info(&self, sld_tld: &str) -> RustFfiResult<Option<StoredOrgInfo>> {
        self.get(Self::ORG_INFO_TABLE, sld_tld)
            .await?
            .map(|bytes| {
                bitcode::decode::<StoredOrgInfo>(&bytes)
                    .or(Err(RustFfiError::UnableToDecodeStoredOrgInfo))
            })
            .transpose()
    }

    pub(crate) async fn set_org_info(
        &self,
        sld_tld: &str,
        org_info: StoredOrgInfo,
    ) -> RustFfiResult<()> {
        self.set(Self::ORG_INFO_TABLE, sld_tld, org_info).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub(crate) struct StoredOrgInfo {
    pub(crate) registered: bool,
    pub(crate) org_info: OrganizationInfo,
    pub(crate) edkp: EphemeralClientDeviceKeypair,
    pub(crate) akp: AsymmetricKeypairBytes,
    pub(crate) identity: FrostCredentialSeed,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ParticipantOrgInfo {
    pub(crate) ecdvk: String,
    pub(crate) avk: String,
    pub(crate) org_info: RustTypeOrganizationInfo,
    pub(crate) identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct RustTypeOrganizationInfo {
    pub name: String,
    pub logo_icon: Vec<u8>,
    pub logo_horizontal: Vec<u8>,
    pub logo_vertical: Vec<u8>,
    pub favicon: Vec<u8>,
    pub support_mail: String,
}
