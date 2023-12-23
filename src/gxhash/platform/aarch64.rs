use core::arch::aarch64::*;

use super::*;

pub type State = int8x16_t;

#[inline(always)]
pub unsafe fn create_empty() -> State {
    vdupq_n_s8(0)
}

#[inline(always)]
pub unsafe fn create_seed(seed: i64) -> State {
    vreinterpretq_s8_s64(vdupq_n_s64(seed))
}

#[inline(always)]
pub unsafe fn load_unaligned(p: *const State) -> State {
    vld1q_s8(p as *const i8)
}

#[inline(always)]
pub unsafe fn get_partial(p: *const State, len: usize) -> State {
    if check_same_page(p) {
        get_partial_unsafe(p, len)
    } else {
        get_partial_safe(p, len)
    }
}

#[inline(never)]
pub unsafe fn get_partial_safe(data: *const State, len: usize) -> State {
    // Temporary buffer filled with zeros
    let mut buffer = [0i8; VECTOR_SIZE];
    // Copy data into the buffer
    std::ptr::copy(data as *const i8, buffer.as_mut_ptr(), len);
    // Load the buffer into a __m256i vector
    let partial_vector = vld1q_s8(buffer.as_ptr());
    vaddq_s8(partial_vector, vdupq_n_s8(len as i8))
}

#[inline(always)]
pub unsafe fn get_partial_unsafe(data: *const State, len: usize) -> State {
    let indices = vld1q_s8([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].as_ptr());
    let mask = vcgtq_s8(vdupq_n_s8(len as i8), indices);
    let partial_vector = vandq_s8(load_unaligned(data), vreinterpretq_s8_u8(mask));
    vaddq_s8(partial_vector, vdupq_n_s8(len as i8))
}

#[inline(always)]
pub unsafe fn ld(array: *const u32) -> State {
    vreinterpretq_s8_u32(vld1q_u32(array))
}

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

#[inline(always)]
pub unsafe fn finalize(hash: State) -> State {
    let mut hash = aes_encrypt(hash, ld(KEYS.as_ptr()));
    hash = aes_encrypt(hash, ld(KEYS.as_ptr().offset(4)));
    hash = aes_encrypt_last(hash, ld(KEYS.as_ptr().offset(8)));

    hash
}

#[inline(always)]
pub unsafe fn compress_8(mut ptr: *const State, end_address: usize, hash_vector: State, len: usize) -> State {

    // Disambiguation vectors
    let mut t1: State = create_empty();
    let mut t2: State = create_empty();

    // Hash is processed in two separate 128-bit parallel lanes
    // This allows the same processing to be applied using 256-bit V-AES instrinsics
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

        t1 = vaddq_s8(t1, ld(KEYS.as_ptr()));
        t2 = vaddq_s8(t2, ld(KEYS.as_ptr().offset(4)));

        lane1 = aes_encrypt_last(aes_encrypt(tmp1, t1), lane1);
        lane2 = aes_encrypt_last(aes_encrypt(tmp2, t2), lane2);
    }
    // For 'Zeroes' test
    let len_vec =  vreinterpretq_s8_u32(vdupq_n_u32(len as u32));
    lane1 = vaddq_s8(lane1, len_vec);
    lane2 = vaddq_s8(lane2, len_vec);
    // Merge lanes
    aes_encrypt(lane1, lane2)
}

#[inline(always)]
pub unsafe fn load_u8(x: u8) -> State {
    vreinterpretq_s8_u8(vdupq_n_u8(x))
}

#[inline(always)]
pub unsafe fn load_u16(x: u16) -> State {
    vreinterpretq_s8_u16(vdupq_n_u16(x))
}

#[inline(always)]
pub unsafe fn load_u32(x: u32) -> State {
    vreinterpretq_s8_u32(vdupq_n_u32(x))
}

#[inline(always)]
pub unsafe fn load_u64(x: u64) -> State {
    vreinterpretq_s8_u64(vdupq_n_u64(x))
}

#[inline(always)]
pub unsafe fn load_u128(x: u128) -> State {
    let ptr = &x as *const u128 as *const i8;
    vld1q_s8(ptr)
}

#[inline(always)]
pub unsafe fn load_i8(x: i8) -> State {
    vdupq_n_s8(x)
}

#[inline(always)]
pub unsafe fn load_i16(x: i16) -> State {
    vreinterpretq_s8_s16(vdupq_n_s16(x))
}

#[inline(always)]
pub unsafe fn load_i32(x: i32) -> State {
    vreinterpretq_s8_s32(vdupq_n_s32(x))
}

#[inline(always)]
pub unsafe fn load_i64(x: i64) -> State {
    vreinterpretq_s8_s64(vdupq_n_s64(x))
}

#[inline(always)]
pub unsafe fn load_i128(x: i128) -> State {
    let ptr = &x as *const i128 as *const i8;
    vld1q_s8(ptr)
}