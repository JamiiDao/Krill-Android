use core::fmt;
use std::collections::HashMap;

use bitcode::{Decode, Encode};
use frost_dkg_types::FrostCredentialSeed;
use krill_common::{ActivityInfo, ActivityStoreKey, OrganizationInfo};
use redb::TableDefinition;

use crate::{
    AppStorage, RustFfiError, RustFfiResult, RustTypeActivityInfo, RustTypeActivityMetadata,
};

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

    pub(crate) async fn get_all_orgs_ffi(
        &self,
        timezone: i32,
    ) -> RustFfiResult<Vec<RustTypeStoredOrgInfoMetadata>> {
        let orgs = self.get_all(Self::ORG_INFO_TABLE).await?;

        orgs.into_iter()
            .map(|bytes| {
                let org = bitcode::decode::<StoredOrgInfo>(&bytes)
                    .or(Err(RustFfiError::UnableToDecodeStoredOrgInfo))?;
                let org: RustTypeStoredOrgInfoMetadata = (timezone, org).try_into()?;

                Ok::<_, RustFfiError>(org)
            })
            .collect()
    }

    pub(crate) async fn get_all_orgs(&self) -> RustFfiResult<Vec<StoredOrgInfo>> {
        let orgs = self.get_all(Self::ORG_INFO_TABLE).await?;

        orgs.into_iter()
            .map(|bytes| {
                let org = bitcode::decode::<StoredOrgInfo>(&bytes)
                    .or(Err(RustFfiError::UnableToDecodeStoredOrgInfo))?;

                Ok::<_, RustFfiError>(org)
            })
            .collect()
    }

    pub(crate) async fn clear_org_info(&self) -> RustFfiResult<()> {
        self.drop_table(Self::ORG_INFO_TABLE).await
    }
}

#[derive(Clone, PartialEq, Eq, Encode, Decode)]
pub(crate) struct StoredOrgInfo {
    pub(crate) sld_tld: String,
    pub(crate) org_info: OrganizationInfo,
    pub(crate) identity: FrostCredentialSeed,
    pub(crate) active: Option<String>,
    pub(crate) activities: HashMap<String, ActivityInfo>,
}

impl fmt::Debug for StoredOrgInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredOrgInfo")
            .field("sld_tld", &self.sld_tld)
            .field("active", &format!("{:?}", &self.active))
            .field("activities", &format!("{:?}", &self.activities))
            .field("org_info", &format!("{:?}", &self.org_info))
            .field("identity", &format!("{:?}", &self.identity))
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, uniffi::Record)]
pub struct RustTypeStoredOrgInfoMetadata {
    pub sld_tld: String,
    pub org_name: String,
    pub logo_icon: Vec<u8>,
    pub support_mail: String,
    pub identity: String,
    pub active: Option<String>,
    pub activities: Vec<RustTypeActivityInfo>,
}

impl fmt::Debug for RustTypeStoredOrgInfoMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustTypeStoredOrgInfoMetadata")
            .field("sld_tld", &self.sld_tld)
            .field("active", &format!("{:?}", &self.active))
            .field("activities", &format!("{:?}", &self.activities))
            .field("org_name", &format!("{:?}", &self.org_name))
            .field("identity", &format!("{:?}", &self.identity))
            .field("logo", &"SOME_BYTES")
            .finish()
    }
}

impl TryFrom<(i32, StoredOrgInfo)> for RustTypeStoredOrgInfoMetadata {
    type Error = RustFfiError;

    fn try_from(values: (i32, StoredOrgInfo)) -> Result<Self, Self::Error> {
        let (zone, info) = values;

        let activities = info
            .activities
            .values()
            .map(|activity| {
                let transformed: RustTypeActivityInfo = (zone, activity.clone()).try_into()?;

                Ok::<_, RustFfiError>(transformed)
            })
            .collect::<Result<Vec<RustTypeActivityInfo>, RustFfiError>>()?;

        Ok(Self {
            sld_tld: info.sld_tld,
            org_name: info.org_info.name,
            logo_icon: info.org_info.logo_icon,
            support_mail: info.org_info.support_mail,
            identity: info.identity.seed().to_string(),
            active: info.active,
            activities,
        })
    }
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
