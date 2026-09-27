// Prints a markdown table of the time per hash of the base and head revisions, for inputs of the sizes given as
// arguments and for HashSet keys. Measurements alternate between revisions, so that both run in the same conditions.

use std::hash::BuildHasher;
use std::hint::black_box;
use std::time::Instant;

const BATCH: usize = 20_000;

// Median time per call over batches, in nanoseconds. Generic, so that the hash is inlined in the loop.
#[inline(never)]
fn time<T: ?Sized, F: Fn(&T, i64) -> u64>(f: F, input: &T) -> f64 {
    let mut samples: Vec<f64> = (0..40)
        .map(|_| {
            let start = Instant::now();
            for i in 0..BATCH {
                black_box(f(black_box(input), black_box(i as i64)));
            }
            start.elapsed().as_secs_f64() * 1e9 / BATCH as f64
        })
        .collect();
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}

fn row<T: ?Sized>(name: &str, input: &T, base: impl Fn(&T, i64) -> u64 + Copy, head: impl Fn(&T, i64) -> u64 + Copy) {
    let (mut b, mut h) = (f64::MAX, f64::MAX);
    for _ in 0..9 {
        b = b.min(time(base, input));
        h = h.min(time(head, input));
    }
    println!("| {name} | {b:.2} | {h:.2} | {:.2} |", h / b);
}

fn main() {
    let data: Vec<u8> = (0..4096).map(|i| (i * 31 + 7) as u8).collect();
    let (base_hasher, head_hasher) = (base::GxBuildHasher::with_seed(42), head::GxBuildHasher::with_seed(42));
    println!("| Input | Base (ns) | Head (ns) | Head / Base |");
    println!("|---|---:|---:|---:|");
    for len in std::env::args().skip(1).map(|arg| arg.parse::<usize>().unwrap()) {
        row(&format!("gxhash64, {len} bytes"), &data[..len], |d, s| base::gxhash64(d, s), |d, s| head::gxhash64(d, s));
    }
    row("hash_one(u64)", &42u64, |k, s| base_hasher.hash_one(k ^ s as u64), |k, s| head_hasher.hash_one(k ^ s as u64));
    for key in ["gxhash", "https://github.com/ogxd/gxhash"] {
        row(&format!("hash_one(&str), {} bytes", key.len()), key, |k, _| base_hasher.hash_one(k), |k, _| head_hasher.hash_one(k));
    }
}
