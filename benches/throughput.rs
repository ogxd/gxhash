use std::{mem::size_of, time::Duration};

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use gxhash::gxhash;

use rand::Rng;

use std::alloc::{alloc, dealloc, Layout};
use std::slice;

fn gxhash_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut len = 1;
    let mut group = c.benchmark_group("gxhash");

    // Allocate 32-bytes-aligned
    let layout = Layout::from_size_align(100000, 16).unwrap();
    let ptr = unsafe { alloc(layout) };
    let slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr, 100000) };

    // Fill with random bytes
    rng.fill(slice);

    for i in 1..9 {
        len *= 4;

        group.throughput(Throughput::Bytes(len as u64));

        let aligned_slice = &slice[0..len];
        group.bench_with_input(format!("{} bytes (aligned)", len), aligned_slice, |bencher, input| {
            bencher.iter(|| black_box(gxhash(input)))
        });

        let unaligned_slice = &slice[1..len];
        group.bench_with_input(format!("{} bytes (unaligned)", len), unaligned_slice, |bencher, input| {
            bencher.iter(|| black_box(gxhash(input)))
        });
    }

    unsafe { dealloc(ptr, layout) };
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(5));  // Set your custom sample size here
    targets = gxhash_benchmark,
}
criterion_main!(benches);
