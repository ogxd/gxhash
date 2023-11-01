mod platform;

use platform::*;

#[inline(always)]
pub fn gxhash32(input: &[u8], seed: i32) -> u32 {
    unsafe {
        let p = &gxhash(input, seed) as *const state as *const u32;
        *p
    }
}

#[inline(always)]
pub fn gxhash64(input: &[u8], seed: i32) -> u64 {
    unsafe {
        let p = &gxhash(input, seed) as *const state as *const u64;
        *p
    }
}

const VECTOR_SIZE: isize = std::mem::size_of::<state>() as isize;

#[inline(always)]
unsafe fn gxhash(input: &[u8], seed: i32) -> state {

    let len: isize = input.len() as isize;
    let ptr = input.as_ptr() as *const state;

    // Fast path with no compression for payloads that fit in a single state
    let hash_vector = if len <= VECTOR_SIZE {
        get_partial(ptr, len)
    } else {
        // Lower sizes first, as comparison/branching overhead will become negligible as input size grows.
        let (mut hash_vector, remaining_bytes, p) = if len <= VECTOR_SIZE * 2 {
            let v1 = load_unaligned(ptr);
            (v1, len - VECTOR_SIZE, ptr.offset(1))
        } else if len <= VECTOR_SIZE * 3 {
            let v1 = load_unaligned(ptr);
            let v2 = load_unaligned(ptr.offset(1));
            (compress(v1, v2), len - VECTOR_SIZE * 2, ptr.offset(2))
        } else if len <= VECTOR_SIZE * 4 {
            let v1 = load_unaligned(ptr);
            let v2 = load_unaligned(ptr.offset(1));
            let v3 = load_unaligned(ptr.offset(2));
            (compress(compress(v1, v2), v3), len - VECTOR_SIZE * 3, ptr.offset(3))
        } else if len < VECTOR_SIZE * 8 {
            gxhash_process_1(ptr, create_empty(), len)
        } else {
            gxhash_process_8(ptr, create_empty(), len)
        };

        if remaining_bytes > 0 {
            hash_vector = compress(hash_vector, get_partial(p, remaining_bytes))
        }

        hash_vector
    };

    finalize(hash_vector, seed)
}

macro_rules! load_unaligned {
    ($ptr:ident, $($var:ident),+) => {
        $(
            let $var = load_unaligned($ptr);
            $ptr = ($ptr).offset(1);
        )+
    };
}

#[inline(always)]
unsafe fn gxhash_process_1(mut ptr: *const state, hash_vector: state, remaining_bytes: isize) -> (state, isize, *const state) {
    
    let end_address = ptr.offset((remaining_bytes / VECTOR_SIZE) as isize) as usize;

    let mut hash_vector = hash_vector;
    while (ptr as usize) < end_address {
        load_unaligned!(ptr, v0);
        hash_vector = compress(hash_vector, v0);
    }

    let remaining_bytes: isize = remaining_bytes & (VECTOR_SIZE - 1);
    (hash_vector, remaining_bytes, ptr)
}

#[inline(always)]
unsafe fn gxhash_process_8(mut ptr: *const state, hash_vector: state, remaining_bytes: isize) -> (state, isize, *const state) {

    const UNROLL_FACTOR: isize = 8;

    let unrollable_blocks_count: isize = remaining_bytes / (VECTOR_SIZE * UNROLL_FACTOR) * UNROLL_FACTOR; 
    let end_address = ptr.offset(unrollable_blocks_count as isize) as usize;
    let mut hash_vector = hash_vector;
    while (ptr as usize) < end_address {
        
        load_unaligned!(ptr, v0, v1, v2, v3, v4, v5, v6, v7);

        prefetch(ptr);

        let mut tmp: state;
        tmp = compress_fast(v0, v1);
        tmp = compress_fast(tmp, v2);
        tmp = compress_fast(tmp, v3);
        tmp = compress_fast(tmp, v4);
        tmp = compress_fast(tmp, v5);
        tmp = compress_fast(tmp, v6);
        tmp = compress_fast(tmp, v7);

        hash_vector = compress(hash_vector, tmp);
    }

    gxhash_process_1(ptr, hash_vector, remaining_bytes - unrollable_blocks_count * VECTOR_SIZE)
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::Rng;
    use rstest::rstest;

    #[rstest]
    #[case(4)]
    #[case(16)]
    #[case(24)]
    #[case(32)]
    #[case(56)]
    #[case(72)]
    #[case(96)]
    #[case(160)]
    #[case(256)]
    #[case(512)]
    #[case(1200)]
    fn all_blocks_are_consumed(#[case] size_bits: usize) {
        let mut bytes = vec![42u8; size_bits];

        let ref_hash = gxhash32(&bytes, 0);

        for i in 0..bytes.len() {
            let swap = bytes[i];
            bytes[i] = 82;
            let new_hash = gxhash32(&bytes, 0);
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
            let new_hash = gxhash32(&mut bytes[..i], 0);
            assert_ne!(ref_hash, new_hash, "Same hash at size {i} ({new_hash})");
            ref_hash = new_hash;
        }
    }

    #[rstest]
    #[case(16, 9)]
    #[case(24, 8)]
    #[case(32, 7)]
    #[case(40, 6)]
    #[case(56, 5)]
    #[case(72, 5)]
    #[case(96, 4)]
    #[case(160, 4)]
    #[case(256, 3)]
    #[case(512, 3)]
    #[case(2048, 2)]
    // Test collisions for all possible inputs of size n bits with m bits set
    // Equivalent to SMHasher "Sparse" test
    fn test_collisions_bits(#[case] size_bits: usize, #[case] bits_to_set: usize) {
        let mut bytes = vec![0u8; size_bits / 8];

        let mut digits: Vec<usize> = vec![0; bits_to_set];

        for i in 0..bits_to_set {
            digits[i] = i;
        }

        let mut i = 0;
        let mut set = ahash::AHashSet::new();
    
        'stop: loop {

            // Set bits
            for d in digits.iter() {
                let bit = 1 << (d % 8);
                bytes[d / 8] |= bit;
            }

            i += 1;
            set.insert(gxhash64(&bytes, 0));

            // Reset bits
            for d in digits.iter() {
                bytes[d / 8] = 0;
            }

            // Increment the rightmost digit
            for i in (0..bits_to_set).rev() {
                digits[i] += 1;
                if digits[i] == size_bits - bits_to_set + i + 1 {
                    if i == 0 {
                        break 'stop;
                    }
                    // Reset digit. It will be set to an appropriate value after.
                    digits[i] = 0;
                } else {
                    break;
                }
            }

            // Make sure digits are coherent
            for i in 1..bits_to_set {
                if digits[i] < digits[i - 1] {
                    digits[i] = digits[i - 1] + 1;
                }
            }
        }

        println!("{}-bit keys with {} bits set. Combinations: {}, Collisions: {}", size_bits, bits_to_set, i, i - set.len());

        assert_eq!(0, i - set.len(), "Collisions!");
    }

    #[test]
    fn hash_of_zero_is_not_zero() {
        assert_ne!(0, gxhash32(&[0u8; 0], 0));
        assert_ne!(0, gxhash32(&[0u8; 1], 0));
        assert_ne!(0, gxhash32(&[0u8; 1200], 0));
    }
}