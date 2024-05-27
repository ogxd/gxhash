// Hybrid SIMD width usage currently requires unstable 'stdsimd'
#![cfg_attr(feature = "hybrid", feature(stdarch_x86_avx512))]

#[cfg(all(feature = "hybrid", not(any(target_arch = "x86_64", target_feature = "aes", target_feature = "vaes", target_feature = "avx2"))))]
compile_error!{"Hybrid feature is only available on x86 processors with avx2 and vaes intrinsics."}

#[cfg(not(target_feature = "aes"))]
compile_error!{"Gxhash requires aes intrinsics. Make sure the processor supports it and build with RUSTFLAGS=\"-C target-cpu=native\" or RUSTFLAGS=\"-C target-feature=+aes\"."}

#[rustfmt::skip]
mod gxhash;
mod hasher;

pub use crate::gxhash::*;
pub use crate::hasher::*;