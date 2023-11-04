use core::arch::x86_64::*;
use std::mem::size_of;

pub type State = __m128i;

#[inline]
pub unsafe fn create_empty() -> State {
    _mm_setzero_si128()
}

#[inline]
pub unsafe fn prefetch(p: *const State) {
    _mm_prefetch(p as *const i8, 3);
}

#[inline]
pub unsafe fn load_unaligned(p: *const State) -> State {
    _mm_loadu_si128(p)
}

#[inline]
pub unsafe fn get_partial(p: *const State, len: isize) -> State {
    let partial_vector: State;
    // Safety check
    if check_same_page(p) {
        let indices = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let mask = _mm_cmpgt_epi8(_mm_set1_epi8(len as i8), indices);
        partial_vector = _mm_and_si128(_mm_loadu_si128(p), mask);
    } else {
        partial_vector = get_partial_safe(p as *const u8, len as usize)
    }
    // Prevents padded zeroes to introduce bias
    _mm_add_epi8(partial_vector, _mm_set1_epi8(len as i8))
}

// 4KiB is the default page size for most systems, and conservative for other systems such as MacOS ARM (16KiB)
const PAGE_SIZE: usize = 0x1000;

#[inline]
unsafe fn check_same_page(ptr: *const State) -> bool {
    let address = ptr as usize;
    // Mask to keep only the last 12 bits (3 bytes)
    let offset_within_page = address & 0xFFF;
    // Check if the 32nd byte from the current offset exceeds the page boundary
    offset_within_page <= PAGE_SIZE - size_of::<State>()
}

#[inline]
unsafe fn get_partial_safe(data: *const u8, len: usize) -> State {
    // Temporary buffer filled with zeros
    let mut buffer = [0u8; size_of::<State>()];
    // Copy data into the buffer
    std::ptr::copy(data, buffer.as_mut_ptr(), len);
    // Load the buffer into a __m256i vector
    _mm_loadu_si128(buffer.as_ptr() as *const State)
}

#[inline]
#[allow(overflowing_literals)]
pub unsafe fn compress(a: State, b: State) -> State {
    let keys_1 = _mm_set_epi32(0xFC3BC28E, 0x89C222E5, 0xB09D3E21, 0xF2784542);
    let keys_2 = _mm_set_epi32(0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39136BD9);

    // 2+1 rounds of AES for compression
    let mut b = _mm_aesenc_si128(b, keys_1);
    b = _mm_aesenc_si128(b, keys_2);
    return _mm_aesenclast_si128(a, b);
}

#[inline]
#[allow(overflowing_literals)]
pub unsafe fn compress_fast(a: State, b: State) -> State {
    return _mm_aesenc_si128(a, b);
}

#[inline]
#[allow(overflowing_literals)]
pub unsafe fn finalize(hash: State, seed: i32) -> State {
    // Hardcoded AES keys
    let keys_1 = _mm_set_epi32(0x713B01D0, 0x8F2F35DB, 0xAF163956, 0x85459F85);
    let keys_2 = _mm_set_epi32(0x1DE09647, 0x92CFA39C, 0x3DD99ACA, 0xB89C054F);
    let keys_3 = _mm_set_epi32(0xC78B122B, 0x5544B1B7, 0x689D2B7D, 0xD0012E32);

    // 4 rounds of AES
    //let mut hash = _mm_aesenc_si128(hash, _mm_set1_epi32(seed));
    let mut hash = _mm_add_epi8(hash, _mm_set1_epi32(seed));
    hash = _mm_aesenc_si128(hash, keys_1);
    hash = _mm_aesenc_si128(hash, keys_2);
    hash = _mm_aesenclast_si128(hash, keys_3);

    hash
}