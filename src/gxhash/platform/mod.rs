#[cfg(target_arch = "aarch64")]
#[path = "arm_128.rs"]
mod platform;

#[cfg(all(feature = "avx2", target_arch = "x86_64", target_feature = "avx2"))]
#[path = "x86_256.rs"]
mod platform;

#[cfg(all(not(feature = "avx2"), target_arch = "x86_64"))]
#[path = "x86_128.rs"]
mod platform;

use std::mem::size_of;

pub use platform::*;

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

pub trait GxPlatform {
    type State;

    unsafe fn create_empty() -> State;
    unsafe fn create_seed(seed: i64) -> State;
    unsafe fn load_unaligned(p: *const State) -> State;
    unsafe fn get_partial(p: *const State, len: usize) -> State;
    unsafe fn get_partial_safe(data: *const State, len: usize) -> State;
    unsafe fn get_partial_unsafe(data: *const State, len: usize) -> State;
    unsafe fn compress(a: State, b: State) -> State;
    unsafe fn compress_fast(a: State, b: State) -> State;
    unsafe fn finalize(hash: State) -> State;
}