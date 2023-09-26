# gxhash-rust
[![DOI](https://zenodo.org/badge/690754256.svg)](https://zenodo.org/badge/latestdoi/690754256)

The fastest non-cryptographic hashing algorithm

WORK IN PROGRESS

## Benchmarks
$env:RUSTFLAGS = "-C target-cpu=native"
RUSTFLAGS="-C target-cpu=native"
cargo bench --bench throughput

### Benchmark Results GCP

#### AVX2

gxhash/4 bytes (aligned)
                        time:   [40.550 ns 40.561 ns 40.573 ns]
                        thrpt:  [94.020 MiB/s 94.048 MiB/s 94.075 MiB/s]
Found 129 outliers among 1000 measurements (12.90%)
  3 (0.30%) low severe
  24 (2.40%) low mild
  33 (3.30%) high mild
  69 (6.90%) high severe
gxhash/16 bytes (aligned)
                        time:   [40.660 ns 40.688 ns 40.719 ns]
                        thrpt:  [374.73 MiB/s 375.02 MiB/s 375.28 MiB/s]
Found 122 outliers among 1000 measurements (12.20%)
  50 (5.00%) high mild
  72 (7.20%) high severe
gxhash/64 bytes (aligned)
                        time:   [57.236 ns 57.316 ns 57.424 ns]
                        thrpt:  [1.0380 GiB/s 1.0399 GiB/s 1.0414 GiB/s]
Found 129 outliers among 1000 measurements (12.90%)
  21 (2.10%) low mild
  28 (2.80%) high mild
  80 (8.00%) high severe
gxhash/256 bytes (aligned)
                        time:   [132.87 ns 132.90 ns 132.93 ns]
                        thrpt:  [1.7936 GiB/s 1.7940 GiB/s 1.7944 GiB/s]
Found 141 outliers among 1000 measurements (14.10%)
  15 (1.50%) low severe
  10 (1.00%) low mild
  39 (3.90%) high mild
  77 (7.70%) high severe
gxhash/1024 bytes (aligned)
                        time:   [565.57 ns 565.68 ns 565.81 ns]
                        thrpt:  [1.6855 GiB/s 1.6859 GiB/s 1.6862 GiB/s]
Found 138 outliers among 1000 measurements (13.80%)
  1 (0.10%) low severe
  22 (2.20%) low mild
  39 (3.90%) high mild
  76 (7.60%) high severe
gxhash/4096 bytes (aligned)
                        time:   [2.2978 µs 2.2985 µs 2.2992 µs]
                        thrpt:  [1.6592 GiB/s 1.6597 GiB/s 1.6601 GiB/s]
Found 138 outliers among 1000 measurements (13.80%)
  20 (2.00%) low mild
  24 (2.40%) high mild
  94 (9.40%) high severe
gxhash/16384 bytes (aligned)
                        time:   [9.2233 µs 9.2276 µs 9.2342 µs]
                        thrpt:  [1.6524 GiB/s 1.6536 GiB/s 1.6544 GiB/s]
Found 152 outliers among 1000 measurements (15.20%)
  2 (0.20%) low severe
  13 (1.30%) low mild
  36 (3.60%) high mild
  101 (10.10%) high severe