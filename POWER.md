# Power of the NIST SP 800-22 Tests

How often each NIST test family detects a specified defect, and how often it
raises a false alarm when there is none.  Run with
`cargo run --release --example power_curves <streams> <threads>` and tabulated by
`scripts/power_report.py stats/nist-power-100-streams.txt POWER.md`.

## Procedure

Each stream is a PCG64 generator with its own sequence, passed through one
defect from `entropy::rng::alternatives` and then through `nist::run_all` at
2²⁰, 2²² or 2²⁴ bits.  A family with m scored results rejects when m times its
smallest p-value is below 0.01 (Bonferroni's bound, valid under any
dependence among the family's results).  Maurer's universal test, run at every
block length its size allows, is shown as the largest count over those
lengths, so that column is not a single-test rate.

The row `none` measures each family's false-alarm rate.  Bonferroni's bound
keeps it at or below 1%; with 100 streams, a count of 3 is still consistent with 1%
(P(X ≥ 3) ≈ 0.08 for one family, and the table has many).  An interval of ±10%
around 1% would need about 38 000 streams per family, so this run bounds
gross miscalibration only, not the tail.  Power counts are likewise
binomial: a count c of 100 has a 95% interval about ±2√(c(100 − c)/100).

Run of 2026-09-17 on dyson, `examples/power_curves.rs` at 84dc79b, 100
streams per cell.  The raw output is in `stats/nist-power-100-streams.txt`.

## Rejections

<!-- power table: begin -->
Streams per cell: 100.  `-`: the family scored no result at that size.

| Defect | Bits | Freq | Block | Cusum+ | Cusum− | Runs | Longest | Rank | DFT | NonOvl | Ovl | Univ | ApEn | Serial1 | Serial2 | LinComp | Exc | ExcVar | Maurer (max L) |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| none | 2^20 | 0 | 1 | 0 | 0 | 0 | 0 | 3 | 2 | 1 | 1 | 1 | 2 | 2 | 3 | 1 | 1 | 0 | 4 |
| none | 2^22 | 2 | 0 | 1 | 2 | 1 | 0 | 0 | 3 | 0 | 0 | 1 | 2 | 1 | 2 | 0 | 0 | 1 | 2 |
| none | 2^24 | 3 | 0 | 3 | 3 | 1 | 1 | 0 | 1 | 3 | 2 | 0 | 0 | 1 | 2 | 1 | 0 | 0 | 3 |
| bits biased to ½ + 2^-9 | 2^20 | 93 | 1 | 91 | 91 | 51 | 1 | 1 | 1 | 2 | 4 | 2 | 1 | 77 | 0 | 1 | 0 | 0 | 3 |
| bits biased to ½ + 2^-9 | 2^22 | 100 | 2 | 100 | 100 | 100 | 1 | 1 | 0 | 6 | 8 | 0 | 19 | 100 | 1 | 0 | 0 | 0 | 3 |
| bits biased to ½ + 2^-9 | 2^24 | 100 | 5 | 100 | 100 | 100 | 9 | 3 | 2 | 89 | 66 | 0 | 100 | 100 | 1 | 1 | 1 | 0 | 3 |
| bits biased to ½ + 2^-11 | 2^20 | 7 | 2 | 6 | 6 | 0 | 1 | 0 | 2 | 1 | 2 | 1 | 0 | 3 | 1 | 2 | 0 | 0 | 2 |
| bits biased to ½ + 2^-11 | 2^22 | 34 | 0 | 32 | 32 | 1 | 0 | 1 | 0 | 0 | 1 | 1 | 0 | 13 | 0 | 1 | 0 | 0 | 5 |
| bits biased to ½ + 2^-11 | 2^24 | 95 | 0 | 94 | 95 | 46 | 2 | 1 | 2 | 1 | 1 | 1 | 3 | 88 | 1 | 1 | 1 | 0 | 3 |
| bits biased to ½ + 2^-13 | 2^20 | 2 | 0 | 2 | 3 | 0 | 1 | 2 | 2 | 1 | 0 | 2 | 0 | 0 | 1 | 0 | 2 | 0 | 3 |
| bits biased to ½ + 2^-13 | 2^22 | 1 | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 3 |
| bits biased to ½ + 2^-13 | 2^24 | 4 | 2 | 5 | 3 | 1 | 3 | 2 | 1 | 0 | 2 | 0 | 1 | 2 | 1 | 1 | 0 | 3 | 1 |
| 1 low bit(s) stuck at 0 | 2^20 | 100 | 100 | 100 | 100 | 100 | 75 | 100 | 100 | 100 | 100 | 13 | 100 | 100 | 1 | 0 | - | - | 33 |
| 1 low bit(s) stuck at 0 | 2^22 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 2 | 3 | - | - | 100 |
| 1 low bit(s) stuck at 0 | 2^24 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 0 | 1 | - | - | 100 |
| 65536-word blocks repeated | 2^20 | 1 | 2 | 1 | 1 | 1 | 2 | 1 | 1 | 2 | 0 | 1 | 3 | 0 | 0 | 0 | 0 | 0 | 3 |
| 65536-word blocks repeated | 2^22 | 8 | 4 | 6 | 8 | 4 | 17 | 8 | 100 | 28 | 12 | 13 | 100 | 13 | 9 | 1 | 2 | 1 | 13 |
| 65536-word blocks repeated | 2^24 | 9 | 4 | 11 | 10 | 8 | 10 | 16 | 100 | 29 | 17 | 1 | 100 | 17 | 10 | 1 | 2 | 1 | 10 |
| 4096-word blocks repeated | 2^20 | 6 | 2 | 7 | 6 | 5 | 23 | 12 | 100 | 29 | 29 | 3 | 100 | 16 | 8 | 1 | 0 | 0 | 4 |
| 4096-word blocks repeated | 2^22 | 8 | 8 | 10 | 10 | 8 | 19 | 8 | 100 | 99 | 13 | 5 | 100 | 14 | 9 | 0 | 2 | 1 | 5 |
| 4096-word blocks repeated | 2^24 | 6 | 6 | 11 | 7 | 7 | 19 | 15 | 100 | 100 | 17 | 1 | 100 | 16 | 11 | 0 | 1 | 0 | 9 |
| top bit copies bit 30 of previous word | 2^20 | 1 | 73 | 1 | 2 | 1 | 1 | 1 | 2 | 0 | 1 | 3 | 0 | 0 | 0 | 0 | 1 | 0 | 3 |
| top bit copies bit 30 of previous word | 2^22 | 1 | 100 | 1 | 1 | 2 | 0 | 1 | 9 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 4 | 0 | 3 |
| top bit copies bit 30 of previous word | 2^24 | 0 | 100 | 0 | 0 | 0 | 2 | 1 | 58 | 2 | 3 | 1 | 2 | 0 | 1 | 2 | 1 | 0 | 2 |
| period 65536 words | 2^20 | 1 | 0 | 3 | 1 | 3 | 1 | 0 | 3 | 2 | 1 | 1 | 2 | 0 | 0 | 2 | 0 | 1 | 4 |
| period 65536 words | 2^22 | 8 | 5 | 8 | 8 | 8 | 17 | 9 | 100 | 30 | 12 | 9 | 100 | 14 | 7 | 1 | 1 | 0 | 9 |
| period 65536 words | 2^24 | 43 | 22 | 39 | 38 | 47 | 86 | 53 | 100 | 100 | 81 | 8 | 100 | 79 | 56 | 1 | 0 | 1 | 37 |
| period 4096 words | 2^20 | 37 | 18 | 34 | 36 | 43 | 91 | 59 | 100 | 100 | 85 | 17 | 100 | 79 | 56 | 0 | 1 | 1 | 31 |
| period 4096 words | 2^22 | 69 | 35 | 67 | 68 | 83 | 99 | 81 | 100 | 100 | 99 | 59 | 100 | 97 | 85 | 1 | 0 | 0 | 59 |
| period 4096 words | 2^24 | 77 | 39 | 77 | 76 | 93 | 100 | 98 | 100 | 100 | 100 | 72 | 100 | 100 | 98 | 3 | 0 | 0 | 83 |
<!-- power table: end -->

## Reading the table

- **No defect.**  No family exceeds 3/100, and Maurer's largest count over
  its block lengths is 4/100.  This is consistent with the 1% bound at this
  resolution.
- **Bias.**  At ½ + 2⁻⁹ the frequency, cumulative-sums and first serial
  families detect it from 2²⁰ bits, and runs from 2²² bits.  At ½ + 2⁻¹¹
  they need 2²⁴ bits, and at ½ + 2⁻¹³ nothing detects it by 2²⁴ bits.  The
  frequency test's statistic has mean 2·bias·√n: 4 at n = 2²⁴ for bias 2⁻¹¹
  (95/100) and 1 for bias 2⁻¹³, well below its two-sided 1% point of 2.58.
- **A stuck low bit** is found at 2²⁰ bits by every family except longest run (75/100), the second
  serial difference, linear complexity and (until 2²² bits) the universal
  test.  Random excursions score nothing: the stuck bit pulls the walk away
  from zero, leaving too few cycles to test.
- **Repeated blocks.**  Approximate entropy and the spectral test detect a
  repeat as soon as the sample holds a block and its copy: for 4 096-word
  blocks from 2²⁰ bits, and for 65 536-word blocks (2²¹ bits each) from 2²²
  bits.  The other families stay weak because every block is new.
- **Short periods.**  The same two families detect a period once two copies
  fit.  As the copies multiply, every family's statistic repeats one
  fluctuation, and most families gain power: a period of 4 096 words at 2²⁴
  bits is found by all but linear complexity and random excursions.
- **The lagged top bit** (equal to bit 30 of the previous word) is caught by
  block frequency, in 73/100 streams at 2²⁰ bits and all from 2²², and
  partly by the spectral test at 2²⁴ bits.  No other family sees it.
- **Linear complexity and random excursions** detect none of these defects.
  None of them is linear, and only the stuck bit moves the walk, which then
  goes unscored.
