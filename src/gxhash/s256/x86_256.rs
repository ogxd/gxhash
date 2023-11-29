use core::arch::x86_64::*;

use super::super::*;

pub struct Adapter256;

impl Adapter for Adapter256 {
    type State = __m256i;

    #[inline(always)]
    unsafe fn create_empty() -> __m256i {
        _mm256_setzero_si256()
    }

    #[inline(always)]
    unsafe fn create_seed(seed: i64) -> __m256i {
        _mm256_set1_epi64x(seed)
    }

    #[inline(always)]
    unsafe fn load_unaligned(p: *const __m256i) -> __m256i {
        _mm256_loadu_si256(p)
    }

    #[inline(always)]
    unsafe fn get_partial_safe(data: *const __m256i, len: usize) -> __m256i {
        // Temporary buffer filled with zeros
        let mut buffer = [0i8; Self::VECTOR_SIZE];
        // Copy data into the buffer
        std::ptr::copy(data as *const i8, buffer.as_mut_ptr(), len);
        // Load the buffer into a __m256i vector
        let partial_vector = _mm256_loadu_si256(buffer.as_ptr() as *const __m256i);
        _mm256_add_epi8(partial_vector, _mm256_set1_epi8(len as i8))
    }

    #[inline(always)]
    unsafe fn get_partial_unsafe(data: *const __m256i, len: usize) -> __m256i {
        let indices = _mm256_set_epi8(31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let mask = _mm256_cmpgt_epi8(_mm256_set1_epi8(len as i8), indices);
        let partial_vector = _mm256_and_si256(_mm256_loadu_si256(data), mask);
        _mm256_add_epi8(partial_vector, _mm256_set1_epi8(len as i8))
    }

    #[inline(always)]
    #[allow(overflowing_literals)]
    unsafe fn compress(a: __m256i, b: __m256i) -> __m256i {
        let keys_1 = _mm256_set_epi32(0xFC3BC28E, 0x89C222E5, 0xB09D3E21, 0xF2784542, 0x4155EE07, 0xC897CCE2, 0x780AF2C3, 0x8A72B781);
        let keys_2 = _mm256_set_epi32(0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39136BD9, 0x7A83D76B, 0xB1E8F9F0, 0x028925A8, 0x3B9A4E71);

        // 2+1 rounds of AES for compression
        let mut b = _mm256_aesenc_epi128(b, keys_1);
        b = _mm256_aesenc_epi128(b, keys_2);
        return _mm256_aesenclast_epi128(a, b);
    }

    #[inline(always)]
    #[allow(overflowing_literals)]
    unsafe fn compress_fast(a: __m256i, b: __m256i) -> __m256i {
        return _mm256_aesenc_epi128(a, b);
    }

    #[inline(always)]
    #[allow(overflowing_literals)]
    unsafe fn finalize(hash: __m256i) -> __m256i {
        // Hardcoded AES keys
        let keys_1 = _mm256_set_epi32(0x713B01D0, 0x8F2F35DB, 0xAF163956, 0x85459F85, 0xB49D3E21, 0xF2784542, 0x2155EE07, 0xC197CCE2);
        let keys_2 = _mm256_set_epi32(0x1DE09647, 0x92CFA39C, 0x3DD99ACA, 0xB89C054F, 0xCB6B2E9B, 0xC361DC58, 0x39136BD9, 0x7A83D76F);
        let keys_3 = _mm256_set_epi32(0xC78B122B, 0x5544B1B7, 0x689D2B7D, 0xD0012E32, 0xE2784542, 0x4155EE07, 0xC897CCE2, 0x780BF2C2);

        // 4 rounds of AES
        let mut hash = _mm256_aesenc_epi128(hash, keys_1);
        hash = _mm256_aesenc_epi128(hash, keys_2);
        hash = _mm256_aesenclast_epi128(hash, keys_3);

        let permuted = _mm256_permute2x128_si256(hash, hash, 0x21);
        _mm256_xor_si256(hash, permuted)
    }
}