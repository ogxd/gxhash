#[cfg(all(feature = "s256", target_arch = "x86_64", target_feature = "avx2"))]
#[path = "x86_256.rs"]
mod s256;