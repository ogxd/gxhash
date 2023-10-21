#![feature(build_hasher_simple_hash_one)]

use std::time::Duration;
use std::alloc::{alloc, dealloc, Layout};
use std::slice;

use criterion::measurement::{WallTime};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput, PlotConfiguration, AxisScale, BenchmarkGroup};
use gxhash::*;
use rand::Rng;

fn benchmark<F>(c: &mut BenchmarkGroup<WallTime>, data: &[u8], name: &str, delegate: F)
    where F: Fn(&[u8], i32) -> u64
{
    for i in 1..8 {
        let len = usize::pow(4, i);

        c.throughput(Throughput::Bytes(len as u64));

        let aligned_slice = &data[0..len];
        c.bench_with_input(format!("{} bytes", len), aligned_slice, |bencher, input| {
            bencher.iter(|| black_box(delegate(input, 0)))
        });

        // let unaligned_slice = &slice[1..len];
        // group.bench_with_input(format!("{} bytes (unaligned)", len), unaligned_slice, |bencher, input| {
        //     bencher.iter(|| black_box(gxhash(input)))
        // });
    }
}

fn benchmark_all(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    // Allocate 32-bytes-aligned
    let layout = Layout::from_size_align(100000, 32).unwrap();
    let ptr = unsafe { alloc(layout) };
    let slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr, 100000) };

    // Fill with random bytes
    rng.fill(slice);

    let mut group = c.benchmark_group("hash algos");
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    // GxHash
    benchmark(&mut group, slice, "gxhash", gxhash64);
    
    // AHash
    let build_hasher = ahash::RandomState::with_seeds(0, 0, 0, 0);
    benchmark(&mut group, slice, "ahash", |data: &[u8], _: i32| -> u64 {
        build_hasher.hash_one(data)
    });

    // T1ha0
    benchmark(&mut group, slice, "t1ha0", |data: &[u8], seed: i32| -> u64 {
        t1ha::t1ha0(data, seed as u64)
    });

    // XxHash (twox-hash)
    benchmark(&mut group, slice, "xxhash (twox-hash)", |data: &[u8], seed: i32| -> u64 {
        twox_hash::xxh3::hash64_with_seed(data, seed as u64)
    });

    group.finish();

    // Free benchmark data
    unsafe { dealloc(ptr, layout) };
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(1000)
        .measurement_time(Duration::from_secs(2));
    targets = benchmark_all,
}
criterion_main!(benches);