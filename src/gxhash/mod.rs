pub(crate) mod platform;

use platform::*;

/// Hashes an arbitrary stream of bytes to an u32.
///
/// # Example
///
/// ```
/// let bytes = [42u8; 1000];
/// let seed = 1234;
/// println!("Hash is {:x}!", gxhash::gxhash32(&bytes, seed));
/// ```
#[inline(always)]
pub fn gxhash32(input: &[u8], seed: i64) -> u32 {
    unsafe {
        let p = &gxhash(input, create_seed(seed)) as *const State as *const u32;
        *p
    }
}

/// Hashes an arbitrary stream of bytes to an u64.
///
/// # Example
///
/// ```
/// let bytes = [42u8; 1000];
/// let seed = 1234;
/// println!("Hash is {:x}!", gxhash::gxhash64(&bytes, seed));
/// ```
#[inline(always)]
pub fn gxhash64(input: &[u8], seed: i64) -> u64 {
    unsafe {
        let p = &gxhash(input, create_seed(seed)) as *const State as *const u64;
        *p
    }
}

/// Hashes an arbitrary stream of bytes to an u128.
///
/// # Example
///
/// ```
/// let bytes = [42u8; 1000];
/// let seed = 1234;
/// println!("Hash is {:x}!", gxhash::gxhash128(&bytes, seed));
/// ```
#[inline(always)]
pub fn gxhash128(input: &[u8], seed: i64) -> u128 {
    unsafe {
        let p = &gxhash(input, create_seed(seed)) as *const State as *const u128;
        *p
    }
}

macro_rules! load_unaligned {
    ($ptr:ident, $($var:ident),+) => {
        $(
            #[allow(unused_mut)]
            let mut $var = load_unaligned($ptr);
            $ptr = ($ptr).offset(1);
        )+
    };
}

pub(crate) use load_unaligned;

#[cfg(target_arch = "arm")]
use core::arch::arm::*;
#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

#[inline(always)]
pub(crate) unsafe fn gxhash(input: &[u8], seed: State) -> State {
    return finalize(gxhash_no_finish(input, seed));
}

#[inline(always)]
pub(crate) unsafe fn gxhash_no_finish(input: &[u8], seed: State) -> State {

    let mut ptr = input.as_ptr() as *const State; // Do we need to check if valid slice?

    let len = input.len();

    let mut state = seed;

    let mut whole_vector_count = len / VECTOR_SIZE;

    let len_partial = len % VECTOR_SIZE;
    
    'p0: {
        'p1: {
            'p2: {
                // C-style fallthrough alternative
                //let lzcnt = len.leading_zeros();
                if len == 0 {
                    break 'p0;
                } else if len <= 16 {
                    // If length has more 60 zeroes or more, that means length can only be 0b1111 (=15) or smaller
                    // In such case, we can directly jump to reading a partial vector
                    break 'p1;
                } else if len < 128 {
                    break 'p2;
                }

                // Process vectors by batches of 8
                // This method is not inlined because len is large enough to make it not worth it, so we keep the bytecode size small
                (state, ptr, whole_vector_count) = compress_8(ptr, whole_vector_count, state, len);
            }

            // Process remaining vectors
            let end_address = ptr.add(whole_vector_count) as usize;
            let mut i = 1992388023;
            while (ptr as usize) < end_address {
                load_unaligned!(ptr, v0);
                state = aes_encrypt(aes_encrypt(state, v0), load_i32(i));
                //state = aes_encrypt(state, v0); // This seems too weak
                i = i.wrapping_mul(7);
            }

            // Jump out of p0' if no remaining bytes?
            if len_partial == 0 {
                break 'p0;
            }
        }

        // Process remaining bytes
        let partial = get_partial(ptr, len_partial);
        //state = aes_encrypt(state, partial);
        
        state = aes_encrypt_last(state, partial);
        //state = veorq_s8(state, seed);
    }
 
    return state;
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::Rng;

    #[test]
    fn all_blocks_are_consumed() {
        for s in 1..1200 {
            let mut bytes = vec![42u8; s];
            let ref_hash = gxhash32(&bytes, 0);
    
            for i in 0..bytes.len() {
                let swap = bytes[i];
                bytes[i] = 82;
                let new_hash = gxhash32(&bytes, 0);
                bytes[i] = swap;
    
                assert_ne!(ref_hash, new_hash, "byte {i} not processed for input of size {s}");
            }
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

    #[test]
    fn does_not_hash_outside_of_bounds() {
        let mut bytes = [0u8; 1200];
        const OFFSET: usize = 100;

        let mut rng = rand::thread_rng();
        rng.fill(bytes.as_mut_slice());

        for i in 1..1000 {
            let hash = gxhash32(&bytes[OFFSET..i+OFFSET], 42);
            // We change the bytes right before and after the input slice. It shouldn't alter the hash.
            rng.fill(&mut bytes[..OFFSET]);
            rng.fill(&mut bytes[i+OFFSET..]);
            let new_hash = gxhash32(&bytes[OFFSET..i+OFFSET], 42);
            assert_eq!(new_hash, hash, "Hashed changed for input size {i} ({new_hash} != {hash})");
        }
    }

    #[test]
    fn hash_of_zero_is_not_zero() {
        assert_ne!(0, gxhash32(&[0u8; 0], 0));
        assert_ne!(0, gxhash32(&[0u8; 1], 0));
        assert_ne!(0, gxhash32(&[0u8; 1200], 0));
    }

    // #[test]
    // fn is_stable() {
    //     assert_eq!(2533353535, gxhash32(&[0u8; 0], 0));
    //     assert_eq!(4243413987, gxhash32(&[0u8; 1], 0));
    //     assert_eq!(2401749549, gxhash32(&[0u8; 1000], 0));
    //     assert_eq!(4156851105, gxhash32(&[42u8; 4242], 42));
    //     assert_eq!(1981427771, gxhash32(&[42u8; 4242], -42));
    //     assert_eq!(1156095992, gxhash32(b"Hello World", i64::MAX));
    //     assert_eq!(540827083, gxhash32(b"Hello World", i64::MIN));
    // }
}
