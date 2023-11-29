#[cfg(all(feature = "s256", target_arch = "x86_64", target_feature = "avx2"))]
#[path = "x86_256.rs"]
mod s256;

pub use s256::*;

#[inline(always)]
pub fn gxhash32(input: &[u8], seed: i64) -> u32 {
    crate::gxhash::gxhash32::<Adapter256>(input, seed)
}

#[inline(always)]
pub fn gxhash64(input: &[u8], seed: i64) -> u64 {
    crate::gxhash::gxhash64::<Adapter256>(input, seed)
}

#[inline(always)]
pub fn gxhash128(input: &[u8], seed: i64) -> u128 {
    crate::gxhash::gxhash128::<Adapter256>(input, seed)
}