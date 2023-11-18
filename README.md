# GxHash
![CI](https://github.com/ogxd/gxhash-rust/actions/workflows/rust.yml/badge.svg)

Up to this date, the fastest non-cryptographic hashing algorithm 🚀 (see benchmarks)  
Passes all [SMHasher](https://github.com/rurban/smhasher) quality tests ✅

#### What makes it so fast?
Here are the principal reasons:
- SIMD all the way (and usage of SIMD AES for efficient bit mixing)
- High ILP processing for large inputs
- Small bytecode for greater inlining opportunities
Checkout the [article](https://github.com/ogxd/gxhash-rust/blob/main/article/article.pdf) for more details.

## Usage
```
cargo add gxhash
```

```rust
use gxhash::*;

// Used as a hashing function
let bytes = [42u8; 1000];
let seed = 1234;
println!("Hash is {:x}!", gxhash::gxhash64(&bytes, seed));

// Used as an Hasher for faster HashSet/HashMap
let mut hashset = GxHashSet::default();
hashset.insert("hello world");
```

## Compatibility
- ARM 64-bit using `NEON` intrinsics.
- x86-64 bit using `SSE2` + `AES` intrinsics.
- (optional) with `avx2` feature enabled, gxhash will use `AVX2` intrinsics, for up to twice as much performance for large inputs. Only compatible on `AVX2` enabled x86-64 platforms.

> **Warning**
> Other platforms are currently not supported (there is no fallback)

## Security
### DOS Resistance
GxHash is a seeded hashing algorithm, meaning that depending on the seed used, it will generate completely different hashes. The default `HasherBuilder` (`GxHasherBuilder::default()`) uses seed randomization, making any `HashMap`/`HashSet` more DOS resistant, as it will make it much more difficult for attackers to be able to predict which hashes may collide without knowing the seed used. This does not mean however that it is completely DOS resistant. This has to be analyzed further.
### Multicollisions Resistance
GxHash uses a 128-bit internal state (and even 256-bit with the `avx2` feature). This makes GxHash [a widepipe construction](https://en.wikipedia.org/wiki/Merkle%E2%80%93Damg%C3%A5rd_construction#Wide_pipe_construction) when generating hashes of size 64-bit or smaller, which had amongst other properties to be inherently more resistant to multicollision attacks. See [this paper](https://www.iacr.org/archive/crypto2004/31520306/multicollisions.pdf) for more details.
### Cryptographic Properties ❌
GxHash is a non-cryptographic hashing algorithm, thus it is not recommended to use it as a cryptographic algorithm (it is not a replacement for SHA). It has not been assessed if GxHash is preimage resistant and how likely it is to be reversed.

## Benchmarks
Displayed numbers are throughput in Mibibytes of data hashed per second. Higher is better.  
To run the benchmarks: `cargo bench --bench throughput`.

### Intel Ice Lake (x86 64-bit) (GCP n2-standard-2)

| Method      |    4 |    16 |    64 |   256 |  1024 |   4096 |  16384 |
|-------------|-----:|------:|------:|------:|------:|-------:|-------:|
| gxhash-avx2 | 4189 | 16734 | 46142 | 72679 | 96109 | 102202 | 100845 |
| gxhash      | 6069 | 24283 | 29465 | 49542 | 58164 |  62511 |  64281 |
| xxhash      |  915 |  4266 | 10339 | 10116 | 17164 |  20135 |  22834 |
| ahash       | 1838 |  8712 | 22473 | 25958 | 35090 |  38440 |  39308 |
| t1ha0       |  740 |  2707 |  8572 | 28659 | 51202 |  59918 |  65902 |
| seahash     |  213 |   620 |  1762 |  2473 |  2761 |   2837 |   2860 |
| metrohash   |  754 |  2556 |  5983 | 10395 | 12738 |  13492 |  13624 |
| highwayhash |  122 |   490 |  3278 |  7057 |  9726 |  10743 |  11036 |
| fnv-1a      | 1169 |  3062 |  1602 |   933 |   833 |    811 |    808 |

### Macbook M1 Pro (ARM 64-bit)

| Method      |    4 |    16 |    64 |   256 |  1024 |  4096 |  16384 |
|-------------|-----:|------:|------:|------:|------:|------:|-------:|
| gxhash      | 6192 | 24901 | 31770 | 59465 | 72476 | 74723 |  76746 |
| xxhash      | 1407 |  5638 | 11432 |  8380 | 16289 | 18690 |  19310 |
| ahash       | 1471 |  5920 | 15597 | 22280 | 28672 | 29631 |  31174 |
| t1ha0       | 1181 |  4254 | 10277 | 15459 | 14120 | 13741 |  13743 |
| seahash     | 1130 |  4428 |  8756 |  9248 |  8357 |  8085 |   8056 |
| metrohash   | 1094 |  3389 |  9709 | 14431 | 17470 | 17679 |  17931 |
| highwayhash |  182 |   743 |  2696 |  5196 |  6573 |  7061 |   7170 |
| fnv-1a      | 1988 |  2627 |  1407 |   896 |   777 |   753 |    745 |

## Debugging
The algorithm is mostly inlined, making most profilers fail at providing useful intrinsics. The best I could achieve is profiling at assembly level. [cargo-asm](https://github.com/gnzlbg/cargo-asm) is an easy way to view the actual generated assembly code (`cargo asm gxhash::gxhash::gxhash`). [AMD μProf](https://www.amd.com/en/developer/uprof.html) gives some useful insights on time spent per instruction.

## Publication
> Author note:
> I'm committed to the open dissemination of scientific knowledge. In an era where access to information is more democratized than ever, I believe that science should be freely available to all – both for consumption and contribution. Traditional scientific journals often involve significant financial costs, which can introduce biases and can shift the focus from purely scientific endeavors to what is currently trendy. 
>
> To counter this trend and to uphold the true spirit of research, I have chosen to share my work on "gxhash" directly on GitHub, ensuring that it's openly accessible to anyone interested. Additionally, the use of a free Zenodo DOI ensures that this research is citable and can be referenced in other works, just as traditional publications are. 
>
> I strongly believe in a world where science is not behind paywalls, and I am in for a more inclusive, unbiased, and open scientific community.

Publication:  
[PDF](https://github.com/ogxd/gxhash-rust/blob/main/article/article.pdf)

Cite this publication / algorithm:  
[![DOI](https://zenodo.org/badge/690754256.svg)](https://zenodo.org/badge/latestdoi/690754256)
