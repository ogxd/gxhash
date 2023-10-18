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
        let mut vec = vandq_s8(load_unaligned(p), mask);

        // To avoid collisions for zero right padded inputs, we mutate this vector using its used length
        vec = vaddq_s8(vec, vdupq_n_s8(len as i8));

        return vec;
    }

    #[inline]
    pub unsafe fn compress(a: state, b: state) -> state {
        ReinterpretUnion{ uint8: aes_encrypt_last(
            ReinterpretUnion{ int8: a }.uint8, 
            ReinterpretUnion{ int8: b }.uint8) }.int8
    }

    #[inline]
    // See https://blog.michaelbrase.com/2018/05/08/emulating-x86-aes-intrinsics-on-armv8-a
    unsafe fn aes_encrypt(data: uint8x16_t, keys: uint8x16_t) -> uint8x16_t {
        // Encrypt
        let encrypted = vaeseq_u8(data, vdupq_n_u8(0));
        // Mix columns
        let mixed = vaesmcq_u8(encrypted);
        // Xor keys
        veorq_u8(mixed, keys)
    }

    #[inline]
    // See https://blog.michaelbrase.com/2018/05/08/emulating-x86-aes-intrinsics-on-armv8-a
    unsafe fn aes_encrypt_last(data: uint8x16_t, keys: uint8x16_t) -> uint8x16_t {
        // Encrypt
        let encrypted = vaeseq_u8(data, vdupq_n_u8(0));
        // Xor keys
        veorq_u8(encrypted, keys)
    }

    #[inline]
    pub unsafe fn finalize(hash: state) -> u32 {
        // Hardcoded AES keys
        let salt1 = vld1q_u32([0x713B01D0, 0x8F2F35DB, 0xAF163956, 0x85459F85].as_ptr());
        let salt2 = vld1q_u32([0x1DE09647, 0x92CFA39C, 0x3DD99ACA, 0xB89C054F].as_ptr());
        let salt3 = vld1q_u32([0xC78B122B, 0x5544B1B7, 0x689D2B7D, 0xD0012E32].as_ptr());

        // 3 rounds of AES
        let mut hash = ReinterpretUnion{ int8: hash }.uint8;
        hash = aes_encrypt(hash, ReinterpretUnion{ uint32: salt1 }.uint8);
        hash = aes_encrypt(hash, ReinterpretUnion{ uint32: salt2 }.uint8);
        hash = aes_encrypt_last(hash, ReinterpretUnion{ uint32: salt3 }.uint8);
        let hash = ReinterpretUnion{ uint8: hash }.int8;

        // Truncate to output hash size
        let p = &hash as *const state as *const u32;
        *p
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
        let partial_vector: state;
        // Safety check
        if check_same_page(p) {
            let indices = _mm256_setr_epi8(
                0, 1, 2, 3, 4, 5, 6, 7,
                8, 9, 10, 11, 12, 13, 14, 15,
                16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31
            );
    
            let mask = _mm256_cmpgt_epi8(_mm256_set1_epi8(len as i8), indices);
            partial_vector = _mm256_and_si256(_mm256_loadu_si256(p), mask);
        } else {
             partial_vector = get_partial_safe(p as *const u8, len as usize)
        }
        // Prevents padded zeroes to introduce bias
        _mm256_add_epi32(partial_vector, _mm256_set1_epi32(len as i32))
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
    #[allow(overflowing_literals)]
    pub unsafe fn compress(a: state, b: state) -> state {
        let keys_1 = _mm256_set_epi32(0xFC3BC28E, 0x89C222E5, 0xB09D3E21, 0xF2784542, 0x4155EE07, 0xC897CCE2, 0x780AF2C3, 0x8A72B781);
        let keys_2 = _mm256_set_epi32(0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39136BD9, 0x7A83D76B, 0xB1E8F9F0, 0x028925A8, 0x3B9A4E71);

        // 2+1 rounds of AES for compression
        let mut b = _mm256_aesdec_epi128(b, keys_1);
        b = _mm256_aesdec_epi128(b, keys_2);
        return _mm256_aesdeclast_epi128(a, b);
    }

    #[inline]
    #[allow(overflowing_literals)]
    pub unsafe fn finalize(hash: state, seed: i32) -> state {
        // Hardcoded AES keys
        let keys_1 = _mm256_set_epi32(0x713B01D0, 0x8F2F35DB, 0xAF163956, 0x85459F85, 0xB49D3E21, 0xF2784542, 0x2155EE07, 0xC197CCE2);
        let keys_2 = _mm256_set_epi32(0x1DE09647, 0x92CFA39C, 0x3DD99ACA, 0xB89C054F, 0xCB6B2E9B, 0xC361DC58, 0x39136BD9, 0x7A83D76F);
        let keys_3 = _mm256_set_epi32(0xC78B122B, 0x5544B1B7, 0x689D2B7D, 0xD0012E32, 0xE2784542, 0x4155EE07, 0xC897CCE2, 0x780BF2C2);

        // 4 rounds of AES
        let mut hash = _mm256_aesdec_epi128(hash, _mm256_set1_epi32(seed));
        hash = _mm256_aesdec_epi128(hash, keys_1);
        hash = _mm256_aesdec_epi128(hash, keys_2);
        hash = _mm256_aesdeclast_epi128(hash, keys_3);

        // Merge the two 128 bit lanes entropy, so we can after safely truncate up to 128-bits
        let permuted = _mm256_permute2x128_si256(hash, hash, 0x21);
        _mm256_xor_si256(hash, permuted)
    }
}

use std::intrinsics::likely;

pub use platform_defs::*;

#[inline] // To be disabled when profiling
pub fn gxhash32(input: &[u8], seed: i32) -> u32 {
    unsafe {
        let p = &gxhash(input, seed) as *const state as *const u32;
        *p
    }
}

#[inline] // To be disabled when profiling
pub fn gxhash64(input: &[u8], seed: i32) -> u64 {
    unsafe {
        let p = &gxhash(input, seed) as *const state as *const u64;
        *p
    }
}

#[inline]
fn gxhash(input: &[u8], seed: i32) -> state {
    unsafe {
        const VECTOR_SIZE: isize = std::mem::size_of::<state>() as isize;
        
        let len: isize = input.len() as isize;

        let p = input.as_ptr() as *const i8;
        let mut v = p as *const state;

        // Quick exit
        if len <= VECTOR_SIZE {
            let partial_vector = get_partial(v, len);
            return finalize(partial_vector, seed);
        }

        let mut end_address: usize;
        let mut remaining_blocks_count: isize = len / VECTOR_SIZE;
        let mut hash_vector: state = create_empty();

        macro_rules! load_unaligned {
            ($($var:ident),+) => {
                $(
                    #[allow(unused_mut)]
                    let mut $var = load_unaligned(v);
                    v = v.offset(1);
                )+
            };
        }

        const UNROLL_FACTOR: isize = 8;
        if len >= VECTOR_SIZE * UNROLL_FACTOR {

            let unrollable_blocks_count: isize = (len / (VECTOR_SIZE * UNROLL_FACTOR)) * UNROLL_FACTOR; 
            end_address = v.offset(unrollable_blocks_count) as usize;
    
            load_unaligned!(s0, s1, s2, s3, s4, s5, s6, s7);
 
            while (v as usize) < end_address {
                
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
        
            let a = compress(compress(s0, s1), compress(s2, s3));
            let b = compress(compress(s4, s5), compress(s6, s7));
            hash_vector = compress(a, b);

            remaining_blocks_count -= unrollable_blocks_count;
        }

        end_address = v.offset(remaining_blocks_count) as usize;

        while likely((v as usize) < end_address) {
            load_unaligned!(v0);
            hash_vector = compress(hash_vector, v0);
        }

        let remaining_bytes = len & (VECTOR_SIZE - 1);
        if likely(remaining_bytes > 0) {
            let partial_vector = get_partial(v, remaining_bytes);
            hash_vector = compress(hash_vector, partial_vector);
        }

        finalize(hash_vector, seed)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::Rng;

    #[test]
    fn all_blocks_are_consumed() {
        let mut bytes = [42u8; 1200];

        let ref_hash = gxhash32(&bytes, 0);

        for i in 0..bytes.len() {
            let swap = bytes[i];
            bytes[i] = 82;
            let new_hash = gxhash32(&bytes, 0);
            bytes[i] = swap;

            assert_ne!(ref_hash, new_hash, "byte {i} not processed");
        }
    }

    #[test]
    fn add_zeroes_mutates_hash() {
        let mut bytes = [0u8; 1200];

        let mut rng = rand::thread_rng();
        rng.fill(&mut bytes[..32]);

        let mut ref_hash = 0;

        for i in 32..100 {
            let new_hash = gxhash32(&mut bytes[..i], 0);
            assert_ne!(ref_hash, new_hash, "Same hash at size {i} ({new_hash})");
            ref_hash = new_hash;
        }
    }

    #[test]
    fn hash_of_zero_is_not_zero() {
        assert_ne!(0, gxhash32(&[0u8; 0], 0));
        assert_ne!(0, gxhash32(&[0u8; 1], 0));
        assert_ne!(0, gxhash32(&[0u8; 1200], 0));
    }
}