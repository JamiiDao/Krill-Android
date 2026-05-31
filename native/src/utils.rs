use std::sync::{LazyLock, OnceLock};

use async_dup::Arc;
use async_lock::RwLock;

use crate::{AppStorage, RustFfiError};

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const KRILL_DIR: &str = ".Krill";

pub(crate) static APP_DB: OnceLock<AppStorage> = OnceLock::new();

pub(crate) fn app_storage() -> Result<&'static AppStorage, RustFfiError> {
    APP_DB.get().ok_or(RustFfiError::AppStorageNotInitialized)
}

pub(crate) type FrostEd25519 = frost_ed25519::Ed25519Sha512;

pub(crate) static FCM_TOKEN: LazyLock<Arc<RwLock<String>>> =
    LazyLock::new(|| Arc::new(RwLock::new(String::default())));

pub struct ClientUtils;

impl ClientUtils {
    pub fn log_to_logcat(message: &str) {
        unsafe {
            android_log_sys::__android_log_print(
                android_log_sys::LogPriority::DEBUG as i32,
                std::ffi::CString::new("Krill>::native ")
                    .unwrap_or_default()
                    .as_ptr(),
                std::ffi::CString::new(message).unwrap_or_default().as_ptr(),
            );
        }
    }
}
