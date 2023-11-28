use core::arch::x86_64::*;

use super::super::*;

pub struct Adapter128;

impl Adapter for Adapter128 {
    type State = __m128i;

    #[inline(always)]
    unsafe fn create_empty() -> __m128i {
        _mm_setzero_si128()
    }
    
    #[inline(always)]
    unsafe fn create_seed(seed: i64) -> __m128i {
        _mm_set1_epi64x(seed)
    }
    
    #[inline(always)]
    unsafe fn load_unaligned(p: *const __m128i) -> __m128i {
        _mm_loadu_si128(p)
    }
    
    #[inline(always)]
    unsafe fn get_partial_safe(data: *const __m128i, len: usize) -> __m128i {
        // Temporary buffer filled with zeros
        let mut buffer = [0i8; Self::VECTOR_SIZE];
        // Copy data into the buffer
        std::ptr::copy(data as *const i8, buffer.as_mut_ptr(), len);
        // Load the buffer into a __m256i vector
        let partial_vector = _mm_loadu_si128(buffer.as_ptr() as *const __m128i);
        _mm_add_epi8(partial_vector, _mm_set1_epi8(len as i8))
    }
    
    #[inline(always)]
    unsafe fn get_partial_unsafe(data: *const __m128i, len: usize) -> __m128i {
        let indices = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let mask = _mm_cmpgt_epi8(_mm_set1_epi8(len as i8), indices);
        let partial_vector = _mm_and_si128(_mm_loadu_si128(data), mask);
        _mm_add_epi8(partial_vector, _mm_set1_epi8(len as i8))
    }
    
    #[inline(always)]
    #[allow(overflowing_literals)]
    unsafe fn compress(a: __m128i, b: __m128i) -> __m128i {
        let keys_1 = _mm_set_epi32(0xF2784542, 0xB09D3E21, 0x89C222E5, 0xFC3BC28E);
        let keys_2 = _mm_set_epi32(0x39136BD9, 0xB361DC58, 0xCB6B2E9B, 0x03FCE279);
    
        // 2+1 rounds of AES for compression
        let mut b = _mm_aesenc_si128(b, keys_1);
        b = _mm_aesenc_si128(b, keys_2);
        _mm_aesenclast_si128(a, b)
    }
    
    #[inline(always)]
    #[allow(overflowing_literals)]
    unsafe fn compress_fast(a: __m128i, b: __m128i) -> __m128i {
        _mm_aesenc_si128(a, b)
    }
    
    #[inline(always)]
    #[allow(overflowing_literals)]
    unsafe fn finalize(hash: __m128i) -> __m128i {
        // Hardcoded AES keys
        let keys_1 = _mm_set_epi32(0x85459F85, 0xAF163956, 0x8F2F35DB, 0x713B01D0);
        let keys_2 = _mm_set_epi32(0xB89C054F, 0x3DD99ACA, 0x92CFA39C, 0x1DE09647);
        let keys_3 = _mm_set_epi32(0xD0012E32, 0x689D2B7D, 0x5544B1B7, 0xC78B122B);
    
        // 4 rounds of AES
        let mut hash = _mm_aesenc_si128(hash, keys_1);
        hash = _mm_aesenc_si128(hash, keys_2);
        hash = _mm_aesenclast_si128(hash, keys_3);
    
        hash
    }
}