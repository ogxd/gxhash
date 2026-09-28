# GxHash
[![Crates.io](https://img.shields.io/crates/v/gxhash.svg)](https://crates.io/crates/gxhash)
[![Docs.rs](https://docs.rs/gxhash/badge.svg)](https://docs.rs/gxhash)
[![Build & Test](https://github.com/ogxd/gxhash/actions/workflows/build_test.yml/badge.svg)](https://github.com/ogxd/gxhash/actions/workflows/build_test.yml)
[![Cross Compile](https://github.com/ogxd/gxhash/actions/workflows/cross_compile.yml/badge.svg)](https://github.com/ogxd/gxhash/actions/workflows/cross_compile.yml)
[![Rust Version Compatibility](https://github.com/ogxd/gxhash/actions/workflows/rust_version.yml/badge.svg)](https://github.com/ogxd/gxhash/actions/workflows/rust_version.yml)

GxHash is a fast, high-quality non-cryptographic hash function for Rust, and a drop-in `Hasher` for `HashMap` and `HashSet`. It uses AES instructions on x86_64 and ARM to hash inputs of any size at very high throughput.

- **Fast**: in our [benchmarks](#benchmarks) on an Apple M5 Pro, GxHash is the fastest of the hash functions tested (XXH3, AHash, FoldHash, FxHash, T1ha, MetroHash, FNV-1a) for inputs of 32 bytes and more, and its lead grows with the input size: it hashes 1 KiB at 146 GiB/s, 2.9 times the next fastest. On integers and short strings, it is within half a nanosecond of the fastest.
- **High quality**: GxHash passes all [SMHasher](https://github.com/rurban/smhasher) tests, the reference quality test suite for non-cryptographic hash functions (collisions, distribution, avalanche).
- **Works everywhere**: `cargo add gxhash` works on every target, with no build flags. GxHash detects AES instructions at runtime, and falls back to a portable implementation on CPUs without them. See [Portability and Hardware Acceleration](#portability-and-hardware-acceleration).
- **Stable hashes**: for a given major version, the hash of an input is the same on every platform and CPU, with or without hardware acceleration.
- **Seeded**: GxHash's `HashMap` and `HashSet` are randomly seeded, like the standard library's, which mitigates HashDoS attacks.
- **Checked with Miri**: the tests run under [Miri](https://github.com/rust-lang/miri) in CI, on the x86_64 hardware and software backends. See [Safety and Soundness](#safety-and-soundness).
- **Lightweight**: no runtime dependencies and `no_std` support.

**[Usage](#usage) | [When to Use GxHash](#when-to-use-gxhash) | [Benchmarks](#benchmarks) | [Safety](#safety-and-soundness) | [Portability](#portability-and-hardware-acceleration) | [Contributing](#contributing)**

## Usage
```bash
cargo add gxhash
```
GxHash requires Rust 1.89 or later.

As a hash function, with 32, 64 or 128-bit outputs:
```rust
let bytes: &[u8] = "hello world".as_bytes();
let seed = 1234;
println!(" 32-bit hash: {:x}", gxhash::gxhash32(&bytes, seed));
println!(" 64-bit hash: {:x}", gxhash::gxhash64(&bytes, seed));
println!("128-bit hash: {:x}", gxhash::gxhash128(&bytes, seed));
```

As the hasher of a `HashMap` or `HashSet`, with the `gxhash::HashMap` and `gxhash::HashSet` aliases:
```rust
use gxhash::{HashMap, HashMapExt};

let mut map: HashMap<&str, i32> = HashMap::new();
map.insert("answer", 42);
```

Or with `GxBuildHasher`, which works with any collection taking a [`BuildHasher`](https://doc.rust-lang.org/std/hash/trait.BuildHasher.html), such as the standard library's or [hashbrown](https://crates.io/crates/hashbrown)'s:
```rust
use std::collections::HashMap;
use gxhash::GxBuildHasher;

let mut map: HashMap<String, i32, GxBuildHasher> = HashMap::default();
map.insert("answer".to_owned(), 42);
```

### Upgrading from 3.x
- GxHash 3.x failed to compile unless AES instructions were enabled at compile time (`-C target-cpu=native`). Since 4.0, no flags are needed: they only make GxHash faster.
- The `hybrid` feature is removed: 256-bit AES instructions (VAES) are used automatically when available.
- Hashes are different from 3.x, as for any major version.

## When to Use GxHash

GxHash is a good default when:
- Keys are strings, byte slices or composite keys: GxHash is the fastest we measured from 32 bytes, and close to the fastest below.
- You hash buffers, such as file chunks, network payloads, or content for deduplication, Bloom filters or sharding, and want the highest throughput with SMHasher-level quality.
- You need hashes that are the same on every machine, for instance to partition data across a cluster, or 128-bit hashes (`gxhash128`) for fewer collisions.

Consider something else when:
- Keys are only integers or short strings, and every fraction of a nanosecond matters: FxHash ([rustc-hash](https://crates.io/crates/rustc-hash)) and [FoldHash](https://crates.io/crates/foldhash) were up to 0.5 ns faster per lookup in our [HashSet benchmark](#benchmarks). FxHash is unseeded by default and trades hash quality for speed.
- Keys come from untrusted input and HashDoS resistance is a hard requirement: use the standard library's default hasher (SipHash-1-3). See [Security](#security).
- You need a cryptographic hash (signatures, integrity against tampering, passwords): use SHA-2, SHA-3 or BLAKE3.
- Hashes are persisted and must never change across upgrades: GxHash output can change between major versions. Pin the major version, or use an algorithm with a frozen specification such as XXH3.
- The CPU has no AES instructions (32-bit ARM, RISC-V, WebAssembly, some old or low-end x86): GxHash works, but is 10 to 40 times slower there. See [Portability](#portability-and-hardware-acceleration).

## Benchmarks

[![Benchmark](https://github.com/ogxd/gxhash/actions/workflows/bench.yml/badge.svg)](https://github.com/ogxd/gxhash/actions/workflows/bench.yml)

The results below are generated by the [Benchmark workflow](https://github.com/ogxd/gxhash/actions/workflows/bench.yml). All hash functions are built with `-C target-cpu=native`, which enables AES instructions at compile time ([tier 1](#portability-and-hardware-acceleration)). In bold, the fastest for each row.
- **HashSet lookup**: time to look up a key in a `HashSet` using each hasher, in nanoseconds (lower is better). The key is not in the set, so that no key comparison adds to the hashing time.
- **Throughput**: bytes hashed per second, in GiB/s (higher is better), for inputs of 4 bytes to 32 KiB. The last column is the throughput of GxHash divided by the throughput of the fastest other function that passes the quality tests.
- **❌**: the hash function fails the quality tests of `cargo bench --bench quality`, which checks avalanche, distribution and collisions on byte slices and on integer keys. FxHash fails avalanche on integer keys: some input bits change a single output bit, and 32,768 keys that differ only in their top 15 bits fall in 32 of 65,536 `HashMap` buckets, which makes inserting them 60 times slower. FoldHash (fast) fails avalanche on integer keys and on inputs of up to 10 bytes. FNV-1a fails avalanche at all sizes. All others pass.

Performance depends on the hardware and on how hashing is used: if it matters for your application, measure it in your own context.

### aarch64

<!-- bench:aarch64 -->
Neoverse-N2, rustc 1.98.1, 2026-09-28

| Key type (ns per lookup) | GxHash | std (SipHash-1-3) | FoldHash (fast) ❌ | FoldHash (quality) | FxHasher (rustc_hash) ❌ | AHash | XxHash (XXH3) | T1ha | FNV-1a ❌ | HighwayHash | MetroHash |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| u32 | 2.50 | 11.06 | 2.09 | 2.47 | **1.82** | 2.64 | 29.27 | 13.43 | 2.73 | 51.45 | 6.38 |
| u64 | 2.49 | 14.38 | 2.10 | 2.47 | **1.84** | 2.63 | 29.26 | 5.34 | 4.54 | 48.10 | 6.40 |
| u128 | 2.68 | 16.47 | 2.20 | 2.59 | **2.01** | 3.37 | 30.07 | 6.04 | 9.72 | 48.76 | 7.87 |
| String, 6 bytes | 6.80 | 13.68 | 4.85 | 5.55 | **4.60** | 5.70 | 33.69 | 13.99 | 5.77 | 59.01 | 10.46 |
| String, 30 bytes | 7.07 | 18.71 | 6.77 | 7.49 | **6.04** | 7.31 | 34.34 | 14.66 | 21.32 | 60.89 | 18.19 |
| String, 128 bytes | **9.45** | 40.76 | 11.63 | 15.28 | 12.28 | 16.12 | 49.48 | 18.04 | 101.50 | 66.13 | 20.02 |
| String, 1024 bytes | **25.21** | 238.28 | 44.55 | 45.80 | 70.28 | 93.61 | 115.43 | 62.22 | 889.24 | 212.21 | 56.80 |

| Throughput (GiB/s) | GxHash | XxHash (XXH3) | FxHasher (rustc_hash) ❌ | AHash | T1ha0 | FoldHash (fast) ❌ | FoldHash (quality) | FNV-1a ❌ | MetroHash | GxHash speedup |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 4 B | 2.40 | 1.00 | **2.46** | 1.20 | 1.05 | 1.40 | 1.12 | 1.64 | 0.85 | 1.99× |
| 8 B | 4.79 | 2.00 | **4.82** | 2.41 | 2.11 | 2.81 | 2.26 | 2.29 | 1.74 | 1.99× |
| 16 B | 9.36 | 3.95 | **9.63** | 4.81 | 3.33 | 5.60 | 4.51 | 2.12 | 2.07 | 1.95× |
| 32 B | **12.18** | 5.55 | 12.13 | 7.05 | 5.19 | 6.78 | 5.81 | 1.99 | 4.05 | 1.73× |
| 64 B | **16.52** | 7.15 | 14.18 | 9.02 | 7.69 | 9.75 | 8.73 | 1.53 | 6.54 | 1.83× |
| 128 B | **20.82** | 8.28 | 16.20 | 10.25 | 11.18 | 12.92 | 11.90 | 1.25 | 7.95 | 1.75× |
| 256 B | **29.56** | 7.03 | 17.01 | 10.94 | 13.60 | 17.42 | 16.39 | 1.16 | 12.32 | 1.80× |
| 512 B | **45.36** | 9.71 | 17.88 | 10.93 | 16.04 | 20.68 | 19.97 | 1.10 | 15.72 | 2.27× |
| 1024 B | **50.85** | 10.65 | 17.87 | 10.68 | 17.22 | 22.58 | 22.14 | 1.08 | 19.59 | 2.30× |
| 2048 B | **58.99** | 11.89 | 17.87 | 10.56 | 18.04 | 23.62 | 23.36 | 1.06 | 21.58 | 2.52× |
| 4096 B | **61.29** | 12.64 | 17.87 | 10.50 | 18.41 | 24.20 | 24.06 | 1.06 | 23.29 | 2.55× |
| 8192 B | **63.15** | 13.03 | 17.87 | 10.41 | 18.63 | 24.49 | 24.42 | 1.06 | 24.11 | 2.59× |
| 16384 B | **68.41** | 13.25 | 17.82 | 10.41 | 18.76 | 24.64 | 24.60 | 1.05 | 24.44 | 2.78× |
| 32768 B | **61.20** | 13.33 | 17.85 | 10.43 | 18.85 | 24.62 | 24.62 | 1.05 | 24.71 | 2.48× |
<!-- /bench:aarch64 -->

![Throughput on aarch64](https://raw.githubusercontent.com/ogxd/gxhash/main/benches/throughput/aarch64.svg)

### x86_64

<!-- bench:x86_64 -->
AMD EPYC 9V74 80-Core Processor, rustc 1.98.1, 2026-09-28

| Key type (ns per lookup) | GxHash | std (SipHash-1-3) | FoldHash (fast) ❌ | FoldHash (quality) | FxHasher (rustc_hash) ❌ | AHash | XxHash (XXH3) | T1ha | FNV-1a ❌ | HighwayHash | MetroHash |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| u32 | 1.90 | 9.90 | 1.63 | 1.64 | **1.49** | 1.91 | 27.17 | 15.16 | 2.06 | 37.08 | 6.77 |
| u64 | 1.90 | 12.45 | 1.90 | 1.63 | **1.49** | 1.90 | 30.88 | 4.87 | 3.63 | 36.89 | 5.42 |
| u128 | 1.90 | 13.06 | 1.90 | 1.90 | **1.63** | 1.90 | 27.50 | 5.42 | 9.14 | 41.22 | 18.77 |
| String, 6 bytes | 4.02 | 12.21 | 3.52 | 3.52 | **3.25** | 4.06 | 41.68 | 18.05 | 4.06 | 39.85 | 8.39 |
| String, 30 bytes | 4.22 | 17.04 | 4.60 | 4.88 | **4.06** | **4.06** | 40.88 | 17.73 | 18.41 | 38.23 | 10.60 |
| String, 128 bytes | **6.59** | 37.68 | 9.15 | 9.76 | 7.36 | 8.39 | 39.28 | 17.73 | 124.33 | 42.22 | 12.91 |
| String, 1024 bytes | **14.80** | 222.81 | 42.96 | 43.19 | 46.88 | 31.33 | 103.35 | 23.74 | 1100.42 | 105.96 | 44.94 |

| Throughput (GiB/s) | GxHash | XxHash (XXH3) | FxHasher (rustc_hash) ❌ | AHash | T1ha0 | FoldHash (fast) ❌ | FoldHash (quality) | FNV-1a ❌ | MetroHash | GxHash speedup |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 4 B | **3.43** | 1.15 | 2.75 | 2.29 | 0.98 | 1.72 | 1.53 | 1.96 | 0.90 | 1.50× |
| 8 B | **6.86** | 2.29 | 5.50 | 4.58 | 1.96 | 3.44 | 3.35 | 2.75 | 1.55 | 1.50× |
| 16 B | **13.73** | 4.52 | 10.99 | 9.17 | 3.33 | 6.88 | 6.68 | 2.29 | 1.69 | 1.50× |
| 32 B | 12.77 | 7.86 | 12.22 | **17.42** | 5.68 | 8.46 | 7.86 | 1.69 | 4.28 | 0.73× |
| 64 B | 17.65 | 11.58 | 16.93 | **23.77** | 12.21 | 12.82 | 11.50 | 1.21 | 7.94 | 0.74× |
| 128 B | **22.16** | 14.90 | 20.10 | 21.35 | 21.25 | 15.63 | 14.67 | 1.00 | 14.05 | 1.04× |
| 256 B | **47.17** | 7.15 | 22.90 | 28.56 | 36.45 | 20.65 | 19.81 | 0.92 | 21.77 | 1.29× |
| 512 B | **71.99** | 10.03 | 22.85 | 31.24 | 53.01 | 23.05 | 22.41 | 0.89 | 24.93 | 1.36× |
| 1024 B | **78.06** | 12.16 | 20.56 | 33.71 | 63.50 | 24.05 | 23.79 | 0.87 | 24.23 | 1.23× |
| 2048 B | **79.08** | 13.98 | 18.92 | 32.74 | 67.90 | 24.67 | 24.53 | 0.86 | 23.30 | 1.16× |
| 4096 B | **127.08** | 15.06 | 18.54 | 31.75 | 73.15 | 25.01 | 24.91 | 0.86 | 22.65 | 1.74× |
| 8192 B | **117.89** | 15.69 | 17.99 | 31.34 | 77.18 | 25.14 | 25.12 | 0.86 | 22.45 | 1.53× |
| 16384 B | **113.48** | 16.04 | 18.08 | 31.02 | 79.49 | 25.06 | 25.05 | 0.85 | 22.31 | 1.43× |
| 32768 B | **111.38** | 16.27 | 17.79 | 30.86 | 78.86 | 25.11 | 25.14 | 0.85 | 22.19 | 1.41× |
<!-- /bench:x86_64 -->

![Throughput on x86_64](https://raw.githubusercontent.com/ogxd/gxhash/main/benches/throughput/x86_64.svg)

### Running the Benchmarks

```bash
# HashSet lookups, printed as a markdown table
cargo bench --bench hashset

# Throughput. Add --features bench-md for a markdown table, bench-csv for CSV, or bench-plot for an .svg plot
cargo bench --bench throughput
```

The `throughput` benchmark does not rely on criterion.rs. To reduce bias, it shuffles seeds, input data and alignment, and it is less of a "black box" than criterion. A criterion-based version, `throughput_criterion`, is also available. Results vary slightly between the two: don't hesitate to submit an issue if you suspect bias.

## Safety and Soundness

GxHash uses `unsafe` code for SIMD and AES intrinsics, and for one optimization: to hash an input of less than 16 bytes, or the last bytes of an input, it loads a full 16-byte vector, which can extend past the end of the input, then masks out the bytes that are not part of the input. This avoids copying the input to a buffer, which matters for small keys. Here is why this is sound:
- Memory is protected per page (4 KiB or more). GxHash only loads past the end of the input when the 16 bytes are in the same page as the start of the input, so the load can never touch an unmapped page or fault. Otherwise, it loads the 16 bytes that end at the end of the input, which are in the pages of the input.
- The load is written in inline assembly (`asm!`), so it is not a Rust memory access: the compiler treats it as opaque, and can't optimize it on the assumption that it is in bounds.
- The bytes outside of the input are masked out before being hashed, so the hash never depends on them. The `does_not_hash_outside_of_bounds` test checks this.
- This is the same technique as optimized C library routines, such as glibc's `strlen` and `memchr`, which load whole vectors that can extend past the end of the data, relying on the same page property.

What is checked:
- The tests run under [Miri](https://github.com/rust-lang/miri) in CI, on the x86_64 hardware backend (with 128-bit AES-NI, and with 256-bit VAES) and on the software backend. Miri detects undefined behavior, such as out-of-bounds or misaligned accesses, in the code the tests run, which covers all input size classes. The `reads_stay_in_bounds` test hashes inputs that are allocations of their exact size, so that Miri reports any read outside of them. The exhaustive tests are skipped, as they are too slow for Miri.
- Miri can't run inline assembly, so for the two loads above, it runs an equivalent that copies the input bytes instead. Miri doesn't support ARM AES intrinsics, so the aarch64 hardware backend is not checked by Miri.
- The tests run on x86_64 and aarch64, with AES instructions enabled at compile time and detected at runtime, and compare the hardware and software backends on inputs of all lengths up to 4400 bytes.

More details on this optimization in [this article](https://ogxd.github.io/articles/unsafe-read-beyond-of-death/).

## Portability and Hardware Acceleration

GxHash builds and runs on every Rust target, with no configuration. It uses AES instructions (`AES-NI` on x86, `AES` on ARM) when the CPU has them. How depends on how your code is compiled and on the CPU it runs on, with three tiers that all produce the same hashes:

1. **Hardware, inlined**: the required target features are enabled at compile time (`aes` and `sse2` on x86, `aes` and `neon` on aarch64). This is the fastest tier, as GxHash is inlined in your code. It is the default on Apple ARM targets (macOS, iOS, ...). On other targets, including x86 PCs (AES-NI is not part of any x86-64 microarchitecture level, so no x86 target enables it by default), build with:
   ```bash
   # When the binary runs on the machine that builds it
   RUSTFLAGS="-C target-cpu=native" cargo build --release
   # Or, for any processor with AES instructions (the binary won't run on processors without them)
   RUSTFLAGS="-C target-feature=+aes" cargo build --release
   ```
   To set this once for a project, add it to `.cargo/config.toml`:
   ```toml
   [build]
   rustflags = ["-C", "target-feature=+aes"]
   ```
2. **Hardware, detected at runtime**: the default on x86 targets and on most other ARM targets. The target features are not enabled at compile time, but GxHash detects that the processor supports them. It then uses the same instructions, through a function call rather than inlined, which costs a few tenths of a nanosecond per hash. This mostly matters for small inputs.
3. **Software**: the processor has no AES instructions (some old or low-end processors, some virtual machines), or GxHash has no hardware implementation for the architecture (32-bit ARM, RISC-V, WebAssembly, ...). GxHash still works, but is roughly 10 to 40 times slower.

Indicative timings, measured on an Apple M5 Pro:

| | Tier 1 | Tier 2 | Tier 3 |
|---|---|---|---|
| `gxhash64`, 16 bytes | 0.8 ns | 1.1 ns | 12 ns |
| `gxhash64`, 1 KiB | 6.4 ns | 6.8 ns | 202 ns |
| `HashMap<u64>` lookup | 1.1 ns | 1.4 ns | 14 ns |
| `HashMap<&str>` lookup | 3.9 ns | 4.1 ns | 43 ns |

On x86 processors with `VAES` and `AVX2`, tiers 1 and 2 also use 256-bit AES instructions for inputs larger than 2 KiB, for higher throughput. These are detected at runtime, unless enabled at compile time (`-C target-cpu=native` or `-C target-feature=+aes,+vaes,+avx2`).

Without the `std` feature, runtime detection is limited:
- On x86, only AES instructions are detected, not `VAES`.
- On aarch64, nothing is detected: the crate fails to build unless the `aes` and `neon` target features are enabled at compile time.

## Hashes Stability
All generated hashes for a given major version of GxHash are stable, meaning that for a given input the output hash will be the same across all platforms and [tiers](#portability-and-hardware-acceleration). This also means that the hash may change between majors versions (eg gxhash 3.x and 4.x).

### Consistency of Hashes When Using the `Hasher` Trait
The `Hasher` trait defines methods to hash specific types. This allows the implementation to circumvent some tricks used when the size is unknown. For this reason, hashing 4 `u32` using a `Hasher` will return a different hash compared to using the `gxhash128` method directly with these same 4 `u32` but represented as 16 `u8`. The rationale being that `Hasher` (mostly used for things like `HashMap` or `HashSet`) and `gxhash128` are used in two different scenarios. Both way are independently stable still.

## Security
GxHash is seeded (with seed randomization) to improve DOS resistance and uses a wide (128-bit) internal state to improve multicollision resistance. Yet, such resistances are just basic safeguards and do not make GxHash secure against all attacks.

For use cases that require deterministic repeatability, you can disable random seeding with the feature "deterministic," but this of course disables DOS mitigation.

The software implementation ([tier 3](#portability-and-hardware-acceleration)) uses lookup tables, whose access times depend on the input and the seed. An attacker able to measure them could learn about the seed, which weakens DOS mitigation.

Also, it is important to note that GxHash is not a cryptographic hash function and should not be used for cryptographic purposes.

## Flags

### `no_std`

The `std` feature flag enables the `Hasher` implementation, the `HashMap`/`HashSet` container convenience type aliases, and full runtime detection of hardware features (see [Portability and Hardware Acceleration](#portability-and-hardware-acceleration)). This is on by default. Disable to make the crate `no_std`:

```toml
[dependencies.gxhash]
...
default-features = false
```

### `deterministic`

Disables random seeding of `GxBuildHasher`, for reproducible hashes across runs. See [Security](#security).

## Contributing

- Feel free to submit PRs
- Repository is entirely usable via `cargo` commands
- Versioning is the following
  - Major for stability breaking changes (output hashes for a same input are different after changes)
  - Minor for API changes/removal
  - Patch for new APIs, bug fixes and performance improvements

#### Useful profiling tools
- [cargo-show-asm](https://github.com/pacak/cargo-show-asm) is an easy way to view the actual generated assembly code. You can use the hello_world example to view the isolated, unoptimized byte code for gxhash. A few useful commands:
  - Line by line generated asm: `cargo asm --rust --simplify --example hello_world hello_world::gxhash`
  - Generated llvm: `cargo asm --llvm --example hello_world hello_world::gxhash`
  - Count of assembly instructions: `cargo asm --simplify --example hello_world hello_world::gxhash | grep -v '^\.' | wc -l`
    - Powershell version: `cargo asm --simplify --example hello_world hello_world::gxhash | where { !$_.StartsWith(".") } | measure -Line`
- [AMD μProf](https://www.amd.com/en/developer/uprof.html) gives some useful insights on time spent per instruction.

## Publication
> Author note:
> I'm committed to the open dissemination of scientific knowledge. In an era where access to information is more democratized than ever, I believe that science should be freely available to all – both for consumption and contribution. Traditional scientific journals often involve significant financial costs, which can introduce biases and can shift the focus from purely scientific endeavors to what is currently trendy.
>
> To counter this trend and to uphold the true spirit of research, I have chosen to share my work on "gxhash" directly on GitHub, ensuring that it's openly accessible to anyone interested. Additionally, the use of a free Zenodo DOI ensures that this research is citable and can be referenced in other works, just as traditional publications are.
>
> I strongly believe in a world where science is not behind paywalls, and I am in for a more inclusive, unbiased, and open scientific community.

The paper describes the design of GxHash and its quality and performance measurements:
[PDF](https://github.com/ogxd/gxhash/blob/main/article/article.pdf)

Cite this publication / algorithm:
[![DOI](https://zenodo.org/badge/690754256.svg)](https://zenodo.org/badge/latestdoi/690754256)
