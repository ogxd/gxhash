extern crate rustc_version;
use rustc_version::{version_meta, Channel};

fn main() {
    if version_meta().unwrap().channel == Channel::Nightly
    && cfg!(target_arch = "x86_64")
    && cfg!(target_feature = "avx2") {
        println!("cargo:rustc-cfg=hybrid");
    }
}