mod result_processor;

use result_processor::*;

use std::hint::black_box;
use std::hash::Hasher;
use std::time::{Instant, Duration};
use std::alloc::{alloc, dealloc, Layout};
use std::slice;

use rand::Rng;

use gxhash::*;

const ITERATIONS: usize = 1000;
const MAX_RUN_DURATION: Duration = Duration::from_millis(1000);

fn main() {
    let mut rng = rand::thread_rng();

    // Allocate 32-bytes-aligned
    let layout = Layout::from_size_align(40_000, 32).unwrap();
    let ptr = unsafe { alloc(layout) };
    let slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr, layout.size()) };

    // Fill with random bytes
    rng.fill(slice);

    let mut processor: Box<dyn ResultProcessor> = if cfg!(feature = "bench-csv") {
        Box::new(OutputCsv::default())
    } else if cfg!(feature = "bench-md") {
        Box::new(OutputMd::default())
    } else if cfg!(feature = "bench-plot") {
        Box::new(OutputPlot::default())
    } else {
        Box::new(OutputSimple::default())
    };

    // GxHash
    let gxhash_name = if cfg!(feature = "hybrid") { "GxHash-Hybrid" } else { "GxHash" };
    benchmark(processor.as_mut(), slice, gxhash_name, |data: &[u8], seed: i64| -> u64 {
        gxhash64(data, seed)
    });

    // XxHash (twox-hash)
    benchmark(processor.as_mut(), slice, "XxHash (XXH3)", |data: &[u8], seed: u64| -> u64 {
        twox_hash::xxh3::hash64_with_seed(data, seed)
    });
    
    // AHash
    let ahash_hasher = ahash::RandomState::with_seed(42);
    benchmark(processor.as_mut(), slice, "AHash", |data: &[u8], _: i32| -> u64 {
        ahash_hasher.hash_one(data)
    });

    // T1ha0
    benchmark(processor.as_mut(), slice, "T1ha0", |data: &[u8], seed: u64| -> u64 {
        t1ha::t1ha0(data, seed)
    });

    // FNV-1a
    benchmark(processor.as_mut(), slice, "FNV-1a", |data: &[u8], seed: u64| -> u64 {
        let mut fnv_hasher = fnv::FnvHasher::with_key(seed);
        fnv_hasher.write(data);
        fnv_hasher.finish()
    });

    // HighwayHash
    benchmark(processor.as_mut(), slice, "HighwayHash", |data: &[u8], _: i32| -> u64 {
        use highway::{HighwayHasher, HighwayHash};
        HighwayHasher::default().hash64(data)
    });

    // SeaHash
    benchmark(processor.as_mut(), slice, "SeaHash", |data: &[u8], seed: u64| -> u64 {
        seahash::hash_seeded(data, seed, 0, 0, 0)
    });

    // MetroHash
    benchmark(processor.as_mut(), slice, "Metrohash", |data: &[u8], seed: i32| -> u64 {
        let mut metrohash_hasher = metrohash::MetroHash64::with_seed(seed as u64);
        metrohash_hasher.write(data);
        metrohash_hasher.finish()
    });

    processor.finish();

    // Free benchmark data
    unsafe { dealloc(ptr, layout) };
}

fn benchmark<F, S>(processor: &mut dyn ResultProcessor, data: &[u8], name: &str, delegate: F)
    where F: Fn(&[u8], S) -> u64, S: Default + TryFrom<u128> + TryInto<usize> + Clone + Copy
{
    processor.on_start(name);
    for i in 2.. {
        let len = usize::pow(2, i);
        if len > data.len() {
            break;
        }

        // Warmup
        time::<_, _, ITERATIONS>(&delegate, &data[..len], S::default()); 

        let mut durations_s = vec![];
        let now = Instant::now();
        while now.elapsed() < MAX_RUN_DURATION {
            // Make seed unpredictable to prevent optimizations
            let seed = S::try_from(now.elapsed().as_nanos()).unwrap_or_else(|_| panic!());
            // Offset slice by an unpredictable amount to prevent optimization (pre caching)
            // and make the benchmark use both aligned and unaligned data
            let start = S::try_into(seed).unwrap_or_else(|_| panic!()) & 0xFF;
            let end = start + len;
            let slice = &data[start..end];
            // Execute method for a new iterations
            let duration = time::<_, _, ITERATIONS>(&delegate, slice, seed);
            durations_s.push(duration.as_secs_f64());
        }
        let average_duration_s = calculate_average_without_outliers(&mut durations_s);
        let throughput = (len as f64) / (1024f64 * 1024f64 * (average_duration_s / ITERATIONS as f64));

        processor.on_result(len, throughput);
    }
    processor.on_end();
}

fn time<F, S, const N: usize>(delegate: F, slice: &[u8], seed: S) -> Duration
    where F: Fn(&[u8], S) -> u64, S: Default + TryFrom<u128> + TryInto<usize> + Clone + Copy
{
    // Time measurement similar to what is done in criterion.rs
    // https://github.com/bheisler/criterion.rs/blob/e1a8c9ab2104fbf2d15f700d0038b2675054a2c8/src/bencher.rs#L87
    let now = Instant::now();
    iter::<F, S, N>(delegate, slice, seed);
    now.elapsed()
}

// The content might be inlined, but the function itself should not be inlined
// This favors benchmarked methods with small byte code size, which is more realistic
#[inline(never)]
fn iter<F, S, const N: usize>(delegate: F, slice: &[u8], seed: S)
    where F: Fn(&[u8], S) -> u64, S: Default + TryFrom<u128> + TryInto<usize> + Clone + Copy
{
    for _ in 0..N {
        // Black box the result to prevent the compiler from optimizing the operation away
        // Black box the slice to prevent the compiler to assume the slice is constant
        // We don't black box the seed because it's likely to be constant in most real-world usage scenarios
        black_box(delegate(black_box(slice), seed));
    }
}

// Outliers are inevitable, especially on a low number of iterations
// To avoid computing a huge number of iterations we can use the interquartile range
fn calculate_average_without_outliers(timings: &mut Vec<f64>) -> f64 {
    timings.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let q1 = percentile(timings, 25.0);
    let q3 = percentile(timings, 75.0);
    let iqr = q3 - q1;

    let lower_bound = q1 - 1.5 * iqr;
    let upper_bound = q3 + 1.5 * iqr;

    let filtered_timings: Vec<f64> = timings
        .iter()
        .filter(|&&x| x >= lower_bound && x <= upper_bound)
        .cloned()
        .collect();

    let sum: f64 = filtered_timings.iter().sum();
    let count = filtered_timings.len();

    sum / count as f64
}

fn percentile(sorted_data: &Vec<f64>, percentile: f64) -> f64 {
    let idx = (percentile / 100.0 * (sorted_data.len() - 1) as f64).round() as usize;
    sorted_data[idx]
}