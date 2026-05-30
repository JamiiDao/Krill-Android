use std::sync::OnceLock;

use async_fs::create_dir_all;
use camino::Utf8PathBuf;
use krill_common::QuicProtocolData;

use crate::{AppStorage, ClientUtils, QuicClient, RustFfiError, PKG_VERSION};

pub(crate) const KRILL_DIR: &str = ".Krill";
pub(crate) static APP_DB: OnceLock<AppStorage> = OnceLock::new();

pub(crate) fn app_storage() -> Result<&'static AppStorage, RustFfiError> {
    APP_DB.get().ok_or(RustFfiError::AppStorageNotInitialized)
}

#[uniffi::export]
pub fn rust_fn_ffi_version() -> String {
    PKG_VERSION.to_string()
}

#[uniffi::export]
pub async fn rust_fn_init_db(path: String) -> Result<(), RustFfiError> {
    let mut path: Utf8PathBuf = path.into();
    path.push(KRILL_DIR);

    create_dir_all(&path).await?;

    path.push("AppStorage");

    let db = AppStorage::init(path).await?;

    if APP_DB.set(db).is_err() {
        Err(RustFfiError::AppStorageNotInitialized)
    } else {
        Ok(())
    }
}

// #[uniffi::export]
// pub async fn rust_fn_quic_hello() -> Result<(), RustFfiError> {
//     #[cfg(debug_assertions)]
//     let server_address =
//         std::net::SocketAddr::V4(SocketAddrV4::new([192, 168, 100, 134].into(), 6766));
//     ClientUtils::log_to_logcat(&format!("Server Received: {:?}", server_address));
//     #[cfg(debug_assertions)]
//     let server_domain_name = "192.168.100.134";

//     QuicClient::connect(&QuicProtocolData::Hello)
//         .await
//         .map_err(|error| {
//             ClientUtils::log_to_logcat(error.to_string().as_str());
//             RustFfiError::Quic(error.to_string())
//         })
// }
