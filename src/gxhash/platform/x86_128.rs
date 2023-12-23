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
pub unsafe fn compress_1(a: State, b: State) -> State {
    let keys_1 = _mm_set_epi32(0xF2784542, 0xB09D3E21, 0x89C222E5, 0xFC3BC28E);

    let mut b = _mm_aesenc_si128(b, keys_1);
    _mm_aesenclast_si128(a, b)
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

#[inline(always)]
pub unsafe fn load_u8(x: u8) -> State {
    _mm_set1_epi8(x as i8)
}

#[inline(always)]
pub unsafe fn load_u16(x: u16) -> State {
    _mm_set1_epi16(x as i16)
}

#[inline(always)]
pub unsafe fn load_u32(x: u32) -> State {
    _mm_set1_epi32(x as i32)
}

#[inline(always)]
pub unsafe fn load_u64(x: u64) -> State {
    _mm_set1_epi64x(x as i64)
}

#[inline(always)]
pub unsafe fn load_u128(x: u128) -> State {
    let ptr = &x as *const u128 as *const State;
    _mm_loadu_si128(ptr)
}

#[inline(always)]
pub unsafe fn load_i8(x: i8) -> State {
    _mm_set1_epi8(x)
}

#[inline(always)]
pub unsafe fn load_i16(x: i16) -> State {
    _mm_set1_epi16(x)
}

#[inline(always)]
pub unsafe fn load_i32(x: i32) -> State {
    _mm_set1_epi32(x)
}

#[inline(always)]
pub unsafe fn load_i64(x: i64) -> State {
    _mm_set1_epi64x(x)
}

#[inline(always)]
pub unsafe fn load_i128(x: i128) -> State {
    let ptr = &x as *const i128 as *const State;
    _mm_loadu_si128(ptr)
}