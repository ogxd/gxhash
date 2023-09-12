use std::arch::aarch64::int8x16_t;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use gxhash::gxhash;

const iterations: u32 = 100000;

fn sum_naive() -> u32 {
    let mut sum = 0;
    for i in 0..iterations {
        sum = black_box(sum + 1);
    }
    sum
}

fn sum_unrolled() -> u32 {
    let mut sum = 0;
    for i in 0..iterations/5 {
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
    }
    sum
}

fn sum_unrolled_high_ilp() -> u32 {
    let mut sum = 0;
    for i in 0..iterations/5 {
        let mut tempSum: u32 = 0;
        tempSum = black_box(tempSum + 1);
        tempSum = black_box(tempSum + 1);
        tempSum = black_box(tempSum + 1);
        tempSum = black_box(tempSum + 1);
        tempSum = black_box(tempSum + 1);

        sum = black_box(sum + tempSum);
    }
    sum
}

fn ilp_benchmark(c: &mut Criterion) {
    c.bench_function("naive", |b| b.iter(|| black_box(sum_naive())));
    c.bench_function("unrolling", |b| b.iter(|| black_box(sum_unrolled())));
    c.bench_function("unrolling + high ILP", |b| b.iter(|| black_box(sum_unrolled_high_ilp())));
}

use rand::Rng;

fn gxhash_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut random_bytes = [0i8; 16384]; // Create an array of 16 bytes, initialized to 0
    rng.fill(&mut random_bytes[..]); // Fill the array with random bytes

    let (prefix, aligned, suffix) = unsafe { random_bytes.align_to_mut::<int8x16_t>() };
    
    // Get the raw pointer and length for the new slice of i8
    let ptr = aligned.as_ptr() as *const i8;
    let len = aligned.len() * std::mem::size_of::<int8x16_t>();

    // Create the new slice of i8
    let i8_slice: &[i8] = unsafe { std::slice::from_raw_parts(ptr, len) };
    c.bench_function("gxhash", |b| b.iter(|| black_box(unsafe { gxhash(&i8_slice) })));
}

criterion_group!(benches, gxhash_benchmark);
//criterion_group!(benches, ilp_benchmark, gxhash_benchmark);
criterion_main!(benches);
