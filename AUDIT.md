# Open issues

Known defects and limitations of the current code, each with where it lives
and what closing it requires.  Improvements that are not defects are in
[SUGGESTIONS.md](SUGGESTIONS.md).

## Statistical validity

### A1 — The n-dimensional minimum-distance transform ignores the cube's boundary

[minimum_distance_nd.rs](src/dieharder/minimum_distance_nd.rs) and
[nearest_pair.rs](src/diehard/nearest_pair.rs).  Points are drawn in the unit
d-cube with ordinary Euclidean distance, but the transform takes each pair's
probability of lying within r as the volume of a whole d-ball.  For
independent uniform points the exact pair probability is

```text
H_d(r) = P(||U − V|| ≤ r) = Σ_{k=0..d} (−1)^k C(d,k) π^((d−k)/2) r^(d+k) / Γ(1 + (d+k)/2),  0 ≤ r ≤ 1,
```

of which the transform keeps only the k = 0 term.  At the minimum-distance
scale the relative error is Θ(n^(−2/d)), which the second-order Q term does
not absorb.  In five dimensions at 500 points the transformed values are
visibly non-uniform (mean 0.526 over 20 000 clouds; KS p ≈ 10⁻³²), while
replacing the ball volume by H_d gives mean 0.501 and KS p = 0.62.  Two
dimensions at 8 000 points are unaffected in practice.

Close by deriving the finite-n law for the cube with H_d (and a re-derived
second-order term), or by defining a toroidal statistic with its own null
model, and calibrating at every shipped dimension and size.

### A2 — Q5 − Q4 is scored as a normal variable

[count_ones.rs](src/diehard/count_ones.rs), used by `count_ones_stream` and
the historical byte test.  z = (Q5 − Q4 − 2 500)/√5 000 treats an approximately
χ²(2 500) statistic as normal.  Over 250 000 null windows the p-values fell
below 0.01 in 1.04% (standard deviation 0.02%), with KS p = 0.003.  Score the
difference through the χ²(2 500) distribution, two-sided, and recalibrate.

### A3 — Some χ² tests drop cells instead of pooling them

[binary_rank.rs](src/diehard/binary_rank.rs) (`rank_test`),
[craps.rs](src/diehard/craps.rs) (throws) and [gcd.rs](src/dieharder/gcd.rs)
leave out cells expecting fewer than 5 counts and reduce the degrees of
freedom, so the retained counts do not have a fixed total.  At the shipped
sizes the omitted probability is tiny except for the GCD tables.  Pool the
weak cells with `math::chi_square_pooled` or `math::chi_square_pooled_tails`,
as bit distribution, squeeze, fill-tree and monobit2 do.

### A4 — Printed four-decimal tables limit the longest-run test at large n

[longest_run.rs](src/nist/longest_run.rs).  The M = 128 and M = 10 000 class
probabilities are SP 800-22's four-decimal values, up to 6.4·10⁻⁵ and
1.6·10⁻³ from the exact distribution.  A fixed error in the cell
probabilities adds a term growing with the number of blocks to Pearson's
statistic, so the test eventually rejects a correct source.  Use the exact
probabilities (the tests already compute them) outside a named
SP 800-22-compatibility mode, or bound n.

### A5 — The R spectral statistic is not a cumulative-periodogram test

[scripts/r_rng_tests.R](scripts/r_rng_tests.R), the KS test of periodogram
heights against Exp(1).  The statistic ignores frequency order and is
conservative on uniform input (3 of 2 000 null blocks below 0.01).  Replace it
with a cumulative-periodogram test and calibrate at the report's 0.001
threshold.

## Interfaces and robustness

### A6 — Invalid p-values can pass, and numerical failures look like skips

[result.rs](src/result.rs).  Every NaN becomes SKIP, though `igamc` also
returns NaN when an expansion fails, and any value ≥ α passes, including
infinity and values above 1.  Distinguish valid p-values, insufficient input,
unsupported parameters and numerical error; accept only finite values in
[0, 1]; make a verification run fail on numerical errors.

### A7 — `lagged_sums` overflows at the largest lag

[lagged_sums.rs](src/dieharder/lagged_sums.rs).  `lag + 1` is unchecked, so
`lagged_sums(&[], usize::MAX)` panics.  Use checked arithmetic and report
invalid parameters.

### A8 — Counters and corpus sizes have undocumented limits

[bit_distribution.rs](src/dieharder/bit_distribution.rs) keeps u32 histogram
counts that overflow at 2³² groups of one cell (32 GiB of constant input) and
allocates 65·2^w counters before checking sample size;
[scripts/r_rng_tests.R](scripts/r_rng_tests.R) cannot represent 2³¹ words.
Use wide or checked counters and reject oversized input before allocating.

### A9 — Default batteries see only the high half of 64-bit generators

`next_u32` for PCG64, Xoshiro256, SFC64 and Xorshift64 returns the upper half
of a 64-bit output and discards the lower half, so a defect confined to the
low bits is invisible to the 32-bit batteries.  Name the tested projection
and add lower-half, full-word and bit-reversed views.

## Build and release evidence

### A10 — Wiping still runs without the `cryptography` feature

[src/rng/os.rs](src/rng/os.rs).  `OsRng::drop` clears its buffer under
`not(feature = "cryptography")`, so a `default-features = false` build still
wipes.  Remove that path so wiping is enabled only with cryptography.

### A11 — Tests and scripts can run stale binaries

[tests/dump_rng.rs](tests/dump_rng.rs) and [tests/registry.rs](tests/registry.rs)
launch binaries that require the `cryptography` feature without being gated
on it, so after a default build a `--no-default-features` test run passes
against old executables and fails in an empty target directory.
[scripts/run_r_report.sh](scripts/run_r_report.sh) builds `dump_rng` only when
it is missing and ignores `CARGO_TARGET_DIR`;
[scripts/bench_rngs.sh](scripts/bench_rngs.sh) runs whatever `pilot_rng` it
finds and reuses cached rows keyed only by machine and generator.  Gate the
tests on their binaries' features, build before every scripted run, and key
cached results by executable identity; see SUGGESTIONS.md.  CI tests default
features only.
