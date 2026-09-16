# Entropy: corrections and algorithmic work — 2026-09-16

Applies to entropy `185d0c8`; findings and fresh experiments are in
[AUDIT.md](AUDIT.md). **None of the implementation fixes below has been made
by this review.** Resolved suggestions from September 11, including recovery
and restoration of historical DIEHARD, have been removed from the active list.
Preserve those sources, variants, reference data and provenance.

## Release requirements

Treat the following as open work. A green implementation-test run is not a
substitute for statistical calibration, and a historical-reference match does
not establish a calibrated null distribution. A new release must either fix
these defects or explicitly exclude the affected behavior from its calibrated
profile and describe the remaining limitations. Compatibility behavior must
never be silently substituted for a corrected statistic.

| Finding | Correction | Evidence required before closure |
|---|---|---|
| E1: cube boundary omitted | Define and implement the intended geometry and its finite-n null transform | Held-out null calibration at every shipped dimension and both quick/full sizes; finite-n approximation error recorded separately from Monte Carlo error |
| E2: R spectral KS | Implement a cumulative-periodogram statistic; distinguish it from histogram-of-heights diagnostics | Full script run with pinned package versions; null size and power checks at the actual 0.001 threshold and shipped input sizes |
| E3: monobit2 | Expected-count partitions, complete blocks, separate histograms, justified family inference | Marginal and joint null calibration; planted bit-bias and scale-specific alternatives; historical result still reproducible under its own name |
| E4: non-cryptographic wiping | Remove the non-cryptographic wipe path; require explicit cryptographic feature selection for wiping dependencies | Feature graphs for default/statistical/cryptographic consumers, including a consumer also linking rump; inspect both OsRng drop paths |
| E5: stale binary artifacts | Feature-gate CLI integration targets | Default and no-default builds tested in separate empty target directories; no missing-binary failures and no reuse of crypto-enabled executables |
| E6: omitted multinomial cells | Partition the entire support and preserve total observed/expected mass; correct fill-tree support and df together | The 2,828-word witness in AUDIT rejects for the intended pattern; boundary cases around each pooling cutoff; independent null simulation |
| E7: invalid-result handling | Typed numerical error/insufficient/unsupported outcomes; validated p-values | Inject NaN, infinities and out-of-range values; numerical errors cause a failed verification run, never PASS or an ordinary SKIP |
| E8: lag overflow | Checked parameter arithmetic before division | Empty/small input and boundary lag cases, in debug and release |
| E9: approximate class probabilities | Compute accurate longest-run probabilities or enforce justified sample-size limits | Independent recurrence/oracle comparison, normalized probabilities and a quantitative bound on accumulated table error at the maximum accepted input |
| E10: incomplete native-word coverage | Name the tested projection; add native-width and lane adapters | A generator with healthy upper bits and deliberately broken lower bits is detected in the lower/full views; stream advancement and bit order pinned |
| E11: counter and corpus-size limits | Wide/checked counts, explicit byte/sample/memory bounds, chunked input where applicable | Near-limit counter tests without huge allocations; an oversized corpus is rejected before allocation with its actual limit reported |

Run the existing release tests including ignored cases, docs, formatting,
clippy, MSRV and the supported platform matrix on the **candidate release**,
with recorded sibling SHAs and dependency lock resolution. The fresh review
ran release tests on macOS with current siblings; it did not rerun the whole
CI/MSRV/Linux matrix. Existing CI pins older siblings and lacks the
no-default-features configuration. Keep those limitations visible until
candidate-release evidence replaces them.

For calibration, choose sample sizes and tolerances before running the
validation seeds. At alpha=0.001, 2,000 replicates yield only two expected
rejections and cannot establish useful tail precision. For example, 100,000
replicates yield about 100; even that has roughly 20% relative 95% uncertainty.
Use simultaneous confidence bounds across the declared configuration grid,
a separate tuning corpus, and held-out streams from several generator
families plus OS bytes. Do not repeatedly rerun a failed calibration until
one seed passes. Report false-alarm rate, power and cost together. TestU01's
[two-level testing methodology](https://www.iro.umontreal.ca/~lecuyer/myftp/papers/testu01.pdf)
is the starting point, not a substitute for error bars at our parameter sizes.

## 1. Correct the geometry and buy power with an exact closest-pair algorithm

Two independent changes are needed: E1 corrects the null model; faster
geometry increases affordable sample size without changing the statistic.

For the ordinary cube metric, start from the exact pair probability H_d(r)
derived in AUDIT, and analyze the dependence error in the minimum-event
Poisson approximation. An inclusion-exclusion or Chen–Stein analysis should
track shared-point events and boundary terms separately. Do not keep the
old Q factor automatically after replacing the underlying pair volume: its
coefficient and model need to be re-derived. Handle r>1 explicitly; the short
polynomial in AUDIT applies only to r≤1. Respect the actual 32-bit coordinate
grid when bounding very small distances and ties.

If an analytic error bound is too loose at small n, a calibrated simulation
rank is a defensible alternative. For an independent null reference sample,
`(1 + number of null statistics >= observed)/(B+1)` gives a discrete rank
p-value under exchangeability. Its resolution is 1/(B+1); fitting a CDF on
the same validation samples is not equivalent. Record the reference seed,
generator, geometry and parameter tuple. Do not simulate the reference with
the generator being judged. A reference pool reused across tests also induces
dependence; retain that fact in family inference.

For computation, compare fixed-dimensional divide-and-conquer, a spatial
hierarchy with exact bounds, and a randomized incremental grid. The published
closest-pair literature offers subquadratic algorithms; see
[Wang, Yu, Gu and Shun (2021)](https://arxiv.org/abs/2010.02379), which also
compares static algorithms. Implement from the published description. A grid
can have expected linear work under the algorithm's hypotheses, but its
neighbor count grows with dimension and adversarial inputs need bounded
fallback behavior. Randomization used by the *search algorithm* must be
separate from the RNG being tested and must not alter its consumption.

Acceptance: exact minimum squared distances must agree with the existing
brute-force oracle on dimensions 2–5, duplicates, boundary points, coarse
grid ties, clusters and deliberately bad generators. Pruning bounds need to
be conservative under floating-point rounding. Measure point comparisons,
memory and wall time separately. Allocate the recovered work to more clouds,
then quantify power against controlled density distortions. Do not infer
that fewer operations alone improve the test's calibration.

## 2. Replace dense transforms and scans with exact structured algorithms

**DCT.** The even-extension identity supplies a simple first implementation:

```text
y[j] = x[j], y[2N-1-j] = x[j], 0 <= j < N
X[k] = Re(exp(-i*pi*k/(2N)) * FFT(y)[k]) / 2
```

This computes the unnormalized DCT-II already used here. Preserve the word
rotations, DC adjustment, argmax tie convention and final class counts.
Reuse a transform plan and work buffers. It replaces O(N²) work per block
with O(N log N), using the existing rustfft dependency. A real/N-point
reduction can follow once the simpler identity is verified. The published
foundation is [Makhoul (1980)](https://doi.org/10.1109/TASSP.1980.1163351).
The scratch experiment in AUDIT establishes feasibility on one 5,000-block
corpus; it does not prove agreement on ties or validate the argmax null law.

Orthogonal coefficients have equal variance under the intended normalization,
but non-Gaussian input does not automatically make all coefficient argmax
positions exchangeable. Validate the class probabilities as well as the FFT
identity. Budget saved by the transform should support larger block counts
or multiple block lengths only after their null laws are checked.

**Pattern histograms.** For each group of 64 values, count only touched
patterns. Initialize each pattern's zero bin to the number of groups; on a
visit with multiplicity c, decrement that pattern's zero bin and increment
its c bin. Each group touches at most min(64,2^w) patterns. This changes the
dense histogram-update work from O(T·2^w) to O(64T+2^w), apart from reading
bits and outputting results. Sparse storage can remove the 65·2^w table
when only a small fraction of patterns occur. Expected-count pooling must
still satisfy E6. Compare every count and p-value with the current algorithm
before making a performance claim. Reject statistically uninformative
parameter choices before allocating enormous tables.

**Probability tables.** Derive longest-run class probabilities through a
finite automaton whose state is the current suffix of ones, absorbing when
the requested run length is reached. Matrix powers or a short recurrence
give `P(longest run < r)`; differences give the class probabilities. Compute
with sufficient precision and record a rounding bound. The repository's
existing exact-distribution test provides an initial independent comparison.
For fill-tree, derive its collision-time law from the bounded insertion
process rather than treating rounded empirical targetData as an exact law
at arbitrary replication counts. Its historical table remains a compatibility
artifact while this derivation is checked.

## 3. Add a sequential test with a proved error guarantee

This is a research direction with a concrete mathematical contract. It is
not a claim of a new theorem or an implemented improvement.

On a canonical bit stream, let a predictor assign q_t to bit 1 using only
past bits. Define

```text
E_0 = 1
E_t = E_(t-1) * 2 * q_t^x_t * (1-q_t)^(1-x_t).
```

Under independent fair bits the next multiplier has conditional expectation
one. E is therefore a nonnegative martingale. A fixed convex mixture of such
predictors also has this property; Ville's inequality bounds the chance it
ever exceeds 1/alpha by alpha. This permits continued testing and stopping
on evidence without repeatedly spending the nominal error rate.
See [Howard et al. (2020)](https://arxiv.org/abs/1808.03204).

Begin with a beta-binomial bias predictor and short Markov contexts; then
consider the published [context-tree weighting method of Willems, Shtarkov
and Tjalkens (1995)](https://research.tue.nl/en/publications/the-context-tree-weighting-method-basic-properties/).
A single mixture can detect bias, lag structure and changing context depth
while avoiding hundreds of nominally independent p-values. Models must
predict before seeing the tested bit. Specify depth/memory bounds, update
cost, reset handling and log-domain mixture arithmetic. Choosing a mixture
weight after seeing its final performance invalidates this simple guarantee.

Evaluate bytes and CPU time needed to detect weak planted bias, sparse
corruption, short-period structure and delayed dependence, alongside the
existing NIST/Dieharder tests at a common false-alarm budget. Include streams
these predictors miss. E-values are not uniform p-values and must not be
fed into the current KS-of-p-values machinery. Keep the established tests:
this adds a different, justified form of evidence, not a cryptographic
security certificate or a universal detector.

## 4. Make evidence reproducible and dependence explicit

Complete the finite-corpus work in this repository. Introduce a fallible
input interface or an explicit preflight budget plus exhaustion outcome;
never cycle or zero-pad a short corpus. Specify little-endian words,
LSB/MSB bit extraction for each test, partial-word policy, sequential versus
rewind mode, offsets consumed, total byte budget and corpus SHA-256.
Variable-consumption tests need an actual cursor and an exhaustion result,
not an average-cost estimate. Verify parity with a generated stream across
buffer boundaries and short-input cases.

Cover high and low lanes, full native words and bit reversal explicitly (E10).
Do not choose the favorable half of a generator and summarize that as all-bit
coverage. Bound the family error across these additional views as well.

Record generator parameters/seed where replayable, suite order, the exact
statistic variant, sample sizes, raw statistic, degrees of freedom,
unrounded p or log-p, and typed status. Keep each template/pattern/window
identity as structured data. A test selector should select work, or clearly
state that it is only an output filter; record enough input offsets that
changing selection does not make comparisons unknowable.

For a fixed battery, valid marginal p-values can be combined with Holm or
Bonferroni without independence assumptions. More powerful joint calibration
must reproduce the complete dependence structure and be validated on a
separate null corpus. Never multiply, Fisher-combine, or Šidák-correct
shared-input outputs on an unproved independence assumption. Correlation
does not change the sum of expected rejection indicators, but does change
the evidence represented by a cluster of failures.

Expose three distinct kinds of output: reference-compatible statistics,
validated calibrated tests within stated parameter ranges, and research
experiments. Preserve the historical sources in all three cases. Existing
AD extreme-tail interpolation and reduced replication counts also need
explicit accuracy/power limits when making stronger claims than the current
ones; the old audit's large simulations are historical evidence, not fresh
measurements from this review.

Still-open external checks: SFC64 needs independently sourced reference
vectors; the FPF truncation comparison needs the identified PractRand
revision or a separately derived, honestly named specification; the exact
NIST §2.9.8 input remains unavailable. A faithful-reference claim stays
unverified until its oracle and consumption contract are reproducible.

The preferred order is: close E4/E5/E7/E8 and create the result/replay
infrastructure; correct and validate E1/E2/E3/E6/E9; replace the expensive
kernels; then evaluate the sequential research suite. Fixing the small
boundary defects alone does not make the statistical release ready.
