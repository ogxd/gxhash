use std::intrinsics::likely;

mod platform;

pub use platform::*;

#[inline] // To be disabled when profiling
pub fn gxhash0_32(input: &[u8], seed: i32) -> u32 {
    unsafe {
        let p = &gxhash::<0>(input, seed) as *const state as *const u32;
        *p
    }
}

#[inline] // To be disabled when profiling
pub fn gxhash0_64(input: &[u8], seed: i32) -> u64 {
    unsafe {
        let p = &gxhash::<0>(input, seed) as *const state as *const u64;
        *p
    }
}

#[inline] // To be disabled when profiling
pub fn gxhash1_32(input: &[u8], seed: i32) -> u32 {
    unsafe {
        let p = &gxhash::<1>(input, seed) as *const state as *const u32;
        *p
    }
}

#[inline] // To be disabled when profiling
pub fn gxhash1_64(input: &[u8], seed: i32) -> u64 {
    unsafe {
        let p = &gxhash::<1>(input, seed) as *const state as *const u64;
        *p
    }
}

#[inline]
fn gxhash<const N: usize>(input: &[u8], seed: i32) -> state {
    unsafe {
        const VECTOR_SIZE: isize = std::mem::size_of::<state>() as isize;
        
        let len: isize = input.len() as isize;

        let p = input.as_ptr() as *const i8;
        let mut v = p as *const state;

        // Quick exit
        if len <= VECTOR_SIZE {
            let partial_vector = get_partial(v, len);
            return finalize(partial_vector, seed);
        }

        let mut end_address: usize;
        let mut remaining_blocks_count: isize = len / VECTOR_SIZE;
        let mut hash_vector: state = create_empty();

        // Choose compression function depending on version.
        // Lower is faster, higher is more collision resistant.
        let c = match N {
            0 => compress_0,
            1 => compress_1,
            _ => compress_1
        };

        macro_rules! load_unaligned {
            ($($var:ident),+) => {
                $(
                    #[allow(unused_mut)]
                    let mut $var = load_unaligned(v);
                    v = v.offset(1);
                )+
            };
        }

        const UNROLL_FACTOR: isize = 8;
        if len >= VECTOR_SIZE * UNROLL_FACTOR {

            let unrollable_blocks_count: isize = (len / (VECTOR_SIZE * UNROLL_FACTOR)) * UNROLL_FACTOR; 
            end_address = v.offset(unrollable_blocks_count) as usize;
    
            load_unaligned!(s0, s1, s2, s3, s4, s5, s6, s7);
 
            while (v as usize) < end_address {
                
                load_unaligned!(v0, v1, v2, v3, v4, v5, v6, v7);

                prefetch(v);

                s0 = c(s0, v0);
                s1 = c(s1, v1);
                s2 = c(s2, v2);
                s3 = c(s3, v3);
                s4 = c(s4, v4);
                s5 = c(s5, v5);
                s6 = c(s6, v6);
                s7 = c(s7, v7);
            }
        
            let a = c(c(s0, s1), c(s2, s3));
            let b = c(c(s4, s5), c(s6, s7));
            hash_vector = c(a, b);

            remaining_blocks_count -= unrollable_blocks_count;
        }

        end_address = v.offset(remaining_blocks_count) as usize;

        while likely((v as usize) < end_address) {
            load_unaligned!(v0);
            hash_vector = c(hash_vector, v0);
        }

        let remaining_bytes = len & (VECTOR_SIZE - 1);
        if likely(remaining_bytes > 0) {
            let partial_vector = get_partial(v, remaining_bytes);
            hash_vector = c(hash_vector, partial_vector);
        }

        finalize(hash_vector, seed)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::Rng;

    #[test]
    fn all_blocks_are_consumed() {
        let mut bytes = [42u8; 1200];

        let ref_hash = gxhash0_32(&bytes, 0);

        for i in 0..bytes.len() {
            let swap = bytes[i];
            bytes[i] = 82;
            let new_hash = gxhash0_32(&bytes, 0);
            bytes[i] = swap;

            assert_ne!(ref_hash, new_hash, "byte {i} not processed");
        }
    }

    #[test]
    fn add_zeroes_mutates_hash() {
        let mut bytes = [0u8; 1200];

        let mut rng = rand::thread_rng();
        rng.fill(&mut bytes[..32]);

        let mut ref_hash = 0;

        for i in 32..100 {
            let new_hash = gxhash0_32(&mut bytes[..i], 0);
            assert_ne!(ref_hash, new_hash, "Same hash at size {i} ({new_hash})");
            ref_hash = new_hash;
        }
    }

    #[test]
    // Test collisions for all possible inputs of size n bits with m bits set
    fn test_collisions_bits() {
        let mut bytes = [0u8; 120];
        let bits_to_set = 2;

        let n = bytes.len() * 8;
        let mut digits: Vec<usize> = vec![0; bits_to_set];

        for i in 0..bits_to_set {
            digits[i] = i;
        }

        let mut i = 0;
        let mut set = std::collections::HashSet::new();
    
        'stop: loop {

            // Set bits
            for d in digits.iter() {
                let bit = 1 << (d % 8);
                bytes[d / 8] |= bit;
            }

            i += 1;
            set.insert(gxhash0_64(&bytes, 0));
            // for &byte in bytes.iter() {
            //     print!("{:08b}", byte);
            // }
            // println!();

            // Reset bits
            for d in digits.iter() {
                bytes[d / 8] = 0;
            }

            // Increment the rightmost digit
            for i in (0..bits_to_set).rev() {
                digits[i] += 1;
                if digits[i] == n - bits_to_set + i + 1 {
                    if i == 0 {
                        break 'stop;
                    }
                    digits[i] = 0;
                } else {
                    break;
                }
            }

            for i in 1..bits_to_set {
                if digits[i] < digits[i - 1] {
                    digits[i] = digits[i - 1] + 1;
                }
            }
        }

        println!("count: {}, collisions: {}", i, i - set.len());

        assert_eq!(0, i - set.len(), "Collisions!");
    }

    #[test]
    fn hash_of_zero_is_not_zero() {
        assert_ne!(0, gxhash0_32(&[0u8; 0], 0));
        assert_ne!(0, gxhash0_32(&[0u8; 1], 0));
        assert_ne!(0, gxhash0_32(&[0u8; 1200], 0));
    }
}