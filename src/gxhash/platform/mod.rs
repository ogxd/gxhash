#[cfg(target_arch = "aarch64")]
#[path = "arm_128.rs"]
pub mod platform;

#[cfg(target_arch = "x86_64")]
#[path = "x86_256.rs"]
pub mod platform;

pub use platform::*;