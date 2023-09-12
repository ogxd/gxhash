// Import ARM NEON intrinsics
#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

use std::{mem, intrinsics::{prefetch_read_data, likely}};

pub unsafe fn gxhash(input: &[i8]) -> u32 {

    const VECTOR_SIZE_SHIFT: usize = 4;
    const UNROLL_FACTOR_SHIFT: usize = 3;

    const VECTOR_SIZE: usize = 1 << VECTOR_SIZE_SHIFT;
    const UNROLL_FACTOR: usize = 1 << UNROLL_FACTOR_SHIFT;

    let len = input.len();

    let unrollable_blocks_count: usize = (len >> (VECTOR_SIZE_SHIFT + UNROLL_FACTOR_SHIFT)) << UNROLL_FACTOR_SHIFT; 
    let remaining_blocks_count: usize = (len >> VECTOR_SIZE_SHIFT) - unrollable_blocks_count;
    let remaining_bytes = len & (VECTOR_SIZE - 1);

    let mut p = input.as_ptr();
    let mut v = p as *const int8x16_t;
    let end_address = v.add(unrollable_blocks_count) as usize;

    let mut hash_vector: int8x16_t = vdupq_n_s8(0);

    prefetch_read_data(p, 2);

    while likely((v as usize) < end_address) {

        let mut block_hash_vector: int8x16_t = vdupq_n_s8(0);
        block_hash_vector = compress(block_hash_vector, *v);
        block_hash_vector = compress(block_hash_vector, *v.offset(1));
        block_hash_vector = compress(block_hash_vector, *v.offset(2));
        block_hash_vector = compress(block_hash_vector, *v.offset(3));
        block_hash_vector = compress(block_hash_vector, *v.offset(4));
        block_hash_vector = compress(block_hash_vector, *v.offset(5));
        block_hash_vector = compress(block_hash_vector, *v.offset(6));
        block_hash_vector = compress(block_hash_vector, *v.offset(7));
        
        hash_vector = compress(hash_vector, block_hash_vector);

        // hash_vector = compress(hash_vector, *v);
        // hash_vector = compress(hash_vector, *v.offset(1));
        // hash_vector = compress(hash_vector, *v.offset(2));
        // hash_vector = compress(hash_vector, *v.offset(3));
        // hash_vector = compress(hash_vector, *v.offset(4));
        // hash_vector = compress(hash_vector, *v.offset(5));
        // hash_vector = compress(hash_vector, *v.offset(6));
        // hash_vector = compress(hash_vector, *v.offset(7));

        v = v.add(UNROLL_FACTOR);
    }

    let end_address = v.add(remaining_blocks_count) as usize;

    while (v as usize) < end_address {

        hash_vector = compress(hash_vector, *v);
        v = v.add(1);
    }

    // Bit-cast the int8x16_t to uint32x4_t
    let vec_u32: uint32x4_t = mem::transmute(hash_vector);

    // Get the first u32 value from the vector
    let first_u32: u32 = vgetq_lane_u32(vec_u32, 3);
    

    //vaeseq_u8(hash_vector, hash_vector);

    first_u32
}

#[inline]
unsafe fn compress(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    vaddq_s8(a, b)
}