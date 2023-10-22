use rand::Rng;

use gxhash::*;

fn main() {

    let mut rng = rand::thread_rng();
    let mut random_bytes = [0u8; 16384]; // Create an array of 16 bytes, initialized to 0
    rng.fill(&mut random_bytes[..]); // Fill the array with random bytes

    let mut sum: u32 = 0;

    for _ in 0..100_000_000 {
        sum = sum.wrapping_add(gxhash0_32(&random_bytes, 0));
    }

    println!("{}", sum);
}