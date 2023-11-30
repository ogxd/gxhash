#[cfg(target_arch = "aarch64")]
#[path = "aarch64.rs"]
mod platform;

#[cfg(target_arch = "x86_64")]
#[path = "x86_64.rs"]
mod platform;

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
unsafe fn compress_8_128(mut ptr: *const State, end_address: usize, hash_vector: State) -> State {
    let mut lane1 = create_empty();
    let mut lane2 = create_empty();
    while (ptr as usize) < end_address {

        crate::gxhash::load_unaligned!(ptr, v0, v1, v2, v3, v4, v5, v6, v7);

        let mut tmp1: State;
        let mut tmp2: State;

        tmp1 = compress_fast(v0, v2);
        tmp2 = compress_fast(v1, v3);

        tmp1 = compress_fast(tmp1, v4);
        tmp2 = compress_fast(tmp2, v5);

        tmp1 = compress_fast(tmp1, v6);
        tmp2 = compress_fast(tmp2, v7);

        lane1 = compress(lane1, tmp1);
        lane2 = compress(lane2, tmp2);
    }
    compress(hash_vector, compress_fast(lane1, lane2))
}