# Entropy suggestions — what to build next

> **Motto:** better data, better algorithms
>
> **Creed:** Experiment is asking God for peer review.

Each proposal says what would be built and the experiment that would accept
it. Predicted improvements are not measurements. What is already wrong or
unmeasured is in [AUDIT.md](AUDIT.md).

| Order | Work | Acceptance experiment |
|---|---|---|
| 1 | A portable OS entropy story | Per-target tests of the backend, short and interrupted reads, permanent and transient failure, and fork |
| 2 | Close the 9% shuffle gap against rand | A paired measurement per element, with the exact-uniformity invariant kept |
| 3 | Power for DIEHARD, DIEHARDER and the research probes | The `power_curves` treatment extended: defect strength against sample size, with intervals |

## Calibration and power

Retain raw statistics, seeds, sample sizes, views, table digests and exact
family membership; fit and validate on different streams. `battery_null`
accumulates per-slot status counts, rejection counts at 0.05, 0.01 and 0.001,
a twenty-bin p-value histogram, and each family's and the whole battery's
Bonferroni decision; it snapshots as it runs, so a campaign can be read early.
Multiplicity correction cannot repair a miscalibrated marginal, so a slot whose
histogram is not flat is a finding about that test, not about the correction.

For count-ones, derive or empirically validate the finite-window tail. For LZ
and minimum distance, cover the supported parameter cells and the thresholds
actually used, and record each empirical table's training size, discrete-tail
convention and out-of-domain behaviour.

## Platforms

For Windows, either implement a documented platform API — which needs FFI, and
the crate has none — or depend on a maintained backend abstraction, or state
that the target is unsupported. That is a dependency-policy decision, distinct
from the rule against deriving code from other implementations. Test supported
targets, short and interrupted reads, permanent and transient failures, real
fork behaviour and high-volume reseeding.

## Application interface

`Seedable`, `CryptoRng`, fallible `OsRng`, `thread_rng` with its bulk fallible
fill, and `Rng::fill_native` are in place; keep documenting which sequences are
value-stable and which system-seeded handles are deliberately not.

Benchmark bounded integers, shuffles and the variates separately from raw
generation, as `fill_throughput` does for bytes. Lemire's method is already
exact; preserve its accepted-preimage invariant. `normal()` and `exponential()` are ziggurats whose tables are derived at run
time, checked against the inversions kept as `normal_inverse()` and
`exponential_inverse()`. What remains behind rand is the shuffle.

Adapters for cryptography's `Csprng` and, if wanted, the public rand traits
belong behind an optional dependency, with conformance from published API
contracts and black-box tests, never from another crate's implementation.
Define whether an error leaves the destination unchanged, partially filled or
unusable, and keep that rule through reseeding.

`Advance` and `Streams` position every deterministic generator. What is not
measured is that consecutive segments behave as independent streams rather
than merely disjoint ones — a battery run on interleaved segments would show
it — and SFC64's and JSF64's streams rest on seeding, not on a partition.

## Numerical work

`igamc` now has a stable large-shape prefactor and Temme's uniform expansion,
and `normal_quantile_ln` works from a log probability; the incomplete beta,
Student's t and ln Γ have moved here from rump with their references and
scripts. What remains is to keep the contracts coherent as they are used
together: state each function's domain, representation and invariant, retain
published known answers and independent identities, and test boundary strata
and algorithm switches as well as ordinary inputs.

Factoring's Student consumer must keep its contenders when numerical
evaluation fails, and its tests should use its actual degrees of freedom and
probabilities rather than symmetric beta fixtures.

## Cross-repository boundaries

The resolved graph is `cryptography → rump`, `entropy → cryptography` when the
cryptographic generators are enabled, `entropy → rump` only for tests, and
`factoring → rump + entropy` with the features it needs. Rump depends on
neither consumer.

| Owner | Keeps | State |
|---|---|---|
| rump | BigInt, modular arithmetic, primality, exact polynomial, finite-field, GF(2) and lattice support | Its probability functions are deleted; entropy::math is the only copy |
| cryptography | Ciphers, hashes, authenticated schemes, DRBG mechanisms, key erasure and state wiping | Owns Hash_DRBG, HMAC_DRBG and fast key erasure; entropy adapts them |
| entropy | Noncryptographic generators, OS seeding, sampling, stream views, thread-local access, probability functions, batteries | Split by feature: the batteries and the FFT are optional, and the minimal build has no dependencies |
| factoring | Rho, ECM, QS and GNFS orchestration, relation and cofactor policy, polynomial selection, cost dispatch | Pins entropy 0.6 for Student's t and ln Γ, with no default features |

Keep the distinction between rump's quality-neutral `RandomSource`,
cryptography's byte-oriented `Csprng` and entropy's generator and `CryptoRng`
interfaces. Adapters are explicit and documented; a cryptographic contract is
never blanket-implemented for every test generator, because a marker describes
a construction, not the entropy in a caller's seed.

## Standard for accepting changes

Derive the formula and state its domain, representation and invariant. Name
every constant and say where it comes from: a specification section, an
equation, a measurement, or a stated policy and its reason. Retain published
known answers, independent identities and reproducible table generation. Test
the boundary strata and the algorithm switches, not only ordinary inputs.

Measure with fixed inputs, seeds, compiler, target, features and sibling
revisions; record wall time, process-tree CPU, memory and work counters;
separate setup, steady state and teardown, then report the whole operation
too. Statistical acceptance, semantic security, exact factorisation and
performance are separate claims with separate evidence.
