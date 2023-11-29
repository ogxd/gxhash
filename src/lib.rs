// AVX2 currently requires unstable 'stdsimd'
#![cfg_attr(all(target_arch = "x86_64", target_feature = "avx2", feature = "unstable"), feature(stdsimd))]

#[rustfmt::skip]
mod gxhash;
mod hasher;

pub use crate::gxhash::*;
pub use crate::hasher::*;