use frost_dkg_types::{AsymmetricKeypairBytes, EphemeralClientDeviceKeypair, FrostCredentialSeed};
use krill_common::{JoinPayload, OrganizationInfo, QuicProtocolOp};

use crate::{
    ParticipantOrgInfo, QuicClient, RustFfiError, RustTypeOrganizationInfo, StoredOrgInfo,
    FCM_TOKEN,
};

#[uniffi::export]
async fn rust_fn_load_stored_organization_info(
    sld_tld: String,
) -> Result<Option<ParticipantOrgInfo>, RustFfiError> {
    Ok(crate::app_storage()?
        .get_org_info(&sld_tld)
        .await?
        .map(|value| {
            let org_info: RustTypeOrganizationInfo = value.org_info.into();

            let encode = |bytes: &[u8]| -> String { bs58::encode(bytes).into_string() };

            ParticipantOrgInfo {
                ecdvk: encode(value.edkp.verifying_key_encodable().0.as_slice()),
                avk: encode(value.akp.verifying_key_encodable().0.as_slice()),
                org_info,
                identity: value.identity.seed().to_string(),
            }
        }))
}

#[uniffi::export]
async fn rust_fn_fetch_org_info(sld_tld: String) -> Result<RustTypeOrganizationInfo, RustFfiError> {
    let payload = QuicProtocolOp::OrganizationInfo;

    Ok(QuicClient::connect::<OrganizationInfo>(&sld_tld, &payload)
        .await
        .map_err(|error| RustFfiError::Quic(error.to_string()))?
        .into())
}

#[uniffi::export]
async fn rust_fn_join(sld_tld: String, info: RustTypeOrganizationInfo) -> Result<(), RustFfiError> {
    let mut stored_org_info;

    if let Some(org_exists) = crate::app_storage()?.get_org_info(&sld_tld).await? {
        crate::ClientUtils::log_to_logcat("ORG EXISTS");

        stored_org_info = org_exists;
    } else {
        crate::ClientUtils::log_to_logcat("ORG NOT FOUND");

        let edkp = EphemeralClientDeviceKeypair::new()?;
        let akp = AsymmetricKeypairBytes::new()?;
        let identity = FrostCredentialSeed::new_anonymous::<crate::FrostEd25519>()?;

        let org_info = info.into();

        stored_org_info = StoredOrgInfo {
            registered: false,
            org_info,
            edkp,
            akp,
            identity,
        };

        crate::app_storage()?
            .set_org_info(&sld_tld, stored_org_info.clone())
            .await?
    }

    let payload = JoinPayload {
        identity: stored_org_info.identity.seed().to_string(),
        fcm_token: FCM_TOKEN.read().await.clone(),
        edvk: stored_org_info.edkp.verifying_key_encodable(),
        akp: stored_org_info.akp.verifying_key_encodable(),
    };

    if stored_org_info.registered {
        crate::ClientUtils::log_to_logcat("BACKEND ALREADY REGISTER");

        Ok(())
    } else {
        let payload = QuicProtocolOp::Join(payload);

        QuicClient::connect::<()>(&sld_tld, &payload).await?;

        crate::ClientUtils::log_to_logcat("BACKEND REGISTER SUCCESS");

        stored_org_info.registered = true;

        crate::app_storage()?
            .set_org_info(&sld_tld, stored_org_info.clone())
            .await
    }
}

impl From<OrganizationInfo> for RustTypeOrganizationInfo {
    fn from(value: OrganizationInfo) -> Self {
        RustTypeOrganizationInfo {
            name: value.name,
            logo_icon: value.logo_icon,
            logo_horizontal: value.logo_horizontal,
            logo_vertical: value.logo_vertical,
            favicon: value.favicon,
            support_mail: value.support_mail,
        }
    }
}

impl From<RustTypeOrganizationInfo> for OrganizationInfo {
    fn from(value: RustTypeOrganizationInfo) -> Self {
        OrganizationInfo {
            name: value.name,
            logo_icon: value.logo_icon,
            logo_horizontal: value.logo_horizontal,
            logo_vertical: value.logo_vertical,
            favicon: value.favicon,
            support_mail: value.support_mail,
            ..Default::default()
        }
    }
}
