# gxhash-rust
[![DOI](https://zenodo.org/badge/690754256.svg)](https://zenodo.org/badge/latestdoi/690754256)

The fastest non-cryptographic hashing algorithm

WORK IN PROGRESS

## Benchmarks
$env:RUSTFLAGS = "-C target-cpu=native"
RUSTFLAGS="-C target-cpu=native"
cargo bench --bench throughput

### Benchmark Results GCP (Intel Ice Lake)

| Month          |      4 |     16 |     64 |    256 |   1024 |   4096 |  16384 |
| -------------- | ------ | ------ | ------ | ------ | ------ | ------ | ------ |
| gxhash AVX2    | 11.638 | 46.292 | 31.353 | 81.483 | 103.91 | 101.29 | 99.666 |
| gxhash AVX-512 | 10.927 | 43.136 | 27.944 | 68.942 | 136.63 | 112.16 | 105.32 |
| t1ha0 AVX2     |  0.921 | 3.5989 | 11.858 | 32.581 | 52.552 | 60.746 | 65.863 |