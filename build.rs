extern crate rustc_version;
use rustc_version::{version_meta, Channel};

fn main() {
    // Avoid platform checks if building for docs.rs
    if &std::env::var("DOCS_RS").unwrap_or_default() == "1" {
        return;
    }

    // When conditions permits, enable hybrid feature to leverage wider intrinsics for even more throughput
    if version_meta().unwrap().channel == Channel::Nightly
    && cfg!(target_arch = "x86_64")
    && cfg!(target_feature = "avx2")
    && cfg!(target_feature = "vaes") {
        println!("cargo:rustc-cfg=hybrid");
    }

    // If not cross compiling, make sure the aes feature is available
    if std::env::var("HOST").unwrap_or_default() == std::env::var("TARGET").unwrap_or_default()
    && cfg!(not(target_feature = "aes")) {
        panic!("| GxHash requires target-feature 'aes' to be enabled.\n\
        | Build with RUSTFLAGS=\"-C target-cpu=native\" or RUSTFLAGS=\"-C target-feature=+aes\" to enable.");
    }
}
