# Entropy suggestions — 2026-09-17

> **Motto:** better that, better algorithms
>
> **Creed:** Experiment is asking God for peer review.

Current evidence and limitations are in [AUDIT.md](AUDIT.md). Each proposal below
has an acceptance experiment. Predicted improvements are not measured speedups.

## How to compete with rand

Keep entropy's strengths: explicit reproducible stream views, exact discrete
sampling and experiments tied to mathematical null laws. Close the measured bulk
and matched-ChaCha gaps first. More generator names will not fix the byte interface,
portable seeding, feature footprint or numerical-tail failures.

| Order | Work | Acceptance experiment |
|---|---|---|
| 1 | Correct large-shape gamma and the normal half-subnormal endpoint | Independent central/tail references and adversarial word streams |
| 2 | Native bulk output with an explicit stream contract | Word/byte/tail/mixed-access vectors; paired scalar and bulk benchmarks |
| 3 | Batched ChaCha in cryptography | Same round count/output layout; complete RNG and erasure costs |
| 4 | Fallible portable OS/TLS access and compatible secure adapters | Backend/failure/reseed/fork tests; no weak-generator substitution |
| 5 | Separate RNG, statistics and batteries by feature | Minimal consumers avoid FFT and cryptographic multiprecision when unused |
| 6 | Calibrate complete decisions and measure power | Independent null and alternative campaigns at the actual thresholds |

## A bulk interface that preserves meaning

`Rng::next_u32` currently defines a battery projection for native 64-bit PRNGs.
Keep it. Add an overridable byte primitive or a separately named native-byte
interface, and let concrete generators exploit their natural word/block size.
Do not keep the only fill implementation in a blanket extension trait that
prevents specialization.

Specify partial-word retention, endianness, mixing 32/64-bit reads, stream seeking
and block boundaries. Decide whether a native-byte stream remains continuous
across any partition of a requested fill; test that property if promised. Keep
the named high-half/low-half/full-word views for statistical experiments. If an
API changes output order, give it an explicit version or name and retain the
prior stream's known answers.

Give ThreadRng a bulk operation that performs its TLS/reseed check at a defined
boundary, accounts for the whole request and cannot cross a required reseed limit
unnoticed. Measure cached-handle and fresh-handle calls separately. Price 4/8-byte,
480/512-byte and long fills, cold setup, OS reads and reseeding. The audit's
~2× noncryptographic fill gain is a diagnostic result; it is not a measurement of
this future TLS API.

For ChaCha, coordinate with cryptography's block/vector work. Rand's current
StdRng uses ChaCha12; choosing fewer rounds is a separate construction decision.
Use the retained 20-round comparison to evaluate implementation progress without
changing that variable. Test x86 and ARM before changing portable dispatch.

## Make application contracts complete

The new `Seedable`, `CryptoRng`, fallible `OsRng` and `thread_rng` APIs are already
present. Build on them. Document which output sequences are value-stable and
which system-seeded handles are deliberately nondeterministic. A u64 seed
expanded by SplitMix has at most 64 bits of uncertainty; application cryptographic
examples should use OS seeding or adequately entropic secret seeds.

Add named adapters for cryptography's `Csprng` and, if desired, the public rand
traits. Keep any ecosystem adapter behind an optional dependency. Conformance
comes from published API contracts and black-box tests, not copying another
crate's implementation. Define whether an error leaves the destination unchanged,
partially filled or unusable, and keep that rule through reseeding.

Implement OS backends from their documented APIs, or deliberately depend on a
maintained backend abstraction if the dependency policy permits it. That is a
library-use decision, separate from the policy against deriving code from other
implementations. Test supported targets, short/interrupted reads, permanent and
transient failures, actual fork behavior and high-volume reseeding. Do not cache a
recoverable initialization failure forever without an explicit reason.

Benchmark bounded integers, shuffles and normal/exponential draws separately from
raw generation. Lemire's multiply/reject method is already implemented; preserve
its exact accepted-preimage invariant. The new normal inverse solves iteratively,
so it needs its own cost comparison. Faster rejection/table methods are candidates
only with independently derived acceptance regions and verified tail behavior.
[Lemire's paper](https://arxiv.org/abs/1805.10941) is the mathematical reference
for bounded integers.

For reproducible parallel simulations, evaluate counter-based indexing or a
proved jump/stream partition. Seeds assigned by worker scheduling do not define
a reproducible task stream. Derive a design from
[Salmon et al., *Parallel Random Numbers*](https://users.cs.utah.edu/~hari/teaching/bigdata/random123sc11.pdf),
then test scheduling invariance, counter exhaustion, stream identifiers and
statistical power. No need to add it before resolving the measured bulk path.

## Numerical acceptance

For gamma, implement the large-shape central regime from
[DLMF §8.12](https://dlmf.nist.gov/8.12) with stable evaluation near `x/a=1`.
Choose switches by an error target, test both sides, and retain direct small-tail
or log-tail evaluation. Include central and asymmetric shapes, subnormals,
near-zero and near-one probabilities, finite extremes and iteration failure.
A value in [0,1] is not an accuracy certificate.

For the normal sampler, carry the lower-tail argument as `ln(u)-ln(2)` when
halving would underflow; this also requires a quantile evaluator that works from
a log probability rather than first rounding its CDF to zero. Test all tiny
subnormal endpoints with crafted word streams and high-precision quantiles.
State whether sampling approximates a continuous law or promises a particular
rounded law. A finite 53-bit uniform grid and dense floating sampling have
different distributions and tail ranges.

Move shared floating probability functions from rump into a statistics feature
here, with coherent domain/error/accuracy contracts. Factoring's Student consumer
must keep contenders when numerical evaluation fails. Test its actual degrees of
freedom and probabilities, not only symmetric beta fixtures.

## Calibrate the decision that users receive

Retain raw statistics, seeds, sample sizes, views, table/script digests and exact
family membership. Use independent fitting and validation streams. At alpha
0.001, about 384,000 independent null trials are needed for an ordinary 95%
normal-approximation interval with a half-width of 10% of alpha; rarer corrected
thresholds and simultaneous claims need more. Three thousand streams cannot
resolve that tail precisely.

For count-ones, derive or empirically validate the finite-window tail. For DCT,
compare the transform output with an exactly multinomial control before assigning
the discrepancy to position bias. For LZ and minimum distance, cover supported
parameter cells and the thresholds actually used. An empirical table should
carry its training size, discrete-tail convention and out-of-domain behavior.

Run the whole configured battery repeatedly under its null, including selection,
skips and completion rules. Then measure power over defect strength and sample
size with intervals. Do not tune and report power on the same streams. For the
sequential test, retain u64 counts and capacity checks, qualify transcendental
rounding, and allocate error across multiple generators and restarts explicitly.

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
