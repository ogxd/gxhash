use rand::Rng;

use gxhash::gxhash;

fn main() {

    let mut rng = rand::thread_rng();
    let mut random_bytes = [0i8; 16384 * 4]; // Create an array of 16 bytes, initialized to 0
    rng.fill(&mut random_bytes[..]); // Fill the array with random bytes

    unsafe { println!("{}", gxhash(&random_bytes)) };
}