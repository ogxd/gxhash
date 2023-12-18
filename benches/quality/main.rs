use std::hash::{Hasher, BuildHasher};
use std::hint::black_box;
use std::time::{Instant, Duration};
use std::alloc::{alloc, dealloc, Layout};
use std::hash::{Hash};

use gxhash::*;

fn main() {
    bench_hasher_quality::<GxBuildHasher>();
}

fn bench_hasher_quality<B>()
    where B : BuildHasher + Default
{
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

    permutations_values::<B, u8>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    permutations_values::<B, u32>(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
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