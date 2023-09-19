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
        //__pld(p as *const i8);
    }

    #[inline]
    pub unsafe fn load_unaligned(p: *const state) -> state {
        vld1q_s8(p as *const i8)
    }

    #[inline]
    pub unsafe fn get_partial(p: *const state, len: isize) -> state {
        const MASK: [u8; 32] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];

        let mask = vld1q_s8((MASK.as_ptr() as *const i8).offset(16 - len));
        vandq_s8(load_unaligned(p), mask)
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
    pub unsafe fn load_unaligned(p: *const state) -> state {
        _mm256_loadu_si256(p)
    }

    #[inline]
    pub unsafe fn get_partial(p: *const state, len: isize) -> state {
        const MASK: [u8; 64] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];

        let mask = _mm256_loadu_epi8((MASK.as_ptr() as *const i8).offset(32 - len));
        _mm256_and_si256(_mm256_loadu_si256(p), mask)
    }

    #[inline]
    pub unsafe fn compress(a: state, b: state) -> state {
        let sum: state = _mm256_add_epi8(a, b);
        _mm256_alignr_epi8(sum, sum, 1) 
    }

    #[inline]
    pub unsafe fn mix(hash: state) -> state {
        let salt = _mm256_set_epi64x(-4860325414534694371, 8120763769363581797, -4860325414534694371, 8120763769363581797);
        let keys = _mm256_mul_epu32(salt, hash);
        _mm256_aesenc_epi128(hash, keys)

    }

    #[inline]
    pub unsafe fn fold(hash: state) -> u32 {
        let p = &hash as *const state as *const u32;
        *p + *p.offset(1) + *p.offset(2) + *p.offset(3) + *p.offset(4) + *p.offset(5) + *p.offset(6)+ *p.offset(7)
    }
}

use std::intrinsics::likely;

pub use platform_defs::*;

macro_rules! load_unaligned {
    ($v:expr, $($var:ident),+) => {
        $(
            let mut $var = load_unaligned($v);
            $v = $v.offset(1);
        )+
    };
}

#[inline]
pub fn gxhash(input: &[u8]) -> u32 {
    unsafe {
        const VECTOR_SIZE: isize = std::mem::size_of::<state>() as isize;
        const UNROLL_FACTOR: isize = 8;
    
        let len: isize = input.len() as isize;
    
        let remaining_bytes = len & (VECTOR_SIZE - 1);
    
        let p = input.as_ptr() as *const i8;
        let mut v = p as *const state;
        let mut end_address: usize;// = v.add(unrollable_blocks_count) as usize;

        //prefetch(v);
    
        let mut hash_vector: state = create_empty();

        if len >= VECTOR_SIZE * UNROLL_FACTOR {

            let unrollable_blocks_count: isize = (len / (VECTOR_SIZE * UNROLL_FACTOR)) * UNROLL_FACTOR; 
            end_address = v.offset(unrollable_blocks_count) as usize;
    
            load_unaligned!(v, s0, s1, s2, s3, s4, s5, s6, s7);
 
            while (v as usize) < end_address {

                load_unaligned!(v, v0, v1, v2, v3, v4, v5, v6, v7);

                s0 = compress(s0, v0);
                s1 = compress(s1, v1);
                s2 = compress(s2, v2);
                s3 = compress(s3, v3);
                s4 = compress(s4, v4);
                s5 = compress(s5, v5);
                s6 = compress(s6, v6);
                s7 = compress(s7, v7);
            }
        
            hash_vector = compress(compress(compress(compress(compress(compress(compress(s0, s1), s2), s3), s4), s5), s6), s7);

            let remaining_blocks_count: isize = (len / VECTOR_SIZE) - unrollable_blocks_count;
            end_address = v.offset(remaining_blocks_count) as usize;
        }
        else
        {
            end_address = v.offset(len / VECTOR_SIZE) as usize;
        }

        while (v as usize) < end_address {
            load_unaligned!(v, v0);
            hash_vector = compress(hash_vector, v0);
        }

        if likely(remaining_bytes > 0) {
            let partial_vector = get_partial(v, remaining_bytes);
            hash_vector = compress(hash_vector, partial_vector);
        }

        fold(mix(hash_vector))
    }
}