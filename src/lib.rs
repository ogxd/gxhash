#![cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#![cfg_attr(not(feature = "std"), no_std)]
// Feature 'avx2' currently requires unstable `stdsimd`
#![cfg_attr(all(feature = "avx2", target_arch = "x86_64"), feature(stdsimd))]
//! A [blazingly fast](#blazingly-fast-) and [robust](#highly-robust-) non-cryptographic hashing algorithm.
//!
//! ## Usage
//!
//! Directly as a hash function:
//!
//! ```
//! let bytes: &[u8] = "hello world".as_bytes();
//! let seed = 1234;
//!
//! println!(" 32-bit hash: {:x}", gxhash::gxhash32(&bytes, seed));
//! println!(" 64-bit hash: {:x}", gxhash::gxhash64(&bytes, seed));
//! println!("128-bit hash: {:x}", gxhash::gxhash128(&bytes, seed));
//! ```
//!
//! GxHash provides an implementation of the [`Hasher`](core::hash::Hasher) trait.
//! To construct a `HashMap` using [`GxHasher`](crate::GxHasher) as its hasher:
//!
//! ```
//! use gxhash::{GxHasher, RandomState};
//! use std::collections::HashMap;
//!
//! let mut map: HashMap<&str, i32, RandomState> = HashMap::default();
//! map.insert("answer", 42);
//! ```
//!
//! ## Cargo Features
//!
//! * `avx2` -- Enables AVX2 support for the `gxhash128` and `gxhash64` functions.
//! * `std` -- Enables the `HashMap`/`HashSet` container convenience type aliases. This is on by default. Disable to make the crate `no_std`:
//!
//!   ```toml
//!   [dependencies.gxhash]
//!   ...
//!   default-features = false
//!   ```
//!
//! ## Features
//!
//! ### Blazingly Fast 🚀
//!
//! As of this writing, GxHash is the fastest, non-cryptographic hashing algorithm of its class, for all input sizes. This performance is possible foremost due
//! to heavy usage of SIMD intrinsics, high ILP construction and a small bytecode (easily inlined and cached).
//!
//! See the [benchmarks](https://github.com/ogxd/gxhash#benchmarks).
//!
//! ### Highly Robust 🗿
//!
//! GxHash uses several rounds of hardware-accelerated AES block cipher for efficient bit mixing.
//! Thanks to this, GxHash passes all [SMHasher](https://github.com/rurban/smhasher) tests, which is the de facto quality benchmark for non-cryptographic hash
//! functions, gathering most of the existing algorithms. GxHash has low collisions, uniform distribution and high avalanche properties.
//!
//! Check out the [paper](https://github.com/ogxd/gxhash/blob/main/article/article.pdf) for more technical details.
//!
//! ## Convenience Aliases
//!
//! For interop with existing crates which require a `std::collection::HashMap` , the type aliases [`HashMap`](crate::HashMap), [`HashSet`](crate::HashSet) are
//! provided.
//!
//! ```
//! use gxhash::{HashMap, HashMapExt};
//!
//! let mut map: HashMap<&str, i32> = HashMap::new();
//! map.insert("answer", 42);
//! ```
//!
//! Note the import of [`HashMapExt`](crate::HashMapExt). This is needed for the constructor.
//!
//! ## Portability
//!
//! ### Supported Architectures
//!
//! GxHash is compatible with:
//!
//! * X86 processors with `AES-NI` intrinsics
//! * ARM processors with `NEON` intrinsics
//!
//! > **⚠️ Warning**
//! >
//! > Other platforms are currently not supported (there is no fallback). Currently the crate does not build on these. If you add support for a new platform,
//! > a PR is highly welcome.
//!
//! ### Stability of Hashes
//!
//! All generated hashes for a given version of GxHash are stable. This means that for a given input the output hash will be the same across all supported
//! platforms.
//!
//! *An exception to this is the AVX2 version of GxHash (requires a `nightly` toolchain).*
//!
//! ## Security
//!
//! ### DOS Resistance
//!
//! GxHash is a seeded hashing algorithm, meaning that depending on the seed used, it will generate completely different hashes. The default `HasherBuilder`
//! (`GxHasherBuilder::default()`) uses seed randomization, making any `HashMap`/`HashSet` more DOS resistant, as it will make it much more difficult for
//! attackers to be able to predict which hashes may collide without knowing the seed used. This does not mean however that it is completely DOS resistant.
//! This has to be analyzed further.
//!
//! ### Multicollisions Resistance
//!
//! GxHash uses a 128-bit internal state (and even 256-bit with the `avx2` feature). This makes GxHash
//! [a widepipe construction](https://en.wikipedia.org/wiki/Merkle%E2%80%93Damg%C3%A5rd_construction#Wide_pipe_construction) when generating hashes of size
//! 64-bit or smaller. Which, among other useful properties, are inherently more resistant to multicollision attacks. See
//! [this paper](https://www.iacr.org/archive/crypto2004/31520306/multicollisions.pdf) for more details.
//!
//! ### Cryptographic Properties
//!
//! GxHash is a non-cryptographic hashing algorithm, thus it is not recommended to use it as a cryptographic algorithm (it is e.g. not a replacement for SHA).
//! It has not been assessed if GxHash is preimage resistant and how difficult it is to be reversed.

#[rustfmt::skip]
mod gxhash;
mod hasher;

pub use crate::gxhash::*;
pub use crate::hasher::*;
