extern crate rustc_version;
use rustc_version::{version_meta, Channel};

fn main() {

    if !has_hardware_support() {
        panic!("Hardware is not supported");
    }

    if version_meta().unwrap().channel == Channel::Nightly
    && cfg!(target_arch = "x86_64")
    && cfg!(target_feature = "avx2")
    && cfg!(target_feature = "vaes") {
        println!("cargo:rustc-cfg=hybrid");
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn has_hardware_support() -> bool {
    std::is_x86_feature_detected!("aes") && std::is_x86_feature_detected!("sse2")
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn has_hardware_support() -> bool {
    cfg!(all(any(target_arch = "arm", target_arch = "aarch64"), target_feature = "aes", target_feature = "neon"))
}