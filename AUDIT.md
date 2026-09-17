# Open issues

Known defects and limitations of the current code, each with where it lives
and what closing it requires.  Improvements that are not defects are in
[SUGGESTIONS.md](SUGGESTIONS.md).

## Statistical validity

### A1 — Q5 − Q4 is only approximately χ²(2 500)

[count_ones.rs](src/diehard/count_ones.rs).  300 000 windows of
`count_ones_bytes` on xoshiro256** rejected at 0.01 in 1.060% (standard
deviation 0.018%); 100 000 on PCG64 gave 1.045%.  A finite-window law for
Q5 − Q4 would remove the excess.

### A2 — The DCT maximum's position is not exactly uniform

[dct.rs](src/dieharder/dct.rs).  100 000 null runs rejected at 0.001 in 0.127%
(standard deviation 0.010%); at 0.01 in 1.041%.  Equal coefficient variances
do not make the absolute coefficients exchangeable.  Derive the position law
or tabulate it by simulation on separate streams.

### A3 — The R report's package tests are calibrated only coarsely

[scripts/r_rng_tests.R](scripts/r_rng_tests.R).  Over 3 000 null streams the
two runs tests rejected at 0.01 in 1.57% (standard deviation 0.18%) and the
periodogram height tests are conservative; 3 000 streams resolve rejection at
the report's α = 0.001 only to about ±0.06 points.  Recalibrate at 0.001 with
enough streams, and correct or relabel the runs tests.

### A4 — Minimum distance and LZ78 are calibrated at chosen points, not by law

[minimum_distance_nd.rs](src/dieharder/minimum_distance_nd.rs) rests on a
Poisson approximation; its decisions reject near 1% at 0.01 in quick and full
mode, but not at other thresholds.  [testu01_lz.rs](src/research/testu01_lz.rs)
limits N to a hundredth of a simulated table, a heuristic rather than a bound
on type-I error; held-out runs above k = 20 number 80 to 300.  Measure both at
0.001 and, for LZ78, at more runs for k ≥ 21.

### A5 — Battery decisions and power are not calibrated

Each test is calibrated alone.  The false-alarm rate of a whole-battery verdict,
and power against the five `--alternatives` defects at several strengths and
sample sizes, have not been measured over repeated independent streams.

## Numerics

### A6 — The sequential test's floating error is checked, not bounded

[sequential.rs](src/research/sequential.rs).  Order 0's log-wealth matches its
closed form over 10⁵ bits; no bound covers 2⁵³ bits or the mixture.  Its
anytime guarantee is for one monitored stream, not for selection across
generators or restarts.

## Cost

### A7 — Nearest pair is quadratic when many points share a first coordinate

[nearest_pair.rs](src/diehard/nearest_pair.rs).  The sweep stops at a zero
distance, but distinct points on one first coordinate still visit every pair.
A grid or multidimensional partition would bound that case.
