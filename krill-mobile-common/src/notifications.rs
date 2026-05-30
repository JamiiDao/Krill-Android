#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct NotificationInfo {
    r#type: NotificationType,
    importance: RustTypeNotificationImportance,
    channel_id: String,
    channel_name: String,
    recipients: Vec<String>, //Maps to a Credential that the server understands
}

impl NotificationInfo {
    pub fn r#type(&self) -> NotificationType {
        self.r#type
    }

    pub fn channel_id(&self) -> &str {
        self.channel_id.as_str()
    }

    pub fn channel_name(&self) -> &str {
        self.channel_name.as_str()
    }

    pub fn importance(&self) -> RustTypeNotificationImportance {
        self.importance
    }

    pub fn recipients(&self) -> &[String] {
        self.recipients.as_slice()
    }
}
