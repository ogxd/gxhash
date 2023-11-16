mod result_processor;

use result_processor::*;

use std::hint::black_box;
use std::time::{Instant, Duration};
use std::alloc::{alloc, dealloc, Layout};
use std::slice;
use std::hash::Hasher;

use rand::Rng;

use gxhash::*;

const ITERATIONS: u32 = 1000;
const MAX_RUN_DURATION: Duration = Duration::from_millis(1000);
const FORCE_NO_INLINING: bool = false;

fn main() {
    let mut rng = rand::thread_rng();

    // Allocate 32-bytes-aligned
    let layout = Layout::from_size_align(300_000, 32).unwrap();
    let ptr = unsafe { alloc(layout) };
    let slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr, layout.size()) };

    // Fill with random bytes
    rng.fill(slice);

    let mut processor = ResultProcessor::default();

    // GxHash
    let algo_name = if cfg!(feature = "avx2") { "gxhash-avx2" } else { "gxhash" };
    benchmark(&mut processor, slice, algo_name, |data: &[u8], seed: i64| -> u64 {
        gxhash64(data, seed)
    });

    // XxHash (twox-hash)
    benchmark(&mut processor, slice, "xxhash", |data: &[u8], seed: u64| -> u64 {
        twox_hash::xxh3::hash64_with_seed(data, seed)
    });
    
    // AHash
    let ahash_hasher = ahash::RandomState::with_seeds(0, 0, 0, 0);
    benchmark(&mut processor, slice, "ahash", |data: &[u8], _: i32| -> u64 {
        ahash_hasher.hash_one(data)
    });

    // T1ha0
    benchmark(&mut processor, slice, "t1ha0", |data: &[u8], seed: u64| -> u64 {
        t1ha::t1ha0(data, seed)
    });

    // SeaHash
    benchmark(&mut processor, slice, "seahash", |data: &[u8], seed: u64| -> u64 {
        seahash::hash_seeded(data, seed, 0, 0, 0)
    });

    // MetroHash
    benchmark(&mut processor, slice, "metrohash", |data: &[u8], seed: i32| -> u64 {
        let mut metrohash_hasher = metrohash::MetroHash64::with_seed(seed as u64);
        metrohash_hasher.write(data);
        metrohash_hasher.finish()
    });

    // HighwayHash
    benchmark(&mut processor, slice, "highwayhash", |data: &[u8], _: i32| -> u64 {
        use highway::{HighwayHasher, HighwayHash};
        HighwayHasher::default().hash64(data)
    });

    // FNV-1a
    benchmark(&mut processor, slice, "fnv-1a", |data: &[u8], seed: u64| -> u64 {
        let mut fnv_hasher = fnv::FnvHasher::with_key(seed);
        fnv_hasher.write(data);
        fnv_hasher.finish()
    });

    // Free benchmark data
    unsafe { dealloc(ptr, layout) };
}

fn benchmark<F, S>(processor: &mut ResultProcessor, data: &[u8], name: &str, delegate: F)
    where F: Fn(&[u8], S) -> u64, S: Default + TryFrom<u128> + TryInto<usize>
{
    processor.on_start(name);
    for i in 2.. {
        let len = usize::pow(2, i);
        if len > data.len() {
            break;
        }

        // Warmup
        black_box(time(ITERATIONS, &|| delegate(&data[..len], S::default()))); 

        let mut total_duration: Duration = Duration::ZERO;
        let mut runs: usize = 0;
        let now = Instant::now();
        while now.elapsed() < MAX_RUN_DURATION {
            // Make seed unpredictable to prevent optimizations
            let seed = S::try_from(total_duration.as_nanos())
                .unwrap_or_else(|_| panic!("Something went horribly wrong!"));
            // Offset slice by an unpredictable amount to prevent optimization (pre caching)
            // and make the benchmark use both aligned and unaligned data
            let start = S::try_into(seed)
                .unwrap_or_else(|_| panic!("Something went horribly wrong!")) & 0xFF;
            let end = start + len;
            let slice = &data[start..end];
            // Execute method for a new iterations
            total_duration += time(ITERATIONS, &|| delegate(slice, S::default()));
            runs += 1;
        }
        let throughput = (len as f64) / (1024f64 * 1024f64 * (total_duration.as_secs_f64() / runs as f64 / ITERATIONS as f64));

        processor.on_result(len, throughput);
    }
    processor.on_end();
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