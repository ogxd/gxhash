use std::hash::{Hasher, BuildHasher};
use std::collections::{HashMap, HashSet};

use rand::Rng;

use crate::gxhash::*;
use crate::gxhash::platform::*;

/// A `Hasher` for hashing an arbitrary stream of bytes.
/// # Features
/// - The fastest [`Hasher`] of its class<sup>1</sup>, for all input sizes
/// - Highly collision resitant
/// - DOS resistant thanks to seed randomization when using [`GxHasher::default()`]
/// 
/// *<sup>1</sup>There might me faster alternatives, such as `fxhash` for very small input sizes, but that usually have low quality properties.*
pub struct GxHasher(State);

impl Default for GxHasher {
    /// Creates a new hasher with a empty seed.
    /// 
    /// # Warning ⚠️
    /// Not using a seed may make your [`Hasher`] vulnerable to DOS attacks.  
    /// It is recommended to use [`GxBuildHasher::default()`] to create a DOS resistant [`Hasher`]s.
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
        GxHasher(unsafe { create_empty() })
    }
}

impl GxHasher {
    /// Creates a new hasher using the provided seed.
    /// 
    /// # Warning ⚠️
    /// Hardcoding a seed may make your [`Hasher`] vulnerable to DOS attacks.  
    /// It is recommended to use [`GxBuildHasher::default()`] to create a DOS resistant [`Hasher`]s.
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
        GxHasher(unsafe { gxhash(&[], create_seed(seed)) })
    }

    /// Finish this hasher and return the hashed value as a 128 bit
    /// unsigned integer.
    #[inline]
    pub fn finish_u128(&self) -> u128 {
        debug_assert!(std::mem::size_of::<State>() >= std::mem::size_of::<u128>());

        unsafe {
            let p = &self.0 as *const State as *const u128;
            *p
        }
    }
}

impl Hasher for GxHasher {
    #[inline]
    fn finish(&self) -> u64 {
        unsafe {
            let p = &self.0 as *const State as *const u64;
            *p
        }
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // Improvement: only compress at this stage and finalize in finish
        self.0 = unsafe { gxhash(bytes, self.0) };
    }
}

/// A builder for building DOS-resistant [`GxHasher`]s.
pub struct GxBuildHasher(i64);

impl Default for GxBuildHasher {
    #[inline]
    fn default() -> GxBuildHasher {
        let mut rng = rand::thread_rng();
        let seed: i64 = rng.gen::<i64>();
        GxBuildHasher(seed)
    }
}

impl BuildHasher for GxBuildHasher {
    type Hasher = GxHasher;

    fn build_hasher(&self) -> GxHasher {
        GxHasher::with_seed(self.0)
    }
}

/// A `HashMap` using a (DOS-resistant) [`GxBuildHasher`].
pub type GxHashMap<K, V> = HashMap<K, V, GxBuildHasher>;

/// A `HashSet` using a (DOS-resistant) [`GxBuildHasher`].
pub type GxHashSet<T> = HashSet<T, GxBuildHasher>;

#[cfg(test)]
mod tests {

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