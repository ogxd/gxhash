// Feature 'avx2' currently requires unstable 'stdsimd'
#![cfg_attr(all(feature = "avx2", target_arch = "x86_64"), feature(stdsimd))]

mod gxhash;
mod hasher;

pub use gxhash::*;
pub use hasher::*;
