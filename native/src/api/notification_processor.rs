use bitcode::{Decode, Encode};
use tracing::Level;

use crate::{api::notification_channels::NotificationChannelInfo, TracingKeys};

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
    data: RustTypeReceivedNotificationData,
) -> Option<RustTypeFetchedNotificationInfo> {
    if data.data.is_none() {
        TracingKeys::Notifications(data).log(Level::ERROR);
        return None;
    }

    // TODO: Do Network op

    Some(RustTypeFetchedNotificationInfo {
        channel_info: NotificationChannelInfo::v0_1_2(),
        group_event_id: "jamiidaoappsigning12345678".to_string(),
        notification_id: 12345678_i32,
        heading: "Admin Creation Event".to_string(),
        subheading: "Participate in group signing event to create an admin".to_string(),
        live_update: false,
    })
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
