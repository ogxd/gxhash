
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[inline]
fn dohash(hash: usize, value: usize) -> usize {
    hash * value + value
}

fn simple_loop(input: &[usize]) -> usize {
    let mut hash = 0;
    let mut i: usize = 0;
    while i < input.len() {
        hash = dohash(hash, input[i]);

        i = i + 1;
    }
    hash
}

fn unrolled_loop(input: &[usize]) -> usize {
    let mut hash: usize = 0;
    let mut i: usize = 0;
    while i < input.len() {
        hash = dohash(hash, input[i]);
        hash = dohash(hash, input[i + 1]);
        hash = dohash(hash, input[i + 2]);
        hash = dohash(hash, input[i + 3]);
        hash = dohash(hash, input[i + 4]);

        i = i + 5;
    }
    hash
}

fn unrolled_loop_temp_per_iteration(input: &[usize]) -> usize {
    let mut hash: usize = 0;
    let mut i: usize = 0;
    while i < input.len() {
        let mut temp_hash: usize = 0;
        temp_hash = dohash(temp_hash, input[i]);
        temp_hash = dohash(temp_hash, input[i + 1]);
        temp_hash = dohash(temp_hash, input[i + 2]);
        temp_hash = dohash(temp_hash, input[i + 3]);
        temp_hash = dohash(temp_hash, input[i + 4]);

        hash = dohash(hash, temp_hash);

        i = i + 5;
    }
    hash
}

fn unrolled_laned_loop(input: &[usize]) -> usize {
    let mut hash1: usize = 0;
    let mut hash2: usize = 0;
    let mut hash3: usize = 0;
    let mut hash4: usize = 0;
    let mut hash5: usize = 0; 
    let mut i: usize = 0;
    while i < input.len() {
        hash1 = dohash(hash1, input[i]);
        hash2 = dohash(hash2, input[i + 1]);
        hash3 = dohash(hash3, input[i + 2]);
        hash4 = dohash(hash4, input[i + 3]);
        hash5 = dohash(hash5, input[i + 4]);

        i = i + 5;
    }
    dohash(dohash(dohash(dohash(hash1, hash2), hash3), hash4), hash5)
}

use rand::Rng;

fn ilp_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut input: [usize; 100000] = [0; 100000];
    for i in 0..input.len() {
        input[i] = rng.gen::<usize>();
    }
    c.bench_function("simple_loop", |b| b.iter(|| black_box(simple_loop(&input))));
    c.bench_function("unrolled_loop", |b| b.iter(|| black_box(unrolled_loop(&input))));
    c.bench_function("unrolled_loop_temp_per_iteration", |b| b.iter(|| black_box(unrolled_loop_temp_per_iteration(&input))));
    c.bench_function("unrolled_laned_loop", |b| b.iter(|| black_box(unrolled_laned_loop(&input))));
}

criterion_group!(benches, ilp_benchmark);
criterion_main!(benches);
