use bitcode::{Decode, Encode};
use krill_common::ActivityStoreKey;
use tracing::Level;

use crate::{api::notification_channels::NotificationChannelInfo, ClientUtils, TracingKeys};

#[derive(Debug, uniffi::Record, Default, PartialEq, Eq, Clone, Encode, Decode)]
pub struct RustTypeReceivedNotificationData {
    /// custom payload type `Map<String, String>`
    pub data: Option<String>,
    /// Unique ID assigned by FCM for this message. Can be null
    pub message_id: Option<String>,
    /// Typically your FCM sender ID
    pub from: Option<String>,
    /// The priority as set by sender before delivery
    pub original_priority: i32,
    /// The actual delivery priority after system processing
    pub priority: i32,
    /// Numeric sender ID of the FCM project
    /// Identifies which Firebase project sent the message
    pub sender_id: Option<String>,
    /// Timestamp (milliseconds since epoch)
    /// When message was sent from FCM backend
    pub sent_time: i64,
    /// maximum FCM retention window after exponential backoff
    pub ttl: i32,
}

#[uniffi::export]
async fn rust_fn_process_notification_info(
    mut data: RustTypeReceivedNotificationData,
) -> RustTypeFetchedNotificationInfo {
    ClientUtils::log_to_logcat(&format!("RECEIVED NOTIFICATION: {data:?}"));

    let channel_info = NotificationChannelInfo::v0_1_2();
    let notification_id = 1234567890_i32;
    let group_event_id = data
        .data
        .take()
        .unwrap_or("error/Invalid-Notification-Error.".to_string());
    let subheading = "You are signature is required to sign the activity".to_string();

    let mut return_data = RustTypeFetchedNotificationInfo {
        channel_info,
        group_event_id,
        notification_id,
        heading: "Signature Requested".to_string(),
        subheading,
        live_update: false,
    };

    let initial = return_data.group_event_id.clone();

    let [_, _] = match initial
        .split("/")
        .map(|value| value.to_string())
        .collect::<Vec<String>>()
        .try_into()
    {
        Ok(value) => value,
        Err(_) => {
            return_data.group_event_id = "error/Invalid notification deeplink.".to_string();
            return_data.heading = "Deeplink Error".to_string();
            return_data.subheading =
                "The notification received contains invalid details.".to_string();

            return return_data;
        }
    };

    return_data
}

#[derive(Debug, Default, uniffi::Record)]
pub struct RustTypeFetchedNotificationInfo {
    pub notification_id: i32,
    pub group_event_id: String,
    pub channel_info: NotificationChannelInfo,
    pub heading: String,
    pub subheading: String,
    pub live_update: bool,
}
