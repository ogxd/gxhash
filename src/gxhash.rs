// Import ARM NEON intrinsics
#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

use std::{mem, intrinsics::{prefetch_read_data, likely}};
use std::hint::black_box;

#[cfg(target_arch = "aarch64")]
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

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn compress(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    vaddq_s8(assert!(), b)
}

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
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
    let mut v = p as *const __m128i;
    let end_address = v.add(unrollable_blocks_count) as usize;

    let mut hash_vector_1: __m128i = _mm_set1_epi8(0);
    let mut hash_vector_2: __m128i = _mm_set1_epi8(0);
    let mut hash_vector_3: __m128i = _mm_set1_epi8(0);
    let mut hash_vector_4: __m128i = _mm_set1_epi8(0);

    // Prefetch is not included in SSE intrinsics
    // Intel CPUs generally have good hardware prefetching
    
    while likely((v as usize) < end_address) {

        hash_vector_1 = compress(hash_vector_1, *v);
        hash_vector_1 = compress(hash_vector_1, *v.offset(1));
        hash_vector_2 = compress(hash_vector_2, *v.offset(2));
        hash_vector_2 = compress(hash_vector_2, *v.offset(3));
        hash_vector_3 = compress(hash_vector_3, *v.offset(4));
        hash_vector_3 = compress(hash_vector_3, *v.offset(5));
        hash_vector_4 = compress(hash_vector_4, *v.offset(6));
        hash_vector_4 = compress(hash_vector_4, *v.offset(7));

        v = v.add(UNROLL_FACTOR);
    }

    let mut hash_vector = compress(compress(hash_vector_1, hash_vector_2), compress(hash_vector_3, hash_vector_4));

    let end_address = v.add(remaining_blocks_count) as usize;

    while (v as usize) < end_address {
        hash_vector = compress(hash_vector, *v);
        v = v.add(1);
    }

    // Extract the last lane and use it for the hash value
    let mut result: [u32; 4] = [0; 4];
    _mm_storeu_si128(result.as_mut_ptr() as *mut __m128i, hash_vector);
    result[3]
}

#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn compress(a: __m128i, b: __m128i) -> __m128i {
    _mm_add_epi8(a, b)
}
