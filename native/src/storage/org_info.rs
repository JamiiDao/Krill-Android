use bitcode::{Decode, Encode};
use frost_dkg_types::{AsymmetricKeypairBytes, EphemeralClientDeviceKeypair, FrostCredentialSeed};
use krill_common::OrganizationInfo;
use redb::TableDefinition;

use crate::{AppStorage, RustFfiError, RustFfiResult};

impl AppStorage {
    pub(crate) const ORG_INFO_TABLE: TableDefinition<'static, &[u8], Vec<u8>> =
        TableDefinition::new("Organizations");

    pub(crate) async fn set_org_info(
        &self,
        sld_tld: &str,
        org_info: StoredOrgInfo,
    ) -> RustFfiResult<()> {
        self.set(Self::ORG_INFO_TABLE, sld_tld, org_info).await
    }

    pub(crate) async fn get_org_info(&self, sld_tld: &str) -> RustFfiResult<Option<StoredOrgInfo>> {
        self.get(Self::ORG_INFO_TABLE, sld_tld)
            .await?
            .map(|bytes| {
                bitcode::decode::<StoredOrgInfo>(&bytes)
                    .or(Err(RustFfiError::UnableToDecodeStoredOrgInfo))
            })
            .transpose()
    }

    pub(crate) async fn get_all_orgs(&self) -> RustFfiResult<Vec<RustTypeStoredOrgInfoMetadata>> {
        let orgs = self.get_all(Self::ORG_INFO_TABLE).await?;

        orgs.into_iter()
            .map(|bytes| {
                let org = bitcode::decode::<StoredOrgInfo>(&bytes)
                    .or(Err(RustFfiError::UnableToDecodeStoredOrgInfo))?;
                let org: RustTypeStoredOrgInfoMetadata = org.into();

                Ok::<_, RustFfiError>(org)
            })
            .collect()
    }

    pub(crate) async fn clear_org_info(&self) -> RustFfiResult<()> {
        self.drop_table(Self::ORG_INFO_TABLE).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub(crate) struct StoredOrgInfo {
    pub(crate) sld_tld: String,
    pub(crate) registered: bool,
    pub(crate) org_info: OrganizationInfo,
    pub(crate) edkp: EphemeralClientDeviceKeypair,
    pub(crate) akp: AsymmetricKeypairBytes,
    pub(crate) identity: FrostCredentialSeed,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, uniffi::Record)]
pub struct RustTypeStoredOrgInfoMetadata {
    pub sld_tld: String,
    pub registered: bool,
    pub org_name: String,
    pub logo_icon: Vec<u8>,
    pub support_mail: String,
    pub identity: String,
}

impl From<StoredOrgInfo> for RustTypeStoredOrgInfoMetadata {
    fn from(info: StoredOrgInfo) -> Self {
        Self {
            sld_tld: info.sld_tld,
            registered: info.registered,
            org_name: info.org_info.name,
            logo_icon: info.org_info.logo_icon,
            support_mail: info.org_info.support_mail,
            identity: info.identity.seed().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl From<OrganizationInfo> for RustTypeOrganizationInfo {
    fn from(org_info: OrganizationInfo) -> Self {
        Self {
            name: org_info.name,
            logo_icon: org_info.logo_icon,
            logo_horizontal: org_info.logo_horizontal,
            logo_vertical: org_info.logo_vertical,
            favicon: org_info.favicon,
            support_mail: org_info.support_mail,
        }
    }
}

impl From<RustTypeOrganizationInfo> for OrganizationInfo {
    fn from(org_info: RustTypeOrganizationInfo) -> Self {
        Self {
            name: org_info.name,
            logo_icon: org_info.logo_icon,
            logo_horizontal: org_info.logo_horizontal,
            logo_vertical: org_info.logo_vertical,
            favicon: org_info.favicon,
            support_mail: org_info.support_mail,
            ..Default::default()
        }
    }
}
