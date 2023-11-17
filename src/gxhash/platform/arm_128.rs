use std::{mem::size_of, intrinsics::likely};
use core::arch::aarch64::*;

use super::*;

pub type State = int8x16_t;

#[repr(C)]
union ReinterpretUnion {
    int64: int64x2_t,
    int32: int32x4_t,
    uint32: uint32x4_t,
    int8: int8x16_t,
    uint8: uint8x16_t,
}

#[inline(always)]
pub unsafe fn create_empty() -> State {
    vdupq_n_s8(0)
}

#[inline(always)]
pub unsafe fn create_seed(seed: i32) -> State {
    vreinterpretq_s8_s32(vdupq_n_s32(seed))
}

#[inline(always)]
pub unsafe fn load_unaligned(p: *const State) -> State {
    vld1q_s8(p as *const i8)
}

#[inline(always)]
pub unsafe fn get_partial(p: *const State, len: isize) -> State {
    let partial_vector: State;
    if likely(check_same_page(p)) {
        // Unsafe (hence the check) but much faster
        let indices = vld1q_s8([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].as_ptr());
        let mask = vcgtq_s8(vdupq_n_s8(len as i8), indices);
        partial_vector = vandq_s8(load_unaligned(p), ReinterpretUnion { uint8: mask }.int8);
    } else {
        partial_vector = get_partial_safe(p as *const i8, len as usize);
    }
    // Prevents padded zeroes to introduce bias
    return vaddq_s8(partial_vector, vdupq_n_s8(len as i8));
}

#[inline(never)]
unsafe fn get_partial_safe(data: *const i8, len: usize) -> State {
    // Temporary buffer filled with zeros
    let mut buffer = [0i8; size_of::<State>()];
    // Copy data into the buffer
    std::ptr::copy(data, buffer.as_mut_ptr(), len);
    // Load the buffer into a __m256i vector
    vld1q_s8(buffer.as_ptr())
}

#[inline(always)]
pub unsafe fn compress(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    let keys_1 = vld1q_u32([0xFC3BC28E, 0x89C222E5, 0xB09D3E21, 0xF2784542].as_ptr());
    let keys_2 = vld1q_u32([0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39136BD9].as_ptr());

    let mut bs = vreinterpretq_u8_s8(b);
    bs = aes_encrypt(bs, vreinterpretq_u8_u32(keys_1));
    bs = aes_encrypt(bs, vreinterpretq_u8_u32(keys_2));

    vreinterpretq_s8_u8(aes_encrypt_last(vreinterpretq_u8_s8(a), bs))
}

#[inline(always)]
pub unsafe fn compress_fast(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    vreinterpretq_s8_u8(aes_encrypt(vreinterpretq_u8_s8(a), vreinterpretq_u8_s8(b)))
}

#[inline(always)]
// See https://blog.michaelbrase.com/2018/05/08/emulating-x86-aes-intrinsics-on-armv8-a
unsafe fn aes_encrypt(data: uint8x16_t, keys: uint8x16_t) -> uint8x16_t {
    // Encrypt
    let encrypted = vaeseq_u8(data, vdupq_n_u8(0));
    // Mix columns
    let mixed = vaesmcq_u8(encrypted);
    // Xor keys
    veorq_u8(mixed, keys)
}

#[inline(always)]
// See https://blog.michaelbrase.com/2018/05/08/emulating-x86-aes-intrinsics-on-armv8-a
unsafe fn aes_encrypt_last(data: uint8x16_t, keys: uint8x16_t) -> uint8x16_t {
    // Encrypt
    let encrypted = vaeseq_u8(data, vdupq_n_u8(0));
    // Xor keys
    veorq_u8(encrypted, keys)
}

#[inline(always)]
pub unsafe fn finalize(hash: State, seed: State) -> State {
    // Hardcoded AES keys
    let keys_1 = vld1q_u32([0x713B01D0, 0x8F2F35DB, 0xAF163956, 0x85459F85].as_ptr());
    let keys_2 = vld1q_u32([0x1DE09647, 0x92CFA39C, 0x3DD99ACA, 0xB89C054F].as_ptr());
    let keys_3 = vld1q_u32([0xC78B122B, 0x5544B1B7, 0x689D2B7D, 0xD0012E32].as_ptr());

    // 3 rounds of AES
    let mut hash = ReinterpretUnion{ int8: hash }.uint8;
    hash = aes_encrypt(hash, ReinterpretUnion{ int8: seed }.uint8);
    hash = aes_encrypt(hash, ReinterpretUnion{ uint32: keys_1 }.uint8);
    hash = aes_encrypt(hash, ReinterpretUnion{ uint32: keys_2 }.uint8);
    hash = aes_encrypt_last(hash, ReinterpretUnion{ uint32: keys_3 }.uint8);

    ReinterpretUnion{ uint8: hash }.int8
}