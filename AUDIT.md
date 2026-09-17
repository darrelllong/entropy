# Entropy audit — 2026-09-17

> **Motto:** better that, better algorithms
>
> **Creed:** Experiment is asking God for peer review.

## Scope and evidence

This review covers the captured sibling combination below on Apple M4 Pro,
`aarch64-apple-darwin`, rustc/Cargo 1.93.1, with separate Rust 1.87 checks.

| Repository | Captured HEAD |
|---|---|
| cryptography | `0242a217f1d79ab01bd43d4e5b79fc2a7be7a88f` |
| entropy | `63592e02ab50a494499a87c3abe0ab406ab01bf5` |
| rump | `ae7566b1b100239e1b511a9b05ff8229ea6613bd` |
| factoring | `732801274f7a27640b3616995b7503855a870e99` |

The reviewed-file manifest for this repository has SHA-256
`aa2d0d9dcb11da46f8e05c9d8bc355548def1519db29d4d91d954e2f03111340` (393 files).
[The manifest](review/2026-09-17/reviewed-files.sha256) contains sorted
`SHA256(file)  relative/path` lines; its own digest identifies the capture.
It covers tracked and nonignored regular files, excluding these two review
documents and the review artifacts added afterward. Entropy's final capture includes its new
seeding, sampling, thread-local and `CryptoRng` APIs through `63592e0`.

The review distinguishes reproduced results, source inspection, retained
measurements and proposed experiments. The files record current findings and
acceptance criteria; they do not implement the proposed changes. Implementation
references are papers, standards and mathematics. External libraries were called
through public APIs for comparison; their implementation source was not used.

## Assessment

Entropy now provides exact bounded-integer/Bernoulli sampling, 53-bit and dense
float sampling, a common seeding interface, fallible OS reads, fast key erasure,
a thread-local generator and a `CryptoRng` marker. These additions substantially
change the application surface. Its remaining competition with rand concerns
bulk execution, platform/error contracts, interoperability and measured sampling
cost—not simply the number of generator names.

Two fresh numerical counterexamples remain: large-shape incomplete gamma returns
an inaccurate central probability, and the dense normal sampler can return
infinity from a positive representable draw. The release suites pass despite
these boundary failures.

## Findings

### E1 — High: incomplete gamma loses central accuracy for large shapes

**Reproduced through the public API.** In [src/math.rs](src/math.rs),
`igamc(a,a)` computes `a*ln(a) - a - lgamma(a)` by subtracting large rounded
terms. The resulting normalization error survives convergence of the series.

| a = x | Computed Q(a,a) | R `pgamma(a,a,lower.tail=FALSE)` |
|---:|---:|---:|
| 10^8 | 0.4999868085568120 | 0.4999867019239859 |
| 10^10 | 0.4999950657431844 | 0.4999986701923987 |
| 10^12 | 0.4990662586865413 | 0.4999998670192399 |
| 10^14 | 0.5902894209603868 | 0.4999999867019240 |

The independent large-a expansion starts
`Q(a,a) = 1/2 - 1/(3*sqrt(2*pi*a)) + O(a^(-3/2))`, consistent with the reference
column. [DLMF §8.12](https://dlmf.nist.gov/8.12) gives the expansion and its
central coefficients. Increasing the iteration budget does not recover digits
lost in the prefactor. Use a stable central normalization and an appropriate
uniform expansion, with tested transitions to the other regimes.

The [retained numerical client](../rump/review/2026-09-17/numerical-probe/src/main.rs)
also exercises tiny shapes and rump's beta/Student functions. Tiny-shape gamma
returns finite positive tails in the tested cases. No ordinary battery run
reaching the largest shapes above was demonstrated; this is a public-domain
accuracy failure, not a reproduced false verdict for every battery.

### E2 — Medium: dense normal sampling can halve a positive draw to zero

**Reproduced.** [src/rng/sample.rs](src/rng/sample.rs), `Sample::normal`, checks
`u > 0` and then calls `normal_quantile(0.5*u)`. At
`u = f64::from_bits(1) = 2^-1074`, that multiplication rounds to zero and the
quantile returns negative infinity. The random sign can return either infinity.

A deterministic word source with sixteen zero u64s, then `1 << 14`, then zero
produces exactly that positive value in `unit_f64_dense`. With sign word zero,
`normal()` returns `-inf`. The same client retained for E1 reproduces it.
This event is extremely rare for ideal random words, but the explicitly claimed
subnormal-tail domain includes it. Evaluate the half probability in log space
or derive another finite-tail representation; do not erase the failure by
clamping all tails or silently narrowing the advertised domain.

### E3 — Medium: generic byte filling wastes native 64-bit output

**Source inspection and fresh experiment.** `Sample::fill_bytes` calls
`next_u32` once per four bytes. For Jsf64 and xoshiro256**, each such call advances
a native 64-bit generator and keeps only its high half. The blanket `Sample`
implementation supplies no concrete-generator bulk override. This is the right
legacy battery projection, but it is costly as an application byte API.

| Generator/path | u64 MiB/s | Bulk MiB/s |
|---|---:|---:|
| Entropy Jsf64 | 9,791.3 | 4,844.1 |
| Entropy xoshiro256** | 10,018.9 | 4,972.4 |
| rand `SmallRng` (xoshiro256++ here) | 10,635.5 | 10,562.8 |
| Entropy ChaCha20 | 753.0 | 719.4 |
| `chacha20` 0.10.2 ChaCha20 RNG | 1,359.9 | 1,564.3 |
| `rand_chacha` 0.10.0 ChaCha20 RNG | 809.0 | 869.4 |
| rand `StdRng` (ChaCha12) | 2,202.7 | 2,720.4 |

An experimental loop writing all eight bytes of each native u64 reaches
9,972.0 MiB/s for Jsf64 and 10,126.6 for xoshiro: 2.06× and 2.04× their current
byte-fill paths. It changes the emitted stream, so this cannot silently replace
the documented four-byte projection.

The [benchmark source, lockfile and raw records](review/2026-09-17/README.md)
use rand 0.10.2 and public APIs only. These are medians from seven measured rounds
after one warm-up, with alternating implementation order, fixed seeds,
20 million scalar calls and 64 MiB fills. They use process CPU time on a heavily
loaded M4 Pro; wall time is also retained. Setup is excluded. Initial generator
sources were captured at `84dc79b`; the final capture adds APIs but preserves the
measured implementations and fill loop. This experiment does not price TLS,
OS reads, reseeding, fast-key erasure or distributions.

Rand is not uniformly faster by a large factor: native noncryptographic scalar
performance is close here. Its bulk interface and the matched ChaCha20 backend
show the substantial gaps. StdRng's 12 versus 20 rounds and SmallRng's ++ versus
** output functions are different algorithms. Current choices are documented by
[rand's StdRng](https://docs.rs/rand/0.10.2/rand/rngs/struct.StdRng.html) and
[SmallRng](https://docs.rs/rand/0.10.2/rand/rngs/struct.SmallRng.html); neither
comparison alone establishes a like-for-like algorithm speedup.

### E4 — Medium: calibration remains local to the tested null and decision

**Retained evidence, not rerun campaigns.** The current records support these
more specific limits:

| Area | Evidence | Remaining question |
|---|---|---|
| Count-ones Q5−Q4 | 300,000 xoshiro windows reject at 0.01 in 1.060%; 100,000 PCG windows in 1.045% | Finite-window tail versus the chi-square approximation |
| DCT position/Pearson | 1,000,000 runs: 0.104%, 1.015%, 5.029% at 0.001, 0.01, 0.05 | Distinguish coefficient-position law from finite-cell Pearson approximation |
| Exactly multinomial control, 256 cells, 5,000 draws | 0.1046% at 0.001 and 1.013% at 0.01; quoted SE at 0.001 about 0.003 percentage points | Approximation error at this cell count; no universal direction of bias for every Pearson test |
| R report | Many rows use 3,000 null streams; runs uses 20,000 and rejects in 0.135% at 0.001 | Enough independent trials at the actual deciding threshold |
| Minimum distance / LZ78 | Selected modes/thresholds calibrated; held-out LZ cases above k=20 have only 80–300 runs | Tail accuracy at 0.001 and across supported parameter cells |
| Whole battery and alternatives | A Bonferroni family/power driver exists | Repeated complete-family null and power curves for the exact procedure |

The DCT control makes the finite-cell approximation a concrete candidate;
matching empirical rates does not prove exact exchangeability of transformed
coefficients. Bonferroni does not require independent tests, but it does require
valid marginal tail probabilities. Multiplicity correction cannot repair a
miscalibrated marginal.

### E5 — Medium: the sequential numerical bound assumes more than Rust promises

**Source inspection; no fresh counterexample.**
[src/research/sequential.rs](src/research/sequential.rs) uses u64 counts, a checked
stream limit and a lower log-wealth estimate with accumulated rounding error.
The transcendental error allowance assumes accuracy of `ln`/`exp` that is not a
portable bound supplied by Rust's f64 API. Rust documents
[unspecified precision for `ln`](https://doc.rust-lang.org/std/primitive.f64.html#method.ln).
State the numerical assumptions, qualify each supported math library, or use
outward certified evaluation before calling the floating result a portable
anytime guarantee. Generator selection, multiple streams and restarts require
their own testing budget.

### E6 — Medium: platform and bulk-access contracts remain narrower than rand's

**Source inspection.** OS reads now return `io::Result`, with Linux initialization
checking, and the thread generator checks PID changes and reseeds by output
volume. OS acquisition still uses Unix device paths; other targets do not gain
native entropy support merely because the crate compiles. `try_thread_rng`
handles initial acquisition, while later draws can still panic on reseed failure.
The one-time Linux readiness cell retains an initial error permanently.

Per-word `ThreadRng` calls also repeat TLS/PID work; generic byte filling repeats
that once per four bytes. Add an explicit bulk/fallible access contract and
exercise transient failure recovery. Compare this with the documented supported
backends of [getrandom](https://docs.rs/getrandom/0.4.3/getrandom/) without using
its implementation source as a template. The new fork and erasure choices are
useful design properties, not yet a measured universal advantage over rand.

### E7 — Medium: nearest-pair pruning retains a quadratic input family

**Source inspection.** [src/diehard/nearest_pair.rs](src/diehard/nearest_pair.rs)
selects the widest coordinate and stops at zero distance. Many distinct points
can still share that coordinate, forcing pairwise comparisons within the tied
slab. Distant outliers can make that coordinate globally widest. Measure this
family as well as uniform and duplicate-heavy data before choosing a grid or
multidimensional partition; such a replacement also needs dimension/scale bounds.

## Fresh verification

| Check | Result |
|---|---|
| Final capture: release, offline/locked, all targets | 493 passed; 1 ignored |
| Final capture: no default features, same suite | 363 passed; 1 ignored |
| Final release doctests | 6 passed |
| Rust 1.87 all-target check | Passed |
| `r_rng_tests.R --self-test` | Passed |
| Empty corpus plus `--fail-on-fail` | Exit 3, insufficient input and missing requested result |
| LZ k=6, 10,001 replications | Exit 3, unsupported result |
| Public numerical boundary client | E1 and E2 reproduced |
| Matched ChaCha20 stream | 100,000 u64s identical across three paths |

[Validation records](review/2026-09-17/validation.json) identify commands and
logs. No new million-stream calibration, full power campaign, real-fork campaign,
Windows/WASM/Linux runtime qualification or large geometric stress benchmark was
run. Keep those gaps separate from the passing functional suite.

## Cross-repository ownership

Keep the four repositories, with a focused boundary refactor. The desired graph
is `cryptography → rump`, `entropy → cryptography` when crypto generators are
enabled, and `factoring → rump + entropy` with only the RNG/statistics features
it needs. Rump must not depend on either consumer.

| Owner | Keep here | Boundary change |
|---|---|---|
| rump | BigInt, modular arithmetic, primality, exact polynomial/finite-field/GF(2)/lattice support, caller-driven BigInt sampling | Move floating probability kernels out; retain reusable arithmetic without factoring policy or OS entropy |
| cryptography | Ciphers, hashes, authenticated schemes, DRBG mechanisms, cryptographic state evolution and erasure | Own Hash_DRBG, HMAC_DRBG and fast-key-erasure cores; entropy supplies their adapters |
| entropy | Noncryptographic PRNGs, OS seeding, sampling, stream views, thread-local access, probability functions and test batteries | Separate application RNG, statistics and batteries by features; make FFT/battery dependencies optional |
| factoring | Rho/ECM/QS/GNFS orchestration, relation/cofactor policy, polynomial selection and size/cost dispatch | Reuse native modular arithmetic; keep schedule, graph forecasting and algorithm selection here |

Generic exact algebra in rump is supporting mathematics, not a reason to move
QS/GNFS policy there. `ln_gamma`, incomplete beta and Student quantiles are
floating statistical functions; entropy already owns most probability kernels
and factoring already depends on entropy. Move them in a coordinated API release
with reference fixtures. A rump forwarding wrapper that calls entropy would
create a dependency cycle and is unsuitable.

Preserve the distinction between rump's quality-neutral `RandomSource`,
cryptography's byte-oriented `Csprng`, and entropy's generator/`CryptoRng`
interfaces. Add explicit adapters with documented security and byte-stream
contracts; never blanket-implement a cryptographic contract for every test RNG.
A marker describes a construction, not the entropy in a caller-supplied seed.

Cryptography enables rump's additive `wipe` feature. Entropy default inherits it;
entropy minimal and standalone factoring do not. Record the resolved graph in
benchmarks: compiling factoring alongside a consumer that enables wipe can change
its arithmetic costs. Separate processes/packages may be needed when measuring
that configuration. Optional features should remove unwanted dependencies, not
silently weaken a cryptographic build's erasure contract.

## Standard for accepting changes

Derive the formula and state its domain, representation and invariant. Retain
published known answers, independent mathematical identities and reproducible
coefficient/table generation. Test boundary strata and algorithm switches as
well as ordinary inputs. Source comments should explain the invariant, assumption
or non-obvious choice and cite the relevant paper section when useful.

Use paired measurements with fixed inputs, seeds, compiler, target, features and
sibling revisions. Record wall time, total process-tree CPU, memory and work
counters. Separate the cost of setup, steady-state work and teardown, then report
the complete operation too. Statistical acceptance, semantic security, exact
factorization and performance are separate claims with separate evidence.
