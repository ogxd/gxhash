pub mod s128;

#[cfg(feature = "s256")]
pub mod s256;

use std::mem::size_of;

use self::s128::Adapter128;

// 4KiB is the default page size for most systems, and conservative for other systems such as MacOS ARM (16KiB)
const PAGE_SIZE: usize = 0x1000;

pub trait BlockProcessor {
    type State;
    unsafe fn compress_8(ptr: *const Self::State, unrollable_blocks_count: usize, hash_vector: Self::State) -> Self::State;
}

pub trait Adapter {
    type State;

    const VECTOR_SIZE: usize = size_of::<Self::State>();

    unsafe fn create_empty() -> Self::State;
    unsafe fn create_seed(seed: i64) -> Self::State;
    unsafe fn load_unaligned(p: *const Self::State) -> Self::State;
    unsafe fn get_partial_safe(data: *const Self::State, len: usize) -> Self::State;
    unsafe fn get_partial_unsafe(data: *const Self::State, len: usize) -> Self::State;
    unsafe fn compress(a: Self::State, b: Self::State) -> Self::State;
    unsafe fn compress_fast(a: Self::State, b: Self::State) -> Self::State;
    unsafe fn finalize(hash: Self::State) -> Self::State;

    #[inline(always)]
    unsafe fn get_partial(p: *const Self::State, len: usize) -> Self::State {
        if Self::check_same_page(p) {
            Self::get_partial_unsafe(p, len)
        } else {
            Self::get_partial_safe(p, len)
        }
    }

    #[inline(always)]
    unsafe fn check_same_page(ptr: *const Self::State) -> bool {
        let address = ptr as usize;
        // Mask to keep only the last 12 bits
        let offset_within_page = address & (PAGE_SIZE - 1);
        // Check if the 16nd byte from the current offset exceeds the page boundary
        offset_within_page < PAGE_SIZE - Self::VECTOR_SIZE
    }
}

pub mod auto {

    #[cfg(not(feature = "s256"))]
    pub type AdapterWidest = crate::s128::Adapter128;
    #[cfg(feature = "s256")]
    pub type AdapterWidest = crate::s256::Adapter256;

    pub fn gxhash32(input: &[u8], seed: i64) -> u32 {
        super::gxhash32::<AdapterWidest>(input, seed)
    }

    pub fn gxhash64(input: &[u8], seed: i64) -> u64 {
        super::gxhash64::<AdapterWidest>(input, seed)
    }

    pub fn gxhash128(input: &[u8], seed: i64) -> u128 {
        super::gxhash128::<AdapterWidest>(input, seed)
    }
}

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
pub(crate) fn gxhash32<T>(input: &[u8], seed: i64) -> u32
    where T: Adapter {
    unsafe {
        let p = &gxhash::<T>(input, T::create_seed(seed)) as *const T::State as *const u32;
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
pub fn gxhash64<T>(input: &[u8], seed: i64) -> u64
    where T: Adapter {
    unsafe {
        let p = &gxhash::<T>(input, T::create_seed(seed)) as *const T::State as *const u64;
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
pub fn gxhash128<T>(input: &[u8], seed: i64) -> u128
    where T: Adapter {
    unsafe {
        let p = &gxhash::<T>(input, T::create_seed(seed)) as *const T::State as *const u128;
        *p
    }
}

macro_rules! load_unaligned {
    ($name:ty, $ptr:ident, $($var:ident),+) => {
        $(
            #[allow(unused_mut)]
            let mut $var = <$name>::load_unaligned($ptr);
            $ptr = ($ptr).offset(1);
        )+
    };
}

type State128 = <Adapter128 as Adapter>::State;

#[inline(always)]
pub(crate) unsafe fn gxhash<T>(input: &[u8], seed: State128) -> State128
    where T: Adapter {
    T::finalize(T::compress_fast(compress_all::<T>(input), seed))
}

#[inline(always)]
pub(crate) unsafe fn compress_all<T>(input: &[u8]) -> State128
    where T: Adapter {

    let len = input.len();
    let mut ptr = input.as_ptr() as *const State128;

    if len <= Adapter128::VECTOR_SIZE {
        // Input fits on a single SIMD vector, however we might read beyond the input message
        // Thus we need this safe method that checks if it can safely read beyond or must copy
        return Adapter128::get_partial(ptr, len);
    }

    let remaining_bytes = len % Adapter128::VECTOR_SIZE;

    // The input does not fit on a single SIMD vector
    let hash_vector: State128;
    if remaining_bytes == 0 {
        load_unaligned!(Adapter128, ptr, v0);
        hash_vector = v0;
    } else {
        // If the input length does not match the length of a whole number of SIMD vectors,
        // it means we'll need to read a partial vector. We can start with the partial vector first,
        // so that we can safely read beyond since we expect the following bytes to still be part of
        // the input
        hash_vector = Adapter128::get_partial_unsafe(ptr, remaining_bytes);
        ptr = ptr.cast::<u8>().add(remaining_bytes).cast();
    }

    #[allow(unused_assignments)]
    if len <= Adapter128::VECTOR_SIZE * 2 {
        // Fast path when input length > 16 and <= 32
        load_unaligned!(Adapter128, ptr, v0);
        Adapter128::compress(hash_vector, v0)
    } else if len <= T::VECTOR_SIZE * 3 {
        // Fast path when input length > 32 and <= 48
        load_unaligned!(Adapter128, ptr, v0, v1);
        Adapter128::compress(hash_vector, Adapter128::compress(v0, v1))
    } else if len <= T::VECTOR_SIZE * 4 {
        // Fast path when input length > 48 and <= 64
        load_unaligned!(Adapter128, ptr, v0, v1, v2);
        Adapter128::compress(hash_vector, Adapter128::compress(Adapter128::compress(v0, v1), v2))
    } else {
        // Input message is large and we can use the high ILP loop
        compress_many::<T>(ptr, hash_vector, len)
    }
}

#[inline(always)]
unsafe fn compress_many<T>(mut ptr: *const State128, hash_vector: State128, remaining_bytes: usize) -> State128
    where T: Adapter+BlockProcessor<State = <T as Adapter>::State> {

    const UNROLL_FACTOR: usize = 8;

    let unrollable_blocks_count: usize = remaining_bytes / (T::VECTOR_SIZE * UNROLL_FACTOR) * UNROLL_FACTOR;

    let remaining_bytes = remaining_bytes - unrollable_blocks_count * T::VECTOR_SIZE;
    let end_address = ptr.add(remaining_bytes / T::VECTOR_SIZE) as usize;

    let mut hash_vector = hash_vector;
    while (ptr as usize) < end_address {
        load_unaligned!(Adapter128, ptr, v0);
        hash_vector = Adapter128::compress(hash_vector, v0);
    }

    T::compress_8(ptr, unrollable_blocks_count, hash_vector)
}

#[cfg(test)]
mod tests {

    use crate::s128::Adapter128;

    use super::*;
    use rand::Rng;
    use rstest::rstest;

    #[test]
    fn all_blocks_are_consumed() {
        for s in 1..1200 {
            let mut bytes = vec![42u8; s];
            let ref_hash = gxhash32::<Adapter128>(&bytes, 0);
    
            for i in 0..bytes.len() {
                let swap = bytes[i];
                bytes[i] = 82;
                let new_hash = gxhash32::<Adapter128>(&bytes, 0);
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
            let new_hash = gxhash32::<Adapter128>(&mut bytes[..i], 0);
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
            set.insert(gxhash64::<Adapter128>(&bytes, 0));

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
        assert_ne!(0, gxhash32::<Adapter128>(&[0u8; 0], 0));
        assert_ne!(0, gxhash32::<Adapter128>(&[0u8; 1], 0));
        assert_ne!(0, gxhash32::<Adapter128>(&[0u8; 1200], 0));
    }

    // GxHash with a 128-bit state must be stable despite the different endianesses / CPU instrinsics
    #[cfg(not(feature = "s256"))]
    #[test]
    fn is_stable() {
        assert_eq!(456576800, gxhash32::<Adapter128>(&[0u8; 0], 0));
        assert_eq!(978957914, gxhash32::<Adapter128>(&[0u8; 1], 0));
        assert_eq!(3325885698, gxhash32::<Adapter128>(&[0u8; 1000], 0));
        assert_eq!(3805815999, gxhash32::<Adapter128>(&[42u8; 4242], 42));
    }
}
