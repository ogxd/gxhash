use std::mem::size_of;
use core::arch::aarch64::*;

pub type state = int8x16_t;

#[repr(C)]
union ReinterpretUnion {
    int64: int64x2_t,
    int32: int32x4_t,
    uint32: uint32x4_t,
    int8: int8x16_t,
    uint8: uint8x16_t,
}

#[inline(always)]
pub unsafe fn create_empty() -> state {
    vdupq_n_s8(0)
}

#[inline(always)]
pub unsafe fn prefetch(p: *const state) {
    //__pld(p as *const i8);
}

#[inline(always)]
pub unsafe fn load_unaligned(p: *const state) -> state {
    vld1q_s8(p as *const i8)
}

#[inline(always)]
pub unsafe fn get_partial(p: *const state, len: isize) -> state {
    let partial_vector: state;
    if check_same_page(p) {
        // Unsafe (hence the check) but much faster
        let indices = vld1q_s8([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].as_ptr());
        let mask = vcgtq_s8(vdupq_n_s8(len as i8), indices);
        partial_vector = vandq_s8(load_unaligned(p), ReinterpretUnion { uint8: mask }.int8);
    } else {
        // Safer but slower, using memcpy
        partial_vector = get_partial_safe(p as *const i8, len as usize);
    }
    // Prevents padded zeroes to introduce bias
    return vaddq_s8(partial_vector, vdupq_n_s8(len as i8));
}

#[inline(always)]
unsafe fn check_same_page(ptr: *const state) -> bool {
    let address = ptr as usize;
    // Mask to keep only the last 12 bits (3 bytes)
    let offset_within_page = address & 0xFFF;
    // Check if the 32nd byte from the current offset exceeds the page boundary
    offset_within_page <= (4096 - size_of::<state>() - 1)
}

#[inline(always)]
unsafe fn get_partial_safe(data: *const i8, len: usize) -> state {
    // Temporary buffer filled with zeros
    let mut buffer = [0i8; size_of::<state>()];
    // Copy data into the buffer
    std::ptr::copy(data, buffer.as_mut_ptr(), len);
    // Load the buffer into a __m256i vector
    vld1q_s8(buffer.as_ptr())
}

#[inline(always)]
pub unsafe fn compress(a: int8x16_t, b: int8x16_t) -> int8x16_t {
    // 37 GiB/s
    let keys_1 = vld1q_u32([0xFC3BC28E, 0x89C222E5, 0xB09D3E21, 0xF2784542].as_ptr());
    let keys_2 = vld1q_u32([0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39136BD9].as_ptr());
    let b = aes_encrypt(vreinterpretq_u8_s8(b), vreinterpretq_u8_u32(keys_1));
    let a = aes_encrypt(vreinterpretq_u8_s8(a), vreinterpretq_u8_u32(keys_2));
    vreinterpretq_s8_u8(aes_encrypt_last(a, b))

    // 70 GiB/s
    //vreinterpretq_s8_u8(aes_encrypt(vreinterpretq_u8_s8(a), vreinterpretq_u8_s8(b)))

    //vreinterpretq_s8_u8(chmuck(vreinterpretq_u8_s8(a), vreinterpretq_u8_s8(b)))
    // 26 GiB/s
    // let keys_1 = vld1q_u32([0x713B01D0, 0x8F2F35DB, 0xAF163956, 0x85459F85].as_ptr());
    // let keys_2 = vld1q_u32([0x1DE09647, 0x92CFA39C, 0x3DD99ACA, 0xB89C054F].as_ptr());
    // let keys_3 = vld1q_u32([0xC78B122B, 0x5544B1B7, 0x689D2B7D, 0xD0012E32].as_ptr());
    // let b1 = vaddq_u32(vreinterpretq_u32_s8(b), keys_2); // Cheap
    // let b2 = vreinterpretq_s8_u32(vmulq_u32(b1, keys_1)); // Cheap
    // let b3 = vreinterpretq_u32_s8(vextq_s8(b2, b2, 3)); // Expensive
    // let b4 = vaddq_u32(b3, keys_3); // Cheap
    // let b5 = vreinterpretq_s8_u32(vmulq_u32(b4, keys_2)); // Cheap
    // let b6 = vreinterpretq_u32_s8(vextq_s8(b5, b5, 3)); // Expensive
    // let b7 = vaddq_u32(b6, keys_1); // Cheap
    // let b8 = vmulq_u32(b7, keys_3); // Cheap
    // let b9 = veorq_s8(a, vreinterpretq_s8_u32(b8));
    // vextq_s8(b9, b9, 7)

    //let primes = vld1q_u32([0x9e3779b9, 0x9e3779b9, 0x9e3779b9, 0x9e3779b9].as_ptr());
    // let keys_2 = vld1q_u32([0x1DE09647, 0x92CFA39C, 0x3DD99ACA, 0xB89C054F].as_ptr());
    // let keys_3 = vld1q_u32([0xC78B122B, 0x5544B1B7, 0x689D2B7D, 0xD0012E32].as_ptr());
    // let b1 = vaddq_u32(vreinterpretq_u32_s8(b), primes); // Cheap
    // let b2 = vreinterpretq_s8_u32(vmulq_u32(vreinterpretq_u32_s8(b), primes)); // Cheap
    // let shifted = vshlq_n_s8::<1>(b2);
    // let b3 = veorq_s8(b, shifted);
    //let b3: uint32x4_t = vreinterpretq_u32_s8(vextq_s8(b2, b2, 3)); // Expensive
    // let b4 = vaddq_u32(b3, keys_3); // Cheap
    // let b5 = vreinterpretq_s8_u32(vmulq_u32(b4, keys_2)); // Cheap
    // let b6 = vreinterpretq_u32_s8(vextq_s8(b5, b5, 3)); // Expensive
    // let b7 = vaddq_u32(b6, keys_1); // Cheap
    // let b8 = vmulq_u32(b7, keys_3); // Cheap
    // let b9 = vaddq_u32(
    //     vreinterpretq_u32_s8(b),
    //     veorq_u32(
    //         primes,
    //         vaddq_u32(
    //             vshrq_n_u32::<2>(vreinterpretq_u32_s8(a)),
    //             vshlq_n_u32::<6>(vreinterpretq_u32_s8(a)))));

    // vextq_s8(vreinterpretq_s8_u32(b9), vreinterpretq_s8_u32(b9), 1)

    // let mut x = vreinterpretq_u32_s8(b);
    // // Round 1
    // x = veorq_u32(x, vshrq_n_u32::<16>(x));
    // x = vmulq_u32(x, vld1q_u32([0x7feb352d, 0x7feb352d, 0x7feb352d, 0x7feb352d].as_ptr()));
    // // Round 2
    // x = veorq_u32(x, vshrq_n_u32::<15>(x));
    // x = vmulq_u32(x, vld1q_u32([0x846ca68b, 0x846ca68b, 0x846ca68b, 0x846ca68b].as_ptr()));
    // // Round 3
    // x = veorq_u32(x, vshrq_n_u32::<16>(x));
    // let f = vaddq_s8(a, vreinterpretq_s8_u32(x));
    // vextq_s8(f, f, 1)

    //ve
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
pub unsafe fn finalize(hash: state, seed: i32) -> state {
    // Hardcoded AES keys
    let keys_1 = vld1q_u32([0x713B01D0, 0x8F2F35DB, 0xAF163956, 0x85459F85].as_ptr());
    let keys_2 = vld1q_u32([0x1DE09647, 0x92CFA39C, 0x3DD99ACA, 0xB89C054F].as_ptr());
    let keys_3 = vld1q_u32([0xC78B122B, 0x5544B1B7, 0x689D2B7D, 0xD0012E32].as_ptr());

    // 3 rounds of AES
    let mut hash = ReinterpretUnion{ int8: hash }.uint8;
    hash = aes_encrypt(hash, ReinterpretUnion{ int32: vdupq_n_s32(seed) }.uint8);
    hash = aes_encrypt(hash, ReinterpretUnion{ uint32: keys_1 }.uint8);
    hash = aes_encrypt(hash, ReinterpretUnion{ uint32: keys_2 }.uint8);
    hash = aes_encrypt_last(hash, ReinterpretUnion{ uint32: keys_3 }.uint8);
    return ReinterpretUnion{ uint8: hash }.int8;
}