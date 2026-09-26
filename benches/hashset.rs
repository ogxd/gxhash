// Time of a HashSet lookup, for common key types and hashers. Prints the results as a markdown table.

use std::collections::HashSet;
use std::hash::{BuildHasher, BuildHasherDefault, Hash};
use std::hint::black_box;
use std::time::{Duration, Instant};

const HASHERS: [&str; 10] = ["GxHash", "std (SipHash-1-3)", "FoldHash", "FxHasher (rustc_hash)", "AHash", "XxHash (XXH3)", "T1ha", "FNV-1a", "HighwayHash", "MetroHash"];
const BATCH: usize = 10_000;
const RUN_DURATION: Duration = Duration::from_millis(500);

fn main() {
    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. ";
    let rows = [
        row("u32", 42u32),
        row("u64", 42u64),
        row("u128", 42u128),
        row("String, 6 bytes", "gxhash".to_owned()),
        row("String, 30 bytes", "https://github.com/ogxd/gxhash".to_owned()),
        row("String, 128 bytes", lorem.repeat(2)[..128].to_owned()),
        row("String, 1024 bytes", lorem.repeat(9)[..1024].to_owned()),
    ];

    println!("| Key type (ns per lookup) | {} |", HASHERS.join(" | "));
    println!("|---|{}", "---:|".repeat(HASHERS.len()));
    for (name, times) in rows {
        let best = times.iter().copied().fold(f64::INFINITY, f64::min);
        let cells: Vec<String> = times.iter().map(|&t| if t == best { format!("**{t:.2}**") } else { format!("{t:.2}") }).collect();
        println!("| {} | {} |", name, cells.join(" | "));
    }
}

fn row<T: Eq + Hash + Default>(name: &str, key: T) -> (&str, Vec<f64>) {
    eprintln!("{name}");
    let times = vec![
        lookup_ns(&key, gxhash::GxBuildHasher::default()),
        lookup_ns(&key, std::collections::hash_map::RandomState::new()),
        lookup_ns(&key, foldhash::fast::RandomState::default()),
        lookup_ns(&key, BuildHasherDefault::<rustc_hash::FxHasher>::default()),
        lookup_ns(&key, ahash::RandomState::new()),
        lookup_ns(&key, BuildHasherDefault::<twox_hash::xxh3::Hash64>::default()),
        lookup_ns(&key, t1ha::T1haBuildHasher::default()),
        lookup_ns(&key, fnv::FnvBuildHasher::default()),
        lookup_ns(&key, highway::HighwayBuildHasher::default()),
        lookup_ns(&key, metrohash::MetroBuildHasher::default()),
    ];
    for (hasher, time) in HASHERS.iter().zip(&times) {
        eprintln!("  | {hasher} > {time:.2} ns");
    }
    (name, times)
}

// Median time of a lookup, in nanoseconds
#[inline(never)]
fn lookup_ns<T: Eq + Hash + Default, B: BuildHasher>(key: &T, build_hasher: B) -> f64 {
    // If the set is empty, the lookup may return early without hashing the key
    let mut set = HashSet::with_hasher(build_hasher);
    set.insert(T::default());
    let mut samples = Vec::new();
    let start = Instant::now();
    while start.elapsed() < RUN_DURATION {
        let batch = Instant::now();
        for _ in 0..BATCH {
            // The key is not in the set, so that no equality check dilutes the hashing time
            black_box(set.contains(black_box(key)));
        }
        samples.push(batch.elapsed().as_secs_f64() * 1e9 / BATCH as f64);
    }
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}
