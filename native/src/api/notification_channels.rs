#[uniffi::export]
pub fn rust_fn_notification_versioning_ops() -> NotificationVersioningOps {
    NotificationVersioningOps::v0_1_2()
}

#[derive(Debug, PartialEq, Eq, Clone, uniffi::Record)]
pub struct NotificationVersioningOps {
    pub add: Vec<NotificationChannelInfo>,
    pub remove: Vec<NotificationChannelInfo>,
}

// Versioning, always select the latest version by deprecating other versions
impl NotificationVersioningOps {
    #[deprecated]
    pub fn v0_1_1() -> Self {
        Self {
            add: vec![],
            remove: vec![],
        }
    }

    pub fn v0_1_2() -> Self {
        Self {
            add: vec![NotificationChannelInfo::v0_1_2()],
            #[allow(deprecated)]
            remove: Self::v0_1_1().add,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, uniffi::Record)]
pub struct NotificationChannelInfo {
    pub channel_id: String,
    pub channel_name: String,
    pub importance: RustTypeNotificationImportance,
    pub icon_type: RustTypeNotificationIconType,
}

impl NotificationChannelInfo {
    pub fn v0_1_2() -> Self {
        Self {
            channel_id: "group-signing".to_string(),
            channel_name: "Notifications For Group Signing Events".to_string(),
            importance: RustTypeNotificationImportance::High,
            icon_type: RustTypeNotificationIconType::Signing,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, uniffi::Enum)]
pub enum RustTypeNotificationImportance {
    /// No notification shown
    None,
    /// Silent, collapsed, bottom of shade
    Min,
    /// Silent, visible in shade
    Low,
    /// Makes sound, no heads-up popup usually
    #[default]
    Default,
    /// Sound + heads-up popup
    High,
    /// System Decides
    Unspecified,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, uniffi::Enum)]
pub enum RustTypeNotificationIconType {
    #[default]
    Default,
    Signing,
}
