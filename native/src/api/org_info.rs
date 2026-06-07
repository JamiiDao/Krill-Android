use std::path::Path;

use frost_dkg_types::FrostCredentialSeed;
use krill_common::{JoinPayload, OrganizationInfo, QuicProtocolOp};

use crate::{
    api::init::FcmTokenDetails, QuicClient, RustFfiError, RustTypeOrganizationInfo,
    RustTypeStoredOrgInfoMetadata, StoredOrgInfo,
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
    crate::app_storage()?.get_all_orgs_ffi().await
}

#[uniffi::export]
async fn rust_fn_clear_org_info() -> Result<(), RustFfiError> {
    crate::app_storage()?.clear_org_info().await
}

#[uniffi::export]
async fn rust_fn_join(
    app_storage_path: String,
    sld_tld: String,
    info: RustTypeOrganizationInfo,
    token: String,
) -> Result<(), RustFfiError> {
    crate::ClientUtils::log_to_logcat(&format!(
        "App storage dir rust_fn_join load: {app_storage_path:?} - {token}"
    ));

    let identity = FrostCredentialSeed::new_anonymous::<crate::FrostEd25519>()?;

    let stored_org_info;

    let fcm_token_details = FcmTokenDetails::load(
        &app_storage_path,
        token,
        false,
        Some((&sld_tld, &identity.seed())),
        true,
    )
    .await?;

    if let Some(org_exists) = crate::app_storage()?.get_org_info(&sld_tld).await? {
        crate::ClientUtils::log_to_logcat("ORG EXISTS");

        stored_org_info = org_exists;
    } else {
        crate::ClientUtils::log_to_logcat("ORG NOT FOUND");

        let org_info: OrganizationInfo = info.into();

        stored_org_info = StoredOrgInfo {
            sld_tld: sld_tld.clone(),
            org_info,
            identity,
        };

        let payload = JoinPayload {
            identity: stored_org_info.identity.seed().to_string(),
            fcm_token: fcm_token_details.token,
        };

        let payload = QuicProtocolOp::Join(payload);

        crate::ClientUtils::log_to_logcat(&format!("JOIN Payload:{payload:?}"));

        QuicClient::connect::<()>(&sld_tld, &payload).await?;

        crate::ClientUtils::log_to_logcat("BACKEND REGISTER SUCCESS");

        crate::app_storage()?
            .set_org_info(&sld_tld, stored_org_info.clone())
            .await?;
    }

    crate::ORG_INFO
        .write()
        .await
        .replace(stored_org_info.clone());

    Ok(())
}
