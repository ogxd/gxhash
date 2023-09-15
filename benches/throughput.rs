use std::{mem::size_of, time::Duration};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use gxhash::gxhash;

use rand::Rng;

fn gxhash_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    let mut len = 1;

    let mut group = c.benchmark_group("gxhash");
    for i in 1..9 {
        len *= 4;
        let mut random_bytes: Vec<u8> = vec![0; len];
        rng.fill(&mut random_bytes[..]);

        let ptr = random_bytes.as_ptr() as *const u8;
        let len = ptr as usize % size_of::<gxhash::state>() == 0;
        
        println!("aligned: {}", len);

        // let (prefix, aligned, suffix) = unsafe { random_bytes.align_to_mut::<gxhash::state>() };
        // let ptr = aligned.as_ptr() as *const u8;
        // let len = aligned.len() * std::mem::size_of::<gxhash::state>();
        // let i8_slice: &[u8] = unsafe { std::slice::from_raw_parts(ptr, len) };

        group.throughput(Throughput::Bytes(random_bytes.len() as u64));
        group.bench_with_input(format!("{} bytes", random_bytes.len()), &random_bytes, |bencher, input| {
            bencher.iter(|| gxhash(input))
        });
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10000)
        .measurement_time(Duration::from_secs(10));  // Set your custom sample size here
    targets = gxhash_benchmark,
}
criterion_main!(benches);
