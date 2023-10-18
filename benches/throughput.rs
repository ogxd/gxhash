#![feature(build_hasher_simple_hash_one)]

use std::time::Duration;
use std::alloc::{alloc, dealloc, Layout};
use std::slice;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use gxhash::*;
use rand::Rng;

fn gxhash_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    // Allocate 32-bytes-aligned
    let layout = Layout::from_size_align(100000, 32).unwrap();
    let ptr = unsafe { alloc(layout) };
    let slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr, 100000) };

    // Fill with random bytes
    rng.fill(slice);

    {
        let mut group = c.benchmark_group("gxhash");

        for i in 1..8 {
            let len = usize::pow(5, i);
    
            group.throughput(Throughput::Bytes(len as u64));
    
            let aligned_slice = &slice[0..len];
            group.bench_with_input(format!("{} bytes (aligned)", len), aligned_slice, |bencher, input| {
                bencher.iter(|| black_box(gxhash32(input, 0)))
            });
    
            // let unaligned_slice = &slice[1..len];
            // group.bench_with_input(format!("{} bytes (unaligned)", len), unaligned_slice, |bencher, input| {
            //     bencher.iter(|| black_box(gxhash(input)))
            // });
        }
    }

    {
        let mut group = c.benchmark_group("ahash");

        for i in 1..8 {
            let len = usize::pow(4, i);

            group.throughput(Throughput::Bytes(len as u64));

            let aligned_slice = &slice[0..len];

            use ahash::*;
            use core::slice;
            use std::hash::{BuildHasher};
            
            let build_hasher = RandomState::with_seeds(0, 0, 0, 0);

            group.bench_with_input(format!("{} bytes (aligned)", len), aligned_slice, |bencher, input| {
                bencher.iter(|| black_box(build_hasher.hash_one(input)))
            });
        }
    }
    
    unsafe { dealloc(ptr, layout) };
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(1000)
        .measurement_time(Duration::from_secs(5));
    targets = gxhash_benchmark,
}
criterion_main!(benches);