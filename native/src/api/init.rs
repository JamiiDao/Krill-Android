use async_fs::create_dir_all;
use camino::Utf8PathBuf;
use krill_common::QuicProtocolOp;

use crate::{AppStorage, QuicClient, RustFfiError, PKG_VERSION};

#[uniffi::export]
pub fn rust_fn_ffi_version() -> String {
    PKG_VERSION.to_string()
}

#[uniffi::export]
pub async fn rust_fn_init_db(path: String) -> Result<(), RustFfiError> {
    let mut path: Utf8PathBuf = path.into();
    path.push(crate::KRILL_DIR);

    create_dir_all(&path).await?;

    path.push("AppStorage");

    let db = AppStorage::init(path).await?;

    if crate::APP_DB.set(db).is_err() {
        Err(RustFfiError::AppStorageNotInitialized)
    } else {
        Ok(())
    }
}

#[uniffi::export]
pub async fn rust_fn_set_fcm_token(token: String) -> Result<(), RustFfiError> {
    crate::ClientUtils::log_to_logcat(&format!("Received FCM token: {}", token));

    *crate::FCM_TOKEN.write().await = token;

    if let Some(stored_info) = crate::ORG_INFO.read().await.as_ref() {
        let payload = QuicProtocolOp::RefreshToken {
            identifier: stored_info.identity.seed().to_string(),
            token: crate::FCM_TOKEN.read().await.to_string(),
        };

        crate::ClientUtils::log_to_logcat("Sending FCM token");

        QuicClient::connect::<()>(&stored_info.sld_tld, &payload).await?;

        crate::ClientUtils::log_to_logcat(&format!(
            "Registered backend: {}",
            crate::FCM_TOKEN.read().await.as_str()
        ));
    } else {
        crate::ClientUtils::log_to_logcat("Ignoring FCM token as no endpoint registered");
    }

    Ok(())
}
