use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hasher};
use std::mem::MaybeUninit;

use rand::RngCore;

use crate::gxhash::platform::*;
use crate::gxhash::*;

/// A `Hasher` for hashing an arbitrary stream of bytes.
/// # Features
/// - The fastest [`Hasher`] of its class<sup>1</sup>, for all input sizes
/// - Highly collision resitant
/// - DOS resistance thanks to seed randomization when using [`GxHasher::default()`]
///
/// *<sup>1</sup>There might me faster alternatives, such as `fxhash` for very small input sizes, but that usually have low quality properties.*
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
    /// Creates a new hasher with a empty seed.
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
        GxHasher::with_state(unsafe { create_empty() })
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
        GxHasher::with_state(unsafe { create_seed(seed) })
    }

    /// Finish this hasher and return the hashed value as a 128 bit
    /// unsigned integer.
    #[inline]
    pub fn finish_u128(&self) -> u128 {
        debug_assert!(std::mem::size_of::<State>() >= std::mem::size_of::<u128>());

        unsafe {
            let p = &finalize(self.state) as *const State as *const u128;
            *p
        }
    }
}

macro_rules! write {
    ($name:ident, $type:ty, $load:expr) => {
        #[inline]
        fn $name(&mut self, value: $type) {
            self.state = unsafe {
                aes_encrypt_last($load(value), aes_encrypt(self.state, ld(KEYS.as_ptr())))
            };
        }
    }
}

impl Hasher for GxHasher {
    #[inline]
    fn finish(&self) -> u64 {
        unsafe {
            let p = &finalize(self.state) as *const State as *const u64;
            *p
        }
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // Improvement: only compress at this stage and finalize in finish
        self.state = unsafe { aes_encrypt_last(compress_all(bytes), aes_encrypt(self.state, ld(KEYS.as_ptr()))) };
    }

    write!(write_u8, u8, load_u8);
    write!(write_u16, u16, load_u16);
    write!(write_u32, u32, load_u32);
    write!(write_u64, u64, load_u64);
    write!(write_u128, u128, load_u128);
    write!(write_i8, i8, load_i8);
    write!(write_i16, i16, load_i16);
    write!(write_i32, i32, load_i32);
    write!(write_i64, i64, load_i64);
    write!(write_i128, i128, load_i128);
}

/// A builder for building GxHasher with randomized seeds by default, for improved DOS resistance.
#[derive(Clone, Debug)]
pub struct GxBuildHasher(State);

impl Default for GxBuildHasher {
    #[inline]
    fn default() -> GxBuildHasher {
        let mut uninit: MaybeUninit<State> = MaybeUninit::uninit();
        let mut rng = rand::thread_rng();
        unsafe {
            let ptr = uninit.as_mut_ptr() as *mut u8;
            let slice = std::slice::from_raw_parts_mut(ptr, VECTOR_SIZE);
            rng.fill_bytes(slice);
            GxBuildHasher(uninit.assume_init())
        }
    }
}

impl BuildHasher for GxBuildHasher {
    type Hasher = GxHasher;
    #[inline]
    fn build_hasher(&self) -> GxHasher {
        GxHasher::with_state(self.0)
    }
}

/// A `HashMap` using a (DOS-resistant) [`GxBuildHasher`].
pub type GxHashMap<K, V> = HashMap<K, V, GxBuildHasher>;

/// A `HashSet` using a (DOS-resistant) [`GxBuildHasher`].
pub type GxHashSet<T> = HashSet<T, GxBuildHasher>;

#[cfg(test)]
mod tests {

    use std::hash::Hash;

    use super::*;

    #[test]
    fn hasher_produces_stable_hashes() {
        let mut hashset = GxHashSet::default();
        assert!(hashset.insert(1234));
        assert!(!hashset.insert(1234));
        assert!(hashset.insert(42));

        let mut hashset = GxHashSet::default();
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
        let hashset_1 = GxHashSet::<u32>::default();
        let hashset_2 = GxHashSet::<u32>::default();

        let mut hasher_1 = hashset_1.hasher().build_hasher();
        let mut hasher_2 = hashset_2.hasher().build_hasher();

        hasher_1.write_i32(42);
        let hash_1 = hasher_1.finish();

        hasher_2.write_i32(42);
        let hash_2 = hasher_2.finish();

        assert_ne!(hash_1, hash_2);
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

        assert_ne!(hash_1, hash_2);
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
