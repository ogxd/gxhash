// Primitives call intrinsics that require the features of this backend. They are not inlined in the primitives,
// but in the functions compiled with these features that the primitives are inlined in (see with_features).
#![allow(unknown_lints, inline_always_mismatching_target_features)]

#[cfg(not(any(feature = "std", all(target_feature = "aes", target_feature = "neon"))))]
compile_error!{"Without the std feature, gxhash requires aes and neon intrinsics. Enable the std feature for runtime detection, or build with RUSTFLAGS=\"-C target-cpu=native\" or RUSTFLAGS=\"-C target-feature=+aes,+neon\"."}

use core::arch::aarch64::*;

use super::{cold_path, Run, KEYS, VECTOR_SIZE};

// Whether the features of this backend are enabled at compile time, in which case it is inlined
pub(crate) const STATIC: bool = cfg!(all(target_feature = "aes", target_feature = "neon"));

// Functions of the algorithm that are not inlined are compiled with the features of this backend, as they may
// not be enabled at compile time. The ones marked inline may still be inlined in callers with these features.
macro_rules! with_features {
    (inline: $($item:item)*) => { $(#[target_feature(enable = "aes,neon")] #[inline] $item)* };
    ($($item:item)*) => { $(#[target_feature(enable = "aes,neon")] #[inline(never)] $item)* };
}

#[path = "../algorithm.rs"]
mod algorithm;
pub(crate) use algorithm::*;

// Without std, the features must be enabled at compile time (see the compile_error above)
#[inline(always)]
pub(crate) fn has_aes() -> bool {
    #[cfg(feature = "std")]
    return STATIC || std::arch::is_aarch64_feature_detected!("aes");
    #[cfg(not(feature = "std"))]
    return STATIC;
}

// No wider AES instructions on this platform
#[inline(always)]
pub(crate) fn has_wide_aes() -> bool {
    false
}
pub(crate) use algorithm::compress_16 as compress_16_wide;

pub type State = int8x16_t;

#[inline(always)]
pub unsafe fn create_empty() -> State {
    vdupq_n_s8(0)
}

#[inline(always)]
pub unsafe fn load_unaligned(p: *const State) -> State {
    vld1q_s8(p as *const i8)
}

#[inline(always)]
pub unsafe fn get_partial_safe(data: *const State, len: usize) -> State {
    // Reading 16 bytes from data would cross a page boundary. Instead, we read the 16 bytes ending at
    // the end of the input: they belong to the page of the first and/or last byte of the input, so the
    // read is always valid. The table lookup then moves the input bytes to the front of the vector,
    // and indices beyond 15 pick the padding from the second table register.
    let end_vector: uint8x16_t;
    let start = (data as *const u8).add(len).sub(VECTOR_SIZE);
    core::arch::asm!("ld1 {{v0.16b}}, [{start}]", start = in(reg) start, out("v0") end_vector, options(nostack, preserves_flags, readonly));
    let indices = vld1q_u8([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].as_ptr());
    let shifted_indices = vaddq_u8(indices, vdupq_n_u8((VECTOR_SIZE - len) as u8));
    let table = uint8x16x2_t(end_vector, vdupq_n_u8(len as u8));
    vreinterpretq_s8_u8(vqtbl2q_u8(table, shifted_indices))
}

#[inline(always)]
pub unsafe fn get_partial_unsafe(data: *const State, len: usize) -> State {
    // May read out-of-bound, BUT we use inline assembly to ensure we can control the behavior
    // and prevent the compiler from doing any kind of optimization that might change the behavior.
    let mut oob_vector: State;
    core::arch::asm!("ld1 {{v0.16b}}, [{data}]", data = in(reg) data, out("v0") oob_vector, options(nostack, preserves_flags, readonly));
    // The mask of the input bytes is a window over a table, which saves a vector op and a vector register
    let mask = vld1q_u8(PARTIAL_MASKS.as_ptr().add(VECTOR_SIZE - len));
    // Input bytes, followed by padding bytes set to the input length
    vbslq_s8(mask, oob_vector, vdupq_n_s8(len as i8))
}

const PARTIAL_MASKS: [u8; 2 * VECTOR_SIZE] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

#[inline(always)]
// See https://blog.michaelbrase.com/2018/05/08/emulating-x86-aes-intrinsics-on-armv8-a
pub unsafe fn aes_encrypt(data: State, keys: State) -> State {
    // Encrypt
    let encrypted = vaeseq_u8(vreinterpretq_u8_s8(data), vdupq_n_u8(0));
    // Mix columns
    let mixed = vaesmcq_u8(encrypted);
    // Xor keys
    vreinterpretq_s8_u8(veorq_u8(mixed, vreinterpretq_u8_s8(keys)))
}

#[inline(always)]
// See https://blog.michaelbrase.com/2018/05/08/emulating-x86-aes-intrinsics-on-armv8-a
pub unsafe fn aes_encrypt_last(data: State, keys: State) -> State {
    // Encrypt
    let encrypted = vaeseq_u8(vreinterpretq_u8_s8(data), vdupq_n_u8(0));
    // Xor keys
    vreinterpretq_s8_u8(veorq_u8(encrypted, vreinterpretq_u8_s8(keys)))
}

// Lane primitives: a chain lane_start, lane_absorb*, lane_end computes F(..F(F(key ^ b0) ^ b1).. ^ bn),
// F being an AES round without key (SubBytes, ShiftRows, MixColumns). On ARM, AESE xors its operands
// before SubBytes, so each block costs a single (fused) AESE + AESMC.
#[inline(always)]
pub unsafe fn lane_start(key: State, block: State) -> State {
    vreinterpretq_s8_u8(vaesmcq_u8(vaeseq_u8(vreinterpretq_u8_s8(key), vreinterpretq_u8_s8(block))))
}

#[inline(always)]
pub unsafe fn lane_absorb(mut lane: State, block: State) -> State {
    // Same as lane_start, but pins the result to the lane register. AESE xors its operands, so LLVM
    // may otherwise write the result in the block register, costing a move per block in loops.
    core::arch::asm!(
        "aese {lane:v}.16b, {block:v}.16b",
        "aesmc {lane:v}.16b, {lane:v}.16b",
        lane = inout(vreg) lane, block = in(vreg) block, options(pure, nomem, nostack, preserves_flags));
    lane
}

#[inline(always)]
pub unsafe fn lane_end(lane: State) -> State {
    lane
}

// Ends a lane and xors x to it. The result is the xor of the returned pair: on ARM, the xor is deferred so that
// it can be done by the AESE of the next round, as AESE xors its operands.
#[inline(always)]
pub unsafe fn lane_end_xor(lane: State, x: State) -> (State, State) {
    (lane, x)
}

// aes_encrypt(data ^ pending, keys). Pins the result to the data register: AESE xors its operands, so LLVM may
// otherwise write the result in the register of pending, which is then lost if it is a reused constant.
#[inline(always)]
pub unsafe fn aes_encrypt_xor(mut data: State, pending: State, keys: State) -> State {
    core::arch::asm!(
        "aese {data:v}.16b, {pending:v}.16b",
        "aesmc {data:v}.16b, {data:v}.16b",
        data = inout(vreg) data, pending = in(vreg) pending, options(pure, nomem, nostack, preserves_flags));
    veorq_s8(data, keys)
}

#[inline(always)]
pub unsafe fn xor(a: State, b: State) -> State {
    veorq_s8(a, b)
}

#[inline(always)]
pub unsafe fn load_len(len: usize) -> State {
    load_u64(len as u64)
}

#[inline(always)]
pub unsafe fn ld(array: *const u32) -> State {
    vreinterpretq_s8_u32(vld1q_u32(array))
}

// Values are loaded in the lowest bits of the vector, the rest being zeroes
#[inline(always)]
pub unsafe fn load_u64(x: u64) -> State {
    // fmov zeroes the upper half of the vector. LLVM otherwise emits a movi + mov pair for this.
    let v: State;
    core::arch::asm!("fmov {v:d}, {x}", v = out(vreg) v, x = in(reg) x, options(pure, nomem, nostack, preserves_flags));
    v
}

#[inline(always)]
pub unsafe fn load_u128(x: u128) -> State {
    let ptr = &x as *const u128 as *const i8;
    vld1q_s8(ptr)
}
