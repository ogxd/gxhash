#![feature(core_intrinsics)]

//use std::intrinsics::likely;
use std::time::Duration;
use std::time::Instant;
use std::hint::black_box;
use rand::Rng;

mod gxhash;

pub use gxhash::gxhash;

// fn main() {
//     println!("{}", sum1(1));
//     println!("{}", sum2(1));

//     let mut rng = rand::thread_rng();
//     let mut random_bytes = [0i8; 16384]; // Create an array of 16 bytes, initialized to 0
//     rng.fill(&mut random_bytes[..]); // Fill the array with random bytes

//     unsafe { println!("{}", gxhash(&random_bytes)) };
// }

// #[inline(never)]
// fn sum1(x: u32) -> u32 {
//     let mut sum = x;
//     for i in 0..1000 {
//         sum = black_box(sum + i);
//         sum = black_box(sum + i);
//         sum = black_box(sum + i);
//         sum = black_box(sum + i);
//         sum = black_box(sum + i);
//     }
//     sum
// }

// #[inline(never)]
// fn sum2(x: u32) -> u32 {
//     let mut sum = x;
//     for i in 0..1000 {
//         let mut tempSum: u32 = 0;
//         tempSum += x;
//         tempSum += x;
//         tempSum += x;
//         tempSum += x;
//         tempSum += x;

//         sum += tempSum;
//     }
//     sum
// }

