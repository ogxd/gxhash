extern crate rustc_version;
use rustc_version::{version_meta, Channel};

fn main() {

    if let Err(e) = check_hardware_support() {
        panic!("Gxhash build failed: {}", e);
    }

    if version_meta().unwrap().channel == Channel::Nightly
    && cfg!(target_arch = "x86_64")
    && cfg!(target_feature = "avx2")
    && cfg!(target_feature = "vaes") {
        println!("cargo:rustc-cfg=hybrid");
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn has_hardware_support() -> Result<(), Box<dyn std::error::Error>> {
    if !std::is_x86_feature_detected!("aes") {
        return Result::Err("CPU feature 'aes' is required")?;
    }
    if !std::is_x86_feature_detected!("sse2") {
        return Result::Err("CPU feature 'sse2' is required")?;
    }
    return Ok(());
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn check_hardware_support() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(any(target_arch = "arm", target_arch = "aarch64")) {
        if cfg!(not(target_feature = "aes")) {
            return Result::Err("CPU feature 'aes' is required")?;
        }
        if cfg!(not(target_feature = "neon")) {
            return Result::Err("CPU feature 'neon' is required")?;
        }
        return Ok(());
    }
    return Result::Err(format!("Target arch '{}' is not supported", std::env::consts::ARCH))?;
}