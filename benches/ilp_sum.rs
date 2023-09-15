use criterion::{black_box, criterion_group, criterion_main, Criterion};

const ITERATIONS: u32 = 100000;

fn sum_naive() -> u32 {
    let mut sum = 0;
    for i in 0..ITERATIONS {
        sum = black_box(sum + 1);
    }
    sum
}

fn sum_unrolled() -> u32 {
    let mut sum = 0;
    for i in 0..ITERATIONS/5 {
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
        sum = black_box(sum + 1);
    }
    sum
}

fn sum_unrolled_tempsum() -> u32 {
    let mut sum = 0;
    for i in 0..ITERATIONS/5 {
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

fn sum_unrolled_5_lanes() -> u32 {
    let mut sum1 = 0;
    let mut sum2 = 0;
    let mut sum3 = 0;
    let mut sum4 = 0;
    let mut sum5 = 0;
    for i in 0..ITERATIONS/5 {
        sum1 = black_box(sum1 + 1);
        sum2 = black_box(sum2 + 1);
        sum3 = black_box(sum3 + 1);
        sum4 = black_box(sum4 + 1);
        sum5 = black_box(sum5 + 1);
    }
    black_box(sum1 + sum2 + sum3 + sum4 + sum5)
}

fn ilp_benchmark(c: &mut Criterion) {
    c.bench_function("sum_naive", |b| b.iter(|| black_box(sum_naive())));
    c.bench_function("sum_unrolled", |b| b.iter(|| black_box(sum_unrolled())));
    c.bench_function("sum_unrolled_tempsum", |b| b.iter(|| black_box(sum_unrolled_tempsum())));
    c.bench_function("sum_unrolled_5_lanes", |b| b.iter(|| black_box(sum_unrolled_5_lanes())));
}

criterion_group!(benches, ilp_benchmark);
criterion_main!(benches);
