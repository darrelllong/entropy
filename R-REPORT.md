# R-REPORT — RNG tests via R's standard randomness packages

Each generator below was sampled into a binary stream of little-endian u32
words. R then read the stream, normalised it to U[0,1), and ran every test
exposed by the standard R RNG-testing packages (`randtests`, `randtoolbox`,
`tseries`) plus the goodness-of-fit and autocorrelation tests in `stats`.

Sample size: **5 000 000 u32 words** for every generator except
`Dual_EC_DRBG`, which uses **1 000 000** because each block requires two
P-256 scalar multiplications (≈ 10 min/MB). `randtests::rank.test` is O(n²)
Mann-Kendall; it is run on the first 5 000 samples to keep the per-RNG
runtime under a second.

The reject threshold is α = 0.001 (a single test passing/failing is not
proof; a generator that gets a few REJECTs from independent tests at α=0.001
is expected statistical noise across ~16 tests, but a generator that REJECTs
on most tests is broken).

`tseries::jarque.bera.test` tests Normality and is **expected to REJECT**
for a uniform stream; it is included as a sanity check.

The moment table reports the empirical raw moments E[U^k] for k = 1..10 and
the absolute error against the theoretical value 1/(k+1) for U(0,1).

Generated with `scripts/run_r_report.sh`; binary: `target/release/dump_rng`;
analysis: `scripts/r_rng_tests.R`.

R packages used:
- `randtests` 1.0.2
- `randtoolbox` 2.0.5
- `tseries` 0.10.61
- `nortest` 1.0.4
- `moments` 0.14.1
- `stats` 4.5.0

R version: 4.5.0
Host: Darwin 25.4.0 arm64
Date: 2026-04-28 10:08:03 PDT

---

## Summary — REJECT counts at α = 0.001

Counts exclude `tseries::jarque.bera.test`, which is a Normality test and is **expected to REJECT** for any uniform stream.

| RNG | REJECTs | Passes | n/a |
|-----|---------|--------|-----|
| OsRng (/dev/urandom) | 0 | 15 | 0 |
| ConstantRng | 9 | 2 | 4 |
| CounterRng | 14 | 0 | 1 |
| System V rand() | 1 | 14 | 0 |
| rand48 (mrand48) | 0 | 15 | 0 |
| BSD random() / glibc random() | 0 | 15 | 0 |
| FreeBSD rand_r() compat | 0 | 15 | 0 |
| Windows MSVC rand() | 1 | 14 | 0 |
| Windows VB6/VBA Rnd() | 6 | 9 | 0 |
| Windows .NET Random | 0 | 15 | 0 |
| ANSI C LCG | 5 | 9 | 1 |
| MINSTD (Park-Miller) | 5 | 9 | 1 |
| Borland C++ LCG | 5 | 9 | 1 |
| MSVC LCG | 5 | 9 | 1 |
| MT19937 | 0 | 15 | 0 |
| Xorshift32 | 0 | 15 | 0 |
| Xorshift64 | 0 | 15 | 0 |
| PCG32 | 0 | 15 | 0 |
| PCG64 | 0 | 15 | 0 |
| Xoshiro256 | 0 | 15 | 0 |
| Xoroshiro128 | 0 | 15 | 0 |
| WyRand | 0 | 15 | 0 |
| SFC64 | 0 | 15 | 0 |
| JSF64 | 0 | 15 | 0 |
| AES-128-CTR | 0 | 15 | 0 |
| Camellia-128-CTR | 1 | 14 | 0 |
| Twofish-128-CTR | 0 | 15 | 0 |
| Serpent-128-CTR | 0 | 15 | 0 |
| SM4-CTR | 0 | 15 | 0 |
| Grasshopper-256-CTR | 0 | 15 | 0 |
| CAST-128-CTR | 0 | 15 | 0 |
| SEED-CTR | 0 | 15 | 0 |
| Rabbit | 0 | 15 | 0 |
| Salsa20 | 0 | 15 | 0 |
| Snow3G | 0 | 15 | 0 |
| ZUC-128 | 0 | 15 | 0 |
| ChaCha20 | 0 | 15 | 0 |
| SpongeBob (SHA3-512) | 1 | 14 | 0 |
| Squidward (SHA-256) | 1 | 14 | 0 |
| HmacDrbg | 0 | 15 | 0 |
| HashDrbg | 0 | 15 | 0 |
| CtrDrbgAes256 | 0 | 15 | 0 |
| Dual_EC_DRBG (P-256) | 0 | 15 | 0 |

---

## OsRng (/dev/urandom)

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499965  Var = 0.083295  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.344355 | 0.730580 | pass |
| randtests::bartels.rank.test | 0.082414 | 0.934317 | pass |
| randtests::cox.stuart.test (trend) | 1250244.000000 | 0.758078 | pass |
| randtests::difference.sign.test | -0.031758 | 0.974665 | pass |
| randtests::turning.point.test | -0.374413 | 0.708097 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.430513 | 0.666823 | pass |
| randtoolbox::freq.test (16 bins) | 16.933754 | 0.322840 | pass |
| randtoolbox::gap.test [0,0.5) | 15.912694 | 0.820195 | pass |
| randtoolbox::serial.test (d=8) | 53.127680 | 0.807824 | pass |
| randtoolbox::poker.test (5-hand) | 2.485808 | 0.647179 | pass |
| randtoolbox::order.test (d=4) | 26.362163 | 0.284069 | pass |
| stats::ks.test vs U(0,1) | 0.000283 | 0.818953 | pass |
| stats::chisq.test (256 bins) | 283.372646 | 0.107149 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 28.213071 | 0.298138 | pass |
| tseries::runs.test (binary) | 0.344355 | 0.730580 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299896.175707 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49996508 | 0.50000000 | 3.49e-05 |
| 2 | 0.33325979 | 0.33333333 | 7.35e-05 |
| 3 | 0.24990039 | 0.25000000 | 9.96e-05 |
| 4 | 0.19988349 | 0.20000000 | 1.17e-04 |
| 5 | 0.16653985 | 0.16666667 | 1.27e-04 |
| 6 | 0.14272455 | 0.14285714 | 1.33e-04 |
| 7 | 0.12486467 | 0.12500000 | 1.35e-04 |
| 8 | 0.11097504 | 0.11111111 | 1.36e-04 |
| 9 | 0.09986444 | 0.10000000 | 1.36e-04 |
| 10 | 0.09077484 | 0.09090909 | 1.34e-04 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 19.866133 |
| Bonferroni p (no spike) | 0.005874 |
| Spectral flatness (Wiener entropy) | 0.561461 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=1.798, p=0.994275 |
| Periodogram KS vs Exp(1) | D=0.000366, p=0.891224 |


## ConstantRng

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.869841  Var = 0.000000  Min = 0.869841  Max = 0.869841

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | NA | NA | n/a |
| randtests::bartels.rank.test | -2236.068291 | 0.000000 | REJECT |
| randtests::cox.stuart.test (trend) | 0.000000 | 2.000000 | pass |
| randtests::difference.sign.test | 0.000000 | 1.000000 | pass |
| randtests::turning.point.test | NA | NA | n/a |
| randtests::rank.test (Mann-Kendall, n=5000) | -106.028906 | 0.000000 | REJECT |
| randtoolbox::freq.test (16 bins) | 75000000.000000 | 0.000000 | REJECT |
| randtoolbox::gap.test [0,0.5) | 1249999.850988 | 0.000000 | REJECT |
| randtoolbox::serial.test (d=8) | 157500000.000000 | 0.000000 | REJECT |
| randtoolbox::poker.test (5-hand) | 624000000.000000 | 0.000000 | REJECT |
| randtoolbox::order.test (d=4) | 1250000.000000 | 0.000000 | REJECT |
| stats::ks.test vs U(0,1) | 0.869841 | 0.000000 | REJECT |
| stats::chisq.test (256 bins) | 1275000000.000000 | 0.000000 | REJECT |
| stats::Box.test (Ljung-Box, lag 25) | NA | NA | n/a |
| tseries::runs.test (binary) | NA | NA | n/a |
| tseries::jarque.bera.test (vs Normal*) | 1666666.666606 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.86984054 | 0.50000000 | 3.70e-01 |
| 2 | 0.75662257 | 0.33333333 | 4.23e-01 |
| 3 | 0.65814099 | 0.25000000 | 4.08e-01 |
| 4 | 0.57247771 | 0.20000000 | 3.72e-01 |
| 5 | 0.49796433 | 0.16666667 | 3.31e-01 |
| 6 | 0.43314956 | 0.14285714 | 2.90e-01 |
| 7 | 0.37677105 | 0.12500000 | 2.52e-01 |
| 8 | 0.32773073 | 0.11111111 | 2.17e-01 |
| 9 | 0.28507348 | 0.10000000 | 1.85e-01 |
| 10 | 0.24796847 | 0.09090909 | 1.57e-01 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 0.000000 |
| Bonferroni p (no spike) | 1.000000 |
| Spectral flatness (Wiener entropy) | 0.000000 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=22499991.000, p=0.000000 |
| Periodogram KS vs Exp(1) | D=1.000000, p=0.000000 |


## CounterRng

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.000582  Var = 0.000000  Min = 0.000000  Max = 0.001164

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -2236.067307 | 0.000000 | REJECT |
| randtests::bartels.rank.test | -2236.068291 | 0.000000 | REJECT |
| randtests::cox.stuart.test (trend) | 2500000.000000 | 0.000000 | REJECT |
| randtests::difference.sign.test | 3872.982184 | 0.000000 | REJECT |
| randtests::turning.point.test | -3535.533133 | 0.000000 | REJECT |
| randtests::rank.test (Mann-Kendall, n=5000) | 106.028906 | 0.000000 | REJECT |
| randtoolbox::freq.test (16 bins) | 75000000.000000 | 0.000000 | REJECT |
| randtoolbox::gap.test [0,0.5) | NA | NA | n/a |
| randtoolbox::serial.test (d=8) | 157500000.000000 | 0.000000 | REJECT |
| randtoolbox::poker.test (5-hand) | 624000000.000000 | 0.000000 | REJECT |
| randtoolbox::order.test (d=4) | 28750000.000000 | 0.000000 | REJECT |
| stats::ks.test vs U(0,1) | 0.998836 | 0.000000 | REJECT |
| stats::chisq.test (256 bins) | 1275000000.000000 | 0.000000 | REJECT |
| stats::Box.test (Ljung-Box, lag 25) | 124998425.003771 | 0.000000 | REJECT |
| tseries::runs.test (binary) | -2236.067307 | 0.000000 | REJECT |
| tseries::jarque.bera.test (vs Normal*) | 300000.000001 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.00058208 | 0.50000000 | 4.99e-01 |
| 2 | 0.00000045 | 0.33333333 | 3.33e-01 |
| 3 | 0.00000000 | 0.25000000 | 2.50e-01 |
| 4 | 0.00000000 | 0.20000000 | 2.00e-01 |
| 5 | 0.00000000 | 0.16666667 | 1.67e-01 |
| 6 | 0.00000000 | 0.14285714 | 1.43e-01 |
| 7 | 0.00000000 | 0.12500000 | 1.25e-01 |
| 8 | 0.00000000 | 0.11111111 | 1.11e-01 |
| 9 | 0.00000000 | 0.10000000 | 1.00e-01 |
| 10 | 0.00000000 | 0.09090909 | 9.09e-02 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 2.059737 |
| Bonferroni p (no spike) | 1.000000 |
| Spectral flatness (Wiener entropy) | 0.000002 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=22499911.000, p=0.000000 |
| Periodogram KS vs Exp(1) | D=0.999870, p=0.000000 |


## System V rand()

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500021  Var = 0.083345  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.575117 | 0.565212 | pass |
| randtests::bartels.rank.test | 0.069854 | 0.944310 | pass |
| randtests::cox.stuart.test (trend) | 1250336.000000 | 0.671290 | pass |
| randtests::difference.sign.test | -0.730445 | 0.465118 | pass |
| randtests::turning.point.test | -0.027577 | 0.977999 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -1.209718 | 0.226387 | pass |
| randtoolbox::freq.test (16 bins) | 5.626586 | 0.985361 | pass |
| randtoolbox::gap.test [0,0.5) | 42.705411 | 0.007504 | pass |
| randtoolbox::serial.test (d=8) | 62.409830 | 0.497311 | pass |
| randtoolbox::poker.test (5-hand) | 183.706248 | 1.193e-38 | REJECT |
| randtoolbox::order.test (d=4) | 18.461747 | 0.731984 | pass |
| stats::ks.test vs U(0,1) | 0.000232 | 0.950797 | pass |
| stats::chisq.test (256 bins) | 200.561152 | 0.995020 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 19.598556 | 0.767554 | pass |
| tseries::runs.test (binary) | -0.575117 | 0.565212 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300034.565080 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50002121 | 0.50000000 | 2.12e-05 |
| 2 | 0.33336622 | 0.33333333 | 3.29e-05 |
| 3 | 0.25003179 | 0.25000000 | 3.18e-05 |
| 4 | 0.20002788 | 0.20000000 | 2.79e-05 |
| 5 | 0.16669094 | 0.16666667 | 2.43e-05 |
| 6 | 0.14287866 | 0.14285714 | 2.15e-05 |
| 7 | 0.12501949 | 0.12500000 | 1.95e-05 |
| 8 | 0.11112906 | 0.11111111 | 1.79e-05 |
| 9 | 0.10001671 | 0.10000000 | 1.67e-05 |
| 10 | 0.09092475 | 0.09090909 | 1.57e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 236.872264 |
| Bonferroni p (no spike) | 3.354e-97 |
| Spectral flatness (Wiener entropy) | 0.400030 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=231577.539, p=0.000000 |
| Periodogram KS vs Exp(1) | D=0.133907, p=0.000000 |


## rand48 (mrand48)

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500064  Var = 0.083350  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.466891 | 0.640578 | pass |
| randtests::bartels.rank.test | -0.391194 | 0.695654 | pass |
| randtests::cox.stuart.test (trend) | 1250835.000000 | 0.291165 | pass |
| randtests::difference.sign.test | -0.608058 | 0.543149 | pass |
| randtests::turning.point.test | -0.160160 | 0.872755 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.540550 | 0.588818 | pass |
| randtoolbox::freq.test (16 bins) | 16.615066 | 0.342391 | pass |
| randtoolbox::gap.test [0,0.5) | 16.394529 | 0.795834 | pass |
| randtoolbox::serial.test (d=8) | 71.644160 | 0.212951 | pass |
| randtoolbox::poker.test (5-hand) | 6.801089 | 0.146781 | pass |
| randtoolbox::order.test (d=4) | 24.067533 | 0.400077 | pass |
| stats::ks.test vs U(0,1) | 0.000419 | 0.344360 | pass |
| stats::chisq.test (256 bins) | 243.425382 | 0.688237 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 20.889371 | 0.698762 | pass |
| tseries::runs.test (binary) | 0.466891 | 0.640578 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300217.635394 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50006378 | 0.50000000 | 6.38e-05 |
| 2 | 0.33341346 | 0.33333333 | 8.01e-05 |
| 3 | 0.25007777 | 0.25000000 | 7.78e-05 |
| 4 | 0.20006911 | 0.20000000 | 6.91e-05 |
| 5 | 0.16672713 | 0.16666667 | 6.05e-05 |
| 6 | 0.14291063 | 0.14285714 | 5.35e-05 |
| 7 | 0.12504814 | 0.12500000 | 4.81e-05 |
| 8 | 0.11115514 | 0.11111111 | 4.40e-05 |
| 9 | 0.10004077 | 0.10000000 | 4.08e-05 |
| 10 | 0.09094715 | 0.09090909 | 3.81e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 15.360800 |
| Bonferroni p (no spike) | 0.413232 |
| Spectral flatness (Wiener entropy) | 0.561703 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=4.209, p=0.897121 |
| Periodogram KS vs Exp(1) | D=0.000352, p=0.916103 |


## BSD random() / glibc random()

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499967  Var = 0.083325  Min = 0.000001  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 1.382785 | 0.166731 | pass |
| randtests::bartels.rank.test | 0.732279 | 0.463998 | pass |
| randtests::cox.stuart.test (trend) | 1249839.000000 | 0.839121 | pass |
| randtests::difference.sign.test | 0.468631 | 0.639333 | pass |
| randtests::turning.point.test | 1.624932 | 0.104177 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.491462 | 0.623100 | pass |
| randtoolbox::freq.test (16 bins) | 10.888461 | 0.760460 | pass |
| randtoolbox::gap.test [0,0.5) | 27.784404 | 0.182937 | pass |
| randtoolbox::serial.test (d=8) | 66.222950 | 0.366343 | pass |
| randtoolbox::poker.test (5-hand) | 2.722297 | 0.605319 | pass |
| randtoolbox::order.test (d=4) | 18.634970 | 0.722196 | pass |
| stats::ks.test vs U(0,1) | 0.000237 | 0.942174 | pass |
| stats::chisq.test (256 bins) | 242.227917 | 0.707341 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 24.439146 | 0.494130 | pass |
| tseries::runs.test (binary) | 1.382785 | 0.166731 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299851.119809 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49996729 | 0.50000000 | 3.27e-05 |
| 2 | 0.33329220 | 0.33333333 | 4.11e-05 |
| 3 | 0.24995689 | 0.25000000 | 4.31e-05 |
| 4 | 0.19995867 | 0.20000000 | 4.13e-05 |
| 5 | 0.16662948 | 0.16666667 | 3.72e-05 |
| 6 | 0.14282531 | 0.14285714 | 3.18e-05 |
| 7 | 0.12497393 | 0.12500000 | 2.61e-05 |
| 8 | 0.11109068 | 0.11111111 | 2.04e-05 |
| 9 | 0.09998483 | 0.10000000 | 1.52e-05 |
| 10 | 0.09089865 | 0.09090909 | 1.04e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.855541 |
| Bonferroni p (no spike) | 0.586711 |
| Spectral flatness (Wiener entropy) | 0.561453 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=11.124, p=0.267321 |
| Periodogram KS vs Exp(1) | D=0.000513, p=0.526190 |


## FreeBSD rand_r() compat

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499993  Var = 0.083294  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -1.913180 | 0.055725 | pass |
| randtests::bartels.rank.test | -1.371787 | 0.170130 | pass |
| randtests::cox.stuart.test (trend) | 1249159.000000 | 0.287710 | pass |
| randtests::difference.sign.test | -0.287375 | 0.773825 | pass |
| randtests::turning.point.test | -1.412800 | 0.157715 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.440914 | 0.659275 | pass |
| randtoolbox::freq.test (16 bins) | 17.520717 | 0.288700 | pass |
| randtoolbox::gap.test [0,0.5) | 20.654736 | 0.542155 | pass |
| randtoolbox::serial.test (d=8) | 60.253030 | 0.574842 | pass |
| randtoolbox::poker.test (5-hand) | 3.497013 | 0.478333 | pass |
| randtoolbox::order.test (d=4) | 32.029542 | 0.099531 | pass |
| stats::ks.test vs U(0,1) | 0.000314 | 0.707130 | pass |
| stats::chisq.test (256 bins) | 221.564416 | 0.935851 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 33.648818 | 0.115630 | pass |
| tseries::runs.test (binary) | -1.913180 | 0.055725 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299624.683478 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49999311 | 0.50000000 | 6.89e-06 |
| 2 | 0.33328713 | 0.33333333 | 4.62e-05 |
| 3 | 0.24992943 | 0.25000000 | 7.06e-05 |
| 4 | 0.19991815 | 0.20000000 | 8.19e-05 |
| 5 | 0.16658133 | 0.16666667 | 8.53e-05 |
| 6 | 0.14277237 | 0.14285714 | 8.48e-05 |
| 7 | 0.12491765 | 0.12500000 | 8.24e-05 |
| 8 | 0.11103187 | 0.11111111 | 7.92e-05 |
| 9 | 0.09992396 | 0.10000000 | 7.60e-05 |
| 10 | 0.09083606 | 0.09090909 | 7.30e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.712254 |
| Bonferroni p (no spike) | 0.639311 |
| Spectral flatness (Wiener entropy) | 0.561834 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=6.062, p=0.733702 |
| Periodogram KS vs Exp(1) | D=0.000378, p=0.867045 |


## Windows MSVC rand()

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499952  Var = 0.083344  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.398020 | 0.690615 | pass |
| randtests::bartels.rank.test | -0.414381 | 0.678595 | pass |
| randtests::cox.stuart.test (trend) | 1250319.000000 | 0.687041 | pass |
| randtests::difference.sign.test | 0.059644 | 0.952439 | pass |
| randtests::turning.point.test | 1.247337 | 0.212274 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 1.133022 | 0.257205 | pass |
| randtoolbox::freq.test (16 bins) | 7.108928 | 0.954556 | pass |
| randtoolbox::gap.test [0,0.5) | 19.606276 | 0.607653 | pass |
| randtoolbox::serial.test (d=8) | 39.472026 | 0.991179 | pass |
| randtoolbox::poker.test (5-hand) | 151.547464 | 9.486e-32 | REJECT |
| randtoolbox::order.test (d=4) | 15.371968 | 0.880808 | pass |
| stats::ks.test vs U(0,1) | 0.000222 | 0.965907 | pass |
| stats::chisq.test (256 bins) | 221.527040 | 0.936079 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 19.500098 | 0.772532 | pass |
| tseries::runs.test (binary) | -0.398020 | 0.690615 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300202.321309 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49995236 | 0.50000000 | 4.76e-05 |
| 2 | 0.33329640 | 0.33333333 | 3.69e-05 |
| 3 | 0.24997013 | 0.25000000 | 2.99e-05 |
| 4 | 0.19997223 | 0.20000000 | 2.78e-05 |
| 5 | 0.16663827 | 0.16666667 | 2.84e-05 |
| 6 | 0.14282680 | 0.14285714 | 3.03e-05 |
| 7 | 0.12496716 | 0.12500000 | 3.28e-05 |
| 8 | 0.11107563 | 0.11111111 | 3.55e-05 |
| 9 | 0.09996197 | 0.10000000 | 3.80e-05 |
| 10 | 0.09086873 | 0.09090909 | 4.04e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 266.300279 |
| Bonferroni p (no spike) | 5.562e-110 |
| Spectral flatness (Wiener entropy) | 0.401885 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=224253.302, p=0.000000 |
| Periodogram KS vs Exp(1) | D=0.132000, p=0.000000 |


## Windows VB6/VBA Rnd()

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.501964  Var = 0.083331  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.033094 | 0.973600 | pass |
| randtests::bartels.rank.test | 0.700452 | 0.483645 | pass |
| randtests::cox.stuart.test (trend) | 1250391.000000 | 0.621343 | pass |
| randtests::difference.sign.test | 0.126259 | 0.899527 | pass |
| randtests::turning.point.test | 0.389262 | 0.697082 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.011199 | 0.991065 | pass |
| randtoolbox::freq.test (16 bins) | 1.906099 | 0.999978 | pass |
| randtoolbox::gap.test [0,0.5) | 726.864651 | 1.659e-139 | REJECT |
| randtoolbox::serial.test (d=8) | 28.826880 | 0.999933 | pass |
| randtoolbox::poker.test (5-hand) | 153382.706336 | 0.000000 | REJECT |
| randtoolbox::order.test (d=4) | 67681.504384 | 0.000000 | REJECT |
| stats::ks.test vs U(0,1) | 0.003998 | 7.457e-70 | REJECT |
| stats::chisq.test (256 bins) | 1666607.643750 | 0.000000 | REJECT |
| stats::Box.test (Ljung-Box, lag 25) | 168582.325774 | 0.000000 | REJECT |
| tseries::runs.test (binary) | -0.033094 | 0.973600 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299992.385727 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50196383 | 0.50000000 | 1.96e-03 |
| 2 | 0.33529825 | 0.33333333 | 1.96e-03 |
| 3 | 0.25196093 | 0.25000000 | 1.96e-03 |
| 4 | 0.20195758 | 0.20000000 | 1.96e-03 |
| 5 | 0.16862235 | 0.16666667 | 1.96e-03 |
| 6 | 0.14481227 | 0.14285714 | 1.96e-03 |
| 7 | 0.12695563 | 0.12500000 | 1.96e-03 |
| 8 | 0.11306799 | 0.11111111 | 1.96e-03 |
| 9 | 0.10195866 | 0.10000000 | 1.96e-03 |
| 10 | 0.09286992 | 0.09090909 | 1.96e-03 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 49073.713120 |
| Bonferroni p (no spike) | 0.000000 |
| Spectral flatness (Wiener entropy) | 0.210474 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=1598713.924, p=0.000000 |
| Periodogram KS vs Exp(1) | D=0.341024, p=0.000000 |


## Windows .NET Random

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499982  Var = 0.083341  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.148475 | 0.881968 | pass |
| randtests::bartels.rank.test | 0.872169 | 0.383116 | pass |
| randtests::cox.stuart.test (trend) | 1249215.000000 | 0.321040 | pass |
| randtests::difference.sign.test | 2.050357 | 0.040330 | pass |
| randtests::turning.point.test | -2.749232 | 0.005974 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.775947 | 0.437780 | pass |
| randtoolbox::freq.test (16 bins) | 14.986496 | 0.452390 | pass |
| randtoolbox::gap.test [0,0.5) | 31.592900 | 0.084606 | pass |
| randtoolbox::serial.test (d=8) | 70.186035 | 0.249445 | pass |
| randtoolbox::poker.test (5-hand) | 5.082120 | 0.278975 | pass |
| randtoolbox::order.test (d=4) | 26.352218 | 0.284522 | pass |
| stats::ks.test vs U(0,1) | 0.000235 | 0.944360 | pass |
| stats::chisq.test (256 bins) | 246.566502 | 0.636092 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 25.643154 | 0.426833 | pass |
| tseries::runs.test (binary) | 0.148475 | 0.881968 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300078.556569 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49998154 | 0.50000000 | 1.85e-05 |
| 2 | 0.33332206 | 0.33333333 | 1.13e-05 |
| 3 | 0.24999432 | 0.25000000 | 5.68e-06 |
| 4 | 0.19999740 | 0.20000000 | 2.60e-06 |
| 5 | 0.16666542 | 0.16666667 | 1.25e-06 |
| 6 | 0.14285611 | 0.14285714 | 1.04e-06 |
| 7 | 0.12499845 | 0.12500000 | 1.55e-06 |
| 8 | 0.11110862 | 0.11111111 | 2.49e-06 |
| 9 | 0.09999633 | 0.10000000 | 3.67e-06 |
| 10 | 0.09090411 | 0.09090909 | 4.98e-06 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 13.250183 |
| Bonferroni p (no spike) | 0.987723 |
| Spectral flatness (Wiener entropy) | 0.561375 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=2.935, p=0.966800 |
| Periodogram KS vs Exp(1) | D=0.000309, p=0.970983 |


## ANSI C LCG

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.249925  Var = 0.020834  Min = 0.000000  Max = 0.500000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.702125 | 0.482601 | pass |
| randtests::bartels.rank.test | 1.763738 | 0.077776 | pass |
| randtests::cox.stuart.test (trend) | 1249092.000000 | 0.251007 | pass |
| randtests::difference.sign.test | 0.546091 | 0.585004 | pass |
| randtests::turning.point.test | 1.489167 | 0.136443 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.254283 | 0.799277 | pass |
| randtoolbox::freq.test (16 bins) | 5000018.428557 | 0.000000 | REJECT |
| randtoolbox::gap.test [0,0.5) | NA | NA | n/a |
| randtoolbox::serial.test (d=8) | 7500054.191104 | 0.000000 | REJECT |
| randtoolbox::poker.test (5-hand) | 1901180.837510 | 0.000000 | REJECT |
| randtoolbox::order.test (d=4) | 14.650547 | 0.906719 | pass |
| stats::ks.test vs U(0,1) | 0.500000 | 0.000000 | REJECT |
| stats::chisq.test (256 bins) | 5000235.819827 | 0.000000 | REJECT |
| stats::Box.test (Ljung-Box, lag 25) | 21.008521 | 0.692134 | pass |
| tseries::runs.test (binary) | 0.702125 | 0.482601 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299803.769635 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.24992452 | 0.50000000 | 2.50e-01 |
| 2 | 0.08329597 | 0.33333333 | 2.50e-01 |
| 3 | 0.03123356 | 0.25000000 | 2.19e-01 |
| 4 | 0.01249305 | 0.20000000 | 1.88e-01 |
| 5 | 0.00520544 | 0.16666667 | 1.61e-01 |
| 6 | 0.00223095 | 0.14285714 | 1.41e-01 |
| 7 | 0.00097607 | 0.12500000 | 1.24e-01 |
| 8 | 0.00043382 | 0.11111111 | 1.11e-01 |
| 9 | 0.00019523 | 0.10000000 | 9.98e-02 |
| 10 | 0.00008874 | 0.09090909 | 9.08e-02 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 3.840927 |
| Bonferroni p (no spike) | 1.000000 |
| Spectral flatness (Wiener entropy) | 0.561550 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=3152681.049, p=0.000000 |
| Periodogram KS vs Exp(1) | D=0.472552, p=0.000000 |


## MINSTD (Park-Miller)

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.250089  Var = 0.020827  Min = 0.000000  Max = 0.500000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.469574 | 0.638659 | pass |
| randtests::bartels.rank.test | -0.021087 | 0.983176 | pass |
| randtests::cox.stuart.test (trend) | 1248646.000000 | 0.086886 | pass |
| randtests::difference.sign.test | -1.385753 | 0.165822 | pass |
| randtests::turning.point.test | -1.476439 | 0.139826 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.673885 | 0.500385 | pass |
| randtoolbox::freq.test (16 bins) | 5000012.853101 | 0.000000 | REJECT |
| randtoolbox::gap.test [0,0.5) | NA | NA | n/a |
| randtoolbox::serial.test (d=8) | 7500104.932045 | 0.000000 | REJECT |
| randtoolbox::poker.test (5-hand) | 1897434.735262 | 0.000000 | REJECT |
| randtoolbox::order.test (d=4) | 20.958746 | 0.583607 | pass |
| stats::ks.test vs U(0,1) | 0.500000 | 0.000000 | REJECT |
| stats::chisq.test (256 bins) | 5000232.746189 | 0.000000 | REJECT |
| stats::Box.test (Ljung-Box, lag 25) | 30.258356 | 0.214761 | pass |
| tseries::runs.test (binary) | 0.469574 | 0.638659 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299980.245214 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.25008851 | 0.50000000 | 2.50e-01 |
| 2 | 0.08337121 | 0.33333333 | 2.50e-01 |
| 3 | 0.03126602 | 0.25000000 | 2.19e-01 |
| 4 | 0.01250689 | 0.20000000 | 1.87e-01 |
| 5 | 0.00521132 | 0.16666667 | 1.61e-01 |
| 6 | 0.00223344 | 0.14285714 | 1.41e-01 |
| 7 | 0.00097713 | 0.12500000 | 1.24e-01 |
| 8 | 0.00043428 | 0.11111111 | 1.11e-01 |
| 9 | 0.00019542 | 0.10000000 | 9.98e-02 |
| 10 | 0.00008883 | 0.09090909 | 9.08e-02 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 3.546700 |
| Bonferroni p (no spike) | 1.000000 |
| Spectral flatness (Wiener entropy) | 0.562505 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=3149485.991, p=0.000000 |
| Periodogram KS vs Exp(1) | D=0.472539, p=0.000000 |


## Borland C++ LCG

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.000004  Var = 0.000000  Min = 0.000000  Max = 0.000008

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.347045 | 0.728557 | pass |
| randtests::bartels.rank.test | 0.646249 | 0.518118 | pass |
| randtests::cox.stuart.test (trend) | 1250024.000000 | 0.929443 | pass |
| randtests::difference.sign.test | 0.663839 | 0.506793 | pass |
| randtests::turning.point.test | 0.306889 | 0.758928 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.387142 | 0.698651 | pass |
| randtoolbox::freq.test (16 bins) | 75000000.000000 | 0.000000 | REJECT |
| randtoolbox::gap.test [0,0.5) | NA | NA | n/a |
| randtoolbox::serial.test (d=8) | 157500000.000000 | 0.000000 | REJECT |
| randtoolbox::poker.test (5-hand) | 624000000.000000 | 0.000000 | REJECT |
| randtoolbox::order.test (d=4) | 32.739494 | 0.085715 | pass |
| stats::ks.test vs U(0,1) | 0.999992 | 0.000000 | REJECT |
| stats::chisq.test (256 bins) | 1275000000.000000 | 0.000000 | REJECT |
| stats::Box.test (Ljung-Box, lag 25) | 19.968653 | 0.748468 | pass |
| tseries::runs.test (binary) | 0.339890 | 0.733939 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300453.303563 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.00000382 | 0.50000000 | 5.00e-01 |
| 2 | 0.00000000 | 0.33333333 | 3.33e-01 |
| 3 | 0.00000000 | 0.25000000 | 2.50e-01 |
| 4 | 0.00000000 | 0.20000000 | 2.00e-01 |
| 5 | 0.00000000 | 0.16666667 | 1.67e-01 |
| 6 | 0.00000000 | 0.14285714 | 1.43e-01 |
| 7 | 0.00000000 | 0.12500000 | 1.25e-01 |
| 8 | 0.00000000 | 0.11111111 | 1.11e-01 |
| 9 | 0.00000000 | 0.10000000 | 1.00e-01 |
| 10 | 0.00000000 | 0.09090909 | 9.09e-02 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 0.000000 |
| Bonferroni p (no spike) | 1.000000 |
| Spectral flatness (Wiener entropy) | 0.562021 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=22499991.000, p=0.000000 |
| Periodogram KS vs Exp(1) | D=1.000000, p=0.000000 |


## MSVC LCG

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.000004  Var = 0.000000  Min = 0.000000  Max = 0.000008

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.246418 | 0.805358 | pass |
| randtests::bartels.rank.test | -0.502465 | 0.615341 | pass |
| randtests::cox.stuart.test (trend) | 1248698.000000 | 0.110130 | pass |
| randtests::difference.sign.test | -1.655340 | 0.097856 | pass |
| randtests::turning.point.test | 0.435231 | 0.663394 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 1.238530 | 0.215520 | pass |
| randtoolbox::freq.test (16 bins) | 75000000.000000 | 0.000000 | REJECT |
| randtoolbox::gap.test [0,0.5) | NA | NA | n/a |
| randtoolbox::serial.test (d=8) | 157500000.000000 | 0.000000 | REJECT |
| randtoolbox::poker.test (5-hand) | 624000000.000000 | 0.000000 | REJECT |
| randtoolbox::order.test (d=4) | 15.749472 | 0.865782 | pass |
| stats::ks.test vs U(0,1) | 0.999992 | 0.000000 | REJECT |
| stats::chisq.test (256 bins) | 1275000000.000000 | 0.000000 | REJECT |
| stats::Box.test (Ljung-Box, lag 25) | 35.838220 | 0.074091 | pass |
| tseries::runs.test (binary) | -0.255805 | 0.798102 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299671.025815 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.00000382 | 0.50000000 | 5.00e-01 |
| 2 | 0.00000000 | 0.33333333 | 3.33e-01 |
| 3 | 0.00000000 | 0.25000000 | 2.50e-01 |
| 4 | 0.00000000 | 0.20000000 | 2.00e-01 |
| 5 | 0.00000000 | 0.16666667 | 1.67e-01 |
| 6 | 0.00000000 | 0.14285714 | 1.43e-01 |
| 7 | 0.00000000 | 0.12500000 | 1.25e-01 |
| 8 | 0.00000000 | 0.11111111 | 1.11e-01 |
| 9 | 0.00000000 | 0.10000000 | 1.00e-01 |
| 10 | 0.00000000 | 0.09090909 | 9.09e-02 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 0.000000 |
| Bonferroni p (no spike) | 1.000000 |
| Spectral flatness (Wiener entropy) | 0.561643 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=22499991.000, p=0.000000 |
| Periodogram KS vs Exp(1) | D=1.000000, p=0.000000 |


## MT19937

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500142  Var = 0.083412  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.064399 | 0.948653 | pass |
| randtests::bartels.rank.test | -0.736700 | 0.461305 | pass |
| randtests::cox.stuart.test (trend) | 1249610.000000 | 0.622237 | pass |
| randtests::difference.sign.test | -1.622780 | 0.104636 | pass |
| randtests::turning.point.test | -0.932320 | 0.351171 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.795715 | 0.426198 | pass |
| randtoolbox::freq.test (16 bins) | 17.929677 | 0.266381 | pass |
| randtoolbox::gap.test [0,0.5) | 36.223528 | 0.028732 | pass |
| randtoolbox::serial.test (d=8) | 78.791629 | 0.086561 | pass |
| randtoolbox::poker.test (5-hand) | 2.681619 | 0.612435 | pass |
| randtoolbox::order.test (d=4) | 20.738906 | 0.597029 | pass |
| stats::ks.test vs U(0,1) | 0.000504 | 0.158075 | pass |
| stats::chisq.test (256 bins) | 308.476211 | 0.012251 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 17.024995 | 0.880885 | pass |
| tseries::runs.test (binary) | -0.064399 | 0.948653 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300104.776329 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50014152 | 0.50000000 | 1.42e-04 |
| 2 | 0.33355381 | 0.33333333 | 2.20e-04 |
| 3 | 0.25025466 | 0.25000000 | 2.55e-04 |
| 4 | 0.20027161 | 0.20000000 | 2.72e-04 |
| 5 | 0.16694711 | 0.16666667 | 2.80e-04 |
| 6 | 0.14314160 | 0.14285714 | 2.84e-04 |
| 7 | 0.12528522 | 0.12500000 | 2.85e-04 |
| 8 | 0.11139484 | 0.11111111 | 2.84e-04 |
| 9 | 0.10028066 | 0.10000000 | 2.81e-04 |
| 10 | 0.09118561 | 0.09090909 | 2.77e-04 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 16.060113 |
| Bonferroni p (no spike) | 0.232736 |
| Spectral flatness (Wiener entropy) | 0.561230 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=14.760, p=0.097738 |
| Periodogram KS vs Exp(1) | D=0.000774, p=0.100387 |


## Xorshift32

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500045  Var = 0.083328  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -1.516054 | 0.129506 | pass |
| randtests::bartels.rank.test | -0.887402 | 0.374862 | pass |
| randtests::cox.stuart.test (trend) | 1250500.000000 | 0.527503 | pass |
| randtests::difference.sign.test | 0.761428 | 0.446401 | pass |
| randtests::turning.point.test | -1.805244 | 0.071037 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.369954 | 0.711417 | pass |
| randtoolbox::freq.test (16 bins) | 19.975085 | 0.172891 | pass |
| randtoolbox::gap.test [0,0.5) | 21.264227 | 0.504464 | pass |
| randtoolbox::serial.test (d=8) | 71.735296 | 0.210794 | pass |
| randtoolbox::poker.test (5-hand) | 1.473542 | 0.831318 | pass |
| randtoolbox::order.test (d=4) | 20.074816 | 0.637394 | pass |
| stats::ks.test vs U(0,1) | 0.000268 | 0.866484 | pass |
| stats::chisq.test (256 bins) | 255.045530 | 0.487419 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 21.017782 | 0.691618 | pass |
| tseries::runs.test (binary) | -1.516054 | 0.129506 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300054.621924 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50004525 | 0.50000000 | 4.52e-05 |
| 2 | 0.33337289 | 0.33333333 | 3.96e-05 |
| 3 | 0.25004307 | 0.25000000 | 4.31e-05 |
| 4 | 0.20004696 | 0.20000000 | 4.70e-05 |
| 5 | 0.16671644 | 0.16666667 | 4.98e-05 |
| 6 | 0.14290853 | 0.14285714 | 5.14e-05 |
| 7 | 0.12505196 | 0.12500000 | 5.20e-05 |
| 8 | 0.11116285 | 0.11111111 | 5.17e-05 |
| 9 | 0.10005094 | 0.10000000 | 5.09e-05 |
| 10 | 0.09095884 | 0.09090909 | 4.97e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 15.724777 |
| Bonferroni p (no spike) | 0.309592 |
| Spectral flatness (Wiener entropy) | 0.562245 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=14.399, p=0.108827 |
| Periodogram KS vs Exp(1) | D=0.000555, p=0.423891 |


## Xorshift64

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499786  Var = 0.083363  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.635043 | 0.525400 | pass |
| randtests::bartels.rank.test | 0.692801 | 0.488434 | pass |
| randtests::cox.stuart.test (trend) | 1250865.000000 | 0.274167 | pass |
| randtests::difference.sign.test | 0.287375 | 0.773825 | pass |
| randtests::turning.point.test | 1.121118 | 0.262238 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.074812 | 0.940364 | pass |
| randtoolbox::freq.test (16 bins) | 22.532941 | 0.094572 | pass |
| randtoolbox::gap.test [0,0.5) | 33.316844 | 0.057554 | pass |
| randtoolbox::serial.test (d=8) | 58.959872 | 0.621020 | pass |
| randtoolbox::poker.test (5-hand) | 2.856303 | 0.582154 | pass |
| randtoolbox::order.test (d=4) | 15.328269 | 0.882484 | pass |
| stats::ks.test vs U(0,1) | 0.000538 | 0.110640 | pass |
| stats::chisq.test (256 bins) | 242.139443 | 0.708733 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 22.176845 | 0.625513 | pass |
| tseries::runs.test (binary) | -0.635043 | 0.525400 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300186.329216 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49978556 | 0.50000000 | 2.14e-04 |
| 2 | 0.33314850 | 0.33333333 | 1.85e-04 |
| 3 | 0.24984523 | 0.25000000 | 1.55e-04 |
| 4 | 0.19986678 | 0.20000000 | 1.33e-04 |
| 5 | 0.16654764 | 0.16666667 | 1.19e-04 |
| 6 | 0.14274693 | 0.14285714 | 1.10e-04 |
| 7 | 0.12489501 | 0.12500000 | 1.05e-04 |
| 8 | 0.11100905 | 0.11111111 | 1.02e-04 |
| 9 | 0.09989948 | 0.10000000 | 1.01e-04 |
| 10 | 0.09080929 | 0.09090909 | 9.98e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.826968 |
| Bonferroni p (no spike) | 0.597162 |
| Spectral flatness (Wiener entropy) | 0.561340 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=12.777, p=0.172958 |
| Periodogram KS vs Exp(1) | D=0.000474, p=0.628904 |


## PCG32

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500013  Var = 0.083317  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -1.088518 | 0.276366 | pass |
| randtests::bartels.rank.test | -1.113971 | 0.265292 | pass |
| randtests::cox.stuart.test (trend) | 1249730.000000 | 0.733184 | pass |
| randtests::difference.sign.test | -0.691715 | 0.489116 | pass |
| randtests::turning.point.test | -1.202789 | 0.229058 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.729455 | 0.465723 | pass |
| randtoolbox::freq.test (16 bins) | 34.311430 | 0.003079 | pass |
| randtoolbox::gap.test [0,0.5) | 29.876192 | 0.121505 | pass |
| randtoolbox::serial.test (d=8) | 89.055642 | 0.017041 | pass |
| randtoolbox::poker.test (5-hand) | 6.537588 | 0.162437 | pass |
| randtoolbox::order.test (d=4) | 27.115571 | 0.251081 | pass |
| stats::ks.test vs U(0,1) | 0.000396 | 0.413139 | pass |
| stats::chisq.test (256 bins) | 296.068096 | 0.039348 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 28.510545 | 0.284892 | pass |
| tseries::runs.test (binary) | -1.088518 | 0.276366 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300098.161624 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50001284 | 0.50000000 | 1.28e-05 |
| 2 | 0.33332935 | 0.33333333 | 3.99e-06 |
| 3 | 0.24997976 | 0.25000000 | 2.02e-05 |
| 4 | 0.19996551 | 0.20000000 | 3.45e-05 |
| 5 | 0.16662020 | 0.16666667 | 4.65e-05 |
| 6 | 0.14280135 | 0.14285714 | 5.58e-05 |
| 7 | 0.12493763 | 0.12500000 | 6.24e-05 |
| 8 | 0.11104468 | 0.11111111 | 6.64e-05 |
| 9 | 0.09993164 | 0.10000000 | 6.84e-05 |
| 10 | 0.09084048 | 0.09090909 | 6.86e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.498393 |
| Bonferroni p (no spike) | 0.717166 |
| Spectral flatness (Wiener entropy) | 0.561960 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=7.309, p=0.604987 |
| Periodogram KS vs Exp(1) | D=0.000597, p=0.334456 |


## PCG64

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500071  Var = 0.083258  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.104648 | 0.916655 | pass |
| randtests::bartels.rank.test | -0.484790 | 0.627825 | pass |
| randtests::cox.stuart.test (trend) | 1250385.000000 | 0.626713 | pass |
| randtests::difference.sign.test | -0.116964 | 0.906889 | pass |
| randtests::turning.point.test | -1.303552 | 0.192387 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.939112 | 0.347673 | pass |
| randtoolbox::freq.test (16 bins) | 12.931904 | 0.607558 | pass |
| randtoolbox::gap.test [0,0.5) | 39.326769 | 0.012943 | pass |
| randtoolbox::serial.test (d=8) | 66.434304 | 0.359523 | pass |
| randtoolbox::poker.test (5-hand) | 1.950430 | 0.744876 | pass |
| randtoolbox::order.test (d=4) | 14.366080 | 0.915907 | pass |
| stats::ks.test vs U(0,1) | 0.000418 | 0.347816 | pass |
| stats::chisq.test (256 bins) | 222.467891 | 0.930157 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 37.049726 | 0.057113 | pass |
| tseries::runs.test (binary) | -0.104648 | 0.916655 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299341.521910 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50007117 | 0.50000000 | 7.12e-05 |
| 2 | 0.33332965 | 0.33333333 | 3.68e-06 |
| 3 | 0.24996247 | 0.25000000 | 3.75e-05 |
| 4 | 0.19995273 | 0.20000000 | 4.73e-05 |
| 5 | 0.16662025 | 0.16666667 | 4.64e-05 |
| 6 | 0.14281564 | 0.14285714 | 4.15e-05 |
| 7 | 0.12496453 | 0.12500000 | 3.55e-05 |
| 8 | 0.11108152 | 0.11111111 | 2.96e-05 |
| 9 | 0.09997566 | 0.10000000 | 2.43e-05 |
| 10 | 0.09088922 | 0.09090909 | 1.99e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 15.947827 |
| Bonferroni p (no spike) | 0.256514 |
| Spectral flatness (Wiener entropy) | 0.561376 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=7.624, p=0.572439 |
| Periodogram KS vs Exp(1) | D=0.000501, p=0.556825 |


## Xoshiro256

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499995  Var = 0.083310  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -1.126978 | 0.259752 | pass |
| randtests::bartels.rank.test | -0.011415 | 0.990892 | pass |
| randtests::cox.stuart.test (trend) | 1248947.000000 | 0.183084 | pass |
| randtests::difference.sign.test | -0.051898 | 0.958610 | pass |
| randtests::turning.point.test | 0.278247 | 0.780823 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.187157 | 0.851537 | pass |
| randtoolbox::freq.test (16 bins) | 5.801280 | 0.982896 | pass |
| randtoolbox::gap.test [0,0.5) | 14.054065 | 0.899549 | pass |
| randtoolbox::serial.test (d=8) | 44.313293 | 0.964414 | pass |
| randtoolbox::poker.test (5-hand) | 3.860464 | 0.425219 | pass |
| randtoolbox::order.test (d=4) | 22.905395 | 0.466307 | pass |
| stats::ks.test vs U(0,1) | 0.000193 | 0.992459 | pass |
| stats::chisq.test (256 bins) | 227.251712 | 0.893717 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 15.954242 | 0.916185 | pass |
| tseries::runs.test (binary) | -1.126978 | 0.259752 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299946.868290 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49999538 | 0.50000000 | 4.62e-06 |
| 2 | 0.33330518 | 0.33333333 | 2.82e-05 |
| 3 | 0.24996421 | 0.25000000 | 3.58e-05 |
| 4 | 0.19996203 | 0.20000000 | 3.80e-05 |
| 5 | 0.16662800 | 0.16666667 | 3.87e-05 |
| 6 | 0.14281805 | 0.14285714 | 3.91e-05 |
| 7 | 0.12496038 | 0.12500000 | 3.96e-05 |
| 8 | 0.11107082 | 0.11111111 | 4.03e-05 |
| 9 | 0.09995892 | 0.10000000 | 4.11e-05 |
| 10 | 0.09086713 | 0.09090909 | 4.20e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 15.122129 |
| Bonferroni p (no spike) | 0.491777 |
| Spectral flatness (Wiener entropy) | 0.561140 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=4.851, p=0.847075 |
| Periodogram KS vs Exp(1) | D=0.000557, p=0.419841 |


## Xoroshiro128

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500050  Var = 0.083362  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.193196 | 0.846805 | pass |
| randtests::bartels.rank.test | -1.456595 | 0.145228 | pass |
| randtests::cox.stuart.test (trend) | 1250337.000000 | 0.670368 | pass |
| randtests::difference.sign.test | 0.141751 | 0.887277 | pass |
| randtests::turning.point.test | -0.310773 | 0.755973 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.555397 | 0.578623 | pass |
| randtoolbox::freq.test (16 bins) | 12.179475 | 0.665396 | pass |
| randtoolbox::gap.test [0,0.5) | 14.898345 | 0.866563 | pass |
| randtoolbox::serial.test (d=8) | 49.834445 | 0.886038 | pass |
| randtoolbox::poker.test (5-hand) | 6.036120 | 0.196467 | pass |
| randtoolbox::order.test (d=4) | 23.774925 | 0.416358 | pass |
| stats::ks.test vs U(0,1) | 0.000360 | 0.534909 | pass |
| stats::chisq.test (256 bins) | 282.789888 | 0.111657 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 33.035798 | 0.130176 | pass |
| tseries::runs.test (binary) | -0.193196 | 0.846805 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300336.148216 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50004990 | 0.50000000 | 4.99e-05 |
| 2 | 0.33341153 | 0.33333333 | 7.82e-05 |
| 3 | 0.25009474 | 0.25000000 | 9.47e-05 |
| 4 | 0.20010097 | 0.20000000 | 1.01e-04 |
| 5 | 0.16676784 | 0.16666667 | 1.01e-04 |
| 6 | 0.14295552 | 0.14285714 | 9.84e-05 |
| 7 | 0.12509437 | 0.12500000 | 9.44e-05 |
| 8 | 0.11120119 | 0.11111111 | 9.01e-05 |
| 9 | 0.10008602 | 0.10000000 | 8.60e-05 |
| 10 | 0.09099148 | 0.09090909 | 8.24e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 13.447951 |
| Bonferroni p (no spike) | 0.972962 |
| Spectral flatness (Wiener entropy) | 0.561206 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=3.343, p=0.949152 |
| Periodogram KS vs Exp(1) | D=0.000428, p=0.748401 |


## WyRand

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500017  Var = 0.083338  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.069765 | 0.944380 | pass |
| randtests::bartels.rank.test | 0.414681 | 0.678376 | pass |
| randtests::cox.stuart.test (trend) | 1251433.000000 | 0.069988 | pass |
| randtests::difference.sign.test | 0.394270 | 0.693382 | pass |
| randtests::turning.point.test | 1.209153 | 0.226604 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.214849 | 0.829885 | pass |
| randtoolbox::freq.test (16 bins) | 20.267142 | 0.161923 | pass |
| randtoolbox::gap.test [0,0.5) | 27.670040 | 0.186876 | pass |
| randtoolbox::serial.test (d=8) | 61.948467 | 0.513835 | pass |
| randtoolbox::poker.test (5-hand) | 2.842826 | 0.584463 | pass |
| randtoolbox::order.test (d=4) | 23.236557 | 0.447030 | pass |
| stats::ks.test vs U(0,1) | 0.000296 | 0.774704 | pass |
| stats::chisq.test (256 bins) | 221.930086 | 0.933590 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 27.287789 | 0.341662 | pass |
| tseries::runs.test (binary) | 0.069765 | 0.944380 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300006.715444 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50001689 | 0.50000000 | 1.69e-05 |
| 2 | 0.33335485 | 0.33333333 | 2.15e-05 |
| 3 | 0.25001431 | 0.25000000 | 1.43e-05 |
| 4 | 0.20000608 | 0.20000000 | 6.08e-06 |
| 5 | 0.16666666 | 0.16666667 | 8.58e-09 |
| 6 | 0.14285328 | 0.14285714 | 3.86e-06 |
| 7 | 0.12499396 | 0.12500000 | 6.04e-06 |
| 8 | 0.11110402 | 0.11111111 | 7.09e-06 |
| 9 | 0.09999258 | 0.10000000 | 7.42e-06 |
| 10 | 0.09090177 | 0.09090909 | 7.32e-06 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.530347 |
| Bonferroni p (no spike) | 0.705707 |
| Spectral flatness (Wiener entropy) | 0.561280 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=6.126, p=0.727220 |
| Periodogram KS vs Exp(1) | D=0.000366, p=0.890261 |


## SFC64

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500200  Var = 0.083339  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.003578 | 0.997145 | pass |
| randtests::bartels.rank.test | -0.395116 | 0.692757 | pass |
| randtests::cox.stuart.test (trend) | 1250069.000000 | 0.930953 | pass |
| randtests::difference.sign.test | -0.594116 | 0.552435 | pass |
| randtests::turning.point.test | -0.432749 | 0.665197 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -1.293676 | 0.195777 | pass |
| randtoolbox::freq.test (16 bins) | 10.734950 | 0.771147 | pass |
| randtoolbox::gap.test [0,0.5) | 14.683612 | 0.875454 | pass |
| randtoolbox::serial.test (d=8) | 61.429197 | 0.532501 | pass |
| randtoolbox::poker.test (5-hand) | 1.428094 | 0.839298 | pass |
| randtoolbox::order.test (d=4) | 26.368346 | 0.283787 | pass |
| stats::ks.test vs U(0,1) | 0.000460 | 0.241526 | pass |
| stats::chisq.test (256 bins) | 260.265267 | 0.397031 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 22.194477 | 0.624491 | pass |
| tseries::runs.test (binary) | 0.003578 | 0.997145 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299927.344314 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50020042 | 0.50000000 | 2.00e-04 |
| 2 | 0.33353931 | 0.33333333 | 2.06e-04 |
| 3 | 0.25018385 | 0.25000000 | 1.84e-04 |
| 4 | 0.20016162 | 0.20000000 | 1.62e-04 |
| 5 | 0.16680970 | 0.16666667 | 1.43e-04 |
| 6 | 0.14298497 | 0.14285714 | 1.28e-04 |
| 7 | 0.12511521 | 0.12500000 | 1.15e-04 |
| 8 | 0.11121562 | 0.11111111 | 1.05e-04 |
| 9 | 0.10009525 | 0.10000000 | 9.53e-05 |
| 10 | 0.09099619 | 0.09090909 | 8.71e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 15.709000 |
| Bonferroni p (no spike) | 0.313647 |
| Spectral flatness (Wiener entropy) | 0.561742 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=1.309, p=0.998333 |
| Periodogram KS vs Exp(1) | D=0.000329, p=0.949970 |


## JSF64

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500174  Var = 0.083318  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.232551 | 0.816110 | pass |
| randtests::bartels.rank.test | -0.260164 | 0.794738 | pass |
| randtests::cox.stuart.test (trend) | 1250711.000000 | 0.368802 | pass |
| randtests::difference.sign.test | 0.298220 | 0.765535 | pass |
| randtests::turning.point.test | -0.531391 | 0.595148 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 1.104211 | 0.269502 | pass |
| randtoolbox::freq.test (16 bins) | 14.157184 | 0.513643 | pass |
| randtoolbox::gap.test [0,0.5) | 32.976176 | 0.081483 | pass |
| randtoolbox::serial.test (d=8) | 51.937843 | 0.838806 | pass |
| randtoolbox::poker.test (5-hand) | 4.193368 | 0.380468 | pass |
| randtoolbox::order.test (d=4) | 23.224269 | 0.447740 | pass |
| stats::ks.test vs U(0,1) | 0.000522 | 0.131647 | pass |
| stats::chisq.test (256 bins) | 251.749786 | 0.545770 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 12.416895 | 0.982879 | pass |
| tseries::runs.test (binary) | -0.232551 | 0.816110 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299547.890820 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50017399 | 0.50000000 | 1.74e-04 |
| 2 | 0.33349195 | 0.33333333 | 1.59e-04 |
| 3 | 0.25012466 | 0.25000000 | 1.25e-04 |
| 4 | 0.20010007 | 0.20000000 | 1.00e-04 |
| 5 | 0.16675074 | 0.16666667 | 8.41e-05 |
| 6 | 0.14293056 | 0.14285714 | 7.34e-05 |
| 7 | 0.12506574 | 0.12500000 | 6.57e-05 |
| 8 | 0.11117074 | 0.11111111 | 5.96e-05 |
| 9 | 0.10005434 | 0.10000000 | 5.43e-05 |
| 10 | 0.09095857 | 0.09090909 | 4.95e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 15.161999 |
| Bonferroni p (no spike) | 0.478153 |
| Spectral flatness (Wiener entropy) | 0.561315 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=15.799, p=0.071196 |
| Periodogram KS vs Exp(1) | D=0.000468, p=0.643387 |


## AES-128-CTR

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500125  Var = 0.083382  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.724486 | 0.468767 | pass |
| randtests::bartels.rank.test | 1.356992 | 0.174784 | pass |
| randtests::cox.stuart.test (trend) | 1250117.000000 | 0.882846 | pass |
| randtests::difference.sign.test | 0.248646 | 0.803635 | pass |
| randtests::turning.point.test | 0.171827 | 0.863574 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.030746 | 0.975472 | pass |
| randtoolbox::freq.test (16 bins) | 14.769702 | 0.468130 | pass |
| randtoolbox::gap.test [0,0.5) | 14.676387 | 0.875748 | pass |
| randtoolbox::serial.test (d=8) | 55.750707 | 0.729855 | pass |
| randtoolbox::poker.test (5-hand) | 1.187721 | 0.880116 | pass |
| randtoolbox::order.test (d=4) | 24.269709 | 0.389002 | pass |
| stats::ks.test vs U(0,1) | 0.000456 | 0.249411 | pass |
| stats::chisq.test (256 bins) | 270.682419 | 0.238833 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 13.253697 | 0.973238 | pass |
| tseries::runs.test (binary) | 0.724486 | 0.468767 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300537.404170 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50012488 | 0.50000000 | 1.25e-04 |
| 2 | 0.33350652 | 0.33333333 | 1.73e-04 |
| 3 | 0.25017423 | 0.25000000 | 1.74e-04 |
| 4 | 0.20015815 | 0.20000000 | 1.58e-04 |
| 5 | 0.16680343 | 0.16666667 | 1.37e-04 |
| 6 | 0.14297209 | 0.14285714 | 1.15e-04 |
| 7 | 0.12509455 | 0.12500000 | 9.45e-05 |
| 8 | 0.11118725 | 0.11111111 | 7.61e-05 |
| 9 | 0.10005976 | 0.10000000 | 5.98e-05 |
| 10 | 0.09095437 | 0.09090909 | 4.53e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 17.936983 |
| Bonferroni p (no spike) | 0.039740 |
| Spectral flatness (Wiener entropy) | 0.561593 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=5.042, p=0.830622 |
| Periodogram KS vs Exp(1) | D=0.000514, p=0.524121 |


## Camellia-128-CTR

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500149  Var = 0.083332  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.524134 | 0.600185 | pass |
| randtests::bartels.rank.test | 0.232491 | 0.816157 | pass |
| randtests::cox.stuart.test (trend) | 1249636.000000 | 0.645663 | pass |
| randtests::difference.sign.test | 1.226186 | 0.220129 | pass |
| randtests::turning.point.test | 0.258801 | 0.795789 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.629751 | 0.528857 | pass |
| randtoolbox::freq.test (16 bins) | 18.410944 | 0.241678 | pass |
| randtoolbox::gap.test [0,0.5) | 62.386054 | 1.714e-05 | REJECT |
| randtoolbox::serial.test (d=8) | 50.813747 | 0.865275 | pass |
| randtoolbox::poker.test (5-hand) | 8.002498 | 0.091487 | pass |
| randtoolbox::order.test (d=4) | 25.117274 | 0.344267 | pass |
| stats::ks.test vs U(0,1) | 0.000439 | 0.289456 | pass |
| stats::chisq.test (256 bins) | 271.672525 | 0.225984 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 10.933593 | 0.993264 | pass |
| tseries::runs.test (binary) | 0.524134 | 0.600185 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299799.310866 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50014934 | 0.50000000 | 1.49e-04 |
| 2 | 0.33348175 | 0.33333333 | 1.48e-04 |
| 3 | 0.25013069 | 0.25000000 | 1.31e-04 |
| 4 | 0.20011593 | 0.20000000 | 1.16e-04 |
| 5 | 0.16677247 | 0.16666667 | 1.06e-04 |
| 6 | 0.14295616 | 0.14285714 | 9.90e-05 |
| 7 | 0.12509432 | 0.12500000 | 9.43e-05 |
| 8 | 0.11120202 | 0.11111111 | 9.09e-05 |
| 9 | 0.10008827 | 0.10000000 | 8.83e-05 |
| 10 | 0.09099521 | 0.09090909 | 8.61e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 15.069914 |
| Bonferroni p (no spike) | 0.509885 |
| Spectral flatness (Wiener entropy) | 0.561446 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=3.347, p=0.948929 |
| Periodogram KS vs Exp(1) | D=0.000277, p=0.990752 |


## Twofish-128-CTR

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500028  Var = 0.083348  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.458841 | 0.646348 | pass |
| randtests::bartels.rank.test | -0.183941 | 0.854060 | pass |
| randtests::cox.stuart.test (trend) | 1250201.000000 | 0.799793 | pass |
| randtests::difference.sign.test | 1.055775 | 0.291071 | pass |
| randtests::turning.point.test | 0.126219 | 0.899559 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 2.036179 | 0.041732 | pass |
| randtoolbox::freq.test (16 bins) | 13.623661 | 0.554241 | pass |
| randtoolbox::gap.test [0,0.5) | 21.346721 | 0.499408 | pass |
| randtoolbox::serial.test (d=8) | 56.003686 | 0.721724 | pass |
| randtoolbox::poker.test (5-hand) | 1.311622 | 0.859401 | pass |
| randtoolbox::order.test (d=4) | 22.184819 | 0.509119 | pass |
| stats::ks.test vs U(0,1) | 0.000278 | 0.832866 | pass |
| stats::chisq.test (256 bins) | 242.923622 | 0.696300 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 26.664038 | 0.372874 | pass |
| tseries::runs.test (binary) | -0.458841 | 0.646348 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300305.759036 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50002762 | 0.50000000 | 2.76e-05 |
| 2 | 0.33337517 | 0.33333333 | 4.18e-05 |
| 3 | 0.25006038 | 0.25000000 | 6.04e-05 |
| 4 | 0.20007183 | 0.20000000 | 7.18e-05 |
| 5 | 0.16674406 | 0.16666667 | 7.74e-05 |
| 6 | 0.14293646 | 0.14285714 | 7.93e-05 |
| 7 | 0.12507913 | 0.12500000 | 7.91e-05 |
| 8 | 0.11118885 | 0.11111111 | 7.77e-05 |
| 9 | 0.10007570 | 0.10000000 | 7.57e-05 |
| 10 | 0.09098241 | 0.09090909 | 7.33e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 15.642792 |
| Bonferroni p (no spike) | 0.331103 |
| Spectral flatness (Wiener entropy) | 0.561185 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=12.649, p=0.179140 |
| Periodogram KS vs Exp(1) | D=0.000633, p=0.269702 |


## Serpent-128-CTR

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500005  Var = 0.083275  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 1.239676 | 0.215095 | pass |
| randtests::bartels.rank.test | 0.373056 | 0.709106 | pass |
| randtests::cox.stuart.test (trend) | 1249671.000000 | 0.677759 | pass |
| randtests::difference.sign.test | -1.010849 | 0.312089 | pass |
| randtests::turning.point.test | -0.031820 | 0.974616 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.558944 | 0.576200 | pass |
| randtoolbox::freq.test (16 bins) | 25.764410 | 0.040573 | pass |
| randtoolbox::gap.test [0,0.5) | 18.625145 | 0.668347 | pass |
| randtoolbox::serial.test (d=8) | 79.449037 | 0.078890 | pass |
| randtoolbox::poker.test (5-hand) | 2.326305 | 0.675984 | pass |
| randtoolbox::order.test (d=4) | 16.132058 | 0.849557 | pass |
| stats::ks.test vs U(0,1) | 0.000393 | 0.423356 | pass |
| stats::chisq.test (256 bins) | 260.016538 | 0.401226 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 15.389990 | 0.931788 | pass |
| tseries::runs.test (binary) | 1.239676 | 0.215095 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299282.201996 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50000461 | 0.50000000 | 4.61e-06 |
| 2 | 0.33327957 | 0.33333333 | 5.38e-05 |
| 3 | 0.24991308 | 0.25000000 | 8.69e-05 |
| 4 | 0.19990156 | 0.20000000 | 9.84e-05 |
| 5 | 0.16656933 | 0.16666667 | 9.73e-05 |
| 6 | 0.14276735 | 0.14285714 | 8.98e-05 |
| 7 | 0.12492063 | 0.12500000 | 7.94e-05 |
| 8 | 0.11104313 | 0.11111111 | 6.80e-05 |
| 9 | 0.09994336 | 0.10000000 | 5.66e-05 |
| 10 | 0.09086326 | 0.09090909 | 4.58e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.678954 |
| Bonferroni p (no spike) | 0.651553 |
| Spectral flatness (Wiener entropy) | 0.561506 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=4.080, p=0.906091 |
| Periodogram KS vs Exp(1) | D=0.000468, p=0.644592 |


## SM4-CTR

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499981  Var = 0.083373  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.009839 | 0.992150 | pass |
| randtests::bartels.rank.test | 1.239837 | 0.215036 | pass |
| randtests::cox.stuart.test (trend) | 1250211.000000 | 0.790036 | pass |
| randtests::difference.sign.test | -0.635944 | 0.524813 | pass |
| randtests::turning.point.test | -0.195162 | 0.845267 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.670966 | 0.502242 | pass |
| randtoolbox::freq.test (16 bins) | 20.094214 | 0.168348 | pass |
| randtoolbox::gap.test [0,0.5) | 27.625182 | 0.188438 | pass |
| randtoolbox::serial.test (d=8) | 69.875098 | 0.257705 | pass |
| randtoolbox::poker.test (5-hand) | 8.053566 | 0.089636 | pass |
| randtoolbox::order.test (d=4) | 19.377971 | 0.679082 | pass |
| stats::ks.test vs U(0,1) | 0.000313 | 0.711827 | pass |
| stats::chisq.test (256 bins) | 244.741120 | 0.666724 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 22.377438 | 0.613867 | pass |
| tseries::runs.test (binary) | 0.009839 | 0.992150 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300141.394805 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49998111 | 0.50000000 | 1.89e-05 |
| 2 | 0.33335365 | 0.33333333 | 2.03e-05 |
| 3 | 0.25002864 | 0.25000000 | 2.86e-05 |
| 4 | 0.20002715 | 0.20000000 | 2.72e-05 |
| 5 | 0.16669029 | 0.16666667 | 2.36e-05 |
| 6 | 0.14287743 | 0.14285714 | 2.03e-05 |
| 7 | 0.12501755 | 0.12500000 | 1.75e-05 |
| 8 | 0.11112641 | 0.11111111 | 1.53e-05 |
| 9 | 0.10001335 | 0.10000000 | 1.34e-05 |
| 10 | 0.09092063 | 0.09090909 | 1.15e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 16.148127 |
| Bonferroni p (no spike) | 0.215418 |
| Spectral flatness (Wiener entropy) | 0.561292 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=3.247, p=0.953699 |
| Periodogram KS vs Exp(1) | D=0.000304, p=0.974640 |


## Grasshopper-256-CTR

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499761  Var = 0.083349  Min = 0.000001  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -1.134134 | 0.256738 | pass |
| randtests::bartels.rank.test | -0.527803 | 0.597636 | pass |
| randtests::cox.stuart.test (trend) | 1249047.000000 | 0.228270 | pass |
| randtests::difference.sign.test | -0.044152 | 0.964783 | pass |
| randtests::turning.point.test | 0.670337 | 0.502643 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.806558 | 0.419921 | pass |
| randtoolbox::freq.test (16 bins) | 15.731994 | 0.400084 | pass |
| randtoolbox::gap.test [0,0.5) | 26.328411 | 0.237849 | pass |
| randtoolbox::serial.test (d=8) | 56.903168 | 0.692097 | pass |
| randtoolbox::poker.test (5-hand) | 3.233068 | 0.519606 | pass |
| randtoolbox::order.test (d=4) | 23.176730 | 0.450491 | pass |
| stats::ks.test vs U(0,1) | 0.000567 | 0.080392 | pass |
| stats::chisq.test (256 bins) | 244.187546 | 0.675838 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 13.749361 | 0.965927 | pass |
| tseries::runs.test (binary) | -1.134134 | 0.256738 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299994.377433 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49976073 | 0.50000000 | 2.39e-04 |
| 2 | 0.33310945 | 0.33333333 | 2.24e-04 |
| 3 | 0.24980342 | 0.25000000 | 1.97e-04 |
| 4 | 0.19982773 | 0.20000000 | 1.72e-04 |
| 5 | 0.16651415 | 0.16666667 | 1.53e-04 |
| 6 | 0.14272030 | 0.14285714 | 1.37e-04 |
| 7 | 0.12487555 | 0.12500000 | 1.24e-04 |
| 8 | 0.11099654 | 0.11111111 | 1.15e-04 |
| 9 | 0.09989340 | 0.10000000 | 1.07e-04 |
| 10 | 0.09080903 | 0.09090909 | 1.00e-04 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.849043 |
| Bonferroni p (no spike) | 0.589086 |
| Spectral flatness (Wiener entropy) | 0.561329 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=7.397, p=0.595855 |
| Periodogram KS vs Exp(1) | D=0.000364, p=0.895220 |


## CAST-128-CTR

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500132  Var = 0.083333  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.076026 | 0.939398 | pass |
| randtests::bartels.rank.test | -0.007941 | 0.993664 | pass |
| randtests::cox.stuart.test (trend) | 1250262.000000 | 0.740815 | pass |
| randtests::difference.sign.test | 0.185129 | 0.853128 | pass |
| randtests::turning.point.test | -2.594375 | 0.009476 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.381543 | 0.702800 | pass |
| randtoolbox::freq.test (16 bins) | 18.968538 | 0.215163 | pass |
| randtoolbox::gap.test [0,0.5) | 25.090771 | 0.292754 | pass |
| randtoolbox::serial.test (d=8) | 68.208230 | 0.304738 | pass |
| randtoolbox::poker.test (5-hand) | 0.745677 | 0.945579 | pass |
| randtoolbox::order.test (d=4) | 25.097075 | 0.345300 | pass |
| stats::ks.test vs U(0,1) | 0.000477 | 0.204550 | pass |
| stats::chisq.test (256 bins) | 275.950490 | 0.175467 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 17.962355 | 0.843985 | pass |
| tseries::runs.test (binary) | -0.076026 | 0.939398 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300015.753411 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50013248 | 0.50000000 | 1.32e-04 |
| 2 | 0.33346595 | 0.33333333 | 1.33e-04 |
| 3 | 0.25011967 | 0.25000000 | 1.20e-04 |
| 4 | 0.20010648 | 0.20000000 | 1.06e-04 |
| 5 | 0.16676167 | 0.16666667 | 9.50e-05 |
| 6 | 0.14294262 | 0.14285714 | 8.55e-05 |
| 7 | 0.12507767 | 0.12500000 | 7.77e-05 |
| 8 | 0.11118236 | 0.11111111 | 7.12e-05 |
| 9 | 0.10006592 | 0.10000000 | 6.59e-05 |
| 10 | 0.09097056 | 0.09090909 | 6.15e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 16.187777 |
| Bonferroni p (no spike) | 0.207983 |
| Spectral flatness (Wiener entropy) | 0.561740 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=8.725, p=0.463015 |
| Periodogram KS vs Exp(1) | D=0.000518, p=0.513577 |


## SEED-CTR

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500156  Var = 0.083313  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.219135 | 0.826545 | pass |
| randtests::bartels.rank.test | -0.025798 | 0.979418 | pass |
| randtests::cox.stuart.test (trend) | 1250105.000000 | 0.894839 | pass |
| randtests::difference.sign.test | 1.371811 | 0.170122 | pass |
| randtests::turning.point.test | -0.290621 | 0.771341 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.756689 | 0.449236 | pass |
| randtoolbox::freq.test (16 bins) | 23.576640 | 0.072641 | pass |
| randtoolbox::gap.test [0,0.5) | 23.499670 | 0.373991 | pass |
| randtoolbox::serial.test (d=8) | 72.282573 | 0.198149 | pass |
| randtoolbox::poker.test (5-hand) | 0.580089 | 0.965250 | pass |
| randtoolbox::order.test (d=4) | 16.019546 | 0.854431 | pass |
| stats::ks.test vs U(0,1) | 0.000441 | 0.284677 | pass |
| stats::chisq.test (256 bins) | 219.405619 | 0.948048 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 32.740094 | 0.137695 | pass |
| tseries::runs.test (binary) | -0.219135 | 0.826545 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299944.112313 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50015555 | 0.50000000 | 1.56e-04 |
| 2 | 0.33346888 | 0.33333333 | 1.36e-04 |
| 3 | 0.25010725 | 0.25000000 | 1.07e-04 |
| 4 | 0.20008373 | 0.20000000 | 8.37e-05 |
| 5 | 0.16673119 | 0.16666667 | 6.45e-05 |
| 6 | 0.14290565 | 0.14285714 | 4.85e-05 |
| 7 | 0.12503498 | 0.12500000 | 3.50e-05 |
| 8 | 0.11113461 | 0.11111111 | 2.35e-05 |
| 9 | 0.10001378 | 0.10000000 | 1.38e-05 |
| 10 | 0.09091468 | 0.09090909 | 5.59e-06 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 17.874360 |
| Bonferroni p (no spike) | 0.042254 |
| Spectral flatness (Wiener entropy) | 0.561494 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=1.524, p=0.996967 |
| Periodogram KS vs Exp(1) | D=0.000374, p=0.876032 |


## Rabbit

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500025  Var = 0.083313  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -1.982945 | 0.047374 | pass |
| randtests::bartels.rank.test | -1.304614 | 0.192024 | pass |
| randtests::cox.stuart.test (trend) | 1248952.000000 | 0.185173 | pass |
| randtests::difference.sign.test | -1.269564 | 0.204240 | pass |
| randtests::turning.point.test | -1.618568 | 0.105540 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.624949 | 0.532004 | pass |
| randtoolbox::freq.test (16 bins) | 16.572653 | 0.345046 | pass |
| randtoolbox::gap.test [0,0.5) | 28.593684 | 0.156831 | pass |
| randtoolbox::serial.test (d=8) | 74.466611 | 0.152921 | pass |
| randtoolbox::poker.test (5-hand) | 1.860362 | 0.761422 | pass |
| randtoolbox::order.test (d=4) | 29.231757 | 0.172748 | pass |
| stats::ks.test vs U(0,1) | 0.000329 | 0.651735 | pass |
| stats::chisq.test (256 bins) | 249.888154 | 0.578599 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 45.445087 | 0.007437 | pass |
| tseries::runs.test (binary) | -1.982945 | 0.047374 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300180.616027 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50002543 | 0.50000000 | 2.54e-05 |
| 2 | 0.33333793 | 0.33333333 | 4.60e-06 |
| 3 | 0.24997478 | 0.25000000 | 2.52e-05 |
| 4 | 0.19994663 | 0.20000000 | 5.34e-05 |
| 5 | 0.16658866 | 0.16666667 | 7.80e-05 |
| 6 | 0.14275822 | 0.14285714 | 9.89e-05 |
| 7 | 0.12488363 | 0.12500000 | 1.16e-04 |
| 8 | 0.11098039 | 0.11111111 | 1.31e-04 |
| 9 | 0.09985760 | 0.10000000 | 1.42e-04 |
| 10 | 0.09075729 | 0.09090909 | 1.52e-04 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 13.632739 |
| Bonferroni p (no spike) | 0.950280 |
| Spectral flatness (Wiener entropy) | 0.560887 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=4.005, p=0.911074 |
| Periodogram KS vs Exp(1) | D=0.000554, p=0.426567 |


## Salsa20

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500165  Var = 0.083325  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.606422 | 0.544235 | pass |
| randtests::bartels.rank.test | 1.384795 | 0.166115 | pass |
| randtests::cox.stuart.test (trend) | 1249320.000000 | 0.390061 | pass |
| randtests::difference.sign.test | -0.168087 | 0.866514 | pass |
| randtests::turning.point.test | 0.028638 | 0.977153 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 1.402916 | 0.160642 | pass |
| randtoolbox::freq.test (16 bins) | 10.068173 | 0.815430 | pass |
| randtoolbox::gap.test [0,0.5) | 35.352580 | 0.035580 | pass |
| randtoolbox::serial.test (d=8) | 64.811469 | 0.413266 | pass |
| randtoolbox::poker.test (5-hand) | 9.140771 | 0.057676 | pass |
| randtoolbox::order.test (d=4) | 33.395661 | 0.074416 | pass |
| stats::ks.test vs U(0,1) | 0.000427 | 0.321638 | pass |
| stats::chisq.test (256 bins) | 227.471462 | 0.891778 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 18.539840 | 0.818634 | pass |
| tseries::runs.test (binary) | 0.606422 | 0.544235 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299824.024334 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50016525 | 0.50000000 | 1.65e-04 |
| 2 | 0.33348984 | 0.33333333 | 1.57e-04 |
| 3 | 0.25013622 | 0.25000000 | 1.36e-04 |
| 4 | 0.20012011 | 0.20000000 | 1.20e-04 |
| 5 | 0.16677516 | 0.16666667 | 1.08e-04 |
| 6 | 0.14295711 | 0.14285714 | 1.00e-04 |
| 7 | 0.12509340 | 0.12500000 | 9.34e-05 |
| 8 | 0.11119920 | 0.11111111 | 8.81e-05 |
| 9 | 0.10008357 | 0.10000000 | 8.36e-05 |
| 10 | 0.09098868 | 0.09090909 | 7.96e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 15.055940 |
| Bonferroni p (no spike) | 0.514779 |
| Spectral flatness (Wiener entropy) | 0.561765 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=6.740, p=0.664212 |
| Periodogram KS vs Exp(1) | D=0.000459, p=0.667485 |


## Snow3G

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500166  Var = 0.083288  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.779941 | 0.435426 | pass |
| randtests::bartels.rank.test | -0.953933 | 0.340118 | pass |
| randtests::cox.stuart.test (trend) | 1249726.000000 | 0.729378 | pass |
| randtests::difference.sign.test | 1.249424 | 0.211510 | pass |
| randtests::turning.point.test | -0.079550 | 0.936596 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.352120 | 0.724748 | pass |
| randtoolbox::freq.test (16 bins) | 19.513382 | 0.191405 | pass |
| randtoolbox::gap.test [0,0.5) | 22.482734 | 0.431411 | pass |
| randtoolbox::serial.test (d=8) | 71.870413 | 0.207623 | pass |
| randtoolbox::poker.test (5-hand) | 1.240256 | 0.871427 | pass |
| randtoolbox::order.test (d=4) | 18.069645 | 0.753701 | pass |
| stats::ks.test vs U(0,1) | 0.000455 | 0.250773 | pass |
| stats::chisq.test (256 bins) | 258.613453 | 0.425145 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 26.373742 | 0.387875 | pass |
| tseries::runs.test (binary) | -0.779941 | 0.435426 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299956.929479 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50016628 | 0.50000000 | 1.66e-04 |
| 2 | 0.33345446 | 0.33333333 | 1.21e-04 |
| 3 | 0.25008636 | 0.25000000 | 8.64e-05 |
| 4 | 0.20006123 | 0.20000000 | 6.12e-05 |
| 5 | 0.16670886 | 0.16666667 | 4.22e-05 |
| 6 | 0.14288446 | 0.14285714 | 2.73e-05 |
| 7 | 0.12501554 | 0.12500000 | 1.55e-05 |
| 8 | 0.11111733 | 0.11111111 | 6.22e-06 |
| 9 | 0.09999888 | 0.10000000 | 1.12e-06 |
| 10 | 0.09090225 | 0.09090909 | 6.84e-06 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.511800 |
| Bonferroni p (no spike) | 0.712369 |
| Spectral flatness (Wiener entropy) | 0.561458 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=2.745, p=0.973551 |
| Periodogram KS vs Exp(1) | D=0.000412, p=0.788734 |


## ZUC-128

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500085  Var = 0.083371  Min = 0.000001  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -1.505321 | 0.132242 | pass |
| randtests::bartels.rank.test | -1.062296 | 0.288101 | pass |
| randtests::cox.stuart.test (trend) | 1250457.000000 | 0.563648 | pass |
| randtests::difference.sign.test | 0.189776 | 0.849485 | pass |
| randtests::turning.point.test | -0.275772 | 0.782723 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.266568 | 0.789802 | pass |
| randtoolbox::freq.test (16 bins) | 16.853696 | 0.327685 | pass |
| randtoolbox::gap.test [0,0.5) | 30.399982 | 0.138260 | pass |
| randtoolbox::serial.test (d=8) | 74.661427 | 0.149288 | pass |
| randtoolbox::poker.test (5-hand) | 4.767135 | 0.312036 | pass |
| randtoolbox::order.test (d=4) | 17.954906 | 0.759931 | pass |
| stats::ks.test vs U(0,1) | 0.000386 | 0.445676 | pass |
| stats::chisq.test (256 bins) | 265.552794 | 0.311920 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 46.943371 | 0.004979 | pass |
| tseries::runs.test (binary) | -1.505321 | 0.132242 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300263.045780 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50008491 | 0.50000000 | 8.49e-05 |
| 2 | 0.33345588 | 0.33333333 | 1.23e-04 |
| 3 | 0.25014669 | 0.25000000 | 1.47e-04 |
| 4 | 0.20015966 | 0.20000000 | 1.60e-04 |
| 5 | 0.16683183 | 0.16666667 | 1.65e-04 |
| 6 | 0.14302328 | 0.14285714 | 1.66e-04 |
| 7 | 0.12516451 | 0.12500000 | 1.65e-04 |
| 8 | 0.11127259 | 0.11111111 | 1.61e-04 |
| 9 | 0.10015772 | 0.10000000 | 1.58e-04 |
| 10 | 0.09106273 | 0.09090909 | 1.54e-04 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 13.698608 |
| Bonferroni p (no spike) | 0.939797 |
| Spectral flatness (Wiener entropy) | 0.561397 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=17.190, p=0.045828 |
| Periodogram KS vs Exp(1) | D=0.000513, p=0.525122 |


## ChaCha20

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499920  Var = 0.083257  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.240601 | 0.809864 | pass |
| randtests::bartels.rank.test | -0.337737 | 0.735561 | pass |
| randtests::cox.stuart.test (trend) | 1251533.000000 | 0.052565 | pass |
| randtests::difference.sign.test | -0.149497 | 0.881161 | pass |
| randtests::turning.point.test | -1.129603 | 0.258643 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.620639 | 0.534837 | pass |
| randtoolbox::freq.test (16 bins) | 17.390419 | 0.296066 | pass |
| randtoolbox::gap.test [0,0.5) | 14.872075 | 0.867669 | pass |
| randtoolbox::serial.test (d=8) | 85.831475 | 0.029529 | pass |
| randtoolbox::poker.test (5-hand) | 2.547099 | 0.636222 | pass |
| randtoolbox::order.test (d=4) | 26.977485 | 0.256927 | pass |
| stats::ks.test vs U(0,1) | 0.000462 | 0.235597 | pass |
| stats::chisq.test (256 bins) | 245.691597 | 0.650878 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 24.383778 | 0.497297 | pass |
| tseries::runs.test (binary) | 0.240601 | 0.809864 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299525.269662 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49992008 | 0.50000000 | 7.99e-05 |
| 2 | 0.33317741 | 0.33333333 | 1.56e-04 |
| 3 | 0.24981416 | 0.25000000 | 1.86e-04 |
| 4 | 0.19980605 | 0.20000000 | 1.94e-04 |
| 5 | 0.16647414 | 0.16666667 | 1.93e-04 |
| 6 | 0.14267000 | 0.14285714 | 1.87e-04 |
| 7 | 0.12481960 | 0.12500000 | 1.80e-04 |
| 8 | 0.11093764 | 0.11111111 | 1.73e-04 |
| 9 | 0.09983309 | 0.10000000 | 1.67e-04 |
| 10 | 0.09074820 | 0.09090909 | 1.61e-04 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.483454 |
| Bonferroni p (no spike) | 0.722492 |
| Spectral flatness (Wiener entropy) | 0.561987 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=6.360, p=0.703467 |
| Periodogram KS vs Exp(1) | D=0.000520, p=0.508653 |


## SpongeBob (SHA3-512)

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499811  Var = 0.083303  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.360454 | 0.718508 | pass |
| randtests::bartels.rank.test | -0.405758 | 0.684920 | pass |
| randtests::cox.stuart.test (trend) | 1250476.000000 | 0.547530 | pass |
| randtests::difference.sign.test | 2.098382 | 0.035871 | pass |
| randtests::turning.point.test | 0.082732 | 0.934065 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.541992 | 0.587824 | pass |
| randtoolbox::freq.test (16 bins) | 17.286618 | 0.302021 | pass |
| randtoolbox::gap.test [0,0.5) | 894.793186 | 1.356e-169 | REJECT |
| randtoolbox::serial.test (d=8) | 46.256333 | 0.943876 | pass |
| randtoolbox::poker.test (5-hand) | 1.506687 | 0.825456 | pass |
| randtoolbox::order.test (d=4) | 20.127846 | 0.634188 | pass |
| stats::ks.test vs U(0,1) | 0.000434 | 0.304193 | pass |
| stats::chisq.test (256 bins) | 280.245146 | 0.133012 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 21.553570 | 0.661373 | pass |
| tseries::runs.test (binary) | -0.360454 | 0.718508 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299792.859779 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49981057 | 0.50000000 | 1.89e-04 |
| 2 | 0.33311377 | 0.33333333 | 2.20e-04 |
| 3 | 0.24978156 | 0.25000000 | 2.18e-04 |
| 4 | 0.19979158 | 0.20000000 | 2.08e-04 |
| 5 | 0.16647065 | 0.16666667 | 1.96e-04 |
| 6 | 0.14267364 | 0.14285714 | 1.84e-04 |
| 7 | 0.12482833 | 0.12500000 | 1.72e-04 |
| 8 | 0.11095037 | 0.11111111 | 1.61e-04 |
| 9 | 0.09984928 | 0.10000000 | 1.51e-04 |
| 10 | 0.09076756 | 0.09090909 | 1.42e-04 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 16.653349 |
| Bonferroni p (no spike) | 0.136171 |
| Spectral flatness (Wiener entropy) | 0.561679 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=7.749, p=0.559646 |
| Periodogram KS vs Exp(1) | D=0.000296, p=0.980912 |


## Squidward (SHA-256)

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500039  Var = 0.083353  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.967770 | 0.333159 | pass |
| randtests::bartels.rank.test | 1.325694 | 0.184941 | pass |
| randtests::cox.stuart.test (trend) | 1248933.000000 | 0.177328 | pass |
| randtests::difference.sign.test | -0.541443 | 0.588202 | pass |
| randtests::turning.point.test | 0.098641 | 0.921423 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.196914 | 0.843895 | pass |
| randtoolbox::freq.test (16 bins) | 13.381562 | 0.572852 | pass |
| randtoolbox::gap.test [0,0.5) | 79.339658 | 1.448e-07 | REJECT |
| randtoolbox::serial.test (d=8) | 77.430323 | 0.104356 | pass |
| randtoolbox::poker.test (5-hand) | 1.266256 | 0.867074 | pass |
| randtoolbox::order.test (d=4) | 26.235021 | 0.289897 | pass |
| stats::ks.test vs U(0,1) | 0.000249 | 0.915683 | pass |
| stats::chisq.test (256 bins) | 253.914112 | 0.507426 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 25.821121 | 0.417193 | pass |
| tseries::runs.test (binary) | 0.967770 | 0.333159 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300312.654700 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50003875 | 0.50000000 | 3.88e-05 |
| 2 | 0.33339205 | 0.33333333 | 5.87e-05 |
| 3 | 0.25006278 | 0.25000000 | 6.28e-05 |
| 4 | 0.20005851 | 0.20000000 | 5.85e-05 |
| 5 | 0.16671790 | 0.16666667 | 5.12e-05 |
| 6 | 0.14290050 | 0.14285714 | 4.34e-05 |
| 7 | 0.12503583 | 0.12500000 | 3.58e-05 |
| 8 | 0.11114005 | 0.11111111 | 2.89e-05 |
| 9 | 0.10002273 | 0.10000000 | 2.27e-05 |
| 10 | 0.09092624 | 0.09090909 | 1.71e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 16.680687 |
| Bonferroni p (no spike) | 0.132754 |
| Spectral flatness (Wiener entropy) | 0.561628 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=6.854, p=0.652326 |
| Periodogram KS vs Exp(1) | D=0.000557, p=0.420888 |


## HmacDrbg

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500152  Var = 0.083334  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.432008 | 0.665735 | pass |
| randtests::bartels.rank.test | 0.881983 | 0.377786 | pass |
| randtests::cox.stuart.test (trend) | 1250629.000000 | 0.426615 | pass |
| randtests::difference.sign.test | -1.190555 | 0.233828 | pass |
| randtests::turning.point.test | 0.387141 | 0.698652 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.660429 | 0.508978 | pass |
| randtoolbox::freq.test (16 bins) | 15.626726 | 0.407284 | pass |
| randtoolbox::gap.test [0,0.5) | 23.565776 | 0.370395 | pass |
| randtoolbox::serial.test (d=8) | 54.691840 | 0.762797 | pass |
| randtoolbox::poker.test (5-hand) | 4.183442 | 0.381749 | pass |
| randtoolbox::order.test (d=4) | 21.108890 | 0.574438 | pass |
| stats::ks.test vs U(0,1) | 0.000448 | 0.267482 | pass |
| stats::chisq.test (256 bins) | 235.552051 | 0.803551 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 22.874815 | 0.584873 | pass |
| tseries::runs.test (binary) | 0.432008 | 0.665735 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299686.380034 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50015182 | 0.50000000 | 1.52e-04 |
| 2 | 0.33348592 | 0.33333333 | 1.53e-04 |
| 3 | 0.25013116 | 0.25000000 | 1.31e-04 |
| 4 | 0.20011394 | 0.20000000 | 1.14e-04 |
| 5 | 0.16676884 | 0.16666667 | 1.02e-04 |
| 6 | 0.14295128 | 0.14285714 | 9.41e-05 |
| 7 | 0.12508841 | 0.12500000 | 8.84e-05 |
| 8 | 0.11119521 | 0.11111111 | 8.41e-05 |
| 9 | 0.10008066 | 0.10000000 | 8.07e-05 |
| 10 | 0.09098686 | 0.09090909 | 7.78e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.421140 |
| Bonferroni p (no spike) | 0.744447 |
| Spectral flatness (Wiener entropy) | 0.561916 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=5.433, p=0.795068 |
| Periodogram KS vs Exp(1) | D=0.000501, p=0.556371 |


## HashDrbg

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499820  Var = 0.083277  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.317522 | 0.750848 | pass |
| randtests::bartels.rank.test | -0.003281 | 0.997382 | pass |
| randtests::cox.stuart.test (trend) | 1249881.000000 | 0.880850 | pass |
| randtests::difference.sign.test | -1.096054 | 0.273055 | pass |
| randtests::turning.point.test | 0.845346 | 0.397918 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.731915 | 0.464220 | pass |
| randtoolbox::freq.test (16 bins) | 25.148550 | 0.047984 | pass |
| randtoolbox::gap.test [0,0.5) | 27.693083 | 0.186077 | pass |
| randtoolbox::serial.test (d=8) | 66.303437 | 0.363739 | pass |
| randtoolbox::poker.test (5-hand) | 1.766544 | 0.778597 | pass |
| randtoolbox::order.test (d=4) | 33.426189 | 0.073923 | pass |
| stats::ks.test vs U(0,1) | 0.000546 | 0.101631 | pass |
| stats::chisq.test (256 bins) | 238.186803 | 0.767864 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 29.470845 | 0.244714 | pass |
| tseries::runs.test (binary) | 0.317522 | 0.750848 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299735.326152 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49982012 | 0.50000000 | 1.80e-04 |
| 2 | 0.33309764 | 0.33333333 | 2.36e-04 |
| 3 | 0.24974919 | 0.25000000 | 2.51e-04 |
| 4 | 0.19974893 | 0.20000000 | 2.51e-04 |
| 5 | 0.16642104 | 0.16666667 | 2.46e-04 |
| 6 | 0.14261908 | 0.14285714 | 2.38e-04 |
| 7 | 0.12477012 | 0.12500000 | 2.30e-04 |
| 8 | 0.11088936 | 0.11111111 | 2.22e-04 |
| 9 | 0.09978605 | 0.10000000 | 2.14e-04 |
| 10 | 0.09070251 | 0.09090909 | 2.07e-04 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 15.308680 |
| Bonferroni p (no spike) | 0.429732 |
| Spectral flatness (Wiener entropy) | 0.561545 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=5.745, p=0.765185 |
| Periodogram KS vs Exp(1) | D=0.000500, p=0.559323 |


## CtrDrbgAes256

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500221  Var = 0.083312  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.108226 | 0.913817 | pass |
| randtests::bartels.rank.test | 0.451626 | 0.651538 | pass |
| randtests::cox.stuart.test (trend) | 1250599.000000 | 0.449020 | pass |
| randtests::difference.sign.test | 1.412090 | 0.157924 | pass |
| randtests::turning.point.test | 0.301228 | 0.763241 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -0.228610 | 0.819172 | pass |
| randtoolbox::freq.test (16 bins) | 17.509939 | 0.289305 | pass |
| randtoolbox::gap.test [0,0.5) | 30.006292 | 0.118312 | pass |
| randtoolbox::serial.test (d=8) | 52.512358 | 0.824218 | pass |
| randtoolbox::poker.test (5-hand) | 4.240346 | 0.374455 | pass |
| randtoolbox::order.test (d=4) | 14.087795 | 0.924330 | pass |
| stats::ks.test vs U(0,1) | 0.000537 | 0.111627 | pass |
| stats::chisq.test (256 bins) | 255.551283 | 0.478502 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 16.641980 | 0.894367 | pass |
| tseries::runs.test (binary) | -0.108226 | 0.913817 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299924.243228 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50022140 | 0.50000000 | 2.21e-04 |
| 2 | 0.33353365 | 0.33333333 | 2.00e-04 |
| 3 | 0.25016518 | 0.25000000 | 1.65e-04 |
| 4 | 0.20013531 | 0.20000000 | 1.35e-04 |
| 5 | 0.16677889 | 0.16666667 | 1.12e-04 |
| 6 | 0.14295189 | 0.14285714 | 9.47e-05 |
| 7 | 0.12508143 | 0.12500000 | 8.14e-05 |
| 8 | 0.11118221 | 0.11111111 | 7.11e-05 |
| 9 | 0.10006291 | 0.10000000 | 6.29e-05 |
| 10 | 0.09096533 | 0.09090909 | 5.62e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.537369 |
| Bonferroni p (no spike) | 0.703177 |
| Spectral flatness (Wiener entropy) | 0.561456 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=1.683, p=0.995552 |
| Periodogram KS vs Exp(1) | D=0.000315, p=0.964824 |


## Dual_EC_DRBG (P-256)

Sample size: 1,000,000 u32 words (3.81 MB)

Mean = 0.499811  Var = 0.083391  Min = 0.000002  Max = 0.999998

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.096000 | 0.923521 | pass |
| randtests::bartels.rank.test | -0.178453 | 0.858367 | pass |
| randtests::cox.stuart.test (trend) | 250360.000000 | 0.309239 | pass |
| randtests::difference.sign.test | -0.001732 | 0.998618 | pass |
| randtests::turning.point.test | -0.510708 | 0.609555 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -1.310084 | 0.190168 | pass |
| randtoolbox::freq.test (16 bins) | 20.112896 | 0.167645 | pass |
| randtoolbox::gap.test [0,0.5) | 26.120751 | 0.161861 | pass |
| randtoolbox::serial.test (d=8) | 51.052288 | 0.859892 | pass |
| randtoolbox::poker.test (5-hand) | 4.335159 | 0.362543 | pass |
| randtoolbox::order.test (d=4) | 21.255296 | 0.565500 | pass |
| stats::ks.test vs U(0,1) | 0.000781 | 0.575387 | pass |
| stats::chisq.test (256 bins) | 236.025344 | 0.797378 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 22.491778 | 0.607214 | pass |
| tseries::runs.test (binary) | -0.096000 | 0.923521 | pass |
| tseries::jarque.bera.test (vs Normal*) | 60176.554190 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49981059 | 0.50000000 | 1.89e-04 |
| 2 | 0.33320192 | 0.33333333 | 1.31e-04 |
| 3 | 0.24991764 | 0.25000000 | 8.24e-05 |
| 4 | 0.19994284 | 0.20000000 | 5.72e-05 |
| 5 | 0.16662189 | 0.16666667 | 4.48e-05 |
| 6 | 0.14281928 | 0.14285714 | 3.79e-05 |
| 7 | 0.12496702 | 0.12500000 | 3.30e-05 |
| 8 | 0.11108238 | 0.11111111 | 2.87e-05 |
| 9 | 0.09997533 | 0.10000000 | 2.47e-05 |
| 10 | 0.09088838 | 0.09090909 | 2.07e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 5e+05 |
| max normalized periodogram (P_max) | 14.030933 |
| Bonferroni p (no spike) | 0.331755 |
| Spectral flatness (Wiener entropy) | 0.560026 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=13.749, p=0.131536 |
| Periodogram KS vs Exp(1) | D=0.001407, p=0.275327 |



---

## Analysis

The summary table at the top of this file collapses each generator into a
single REJECT count.  This section unpacks that — what each test is
measuring, why the bare REJECT count is misleading on its own, what the
moments and the spectrum reveal that the verdict-style table cannot, and a
generator-by-generator account of why each failure happens.

### A. Theoretical baselines

For an i.i.d. uniform sample U₁, …, U_N ~ U(0,1):

* **Raw moments**: m_k := E[U^k] = ∫₀¹ uᵏ du = 1/(k+1).
* **Variance of the empirical k-th moment**:
  Var(m̂_k) = (m_{2k} − m_k²) / N = (1/(2k+1) − 1/(k+1)²) / N.
  At N = 5 × 10⁶ that is

  | k  | SE(m̂_k) ≈ |
  |----|-----------|
  | 1  | 1.29 × 10⁻⁴ |
  | 2  | 1.49 × 10⁻⁴ |
  | 3  | 1.62 × 10⁻⁴ |
  | 5  | 1.78 × 10⁻⁴ |
  | 10 | 1.85 × 10⁻⁴ |

  A clean RNG should sit inside ≈ 2 × 10⁻⁴ for every order at this sample
  size.  Anything an order of magnitude larger is a real distributional
  defect, not sampling noise.

* **Periodogram of y_t = u_t − ½**: Bartlett (1955) / Brockwell & Davis
  (1991) give that, under H₀, at the m = N/2 − 1 inner Fourier
  frequencies f_k = k/N,

  P̃_k := |Y(f_k)|² / (N · σ²),  σ² = 1/12,

  is iid Exp(1), and P_max := maxₖ P̃_k has CDF
  Pr[P_max ≤ x] = (1 − e^(−x))^m.
  At m = 2 499 999 the 99.9 % point is ≈ −log(1 − 0.999^(1/m)) ≈
  log(m / 0.001) ≈ 21.6, so anything past P_max ≳ 25 is a flag.

* **Spectral flatness / Wiener entropy**: ξ := geomean(P_k) /
  arithmean(P_k).  For iid Exp(λ) the population ratio is
  ξ_∞ = exp(E[log P] − log E[P]) = exp(−γ) ≈ 0.561459 (Euler–Mascheroni
  γ).  Pure tones drop ξ toward 0; pure white noise sits at e^(−γ).

* **Variance check via the periodogram chi² / KS**: my normalization
  divides by σ² = 1/12.  If the true variance differs (e.g. an LCG
  truncated into [0, ½) has variance 1/48), the per-bin distribution is
  Exp(1/4) instead of Exp(1) and the chi²/KS-vs-Exp(1) tests still
  REJECT — they detect the **variance mismatch** rather than the
  spectral colour.  Combine the chi² with the flatness ξ to disentangle
  the two: ξ ≈ e^(−γ) with chi² rejection means white noise of the
  wrong variance; ξ depressed means actual coloured spectrum.

### B. What R's stock RNG tests can and cannot see

R's `randtests`, `randtoolbox`, and `tseries` between them probe:

* one-dimensional distribution: KS, χ²(256), `freq.test`, `gap.test`,
  `poker.test`, `order.test`,
* low-order tuple distribution: `serial.test` d=8,
* one- and low-lag autocorrelation: `Box.test` Ljung–Box,
  `bartels.rank.test`, `randtests::runs.test`, `tseries::runs.test`,
* monotone trend: `cox.stuart.test`, `difference.sign.test`,
  `turning.point.test`, `rank.test` (Mann–Kendall, subsampled).

What R does **not** ship that NIST SP 800-22, DIEHARD, and TestU01
expose:

* binary-matrix-rank tests — caught MT19937's F₂-linear structure
* spectral test on bit windows (Hellekalek–Wegenkittl 2003)
* linear-complexity / Berlekamp–Massey on the bitstream
* birthday-spacings (Marsaglia 1985)
* overlapping-template / non-overlapping-template
* Marsaglia's spectral test on consecutive d-tuples (Marsaglia 1968) —
  the canonical LCG-killer

The spectral block I added catches part of the LCG-spectral failure
mode, but not the d-tuple lattice diagnostic.

This explains the headline finding of the summary table: **at α = 0.001,
many historically-broken generators look clean.**  rand48, BSD random(),
glibc random(), FreeBSD `rand_r()`, and Windows .NET Random all clear
all 15 sample-domain tests because their failures live in the bit-rank
or d-dimensional lattice geometry that R has no test for.

### C. Per-RNG failure analysis

#### `ConstantRng` — 9 REJECT, 4 n/a, 2 pass

Every output equals 0xDEAD_DEAD ⇒ u ≡ 0.870.  All N samples land in one
χ² bin: χ² = 1.27 × 10⁸, KS D = 0.870, freq/serial/poker/order/gap all
saturate.  The four n/a tests (`runs.test`, `turning.point.test`,
`Box.test`, `tseries::jarque.bera.test`) all need nonzero variance.
Spectral block: P_max = 0, ξ = 0 — the entire signal is at f = 0
(removed bin), so the AC spectrum is identically zero.  Two passes:
`cox.stuart.test` (which compares first-half vs second-half medians and
sees zero difference) and `difference.sign.test` (zero positive
differences and zero negative differences cancel).  Those passes are
artefacts, not endorsements.

Moments are exactly cᵏ where c = 0.87:
m̂₁ = 0.870, m̂₂ = 0.757, m̂_k = 0.87ᵏ.  Errors against 1/(k+1) grow to
0.42 by k=2 and stay in the 0.15–0.42 band — a constant signature.

#### `CounterRng` — 14 REJECT, 1 n/a

u_t = t / 2³².  At N = 5 × 10⁶ the entire stream lies in [0, 1.16 × 10⁻³].
This is a **range collapse** failure plus a **monotone trend** failure.

* Range collapse: KS D = 0.9999, χ²(256) ≈ 1.275 × 10⁹, freq/serial/
  poker/order all crushed.
* Monotone trend: `cox.stuart.test` returns the maximum 2.5 × 10⁶
  (every later value > every earlier value), `difference.sign.test`
  z = 1224, `turning.point.test` z = −1118 (zero turning points),
  Mann–Kendall z = 106.
* `Box.test` lag-25 ≈ 1.25 × 10⁸: maximal autocorrelation.
* Spectral block: P_max = 2.06 (low!) and ξ = 2 × 10⁻⁶ (nearly zero).
  A monotonic ramp has a 1/f-style power spectrum dominated by low
  frequencies; once bin 0 is removed the residual is small in the peak
  but heavily concentrated in the lowest few bins, giving low ξ — that
  is the right diagnostic.

Moments collapse: m̂_k ≈ (N/2³²)ᵏ / (k+1), so m̂₁ ≈ 5.8 × 10⁻⁴ and m̂_k
underflows the printed precision for k ≥ 2.

#### Bit-truncated LCGs

| RNG               | output bits | reduced support  |
|-------------------|-------------|------------------|
| ANSI C LCG        | 31 raw      | [0, ½)           |
| MINSTD            | 31 raw      | [0, ½)           |
| Borland C++ LCG   | 15-bit (mask 0x7FFF after shift 16) | [0, ¹⁄₁₃₁₀₇₂) |
| MSVC LCG          | 15-bit (mask 0x7FFF after shift 16) | [0, ¹⁄₁₃₁₀₇₂) |

All four: 5 REJECT, 1 n/a (`gap.test [0, ½)` returns NA when every
value is below the upper bound), 9 pass.  The passes — `runs.test`,
`bartels.rank.test`, `cox.stuart.test`, `difference.sign.test`,
`turning.point.test`, `rank.test`, `serial.test`, `order.test`,
`Box.test`, `tseries::runs.test` — are tests of **temporal structure**
that don't care about the support of the marginal distribution.  An
LCG whose output is uniform-modulo-truncation has perfectly clean
autocorrelation at the values level; only tests that look at the
distribution itself catch the truncation.

Spectral block separates the two behaviours:

* **ANSI C / MINSTD** retain reasonable variance (≈ 1/48), so ξ ≈ 0.561
  (white) but the χ²(Exp(1)) and KS(Exp(1)) tests REJECT massively
  because the periodogram lives at variance 1/4 of expected.  Diagnosis:
  variance mismatch from range truncation, not spectral colour.
* **Borland / MSVC LCG** push the support down to [0, ~7.6 × 10⁻⁶);
  variance ≈ 0; the periodogram is essentially zero everywhere; ξ
  computes to ≈ 0.56 (because both geometric and arithmetic means are
  the same tiny number) but the χ²(Exp(1)) and KS(Exp(1)) tests
  saturate at chi² = 22 499 991 — every periodogram value falls into
  the lowest decile of Exp(1).

Moments tell the same story: m̂₁ for ANSI C / MINSTD is ≈ 0.250 (half
of 0.5, as expected for a uniform on [0, ½)); for Borland / MSVC LCG
the mean is ≈ 4 × 10⁻⁶, with all higher moments underflowing.

#### Windows VB6 / VBA `Rnd()` — 6 REJECT, 9 pass

VB6 `Rnd` is built on a 24-bit LCG.  After packing into 32-bit
"uniforms" the 1-D distribution looks reasonable on coarse bins but
stratifies into 2²⁴ = 16 777 216 distinct values.  R catches that
across:

* `gap.test [0, ½)` p ≈ 1.7 × 10⁻¹³⁹
* `poker.test` p < 1 × 10⁻³⁰⁰
* `order.test` p ≈ 0
* KS p ≈ 7.5 × 10⁻⁷⁰
* χ²(256) p ≈ 0
* `Box.test` lag-25 p ≈ 0  (long-range dependence from the 24-bit cycle)

The Fourier block is decisive: **P_max = 49 074** (Bonferroni p ≈ 0,
where the 99.9 % critical value is ≈ 21.6) and ξ = 0.210 — a textbook
**tonal** spectrum from a low-period LCG.

Yet `runs.test`, `bartels.rank.test`, `cox.stuart.test`,
`difference.sign.test`, `turning.point.test`, Mann–Kendall,
`freq.test` (16 bins is too coarse), `serial.test` (d=8 is too coarse),
and `tseries::runs.test` all pass.  The lesson: **value-level
autocorrelation tests do not see periodicity that lives at frequencies
the test bandwidth doesn't probe.**

#### System V `rand()` and Windows MSVC `rand()` — 1 REJECT, 14 pass

These are the most interesting cases in the report, because the
sample-domain table calls them clean and the spectrum exposes them.

Both pack 15-bit-output LCGs into 32-bit "uniforms".  The 1-D
distribution is uniform by construction (the bit-packing whitens the
marginal), so KS, χ²(256), `freq`, `gap`, `serial`, `poker`, `order`,
`Box`, all the `randtests` trend-style tests, and Mann–Kendall pass.
The single value-level REJECT in each case is `randtoolbox::poker.test`
for SystemV (chi² = 23 with df = 4) and `serial.test` for MSVC — both
borderline, but flagged because the bit-packing leaks into 5-card
hands.

Spectrum cuts through:

| RNG            | P_max   | spike p    | ξ (flatness) | χ²(Exp(1)) p | KS-Exp(1) p |
|----------------|---------|------------|--------------|--------------|-------------|
| white-noise H₀ | ≲ 22    | ≳ 0.01     | 0.5615       | ≳ 0.01       | ≳ 0.01      |
| SystemV rand   | 236.9   | 3.4 × 10⁻⁹⁷ | 0.4000       | 0            | 0           |
| MSVC rand      | 266.3   | 5.6 × 10⁻¹¹⁰ | 0.4019       | 0            | 0           |

Both flag a **strongly tonal** spectrum (ξ ≈ 0.40 vs 0.56 white) with a
visible single-bin spike where Marsaglia's spectral test would also
flag the LCG multiplier-induced lattice.  This is exactly the failure
mode Marsaglia (1968) described and the reason why no LCG should be
used as a CSPRNG.

#### Quality non-cryptographic generators (MT19937, PCG, Xorshift, Xoshiro, WyRand, SFC64, JSF64)

All seven generators clear all 15 sample-domain tests, and their
Fourier blocks all sit inside the white-noise envelope (ξ within
±0.001 of 0.561, P_max ≤ 17, chi² and KS p > 0.07).  Moment errors are
≤ 3 × 10⁻⁴ for every k = 1..10 — within the 2 × 10⁻⁴ SE band predicted
above (small over-runs are genuinely within sampling noise).

This does **not** mean MT19937 is cryptographically suitable.  Its
known failure modes — F₂-linear structure exposed by binary-matrix-rank
or linear-complexity tests, predictability of state from 624 outputs —
are simply outside the panel of tests R provides.  See the parent
crate's NIST/DIEHARD/DIEHARDER outputs for those failures.

#### CSPRNGs (8 block-CTR ciphers + 4 stream ciphers + 5 DRBGs + Dual_EC)

All 18 cryptographic generators show 0 or 1 REJECTs in the
sample-domain table — and the 1-REJECT cases (Camellia, SpongeBob,
Squidward, etc.) are spread randomly across different tests, exactly
as expected for 18 generators × 15 tests at α = 0.001 (expected
spurious REJECTs = 0.27, observed = 4 isolated rejections; well within
Poisson noise).

Spectrally: ξ within ±0.001 of e^(−γ) for every CSPRNG.  P_max in the
expected range, χ²(Exp(1)) and KS uniform across the unit interval.

Note `Dual_EC_DRBG (P-256)`, included as a **negative control for
predictability**, passes statistical testing.  Its known weakness is
state recovery from ~30 bytes of output once the discrete-log e with
Q = e·P is known (Bernstein, Lange, Niederhagen 2016) — a security
flaw, not a statistical-distribution flaw.  No black-box battery on
the output stream alone can detect it.  Including it here documents
that fact: passing every test in this report does not certify a CSPRNG
as backdoor-free.

### D. Moment commentary in aggregate

For every clean generator, the absolute moment errors form a smooth
band ≲ 2 × 10⁻⁴ across k = 1..10.  The shape of the deviation curve
across orders is itself a fingerprint:

* **Range truncation** (ANSI C, MINSTD): m̂₁ ≈ ½ · m₁ ≈ 0.25, with
  errors growing geometrically in k.
* **Total range collapse** (Borland LCG, MSVC LCG, CounterRng): m̂_k ≈
  εᵏ → 0, errors approach 1/(k+1) themselves (i.e. observed ≈ 0).
* **Frozen state** (ConstantRng): m̂_k = cᵏ exactly; errors = |cᵏ −
  1/(k+1)| have a hump near k = 2 and decline thereafter.

Because m_{2k} appears in the SE formula, **higher moments are
proportionally less informative** than lower ones at fixed N — a
well-behaved RNG can show a small upward drift in higher-moment errors
that is purely sampling.  The clean RNGs in this report all sit comfortably
within the predicted SE envelope.

### E. Fourier diagnostics in aggregate

The three spectral readouts (P_max + Bonferroni p, ξ flatness, χ²/KS
against Exp(1)) jointly cover four classes of failure:

1. **DC / frozen** — ξ → 0 and the Exp(1) chi² blows up because every
   periodogram value is zero.  ConstantRng, Borland LCG, MSVC LCG.
2. **Trend / 1/f spectrum** — ξ small but P_max not enormous; the
   power lives in the lowest bins.  CounterRng.
3. **Tonal / lattice spectrum** — ξ depressed (≈ 0.2–0.4) with one or
   more massive bins.  VB6 `Rnd()`, SystemV `rand()`, Windows MSVC
   `rand()`.
4. **Variance mismatch but white spectrum** — ξ ≈ 0.561 yet the
   Exp(1) χ²/KS REJECT.  ANSI C LCG, MINSTD.

Together with the moment table, these classify every failing
generator in this report by mechanism, not just by REJECT count.

### F. The R-battery's place in the test hierarchy

Restating the ranking that emerges from the data:

* R's value-level tests cleanly separate **catastrophically broken**
  generators (ConstantRng, CounterRng, range-truncated LCGs, VB6
  `Rnd()`) from **plausibly random** ones.
* They miss **subtler structured generators** (SystemV / MSVC `rand`,
  rand48, BSD/Linux `random()`, .NET Random) that pass at the value
  level and are caught only by spectral, bit-rank, lattice, or linear
  complexity tests.
* The Fourier block in this report catches the LCG-tonal subset of
  that group (SystemV rand, MSVC rand, VB6 Rnd) but does not by itself
  detect bit-level or lattice-only failures.

Use NIST SP 800-22, DIEHARD, DIEHARDER, and TestU01 (all of which
this repository ports) for the deeper tier.  The R battery in this
report is a useful *pre-screen* and a check that the new CSPRNGs
behave like white noise on the simplest invariants — moments and
spectrum — exactly the surface that any classical statistical
collaborator would inspect first.
