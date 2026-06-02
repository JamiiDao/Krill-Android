use async_fs::create_dir_all;
use camino::Utf8PathBuf;

use crate::{AppStorage, RustFfiError, PKG_VERSION};

#[uniffi::export]
pub fn rust_fn_ffi_version() -> String {
    PKG_VERSION.to_string()
}

#[uniffi::export]
pub async fn rust_fn_set_fcm_token(token: String) {
    *crate::FCM_TOKEN.write().await = token;

    crate::ClientUtils::log_to_logcat(&format!(
        "Registered backend: {}",
        crate::FCM_TOKEN.read().await.as_str()
    ));
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
