use std::arch::aarch64::int8x16_t;

use rand::Rng;

use gxhash::gxhash;

fn main() {

    let mut rng = rand::thread_rng();
    let mut random_bytes = [0i8; 16384]; // Create an array of 16 bytes, initialized to 0
    rng.fill(&mut random_bytes[..]); // Fill the array with random bytes

    let (prefix, aligned, suffix) = unsafe { random_bytes.align_to_mut::<int8x16_t>() };
    
    // Get the raw pointer and length for the new slice of i8
    let ptr = aligned.as_ptr() as *const i8;
    let len = aligned.len() * std::mem::size_of::<int8x16_t>();

    // Create the new slice of i8
    let i8_slice: &[i8] = unsafe { std::slice::from_raw_parts(ptr, len) };

    println!("bytes: {}", prefix.len());
    println!("bytes: {}", i8_slice.len());

    unsafe { println!("{}", gxhash(&i8_slice)) };
}