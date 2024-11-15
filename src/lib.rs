#![cfg_attr(not(feature = "std"), no_std)]
// Hybrid feature is only available for x86 processors supporting AVX2 instrinsics and requires nightly rust.
#![cfg_attr(all(feature = "hybrid", any(target_arch = "x86", target_arch = "x86_64")), feature(stdarch_x86_avx512))]

#[rustfmt::skip]
mod gxhash;
pub use crate::gxhash::*;

#[cfg(feature = "std")]
mod hasher;
#[cfg(feature = "std")]
pub use crate::hasher::*;