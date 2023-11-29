#[cfg(target_arch = "aarch64")]
#[path = "arm_128.rs"]
mod s128;

#[cfg(target_arch = "x86_64")]
#[path = "x86_128.rs"]
mod s128;

pub use s128::*;

use crate::{Adapter, BlockProcessor};

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

type State128 = <Adapter128 as Adapter>::State;

impl BlockProcessor for Adapter128 {
    type State = State128;

    #[inline(always)]
    unsafe fn compress_8(mut ptr: *const State128, unrollable_blocks_count: usize, hash_vector: State128) -> State128 {
    
        let end_address = ptr.add(unrollable_blocks_count) as usize;
        let mut h1 = hash_vector;
        let mut h2 = Adapter128::create_empty();
        while (ptr as usize) < end_address {
    
            //load_unaligned!(Adapter128, ptr, v0, v1, v2, v3, v4, v5, v6, v7);
            let mut v0 = Self::load_unaligned(ptr);
            ptr = ptr.offset(1);
            let mut v1 = Self::load_unaligned(ptr);
            ptr = ptr.offset(1);
            let mut v2 = Self::load_unaligned(ptr);
            ptr = ptr.offset(1);
            let mut v3 = Self::load_unaligned(ptr);
            ptr = ptr.offset(1);
            let mut v4 = Self::load_unaligned(ptr);
            ptr = ptr.offset(1);
            let mut v5 = Self::load_unaligned(ptr);
            ptr = ptr.offset(1);
            let mut v6 = Self::load_unaligned(ptr);
            ptr = ptr.offset(1);
            let mut v7 = Self::load_unaligned(ptr);
            ptr = ptr.offset(1);
    
            let mut tmp1: State128;
            tmp1 = Self::compress_fast(v0, v2);
            tmp1= Self::compress_fast(tmp1, v4);
            tmp1 = Self::compress_fast(tmp1, v6);
            h1 = Self::compress(h1, tmp1);
    
            let mut tmp2: State128;
            tmp2 = Self::compress_fast(v1, v3);
            tmp2 = Self::compress_fast(tmp2, v5);
            tmp2 = Self::compress_fast(tmp2, v7);
            h2 = Self::compress(h2, tmp2);
        }
        Adapter128::compress(h1, h2)
    }
}