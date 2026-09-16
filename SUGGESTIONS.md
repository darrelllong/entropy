# Suggestions

Improvements beyond the open defects in [AUDIT.md](AUDIT.md).

## Release verification

- Verify a release candidate from a clean checkout with an empty
  `CARGO_TARGET_DIR` for each feature set, toolchain and platform:

  ```bash
  set -euo pipefail
  root=$(mktemp -d "${TMPDIR:-/tmp}/entropy-verify.XXXXXX")
  CARGO_TARGET_DIR="$root/default" cargo test --locked --release -- --include-ignored
  CARGO_TARGET_DIR="$root/no-default" cargo test --locked --release --no-default-features -- --include-ignored
  ```

- Make every report and benchmark script build its binary first, stop on a
  failed build, and pass the resolved executable path to subprocesses.
- Record with every report the source revision, sibling revisions, lockfile
  hash, toolchain, features and executable SHA-256, and key cached benchmark
  rows by that identity.
- Add the no-default-features configuration to CI.

## Calibration practice

- Choose sample sizes and tolerances before running validation seeds.  At
  α = 0.001, 100 000 null replicates give about 100 expected rejections, still
  roughly ±20% at 95% confidence.
- Validate on held-out streams from several generator families, and report
  false-alarm rate, power and cost together.
- Combine dependent results with Holm or Bonferroni bounds; never multiply,
  Fisher-combine or Šidák-correct p-values that share input.

## Faster exact algorithms

- **Closest pair.**  The minimum-distance tests visit every pair,
  100 × C(8 000, 2) ≈ 3.2·10⁹ distances per battery.  A grid or
  divide-and-conquer closest-pair algorithm gives the same minimum in far less
  time, which could buy more repetitions.  Check it against the brute-force
  scan on duplicates, boundary points and coarse grids.
- **DCT.**  The DCT test performs 5 000 × 256² coefficient products.  With the
  even extension y of a block, X[k] = Re(e^(−iπk/(2N))·FFT(y)[k])/2 gives the
  same unnormalised DCT-II in O(N log N) (J. Makhoul, *IEEE Trans. ASSP* 28,
  1980).  Preserve the rotations, DC adjustment and argmax tie rule, and check
  every block's argmax against the direct transform.
- **Sparse pattern histograms.**  Bit distribution clears and scans all 2^w
  patterns for each group of 64 values, of which at most 64 occur.  Start each
  pattern's zero cell at the number of groups and update only touched
  patterns, removing the 2^w factor.

## A sequential test with an error guarantee

A predictor q_t for the next bit, built only from past bits, defines
E_t = E_{t−1}·2·q_t^{x_t}(1 − q_t)^{1−x_t}, a nonnegative martingale under fair
independent bits; by Ville's inequality it exceeds 1/α with probability at
most α, however long the test runs (S. R. Howard et al., *Annals of
Statistics* 2021).  Mixtures of Markov and context-tree predictors (Willems,
Shtarkov and Tjalkens 1995) could detect bias, lag structure and short periods
with one guarantee instead of hundreds of p-values.  E-values are not uniform
p-values and must not be fed into the KS machinery.

## Reproducibility

- A finite-corpus input with an explicit byte budget, exhaustion outcome and
  consumption record, instead of the infallible `Rng::next_u32`.
- A selection flag that selects work rather than filtering output, so that
  choosing tests does not change which input later tests read.
- Structured output with each result's statistic, degrees of freedom,
  unrounded p-value and input offsets.
