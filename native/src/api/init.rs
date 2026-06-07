use std::io::SeekFrom;

use async_fs::{create_dir_all, OpenOptions};
use bitcode::{Decode, Encode};
use camino::Utf8PathBuf;
use futures_lite::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use krill_common::QuicProtocolOp;

use crate::{AppStorage, QuicClient, RustFfiError, RustFfiResult, PKG_VERSION};

#[uniffi::export]
pub fn rust_fn_ffi_version() -> String {
    PKG_VERSION.to_string()
}

#[uniffi::export]
pub fn rust_fn_init_db(path: String) -> Result<(), RustFfiError> {
    crate::ClientUtils::log_to_logcat(&format!("App storage dir rust_fn_init_db load: {path}"));

    futures_lite::future::block_on(async move {
        let mut path: Utf8PathBuf = path.into();
        path.push(crate::KRILL_DIR);

        create_dir_all(&path).await?;

        path.push("AppStorage");

        let db = AppStorage::init(path).await?;

        if crate::APP_DB.set(db).is_err() {
            crate::ClientUtils::log_to_logcat("App Storage Already Initialized");
        }

        Ok(())
    })
}

#[uniffi::export]
pub async fn rust_fn_set_fcm_token(
    app_storage_path: String,
    token: String,
) -> Result<(), RustFfiError> {
    crate::ClientUtils::log_to_logcat(&format!("Received FCM token: {}", token));

    FcmTokenDetails::load(&app_storage_path, token, true, None, false).await?;

    Ok(())
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Encode, Decode)]
pub struct FcmTokenDetails {
    pub domain: String,
    pub identity: String,
    pub token: String,
}

impl FcmTokenDetails {
    pub async fn load(
        app_storage_path: &str,
        token: String,
        use_if_exists: bool,
        domain_and_identifier: Option<(&str, &str)>,
        first_run: bool,
    ) -> RustFfiResult<Self> {
        let mut path: Utf8PathBuf = app_storage_path.into();
        path.push("FCM.txt");

        crate::ClientUtils::log_to_logcat(&format!("App storage dir FcmTokenDetails load: {path}"));

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .await?;

        let mut stored_token = Vec::<u8>::new();
        file.read_to_end(&mut stored_token).await?;

        let mut decoded = if use_if_exists {
            Self::decode_token_data(&stored_token).inspect_err(|error| {
                crate::ClientUtils::log_to_logcat(&format!(
                    "Unable to decoded FCM data, FCM token not sent to relay! {error:?}"
                ));
            })?
        } else {
            if let Some((domain, identifier)) = domain_and_identifier.as_ref() {
                Self {
                    domain: domain.to_string(),
                    identity: identifier.to_string(),
                    token: token.clone(),
                }
            } else {
                return Err(RustFfiError::InvalidFcmTokenData);
            }
        };

        crate::ClientUtils::log_to_logcat(&format!("Decoded FCM token: {decoded:?}"));

        decoded.token = token.clone();

        if decoded.domain.is_empty() || decoded.identity.is_empty() {
            return Ok(decoded);
        }

        if !first_run && decoded.token.as_str() == token.as_str() {
            return Ok(decoded);
        }

        file.seek(SeekFrom::Start(0)).await?;
        file.set_len(0).await?;
        file.write_all(&decoded.encode()).await?;

        let payload = QuicProtocolOp::RefreshToken {
            identifier: decoded.identity.clone(),
            token,
        };

        crate::ClientUtils::log_to_logcat(&format!("Refresh FCM payload:{payload:?}"));
        QuicClient::connect::<()>(&decoded.domain, &payload).await?;

        crate::ClientUtils::log_to_logcat("Registered FCM to the relay");

        Ok(decoded)
    }

    fn encode(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    fn decode_token_data(bytes: &[u8]) -> RustFfiResult<Self> {
        bitcode::decode(bytes).or(Err(RustFfiError::InvalidFcmTokenData))
    }
}
