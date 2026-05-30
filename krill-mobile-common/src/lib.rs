mod notifications;
pub use notifications::*;

uniffi::setup_scaffolding!("rustkrill_mobile_common");
uniffi_reexport_scaffolding!();
