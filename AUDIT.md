# Open issues

Known defects and limitations of the current code, each with where it lives
and what closing it requires.  Improvements that are not defects are in
[SUGGESTIONS.md](SUGGESTIONS.md).

## Statistical validity

### A1 — Minimum distance ignores dependence between pairs

[minimum_distance_nd.rs](src/dieharder/minimum_distance_nd.rs).  The transform
1 − exp(−C(n, 2)·H_d(r)) treats close pairs as Poisson.  At 500 points in
three and four dimensions 40 000 clouds resolve a bias of a few thousandths
in u (KS p = 0.005 and 0.02); the test's 20-cloud KS does not.  Derive the
second-order term for the cube, or bound the quick mode's repetitions.

### A2 — Q5 − Q4 is only approximately χ²(2 500)

[count_ones.rs](src/diehard/count_ones.rs).  100 000 windows of
`count_ones_bytes` rejected at 0.01 in 1.045% (standard deviation 0.031%),
KS p = 0.028.  A finite-window law would remove the excess.

### A3 — Simulated LZ78 tables bound the replication count

[testu01_lz.rs](src/research/testu01_lz.rs).  Tables of 10⁶, 10⁵ and 10⁴
replications support N ≤ 10 000 (k ≤ 16), 1 000 (k ≤ 22) and 100 (k ≥ 23).
Held-out runs reject near nominal rates, except k = 20 at N = 1 000 (2.3% and
2.5% below 0.01 over 400 runs) and the KS statistic pooled over k ≥ 23 (6.0%
below 0.05 over 1 440 runs, standard deviation 0.6%).  Larger tables would
settle both and admit more replications.

### A4 — The R report is calibrated only at 0.01, and only in part

[scripts/r_rng_tests.R](scripts/r_rng_tests.R).  Bartlett's cumulative
periodogram test rejected at 0.01 in 1.30% and 0.90% of null streams, too few
runs to measure the report's α = 0.001.  The periodogram height KS is
conservative on uniform input.  The package tests are uncalibrated.
