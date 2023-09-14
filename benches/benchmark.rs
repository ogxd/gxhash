use criterion::{black_box, criterion_group, criterion_main, Criterion};

use gxhash::gxhash;

const iterations: u32 = 100000;

fn sum_naive() -> u32 {
    let mut sum = 0;
    for i in 0..iterations {
        sum = black_box(sum + 1);
    }
    sum
}

fn sum_unrolled() -> u32 {
    let mut sum = 0;
    for i in 0..iterations/5 {
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
    }
    sum
}

fn sum_unrolled_high_ilp() -> u32 {
    let mut sum = 0;
    for i in 0..iterations/5 {
        let mut tempSum: u32 = 0;
        tempSum = black_box(tempSum + 1);
        tempSum = black_box(tempSum + 1);
        tempSum = black_box(tempSum + 1);
        tempSum = black_box(tempSum + 1);
        tempSum = black_box(tempSum + 1);

        sum = black_box(sum + tempSum);
    }
    sum
}

fn ilp_benchmark(c: &mut Criterion) {
    c.bench_function("naive", |b| b.iter(|| black_box(sum_naive())));
    c.bench_function("unrolling", |b| b.iter(|| black_box(sum_unrolled())));
    c.bench_function("unrolling + high ILP", |b| b.iter(|| black_box(sum_unrolled_high_ilp())));
}

use rand::Rng;

fn gxhash_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    let mut len = 1;

    for i in 1..9 {
        len *= 4;
        let mut random_bytes: Vec<i8> = vec![0; len];
        rng.fill(&mut random_bytes[..]);

        c.bench_function(format!("gxhash({})", len).as_str(), |b| b.iter(|| black_box(unsafe { gxhash(&random_bytes) })));
    }
}

criterion_group!(benches, gxhash_benchmark);
//criterion_group!(benches, ilp_benchmark, gxhash_benchmark);
criterion_main!(benches);
