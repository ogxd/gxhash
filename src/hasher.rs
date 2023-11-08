use std::hash::{Hasher, BuildHasherDefault};
use std::collections::{HashMap, HashSet};
use crate::gxhash::*;
use crate::gxhash::platform::*;

pub struct GxHasher(State);

impl Default for GxHasher {
    #[inline]
    fn default() -> GxHasher {
        GxHasher(unsafe { create_empty() })
    }
}

impl GxHasher {
    #[inline]
    pub fn with_seed(seed: i32) -> GxHasher {
        // Use gxhash64 to generate an initial state from a seed
        GxHasher(unsafe { gxhash(&[], create_seed(seed)) })
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

/// A builder for default FNV hashers.
pub type GxBuildHasher = BuildHasherDefault<GxHasher>;

/// A `HashMap` using a default GxHash hasher.
//#[cfg(feature = "std")]
pub type GxHashMap<K, V> = HashMap<K, V, GxBuildHasher>;

/// A `HashSet` using a default GxHash hasher.
//#[cfg(feature = "std")]
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