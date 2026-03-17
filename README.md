# entropy

`entropy` is a pure Rust statistical test suite for pseudorandom number generators.

It aims to provide a readable, hackable implementation of the major classic batteries:

- NIST SP 800-22 Rev. 1a
- DIEHARD
- DIEHARDER

This is a serious audit tool, but it is not a magical oracle. Some tests are fully faithful to the published or reference implementations, some are close ports of the Dieharder source, and a small number are still approximate. The project is strongest when it is explicit about which is which.

## Dependency Note

This repository depends on Darrell Long's [`cryptography`](https://github.com/darrelllong/cryptography) repository via a local Cargo path dependency. It is currently used for elliptic-curve support and related primitives needed by some of the bundled RNG implementations, and may also supply other low-level cryptographic building blocks over time.

## What This Repository Is For

Use this repository when you want to:

- run a broad battery of statistical checks against RNGs implemented in Rust
- compare obviously bad generators against stronger ones
- inspect the test code directly instead of treating a binary as a black box
- experiment with classic randomness batteries in one codebase

Do not use it as the sole basis for claiming a generator is cryptographically secure.

## Current State

The crate builds and tests cleanly:

```sh
cargo build
cargo test
```

The test runner lives in [src/main.rs](src/main.rs) and the library entrypoints are split across:

- [src/nist](src/nist)
- [src/diehard](src/diehard)
- [src/dieharder](src/dieharder)

The current external audit is in [PEERREVIEW.md](PEERREVIEW.md).

## Running

### Full audit (canonical)

```sh
tests/run_all.sh
```

Runs the complete audit path — NIST/DIEHARD/DIEHARDER battery plus all five
auxiliary probes — and saves a timestamped log to
`logs/run_all-<host>-<date>.log`.
Feed that log to `scripts/parse_battery.py` to regenerate `TESTS.md`.

### Main battery only

```sh
tests/run_battery.sh
# or with options:
tests/run_battery.sh --suite nist
tests/run_battery.sh --suite diehard --quick
tests/run_battery.sh --test nist::spectral
```

### Auxiliary probes only

```sh
tests/run_aux.sh
```

Runs the five standalone research probes (Knuth + ApEn, TestU01 Hamming,
TestU01 Lempel-Ziv, Webster-Tavares, Gorilla) with their default parameters.
Use the individual binaries for filtered or resized runs:

```sh
cargo run --release --bin bib_tests    -- --rng AES
cargo run --release --bin upstream_tests -- --rng AES
cargo run --release --bin testu01_lz   -- --rng AES --k 27
cargo run --release --bin webster_tavares -- --samples 2048
cargo run --release --bin gorilla      -- --rng AES
```

### Benchmarks

```sh
tests/run_benchmarks.sh
```

## What The Runner Exercises

The default runner compares several built-in generators, including:

- `/dev/urandom`
- MT19937
- Xorshift32 and Xorshift64
- historical weak Unix libc generators:
  System V `rand()` and `mrand48()`, BSD `random()`, Linux glibc `rand()/random()`, and the old FreeBSD `rand_r()` compatibility path
- historical weak Microsoft/Windows-family generators:
  CRT `rand()`, VB6/VBA `Rnd()`, and classic `.NET Random(seed)` compatibility
- classic standalone LCG-style generators
- Blum Blum Shub
- Blum-Micali
- AES-128-CTR as a deterministic keystream source
- intentionally bad generators like a constant stream and a counter

That makes the output useful both for regression testing and for sanity-checking that the batteries still punish obviously bad constructions.

Those Unix libc APIs are included precisely because they are bad historical RNGs. They are useful as negative controls, not as designs to copy.

## Implementation Status

Status here means "how comfortable this repository should be claiming fidelity," not "whether the test compiles."

| Area | Status |
|------|--------|
| NIST SP 800-22: frequency, block_frequency, runs, longest_run, matrix_rank, spectral, serial, approximate_entropy, cumulative_sums, universal, linear_complexity | Faithful or close faithful implementations |
| Maurer (1992): parametric universal family `L=6..16` | Added alongside the NIST-locked single setting; emits every parameter set that fits the available sample |
| NIST SP 800-22: non_overlapping_template | Faithful for all 148 aperiodic 9-bit templates with the standard `N = 8` block setup |
| NIST SP 800-22: random_excursions, random_excursions_variant | Faithful family outputs; runner emits all per-state results |
| DIEHARD: runs_float, binary_rank, birthday_spacings, bitstream, monkey tests, count_ones_stream, craps | Faithful or close to the Dieharder reference implementation |
| Removed on purpose | See the explicit removed-test list below |
| DIEHARDER: fill_tree, gcd | Faithful; runner emits both underlying sub-results |
| DIEHARDER: bit_distribution | Faithful `rgb_bitdist` core statistic with explicit per-width, per-pattern Vtest outputs instead of Brown's random one-pattern collapse |
| Several geometric / higher-level Dieharder-style tests | Plausible and useful, but still best treated as implementation-reviewed rather than externally validated |
| Webster–Tavares (1985): strict avalanche / bit-independence probe over seeded RNG families | Implemented as a research binary (`webster_tavares`); computes the dependence matrix and avalanche-variable correlations from the paper |
| Knuth TAOCP Vol. 2 §3.3.2: permutation, gap, and Wald-Wolfowitz runs-above/below-median tests | Implemented as a research binary (`bib_tests`) over uniform `[0,1)` streams |
| NIST SP 800-22 §2.12 ApEn statistic swept over multiple embedding dimensions `m=2..6` | Implemented as part of `bib_tests`; reveals at which pattern lengths a sequence departs from randomness beyond the single fixed NIST setting |
| TestU01 (2009): `scomp_LempelZiv` core statistic and official empirical calibration table | Implemented as a research binary (`testu01_lz`); exact per-replication `LZ78` phrase count and TestU01 `μ/σ` normalization, but not yet the full TestU01 goodness-of-fit reporting stack |
| TestU01 (2009): `sstring_HammingCorr` and `sstring_HammingIndep` core statistics | Implemented as part of `upstream_tests`; faithful TestU01 bit extraction, asymptotic normal `HammingCorr`, and TestU01-style `gofs_MinExpected=10` lumping for the main `HammingIndep` chi-square |
| PractRand pre-0.95: `FPF(4,14,6)` core statistic | Implemented as part of `upstream_tests`; faithful stride-spaced windowing and exponent/significand bucket counts, but without PractRand's empirical calibration tables/suspicion scores |

## Important Caveats

- Passing these tests does not prove unpredictability, backtracking resistance, or cryptographic suitability.
- A single low p-value is not automatically evidence that a generator is broken.
- Some tests naturally emit families of p-values; the runner now preserves many of those families instead of flattening them into one fake verdict.
- A few historically famous tests are themselves weak. In particular, Dieharder explicitly calls out some classic tests as poor discriminators.

## Removed On Purpose

These are not accidental omissions. They were removed because the Dieharder reference source or documentation says they are broken, deprecated, or effectively obsolete.

- `DIEHARD` removed: `operm5`
  Dieharder describes the original overlapping Diehard OPERM5 as the broken/defunct test that `rgb_operm` was meant to replace.
- `DIEHARD` removed: `overlapping_sums`
  Dieharder says this test is completely useless, broken, and not worth fixing, and explicitly says not to use it.
- `DIEHARD` removed: `count_ones_specific_bytes`
  Dieharder says this byte-lane variant is effectively obsolete compared to the stream variant and `rgb_bitdist`.
- `DIEHARDER` removed: none currently
  Deprecated internals such as the Kuiper KS path are intentionally not exposed as active tests in this crate.

## Project Layout

- [src/math.rs](src/math.rs): special functions, KS helper, FFT support
- [src/result.rs](src/result.rs): shared result type and display logic
- [src/rng](src/rng): RNG implementations used by the harness
- [src/nist](src/nist): NIST SP 800-22 tests
- [src/diehard](src/diehard): DIEHARD tests
- [src/dieharder](src/dieharder): DIEHARDER tests

## Attribution

Functions adapted from DIEHARD or DIEHARDER include `# Author` citations in their doc comments. The goal is not to erase provenance behind a Rust rewrite.

## Reference Corpus

This repository keeps a local reference shelf under [pubs/](pubs) so people can check the implementation work against the actual standards, manuals, source releases, and papers instead of trusting summaries.

Included now:

- standards: `NIST-SP-800-22r1a.pdf`, `NIST-SP-800-90Ar1.pdf`, `NIST-SP-800-90B.pdf`, `NIST-SP-800-90C.pdf`, `NIST-FIPS-140-3.pdf`
- classic source and docs: `Diehard.zip`, `diehard-doc.txt`, `diehard-tests.txt`, `dieharder-3.31.1.tgz`, `dieharder-manual.pdf`, `dieharder-tests.txt`
- core survey and extension papers: `lecuyer-simard-2007-testu01.pdf`, `maurer-1992-universal-test.pdf`, `marsaglia-tsang-2002-difficult-tests.pdf`, `webster-tavares-1985-sbox-design.pdf`, `hughes-2022-badrandom-the-effect-and-mitigations-for-low-entropy-random-numbers-in-tls.pdf`

When the code claims fidelity to a published test, these are the documents the project is expected to match.

## References

Primary references used by the code and audit:

- NIST SP 800-22 Rev. 1a
- George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995)
- Robert G. Brown, *Dieharder* 3.31.x source
- Marsaglia and Tsang, "Some Difficult-to-pass Tests of Randomness," *Journal of Statistical Software* 7(3), 2002

Additional suites and tests surveyed (candidates for future implementation):

- L'Ecuyer and Simard, "TestU01: A C Library for Empirical Testing of Random Number Generators," *ACM TOMS* 33(4), 2007 — the current gold standard; BigCrush contains ~106 tests including BirthdaySpacings, Gap, CouponCollector, MaxOft, LempelZiv, HammingCorr, RandomWalk, and LinearComplexity profile tests, many of which catch defects invisible to all three batteries here.
- Chris Doty-Humphrey (Crow), *PractRand* 0.95, 2018 — streaming suite; BCFN, DC6, FPF, and TMFn tests are designed specifically for small-state generators (xorshift*, PCG) that pass all classic batteries.
- Knuth, *The Art of Computer Programming* Vol. 2 §3.3.2 — classical tests not in NIST/Diehard: Gap, Poker (hand-type), Permutation, Wald-Wolfowitz runs above/below median, and the Serial Correlation Coefficient with exact variance.
- Maurer, "A Universal Statistical Test for Random Bit Generators," *Journal of Cryptology* 5(2), 1992 — the full parametric form (L=10–16) is substantially more sensitive than the fixed NIST implementation.
- Hellekalek and Wegenkittl, "Empirical Evidence Concerning AES," *ACM Trans. Modeling and Computer Simulation* 13(4), 2003 — Walsh-Hadamard spectral test; sensitive to nonlinear Boolean structure in keystream generators.
- Golić, "On the Linear Complexity and Multidimensional Distribution of Decimated m-Sequences," *IEEE Trans. Inf. Theory* 43(3), 1997 — decimated linear complexity; directly relevant to stream ciphers and LFSR-based generators.
- Doganaksoy and Göloglu, "On the Weakness of Non-Dual Bent Functions," *SAC 2005*, LNCS 3897 — L1-norm DFT variant; catches diffuse periodic structure missed by NIST's peak-count statistic.
- Webster and Tavares, "On the Design of S-Boxes," *CRYPTO 1985* — Strict Avalanche Criterion and Bit Independence Criterion; applicable to seeded PRNGs to test differential output behavior.

The source PDFs, manuals, and source archives live under `pubs/`. Full BibTeX entries and implementation notes are in [BIB.md](BIB.md).

---

<p align="center">
  <a href="https://commons.wikimedia.org/wiki/File%3AHet_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg">
    <img src="https://upload.wikimedia.org/wikipedia/commons/d/dd/Het_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg" alt="Extracting the stone of madness" width="360" />
  </a>
</p>

<p align="center">
  <em>Extracting the Stone of Madness</em>, after Hieronymus Bosch. Image source: <a href="https://commons.wikimedia.org/wiki/File%3AHet_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg">Wikimedia Commons</a>.
</p>
