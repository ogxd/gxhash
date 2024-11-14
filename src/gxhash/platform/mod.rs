#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
#[path = "arm.rs"]
mod platform;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path = "x86.rs"]
mod platform;

pub use platform::*;

use core::mem::size_of;

pub(crate) const VECTOR_SIZE: usize = size_of::<State>();
// 4KiB is the default page size for most systems, and conservative for other systems such as macOS ARM (16KiB)
const PAGE_SIZE: usize = 0x1000;

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
unsafe fn check_same_page(ptr: *const State) -> bool {
    let address = ptr as usize;
    // Mask to keep only the last 12 bits
    let offset_within_page = address & (PAGE_SIZE - 1);
    // Check if the 16th byte from the current offset exceeds the page boundary
    offset_within_page < PAGE_SIZE - VECTOR_SIZE
}

#[cfg(target_arch = "arm")]
use core::arch::arm::*;
#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

#[inline(always)]
pub unsafe fn finalize(hash: State) -> State {
    let mut hash = aes_encrypt(hash, ld(KEYS.as_ptr()));
    hash = aes_encrypt(hash, ld(KEYS.as_ptr().offset(4)));
    hash = aes_encrypt_last(hash, ld(KEYS.as_ptr().offset(8)));

    hash
}

pub const PRIME_1: u32 = 2_654_435_761;
pub const PRIME_2: u32 = 2_246_822_519;
pub const PRIME_3: u32 = 3_266_489_917;
pub const PRIME_4: u32 = 668_265_263;
pub const PRIME_5: u32 = 374_761_393;

#[inline(always)]
pub unsafe fn finalize_ez(hash: State) -> State {
    // let mut hash = aes_encrypt(hash, ld(KEYS.as_ptr()));
    // hash = aes_encrypt(hash, ld(KEYS.as_ptr().offset(4)));
    // hash = aes_encrypt_last(hash, ld(KEYS.as_ptr().offset(8)));
    
    let mut hash_u64x2 = vreinterpretq_u64_s8(hash);
    // let low = vgetq_lane_u64(hash_u64x2, 0);
    // let high = vgetq_lane_u64(hash_u64x2, 1);

    hash_u64x2 = vreinterpretq_u64_u32(vmulq_n_u32(vreinterpretq_u32_u64(hash_u64x2), PRIME_1));
    hash_u64x2 = veorq_u64(vshrq_n_u64::<37>(hash_u64x2), hash_u64x2);
    hash_u64x2 = vreinterpretq_u64_u32(vmulq_n_u32(vreinterpretq_u32_u64(hash_u64x2), PRIME_2));
    hash_u64x2 = veorq_u64(vshrq_n_u64::<32>(hash_u64x2), hash_u64x2);
    hash_u64x2 = vreinterpretq_u64_u32(vmulq_n_u32(vreinterpretq_u32_u64(hash_u64x2), PRIME_3));

    return vreinterpretq_s8_u64(hash_u64x2);

    // tl;dr; it's just avalanche((value as i32) * PRIME32_1) * PRIME64_2

    // avalanche:
    // h64 ^= h64 >> 37;
    // h64 = h64.wrapping_mul(PRIME64_3);
    // h64 ^ (h64 >> 32)
}

pub const KEYS: [u32; 12] = 
   [0xF2784542, 0xB09D3E21, 0x89C222E5, 0xFC3BC28E,
    0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39132BD9,
    0xD0012E32, 0x689D2B7D, 0x5544B1B7, 0xC78B122B];