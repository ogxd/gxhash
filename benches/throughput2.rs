use std::hint::black_box;
use std::time::{Instant, Duration};
use std::alloc::{alloc, dealloc, Layout};
use std::slice;

use rand::Rng;

use gxhash::*;
mod fnv;

const ITERATIONS: u32 = 1000;
const MAX_RUN_DURATION: Duration = Duration::from_millis(500);
const FORCE_NO_INLINING: bool = false;

fn main() {
    let mut rng = rand::thread_rng();

    // Allocate 32-bytes-aligned
    let layout = Layout::from_size_align(300_000, 32).unwrap();
    let ptr = unsafe { alloc(layout) };
    let slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr, layout.size()) };

    // Fill with random bytes
    rng.fill(slice);

    print!("Input size (bytes), ");
    for i in 2.. {
        let len = usize::pow(2, i);
        if len > slice.len() {
            break;
        }  
        print!("{}, ", len); 
    }
    println!();

    // GxHash
    let algo_name = if cfg!(feature = "avx2") { "gxhash-avx2" } else { "gxhash" };
    benchmark(slice, algo_name, |data: &[u8], seed: i32| -> u64 {
        gxhash64(data, seed)
    });
    
    // AHash
    let ahash_hasher = ahash::RandomState::with_seeds(0, 0, 0, 0);
    benchmark(slice, "ahash", |data: &[u8], _: i32| -> u64 {
        ahash_hasher.hash_one(data)
    });

    // T1ha0
    benchmark(slice, "t1ha0", |data: &[u8], seed: i32| -> u64 {
        t1ha::t1ha0(data, seed as u64)
    });

    // XxHash (twox-hash)
    benchmark(slice, "xxhash", |data: &[u8], seed: i32| -> u64 {
        twox_hash::xxh3::hash64_with_seed(data, seed as u64)
    });

    // HighwayHash
    benchmark(slice, "highwayhash", |data: &[u8], _: i32| -> u64 {
        use highway::{HighwayHasher, HighwayHash};
        HighwayHasher::default().hash64(data)
    });

    // FNV-1a
    benchmark(slice, "fnv-1a", |data: &[u8], seed: i32| -> u64 {
        fnv::fnv_hash(data, seed as u64)
    });

    // Free benchmark data
    unsafe { dealloc(ptr, layout) };
}

fn benchmark<F>(data: &[u8], name: &str, delegate: F)
    where F: Fn(&[u8], i32) -> u64
{
    print!("{}, ", name);
    for i in 2.. {
        let len = usize::pow(2, i);
        if len > data.len() {
            break;
        }

        // Warmup
        black_box(time(ITERATIONS, &|| delegate(&data[..len], 0))); 

        let mut total_duration: Duration = Duration::ZERO;
        let mut runs: usize = 0;
        let now = Instant::now();
        while now.elapsed() < MAX_RUN_DURATION {
            // Prevent optimizations from predictable seed
            let seed = total_duration.as_nanos() as i32;
            // Prevent optimizations from predictable slice
            // Also makes the benchmark use both aligned on unaligned data
            let start = seed as usize & 0xFF;
            let end = start + len;
            let slice = &data[start..end];
            // Execute method for a new iterations
            total_duration += time(ITERATIONS, &|| delegate(slice, seed));
            runs += 1;
        }
        let throughput = (len as f64) / (1024f64 * 1024f64 * (total_duration.as_secs_f64() / runs as f64 / ITERATIONS as f64));

        print!("{:.2}, ", throughput); 
    }
    println!();
}

#[inline(never)]
fn time<F>(iterations: u32, delegate: &F) -> Duration
    where F: Fn() -> u64
{
    let now = Instant::now();
    for _ in 0..iterations {  
        if FORCE_NO_INLINING {
            black_box(execute_noinlining(delegate));
        } else {
            black_box(delegate());
        }
    }
    now.elapsed()
}

#[inline(never)]
fn execute_noinlining<F>(delegate: &F) -> u64
    where F: Fn() -> u64
{
    delegate()
}