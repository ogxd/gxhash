use std::hint::black_box;
use std::time::Instant;
use std::alloc::{alloc, dealloc, Layout};
use std::slice;

use rand::Rng;

use gxhash::*;
mod fnv;

fn main() {
    benchmark_all();
}

#[inline(never)]
fn noop(data: &[u8], seed: i32) -> u64 {
    return seed as u64;
}

fn benchmark<F>(data: &[u8], name: &str, delegate: F)
    where F: Fn(&[u8], i32) -> u64
{
    print!("{}, ", name);
    for i in 1.. {
        let len = usize::pow(4, i);
        if len > data.len() {
            break;
        }  

        // Warmup
        black_box( time(len, data, &delegate)); 

        let mut total_time: f64 = 0f64;
        let mut runs: usize = 0;
        let now = Instant::now();
        while runs == 0 || now.elapsed().as_millis() < 500 {
            let time = time(len, data, &delegate);
            if time < 0.1f64 {
                // Invalid timing, ignore
                continue;
            }
            total_time += time;
            runs += 1;
        }
        let average_time = total_time / (runs as f64);
        let throughput = (len as f64) / (0.00_000_0001f64 * 1024f64 * 1024f64 * average_time);

        //println!("{}/{}\t\t{:.0} MiB/s", name, len, throughput);
        print!("{:.2}, ", throughput); 
    }
    println!();
}

#[inline(never)]
fn time<F>(len: usize, data: &[u8], delegate: &F) -> f64
    where F: Fn(&[u8], i32) -> u64 {

    let mut seed: i32 = 0;
    let iterations = isize::max(100_000 - len as isize, 100);

    let now = Instant::now();
    for j in 0..iterations {   
        let slice_start: usize = (seed & 0xFF) as usize;
        let slice_end = slice_start + len;
        let slice = &data[slice_start..slice_end];
        let hash = black_box(noop(slice, seed));
        seed = hash as i32;
    }
    let overhead = now.elapsed().as_nanos();
    
    let now = Instant::now();
    for j in 0..iterations {   
        let slice_start: usize = (seed & 0xFF) as usize;
        let slice_end = slice_start + len;
        let slice = &data[slice_start..slice_end];
        let hash = black_box(delegate(slice, seed));
        seed = hash as i32;
    }
    let time = now.elapsed().as_nanos();
    return (time - overhead) as f64 / (iterations as f64);
}

fn benchmark_all() {
    let mut rng = rand::thread_rng();

    // Allocate 32-bytes-aligned
    let layout = Layout::from_size_align(300_000, 32).unwrap();
    let ptr = unsafe { alloc(layout) };
    let slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr, layout.size()) };

    // Fill with random bytes
    rng.fill(slice);

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