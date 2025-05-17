#![feature(portable_simd)]
#![feature(core_intrinsics)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::simd::*;
use std::mem::transmute;

// Macbook pro M1 | rustc 1.89.0
// - copy:             5.2783 ns
// - urbd:             1.2450 ns
// - urbd_asm:         1.2444 ns
// - simd_masked_load: 3.0270 ns 
// - portable_simd:    3.8833 ns

// AMD Ryzen 9 5950X | rustc 1.89.0
// - copy:             8.0726 ns
// - urbd:             0.9856 ns
// - urbd_asm:         0.9748 ns
// - simd_masked_load: 2.5433 ns
// - portable_simd:    2.5748 ns

#[cfg(target_arch = "aarch64")]
mod arch {

    use super::*;
    use core::arch::aarch64::*;

    pub type State = int8x16_t;

    #[inline(always)]
    pub unsafe fn copy(data: *const State, len: usize) -> State {
        // Temporary buffer filled with zeros
        let mut buffer = [0i8; 16];
        // Copy data into the buffer
        core::ptr::copy(data as *const i8, buffer.as_mut_ptr(), len);
        // Load the buffer into a __m256i vector
        vld1q_s8(buffer.as_ptr())
    }

    #[inline(always)]
    pub unsafe fn urbd(data: *const State, len: usize) -> State {
        // May read out-of-bound, qualifying this as 'undefined behavior'.
        let oob_vector = vld1q_s8(data as *const i8);
        let indices = vld1q_s8([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].as_ptr());
        let len_vec = vdupq_n_s8(len as i8);
        let mask = vcgtq_s8(len_vec, indices);
        vandq_s8(oob_vector, vreinterpretq_s8_u8(mask))
    }

    #[inline(always)]
    pub unsafe fn urbd_asm(data: *const State, len: usize) -> State {
        // May read out-of-bound, BUT we use inline assembly to ensure we can control the behavior
        // and prevent the compiler from doing any kind of optimization that might change the behavior.
        let mut oob_vector: State;
        core::arch::asm!("ld1 {{v0.16b}}, [{data}]", data = in(reg) data, out("v0") oob_vector, options(nostack, preserves_flags, readonly));
        let indices = vld1q_s8([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].as_ptr());
        let len_vec = vdupq_n_s8(len as i8);
        let mask = vcltq_s8(indices, len_vec);
        vandq_s8(oob_vector, vreinterpretq_s8_u8(mask))
    }

    #[inline(always)]
    pub unsafe fn simd_masked_load(data: *const State, len: usize) -> State {
        let indices = vld1q_s8([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].as_ptr());
        let mask = vreinterpretq_s8_u8(vcgtq_s8(vdupq_n_s8(len as i8), indices));
        std::intrinsics::simd::simd_masked_load(mask, data as *const i8, vdupq_n_s8(len as i8))
    }

    #[inline(always)]
    pub unsafe fn portable_simd(data: *const State, len: usize) -> State {
        let slice = std::slice::from_raw_parts(data as *const i8, len);
        let data: Simd<i8, 16> = Simd::<i8, 16>::load_or_default(&slice);
        transmute(data)
    }
}

#[cfg(target_arch = "x86_64")]
mod arch {
    use super::*;
    use core::arch::x86_64::*;

    pub type State = __m128i;

    #[inline(always)]
    pub unsafe fn copy(data: *const State, len: usize) -> State {
        // Temporary buffer filled with zeros
        let mut buffer = [0i8; 16];
        // Copy data into the buffer
        core::ptr::copy(data as *const i8, buffer.as_mut_ptr(), len);
        // // Load the buffer into a __m256i vector
        _mm_loadu_si128(buffer.as_ptr() as *const State)
    }

    #[inline(always)]
    pub unsafe fn urbd(data: *const State, len: usize) -> State {
        // May read out-of-bound, qualifying this as 'undefined behavior'.
        let oob_vector = _mm_loadu_si128(data);
        let indices = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let len_vec = _mm_set1_epi8(len as i8);
        let mask = _mm_cmpgt_epi8(len_vec, indices);
        _mm_and_si128(oob_vector, mask)
    }

    #[inline(always)]
    pub unsafe fn urbd_asm(data: *const State, len: usize) -> State {
        // May read out-of-bound, BUT we use inline assembly to ensure we can control the behavior
        // and prevent the compiler from doing any kind of optimization that might change the behavior.
        let mut oob_vector: State;
        core::arch::asm!("movdqu {0}, [{1}]", out(xmm_reg) oob_vector, in(reg) data, options(nostack, preserves_flags, readonly));
        let indices = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let len_vec = _mm_set1_epi8(len as i8);
        let mask = _mm_cmpgt_epi8(len_vec, indices);
        _mm_and_si128(oob_vector, mask)
    }

    #[inline(always)]
    pub unsafe fn simd_masked_load(data: *const State, len: usize) -> State {
        let indices = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let mask = _mm_cmpgt_epi8(_mm_set1_epi8(len as i8), indices);
        State::from(std::intrinsics::simd::simd_masked_load(core::simd::i8x16::from(mask), data as *const i8, core::simd::i8x16::from(_mm_set1_epi8(len as i8))))
    }

    #[inline(always)]
    pub unsafe fn portable_simd(data: *const State, len: usize) -> State {
        let slice = std::slice::from_raw_parts(data as *const i8, len);
        let data: Simd<i8, 16> = Simd::<i8, 16>::load_or_default(&slice);
        transmute(data)
    }
}

pub(crate) const VECTOR_SIZE: usize = size_of::<arch::State>();
// 4KiB is the default page size for most systems, and conservative for other systems such as macOS ARM (16KiB)
const PAGE_SIZE: usize = 0x1000;

#[inline(always)]
pub unsafe fn urbd(p: *const arch::State, len: usize) -> arch::State {
    // Safety check
    if check_same_page(p) {
        arch::urbd(p, len)
    } else {
        arch::copy(p, len)
    }
}

#[inline(always)]
pub unsafe fn urbd_asm(p: *const arch::State, len: usize) -> arch::State {
    // Safety check
    if check_same_page(p) {
        arch::urbd_asm(p, len)
    } else {
        arch::copy(p, len)
    }
}

#[inline(always)]
unsafe fn check_same_page(ptr: *const arch::State) -> bool {
    let address = ptr as usize;
    // Mask to keep only the last 12 bits
    let offset_within_page = address & (PAGE_SIZE - 1);
    // Check if the 16th byte from the current offset exceeds the page boundary
    offset_within_page < PAGE_SIZE - VECTOR_SIZE
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_partial_safe");

    // Prepare test data
    let test_data: arch::State = unsafe { std::mem::zeroed() };

    // Benchmark with different lengths
    for &len in &[4, 8, 12, 16] {
        group.bench_function(format!("copy ({})", len), |b| {
            b.iter(|| unsafe {
                black_box(arch::copy(
                    black_box(&test_data as *const arch::State),
                    black_box(len),
                ))
            })
        });

        group.bench_function(format!("urbd ({})", len), |b| {
            b.iter(|| unsafe {
                black_box(urbd(
                    black_box(&test_data as *const arch::State),
                    black_box(len),
                ))
            })
        });

        group.bench_function(format!("urbd_asm ({})", len), |b| {
            b.iter(|| unsafe {
                black_box(urbd_asm(
                    black_box(&test_data as *const arch::State),
                    black_box(len),
                ))
            })
        });

        group.bench_function(format!("simd_masked_load ({})", len), |b| {
            b.iter(|| unsafe {
                black_box(arch::simd_masked_load(
                    black_box(&test_data as *const arch::State),
                    black_box(len),
                ))
            })
        });

        group.bench_function(format!("portable_simd ({})", len), |b| {
            b.iter(|| unsafe {
                black_box(arch::portable_simd(
                    black_box(&test_data as *const arch::State),
                    black_box(len),
                ))
            })
        });
    }

    group.finish();
}
criterion_group!(benches, benchmark);
criterion_main!(benches);