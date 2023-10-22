use std::intrinsics::likely;

mod platform;

pub use platform::*;

#[inline(always)] // To be disabled when profiling
pub fn gxhash0_32(input: &[u8], seed: i32) -> u32 {
    unsafe {
        let p = &gxhash::<0>(input, seed) as *const state as *const u32;
        *p
    }
}

#[inline(always)] // To be disabled when profiling
pub fn gxhash0_64(input: &[u8], seed: i32) -> u64 {
    unsafe {
        let p = &gxhash::<0>(input, seed) as *const state as *const u64;
        *p
    }
}

#[inline(always)] // To be disabled when profiling
pub fn gxhash1_32(input: &[u8], seed: i32) -> u32 {
    unsafe {
        let p = &gxhash::<1>(input, seed) as *const state as *const u32;
        *p
    }
}

#[inline(always)] // To be disabled when profiling
pub fn gxhash1_64(input: &[u8], seed: i32) -> u64 {
    unsafe {
        let p = &gxhash::<1>(input, seed) as *const state as *const u64;
        *p
    }
}

const VECTOR_SIZE: isize = std::mem::size_of::<state>() as isize;


#[inline(always)]
unsafe fn compress<const N: usize>(a: state, b: state) -> state {
    match N {
        0 => compress_0(a, b),
        1 => compress_1(a, b),
        _ => compress_1(a, b)
    }
}

#[inline(always)]
fn gxhash<const N: usize>(input: &[u8], seed: i32) -> state {
    unsafe {
        let len: isize = input.len() as isize;

        let p = input.as_ptr() as *const i8;
        let v = p as *const state;

        let hash_vector = if len <= 16 {
            get_partial(v, len)
        } else if len < 128 {
            gxhash_process_1::<N>(v, create_empty(), len)
        } else {
            gxhash_process_8::<N>(v, create_empty(), len)
        };

        finalize(hash_vector, seed)
    }
}

macro_rules! load_unaligned {
    ($ptr:ident, $($var:ident),+) => {
        $(
            #[allow(unused_mut)]
            let mut $var = load_unaligned($ptr);
            $ptr = $ptr.offset(1);
        )+
    };
}

#[inline(always)]
unsafe fn gxhash_process_8<const N: usize>(mut v: *const state, hash_vector: state, remaining_bytes: isize) -> state {

    const UNROLL_FACTOR: isize = 8;

    let unrollable_blocks_count: isize = remaining_bytes / (VECTOR_SIZE * UNROLL_FACTOR) * UNROLL_FACTOR; 
    let end_address = v.offset(unrollable_blocks_count as isize) as usize;

    load_unaligned!(v, s0, s1, s2, s3, s4, s5, s6, s7);

    while (v as usize) < end_address {
        
        load_unaligned!(v, v0, v1, v2, v3, v4, v5, v6, v7);

        prefetch(v);

        s0 = compress::<N>(s0, v0);
        s1 = compress::<N>(s1, v1);
        s2 = compress::<N>(s2, v2);
        s3 = compress::<N>(s3, v3);
        s4 = compress::<N>(s4, v4);
        s5 = compress::<N>(s5, v5);
        s6 = compress::<N>(s6, v6);
        s7 = compress::<N>(s7, v7);
    }

    let a = compress::<N>(compress::<N>(s0, s1), compress::<N>(s2, s3));
    let b = compress::<N>(compress::<N>(s4, s5), compress::<N>(s6, s7));
    let hash_vector = compress::<N>(hash_vector, compress::<N>(a, b));

    gxhash_process_1::<N>(v, hash_vector, remaining_bytes - unrollable_blocks_count * VECTOR_SIZE)
}

#[inline(always)]
unsafe fn gxhash_process_1<const N: usize>(mut v: *const state, hash_vector: state, remaining_bytes: isize) -> state {
    
    let end_address = v.offset((remaining_bytes / VECTOR_SIZE) as isize) as usize;

    let mut hash_vector = hash_vector;
    while (v as usize) < end_address {
        load_unaligned!(v, v0);
        hash_vector = compress::<N>(hash_vector, v0);
    }

    let remaining_bytes = remaining_bytes & (VECTOR_SIZE - 1);
    if remaining_bytes > 0 {
        hash_vector = gxhash_process_last::<N>(v, hash_vector, remaining_bytes);
    }
    hash_vector
}

#[inline(always)]
unsafe fn gxhash_process_last<const N: usize>(v: *const state, hash_vector: state, remaining_bytes: isize) -> state {

    let partial_vector = get_partial(v, remaining_bytes);
    compress::<N>(hash_vector, partial_vector)
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