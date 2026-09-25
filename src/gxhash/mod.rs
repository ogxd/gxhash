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

#[inline(always)]
pub(crate) unsafe fn gxhash(input: &[u8], seed: State) -> State {
    return finalize(gxhash_no_finish(input, seed));
}

#[inline(always)]
pub(crate) unsafe fn gxhash_no_finish(input: &[u8], seed: State) -> State {

    let len = input.len();
    let mut ptr = input.as_ptr() as *const State;

    if len == 0 {
        return create_empty();
    }

    if len <= VECTOR_SIZE {
        // Input fits on a single SIMD vector, however we might read beyond the input message
        // Thus we need this safe method that checks if it can safely read beyond or must copy
        return get_partial(ptr, len);
    }

    // Process any partial (sub-vector) bytes FIRST. Because len > VECTOR_SIZE we know the next
    // full vector is still within the input buffer, so get_partial_unsafe is safe here: it reads
    // VECTOR_SIZE bytes starting at ptr but the mask zeroes out the bytes beyond extra_bytes_count.
    // The partial vector encodes its own byte-count (get_partial adds len to every lane byte), so
    // the hash is length-sensitive without a separate len-mixing step.
    let extra_bytes_count = len % VECTOR_SIZE;
    let whole_vector_count: usize;
    let mut lane1: State;
    let mut lane2: State;
    if extra_bytes_count == 0 {
        lane1 = seed;
        lane2 = seed;
        whole_vector_count = len / VECTOR_SIZE;
    } else {
        let partial = get_partial_unsafe(ptr, extra_bytes_count);
        ptr = ptr.cast::<u8>().add(extra_bytes_count).cast();
        // Fold partial + seed into both lanes; KEY1/KEY2 diverge them during the Duff loop.
        lane1 = aes_encrypt(seed, partial);
        lane2 = aes_encrypt(seed, partial);
        whole_vector_count = (len - extra_bytes_count) / VECTOR_SIZE;
    }

    // Duff's device via platform-specific `duff_compress`.
    // On ARM: true computed-jump dispatch (adr+lsl+br, 3 instructions, no jump table).
    // On x86: labeled-block fallback (variable-width instructions prevent clean adr+lsl).
    // tmp1/tmp2 accumulate independently and fold into lane1/lane2 via aes_encrypt_last,
    // keeping the iteration-to-iteration carry to a single AES round.
    let (lane1, lane2) = duff_compress(ptr, lane1, lane2, whole_vector_count);

    // Merge lanes.
    return aes_encrypt(lane1, lane2);
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
