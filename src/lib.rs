#![cfg_attr(not(feature = "std"), no_std)]
// Hybrid SIMD width usage currently requires unstable 'stdsimd'
#![cfg_attr(hybrid, feature(stdsimd))]

#[rustfmt::skip]
mod gxhash;
pub use crate::gxhash::*;

#[cfg(feature = "std")]
mod hasher;
#[cfg(feature = "std")]
pub use crate::hasher::*;