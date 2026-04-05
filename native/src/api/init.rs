use crate::PKG_VERSION;

#[uniffi::export]
pub fn rustffi_ffi_version() -> String {
    PKG_VERSION.to_string()
}
