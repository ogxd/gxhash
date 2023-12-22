use std::hash::{Hash, Hasher, BuildHasher};
use rand::Rng;
use criterion::black_box;

fn main() {
    bench_hasher_quality::<gxhash::GxBuildHasher>("GxHash");
    bench_hasher_quality::<ahash::RandomState>("AHash");
    bench_hasher_quality::<fnv::FnvBuildHasher>("FNV-1a");
    bench_hasher_quality::<twox_hash::xxh3::RandomHashBuilder64>("XxHash3");
    bench_hasher_quality::<std::collections::hash_map::RandomState>("Default");
}

macro_rules! trace_call {
    ($func:expr) => {
        let score = $func;
        let name = stringify!($func).replace('\n', "").replace(" ", "");
        if score == 0.0 {
            println!("  ✅ {}", name);
        } else {
            println!("  ❌ {}", name);
            println!("     Score: {}", score);
        };
    };
}

fn bench_hasher_quality<B>(name: &str)
    where B : BuildHasher + Default
{
    println!("Bench {}", name);

    trace_call!(avalanche::<B, 4>());
    trace_call!(avalanche::<B, 10>());

    trace_call!(distribution_bits::<B, 10>());

    trace_call!(zeroes::<B>(0, 200_000));

    trace_call!(collisions_bits::<B>(16, 9));
    trace_call!(collisions_bits::<B>(24, 9));
    // collisions_bits::<B>(32, 7);
    // collisions_bits::<B>(40, 6);
    // collisions_bits::<B>(56, 5);
    // collisions_bits::<B>(72, 5);
    // collisions_bits::<B>(96, 4);
    // collisions_bits::<B>(160, 4);
    // collisions_bits::<B>(256, 3);
    // collisions_bits::<B>(512, 3);
    // collisions_bits::<B>(2048, 2);

    //powerset_bytes::<B>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    trace_call!(permutations_values::<B, u8>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
    // permutations_values::<B, u32>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    trace_call!(powerset_values::<B, u32>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
}

fn permutations_values<B, D>(data: &[impl Hash]) -> f64
    where B : BuildHasher + Default
{
    use itertools::Itertools;

    let build_hasher = B::default();

    let mut set = ahash::AHashSet::new();
    let mut i = 0;

    for perm in data.iter().permutations(data.len()) {
        let mut hasher = build_hasher.build_hasher();
        perm.hash(&mut hasher);
        set.insert(hasher.finish());
        i += 1;
    }

    //println!("Permutations. Combinations: {}, Collisions: {}", i, i - set.len());

    // Collision rate
    (i - set.len()) as f64 / i as f64
}

fn powerset_values<B, D>(data: &[impl Hash]) -> f64
    where B : BuildHasher + Default
{
    use itertools::Itertools;

    let build_hasher = B::default();

    let mut set = ahash::AHashSet::new();
    let mut i = 0;

    for perm in data.iter().powerset() {
        let mut hasher = build_hasher.build_hasher();
        perm.hash(&mut hasher);
        set.insert(hasher.finish());
        i += 1;
    }

    //println!("Permutations. Combinations: {}, Collisions: {}", i, i - set.len());

    // Collision rate
    (i - set.len()) as f64 / i as f64
}

fn powerset_bytes<B>(data: &[u8]) -> f64
    where B : BuildHasher + Default
{
    use itertools::Itertools;

    let build_hasher = B::default();

    let mut set = ahash::AHashSet::new();
    let mut i = 0;

    for perm in data.iter().powerset() {
        let mut hasher = build_hasher.build_hasher();
        let features: Vec<u8> = perm.iter().map(|f| **f).collect();
        hasher.write(&features);
        set.insert(hasher.finish());
        i += 1;
    }

    //println!("Permutations. Combinations: {}, Collisions: {}", i, i - set.len());

    // Collision rate
    (i - set.len()) as f64 / i as f64
}

fn zeroes<B>(min_size: usize, max_size: usize) -> f64
    where B : BuildHasher + Default
{
    let build_hasher = B::default();
    let bytes = vec![0u8; max_size];

    let mut set = ahash::AHashSet::new();

    let mut i = 0;

    for _ in min_size..max_size {
        let slice = &bytes[0..i];
        let mut hasher = build_hasher.build_hasher();
        hasher.write(slice);
        set.insert(hasher.finish());
        i += 1;
    }

    //println!("Zeroes-filled inputs from {} to {} bytes. Combinations: {}, Collisions: {}", min_size, max_size, i, i - set.len());

    // Collision rate
    (i - set.len()) as f64 / i as f64
}

fn collisions_bits<B>(size_bits: usize, bits_to_set: usize) -> f64
    where B : BuildHasher + Default
{
    let build_hasher = B::default();
    let mut bytes = vec![0u8; size_bits / 8];

    let mut digits: Vec<usize> = vec![0; bits_to_set];

    for i in 0..bits_to_set {
        digits[i] = i;
    }

    let mut i = 0;
    let mut set = ahash::AHashSet::new();

    'stop: loop {

        // Set bits
        for d in digits.iter() {
            let bit = 1 << (d % 8);
            bytes[d / 8] |= bit;
        }

        i += 1;
        let mut hasher = build_hasher.build_hasher();
        hasher.write(&bytes);
        set.insert(hasher.finish());

        // Reset bits
        for d in digits.iter() {
            bytes[d / 8] = 0;
        }

        // Increment the rightmost digit
        for i in (0..bits_to_set).rev() {
            digits[i] += 1;
            if digits[i] == size_bits - bits_to_set + i + 1 {
                if i == 0 {
                    break 'stop;
                }
                // Reset digit. It will be set to an appropriate value after.
                digits[i] = 0;
            } else {
                break;
            }
        }

        // Make sure digits are coherent
        for i in 1..bits_to_set {
            if digits[i] < digits[i - 1] {
                digits[i] = digits[i - 1] + 1;
            }
        }
    }

    //println!("{}-bit keys with {} bits set. Combinations: {}, Collisions: {}", size_bits, bits_to_set, i, i - set.len());

    // Collision rate
    (i - set.len()) as f64 / i as f64
}

pub fn avalanche<B, const N: usize>() -> f64
    where B : BuildHasher + Default
{
    const AVALANCHE_ITERATIONS: usize = 1000;
    const AVG_ITERATIONS: usize = 100;

    let mut sum: f64 = 0f64;
    for _ in 0..AVG_ITERATIONS {
        sum += avalanche_iterations::<B, N>(AVALANCHE_ITERATIONS);
    }

    let score = sum / AVG_ITERATIONS as f64;
    // It's important to round to ignore precision biais from avalanche computation
    // It will make an important difference in cases where avalanche is very small
    let rounded = (score * AVALANCHE_ITERATIONS as f64).round() / AVALANCHE_ITERATIONS as f64;

    //println!("Avalanche score for input of size {}: {}", N, rounded);

    rounded
}

// Compute avalanche score for a given number of iterations.
// The more iterations, the more precise the computation will be.
// Precision is up to log10(iterations) decimals.
// For very small score, results can be rounded up to the precision level.
pub fn avalanche_iterations<B, const N: usize>(iterations: usize) -> f64
    where B : BuildHasher + Default
{
    let build_hasher = B::default();

    const SIZE_R: usize = std::mem::size_of::<u64>();
    let mut scores_sum = 0f64;

    let mut rng = rand::thread_rng();

    let input: &mut [u8] = &mut [0u8; N];

    for _ in 0..iterations {

        // Random input on each iteration
        rng.fill(input);

        let mut hasher1 = build_hasher.build_hasher();
        hasher1.write(&input);
        let v1 = hasher1.finish();

        let bytes_bit_changed = &mut input.to_vec().clone();

        // Flip every bit
        for i in 0..(N * 8) {
    
            // Flip bit at position i
            bytes_bit_changed[i / 8] = input[i / 8] ^ (1 << (i % 8));

            let mut hasher2 = build_hasher.build_hasher();
            hasher2.write(black_box(&bytes_bit_changed)); // It seems there is a LLVM bug!?? Using black_box to prevent breaking compiler optimization
            let v2 = black_box(hasher2.finish());
    
            // Compute diffs
            let diffs = (v1 ^ v2).count_ones();

            // Score is the ratio of bits changed (0 = no bit changed, 1 = all bits changed)
            let diff_ratio = diffs as f64 / (SIZE_R * 8) as f64;
            scores_sum += diff_ratio;

            // Reset byte
            bytes_bit_changed[i / 8] = input[i / 8];
        }
    }

    let count = iterations * N * 8;
    let score = (1.0 - 2.0 * (scores_sum / count as f64)).abs();

    score
}

pub fn distribution_bits<B, const N: usize>() -> f64
    where B : BuildHasher + Default
{
    const DISTRIBUTION_ITERATIONS: usize = 10000;
    const AVG_ITERATIONS: usize = 100;

    let mut sum: f64 = 0f64;
    for _ in 0..AVG_ITERATIONS {
        sum += distribution_bits_iterations::<B, N>(DISTRIBUTION_ITERATIONS);
    }

    let score = sum / AVG_ITERATIONS as f64;
    // It's important to round to ignore precision biais from distribution computation
    // It will make an important difference in cases where distribution is very small
    let rounded = (score * 0.1f64 * DISTRIBUTION_ITERATIONS as f64).round() / 0.1f64 / DISTRIBUTION_ITERATIONS as f64;

    //println!("Distribution of bits score for input of size {}: {}", N, rounded);

    rounded
}

pub fn distribution_bits_iterations<B, const N: usize>(iterations: usize) -> f64
    where B : BuildHasher + Default
{
    let build_hasher = B::default();

    const SIZE_R: usize = std::mem::size_of::<u64>();

    let mut bit_buckets = vec![0f64; SIZE_R * 8];

    let mut rng = rand::thread_rng();

    let input: &mut [u8] = &mut [0u8; N];

    for _ in 0..iterations {

        // Random input on each iteration
        rng.fill(input);

        let mut hasher = build_hasher.build_hasher();
        hasher.write(&input);
        let hash = hasher.finish();

        let hash_bytes = hash.to_ne_bytes();
        for b in 0..SIZE_R {
            for k in 0..8 {
                bit_buckets[8 * b + k] += ((hash_bytes[b] >> k) & 1) as f64;
            }
        }
    }

    bit_buckets = bit_buckets.iter().map(|x| x / iterations as f64).collect();
    let std = variance(&bit_buckets);

    // The worst possible variance for a set of values between 0 and 1 is 0.25
    let worst_variance = 0.25f64;

    // Divide by the theoritical worst variance to normalize result from 0 to 1
    let score = std / worst_variance;

    score
}

fn variance(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mean = data.iter().sum::<f64>() / data.len() as f64;

    let variance = data.iter().map(|value| {
        let diff = mean - value;
        diff * diff
    }).sum::<f64>() / data.len() as f64;

    variance
}