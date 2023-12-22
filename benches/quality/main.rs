use std::hash::{Hasher, BuildHasher};
use std::time::{Instant, Duration};
use std::alloc::{alloc, dealloc, Layout};
use std::hash::{Hash};

use ahash::RandomState;
use criterion::black_box;
use gxhash::*;
use rand::{Rng, RngCore};

fn main() {
    bench_hasher_quality::<GxBuildHasher>();
}

fn bench_hasher_quality<B>()
    where B : BuildHasher + Default
{

    //avalanche::<B, 1>();
    avalanche::<B, 10>();
    avalanche::<RandomState, 10>();
    //avalanche::<B, 100>();

    zeroes::<B>(0, 200_000);

    collisions_bits::<B>(16, 9);
    collisions_bits::<B>(24, 8);
    collisions_bits::<B>(32, 7);
    collisions_bits::<B>(40, 6);
    collisions_bits::<B>(56, 5);
    collisions_bits::<B>(72, 5);
    collisions_bits::<B>(96, 4);
    collisions_bits::<B>(160, 4);
    collisions_bits::<B>(256, 3);
    collisions_bits::<B>(512, 3);
    collisions_bits::<B>(2048, 2);

    powerset_bytes::<B>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    permutations_values::<B, u8>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    permutations_values::<B, u32>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    powerset_values::<B, u32>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]);
}

fn permutations_values<B, D>(data: &[impl Hash])
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

    println!("Permutations. Combinations: {}, Collisions: {}", i, i - set.len());
}

fn powerset_values<B, D>(data: &[impl Hash])
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

    println!("Permutations. Combinations: {}, Collisions: {}", i, i - set.len());
}

fn powerset_bytes<B>(data: &[u8])
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

    println!("Permutations. Combinations: {}, Collisions: {}", i, i - set.len());
}

fn zeroes<B>(min_size: usize, max_size: usize)
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

    println!("Zeroes-filled inputs from {} to {} bytes. Combinations: {}, Collisions: {}", min_size, max_size, i, i - set.len());

    //assert_eq!(0, i - set.len(), "Collisions!");
}

fn collisions_bits<B>(size_bits: usize, bits_to_set: usize)
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

    println!("{}-bit keys with {} bits set. Combinations: {}, Collisions: {}", size_bits, bits_to_set, i, i - set.len());

    //assert_eq!(0, i - set.len(), "Collisions!");
}

pub fn avalanche<B, const N: usize>()
    where B : BuildHasher + Default
{
    let build_hasher = B::default();

    const SIZE_R: usize = std::mem::size_of::<u64>();
    let iterations = 10_000_000;
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
            scores_sum += diffs as f64 / (SIZE_R * 8) as f64;

            // Reset byte
            bytes_bit_changed[i / 8] = input[i / 8];
        }
    }

    let count = iterations * N * 8;
    let score = (1.0 - 2.0 * (scores_sum / count as f64)).abs();

    println!("Avalanche score for input of size {}: {}", N, score);
}