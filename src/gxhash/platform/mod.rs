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
        cold_path();
        get_partial_safe(p, len)
    }
}

// Hints that the branch is unlikely, so that the likely branch is the one that doesn't jump
#[rustversion::since(1.95)]
#[inline(always)]
fn cold_path() {
    core::hint::cold_path()
}

#[rustversion::before(1.95)]
#[inline(always)]
fn cold_path() {}

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
    finalize_xor(hash, create_empty())
}

// Finalizes hash ^ pending
#[inline(always)]
pub unsafe fn finalize_xor(hash: State, pending: State) -> State {
    // A single key is enough to break the symmetry of states preserved by AES rounds. Other keys would only
    // xor constants to intermediate states, which doesn't change the quality, but costs a register each.
    let mut hash = aes_encrypt_xor(hash, pending, ld(KEYS.as_ptr()));
    hash = aes_encrypt(hash, create_empty());
    hash = aes_encrypt_last(hash, create_empty());

    hash
}

pub const KEYS: [u32; 12] =
   [0xF2784542, 0xB09D3E21, 0x89C222E5, 0xFC3BC28E,
    0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39132BD9,
    0xD0012E32, 0x689D2B7D, 0x5544B1B7, 0xC78B122B];
