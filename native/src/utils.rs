pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

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
