use criterion::{criterion_group, criterion_main, Criterion};
use fnv::FnvHashSet;
use gxhash::*;
use twox_hash::xxh3;
use std::collections::HashSet;
use std::hash::BuildHasherDefault;

fn hashmap_insertion(c: &mut Criterion) {

    // Long keys
    benchmark_for_string(c, "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.");

    // Medium keys
    benchmark_for_string(c, "https://github.com/ogxd/gxhash");

    // Short keys
    benchmark_for_string(c, "gxhash");
}

fn benchmark_for_string(c: &mut Criterion, string: &str) {
    let mut group = c.benchmark_group(format!("HashSet<&str[{}]>", string.len()));

    let mut set = HashSet::new();
    group.bench_function("Default Hasher", |b| {
        b.iter(|| set.insert(string))
    });

    let mut set = GxHashSet::default();
    group.bench_function("GxHash", |b| {
        b.iter(|| set.insert(string))
    });

    let mut set = HashSet::<&str, BuildHasherDefault<xxh3::Hash64>>::default();
    group.bench_function("XxHash", |b| {
        b.iter(|| set.insert(string))
    });

    let mut set = FnvHashSet::default();
    group.bench_function("FNV-1a", |b| {
        b.iter(|| set.insert(string))
    });

    group.finish();
}

criterion_group!(benches, hashmap_insertion);
criterion_main!(benches);