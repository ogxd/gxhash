#[cfg(not(any(all(target_feature = "aes", target_feature = "sse2"), docsrs)))] // docs.rs bypasses the target_feature check
compile_error!{"Gxhash requires aes and sse2 intrinsics. Make sure the processor supports it and build with RUSTFLAGS=\"-C target-cpu=native\" or RUSTFLAGS=\"-C target-feature=+aes,+sse2\"."}

#[cfg(all(feature = "hybrid", not(all(target_feature = "aes", target_feature = "avx2"))))]
compile_error!{"Hybrid feature is only available on x86 processors with aes and avx2 intrinsics."}

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
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

// Rarely called, it's worth not inlining it to reduce code size
#[inline(never)]
pub unsafe fn get_partial_safe(data: *const State, len: usize) -> State {
    // Temporary buffer filled with zeros
    let mut buffer = [0i8; VECTOR_SIZE];
    core::ptr::copy(data as *const i8, buffer.as_mut_ptr(), len);
    let partial_vector = _mm_loadu_si128(buffer.as_ptr() as *const State);
    _mm_add_epi8(partial_vector, _mm_set1_epi8(len as i8))
}

#[inline(always)]
pub unsafe fn get_partial_unsafe(data: *const State, len: usize) -> State {
    // Using inline assembly to load out-of-bounds
    use std::arch::asm;
    let indices = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let mask = _mm_cmpgt_epi8(_mm_set1_epi8(len as i8), indices);
    let mut result: State;
    asm!("movdqu {0}, [{1}]", out(xmm_reg) result, in(reg) data, options(pure, nomem, nostack));
    let partial_vector = _mm_and_si128(result, mask);
    _mm_add_epi8(partial_vector, _mm_set1_epi8(len as i8))
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn aes_encrypt(data: State, keys: State) -> State {
    _mm_aesenc_si128(data, keys)
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn aes_encrypt_last(data: State, keys: State) -> State {
    _mm_aesenclast_si128(data, keys)
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn ld(array: *const u32) -> State {
    _mm_loadu_si128(array as *const State)
}

#[cfg(not(feature = "hybrid"))]
#[inline(always)]
pub unsafe fn compress_8(mut ptr: *const State, whole_vector_count: usize, hash_vector: State, len: usize) -> (State, *const State, usize) {

    let end_address = ptr.add((whole_vector_count / 8) * 8) as usize;

    // Disambiguation vectors
    let mut t1: State = create_empty();
    let mut t2: State = create_empty();

    // Hash is processed in two separate 128-bit parallel lanes
    // This allows the same processing to be applied using 256-bit V-AES intrinsics
    // so that hashes are stable in both cases. 
    let mut lane1 = hash_vector;
    let mut lane2 = hash_vector;

    while (ptr as usize) < end_address {

        crate::gxhash::load_unaligned!(ptr, v0, v1, v2, v3, v4, v5, v6, v7);

        let mut tmp1 = aes_encrypt(v0, v2);
        let mut tmp2 = aes_encrypt(v1, v3);

        tmp1 = aes_encrypt(tmp1, v4);
        tmp2 = aes_encrypt(tmp2, v5);

        tmp1 = aes_encrypt(tmp1, v6);
        tmp2 = aes_encrypt(tmp2, v7);

        t1 = _mm_add_epi8(t1, ld(KEYS.as_ptr()));
        t2 = _mm_add_epi8(t2, ld(KEYS.as_ptr().offset(4)));

        lane1 = aes_encrypt_last(aes_encrypt(tmp1, t1), lane1);
        lane2 = aes_encrypt_last(aes_encrypt(tmp2, t2), lane2);
    }
    // For 'Zeroes' test
    let len_vec =  _mm_set1_epi32(len as i32);
    lane1 = _mm_add_epi8(lane1, len_vec);
    lane2 = _mm_add_epi8(lane2, len_vec);

    // Merge lanes
    (aes_encrypt(lane1, lane2), ptr, whole_vector_count % 8)
}

#[cfg(feature = "hybrid")]
#[inline(always)]
pub unsafe fn compress_8(ptr: *const State, whole_vector_count: usize, hash_vector: State, len: usize) -> (State, *const State, usize) {
    macro_rules! load_unaligned_x2 {
        ($ptr:ident, $($var:ident),+) => {
            $(
                #[allow(unused_mut)]
                let mut $var = _mm256_loadu_si256($ptr);
                $ptr = ($ptr).offset(1);
            )+
        };
    }

    let end_address = ptr.add((whole_vector_count / 8) * 8) as usize;
    
    let mut ptr = ptr as *const __m256i;
    let mut t = _mm256_setzero_si256();
    let mut lane = _mm256_set_m128i(hash_vector, hash_vector);
    while (ptr as usize) < end_address {

        load_unaligned_x2!(ptr, v0, v1, v2, v3);

        let mut tmp = _mm256_aesenc_epi128(v0, v1);
        tmp = _mm256_aesenc_epi128(tmp, v2);
        tmp = _mm256_aesenc_epi128(tmp, v3);

        t = _mm256_add_epi8(t, _mm256_loadu_si256(KEYS.as_ptr() as *const __m256i));

        lane = _mm256_aesenclast_epi128(_mm256_aesenc_epi128(tmp, t), lane);
    }
    
    // Extract the two 128-bit lanes
    let mut lane1 = _mm256_castsi256_si128(lane);
    let mut lane2 = _mm256_extracti128_si256(lane, 1);
    // For 'Zeroes' test
    let len_vec =  _mm_set1_epi32(len as i32);
    lane1 = _mm_add_epi8(lane1, len_vec);
    lane2 = _mm_add_epi8(lane2, len_vec);
    // Merge lanes
    (aes_encrypt(lane1, lane2), ptr as *const __m128i, whole_vector_count % 8)
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