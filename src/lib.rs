// Hybrid SIMD width usage currently requires unstable 'stdsimd'
#![cfg_attr(hybrid, feature(stdsimd))]

#[rustfmt::skip]
mod gxhash;
mod hasher;

pub use crate::gxhash::*;
pub use crate::hasher::*;