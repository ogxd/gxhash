use std::hash::{Hasher, BuildHasherDefault};
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
    /// Create a new hasher with a random seed for DOS resistance.
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
        let mut rng = rand::thread_rng();
        let seed: i64 = rng.gen::<i64>();
        GxHasher(unsafe { create_seed(seed) })
    }
}

impl GxHasher {
    /// Creates a new hasher using the provided seed.
    /// 
    /// # Warning ⚠️
    /// Hardcoding a seed may make your [`Hasher`] vulnerable to DOS attacks.  
    /// It is recommended to use [`GxHasher::default()`] to create a DOS resistant [`Hasher`].
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

/// A builder for default GxHash hashers.
pub type GxBuildHasher = BuildHasherDefault<GxHasher>;

/// A `HashMap` using a default GxHash hasher.
pub type GxHashMap<K, V> = HashMap<K, V, GxBuildHasher>;

/// A `HashSet` using a default GxHash hasher.
pub type GxHashSet<T> = HashSet<T, GxBuildHasher>;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn hasher_works() {
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
}