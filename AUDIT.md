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

### A2 — Pearson χ² with about 20 expected counts per cell runs slightly high

Exactly multinomial counts in 256 equal cells of 5 000 reject against
χ²(255) at 0.01 in 1.013% and at 0.001 in 0.1046% (standard deviation 0.003%),
and the DCT test matches that over 10⁶ runs.  Every Pearson test with small
expected counts shares the excess.  An exact or simulated null for the
Pearson statistic at the battery's cell counts would remove it.

### A3 — The R report's 0.001 threshold is calibrated only coarsely

[scripts/r_rng_tests.R](scripts/r_rng_tests.R).  3 000 null streams per row
resolve rejection at the report's α = 0.001 only to about ±0.06 points; the
runs tests, rerun on 20 000 streams, give 1.09% at 0.01 and 0.135% at 0.001.
The periodogram height tests are conservative by construction.  Recalibrate
the other rows at 0.001 with enough streams.

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

### A6 — The sequential test's error bound assumes a correctly rounded ln

[sequential.rs](src/research/sequential.rs).  Each model now carries a bound
on its log-wealth's floating-point error and p uses the resulting lower bound,
which covers the observed error against order 0's closed form.  The bound
assumes `f64::ln` is within one relative ε, which the platform library does
not promise.  The anytime guarantee is for one monitored stream, not for
selection across generators or restarts.

## Cost

### A7 — Nearest pair is quadratic when many points share the axis value

[nearest_pair.rs](src/diehard/nearest_pair.rs).  The sweep runs along the
widest coordinate and stops at a zero distance, but many distinct points with
one value on that coordinate still visit every pair among themselves.  A grid
or multidimensional partition would bound that case.
