# Open issues

Known defects and limitations of the current code, each with where it lives
and what closing it requires.  Improvements that are not defects are in
[SUGGESTIONS.md](SUGGESTIONS.md).

## Specification

### A1 — WyRand's constants have no mathematical specification

[wyrand.rs](src/rng/wyrand.rs).  The implemented constants, and the later
0x2d358dccaa6c78a5 and 0x8bb84b93962eacc9, appear only in the author's source.
The author's README specifies a successor, `w1rand`, with one constant
c = 0xd07ebc63274654c7 for both the increment and the mix.  Implement that
from the README, or mark WyRand as unspecified and withdraw its claims.

## Statistical validity

### A2 — Minimum distance ignores dependence between pairs

[minimum_distance_nd.rs](src/dieharder/minimum_distance_nd.rs).  The transform
1 − exp(−C(n, 2)·H_d(r)) treats close pairs as Poisson.  At 500 points in
three and four dimensions 40 000 clouds resolve a bias of a few thousandths
in u (KS p = 0.005 and 0.02); the test's 20-cloud KS does not.  Derive the
second-order term for the cube, or bound the quick mode's repetitions.

### A3 — Q5 − Q4 is only approximately χ²(2 500)

[count_ones.rs](src/diehard/count_ones.rs).  100 000 windows of
`count_ones_bytes` rejected at 0.01 in 1.045% (standard deviation 0.031%),
KS p = 0.028.  A finite-window law would remove the excess.

### A4 — Simulated LZ78 tables bound the replication count

[testu01_lz.rs](src/research/testu01_lz.rs).  Tables of 10⁶, 10⁵ and 10⁴
replications support N ≤ 10 000 (k ≤ 16), 1 000 (k ≤ 22) and 100 (k ≥ 23).
Held-out runs reject near nominal rates, except k = 20 at N = 1 000 (2.3% and
2.5% below 0.01 over 400 runs) and the KS statistic pooled over k ≥ 23 (6.0%
below 0.05 over 1 440 runs, standard deviation 0.6%).  Larger tables would
settle both and admit more replications.

### A5 — The R report is calibrated only at 0.01, and only in part

[scripts/r_rng_tests.R](scripts/r_rng_tests.R).  Bartlett's cumulative
periodogram test rejected at 0.01 in 1.30% and 0.90% of null streams, too few
runs to measure the report's α = 0.001.  The periodogram height KS is
conservative on uniform input.  The package tests are uncalibrated.

## Coverage

### A6 — Batteries test only the high half of 64-bit generators

[views.rs](src/rng/views.rs) provides low-half, full-word and bit-reversed
views, but the batteries run the default high-half `next_u32`, so a defect
confined to low bits is unseen.  Run the views as named, dependent
experiments.
