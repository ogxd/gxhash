use std::hint::black_box;

use rand::Rng;

use gxhash::gxhash;

fn main() {

    let mut rng = rand::thread_rng();
    let mut random_bytes = [0u8; 16384]; // Create an array of 16 bytes, initialized to 0
    rng.fill(&mut random_bytes[..]); // Fill the array with random bytes

    for i in 0..100000 {
        black_box(gxhash(&random_bytes));
    }
}