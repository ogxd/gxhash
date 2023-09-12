use std::time::Duration;
use std::time::Instant;
use std::hint::black_box;

fn main() {
    println!("{}", sum1(1));
    println!("{}", sum2(1));
}

#[inline(never)]
fn sum1(x: u32) -> u32 {
    let mut sum = x;
    for i in 0..1000 {
        sum = black_box(sum + i);
        sum = black_box(sum + i);
        sum = black_box(sum + i);
        sum = black_box(sum + i);
        sum = black_box(sum + i);
    }
    sum
}

#[inline(never)]
fn sum2(x: u32) -> u32 {
    let mut sum = x;
    for i in 0..1000 {
        let mut tempSum: u32 = 0;
        tempSum += x;
        tempSum += x;
        tempSum += x;
        tempSum += x;
        tempSum += x;

        sum += tempSum;
    }
    sum
}