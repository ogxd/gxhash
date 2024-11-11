use gxhash::*;
use std::hint::black_box;

fn main() {
    let data = b"Hello, world!";
    let hash = black_box(gxhash(black_box(data)));
    println!("Hash: {}", hash);
}

#[inline(never)]
fn gxhash(input: &[u8]) -> u64 {
    gxhash64(input, 42)
}