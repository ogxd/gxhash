#![cfg_attr(not(feature = "std"), no_std)]
// Parts of the backends are only used by the Hasher, which requires std
#![cfg_attr(not(feature = "std"), allow(dead_code))]

#[rustfmt::skip]
mod gxhash;
pub use crate::gxhash::*;

#[cfg(feature = "std")]
mod hasher;
#[cfg(feature = "std")]
pub use crate::hasher::*;