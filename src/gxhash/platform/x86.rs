#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::sync::atomic::{AtomicU8, Ordering};

use super::{cold_path, Run, KEYS, VECTOR_SIZE};

// Whether the features of this backend are enabled at compile time, in which case it is inlined
pub(crate) const STATIC: bool = cfg!(all(target_feature = "aes", target_feature = "sse2"));

// Functions of the algorithm that are not inlined are compiled with the features of this backend, as they may
// not be enabled at compile time. The ones marked inline may still be inlined in callers with these features.
macro_rules! with_features {
    (inline: $($item:item)*) => { $(#[target_feature(enable = "aes,sse2")] #[inline] $item)* };
    ($($item:item)*) => { $(#[target_feature(enable = "aes,sse2")] #[inline(never)] $item)* };
}

#[path = "../algorithm.rs"]
mod algorithm;
pub(crate) use algorithm::*;

// CPU features, detected once at runtime (0 until then)
static FEATURES: AtomicU8 = AtomicU8::new(0);
const DETECTED: u8 = 1;
const AES: u8 = 2;
const WIDE_AES: u8 = 4;

#[inline(always)]
fn features() -> u8 {
    match FEATURES.load(Ordering::Relaxed) {
        0 => detect(),
        features => features,
    }
}

#[cold]
#[inline(never)]
fn detect() -> u8 {
    #[cfg(feature = "std")]
    let features = DETECTED
        | if std::is_x86_feature_detected!("aes") { AES } else { 0 }
        | if std::is_x86_feature_detected!("vaes") && std::is_x86_feature_detected!("avx2") { WIDE_AES } else { 0 };
    // Without std, 256-bit AES is not detected, as it also requires checking that the OS saves 256-bit registers
    #[cfg(not(feature = "std"))]
    #[allow(unused_unsafe)] // Safe since Rust 1.94
    let features = DETECTED | if unsafe { __cpuid(1) }.ecx & (1 << 25) != 0 { AES } else { 0 };
    FEATURES.store(features, Ordering::Relaxed);
    features
}

#[inline(always)]
pub(crate) fn has_aes() -> bool {
    STATIC || features() & AES != 0
}

// Whether 256-bit AES instructions (VAES) are available, to process two lanes per instruction
#[inline(always)]
pub(crate) fn has_wide_aes() -> bool {
    cfg!(all(target_feature = "vaes", target_feature = "avx2")) || features() & WIDE_AES != 0
}

pub type State = __m128i;

#[inline(always)]
pub unsafe fn create_empty() -> State {
    _mm_setzero_si128()
}

#[inline(always)]
pub unsafe fn load_unaligned(p: *const State) -> State {
    _mm_loadu_si128(p)
}

// Reading 16 bytes from data would cross a page boundary. Instead, we read the 16 bytes ending at the end of
// the input: they belong to the page of the first and/or last byte of the input, so the read is always valid.
// The input bytes are then moved to the front of the vector, followed by padding bytes set to the input length.
#[inline(always)]
unsafe fn load_end(data: *const State, len: usize) -> State {
    let end_vector: State;
    let start = (data as *const u8).add(len).sub(VECTOR_SIZE);
    core::arch::asm!("movdqu {0}, [{1}]", out(xmm_reg) end_vector, in(reg) start, options(nostack, preserves_flags, readonly));
    end_vector
}

#[cfg(target_feature = "ssse3")]
#[inline(always)]
pub unsafe fn get_partial_safe(data: *const State, len: usize) -> State {
    let indices = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let shifted = _mm_shuffle_epi8(load_end(data, len), _mm_add_epi8(indices, _mm_set1_epi8((VECTOR_SIZE - len) as i8)));
    let len_vec = _mm_set1_epi8(len as i8);
    let mask = _mm_cmpgt_epi8(len_vec, indices);
    _mm_xor_si128(_mm_and_si128(_mm_xor_si128(shifted, len_vec), mask), len_vec)
}

#[cfg(not(target_feature = "ssse3"))]
#[cold]
#[inline(never)]
pub unsafe fn get_partial_safe(data: *const State, len: usize) -> State {
    let mut buffer = [len as u8; VECTOR_SIZE * 2];
    _mm_storeu_si128(buffer.as_mut_ptr() as *mut State, load_end(data, len));
    _mm_loadu_si128(buffer.as_ptr().add(VECTOR_SIZE - len) as *const State)
}

#[inline(always)]
pub unsafe fn get_partial_unsafe(data: *const State, len: usize) -> State {
    // May read out-of-bound, BUT we use inline assembly to ensure we can control the behavior
    // and prevent the compiler from doing any kind of optimization that might change the behavior.
    let mut oob_vector: State;
    core::arch::asm!("movdqu {0}, [{1}]", out(xmm_reg) oob_vector, in(reg) data, options(nostack, preserves_flags, readonly));
    let indices = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let len_vec = _mm_set1_epi8(len as i8);
    let mask = _mm_cmpgt_epi8(len_vec, indices);
    // Input bytes, followed by padding bytes set to the input length
    _mm_xor_si128(_mm_and_si128(_mm_xor_si128(oob_vector, len_vec), mask), len_vec)
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

// Lane primitives: a chain lane_start, lane_absorb*, lane_end computes F(..F(F(key ^ b0) ^ b1).. ^ bn),
// F being an AES round without key (SubBytes, ShiftRows, MixColumns). AESENC xors its key after
// MixColumns, so the lane holds the value to encrypt, already xored with the last absorbed block:
// each block costs a single AESENC, plus one at the end of the chain.
#[inline(always)]
pub unsafe fn lane_start(key: State, block: State) -> State {
    _mm_xor_si128(key, block)
}

#[inline(always)]
pub unsafe fn lane_absorb(lane: State, block: State) -> State {
    _mm_aesenc_si128(lane, block)
}

#[inline(always)]
pub unsafe fn lane_end(lane: State) -> State {
    _mm_aesenc_si128(lane, _mm_setzero_si128())
}

// Ends a lane and xors x to it. The result is the xor of the returned pair: on x86, AESENC xors its key after
// the round, so the xor is done by the last round of the lane.
#[inline(always)]
pub unsafe fn lane_end_xor(lane: State, x: State) -> (State, State) {
    (_mm_aesenc_si128(lane, x), _mm_setzero_si128())
}

// aes_encrypt(data ^ pending, keys)
#[inline(always)]
pub unsafe fn aes_encrypt_xor(data: State, pending: State, keys: State) -> State {
    _mm_aesenc_si128(_mm_xor_si128(data, pending), keys)
}

#[inline(always)]
pub unsafe fn xor(a: State, b: State) -> State {
    _mm_xor_si128(a, b)
}

#[inline(always)]
pub unsafe fn load_len(len: usize) -> State {
    load_u64(len as u64)
}

// Same as compress_16, but each 256-bit instruction processes two consecutive lanes
#[allow(improper_ctypes_definitions)]
#[target_feature(enable = "aes,vaes,avx2")]
#[inline(never)]
pub unsafe extern "C" fn compress_16_wide<const GXHASH: bool>(ptr: *const State, len: usize, seed: State) -> State {
    let ptr = ptr as *const __m256i;
    let last = ptr.cast::<u8>().add(len - 16 * VECTOR_SIZE).cast::<__m256i>();
    let seed = _mm256_set_m128i(seed, seed);
    let mut lanes = [seed; 8];
    for i in 0..8 {
        lanes[i] = _mm256_xor_si256(seed, _mm256_loadu_si256(ptr.add(i)));
    }
    let mut ptr = ptr.add(8);
    while ptr < last {
        for i in 0..8 {
            lanes[i] = _mm256_aesenc_epi128(lanes[i], _mm256_loadu_si256(ptr.add(i)));
        }
        ptr = ptr.add(8);
    }
    let zero = _mm256_setzero_si256();
    for i in 0..8 {
        lanes[i] = _mm256_aesenc_epi128(_mm256_aesenc_epi128(lanes[i], _mm256_loadu_si256(last.add(i))), zero);
    }
    // Same merge tree as merge_lanes: lane i is merged with lane i + 8, then i + 4, i + 2 and i + 1.
    // Register j holds lanes 2j and 2j + 1, so the first three levels operate on pairs of lanes.
    for i in 0..4 {
        lanes[i] = _mm256_aesenc_epi128(lanes[i], lanes[i + 4]);
    }
    for i in 0..2 {
        lanes[i] = _mm256_aesenc_epi128(_mm256_aesenc_epi128(lanes[i], zero), lanes[i + 2]);
    }
    let lanes = _mm256_aesenc_epi128(_mm256_aesenc_epi128(lanes[0], zero), lanes[1]);
    let hash = merge(_mm256_castsi256_si128(lanes), _mm256_extracti128_si256(lanes, 1));
    finish::<GXHASH>(xor(hash, load_len(len)))
}

// Values are loaded in the lowest bits of the vector, the rest being zeroes
#[inline(always)]
pub unsafe fn load_u64(x: u64) -> State {
    _mm_set_epi64x(0, x as i64)
}

#[inline(always)]
pub unsafe fn load_u128(x: u128) -> State {
    let ptr = &x as *const u128 as *const State;
    _mm_loadu_si128(ptr)
}
