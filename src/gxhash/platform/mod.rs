use core::mem::size_of;

#[cfg(target_arch = "aarch64")]
mod arm_128;
#[cfg(target_arch = "aarch64")]
pub use arm_128::*;

#[cfg(all(feature = "avx2", target_arch = "x86_64"))]
mod x86_256;
#[cfg(all(feature = "avx2", target_arch = "x86_64"))]
pub use x86_256::*;

#[cfg(all(not(feature = "avx2"), target_arch = "x86_64"))]
mod x86_128;
#[cfg(all(not(feature = "avx2"), target_arch = "x86_64"))]
pub use x86_128::*;

pub(crate) const VECTOR_SIZE: usize = size_of::<State>();

// 4KiB is the default page size for most systems, and conservative for other systems such as macOS ARM (16KiB)
const PAGE_SIZE: usize = 0x1000;

#[inline(always)]
unsafe fn check_same_page(ptr: *const State) -> bool {
    let address = ptr as usize;
    // Mask to keep only the last 12 bits
    let offset_within_page = address & (PAGE_SIZE - 1);
    // Check if the 16th byte from the current offset exceeds the page boundary
    offset_within_page < PAGE_SIZE - VECTOR_SIZE
}
