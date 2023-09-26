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
                        time:   [319.68 ps 320.09 ps 320.51 ps]
                        thrpt:  [11.623 GiB/s 11.638 GiB/s 11.653 GiB/s]
                 change:
                        time:   [-99.203% -99.202% -99.200%] (p = 0.00 < 0.05)
                        thrpt:  [+12399% +12424% +12446%]
                        Performance has improved.
Found 74 outliers among 1000 measurements (7.40%)
  32 (3.20%) low mild
  27 (2.70%) high mild
  15 (1.50%) high severe
gxhash/16 bytes (aligned)
                        time:   [321.37 ps 321.90 ps 322.45 ps]
                        thrpt:  [46.213 GiB/s 46.292 GiB/s 46.367 GiB/s]
                 change:
                        time:   [-99.202% -99.200% -99.198%] (p = 0.00 < 0.05)
                        thrpt:  [+12377% +12402% +12427%]
                        Performance has improved.
Found 61 outliers among 1000 measurements (6.10%)
  20 (2.00%) low mild
  28 (2.80%) high mild
  13 (1.30%) high severe
gxhash/64 bytes (aligned)
                        time:   [1.8990 ns 1.9011 ns 1.9040 ns]
                        thrpt:  [31.305 GiB/s 31.353 GiB/s 31.387 GiB/s]
                 change:
                        time:   [-96.656% -96.651% -96.644%] (p = 0.00 < 0.05)
                        thrpt:  [+2880.0% +2885.5% +2890.7%]
                        Performance has improved.
Found 100 outliers among 1000 measurements (10.00%)
  6 (0.60%) low mild
  31 (3.10%) high mild
  63 (6.30%) high severe
gxhash/256 bytes (aligned)
                        time:   [2.9239 ns 2.9260 ns 2.9284 ns]
                        thrpt:  [81.416 GiB/s 81.483 GiB/s 81.541 GiB/s]
                 change:
                        time:   [-97.802% -97.799% -97.796%] (p = 0.00 < 0.05)
                        thrpt:  [+4438.2% +4443.8% +4449.8%]
                        Performance has improved.
Found 104 outliers among 1000 measurements (10.40%)
  10 (1.00%) low mild
  26 (2.60%) high mild
  68 (6.80%) high severe
gxhash/1024 bytes (aligned)
                        time:   [9.1729 ns 9.1782 ns 9.1843 ns]
                        thrpt:  [103.84 GiB/s 103.91 GiB/s 103.97 GiB/s]
                 change:
                        time:   [-98.358% -98.356% -98.354%] (p = 0.00 < 0.05)
                        thrpt:  [+5975.7% +5983.8% +5991.7%]
                        Performance has improved.
Found 116 outliers among 1000 measurements (11.60%)
  11 (1.10%) low mild
  23 (2.30%) high mild
  82 (8.20%) high severe
gxhash/4096 bytes (aligned)
                        time:   [37.621 ns 37.662 ns 37.708 ns]
                        thrpt:  [101.16 GiB/s 101.29 GiB/s 101.40 GiB/s]
                 change:
                        time:   [-98.338% -98.336% -98.333%] (p = 0.00 < 0.05)
                        thrpt:  [+5897.2% +5908.2% +5917.4%]
                        Performance has improved.
Found 131 outliers among 1000 measurements (13.10%)
  27 (2.70%) high mild
  104 (10.40%) high severe
gxhash/16384 bytes (aligned)
                        time:   [152.17 ns 153.10 ns 154.18 ns]
                        thrpt:  [98.970 GiB/s 99.666 GiB/s 100.28 GiB/s]
                 change:
                        time:   [-98.332% -98.328% -98.322%] (p = 0.00 < 0.05)
                        thrpt:  [+5861.1% +5879.2% +5894.1%]
                        Performance has improved.
Found 128 outliers among 1000 measurements (12.80%)
  8 (0.80%) low mild
  21 (2.10%) high mild
  99 (9.90%) high severe

----

t1ha0/t1ha0_ia32aes_avx2/4                                                                             
                        time:   [4.1367 ns 4.1389 ns 4.1412 ns]
                        thrpt:  [921.15 MiB/s 921.68 MiB/s 922.17 MiB/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
t1ha0/t1ha0_ia32aes_avx2/16                                                                             
                        time:   [4.1386 ns 4.1405 ns 4.1428 ns]
                        thrpt:  [3.5969 GiB/s 3.5989 GiB/s 3.6006 GiB/s]
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) high mild
  4 (4.00%) high severe
t1ha0/t1ha0_ia32aes_avx2/64                                                                             
                        time:   [5.0245 ns 5.0266 ns 5.0289 ns]
                        thrpt:  [11.852 GiB/s 11.858 GiB/s 11.863 GiB/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
t1ha0/t1ha0_ia32aes_avx2/256                                                                             
                        time:   [7.3143 ns 7.3177 ns 7.3215 ns]
                        thrpt:  [32.564 GiB/s 32.581 GiB/s 32.596 GiB/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
t1ha0/t1ha0_ia32aes_avx2/1024                                                                             
                        time:   [18.137 ns 18.147 ns 18.158 ns]
                        thrpt:  [52.520 GiB/s 52.552 GiB/s 52.581 GiB/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
t1ha0/t1ha0_ia32aes_avx2/4096                                                                            
                        time:   [62.738 ns 62.797 ns 62.850 ns]
                        thrpt:  [60.695 GiB/s 60.746 GiB/s 60.804 GiB/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  5 (5.00%) high severe
t1ha0/t1ha0_ia32aes_avx2/16384                                                                            
                        time:   [231.23 ns 231.67 ns 232.55 ns]
                        thrpt:  [65.614 GiB/s 65.863 GiB/s 65.990 GiB/s]

#### AVX-512

gxhash/4 bytes (aligned)
                        time:   [339.88 ps 340.92 ps 341.95 ps]
                        thrpt:  [10.894 GiB/s 10.927 GiB/s 10.961 GiB/s]
                 change:
                        time:   [-0.2666% +0.1830% +0.6240%] (p = 0.44 > 0.05)
                        thrpt:  [-0.6202% -0.1826% +0.2673%]
                        No change in performance detected.
Found 35 outliers among 1000 measurements (3.50%)
  1 (0.10%) low mild
  11 (1.10%) high mild
  23 (2.30%) high severe
gxhash/16 bytes (aligned)
                        time:   [344.24 ps 345.45 ps 346.67 ps]
                        thrpt:  [42.984 GiB/s 43.136 GiB/s 43.288 GiB/s]
                 change:
                        time:   [+0.6706% +1.1759% +1.7034%] (p = 0.00 < 0.05)
                        thrpt:  [-1.6749% -1.1622% -0.6661%]
                        Change within noise threshold.
Found 31 outliers among 1000 measurements (3.10%)
  28 (2.80%) high mild
  3 (0.30%) high severe
gxhash/64 bytes (aligned)
                        time:   [2.1319 ns 2.1330 ns 2.1342 ns]
                        thrpt:  [27.929 GiB/s 27.944 GiB/s 27.959 GiB/s]
                 change:
                        time:   [-0.0697% +0.0331% +0.1467%] (p = 0.55 > 0.05)
                        thrpt:  [-0.1465% -0.0331% +0.0698%]
                        No change in performance detected.
Found 119 outliers among 1000 measurements (11.90%)
  7 (0.70%) low severe
  3 (0.30%) low mild
  26 (2.60%) high mild
  83 (8.30%) high severe
gxhash/256 bytes (aligned)
                        time:   [3.4561 ns 3.4583 ns 3.4612 ns]
                        thrpt:  [68.884 GiB/s 68.942 GiB/s 68.984 GiB/s]
                 change:
                        time:   [-0.6290% -0.3926% -0.1760%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1763% +0.3941% +0.6330%]
                        Change within noise threshold.
Found 118 outliers among 1000 measurements (11.80%)
  18 (1.80%) low mild
  22 (2.20%) high mild
  78 (7.80%) high severe
gxhash/1024 bytes (aligned)
                        time:   [6.9769 ns 6.9797 ns 6.9830 ns]
                        thrpt:  [136.57 GiB/s 136.63 GiB/s 136.69 GiB/s]
                 change:
                        time:   [-13.149% -13.061% -12.966%] (p = 0.00 < 0.05)
                        thrpt:  [+14.897% +15.023% +15.140%]
                        Performance has improved.
Found 122 outliers among 1000 measurements (12.20%)
  18 (1.80%) low mild
  35 (3.50%) high mild
  69 (6.90%) high severe
gxhash/4096 bytes (aligned)
                        time:   [33.984 ns 34.011 ns 34.044 ns]
                        thrpt:  [112.05 GiB/s 112.16 GiB/s 112.25 GiB/s]
                 change:
                        time:   [-1.6807% -1.5685% -1.4484%] (p = 0.00 < 0.05)
                        thrpt:  [+1.4697% +1.5935% +1.7094%]
                        Performance has improved.
Found 90 outliers among 1000 measurements (9.00%)
  43 (4.30%) high mild
  47 (4.70%) high severe
gxhash/16384 bytes (aligned)
                        time:   [144.79 ns 144.88 ns 145.01 ns]
                        thrpt:  [105.23 GiB/s 105.32 GiB/s 105.39 GiB/s]
                 change:
                        time:   [+0.3307% +0.4535% +0.5658%] (p = 0.00 < 0.05)
                        thrpt:  [-0.5626% -0.4515% -0.3296%]
                        Change within noise threshold.
Found 123 outliers among 1000 measurements (12.30%)
  17 (1.70%) low mild
  31 (3.10%) high mild
  75 (7.50%) high severe