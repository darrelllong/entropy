# Open issues

Known defects and limitations of the current code, each with where it lives
and what closing it requires.  Improvements that are not defects are in
[SUGGESTIONS.md](SUGGESTIONS.md).

## Statistical validity

### A1 — Q5 − Q4 is only approximately χ²(2 500)

[count_ones.rs](src/diehard/count_ones.rs).  300 000 windows of
`count_ones_bytes` on xoshiro256** rejected at 0.01 in 1.060% (standard
deviation 0.018%); an earlier 100 000 on PCG64 gave 1.045%.  The excess is
real but small.  A finite-window law for Q5 − Q4 would remove it.

### A2 — Simulated LZ78 tables bound the replication count

[testu01_lz.rs](src/research/testu01_lz.rs).  Tables of 10⁶, 10⁵ and 10⁴
replications support N ≤ 10 000 (k ≤ 16), 1 000 (k ≤ 22) and 100 (k ≥ 23).
Held-out runs reject near nominal rates, except k = 20 at N = 1 000 (2.3% and
2.5% below 0.01 over 400 runs) and the KS statistic pooled over k ≥ 23 (6.0%
below 0.05 over 1 440 runs, standard deviation 0.6%).  Larger tables would
settle both and admit more replications.

### A3 — The R report is calibrated only at 0.01, and only in part

[scripts/r_rng_tests.R](scripts/r_rng_tests.R).  Bartlett's cumulative
periodogram test rejected at 0.01 in 1.30% and 0.90% of null streams, too few
runs to measure the report's α = 0.001.  The periodogram height KS is
conservative on uniform input.  The package tests are uncalibrated.
