// For ARM architecture
#[cfg(target_arch = "aarch64")]
mod platform_defs {
    use std::mem;
    use core::arch::aarch64::*;

    pub type state = int8x16_t;

    #[inline]
    pub unsafe fn create_empty() -> state {
        vdupq_n_s8(0)
    }
    
    #[inline]
    pub unsafe fn prefetch(p: *const state) {
        __pld(p as *const i8);
    }

    #[inline]
    pub unsafe fn compress(a: state, b: state) -> state {
        let sum: state = vaddq_s8(a, b);
        vextq_s8(sum, sum, 1) 
    }

    #[inline]
    pub unsafe fn mix(hash: state) -> state {
        hash
    }

    #[inline]
    pub unsafe fn fold(hash: state) -> u32 {
        // Bit-cast the int8x16_t to uint32x4_t
        let vec_u32: uint32x4_t = mem::transmute(hash);
        // Get the first u32 value from the vector
        vgetq_lane_u32(vec_u32, 3)
    }
}

// For x86 architecture
#[cfg(target_arch = "x86_64")]
mod platform_defs {
    use core::arch::x86_64::*;

    pub type state = __m256i;

    #[inline]
    pub unsafe fn create_empty() -> state {
        _mm256_setzero_si256()
    }

    #[inline]
    pub unsafe fn prefetch(p: *const state) {
        _mm_prefetch(p as *const i8, 3);
    }

    #[inline]
    pub unsafe fn compress(a: state, b: state) -> state {
        let sum: state = _mm256_add_epi8(a, b);
        _mm256_alignr_epi8(sum, sum, 1) 
    }

    #[inline]
    pub unsafe fn mix(hash: state) -> state {
        hash
    }

    #[inline]
    pub unsafe fn fold(hash: state) -> u32 {
        let p = &hash as *const state as *const u32;
        *p + *p.offset(1) + *p.offset(2) + *p.offset(3) + *p.offset(4) + *p.offset(5) + *p.offset(6)+ *p.offset(7)

        // let mut result: [u32; 8] = [0; 8];
        // _mm256_storeu_si256(result.as_mut_ptr() as *mut state, hash);
        // result[7]

        // let sum128 = _mm_add_epi32( 
        //     _mm256_castsi256_si128(hash),
        //     _mm256_extracti128_si256(hash, 1)); // silly GCC uses a longer AXV512VL instruction if AVX512 is enabled :/
        // _mm128_hadd_(sum128);
    }
}

pub use platform_defs::*;

#[inline]
pub fn gxhash(input: &[u8]) -> u32 {
    unsafe {
        const VECTOR_SIZE: isize = std::mem::size_of::<state>() as isize;
        const UNROLL_FACTOR: isize = 8;
    
        let len: isize = input.len() as isize;
    
        //let remaining_bytes = len & (VECTOR_SIZE - 1);
    
        let p = input.as_ptr();
        let mut v = p as *const state;
        let mut end_address: usize;// = v.add(unrollable_blocks_count) as usize;

        //prefetch(v);
    
        let mut hash_vector: state = create_empty();

        if len >= VECTOR_SIZE * UNROLL_FACTOR {

            let unrollable_blocks_count: isize = (len / (VECTOR_SIZE * UNROLL_FACTOR)) * UNROLL_FACTOR; 
            end_address = v.offset(unrollable_blocks_count) as usize;
    
            let mut hash_vector_1: state = create_empty();
            let mut hash_vector_2: state = create_empty();
            let mut hash_vector_3: state = create_empty();
            let mut hash_vector_4: state = create_empty();
            let mut hash_vector_5: state = create_empty();
            let mut hash_vector_6: state = create_empty();
            let mut hash_vector_7: state = create_empty();
            let mut hash_vector_8: state = create_empty();

            while (v as usize) < end_address {

                hash_vector_1 = compress(hash_vector_1, *v);
                hash_vector_2 = compress(hash_vector_2, *v.offset(1));
                hash_vector_3 = compress(hash_vector_3, *v.offset(2));
                hash_vector_4 = compress(hash_vector_4, *v.offset(3));
                hash_vector_5 = compress(hash_vector_5, *v.offset(4));
                hash_vector_6 = compress(hash_vector_6, *v.offset(5));
                hash_vector_7 = compress(hash_vector_7, *v.offset(6));
                hash_vector_8 = compress(hash_vector_8, *v.offset(7));
        
                v = v.offset(UNROLL_FACTOR);
            }
        
            hash_vector = compress(compress(compress(compress(compress(compress(compress(hash_vector_1, hash_vector_2), hash_vector_3), hash_vector_4), hash_vector_5), hash_vector_6), hash_vector_7), hash_vector_8);
            let remaining_blocks_count: isize = (len / VECTOR_SIZE) - unrollable_blocks_count;
            end_address = v.offset(remaining_blocks_count) as usize;
        }
        else
        {
            end_address = v.offset(len / VECTOR_SIZE) as usize;
        }
    
        while (v as usize) < end_address {
            hash_vector = compress(hash_vector, *v);
            v = v.offset(1);
        }

        fold(mix(hash_vector))
    }
}