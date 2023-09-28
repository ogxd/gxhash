// For ARM architecture
#[cfg(target_arch = "aarch64")]
mod platform_defs {
    use std::mem::size_of;
    use core::arch::aarch64::*;

    pub type state = int8x16_t;

    #[repr(C)]
    union ReinterpretUnion {
        int64: int64x2_t,
        int32: int32x4_t,
        uint32: uint32x4_t,
        int8: int8x16_t,
        uint8: uint8x16_t,
    }

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
        const MASK: [u8; size_of::<state>() * 2] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];

        let mask = vld1q_s8((MASK.as_ptr() as *const i8).offset(size_of::<state>() as isize - len));
        vandq_s8(load_unaligned(p), mask)
    }

    #[inline]
    pub unsafe fn compress(a: state, b: state) -> state {
        let sum: state = vaddq_s8(a, b);
        vextq_s8(sum, sum, 1) 
    }

    #[inline]
    pub unsafe fn mix(hash: state) -> state {
        let salt = vcombine_s64(vcreate_s64(4860325414534694371), vcreate_s64(8120763769363581797));
        let keys = vmulq_s32(
            ReinterpretUnion { int64: salt }.int32,
            ReinterpretUnion { int8: hash }.int32);
        let a = vaeseq_u8(ReinterpretUnion { int8: hash }.uint8, vdupq_n_u8(0));
        let b = vaesmcq_u8(a);
        let c = veorq_u8(b, ReinterpretUnion{ int32: keys }.uint8);
        ReinterpretUnion{ uint8: c }.int8
    }

    #[inline]
    pub unsafe fn fold(hash: state) -> u32 {
        // Bit-cast the int8x16_t to uint32x4_t
        let vec_u32: uint32x4_t = ReinterpretUnion { int8: hash }.uint32;
        // Get the first u32 value from the vector
        vgetq_lane_u32(vec_u32, 3)
    }
}

// For x86 architecture
#[cfg(target_arch = "x86_64")]
mod platform_defs {
    use core::arch::x86_64::*;
    use std::mem::size_of;

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
        const MASK: [u8; size_of::<state>() * 2] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];

        // Safety check
        if check_same_page(p) {
            let mask = _mm256_loadu_epi8((MASK.as_ptr() as *const i8).offset(32 - len));
            _mm256_and_si256(_mm256_loadu_si256(p), mask)
        } else {
            get_partial_safe(p as *const u8, len as usize)
        }
    }

    #[inline]
    unsafe fn check_same_page(ptr: *const state) -> bool {
        let address = ptr as usize;
        // Mask to keep only the last 12 bits (3 bytes)
        let offset_within_page = address & 0xFFF;
        // Check if the 32nd byte from the current offset exceeds the page boundary
        offset_within_page <= (4096 - size_of::<state>() - 1)
    }

    #[inline]
    unsafe fn get_partial_safe(data: *const u8, len: usize) -> state {
        // Temporary buffer filled with zeros
        let mut buffer = [0u8; size_of::<state>()];
        // Copy data into the buffer
        std::ptr::copy(data, buffer.as_mut_ptr(), len);
        // Load the buffer into a __m256i vector
        _mm256_loadu_si256(buffer.as_ptr() as *const state)
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
        (*p).wrapping_add(*p.offset(1))
            .wrapping_add(*p.offset(2))
            .wrapping_add(*p.offset(3))
            .wrapping_add(*p.offset(4))
            .wrapping_add(*p.offset(5))
            .wrapping_add(*p.offset(6))
            .wrapping_add(*p.offset(7))
    }
}

use std::intrinsics::likely;

pub use platform_defs::*;

#[cfg(test)]
pub static mut COUNTERS : Vec<usize> = vec![];

#[inline]
pub fn gxhash(input: &[u8]) -> u32 {
    unsafe {
        const VECTOR_SIZE: isize = std::mem::size_of::<state>() as isize;
        const UNROLL_FACTOR: isize = 8;
    
        let len: isize = input.len() as isize;
    
        let remaining_bytes = len & (VECTOR_SIZE - 1);
    
        let p = input.as_ptr() as *const i8;
        let mut v = p as *const state;
        let mut end_address: usize;

        let mut hash_vector: state = create_empty();

        macro_rules! count_for_tests {
            () => {
                #[cfg(test)]
                {
                    let index = ((v as usize) - (p as usize)) / VECTOR_SIZE as usize;
                    COUNTERS.push(index);
                }
            };
        }
        
        macro_rules! load_unaligned {
            ($($var:ident),+) => {
                $(
                    #[allow(unused_mut)]
                    let mut $var = load_unaligned(v);
                    v = v.offset(1);
                )+
            };
        }

        if len >= VECTOR_SIZE * UNROLL_FACTOR {

            let unrollable_blocks_count: isize = (len / (VECTOR_SIZE * UNROLL_FACTOR)) * UNROLL_FACTOR; 
            end_address = v.offset(unrollable_blocks_count) as usize;
    
            count_for_tests!();
            load_unaligned!(s0, s1, s2, s3, s4, s5, s6, s7);
 
            while (v as usize) < end_address {
                
                count_for_tests!();
                load_unaligned!(v0, v1, v2, v3, v4, v5, v6, v7);

                prefetch(v);

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
            count_for_tests!();
            load_unaligned!(v0);
            hash_vector = compress(hash_vector, v0);
        }

        if likely(remaining_bytes > 0) {
            count_for_tests!();
            let partial_vector = get_partial(v, remaining_bytes);
            hash_vector = compress(hash_vector, partial_vector);
        }

        fold(mix(hash_vector))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn all_blocks_are_consumed() {
        let expected: [usize; 10] = [0, 8, 16, 24, 32, 33, 34, 35, 36, 37];
        let mut rng = rand::thread_rng();
        let mut random_bytes = [0u8; 1200];
        rng.fill(&mut random_bytes[..]);
    
        unsafe
        {
            COUNTERS.clear();
            let hash = gxhash(&random_bytes);
            assert_ne!(0, hash);

            // cargo test -- --nocapture
            println!("{:?}", &COUNTERS);
            assert!(COUNTERS.as_slice() == expected.as_slice());
            
        }
    }

    #[test]
    fn hash_of_zero_is_not_zero() {
        let zero_bytes = [0u8; 1200];

        let hash = gxhash(&zero_bytes);
        assert_ne!(0, hash);
    }
}