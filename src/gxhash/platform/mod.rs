//#[cfg(target_arch = "aarch64")]
#[path = "soft.rs"]
mod platform;

// #[cfg(target_arch = "aarch64")]
// #[path = "aarch64.rs"]
// mod platform;

// #[cfg(target_arch = "x86_64")]
// #[path = "x86_64.rs"]
// mod platform;

pub use platform::*;

use std::mem::size_of;

pub(crate) const VECTOR_SIZE: usize = size_of::<State>();
// 4KiB is the default page size for most systems, and conservative for other systems such as MacOS ARM (16KiB)
const PAGE_SIZE: usize = 0x1000;

#[inline(always)]
unsafe fn check_same_page(ptr: *const State) -> bool {
    let address = ptr as usize;
    // Mask to keep only the last 12 bits
    let offset_within_page = address & (PAGE_SIZE - 1);
    // Check if the 16nd byte from the current offset exceeds the page boundary
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
   [0xF2784542, 0xB09D3E21, 0x89C222E5, 0xFC3BC28E,
    0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39132BD9,
    0xD0012E32, 0x689D2B7D, 0x5544B1B7, 0xC78B122B];