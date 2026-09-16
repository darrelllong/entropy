# Entropy audit — 2026-09-16

Reviewed `185d0c869d2a143fe757f26154ed6748044100c8`, with cryptography
`601c97bbd9049a4a2ba02e3b37d113f00d063e66` and rump
`0faea340caae1ec3088fc2f64e68b97422c26dde`. All three working trees were clean
when frozen. This is a review, with experiments in a separate scratch tree;
only this document and SUGGESTIONS.md change in the repository.

**The main remaining problem is statistical validity, not reference fidelity.**
The 5-D distance transform omits the boundary of the domain actually sampled.
The R spectral KS statistic is substantially conservative on uniform input.
Monobit2's previously acknowledged calibration defect remains in the default
battery. Passing reference vectors and implementation tests does not close
these issues. There are also substantial opportunities to replace exhaustive
work with exact, faster algorithms; see [SUGGESTIONS.md](SUGGESTIONS.md).

## Current findings

### E1 — P1: minimum-distance p-values use the wrong finite-domain model

Evidence: [minimum_distance_nd.rs](src/dieharder/minimum_distance_nd.rs),
lines 76–100; [nearest_pair.rs](src/diehard/nearest_pair.rs), lines 24–40.
The sampler draws points in the unit cube and measures ordinary Euclidean
distance, without periodic wrapping. The transform uses the volume of a
whole d-ball. Near a face, edge or corner, some of that ball lies outside
the sampling domain. Matching an upstream formula does not remove this error.

For independent continuous uniform points U,V in the unit d-cube and 0 ≤ r ≤ 1,
the exact **single-pair** probability is

```text
H_d(r) = P(||U-V|| ≤ r)
       = integral_{||z||≤r} product_i (1-|z_i|) dz
       = sum_{k=0}^d (-1)^k binom(d,k)
           π^((d-k)/2) r^(d+k) / Γ(1+(d+k)/2).
```

This follows by expanding the product of the triangular coordinate-difference
densities and integrating monomials over the ball. For d=1 it gives 2r−r²;
for d=2, πr²−(8/3)r³+r⁴/2. The implementation keeps only the k=0 term.
At minimum-distance scale r=Θ(n^(-2/d)), the relative boundary correction
is Θ(n^(-2/d)); its Q correction is only O(1/n). In five dimensions those
orders differ substantially. The existing Q correction cannot stand in for
the missing boundary term.

Fresh experiments used the production nearest-pair routine and checked the
reconstructed transform against the public test. Each quick experiment had
1,000 batteries × 20 clouds × 500 points, with separate continuous streams
from the named generators. These are two different generator designs, not
proofs that their streams are mathematically independent random variables.

| Stream | Transform | Mean of 20,000 transformed values | Fraction below 0.5 | KS uniformity p | Batteries below 0.01 |
|---|---|---:|---:|---:|---:|
| MT19937, seed 20260916 | Current | 0.526144 | 0.46365 | 8.33e-33 | 16/1,000 |
| PCG64, state 20260916, stream 53 | Current | 0.523818 | 0.46895 | 5.54e-25 | 15/1,000 |
| Same MT clouds | Pair-probability Poisson approximation | 0.501012 | 0.49525 | 0.624 | 10/1,000 |
| Same PCG clouds | Pair-probability Poisson approximation | 0.498625 | 0.50095 | 0.814 | 9/1,000 |

The comparison replaces the transform by `-expm1(-n*(n-1)*H_d(r)/2)`.
**This is an experimental approximation to the minimum distribution**, not
an exact solution: pair events sharing a point are dependent. It fixes the
leading geometric error in these samples. It still needs finite-n error
analysis and held-out calibration. A d=2 control, MT seed 20260917, gave a
current-transform mean of 0.502677 and KS p=0.139 over 20,000 clouds.

The quick-mode outer rejection counts alone are not strong evidence of a
1% size violation: their individual 95% binomial intervals include 1%.
The strong evidence is the distortion of the inner distribution that the
outer KS test assumes uniform. Do not turn the small outer experiment into
an exaggerated false-alarm claim.

A separate full-size run used MT seed 20260921, 100 batteries × 100 clouds
× 8,000 points. Current and candidate inner means were 0.505631 and
0.497167; their KS p-values were 0.0760 and 0.381. Outer 1% rejections were
0/100 and 1/100. This smaller full-size experiment does not establish a
full-mode size violation or certify the replacement. The geometric mismatch
is present in both modes; the strong measured distortion above is quick-mode.

Resolution: retain the historical transform as an identified compatibility
profile; implement and calibrate a cube-aware profile, or define a separately
named toroidal statistic with a null model for that metric. Merely changing
the distance to wrap would change the statistic. Minimum-interpoint Poisson
limits are asymptotic results, not finite-cube identities; see
[Kanagawa, Mochizuki and Tanaka (1992)](https://www.ism.ac.jp/editsec/aism/44/9.html).

### E2 — P1: entropy's R spectral KS statistic remains miscalibrated

Evidence: [scripts/r_rng_tests.R](scripts/r_rng_tests.R), lines 217–286,
especially line 269. It applies an ordinary one-sample KS test to the
**unordered distribution of periodogram heights** against Exp(1). Despite
the nearby comment, this is not a cumulative-periodogram test. Reordering
frequency bins leaves this statistic unchanged, so it also discards where
spectral energy occurs.

Individual periodogram heights have an asymptotic exponential law. That
alone does not establish the iid empirical-process law assumed by this KS
test when all the ordinates come from the same non-Gaussian sequence.

A base-R reproduction of the current formulas used 2,000 independent
`runif` blocks, seed 20260916, N=32,768, quantized to the same 32-bit uniform
grid as the script. For the comparison, normalize the periodogram by its
sum, cumulatively sum it **in frequency order**, exclude the terminal 1,
and compare those cumulative values with uniform order statistics.

| Statistic | Rejections at 1% | Rejections at 5% | Mean p |
|---|---:|---:|---:|
| Current ordinate KS against Exp(1) | 3/2,000 | 33/2,000 | 0.58418 |
| Current 10-bin ordinate chi-square | 12/2,000 | 85/2,000 | 0.53886 |
| Cumulative-periodogram comparison | 19/2,000 | 97/2,000 | 0.49652 |
| Current maximum-spike formula | 20/2,000 | 94/2,000 | 0.49861 |

For the current KS test, the 95% exact interval for the 1% rejection rate is
[0.031%, 0.438%]; for the cumulative comparison it is [0.573%, 1.480%].
The current test loses sensitivity. This experiment does **not** establish
calibration at the script's 0.001 reporting threshold: only two rejections
per statistic would be expected in 2,000 trials. Nor does it certify every
input length or the chi-square and maximum-spike approximations.

The R packages needed by the complete script were not installed locally;
these were isolated formula-level experiments using base R, not a claimed
end-to-end package run. The complete reproducer is below. The spectral
method and its asymptotic qualification are described in
[Stata's Bartlett-test methods](https://www.stata.com/manuals/tswntestb.pdf).

The gap-test tail pooling **is already present** here. A separate diagnostic
using the official CRAN randtoolbox 2.0.5 function and this script's pooling
rule gave 25/2,000 rejections at 1%, 3/2,000 at 0.1%, N=32,768, seed 20260920.
Those counts do not substantiate the old claim of catastrophic gap-test
miscalibration for this revised script. They are also insufficient to certify
its tail. Do not transplant old cryptography-battery conclusions to entropy
without testing entropy's current statistic.

### E3 — P2: known-invalid monobit2 calibration is still a default verdict

Evidence: [monobit2.rs](src/dieharder/monobit2.rs), lines 55–67 and 152–196;
[dieharder/mod.rs](src/dieharder/mod.rs), line 45.
The code correctly discloses the problem: chi-square cells are selected
using **observed counts > 10**, then a conventional chi-square reference
law is used. Subsequent levels reuse the same words, and the minimum of
folded p-values is Šidák-corrected without an independence justification.
The latter cannot repair invalid marginal p-values.

Fresh 20,000-trial experiments at 2,000 words per trial:

| Stream | p < 0.01 | Rejection rate | 95% exact interval |
|---|---:|---:|---:|
| MT19937 seed 20260919 | 247 | 1.235% | [1.087%, 1.398%] |
| PCG64 state 20260919, stream 53 | 269 | 1.345% | [1.190%, 1.514%] |

This confirms a previously documented, still-open statistical defect; it is
not a newly discovered porting mistake. Preserve the historical statistic,
but do not present it as a calibrated default test. A corrected profile
needs expected-count pooling, complete blocks with disjoint histograms,
and dependence-aware family inference. Fixing only the shared histogram
cell will not address the demonstrated null defect.

### E4 — P2: wiping is still performed without cryptography enabled

Evidence: [src/rng/os.rs](src/rng/os.rs), lines 92–107, and
[Cargo.toml](Cargo.toml), feature/dependency declarations.
`OsRng::drop` calls `buf.fill(0)` and `black_box` under
`not(feature = "cryptography")`. Consequently even an explicit
`default-features = false` build attempts to wipe its buffer. This conflicts
with the owner's requirement that wiping be specifically enabled for
cryptography. The default `cryptography` feature also automatically pulls in
cryptography-rs, which enables rump's `wipe` feature.

The useful dependency-isolation change in ad81b03 is real: a statistical-only
consumer can now avoid cryptographic multiprecision and its limb wiping.
The remaining 256-byte OsRng clear does **not** re-enable rump wiping, and
no material factoring slowdown is attributed to that small clear here.
Remove the non-cryptographic wipe path and make the intended opt-in contract
explicit across the default library features and cryptographic binaries.

### E5 — P2: feature-disabled integration tests falsely pass with stale binaries

Evidence: [tests/dump_rng.rs](tests/dump_rng.rs), line 20;
[tests/registry.rs](tests/registry.rs), lines 38–49; binary `required-features`
in [Cargo.toml](Cargo.toml). Neither integration test is feature-gated.

After a default-feature build, the complete no-default-features test command
reported **325 passed**, because these tests launched old `dump_rng` and
`pilot_rng` binaries left in the target directory. In an empty target
directory, the same two integration targets fail **all 11 tests**, attempting
to spawn binaries that Cargo correctly did not build.

Reproducer, using a fresh directory each time:

```sh
audit_target=$(mktemp -d)
CARGO_TARGET_DIR="$audit_target" cargo test --release --no-default-features \
  --test dump_rng --test registry --no-fail-fast
```

Gate these tests on the same feature as their binaries. Add the statistical
library configuration to CI and test it in a separate target directory.
The existing CI matrix covers default features only and pins older sibling
revisions, so its green result is not evidence that the new isolation path
is tested. The no-default-features **library** tests do pass: 279/279.

### E6 — P2: incomplete multinomial tables are still scored as complete ones

Evidence: [math::vtest_pvalue](src/math.rs), lines 808–854, and
[fill_tree.rs](src/dieharder/fill_tree.rs), lines 152–165.
Vtest groups small expected cells but **drops their pooled contribution**
when it remains below the cutoff. It then uses df = retained cells − 1.
The retained observed counts do not have a fixed sum, so this is not the
usual complete-multinomial Pearson statistic. In the asymptotic multinomial
model, retaining k cells with total mass S gives a quadratic form with
k−1 unit eigenvalues and one eigenvalue 1−S, not simply χ²_(k−1).
The omitted term is small only when the omitted probability is small.

A concrete public-API example uses 202 blocks of 64 seven-bit values:
100 blocks contain no zeros, 50 contain one zero, 52 contain ten zeros;
all other values equal one. Encode each value MSB-first into 2,828 u32
words. For width 7, pattern 0, the expected histogram masses for counts
0, 1 and ≥2 are 122.278880, 61.620853 and 18.100267. At the configured
cutoff of 20, the last group is discarded. The current result is **PASS,
p=0.012415**, χ²=6.2507, df=1. Keeping all three cells gives χ²=69.7410,
df=2, p=7.18e-16. This is one pattern's verdict; other patterns can reject
the same deliberately defective stream. It is not a whole-battery false pass.

For a calibrated variant, merge a too-small residual pool into a retained
cell rather than silently dropping it. In this example a three-cell test
already satisfies the usual expected-count ≥5 condition; if retaining the
stricter cutoff of 20, merge the tail into another cell. Include every
observation exactly once and derive df from the resulting partition.

Fill-tree likewise retains the historical exclusive upper endpoint and
omits a cell with expected count 23.52 at the current sample size. Its
previous audit removed an unreachable bailout, **not this statistical
problem**. Its retained cells do not exhaust the distribution. Move the
historical calculation into the compatibility profile and derive a complete
partition for the calibrated profile. Do not merely include an extra cell
while leaving df unchanged. No fresh fill-tree size estimate is claimed here.

### E7 — P2: numerical errors can become SKIP, and invalid p-values can PASS

Evidence: [result.rs](src/result.rs), lines 26–75;
[math.rs](src/math.rs), lines 234–251. Every NaN becomes SKIP, although
`igamc` explicitly also returns NaN for calculation failure, not just
insufficient data. Constructors and the public fields accept arbitrary
floating-point values; `TestResult::new("audit", f64::INFINITY).passed()`
returns true, confirmed in a release diagnostic. Values greater than one
also pass. This is an interface defect, not evidence that ordinary default
inputs currently cause an infinite p-value.

Use separate outcomes for a valid p-value, insufficient input, unsupported
parameters and numerical error. Only finite values in [0,1] may be tested
against alpha. Preserve numerical failures in machine-readable output and
make a release verification run fail on them; they must not disappear into
a benign skip count. Include a policy for significant underflow, with
log-survival values where the model supports them.

### E8 — P2: maximal public lag panics even on empty input

Evidence: [lagged_sums.rs](src/dieharder/lagged_sums.rs), lines 33–35.
`lag + 1` is unchecked and precedes the sample-size guard.
`lagged_sums(&[], usize::MAX)` overflows in a checked build and divides by
zero after wrapping in release; the release panic was reproduced. Use
`checked_add` and return an explicit invalid-parameter outcome. Exercise
0, 1, len, len+1 and usize::MAX at the API boundary. The stock CLI's fixed
lag does not reach this defect.

### E9 — P2: reference probability tables impose an unbounded scaling error

Evidence: [longest_run.rs](src/nist/longest_run.rs), lines 27–44, and its
exact-distribution unit test. M=128 and M=10,000 still use the printed
four-decimal tables. The existing exact recurrence check confirms maximum
absolute discrepancies up to 6.4e-5 and 1.6e-3 respectively. M=10,000
matches STS's table; reference agreement does not make that table exact.

With exact class probabilities p and a fixed approximate table q, Pearson's
statistic develops an additional term of order
`N * sum_i((p_i-q_i)^2/q_i)` as the number N of independent blocks grows.
Thus a fixed approximation eventually rejects the correct source. Derive
high-precision probabilities from the longest-run finite-state recurrence,
or bound N for the approximation and report that restriction. Keep the
rounded table only in a named standard-compatibility profile. This review
has not measured an excess rejection rate at the default 16-million-bit
input; the defect concerns unrestricted scaling and precision claims.

### E10 — P2: default tests omit half of native 64-bit outputs

Evidence: `Rng::collect_u32s`/`collect_bits`/`next_f64` in
[src/rng/mod.rs](src/rng/mod.rs), and `next_u32` in
[pcg.rs](src/rng/pcg.rs), [xoshiro.rs](src/rng/xoshiro.rs),
[sfc.rs](src/rng/sfc.rs) and [xorshift.rs](src/rng/xorshift.rs).
For PCG64, Xoshiro256, SFC64 and Xorshift64, `next_u32` advances the native
state and returns the upper half. The omitted lower half is not buffered for
the next call. Thus the default word/bit/float batteries judge a selected
32-bit projection, not the complete 64-bit output stream. A defect confined
to the discarded half can be invisible to these tests.

This is a coverage limitation, not an arithmetic bug in that projection.
Name the tested view and add explicit lower-half, upper-half, full-word
serialization and bit-reversal adapters with consumption records. A result
for one view must not be presented as validating all native bits. The
`next_f64` comment additionally says the *upper* half is discarded and that
floats are used only for p-value lookup; both claims are contradicted by
the implementations and the geometric samplers.

### E11 — P2: sample-count types impose undocumented large-input limits

Evidence: [bit_distribution.rs](src/dieharder/bit_distribution.rs), lines
59–75, and [r_rng_tests.R](scripts/r_rng_tests.R), lines 42–45.
The bit-pattern histograms store u32 counts while the number of input groups
is usize. On a 64-bit host, width 1 with 2^33 constant u32 words produces
2^32 groups in one histogram cell: incrementing it overflows in a checked
build or wraps in release. This input is 32 GiB before working storage.
There is no group-count bound before processing. This is an arithmetic
bound established from the code, not a claimed 32-GiB execution.

The R script converts file-size/4 to an R integer; at 2^31 words (8 GiB)
that conversion cannot represent the length and produces NA. It also
materializes multiple complete copies of the corpus in raw and numeric
forms. Neither implementation advertises or enforces a useful input-memory
contract before expensive work. Use wide/checked counters and explicit
preflight limits; streaming statistics should consume chunks where possible.
An FFT can retain its separately declared finite buffer requirement. Test
counter boundaries through a bounded diagnostic, not by allocating tens of
GiB in ordinary CI. Default-size inputs are below these limits.

## Algorithmic limitations verified in the current implementation

These are opportunities to recover statistical power per unit work, not
claims that slow computation alone produces an incorrect result.

- **All-pairs distance search.** `nearest_pair.rs` visits every pair. The
  full ND test needs 100 × binom(8,000,2) = **3,199,600,000** pair distances.
  Moving sqrt outside the pair loop was useful but leaves quadratic work.
  Exact closest-pair algorithms can preserve the statistic while supporting
  more replications or larger clouds. The current source itself notes that
  100 replications have limited power to detect modest density changes.
- **Quadratic DCT.** `dct.rs` performs 5,000 × 256² = **327,680,000**
  coefficient products. An independently written scratch prototype used a
  512-point FFT of the even extension. On MT19937 seed 20260922, every one
  of 5,000 argmax indices matched the direct transform; the final p-values
  were identical, **0.53419911835299028**. Maximum coefficient error,
  normalized by 256·(2³²−1), was 2.64e-15 or less. One concurrent local run
  took 0.112 s for production and 0.008 s for the FFT kernels; those timings
  exclude different setup costs and are **not** an end-to-end speedup claim.
  Ties, structured inputs and other hosts still require differential checks.
- **Dense scans for sparse pattern occupancy.** `bit_distribution.rs`,
  lines 59–75, clears and scans all 2^w patterns for every group of only
  64 w-bit values. At most 64 patterns can occur in a group. Recording
  touched patterns and accounting for the zero bin implicitly can remove
  the 2^w factor from this inner loop, exactly. Public width 20 currently
  allocates 65·2²⁰ u32 histogram entries (260 MiB), plus other buffers, even
  with very little input. Statistical sample-size gates should precede
  large allocations; a sparse representation can avoid those dense tables.

## Reporting and remaining scope

The finite-corpus adapter requested in the previous suggestions is still
absent. `Rng::next_u32` is infallible, so it cannot naturally express corpus
exhaustion. `run_one` advances the same generator through selected suites;
changing suite selection changes later input. A `--test` filter limits
reported results after the selected suite has run. These contracts need
explicit replay and consumption records before cross-language disagreements
can be diagnosed reliably.

The CLI reports correlated slot counts, not a calibrated battery-level
verdict. Its comment around `src/main.rs:864` incorrectly suggests that
correlation changes the expected count nα. For calibrated marginals,
linearity of expectation gives nα regardless of dependence; dependence
changes the count's distribution. Conversely, a whole family failing
together need not be stronger evidence if its members are strongly
correlated. The 510 bit-pattern outputs should retain their identities, but
should not be treated as 510 independent confirmations.

The old outstanding SFC64 external-vector check, PractRand FPF truncation
oracle, and missing NIST §2.9.8 input were not resolved by this review.
This review does not certify every generator or every research test, all
sample sizes, the MSRV/platform matrix, or cryptographic security. It focused
on null models, numerical/statistical interfaces, scaling, feature isolation,
and the current review's open claims. No full all-generator battery was run.

## Verification and preservation

Fresh release verification on Darwin arm64, rustc 1.93.1, in the frozen tree:

| Check | Result |
|---|---|
| `cargo test --release -j 2 -- --include-ignored --test-threads=2` | 430 passed, including doc tests |
| `cargo test --release --no-default-features --lib -j 2` | 279 passed |
| Complete no-default-features run after the default build | 325 passed; contaminated by old binary artifacts, see E5 |
| Both CLI integration targets, no default features, empty target directory | 11 failed with missing binaries; E5 reproduced |
| Geometry, monobit2, spectral and DCT diagnostics | Results above; no implementation changes |

Resolved items from the long September 10–11 review have been removed from
the active issue list. Their evidence and disclosures remain in Git:
`git show 185d0c8:AUDIT.md`. In particular, do not reopen the fixed DRBG
buffer bounds, .NET debug overflow, PCG step ordering, NIST linear-complexity
sign, or historical-suite restoration without new evidence.

The historical DIEHARD source recovery and inventory are **complete**, not
pending suggestions. `--suite diehard-historical` exists and remains opt-in.
README records the selected variants; notably OPERM5 uses the corrected
Dieharder covariance table rather than silently calling it the original
Marsaglia table. All archives, source inventories, citations and historical
implementations were left untouched. Fresh SHA-256 checks:

| Archive | SHA-256 |
|---|---|
| `pubs/diehard-fortran-1996.tar.gz` | `99666a65c58b39802d8fabf545341c835a2746f41bf0d130e292805053eb3fe1` |
| `pubs/diehard-f2c-source-1996.tar.gz` | `e68b14abec617f459ee3f3232d0eca957444093f9ac038abef880f15329b9750` |
| `pubs/diehard-c-wang-1998.tar.gz` | `e028a755c1441e5af90f060cd5d8fceaa5967a6a66fbc1f4c66410be712208ae` |

Origins remain in [pubs/SOURCES.tsv](pubs/SOURCES.tsv). `pubs/Diehard.zip`
is the separate DOS distribution. These archives establish what is now
preserved, not the identity of the owner's particular lost copy.

## Reproduction record

Local scratch directory:
`/var/folders/gb/w1nlpxrn08g9p44vcrxm2tlm0000gn/T/entropy-review-20260916-kdmnfc5m`.
It contains the three snapshots, per-file SHA-256 manifest, prior review
copies, build logs, diagnostics and R p-value CSVs. This temporary path is
not a durable public artifact; the main calibration programs are embedded
below so the review does not depend on retaining it.

The only newly downloaded external source was the **official CRAN**
[randtoolbox 2.0.5 distribution](https://cran.r-project.org/src/contrib/randtoolbox_2.0.5.tar.gz),
SHA-256 `ab6eb953feb928e205a1d21dd577bef2b71d2108a61c4ce5e3879d7b53fed61f`,
used as a gap-test oracle in scratch. It was not installed or added to this
repository. No CADO code was consulted or copied.

### Executable diagnostic recipes

Use a disposable copy with the reviewed sibling revisions. The Rust examples
below add diagnostic files only to that copy, then run with
`cargo run --release --no-default-features --example NAME`.
They are independent review programs, not added production tests.

<details>
<summary>Geometry and monobit2: examples/audit_calibration.rs</summary>

```rust
use entropy::{rng::{Rng,Mt19937,Pcg64},math::{ks_test,lgamma},dieharder::{minimum_distance_nd::minimum_distance_nd,monobit2::monobit2}};
#[path="../src/diehard/nearest_pair.rs"] mod nearest_pair;
fn pair_cdf(r:f64,d:usize)->f64 {
    let mut choose=1.0; let mut s=0.0;
    for k in 0..=d {
        let term=choose*std::f64::consts::PI.powf((d-k) as f64/2.0)*r.powi((d+k) as i32)/(lgamma(1.0+(d+k) as f64/2.0).exp());
        s+=if k%2==0 {term} else {-term};
        choose*= (d-k) as f64/(k+1) as f64;
    } s
}
fn cloud<const D:usize>(rng:&mut impl Rng,n:usize)->(f64,f64) {
    let points:Vec<[f64;D]>=(0..n).map(|_|std::array::from_fn(|_|rng.next_f64())).collect();
    let r=nearest_pair::min_squared_distance(&points).sqrt();
    let v=match D {2=>std::f64::consts::PI*r.powi(2),3=>4.0*std::f64::consts::PI*r.powi(3)/3.0,4=>std::f64::consts::PI.powi(2)*r.powi(4)/2.0,5=>8.0*std::f64::consts::PI.powi(2)*r.powi(5)/15.0,_=>unreachable!()};
    let nf=n as f64;let q=[0.0,0.0,0.4135,0.5312,0.6202,1.3789][D];
    let old=(1.0-(-nf*(nf-1.0)*v/2.0).exp()*(1.0+(2.0+q)/6.0*nf.powi(3)*v.powi(2))).clamp(1e-15,1.0-1e-15);
    let proposed=-(-nf*(nf-1.0)*pair_cdf(r,D)/2.0).exp_m1();
    (old,proposed)
}
fn geometry<const D:usize>(name:&str,rng:&mut impl Rng,trials:usize,n:usize,repeats:usize) {
    let start=std::time::Instant::now(); let(mut all_old,mut all_new)=(Vec::new(),Vec::new());let(mut outer_old,mut outer_new)=(Vec::new(),Vec::new());
    for _ in 0..trials {let(mut a,mut b)=(Vec::new(),Vec::new());for _ in 0..repeats {let(x,y)=cloud::<D>(rng,n);a.push(x);b.push(y);all_old.push(x);all_new.push(y);}outer_old.push(ks_test(&mut a));outer_new.push(ks_test(&mut b));}
    for (label,raw,out) in [("current",&mut all_old,&outer_old),("pair-Poisson",&mut all_new,&outer_new)] {
        let mean=raw.iter().sum::<f64>()/raw.len() as f64;let frac=raw.iter().filter(|&&x|x<0.5).count() as f64/raw.len() as f64;
        println!("geometry,{name},d={D},n={n},trials={trials},repeats={repeats},{label},raw_mean={mean:.8},raw_cdf_half={frac:.8},raw_KS={:.8e},outer_below_01={},outer_below_05={},secs={:.3}",ks_test(raw),out.iter().filter(|&&x|x<0.01).count(),out.iter().filter(|&&x|x<0.05).count(),start.elapsed().as_secs_f64());
    }
}
fn mono(name:&str,rng:&mut impl Rng,trials:usize,n:usize) {let mut ps=Vec::new();for _ in 0..trials {let ws:Vec<u32>=(0..n).map(|_|rng.next_u32()).collect();ps.push(monobit2(&ws).p_value);}println!("monobit2,{name},trials={trials},words={n},below_01={},below_05={},mean={:.8},KS={:.8e}",ps.iter().filter(|&&x|x<0.01).count(),ps.iter().filter(|&&x|x<0.05).count(),ps.iter().sum::<f64>()/trials as f64,ks_test(&mut ps));}
fn main() {
    let mut a=Mt19937::new(91721);let mut raw=Vec::new();for _ in 0..20 {raw.push(cloud::<5>(&mut a,500).0);} let ours=ks_test(&mut raw);
    let actual=minimum_distance_nd(&mut Mt19937::new(91721),5,true).p_value;
    assert!((ours-actual).abs()<1e-12,"{ours} {actual}"); println!("production_transform_check={ours:.17},{actual:.17}");
    geometry::<5>("MT19937-seed-20260916",&mut Mt19937::new(20260916),1000,500,20);
    geometry::<5>("PCG64-state-20260916-stream-53",&mut Pcg64::new(20260916,53),1000,500,20);
    geometry::<2>("MT19937-seed-20260917",&mut Mt19937::new(20260917),1000,500,20);
    geometry::<5>("MT19937-seed-20260918-full-exploratory",&mut Mt19937::new(20260918),3,8000,100);
    mono("MT19937-seed-20260919",&mut Mt19937::new(20260919),20000,2000);
    mono("PCG64-state-20260919-stream-53",&mut Pcg64::new(20260919,53),20000,2000);
}
```

For the separate full-size result, replace `main` with:

```rust
fn main() {
    geometry::<5>("MT19937-seed-20260921-full",
        &mut Mt19937::new(20260921), 100, 8000, 100);
}
```

</details>

<details>
<summary>Spectral comparison: Rscript spectral_calibration.R (base R 4.2.0)</summary>

```r
set.seed(20260916)
N<-32768L; reps<-2000L
ps<-matrix(NA_real_,reps,4L);colnames(ps)<-c('ordinate_KS','ordinate_chi2','cumulative_KS','spike')
for(i in seq_len(reps)) {
 u<-floor(runif(N)*2^32)/2^32
 ft<-fft(u-.5);m<-floor(N/2)-1L;P<-Mod(ft[2:(m+1L)])^2/N;Pn<-P/(1/12)
 ps[i,1]<-suppressWarnings(ks.test(Pn,'pexp',rate=1,exact=FALSE)$p.value)
 cnt<-tabulate(findInterval(Pn,qexp(seq(0,1,length.out=11))),nbins=10)
 ps[i,2]<-pchisq(sum((cnt-m/10)^2/(m/10)),df=9,lower.tail=FALSE)
 cp<-cumsum(P)/sum(P)
 ps[i,3]<-ks.test(cp[seq_len(m-1L)],'punif',exact=FALSE)$p.value
 ps[i,4]<- -expm1(m*log1p(-exp(-max(Pn))))
}
write.csv(ps,'spectral-pvalues.csv',row.names=FALSE)
for(j in seq_len(ncol(ps))) {
 cat(colnames(ps)[j], 'trials=',reps,'N=',N,'mean=',mean(ps[,j]),'below_001=',sum(ps[,j]<.001),'below_01=',sum(ps[,j]<.01),'below_05=',sum(ps[,j]<.05),'\n')
}
```

</details>

<details>
<summary>Tail, result and parameter witnesses: examples/audit_boundaries.rs</summary>

```rust
use entropy::{math::{binomial_pmf,igamc},dieharder::{bit_distribution::bit_distribution_all,lagged_sums::lagged_sums},result::TestResult};
fn main(){
 let mut words=vec![0u32;2828];let mut bit=0;
 for b in 0..202 {let zeros=if b<100{0}else if b<150{1}else{10};for j in 0..64 {let x=if j<zeros{0u32}else{1};for k in (0..7).rev(){words[bit/32]|=((x>>k)&1)<<(31-bit%32);bit+=1;}}}
 let rs=bit_distribution_all(&words,7);let r=rs.iter().find(|r|r.note.as_ref().unwrap().starts_with("width=7, pattern=0,")).unwrap();
 let e0=202.0*binomial_pmf(64,0,1.0/128.0);let e1=202.0*binomial_pmf(64,1,1.0/128.0);let et=202.0-e0-e1;
 let chi=(100.0-e0).powi(2)/e0+(50.0-e1).powi(2)/e1+(52.0-et).powi(2)/et;
 println!("vtest {r}; complete_three_cell_chi2={chi:.12} df=2 p={:.12e} expected={e0:.12},{e1:.12},{et:.12}",igamc(1.0,chi/2.0));
 println!("infinite_result_passes={}",TestResult::new("audit",f64::INFINITY).passed());
 println!("lag_usize_max_panics={}",std::panic::catch_unwind(||lagged_sums(&[],usize::MAX)).is_err());
}
```

The panic printed by the last witness is intentional and caught: the
program records the API defect without stopping the remaining report.

</details>

<details>
<summary>DCT differential experiment: examples/audit_dct.rs</summary>

```rust
use entropy::{rng::{Rng,Mt19937},dieharder::dct::dct,math::igamc};
use rustfft::{FftPlanner,num_complex::Complex};
fn main() {
 let n=256;let blocks=5000;let mut rng=Mt19937::new(20260922);let words:Vec<u32>=(0..n*blocks).map(|_|rng.next_u32()).collect();
 let start=std::time::Instant::now();let prod=dct(&words);let prod_time=start.elapsed().as_secs_f64();
 let table:Vec<f64>=(0..n).flat_map(|k|(0..n).map(move |j|((j as f64+0.5)*k as f64*std::f64::consts::PI/n as f64).cos())).collect();
 let mut planner=FftPlanner::<f64>::new();let fft=planner.plan_fft_forward(2*n);let phase:Vec<Complex<f64>>=(0..n).map(|k|Complex::from_polar(0.5,-std::f64::consts::PI*k as f64/(2*n) as f64)).collect();
 let mut data=vec![Complex::new(0.0,0.0);2*n];let mut scratch=vec![Complex::new(0.0,0.0);fft.get_inplace_scratch_len()];let mut counts=vec![0;256];let mut mismatches=0;let mut max_normalized_error=0.0f64;let mut fast_time=0.0;
 for (j,w) in words.chunks_exact(n).enumerate() {
   let rot=(j/(blocks/4)*8) as u32;
   let x:Vec<f64>=w.iter().map(|&w|w.rotate_left(rot) as f64).collect();
   let mut direct:Vec<f64>=(0..n).map(|k|x.iter().zip(&table[k*n..(k+1)*n]).map(|(&a,&b)|a*b).sum()).collect();
   direct[0]=(direct[0]-n as f64*(2147483648.0-0.5))/2f64.sqrt();
   let start=std::time::Instant::now();
   for (i,&v) in x.iter().enumerate(){data[i]=Complex::new(v,0.0);data[2*n-1-i]=Complex::new(v,0.0);}
   fft.process_with_scratch(&mut data,&mut scratch);
   let mut fast:Vec<f64>=(0..n).map(|k|(data[k]*phase[k]).re).collect();fast[0]=(fast[0]-n as f64*(2147483648.0-0.5))/2f64.sqrt();
   let arg=|xs:&[f64]|xs.iter().enumerate().max_by(|(_,a),(_,b)|a.abs().partial_cmp(&b.abs()).unwrap()).unwrap().0;
   let fi=arg(&fast);counts[fi]+=1;fast_time+=start.elapsed().as_secs_f64();if arg(&direct)!=fi{mismatches+=1;}
   for (&a,&b) in direct.iter().zip(&fast){max_normalized_error=max_normalized_error.max((a-b).abs()/(n as f64*u32::MAX as f64));}
 }
 let expected=blocks as f64/n as f64;let chi: f64=counts.iter().map(|&c|(c as f64-expected).powi(2)/expected).sum();let p=igamc((n-1) as f64/2.0,chi/2.0);
 assert_eq!(prod.p_value,p);
 println!("seed=20260922 blocks={blocks} n={n} argmax_mismatches={mismatches} normalized_max_coefficient_error={max_normalized_error:.5e} production_p={:.17} fft_p={p:.17} production_seconds={prod_time:.6} fft_kernel_seconds={fast_time:.6}",prod.p_value);
}
```

</details>
