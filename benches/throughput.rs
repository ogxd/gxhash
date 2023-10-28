use std::time::Duration;
use std::alloc::{alloc, dealloc, Layout};
use std::slice;

use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput, PlotConfiguration, AxisScale, BenchmarkGroup, BenchmarkId};
use rand::Rng;

use gxhash::*;
mod fnv;

fn benchmark<F>(c: &mut BenchmarkGroup<WallTime>, data: &[u8], name: &str, delegate: F)
    where F: Fn(&[u8], i32) -> u64
{
    for i in 1.. {
        let len = usize::pow(4, i);
        if len > data.len() {
            break;
        }  

        c.throughput(Throughput::Bytes(len as u64));

        let slice = &data[0..len]; // Aligned
        // let slice = &data[1..len]; // Unaligned
        c.bench_with_input(BenchmarkId::new(name, len), slice, |bencher, input| {
            bencher.iter(|| black_box(delegate(input, 0)))
        });
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

    let mut group = c.benchmark_group("all");
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    // GxHash0
    // benchmark(&mut group, slice, "gxhash0", |data: &[u8], _: i32| -> u64 {
    //     gxhash0_64(data, 0)
    // });

    // GxHash1
    benchmark(&mut group, slice, "gxhash", |data: &[u8], _: i32| -> u64 {
        gxhash1_64(data, 0)
    });
    
    // AHash
    let ahash_hasher = ahash::RandomState::with_seeds(0, 0, 0, 0);
    benchmark(&mut group, slice, "ahash", |data: &[u8], _: i32| -> u64 {
        ahash_hasher.hash_one(data)
    });

    // T1ha0
    benchmark(&mut group, slice, "t1ha0", |data: &[u8], seed: i32| -> u64 {
        t1ha::t1ha0(data, seed as u64)
    });

    // XxHash (twox-hash)
    benchmark(&mut group, slice, "xxhash", |data: &[u8], seed: i32| -> u64 {
        twox_hash::xxh3::hash64_with_seed(data, seed as u64)
    });

    // HighwayHash
    benchmark(&mut group, slice, "highwayhash", |data: &[u8], _: i32| -> u64 {
        use highway::{HighwayHasher, HighwayHash};
        HighwayHasher::default().hash64(data)
    });

    // FNV-1a
    benchmark(&mut group, slice, "fnv-1a", |data: &[u8], _: i32| -> u64 {
        fnv::fnv_hash(data)
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