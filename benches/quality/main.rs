use std::{hash::{Hash, Hasher, BuildHasher}, collections::HashSet};
use rand::Rng;
use criterion::black_box;

fn main() {
    bench_hasher_quality::<gxhash::GxBuildHasher>("GxHash");
    bench_hasher_quality::<ahash::RandomState>("AHash");
    bench_hasher_quality::<t1ha::T1haBuildHasher>("T1ha");
    bench_hasher_quality::<twox_hash::xxh3::RandomHashBuilder64>("XxHash3");
    bench_hasher_quality::<std::collections::hash_map::RandomState>("Default");
    bench_hasher_quality::<fnv::FnvBuildHasher>("FNV-1a");
}

macro_rules! check {
    ($func:expr) => {
        let score = $func;
        let name = stringify!($func).replace('\n', "").replace(" ", "");
        if score == 0.0 {
            println!("  ✅ {}", name);
        } else {
            println!("  ❌ {}", name);
            println!("     | Score: {}. Expected is 0.", score);
        };
    };
}

fn bench_hasher_quality<B>(name: &str)
    where B : BuildHasher + Default
{
    println!("Bench {}", name);

    check!(avalanche::<B, 4>());
    check!(avalanche::<B, 10>());
    check!(avalanche::<B, 32>());
    check!(avalanche::<B, 128>());
    check!(avalanche::<B, 512>());

    check!(distribution_values::<B, 4>(128 * 128));
    check!(distribution_values::<B, 16>(128 * 128));
    check!(distribution_values::<B, 128>(128 * 128));
    check!(distribution_values::<B, 512>(128 * 128));

    check!(distribution_bits::<B, 4>());
    check!(distribution_bits::<B, 16>());
    check!(distribution_bits::<B, 128>());
    check!(distribution_bits::<B, 512>());

    check!(collisions_padded_zeroes::<B>(128 * 128));

    check!(collisions_flipped_bits::<B, 2>(9));
    check!(collisions_flipped_bits::<B, 3>(9));
    check!(collisions_flipped_bits::<B, 4>(7));
    check!(collisions_flipped_bits::<B, 5>(6));
    check!(collisions_flipped_bits::<B, 6>(5));
    check!(collisions_flipped_bits::<B, 7>(5));
    check!(collisions_flipped_bits::<B, 9>(4));
    check!(collisions_flipped_bits::<B, 20>(4));
    check!(collisions_flipped_bits::<B, 32>(3));
    check!(collisions_flipped_bits::<B, 64>(3));
    check!(collisions_flipped_bits::<B, 256>(2));

    check!(collisions_powerset_bytes::<B>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
    check!(collisions_powerset_bytes::<B>(&[0, 1, 2, 4, 8, 16, 32, 64, 128]));

    check!(collisions_permuted_hasher_values::<B, u8>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
    check!(collisions_permuted_hasher_values::<B, u32>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
    check!(collisions_permuted_hasher_values::<B, u32>(&[0, 1, 2, 4, 8, 16, 32, 64, 128, 256]));

    check!(collisions_powerset_hasher_values::<B, u32>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
    check!(collisions_powerset_hasher_values::<B, u32>(&[0, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384]));
}

fn collisions_permuted_hasher_values<B, D>(data: &[impl Hash]) -> f64
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

fn collisions_powerset_hasher_values<B, D>(data: &[impl Hash]) -> f64
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

fn collisions_powerset_bytes<B>(data: &[u8]) -> f64
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

fn collisions_padded_zeroes<B>(max_size: usize) -> f64
    where B : BuildHasher + Default
{
    let build_hasher = B::default();
    let bytes = vec![0u8; max_size];

    let mut set = ahash::AHashSet::new();

    let mut i = 0;

    for _ in 0..max_size {
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

fn collisions_flipped_bits<B, const N: usize>(bits_to_set: usize) -> f64
    where B : BuildHasher + Default
{
    let build_hasher = B::default();
    let mut input = [0u8; N];
    let mut hashes = Vec::new();

    let mut hasher = build_hasher.build_hasher();
    hasher.write(&input);
    hashes.push(hasher.finish());

    flip_n_bits_recurse::<B, N>(&build_hasher, 0, bits_to_set, &mut input, &mut hashes);

    let hashes_count = hashes.len();

    let set: HashSet<u64> = HashSet::from_iter(hashes);

    //println!("{}-bit keys with 0 to {} bits set. Combinations: {}, Collisions: {}", N * 8, bits_to_set, hashes_count, hashes_count - set.len());

    (hashes_count - set.len()) as f64 / hashes_count as f64
}

fn flip_n_bits_recurse<B, const N: usize>(
    build_hasher: &B, start: usize, bits_left: usize, input: &mut [u8], hashes: &mut Vec<u64>)
    where B : BuildHasher + Default
{
    let nbits: usize = N * 8;

    for i in start..nbits {
        // Flip bit
        let bit = 1 << (i % 8);
        input[i / 8] ^= bit;

        let mut hasher = build_hasher.build_hasher();
        hasher.write(&input);
        hashes.push(hasher.finish());

        if bits_left > 1 {
            flip_n_bits_recurse::<B, N>(build_hasher, i + 1, bits_left - 1, input, hashes);
        }

        // Flip bit
        let bit = 1 << (i % 8);
        input[i / 8] ^= bit;
    }
}

fn avalanche<B, const N: usize>() -> f64
    where B : BuildHasher + Default
{
    const AVALANCHE_ITERATIONS: usize = 1000;
    const AVG_ITERATIONS: usize = 10;

    let mut sum: f64 = 0f64;
    for _ in 0..AVG_ITERATIONS {
        sum += avalanche_iterations::<B, N>(AVALANCHE_ITERATIONS);
    }

    let score = sum / AVG_ITERATIONS as f64;
    // It's important to round to ignore precision biais from avalanche computation
    // It will make an important difference in cases where avalanche is very small
    let rounded = round_to_decimal(score, (AVALANCHE_ITERATIONS as f64).log10() as usize);

    //println!("Avalanche score for input of size {}: {}", N, rounded);

    rounded
}

// Compute avalanche score for a given number of iterations.
// The more iterations, the more precise the computation will be.
// Precision is up to log10(iterations) decimals.
// For very small score, results can be rounded up to the precision level.
fn avalanche_iterations<B, const N: usize>(iterations: usize) -> f64
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

fn distribution_bits<B, const N: usize>() -> f64
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
    let rounded = round_to_decimal(score, (DISTRIBUTION_ITERATIONS as f64).log10() as usize);

    //println!("Distribution of bits score for input of size {}: {}", N, rounded);

    rounded
}

fn distribution_bits_iterations<B, const N: usize>(iterations: usize) -> f64
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
    let std = variance_to_mean(&bit_buckets, 0.5);

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

fn variance_to_mean(data: &[f64], mean: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let variance = data.iter().map(|value| {
        let diff = mean - value;
        diff * diff
    }).sum::<f64>() / data.len() as f64;

    variance
}

fn distribution_values<B, const N: usize>(buckets_count: usize) -> f64
    where B : BuildHasher + Default
{
    const DISTRIBUTION_ITERATIONS: usize = 100000;
    const AVG_ITERATIONS: usize = 100;

    let mut sum: f64 = 0f64;
    for _ in 0..AVG_ITERATIONS {
        sum += distribution_values_iterations::<B, N>(DISTRIBUTION_ITERATIONS, buckets_count);
    }

    let score = sum / AVG_ITERATIONS as f64;
    // It's important to round to ignore precision biais from distribution computation
    // It will make an important difference in cases where distribution is very small
    let rounded = round_to_decimal(score, (DISTRIBUTION_ITERATIONS as f64).log10() as usize);

    //println!("Distribution of values score for input of size {}: {}", N, score);

    rounded
}

fn distribution_values_iterations<B, const N: usize>(iterations: usize, buckets_count: usize) -> f64
    where B : BuildHasher + Default
{
    let build_hasher = B::default();

    const MAX_U64: u64 = u64::MAX;

    let mut buckets = vec![0f64; buckets_count];

    let mut rng = rand::thread_rng();

    let input: &mut [u8] = &mut [0u8; N];

    for _ in 0..iterations {

        // Random input on each iteration
        rng.fill(input);

        let mut hasher = build_hasher.build_hasher();
        hasher.write(&input);
        let hash = hasher.finish();

        let hash_f = hash as f64;

        let bucketed_f = hash_f / MAX_U64 as f64;

        let index = (buckets_count as f64 * bucketed_f).floor() as usize;

        buckets[index] += 1f64;
    }

    buckets = buckets.iter().map(|x| x / iterations as f64).collect();
    let std = variance(&buckets);

    // The worst possible variance for these buckets is 1 / buckets_count
    let worst_variance = 1f64 / buckets_count as f64;

    // Divide by the theoritical worst variance to normalize result from 0 to 1
    let score = std / worst_variance;

    score
}

fn round_to_decimal(value: f64, decimals: usize) -> f64
{
    let factor = 10f64.powi(decimals as i32 - 1);
    (value * factor).round() / factor
}