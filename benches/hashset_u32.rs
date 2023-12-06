use ahash::AHashSet;
use criterion::{criterion_group, criterion_main, Criterion};
use fnv::FnvHashSet;
use gxhash::*;
use rand::Rng;
use twox_hash::xxh3;
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, BuildHasher};

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("HashSet<u32>"));

    let value: u32 = rand::thread_rng().gen();

    let mut set = HashSet::<u32>::new();
    group.bench_function("Default Hasher", |b| {
        iterate(b, value, &mut set);
    });

    let mut set: HashSet::<u32, GxBuildHasher> = GxHashSet::<u32>::default();
    group.bench_function("GxHash", |b| {
        iterate(b, value, &mut set);
    });

    let mut set = AHashSet::<u32>::default();
    group.bench_function("AHash", |b| {
        iterate(b, value, &mut set);
    });

    let mut set = HashSet::<u32, BuildHasherDefault<xxh3::Hash64>>::default();
    group.bench_function("XxHash", |b| {
        iterate(b, value, &mut set);
    });

    let mut set = FnvHashSet::<u32>::default();
    group.bench_function("FNV-1a", |b| {
        iterate(b, value, &mut set);
    });

    group.finish();
}

#[inline(never)]
fn iterate<T>(b: &mut criterion::Bencher<'_>, value: u32, set: &mut HashSet<u32, T>)
    where T: BuildHasher
{
    // If hashmap is empty, it may skip hashing the key and simply return false
    // So we add a single value to prevent this optimization
    set.insert(12);
    b.iter(|| {
        // We intentionally check on a string that is not present, otherwise there will be an
        // additional equality check perform, diluting the hashing time and biasing the benchmark.
        set.contains(&value)
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);