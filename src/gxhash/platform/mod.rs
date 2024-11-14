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

#[inline(always)]
pub unsafe fn finalize(hash: State) -> State {
    let mut hash = aes_encrypt(hash, ld(KEYS.as_ptr()));
    hash = aes_encrypt(hash, ld(KEYS.as_ptr().offset(4)));
    hash = aes_encrypt_last(hash, ld(KEYS.as_ptr().offset(8)));

    hash
}

pub const KEYS: [u32; 12] = 
   [0xbe12445a, 0xad14c56e, 0xfe099832, 0xc32d962a,
    0x6782a174, 0xca96641a, 0x349ffc28, 0xf7b26a02,
    0x5280d61c, 0x9816b206, 0xac894e2e, 0x5b3b242c];