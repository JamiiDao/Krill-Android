use base64ct::{Base64, Encoding};
use bitcode::{Decode, Encode};
use tracing::Level;

use crate::api::RustTypeReceivedNotificationData;

#[derive(Debug, Default, PartialEq, Eq, Clone, Encode, Decode)]
pub enum TracingKeys {
    Notifications(RustTypeReceivedNotificationData),
    #[default]
    Unsupported,
}

impl TracingKeys {
    pub fn log_key(&self) -> &'static str {
        match self {
            Self::Notifications(_) => "Notification",
            Self::Unsupported => "Unsupported",
        }
    }

    pub fn log(&self, trace_level: Level) {
        let data = bitcode::encode(self);
        let base64_encoded = Base64::encode_string(&data);
        let data = self.log_key().to_string() + ":" + base64_encoded.as_str();

        match trace_level {
            Level::INFO => tracing::info!(data),
            Level::DEBUG => tracing::debug!(data),
            Level::TRACE => tracing::trace!(data),
            Level::ERROR => tracing::error!(data),
            Level::WARN => tracing::warn!(data),
        }
    }

    pub fn decode_notification<T: Decode<'static>>(data: &str) -> Self {
        let split = data.split(":").collect::<Vec<&str>>();

        if split.len() != 2 {
            return Self::default();
        }

        let bytes = if let Ok(value) = Base64::decode_vec(split[1]) {
            value
        } else {
            return Self::default();
        };

        bitcode::decode::<Self>(&bytes).unwrap_or_default()
    }
}
