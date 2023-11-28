#[cfg(target_arch = "aarch64")]
#[path = "arm_128.rs"]
mod s128;

#[cfg(all(not(feature = "s256"), target_arch = "x86_64"))]
#[path = "x86_128.rs"]
mod s128;

pub use s128::*;

#[inline(always)]
pub fn gxhash32(input: &[u8], seed: i64) -> u32 {
    crate::gxhash::gxhash32::<Adapter128>(input, seed)
}

#[inline(always)]
pub fn gxhash64(input: &[u8], seed: i64) -> u64 {
    crate::gxhash::gxhash64::<Adapter128>(input, seed)
}

#[inline(always)]
pub fn gxhash128(input: &[u8], seed: i64) -> u128 {
    crate::gxhash::gxhash128::<Adapter128>(input, seed)
}