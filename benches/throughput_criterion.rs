use std::time::Duration;
use std::alloc::{alloc, dealloc, Layout};
use std::slice;
use std::hash::{BuildHasher, Hasher};

use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, Criterion, Throughput, PlotConfiguration, AxisScale, BenchmarkGroup, BenchmarkId};
use std::hint::black_box;
use rand::Rng;

use gxhash::*;

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
            bencher.iter(|| black_box(delegate(black_box(input), 42)))
        });
    }
}

fn benchmark_all(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    // Allocate 32-bytes-aligned
    let layout = Layout::from_size_align(300_000, 32).unwrap();
    let ptr = unsafe { alloc(layout) };
    let slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr, layout.size()) };

    // Fill with random bytes
    rng.fill(slice);

    let mut group = c.benchmark_group("all");
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    // GxHash
    let gxhash_name = if cfg!(feature = "hybrid") { "GxHash-Hybrid" } else { "GxHash" };
    benchmark(&mut group, slice, gxhash_name, |data: &[u8], seed: i32| -> u64 {
        gxhash64(data, seed as i64)
    });

    // XxHash (twox-hash)
    benchmark(&mut group, slice, "XxHash (XXH3)", |data: &[u8], seed: i32| -> u64 {
        twox_hash::xxh3::hash64_with_seed(data, seed as u64)
    });

    // FoldHash
    let foldhash_hasher: foldhash::quality::RandomState = foldhash::quality::RandomState::default();
    benchmark(&mut group, slice, "FoldHash", |data: &[u8], _: i32| -> u64 {
        foldhash_hasher.hash_one(data)
    });

    // AHash
    benchmark(&mut group, slice, "AHash", |data: &[u8], seed: i32| -> u64 {
        let ahash_hasher = ahash::RandomState::with_seeds(seed as u64, 0, 0, 0);
        ahash_hasher.hash_one(data)
    });

    // T1ha0
    benchmark(&mut group, slice, "T1ha0", |data: &[u8], seed: i32| -> u64 {
        t1ha::t1ha0(data, seed as u64)
    });

    // FNV-1a
    benchmark(&mut group, slice, "FNV-1a", |data: &[u8], seed: i32| -> u64 {
        let mut fnv_hasher = fnv::FnvHasher::with_key(seed as u64);
        fnv_hasher.write(data);
        fnv_hasher.finish()
    });

    // HighwayHash
    benchmark(&mut group, slice, "HighwayHash", |data: &[u8], _: i32| -> u64 {
        use highway::{HighwayHasher, HighwayHash};
        HighwayHasher::default().hash64(data)
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