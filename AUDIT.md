# Entropy audit — open work

What is known to be wrong, unmeasured or unfinished, with the evidence that
says so. A finding that has been fixed moves to [Closed this round](#closed-this-round)
with the commit that closed it, so the list stays short.

The third external review (2026-09-17) raised E1–E7 against entropy `63592e0`;
its evidence, benchmark driver and logs are kept outside the repository, under
`review/2026-09-17/`, and are not release qualification.

## Open

### A1 — Calibration at the thresholds decisions actually use

Most calibration is measured at 0.01 on thousands of streams. At 0.001 an
ordinary 95% interval of half-width 10% of alpha needs about 384 000
independent null trials, and rarer corrected thresholds need more.

| Area | Measured | Missing |
|---|---|---|
| Whole battery, every slot | 200 000 null streams over the four suites on PCG64, Xoshiro256 and SFC64 (`examples/battery_null.rs`, `stats/battery-null-moore.txt`, read with `scripts/battery_null_report.py`): the standard error at 0.01 is 0.022 points and at 0.001 it is 0.007, so both columns now resolve. 44 of the 57 scored slots are within three standard errors of nominal at 0.01; the thirteen that are not are listed below | The listed slots, each of which is a property of its test's published approximation |
| Battery-wide decision | Some family rejects 93.5% of null streams at 0.05, 42.3% at 0.01 and 5.5% at 0.001, over about sixty families each held at its own level | A stated battery-level rule, if a single verdict is wanted |
| Count-ones Q5−Q4 | 5 000 000 byte windows in the campaign reject 5.12% at 0.05, 1.04% at 0.01 and 0.106% at 0.001, a 4% excess at 0.01 that 300 000 earlier windows had put at 6%; the stream form rejects 1.06% at 0.01 over 200 000 | Deriving the finite-window tail rather than measuring it |
| DCT position law | 10⁶ runs: 0.104%, 1.015%, 5.029%; an exactly multinomial control gives 0.1046% and 1.013% | Nothing at this resolution: the residual is the Pearson approximation, not the transform |
| R report | 3 170 held-out null streams through every row (`stats/r-report-null-summary.txt`): 15 of the 18 scored rows are within 2 standard errors of nominal at 0.01, and the runs tests' 20 000-stream rate is 1.09% and 0.135% | More streams at 0.001; and see the two conservative rows below |
| LZ78 above k = 20 | k = 21, 22, 23 and 25 at 3 000 runs each (2 000 at k = 25) on both xoshiro256** and SFC64: every cell within three standard deviations of 1% at 0.01 and every Kolmogorov–Smirnov test of its p-values passing (`stats/lz78-held-out-cells.txt`); the 3.33% once seen at k = 21 was one seed range | The tail at 0.001, which 3 000 runs do not reach |
| Minimum distance | Selected modes and thresholds calibrated | The supported parameter cells, at the thresholds used |

The slots the campaign puts more than three standard errors from nominal at
0.01, over 200 000 streams:

| Slot | 0.05 | 0.01 | 0.001 | z at 0.01 |
|---|--:|--:|--:|--:|
| maurer::universal_l05 | 6.54% | 1.50% | 0.177% | +22.5 |
| maurer::universal_l06 | 6.08% | 1.37% | 0.157% | +16.7 |
| maurer::universal_l07 | 5.68% | 1.26% | 0.146% | +11.6 |
| nist::spectral | 5.68% | 1.21% | 0.146% | +9.6 |
| diehard_historical::count_ones_bytes | 5.12% | 1.04% | 0.106% | +8.5 |
| nist::random_excursions | 5.08% | 1.07% | 0.119% | +8.3 |
| maurer::universal_l08 | 5.37% | 1.12% | 0.129% | +5.6 |
| diehard::parking_lot | 4.79% | 1.09% | 0.097% | +4.0 |
| dieharder::bit_distribution | 5.00% | 1.00% | 0.102% | +3.9 |
| nist::random_excursions_variant | 4.95% | 1.02% | 0.118% | +3.7 |
| nist::universal, maurer::universal_l10 | 4.72% | 0.92% | 0.093% | −3.6 |
| nist::block_frequency | 4.93% | 0.92% | 0.078% | −3.6 |

Three are worth a reader's attention and the rest are within a few
hundredths of a point. Maurer's universal test over-rejects at L = 5 … 8, and
it is the standard's variance that is short. Over 4 000 null streams
(`examples/universal_variance.rs`) the z it computes has standard deviation
1.060 at L = 5, 1.034 at L = 6, 1.023 at L = 7 and 1.027 at L = 8, falling to
1.00 by L = 9, which is the 1.5% rejection rate at the 1% level the campaign
sees. μ and σ² are exact properties of the gap law, so the shortfall is in
c(L, K), and at K of a few million both published forms of c agree to four
decimals. The test is left as SP 800-22 defines it; a reader should treat a
single low p-value from L = 5 … 8 as weaker evidence than its level says.

The spectral test rejects 1.21% at 0.01 and 0.146% at 0.001, with the
Kim–Umeno–Hasegawa variance n·0.95·0.05/4 that SP 800-22 Rev. 1a adopted
already in place. Its statistic counts peaks below a threshold, and the peaks
are not independent, so that variance is itself an approximation; the
measurement that would settle it is the one `universal_variance` makes for
Maurer's test, the empirical standard deviation of d over null streams, which
would give the divisor the data support. Not done. Random excursions rejects
1.07%, from the small expected counts in its outer χ² cells at J near 500.

Two R rows are conservative rather than calibrated: the periodogram-height
Kolmogorov–Smirnov test against Exp(1) rejects 1.44% of null streams at 0.05
and 0.03% at 0.01 (z = −5.4), and the periodogram χ² over ten Exp(1) bins
3.66% and 0.52% (z = −2.6). Both read a periodogram whose heights are
estimated from the same stream, so their null is not the independent Exp(1)
the test assumes. A REJECT from either is evidence; a pass is weaker than its
level suggests. `tseries::jarque.bera.test` rejects every null stream, as it
must: it tests normality of a uniform stream, and the report marks it invalid
rather than failing the generator.

Power is measured for the NIST families in [POWER.md](POWER.md): 100 streams
per cell, ten defect settings, three sample sizes, a Bonferroni decision per
family. Nothing equivalent exists for DIEHARD, DIEHARDER or the research
probes. Null and power are measured on separate streams, as they must be.

### A2 — OS entropy is Unix-only

`OsRng` reads `/dev/urandom`, and `os_random` waits on `/dev/random` on Linux
first. Windows has no path here, and no other target gains native entropy
merely by compiling. The choice is to implement a documented platform API
(Windows has none without FFI), to depend on a maintained backend abstraction,
or to leave the target unsupported and say so. Darrell is finding a Windows
machine to test against.

### A4 — The shuffle trails rand

Every sampling method is measured here (`examples/variate_throughput.rs`), and
against rand 0.10.2 through public APIs on this Mac, best of seven rounds of
five million draws, in millions per second:

| Draw | entropy | rand 0.10.2 |
|---|--:|--:|
| `next_u64`, PCG64 | 763 | 763 |
| `next_u64`, xoshiro256\*\* / SmallRng | 1 344 | 1 393 |
| uniform double | 760 | 760 |
| integer in 1 … 6 | 314 | 317 |
| normal | 270 | 286 |
| exponential | 243 | 180 |
| shuffle of 1 000 | 701 | 771 |

The generators, the bounded integers and the uniform doubles match, the normal
is within 6%, and the exponential is ahead since it too became a ziggurat.
The shuffle is 9% behind and unexplained: both are Fisher–Yates over the same
bounded-integer method, so the difference is in how the index is drawn per
element.

### A5 — Parallel streams exist only for the linear generators

`Xoshiro256` and `Xoroshiro128` now jump, from a polynomial derived from the
generator rather than a table, so their streams partition reproducibly. The
other generators have no such partition: PCG can be advanced in logarithmic
time by its own arithmetic, the counter-based designs would need a counter
interface, and the cryptographic generators partition by nonce. Deciding one
interface across them, and measuring that the segments are statistically
independent rather than merely disjoint, is open.

## Closed this round

| Finding | What was wrong | Closed by |
|---|---|---|
| E1 | `igamc(a, a)` subtracted terms of size a·ln a: Q(10¹⁴, 10¹⁴) came out 0.59 against 0.5 | A Stirling prefactor from a = 20 and Temme's uniform expansion from a = 10⁵, within 2·10⁻¹⁴ of R's pgamma from a = 20 to 10¹⁴ (f8be8e5) |
| E2 | `normal()` returned ±∞ at U = 2⁻¹⁰⁷⁴, whose half rounds to zero | `math::normal_quantile_ln` solves ln Φ(x) = ln p from Mills' ratio; the sampler carries ln U − ln 2 where halving would round (9365070) |
| E3 | The only byte interface was four-byte `next_u32` words, half a 64-bit generator's output | `Rng::fill_native`, overridden by every buffered generator, beside the unchanged battery projection (5ebe60e) |
| E5 | The anytime-valid bound assumed an accuracy of `ln` that Rust does not specify | `ROUNDING_EPSILONS` names the assumption, both error terms are charged against it, and a test measures exp(ln q) over the estimator's own values (ceabc3f) |
| E6, in part | Per-word thread-local and process-id work, a panic on reseed failure, and a permanently cached readiness error | `ThreadRng::try_fill`, `fill` and `try_next_u64`: one check per request, split at the reseed limit; `pool_ready` remembers only success (ceabc3f) |
| A3, the feature split | The statistics and the application RNG pulled in the FFT and, by default, cryptography | The `batteries` feature carries the four suites and `rustfft`; the four configurations are tested, and the minimal one has no dependencies |
| A6, the cross-repository moves | Two owners for the probability functions and for the DRBG mechanisms | `ln_gamma`, the incomplete beta, Student's t and `NumericalError` live in `entropy::math` (19fb7fc), factoring pins entropy 0.6 for them and rump has deleted its copies; Hash_DRBG, HMAC_DRBG and fast key erasure live in cryptography with entropy adapting them (ab1b7e5); rump is a dev-dependency here only |
| E7 | Points sharing the sweep axis were compared pairwise, and outliers make that axis the widest | A tied run is swept on the widest coordinate it has not used: 162 times faster on the slab family, unchanged on uniform points (acea67a) |

## Standard of evidence

A finding stays open until an experiment at the threshold in question closes
it. Retain the raw statistics, seeds, sample sizes, views and table digests;
calibrate on streams other than those used to fit; never report power and null
from the same streams. A result in [0, 1] is not an accuracy certificate, and
matching empirical rates do not prove an exact null law.
