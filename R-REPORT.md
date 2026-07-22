# R-REPORT — RNG tests via R's standard randomness packages

Each generator below was sampled into a binary stream of little-endian u32
words. R then read the stream, normalised it to U[0,1), and ran the
randomness tests exposed by the standard R RNG-testing packages that apply
to a value-level uniform stream — `randtests` (runs, Bartels rank,
Cox-Stuart, difference-sign, turning-point, Mann-Kendall rank),
`randtoolbox` (freq, gap, serial, poker, order) — plus `tseries`
(runs, Jarque-Bera), the goodness-of-fit and autocorrelation tests in
`stats` (KS, χ²(256), Ljung-Box), and `moments` for the raw-moment
diagnostics.  (`randtoolbox::coll.test` was not run.)

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
- `tseries` 0.10.62
- `moments` 0.14.1
- `stats` 4.6.1

R version: 4.6.1
Host: baase — Linux 6.17.0-1026-nvidia aarch64, ARM Cortex-X925, 20 cores
Date: 2026-07-22 12:41:38 PDT

---

## OsRng (/dev/urandom)

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499765  Var = 0.083316  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.334516 | 0.737990 | pass |
| randtests::bartels.rank.test | -0.224152 | 0.822639 | pass |
| randtests::cox.stuart.test (trend) | 1249902.000000 | 0.901847 | pass |
| randtests::difference.sign.test | 0.824945 | 0.409403 | pass |
| randtests::turning.point.test | -1.008688 | 0.313124 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.306018 | 0.759591 | pass |
| randtoolbox::freq.test (16 bins) | 11.428774 | 0.721650 | pass |
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 18.176629 | 0.313656 | pass |
| randtoolbox::serial.test (d=8) | 55.247770 | 0.745729 | pass |
| randtoolbox::poker.test (5-hand) | 3.768045 | 0.438307 | pass |
| randtoolbox::order.test (d=4) | 18.224819 | 0.745183 | pass |
| stats::ks.test vs U(0,1) | 0.000466 | 0.227867 | pass |
| stats::chisq.test (256 bins) | 246.063411 | 0.644616 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 46.290112 | 0.005938 | pass |
| tseries::runs.test (binary) | 0.334516 | 0.737990 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299768.266620 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49976516 | 0.50000000 | 2.35e-04 |
| 2 | 0.33308104 | 0.33333333 | 2.52e-04 |
| 3 | 0.24975180 | 0.25000000 | 2.48e-04 |
| 4 | 0.19976260 | 0.20000000 | 2.37e-04 |
| 5 | 0.16644285 | 0.16666667 | 2.24e-04 |
| 6 | 0.14264767 | 0.14285714 | 2.09e-04 |
| 7 | 0.12480453 | 0.12500000 | 1.95e-04 |
| 8 | 0.11092877 | 0.11111111 | 1.82e-04 |
| 9 | 0.09982970 | 0.10000000 | 1.70e-04 |
| 10 | 0.09074970 | 0.09090909 | 1.59e-04 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 15.075504 |
| Max-spike exact p (no spike) | 0.507933 |
| Spectral flatness (Wiener entropy) | 0.561672 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=2.660, p=0.976259 |
| Periodogram KS vs Exp(1) | D=0.000397, p=0.825001 |


## ConstantRng

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.869841  Var = 0.000000  Min = 0.869841  Max = 0.869841

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | NA | NA | n/a |
| randtests::bartels.rank.test | NA | NA | n/a |
| randtests::cox.stuart.test (trend) | 0.000000 | 2.000000 | pass |
| randtests::difference.sign.test | 0.000000 | 1.000000 | pass |
| randtests::turning.point.test | NA | NA | n/a |
| randtests::rank.test (Mann-Kendall, n=5000) | -106.028906 | 0.000000 | REJECT |
| randtoolbox::freq.test (16 bins) | 75000000.000000 | 0.000000 | REJECT |
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 1249999.850988 | 0.000000 | REJECT |
| randtoolbox::serial.test (d=8) | 157500000.000000 | 0.000000 | REJECT |
| randtoolbox::poker.test (5-hand) | 624000000.000000 | 0.000000 | REJECT |
| randtoolbox::order.test (d=4) | 1250000.000000 | 0.000000 | REJECT |
| stats::ks.test vs U(0,1) | 0.869841 | 0.000000 | REJECT |
| stats::chisq.test (256 bins) | 1275000000.000000 | 0.000000 | REJECT |
| stats::Box.test (Ljung-Box, lag 25) | NA | NA | n/a |
| tseries::runs.test (binary) | NA | NA | n/a |
| tseries::jarque.bera.test (vs Normal*) | NA | NA | n/a |

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
| Max-spike exact p (no spike) | 1.000000 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 1249998.052429 | 0.000000 | REJECT |
| randtoolbox::serial.test (d=8) | 157500000.000000 | 0.000000 | REJECT |
| randtoolbox::poker.test (5-hand) | 624000000.000000 | 0.000000 | REJECT |
| randtoolbox::order.test (d=4) | 28750000.000000 | 0.000000 | REJECT |
| stats::ks.test vs U(0,1) | 0.998836 | 0.000000 | REJECT |
| stats::chisq.test (256 bins) | 1275000000.000000 | 0.000000 | REJECT |
| stats::Box.test (Ljung-Box, lag 25) | 124998425.003579 | 0.000000 | REJECT |
| tseries::runs.test (binary) | -2236.067307 | 0.000000 | REJECT |
| tseries::jarque.bera.test (vs Normal*) | 300000.000000 | 0.000000 | REJECT |

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
| Max-spike exact p (no spike) | 1.000000 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 18.070866 | 0.319764 | pass |
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
| Max-spike exact p (no spike) | 3.354e-97 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 14.967805 | 0.526998 | pass |
| randtoolbox::serial.test (d=8) | 71.644160 | 0.212951 | pass |
| randtoolbox::poker.test (5-hand) | 6.801089 | 0.146781 | pass |
| randtoolbox::order.test (d=4) | 24.067533 | 0.400077 | pass |
| stats::ks.test vs U(0,1) | 0.000419 | 0.344360 | pass |
| stats::chisq.test (256 bins) | 243.425382 | 0.688237 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 20.889371 | 0.698762 | pass |
| tseries::runs.test (binary) | 0.466891 | 0.640578 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300217.635393 | 0.000000 | REJECT |

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
| Max-spike exact p (no spike) | 0.413232 |
| Spectral flatness (Wiener entropy) | 0.561703 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=4.209, p=0.897121 |
| Periodogram KS vs Exp(1) | D=0.000352, p=0.916103 |


## BSD random()

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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 24.679959 | 0.075670 | pass |
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
| Max-spike exact p (no spike) | 0.586711 |
| Spectral flatness (Wiener entropy) | 0.561453 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=11.124, p=0.267321 |
| Periodogram KS vs Exp(1) | D=0.000513, p=0.526190 |


## Linux glibc rand()/random()

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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 24.679959 | 0.075670 | pass |
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
| Max-spike exact p (no spike) | 0.586711 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 18.442818 | 0.298611 | pass |
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
| Max-spike exact p (no spike) | 0.639311 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 15.403716 | 0.495292 | pass |
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
| Max-spike exact p (no spike) | 5.562e-110 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 726.864651 | 2.468e-144 | REJECT |
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
| Max-spike exact p (no spike) | -0.000000 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 19.813346 | 0.228745 | pass |
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
| Max-spike exact p (no spike) | 0.987723 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 1249998.052429 | 0.000000 | REJECT |
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
| Max-spike exact p (no spike) | 1.000000 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 1249998.052429 | 0.000000 | REJECT |
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
| Max-spike exact p (no spike) | 1.000000 |
| Spectral flatness (Wiener entropy) | 0.562505 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=3149485.991, p=0.000000 |
| Periodogram KS vs Exp(1) | D=0.472539, p=0.000000 |


## Borland C++ LCG

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499991  Var = 0.083314  Min = 0.000001  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.008050 | 0.993577 | pass |
| randtests::bartels.rank.test | 0.432238 | 0.665568 | pass |
| randtests::cox.stuart.test (trend) | 1249604.000000 | 0.616883 | pass |
| randtests::difference.sign.test | -0.222309 | 0.824073 | pass |
| randtests::turning.point.test | 0.817769 | 0.413489 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 1.484328 | 0.137722 | pass |
| randtoolbox::freq.test (16 bins) | 25.020608 | 0.049667 | pass |
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 16.048596 | 0.449574 | pass |
| randtoolbox::serial.test (d=8) | 53.539840 | 0.796414 | pass |
| randtoolbox::poker.test (5-hand) | 681.072029 | 4.370e-146 | REJECT |
| randtoolbox::order.test (d=4) | 21.644378 | 0.541798 | pass |
| stats::ks.test vs U(0,1) | 0.000281 | 0.824285 | pass |
| stats::chisq.test (256 bins) | 212.415386 | 0.975711 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 40.627624 | 0.025113 | pass |
| tseries::runs.test (binary) | 0.008050 | 0.993577 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299718.813871 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49999082 | 0.50000000 | 9.18e-06 |
| 2 | 0.33330463 | 0.33333333 | 2.87e-05 |
| 3 | 0.24995788 | 0.25000000 | 4.21e-05 |
| 4 | 0.19995227 | 0.20000000 | 4.77e-05 |
| 5 | 0.16661675 | 0.16666667 | 4.99e-05 |
| 6 | 0.14280603 | 0.14285714 | 5.11e-05 |
| 7 | 0.12494773 | 0.12500000 | 5.23e-05 |
| 8 | 0.11105750 | 0.11111111 | 5.36e-05 |
| 9 | 0.09994490 | 0.10000000 | 5.51e-05 |
| 10 | 0.09085244 | 0.09090909 | 5.67e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 250.016027 |
| Max-spike exact p (no spike) | 6.567e-103 |
| Spectral flatness (Wiener entropy) | 0.397833 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=236130.963, p=0.000000 |
| Periodogram KS vs Exp(1) | D=0.135023, p=0.000000 |


## MSVC LCG

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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 15.403716 | 0.495292 | pass |
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
| Max-spike exact p (no spike) | 5.562e-110 |
| Spectral flatness (Wiener entropy) | 0.401885 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=224253.302, p=0.000000 |
| Periodogram KS vs Exp(1) | D=0.132000, p=0.000000 |


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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 24.192976 | 0.085374 | pass |
| randtoolbox::serial.test (d=8) | 78.791629 | 0.086561 | pass |
| randtoolbox::poker.test (5-hand) | 2.681619 | 0.612435 | pass |
| randtoolbox::order.test (d=4) | 20.738906 | 0.597029 | pass |
| stats::ks.test vs U(0,1) | 0.000504 | 0.158075 | pass |
| stats::chisq.test (256 bins) | 308.476211 | 0.012251 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 17.024995 | 0.880885 | pass |
| tseries::runs.test (binary) | -0.064399 | 0.948653 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300104.776328 | 0.000000 | REJECT |

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
| Max-spike exact p (no spike) | 0.232736 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 17.451373 | 0.356986 | pass |
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
| Max-spike exact p (no spike) | 0.309592 |
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
| randtests::bartels.rank.test | 0.692802 | 0.488434 | pass |
| randtests::cox.stuart.test (trend) | 1250865.000000 | 0.274167 | pass |
| randtests::difference.sign.test | 0.287375 | 0.773825 | pass |
| randtests::turning.point.test | 1.121118 | 0.262238 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.074812 | 0.940364 | pass |
| randtoolbox::freq.test (16 bins) | 22.532941 | 0.094572 | pass |
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 28.448810 | 0.027929 | pass |
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
| Max-spike exact p (no spike) | 0.597162 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 19.345846 | 0.251137 | pass |
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
| Max-spike exact p (no spike) | 0.717166 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 25.054576 | 0.068869 | pass |
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
| Max-spike exact p (no spike) | 0.256514 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 11.911502 | 0.750049 | pass |
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
| Max-spike exact p (no spike) | 0.491777 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 9.785092 | 0.877608 | pass |
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
| Max-spike exact p (no spike) | 0.972962 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 20.459635 | 0.200228 | pass |
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
| Max-spike exact p (no spike) | 0.705707 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 13.845577 | 0.610217 | pass |
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
| Max-spike exact p (no spike) | 0.313647 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 18.153012 | 0.315013 | pass |
| randtoolbox::serial.test (d=8) | 51.937843 | 0.838806 | pass |
| randtoolbox::poker.test (5-hand) | 4.193368 | 0.380468 | pass |
| randtoolbox::order.test (d=4) | 23.224269 | 0.447740 | pass |
| stats::ks.test vs U(0,1) | 0.000522 | 0.131647 | pass |
| stats::chisq.test (256 bins) | 251.749786 | 0.545770 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 12.416895 | 0.982879 | pass |
| tseries::runs.test (binary) | -0.232551 | 0.816110 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299547.890819 | 0.000000 | REJECT |

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
| Max-spike exact p (no spike) | 0.478153 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 11.382042 | 0.785313 | pass |
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
| Max-spike exact p (no spike) | 0.039740 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 8.797652 | 0.921512 | pass |
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
| Max-spike exact p (no spike) | 0.509885 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 14.874448 | 0.533853 | pass |
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
| Max-spike exact p (no spike) | 0.331103 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 12.886049 | 0.681071 | pass |
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
| Max-spike exact p (no spike) | 0.651553 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 13.603161 | 0.628251 | pass |
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
| Max-spike exact p (no spike) | 0.215418 |
| Spectral flatness (Wiener entropy) | 0.561292 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=3.247, p=0.953699 |
| Periodogram KS vs Exp(1) | D=0.000304, p=0.974640 |


## Grasshopper-CTR

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499761  Var = 0.083349  Min = 0.000001  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -1.134134 | 0.256738 | pass |
| randtests::bartels.rank.test | -0.527802 | 0.597636 | pass |
| randtests::cox.stuart.test (trend) | 1249047.000000 | 0.228270 | pass |
| randtests::difference.sign.test | -0.044152 | 0.964783 | pass |
| randtests::turning.point.test | 0.670337 | 0.502643 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.806558 | 0.419921 | pass |
| randtoolbox::freq.test (16 bins) | 15.731994 | 0.400084 | pass |
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 17.931546 | 0.327921 | pass |
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
| Max-spike exact p (no spike) | 0.589086 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 24.123109 | 0.086850 | pass |
| randtoolbox::serial.test (d=8) | 68.208230 | 0.304738 | pass |
| randtoolbox::poker.test (5-hand) | 0.745677 | 0.945579 | pass |
| randtoolbox::order.test (d=4) | 25.097075 | 0.345300 | pass |
| stats::ks.test vs U(0,1) | 0.000477 | 0.204550 | pass |
| stats::chisq.test (256 bins) | 275.950490 | 0.175467 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 17.962355 | 0.843985 | pass |
| tseries::runs.test (binary) | -0.076026 | 0.939398 | pass |
| tseries::jarque.bera.test (vs Normal*) | 300015.753410 | 0.000000 | REJECT |

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
| Max-spike exact p (no spike) | 0.207983 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 11.396461 | 0.784378 | pass |
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
| Max-spike exact p (no spike) | 0.042254 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 15.724272 | 0.472365 | pass |
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
| Max-spike exact p (no spike) | 0.950280 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 28.785357 | 0.025424 | pass |
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
| Max-spike exact p (no spike) | 0.514779 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 9.127839 | 0.908056 | pass |
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
| Max-spike exact p (no spike) | 0.712369 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 14.483421 | 0.562747 | pass |
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
| Max-spike exact p (no spike) | 0.939797 |
| Spectral flatness (Wiener entropy) | 0.561397 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=17.190, p=0.045828 |
| Periodogram KS vs Exp(1) | D=0.000513, p=0.525122 |


## ChaCha20

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.500097  Var = 0.083285  Min = 0.000000  Max = 0.999999

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | -0.261173 | 0.793959 | pass |
| randtests::bartels.rank.test | -0.771297 | 0.440531 | pass |
| randtests::cox.stuart.test (trend) | 1249885.000000 | 0.884843 | pass |
| randtests::difference.sign.test | -1.793191 | 0.072942 | pass |
| randtests::turning.point.test | -0.572757 | 0.566810 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -1.101241 | 0.270792 | pass |
| randtoolbox::freq.test (16 bins) | 16.926675 | 0.323267 | pass |
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 8.307190 | 0.939191 | pass |
| randtoolbox::serial.test (d=8) | 60.389632 | 0.569931 | pass |
| randtoolbox::poker.test (5-hand) | 0.792971 | 0.939388 | pass |
| randtoolbox::order.test (d=4) | 15.689459 | 0.868237 | pass |
| stats::ks.test vs U(0,1) | 0.000447 | 0.270274 | pass |
| stats::chisq.test (256 bins) | 236.415693 | 0.792207 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 22.747141 | 0.592326 | pass |
| tseries::runs.test (binary) | -0.261173 | 0.793959 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299647.230082 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.50009734 | 0.50000000 | 9.73e-05 |
| 2 | 0.33338243 | 0.33333333 | 4.91e-05 |
| 3 | 0.25000448 | 0.25000000 | 4.48e-06 |
| 4 | 0.19997442 | 0.20000000 | 2.56e-05 |
| 5 | 0.16662075 | 0.16666667 | 4.59e-05 |
| 6 | 0.14279696 | 0.14285714 | 6.02e-05 |
| 7 | 0.12492946 | 0.12500000 | 7.05e-05 |
| 8 | 0.11103289 | 0.11111111 | 7.82e-05 |
| 9 | 0.09991600 | 0.10000000 | 8.40e-05 |
| 10 | 0.09082072 | 0.09090909 | 8.84e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 14.790794 |
| Max-spike exact p (no spike) | 0.610431 |
| Spectral flatness (Wiener entropy) | 0.561435 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=9.625, p=0.381651 |
| Periodogram KS vs Exp(1) | D=0.000593, p=0.342442 |


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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 19.233506 | 0.256740 | pass |
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
| Max-spike exact p (no spike) | 0.136171 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 17.762930 | 0.337961 | pass |
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
| Max-spike exact p (no spike) | 0.132754 |
| Spectral flatness (Wiener entropy) | 0.561628 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=6.854, p=0.652326 |
| Periodogram KS vs Exp(1) | D=0.000557, p=0.420888 |


## HmacDrbg

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499793  Var = 0.083316  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 2.263795 | 0.023587 | pass |
| randtests::bartels.rank.test | 2.935147 | 0.003334 | pass |
| randtests::cox.stuart.test (trend) | 1250221.000000 | 0.780312 | pass |
| randtests::difference.sign.test | 0.261039 | 0.794062 | pass |
| randtests::turning.point.test | 1.315219 | 0.188436 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | 0.552767 | 0.580423 | pass |
| randtoolbox::freq.test (16 bins) | 14.724736 | 0.471421 | pass |
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 12.838020 | 0.684562 | pass |
| randtoolbox::serial.test (d=8) | 69.559091 | 0.266268 | pass |
| randtoolbox::poker.test (5-hand) | 2.426753 | 0.657798 | pass |
| randtoolbox::order.test (d=4) | 32.022054 | 0.099686 | pass |
| stats::ks.test vs U(0,1) | 0.000485 | 0.191045 | pass |
| stats::chisq.test (256 bins) | 261.436723 | 0.377473 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 30.934210 | 0.191207 | pass |
| tseries::runs.test (binary) | 2.263795 | 0.023587 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299817.348323 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49979310 | 0.50000000 | 2.07e-04 |
| 2 | 0.33310923 | 0.33333333 | 2.24e-04 |
| 3 | 0.24977426 | 0.25000000 | 2.26e-04 |
| 4 | 0.19977861 | 0.20000000 | 2.21e-04 |
| 5 | 0.16645334 | 0.16666667 | 2.13e-04 |
| 6 | 0.14265404 | 0.14285714 | 2.03e-04 |
| 7 | 0.12480819 | 0.12500000 | 1.92e-04 |
| 8 | 0.11093089 | 0.11111111 | 1.80e-04 |
| 9 | 0.09983119 | 0.10000000 | 1.69e-04 |
| 10 | 0.09075122 | 0.09090909 | 1.58e-04 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 17.550286 |
| Max-spike exact p (no spike) | 0.057949 |
| Spectral flatness (Wiener entropy) | 0.561289 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=3.460, p=0.943229 |
| Periodogram KS vs Exp(1) | D=0.000390, p=0.840567 |


## HashDrbg

Sample size: 5,000,000 u32 words (19.07 MB)

Mean = 0.499940  Var = 0.083343  Min = 0.000000  Max = 1.000000

### Tests (alpha = 0.001 reject threshold)

| Test | Statistic | p-value | Verdict |
|------|-----------|---------|---------|
| randtests::runs.test (median) | 0.914105 | 0.360662 | pass |
| randtests::bartels.rank.test | 1.632422 | 0.102591 | pass |
| randtests::cox.stuart.test (trend) | 1250074.000000 | 0.925927 | pass |
| randtests::difference.sign.test | -1.757560 | 0.078822 | pass |
| randtests::turning.point.test | -0.013789 | 0.988999 | pass |
| randtests::rank.test (Mann-Kendall, n=5000) | -1.378227 | 0.168133 | pass |
| randtoolbox::freq.test (16 bins) | 19.713798 | 0.183189 | pass |
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 19.790218 | 0.229818 | pass |
| randtoolbox::serial.test (d=8) | 58.140723 | 0.649782 | pass |
| randtoolbox::poker.test (5-hand) | 2.500490 | 0.644548 | pass |
| randtoolbox::order.test (d=4) | 17.853338 | 0.765395 | pass |
| stats::ks.test vs U(0,1) | 0.000325 | 0.664966 | pass |
| stats::chisq.test (256 bins) | 224.664678 | 0.914772 | pass |
| stats::Box.test (Ljung-Box, lag 25) | 24.927675 | 0.466433 | pass |
| tseries::runs.test (binary) | 0.914105 | 0.360662 | pass |
| tseries::jarque.bera.test (vs Normal*) | 299977.363268 | 0.000000 | REJECT |

*Note*: Jarque-Bera tests Normality; uniform output is expected to REJECT.

### Raw moments E[U^k] vs theoretical 1/(k+1)

| k | observed | theoretical | abs error |
|---|----------|-------------|-----------|
| 1 | 0.49993981 | 0.50000000 | 6.02e-05 |
| 2 | 0.33328234 | 0.33333333 | 5.10e-05 |
| 3 | 0.24993800 | 0.25000000 | 6.20e-05 |
| 4 | 0.19992547 | 0.20000000 | 7.45e-05 |
| 5 | 0.16658303 | 0.16666667 | 8.36e-05 |
| 6 | 0.14276796 | 0.14285714 | 8.92e-05 |
| 7 | 0.12490797 | 0.12500000 | 9.20e-05 |
| 8 | 0.11101806 | 0.11111111 | 9.30e-05 |
| 9 | 0.09990711 | 0.10000000 | 9.29e-05 |
| 10 | 0.09081709 | 0.09090909 | 9.20e-05 |

### Fourier / spectral analysis (centred series y_t = u_t - 1/2)

| Metric | Value |
|--------|-------|
| Periodogram bins tested (m = N/2 - 1) | 2,499,999 |
| max normalized periodogram (P_max) | 17.620433 |
| Max-spike exact p (no spike) | 0.054132 |
| Spectral flatness (Wiener entropy) | 0.561283 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=11.253, p=0.258786 |
| Periodogram KS vs Exp(1) | D=0.000480, p=0.612797 |


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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=16) | 22.695158 | 0.122130 | pass |
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
| Max-spike exact p (no spike) | 0.703177 |
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
| randtoolbox::gap.test [0,0.5) (Cochran-trimmed, df=14) | 19.210057 | 0.157077 | pass |
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
| Max-spike exact p (no spike) | 0.331755 |
| Spectral flatness (Wiener entropy) | 0.560026 |
| Theoretical flatness for white noise | 0.561459 |
| Periodogram chi^2 (10 Exp(1) bins, df=9) | chi2=13.749, p=0.131536 |
| Periodogram KS vs Exp(1) | D=0.001407, p=0.275327 |


---

## Analysis (2026-07-22 run)

The March analysis sections (§A–§G) described the pre-fix harvest and live in
git history; this section covers the present run only.

**A. False-reject budget.** Excluding the deliberately-inverted Jarque-Bera
row, each generator faces 15 tests at α = 0.001, so the whole 44-generator run
carries an expected false-REJECT count of 44 × 15 × 0.001 ≈ 0.66.  Observed:
**zero REJECTs across every non-legacy generator** — all 19 cryptographic
generators (8 block-CTR, 4 stream, 6 DRBG-class, Dual_EC), the statistical
PRNGs, and the OS source are clean.  The single HmacDrbg REJECT in the March
run (cox.stuart, p = 0.000826) does not recur; it was the statistical noise
the March analysis judged it to be.

**B. The battery still bites.** CounterRng REJECTs 15/15, ConstantRng 8
(rank, freq, gap, serial, poker, order, KS, χ²; its remaining rows are
degenerate-input skips rather than passes).  VB6 Rnd(), the ANSI C LCG, and
MINSTD each REJECT 6.  The four 15-bit-LCG-family generators (System V,
MSVC rand(), MSVC LCG, Borland) each show exactly one REJECT —
poker.test, at p between 1e-146 and 1e-31 — and System V additionally owns
the run's most spectacular number: a periodogram max-spike exact p of
3.4e-97, a pure LCG lattice lighting up the Fourier domain.  The
`windows_msvc_rand` and `msvc_lcg` rows agree to the last digit, as they
must — they are the same stream under two registry names.

**C. Fixed-seed determinism.** Generators with fixed seeds reproduce their
March statistics exactly where the R code is unchanged — e.g. Squidward's
Cochran-trimmed gap.test (χ² = 17.7629, df = 16, p = 0.3380) matches the
March corrected value to six digits on different hardware (Cortex-X925 vs
the March host), confirming the dump path and the trimmed gap test are
bit-stable.

**D. Value-level blindness, by design.** BSD random() and glibc
rand()/random() — now correctly separated into two rows — both score 0
REJECTs here: their known weaknesses live in low-order bit correlations
that value-level uniform tests do not see.  The bit-level main battery
(TESTS.md) is where they fail.  Dual_EC_DRBG likewise scores 0 at the
value level: the backdoor is statistically invisible, which is its entire
lesson.

**E. Sanity rows.** Jarque-Bera REJECTs in all 44 sections, as it must for
uniform input; no test emitted NaN (the all-zero-periodogram flatness guard
now reports a clean value or NA); every gap.test row is Cochran-trimmed
with its merged df stated inline.
