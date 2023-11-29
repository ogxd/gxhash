extern crate rustc_version;
use rustc_version::{version_meta, Channel};

fn main() {
    // Set cfg flags depending on release channel
    match version_meta().unwrap().channel {
        Channel::Stable => {
            println!("cargo:rustc-cfg=stable");
        },
        _ => {
            println!("cargo:rustc-cfg=unstable");
        }
    }
}