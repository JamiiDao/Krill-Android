use frost_dkg_types::{AsymmetricKeypairBytes, EphemeralClientDeviceKeypair, FrostCredentialSeed};
use krill_common::{JoinPayload, OrganizationInfo, QuicProtocolOp};

use crate::{
    QuicClient, RustFfiError, RustTypeOrganizationInfo, RustTypeStoredOrgInfoMetadata,
    StoredOrgInfo, FCM_TOKEN,
};

#[uniffi::export]
async fn rust_fn_load_stored_organization_info(
    sld_tld: String,
) -> Result<Option<RustTypeStoredOrgInfoMetadata>, RustFfiError> {
    Ok(crate::app_storage()?
        .get_org_info(&sld_tld)
        .await?
        .map(|value| {
            let org_info: RustTypeStoredOrgInfoMetadata = value.into();
            org_info
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
async fn rust_fn_get_orgs_metadata() -> Result<Vec<RustTypeStoredOrgInfoMetadata>, RustFfiError> {
    crate::app_storage()?.get_all_orgs().await
}

#[uniffi::export]
async fn rust_fn_clear_org_info() -> Result<(), RustFfiError> {
    crate::app_storage()?.clear_org_info().await
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

        let org_info: OrganizationInfo = info.into();

        stored_org_info = StoredOrgInfo {
            sld_tld: sld_tld.clone(),
            registered: false,
            org_info,
            edkp,
            akp,
            identity,
        };

        crate::app_storage()?
            .set_org_info(&sld_tld, stored_org_info.clone())
            .await?;

        crate::ORG_INFO
            .write()
            .await
            .replace(stored_org_info.clone());
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
