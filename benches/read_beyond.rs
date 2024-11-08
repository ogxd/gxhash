#![feature(portable_simd)]
#![feature(core_intrinsics)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::simd::*;
use std::mem::transmute;

#[cfg(target_arch = "aarch64")]
mod arch {

    // Macbook pro M1
    // get_partial_safe/copy (4)
    //                         time:   [7.5658 ns 7.6379 ns 7.7465 ns]
    // get_partial_safe/urbd (4)
    //                         time:   [1.2707 ns 1.2803 ns 1.2944 ns]
    // get_partial_safe/simd_masked_load (4)
    //                         time:   [2.9972 ns 3.0029 ns 3.0107 ns]
    // get_partial_safe/portable_simd (4)
    //                         time:   [3.8087 ns 3.8305 ns 3.8581 ns]

    // AMD Ryzen 5 5625U
    // get_partial_safe/copy (4)
    //                         time:   [9.0579 ns 9.0854 ns 9.1167 ns]
    // get_partial_safe/urbd (4)
    //                         time:   [4.6165 ns 4.6203 ns 4.6244 ns]
    // get_partial_safe/simd_masked_load (4)
    //                         time:   [3.2439 ns 3.2556 ns 3.2746 ns]
    // get_partial_safe/portable_simd (4)
    //                         time:   [3.3122 ns 3.3192 ns 3.3280 ns]

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
        let partial_vector = vld1q_s8(buffer.as_ptr());
        vaddq_s8(partial_vector, vdupq_n_s8(len as i8))
    }

    #[inline(always)]
    pub unsafe fn urbd(data: *const State, len: usize) -> State {
        // Stripped of page check for simplicity, might crash program
        let indices = vld1q_s8([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].as_ptr());
        let mask = vcgtq_s8(vdupq_n_s8(len as i8), indices);
        vandq_s8(vld1q_s8(data as *const i8), vreinterpretq_s8_u8(mask))
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
        let partial_vector = _mm_loadu_si128(buffer.as_ptr() as *const State);
        _mm_add_epi8(partial_vector, _mm_set1_epi8(len as i8))
    }

    #[inline(always)]
    pub unsafe fn urbd(data: *const State, len: usize) -> State {
        // Stripped of page check for simplicity, might crash program
        let indices = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let mask = _mm_cmpgt_epi8(_mm_set1_epi8(len as i8), indices);
        _mm_and_si128(_mm_loadu_si128(data), mask)
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
                black_box(arch::urbd(
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