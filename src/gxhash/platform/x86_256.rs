use core::arch::x86_64::*;

use super::*;

pub type State = __m256i;

#[inline(always)]
pub unsafe fn create_empty() -> State {
    _mm256_setzero_si256()
}

#[inline(always)]
pub unsafe fn create_seed(seed: i64) -> State {
    _mm256_set1_epi64x(seed)
}

#[inline(always)]
pub unsafe fn load_unaligned(p: *const State) -> State {
    _mm256_loadu_si256(p)
}

#[inline(always)]
pub unsafe fn get_partial(p: *const State, len: usize) -> State {
    // Safety check
    if check_same_page(p) {
        get_partial_unsafe(p, len as usize)
    } else {
        get_partial_safe(p, len as usize)
    }
}

#[inline(always)]
pub unsafe fn get_partial_safe(data: *const State, len: usize) -> State {
    // Temporary buffer filled with zeros
    let mut buffer = [0i8; VECTOR_SIZE];
    // Copy data into the buffer
    std::ptr::copy(data as *const i8, buffer.as_mut_ptr(), len);
    // Load the buffer into a __m256i vector
    let partial_vector = _mm256_loadu_si256(buffer.as_ptr() as *const State);
    _mm256_add_epi8(partial_vector, _mm256_set1_epi8(len as i8))
}

#[inline(always)]
pub unsafe fn get_partial_unsafe(data: *const State, len: usize) -> State {
    let indices = _mm256_set_epi8(31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let mask = _mm256_cmpgt_epi8(_mm256_set1_epi8(len as i8), indices);
    let partial_vector = _mm256_and_si256(_mm256_loadu_si256(data), mask);
    _mm256_add_epi8(partial_vector, _mm256_set1_epi8(len as i8))
}

#[inline(always)]
#[allow(overflowing_literals)]
pub unsafe fn compress(a: State, b: State) -> State {
    let keys_1 = _mm256_set_epi32(0xFC3BC28E, 0x89C222E5, 0xB09D3E21, 0xF2784542, 0x4155EE07, 0xC897CCE2, 0x780AF2C3, 0x8A72B781);
    let keys_2 = _mm256_set_epi32(0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39136BD9, 0x7A83D76B, 0xB1E8F9F0, 0x028925A8, 0x3B9A4E71);

    // 2+1 rounds of AES for compression
    let mut b = _mm256_aesenc_epi128(b, keys_1);
    b = _mm256_aesenc_epi128(b, keys_2);
    return _mm256_aesenclast_epi128(a, b);
}

#[inline(always)]
#[allow(overflowing_literals)]
pub unsafe fn compress_fast(a: State, b: State) -> State {
    return _mm256_aesenc_epi128(a, b);
}

#[inline(always)]
#[allow(overflowing_literals)]
pub unsafe fn finalize(hash: State) -> State {
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

#[inline(always)]
pub unsafe fn load_u8(x: u8) -> State {
    _mm256_set1_epi8(x as i8)
}

#[inline(always)]
pub unsafe fn load_u16(x: u16) -> State {
    _mm256_set1_epi16(x as i16)
}

#[inline(always)]
pub unsafe fn load_u32(x: u32) -> State {
    _mm256_set1_epi32(x as i32)
}

#[inline(always)]
pub unsafe fn load_u64(x: u64) -> State {
    _mm256_set1_epi64x(x as i64)
}

#[inline(always)]
pub unsafe fn load_u128(x: u128) -> State {
    let ptr = &x as *const u128 as *const __m128i;
    let s128 = _mm_loadu_si128(ptr);
    _mm256_set_m128i(s128, s128)
}

#[inline(always)]
pub unsafe fn load_i8(x: i8) -> State {
    _mm256_set1_epi8(x)
}

#[inline(always)]
pub unsafe fn load_i16(x: i16) -> State {
    _mm256_set1_epi16(x)
}

#[inline(always)]
pub unsafe fn load_i32(x: i32) -> State {
    _mm256_set1_epi32(x)
}

#[inline(always)]
pub unsafe fn load_i64(x: i64) -> State {
    _mm256_set1_epi64x(x)
}

#[inline(always)]
pub unsafe fn load_i128(x: i128) -> State {
    let ptr = &x as *const i128 as *const __m128i;
    let s128 = _mm_loadu_si128(ptr);
    _mm256_set_m128i(s128, s128)
}