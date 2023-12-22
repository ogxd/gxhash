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

#[inline(always)]
pub(crate) unsafe fn gxhash(input: &[u8], seed: State) -> State {
    finalize(compress_fast(compress_all(input), seed))
}

#[inline(always)]
pub(crate) unsafe fn compress_all(input: &[u8]) -> State {

    let len = input.len();

    if len == 0 {
        return create_empty();
    }

    let mut ptr = input.as_ptr() as *const State;

    if len <= VECTOR_SIZE {
        // Input fits on a single SIMD vector, however we might read beyond the input message
        // Thus we need this safe method that checks if it can safely read beyond or must copy
        return get_partial(ptr, len);
    }

    let extra_bytes_count = len % VECTOR_SIZE;
    let remaining_bytes: usize;

    // The input does not fit on a single SIMD vector
    let hash_vector: State;
    if extra_bytes_count == 0 {
        load_unaligned!(ptr, v0);
        hash_vector = v0;
        remaining_bytes = len - VECTOR_SIZE;
    } else {
        // If the input length does not match the length of a whole number of SIMD vectors,
        // it means we'll need to read a partial vector. We can start with the partial vector first,
        // so that we can safely read beyond since we expect the following bytes to still be part of
        // the input
        hash_vector = get_partial_unsafe(ptr, extra_bytes_count);
        ptr = ptr.cast::<u8>().add(extra_bytes_count).cast();
        remaining_bytes = len - extra_bytes_count;
    }

    #[allow(unused_assignments)]
    if len <= VECTOR_SIZE * 2 {
        // Fast path when input length > 16 and <= 32
        load_unaligned!(ptr, v0);
        compress(hash_vector, v0)
    } else if len <= VECTOR_SIZE * 3 {
        // Fast path when input length > 32 and <= 48
        load_unaligned!(ptr, v0, v1);
        compress(hash_vector, compress(v0, v1))
    } else {
        // Input message is large and we can use the high ILP loop
        compress_many(ptr, hash_vector, remaining_bytes)
    }
}

#[inline(always)]
unsafe fn compress_many(mut ptr: *const State, hash_vector: State, remaining_bytes: usize) -> State {

    const UNROLL_FACTOR: usize = 8;

    let unrollable_blocks_count: usize = remaining_bytes / (VECTOR_SIZE * UNROLL_FACTOR) * UNROLL_FACTOR; 
    let end_address = ptr.add(unrollable_blocks_count) as usize;
    let mut hash_vector = hash_vector;
    while (ptr as usize) < end_address {

        load_unaligned!(ptr, v0, v1, v2, v3, v4, v5, v6, v7);

        let mut tmp: State;
        tmp = compress_fast(v0, v1);
        tmp = compress_fast(tmp, v2);
        tmp = compress_fast(tmp, v3);
        tmp = compress_fast(tmp, v4);
        tmp = compress_fast(tmp, v5);
        tmp = compress_fast(tmp, v6);
        tmp = compress_fast(tmp, v7);

        hash_vector = compress(hash_vector, tmp);
    }

    let remaining_bytes = remaining_bytes - unrollable_blocks_count * VECTOR_SIZE;
    let end_address = ptr.add(remaining_bytes / VECTOR_SIZE) as usize;

    while (ptr as usize) < end_address {
        load_unaligned!(ptr, v0);
        hash_vector = compress(hash_vector, v0);
    }

    hash_vector
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

    // GxHash with a 128-bit state must be stable despite the different endianesses / CPU instrinsics
    #[cfg(not(feature = "avx2"))]
    #[test]
    fn is_stable() {
        assert_eq!(456576800, gxhash32(&[0u8; 0], 0));
        assert_eq!(978957914, gxhash32(&[0u8; 1], 0));
        assert_eq!(3325885698, gxhash32(&[0u8; 1000], 0));
        assert_eq!(3805815999, gxhash32(&[42u8; 4242], 42));
    }
}
