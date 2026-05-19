#[derive(Debug, uniffi::Enum, Clone, Copy, PartialEq, Eq, Default)]
pub enum RustTypeAppPermissionState {
    Granted,
    #[default]
    Denied,
    PermanentlyDenied,
}
