use std::hash::{BuildHasher, Hash, Hasher};

use crate::gxhash::*;

/// A `Hasher` for hashing an arbitrary stream of bytes.
/// # Features
/// - The fastest [`Hasher`] of its class<sup>1</sup>, for all input sizes
/// - Highly collision resistant
/// - DOS resistance thanks to seed randomization when using [`GxHasher::default()`] unless the
///   "deterministic" feature is enabled.
///
/// *<sup>1</sup>There might be faster alternatives, such as `fxhash` for very small input sizes,
/// but that usually have low quality properties.*
#[derive(Clone, Debug)]
pub struct GxHasher {
    state: State,
}

impl GxHasher {
    #[inline]
    fn with_state(state: State) -> GxHasher {
        GxHasher { state }
    }
}

impl Default for GxHasher {
    /// Creates a new hasher with an empty seed.
    ///
    /// # Warning ⚠️
    /// Not using a seed may make your [`Hasher`] vulnerable to DOS attacks.
    /// It is recommended to use [`GxBuildHasher::default()`] for improved DOS resistance.
    ///
    /// # Example
    ///
    /// ```
    /// use std::hash::Hasher;
    /// use gxhash::GxHasher;
    ///
    /// let mut hasher = GxHasher::default();
    ///
    /// hasher.write(b"Hello");
    /// hasher.write_u32(123);
    /// hasher.write_u8(42);
    ///
    /// println!("Hash is {:x}!", hasher.finish());
    /// ```
    #[inline]
    fn default() -> GxHasher {
        GxHasher::with_state(from_u128(0))
    }
}

impl GxHasher {
    /// Creates a new hasher using the provided seed.
    ///
    /// # Warning ⚠️
    /// Hardcoding a seed may make your [`Hasher`] vulnerable to DOS attacks.
    /// It is recommended to use [`GxBuildHasher::default()`] for improved DOS resistance.
    ///
    /// # Example
    ///
    /// ```
    /// use std::hash::Hasher;
    /// use gxhash::GxHasher;
    ///
    /// let mut hasher = GxHasher::with_seed(1234);
    ///
    /// hasher.write(b"Hello");
    /// hasher.write_u32(123);
    /// hasher.write_u8(42);
    ///
    /// println!("Hash is {:x}!", hasher.finish());
    /// ```
    #[inline]
    pub fn with_seed(seed: i64) -> GxHasher {
        // Use gxhash64 to generate an initial state from a seed
        GxHasher::with_state(create_seed(seed))
    }

    /// Finish this hasher and return the hashed value as a 128-bit
    /// unsigned integer.
    #[inline]
    pub fn finish_u128(&self) -> u128 {
        to_u128(dispatched::finalize(self.state))
    }
}

// Implements Hasher with the operations of a backend. Integers are zero-extended to 64 bits, and signed integers are
// written as unsigned integers (default implementation).
macro_rules! impl_hasher {
    ($hasher:ident, $($ops:ident)::+) => {
        #[allow(unused_unsafe)]
        impl Hasher for $hasher {
            #[inline]
            fn finish(&self) -> u64 {
                to_u128(unsafe { $($ops)::+::finalize(self.state) }) as u64
            }

            #[inline]
            fn write(&mut self, bytes: &[u8]) {
                self.state = unsafe { $($ops)::+::absorb(self.state, bytes) };
            }

            #[inline]
            fn write_u8(&mut self, value: u8) {
                self.write_u64(value as u64);
            }

            #[inline]
            fn write_u16(&mut self, value: u16) {
                self.write_u64(value as u64);
            }

            #[inline]
            fn write_u32(&mut self, value: u32) {
                self.write_u64(value as u64);
            }

            #[inline]
            fn write_u64(&mut self, value: u64) {
                self.state = unsafe { $($ops)::+::absorb_u64(self.state, value) };
            }

            #[inline]
            fn write_u128(&mut self, value: u128) {
                self.state = unsafe { $($ops)::+::absorb_u128(self.state, value) };
            }
        }
    };
}

impl_hasher!(GxHasher, dispatched);

// Hasher of the hardware backend, once detected at runtime (see hash_one)
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
struct HwHasher {
    state: State,
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
impl_hasher!(HwHasher, hw::outlined);

/// A builder for building GxHasher with randomized seeds by default, for improved DOS resistance.
#[derive(Clone, Debug)]
pub struct GxBuildHasher(State);

#[cfg(not(feature = "deterministic"))]
use std::hash::RandomState;

impl GxBuildHasher {
    /// Creates a new builder using the provided seed.
    ///
    /// # Warning ⚠️
    /// Hardcoding a seed may make your [`Hasher`] vulnerable to DOS attacks.
    /// It is recommended to use [`GxBuildHasher::default()`] for improved DOS resistance.
    #[inline]
    pub fn with_seed(seed: i64) -> GxBuildHasher {
        // Use gxhash64 to generate an initial state from a seed
        GxBuildHasher(create_seed(seed))
    }
}

impl Default for GxBuildHasher {
    #[inline]
    fn default() -> GxBuildHasher {
        #[cfg(feature = "deterministic")]
        let state = from_u128(42);
        #[cfg(not(feature = "deterministic"))]
        let state = unsafe { std::mem::transmute::<RandomState, State>(RandomState::new()) };
        GxBuildHasher(state)
    }
}

impl BuildHasher for GxBuildHasher {
    type Hasher = GxHasher;
    #[inline]
    fn build_hasher(&self) -> GxHasher {
        GxHasher::with_state(self.0)
    }

    // A single runtime dispatch for all the writes of the value, rather than one per write
    #[inline]
    fn hash_one<T: Hash>(&self, x: T) -> u64 {
        with_backend(HashOne(self.0, x))
    }
}

struct HashOne<T>(State, T);

impl<T: Hash> Run for HashOne<T> {
    type Output = u64;

    #[inline(always)]
    fn run(self) -> u64 {
        hash_with(GxHasher::with_state(self.0), self.1)
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
    #[inline(always)]
    unsafe fn run_hw(self) -> u64 {
        hash_with(HwHasher { state: self.0 }, self.1)
    }
}

#[inline(always)]
fn hash_with<H: Hasher, T: Hash>(mut hasher: H, x: T) -> u64 {
    x.hash(&mut hasher);
    hasher.finish()
}

/// A `HashMap` using a (DOS-resistant) [`GxBuildHasher`].
pub type HashMap<K, V> = std::collections::HashMap<K, V, GxBuildHasher>;

/// A convenience trait that can be used together with the type aliases defined
/// to get access to the `new()` and `with_capacity()` methods for the
/// [`HashMap`] type alias.
pub trait HashMapExt {
    /// Constructs a new HashMap.
    fn new() -> Self;
    /// Constructs a new HashMap with a given initial capacity.
    fn with_capacity(capacity: usize) -> Self;
}

impl<K, V, S> HashMapExt for std::collections::HashMap<K, V, S>
where
    S: BuildHasher + Default,
{
    fn new() -> Self {
        std::collections::HashMap::with_hasher(S::default())
    }

    fn with_capacity(capacity: usize) -> Self {
        std::collections::HashMap::with_capacity_and_hasher(capacity, S::default())
    }
}

/// A `HashSet` using a (DOS-resistant) [`GxBuildHasher`].
pub type HashSet<T> = std::collections::HashSet<T, GxBuildHasher>;

/// A convenience trait that can be used together with the type aliases defined
/// to get access to the `new()` and `with_capacity()` methods for the
/// [`HashSet`] type alias.
pub trait HashSetExt {
    /// Constructs a new HashMap.
    fn new() -> Self;
    /// Constructs a new HashMap with a given initial capacity.
    fn with_capacity(capacity: usize) -> Self;
}

impl<K, S> HashSetExt for std::collections::HashSet<K, S>
    where S: BuildHasher + Default,
{
    fn new() -> Self {
        std::collections::HashSet::with_hasher(S::default())
    }

    fn with_capacity(capacity: usize) -> Self {
        std::collections::HashSet::with_capacity_and_hasher(capacity, S::default())
    }
}

#[cfg(test)]
mod tests {

    use std::hash::Hash;

    use super::*;

    #[test]
    fn contructors_work() {
        let mut map: std::collections::HashMap<&str, i32, GxBuildHasher> = HashMap::new();
        assert_eq!(true, map.insert("foo", 1).is_none());

        let mut map = HashMap::with_capacity(3);
        assert_eq!(3, map.capacity());
        assert_eq!(true, map.insert("bar", 2).is_none());

        let mut set: std::collections::HashSet<i32, GxBuildHasher> = HashSet::new();
        assert_eq!(true, set.insert(42));

        let mut set = HashSet::with_capacity(3);
        assert_eq!(true, set.insert(42));
        assert_eq!(3, set.capacity());
    }

    #[test]
    fn hasher_produces_stable_hashes() {
        let mut hashset = HashSet::default();
        assert!(hashset.insert(1234));
        assert!(!hashset.insert(1234));
        assert!(hashset.insert(42));

        let mut hashset = HashSet::default();
        assert!(hashset.insert("hello"));
        assert!(hashset.insert("world"));
        assert!(!hashset.insert("hello"));
        assert!(hashset.insert("bye"));
    }

    // By no mean a quality test, but rather a sanity check
    #[test]
    fn hasher_resists_permutations() {
        let build_hasher = GxBuildHasher::default();
        let mut hasher1 = build_hasher.build_hasher();
        (1, 2).hash(&mut hasher1);
        let mut hasher2 = build_hasher.build_hasher();
        (2, 1).hash(&mut hasher2);
        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    // This is important for DOS resistance
    #[test]
    fn gxhashset_uses_default_gxhasherbuilder() {
        let hashset_1 = HashSet::<u32>::default();
        let hashset_2 = HashSet::<u32>::default();

        let mut hasher_1 = hashset_1.hasher().build_hasher();
        let mut hasher_2 = hashset_2.hasher().build_hasher();

        hasher_1.write_i32(42);
        let hash_1 = hasher_1.finish();

        hasher_2.write_i32(42);
        let hash_2 = hasher_2.finish();

        if cfg!(feature = "deterministic") {
            assert_eq!(hash_1, hash_2);
        } else {
            assert_ne!(hash_1, hash_2);
        }
    }

    // This is important for DOS resistance
    #[test]
    fn default_gxhasherbuilder_is_randomly_seeded() {
        let buildhasher_1 = GxBuildHasher::default();
        let buildhasher_2 = GxBuildHasher::default();

        let mut hasher_1 = buildhasher_1.build_hasher();
        let mut hasher_2 = buildhasher_2.build_hasher();

        hasher_1.write_i32(42);
        let hash_1 = hasher_1.finish();

        hasher_2.write_i32(42);
        let hash_2 = hasher_2.finish();

        if cfg!(feature = "deterministic") {
            assert_eq!(hash_1, hash_2);
        } else { 
            assert_ne!(hash_1, hash_2);
        }
    }

    #[test]
    fn hasher_is_stable() {
        // Hashes must be the same on all platforms and backends
        let data: Vec<u8> = (0..300usize).map(|i| (i * 31 + 7) as u8).collect();
        let mut hasher = GxHasher::with_seed(1234);
        hasher.write_u8(1);
        hasher.write_u16(2);
        hasher.write_u32(3);
        hasher.write_u64(4);
        hasher.write_u128(5);
        hasher.write(&data[..15]);
        hasher.write(&data[..16]);
        hasher.write(&data[..40]);
        hasher.write(&data[..300]);
        assert_eq!(0x9b9eb762bf1373aa, hasher.finish());
        assert_eq!(0xc12991e14a52e3289b9eb762bf1373aa, hasher.finish_u128());
    }

    #[test]
    fn hasher_inputs_of_different_lengths_do_not_collide() {
        let build_hasher = GxBuildHasher::default();
        let hash = |bytes: &[u8]| {
            let mut hasher = build_hasher.build_hasher();
            hasher.write(bytes);
            hasher.finish()
        };
        for len in 0..16u8 {
            // An input of less than 16 bytes vs the same bytes followed by its padding
            let short: Vec<u8> = (0..len).collect();
            let mut long = short.clone();
            long.resize(16, len);
            assert_ne!(hash(&short), hash(&long), "length {len}");
        }
        assert_ne!(hash(&[0u8; 20]), hash(&[0u8; 21]));
    }

    #[test]
    fn gxhasherbuilder_builds_same_hashers() {
        let buildhasher = GxBuildHasher::default();

        let mut hasher = buildhasher.build_hasher();

        hasher.write_i32(42);
        let hash = hasher.finish();

        let mut hasher = buildhasher.build_hasher();

        hasher.write_i32(42);
        assert_eq!(hash, hasher.finish());
    }
}
