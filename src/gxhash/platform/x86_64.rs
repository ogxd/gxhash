use core::arch::x86_64::*;

use super::*;

pub type State = __m128i;

#[inline(always)]
pub unsafe fn create_empty() -> State {
    _mm_setzero_si128()
}

#[inline(always)]
pub unsafe fn create_seed(seed: i64) -> State {
    _mm_set1_epi64x(seed)
}

#[inline(always)]
pub unsafe fn load_unaligned(p: *const State) -> State {
    _mm_loadu_si128(p)
}

#[inline(always)]
pub unsafe fn get_partial(p: *const State, len: usize) -> State {
    // Safety check
    if check_same_page(p) {
        get_partial_unsafe(p, len)
    } else {
        get_partial_safe(p, len)
    }
}

#[inline(always)]
pub unsafe fn get_partial_safe(data: *const State, len: usize) -> State {
    // Temporary buffer filled with zeros
    let mut buffer = [0i8; VECTOR_SIZE];
    // Copy data into the buffer
    std::ptr::copy(data as *const i8, buffer.as_mut_ptr(), len);
    // Load the buffer into a __m256i vector
    let partial_vector = _mm_loadu_si128(buffer.as_ptr() as *const State);
    _mm_add_epi8(partial_vector, _mm_set1_epi8(len as i8))
}

#[inline(always)]
pub unsafe fn get_partial_unsafe(data: *const State, len: usize) -> State {
    let indices = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let mask = _mm_cmpgt_epi8(_mm_set1_epi8(len as i8), indices);
    let partial_vector = _mm_and_si128(_mm_loadu_si128(data), mask);
    _mm_add_epi8(partial_vector, _mm_set1_epi8(len as i8))
}

#[inline(always)]
#[allow(overflowing_literals)]
pub unsafe fn compress(a: State, b: State) -> State {
    let keys_1 = _mm_set_epi32(0xF2784542, 0xB09D3E21, 0x89C222E5, 0xFC3BC28E);
    let keys_2 = _mm_set_epi32(0x39136BD9, 0xB361DC58, 0xCB6B2E9B, 0x03FCE279);

    // 2+1 rounds of AES for compression
    let mut b = _mm_aesenc_si128(b, keys_1);
    b = _mm_aesenc_si128(b, keys_2);
    _mm_aesenclast_si128(a, b)
}

#[inline(always)]
#[allow(overflowing_literals)]
pub unsafe fn compress_fast(a: State, b: State) -> State {
    _mm_aesenc_si128(a, b)
}

#[inline(always)]
#[allow(overflowing_literals)]
pub unsafe fn finalize(hash: State) -> State {
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

#[cfg(not(target_feature = "avx2"))]
#[inline(always)]
pub unsafe fn compress_8(mut ptr: *const State, end_address: usize, hash_vector: State) -> State {
    let mut h1 = create_empty();
    let mut h2 = create_empty();
    while (ptr as usize) < end_address {

        crate::gxhash::load_unaligned!(ptr, v0, v1, v2, v3, v4, v5, v6, v7);

        let mut tmp1: State;
        tmp1 = compress_fast(v0, v2);
        tmp1 = compress_fast(tmp1, v4);
        tmp1 = compress_fast(tmp1, v6);
        h1 = compress(h1, tmp1);

        let mut tmp2: State;
        tmp2 = compress_fast(v1, v3);
        tmp2 = compress_fast(tmp2, v5);
        tmp2 = compress_fast(tmp2, v7);
        h2 = compress(h2, tmp2);
    }
    compress(hash_vector, compress(h1, h2))
}

#[cfg(target_feature = "avx2")]
#[inline(always)]
#[allow(overflowing_literals)]
pub unsafe fn compress_x2(a: __m256i, b: __m256i) -> __m256i {
    let keys_1 = _mm256_set_epi32(0xF2784542, 0xB09D3E21, 0x89C222E5, 0xFC3BC28E, 0xF2784542, 0xB09D3E21, 0x89C222E5, 0xFC3BC28E);
    let keys_2 = _mm256_set_epi32(0x39136BD9, 0xB361DC58, 0xCB6B2E9B, 0x03FCE279, 0x39136BD9, 0xB361DC58, 0xCB6B2E9B, 0x03FCE279);

    // 2+1 rounds of AES for compression
    let mut b = _mm256_aesenc_epi128(b, keys_1);
    b = _mm256_aesenc_epi128(b, keys_2);
    return _mm256_aesenclast_epi128(a, b);
}

#[cfg(target_feature = "avx2")]
#[inline(always)]
#[allow(overflowing_literals)]
pub unsafe fn compress_fast_x2(a: __m256i, b: __m256i) -> __m256i {
    return _mm256_aesenc_epi128(a, b);
}

#[cfg(target_feature = "avx2")]
#[inline(always)]
pub unsafe fn compress_8(mut ptr: *const State, end_address: usize, hash_vector: State) -> State {
    let mut ptr = ptr as *const __m256i;
    let mut h = _mm256_setzero_si256();
    while (ptr as usize) < end_address {

        crate::gxhash::load_unaligned!(ptr, v0, v1, v2, v3);

        let mut tmp: __m256i;
        tmp = compress_fast(v0, v1);
        tmp = compress_fast(tmp, v2);
        tmp = compress_fast(tmp, v3);
        h = compress(h, tmp);
    }
    
    // Extract the two 128-bit lanes
    let h1 = _mm256_castsi256_si128(h);
    let h2 = _mm256_extracti128_si256(h, 1);

    compress(hash_vector, compress(h1, h2))
}