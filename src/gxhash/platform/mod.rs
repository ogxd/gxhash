#[cfg(target_arch = "aarch64")]
#[path = "arm_128.rs"]
pub mod platform;

#[cfg(all(
    feature = "256-bit",
    target_arch = "x86_64",
    target_feature = "avx2")
)]
#[path = "x86_256.rs"]
pub mod platform;

#[cfg(all(
    not(feature = "256-bit"),
    target_arch = "x86_64"
))]
#[path = "x86_128.rs"]
pub mod platform;

pub use platform::*;