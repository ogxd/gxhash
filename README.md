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

> **Warning**
> This is a non-cryptographic hashing algorithm, thus it is not recommended to use it as a cryptographic algorithm (it is not a replacement for SHA).

## Compatibility
- ARM 64-bit using `NEON` intrinsics.
- x86-64 bit using `SSE2` + `AES` intrinsics.
- (optional) with `avx2` feature enabled, gxhash will use `AVX2` intrinsics, for up to twice as much performance for large inputs. Only compatible on `AVX2` enabled x86-64 platforms.

> **Warning**
> Other platforms are currently not supported (there is no fallback)

## Benchmarks
Displayed numbers are throughput in Mibibytes of data hashed per second. Higher is better.  
To run the benchmarks: `cargo bench --bench throughput`.

### Intel Ice Lake (x86 64-bit) (GCP n2-standard-2)

| Method      |       4 |       16 |       64 |      256 |     1024 |     4096 |    16384 |
|-------------|--------:|---------:|---------:|---------:|---------:|---------:|---------:|
| gxhash-avx2 | 4021.94 | 16113.58 | 42936.69 |  72145.2 | 94127.12 | 98261.24 | 100333.4 |
| gxhash      | 6122.63 | 24476.94 |  25591.9 | 51949.28 | 61253.58 | 64774.75 | 65708.38 |
| xxhash      |  915.69 |  4266.94 | 10339.13 | 10116.71 | 17164.93 | 20135.65 | 22834.07 |
| ahash       | 1838.59 |  8712.95 | 22473.84 | 25958.66 | 35090.25 | 38440.04 |  39308.7 |
| t1ha0       |  740.15 |  2707.93 |  8572.39 | 28659.06 | 51202.34 | 59918.76 | 65902.36 |
| seahash     |  213.04 |   620.54 |  1762.72 |  2473.87 |  2761.71 |  2837.24 |  2860.51 |
| metrohash   |  754.55 |  2556.83 |  5983.26 | 10395.86 | 12738.02 | 13492.63 | 13624.54 |
| highwayhash |  122.52 |   490.89 |  3278.71 |  7057.25 |  9726.72 | 10743.01 | 11036.79 |
| fnv-1a      | 1169.76 |  3062.36 |  1602.71 |   933.96 |   833.82 |   811.77 |   808.07 |

### Macbook M1 Pro (ARM 64-bit)

| Method             |       4 |       16 |       64 |      256 |     1024 |     4096 |    16384 |
|--------------------|--------:|---------:|---------:|---------:|---------:|---------:|---------:|
| gxhash             | 5441.06 | 21635.99 | 26282.95 | 59859.19 | 70175.71 | 74723.96 | 75020.74 |
| xxhash             | 1407.55 |  5638.49 | 11432.47 |  8380.32 | 16289.65 | 18690.69 | 19310.57 |
| ahash              | 1471.71 |  5920.45 | 15597.47 |  22280.2 | 28672.62 |    29631 | 31174.07 |
| t1ha0              | 1181.94 |  4254.77 | 10277.71 | 15459.97 | 14120.73 | 13741.89 |  13743.4 |
| seahash            |    1130 |   4428.8 |   8756.7 |   9248.1 |  8357.73 |  8085.24 |   8056.4 |
| metrohash          |  1094.4 |  3389.34 |  9709.14 | 14431.34 |    17470 | 17679.48 |  17931.1 |
| highwayhash        |  182.95 |   743.38 |  2696.71 |  5196.88 |  6573.42 |  7061.91 |  7170.97 |
| fnv-1a             | 1988.88 |  2627.51 |   1407.3 |   896.08 |   777.74 |   753.23 |   745.68 |

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
