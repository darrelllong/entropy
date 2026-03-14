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

The test runner lives in [src/main.rs](/Users/darrell/entropy/src/main.rs) and the library entrypoints are split across:

- [src/nist](/Users/darrell/entropy/src/nist)
- [src/diehard](/Users/darrell/entropy/src/diehard)
- [src/dieharder](/Users/darrell/entropy/src/dieharder)

The current external audit is in [PEERREVIEW.md](/Users/darrell/entropy/PEERREVIEW.md).

## Running

Full runner:

```sh
cargo run --release --bin run_tests
```

Useful variants:

```sh
cargo run --release --bin run_tests -- --suite nist
cargo run --release --bin run_tests -- --suite diehard --quick
cargo run --release --bin run_tests -- --test nist::spectral
```

Benchmark harness:

```sh
cargo run --release --bin bench_rngs
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
| DIEHARDER: bit_distribution | Approximate aggregate pattern-frequency test, not Brown's `rgb_bitdist` |
| Several geometric / higher-level Dieharder-style tests | Plausible and useful, but still best treated as implementation-reviewed rather than externally validated |

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

- [src/math.rs](/Users/darrell/entropy/src/math.rs): special functions, KS helper, FFT support
- [src/result.rs](/Users/darrell/entropy/src/result.rs): shared result type and display logic
- [src/rng](/Users/darrell/entropy/src/rng): RNG implementations used by the harness
- [src/nist](/Users/darrell/entropy/src/nist): NIST SP 800-22 tests
- [src/diehard](/Users/darrell/entropy/src/diehard): DIEHARD tests
- [src/dieharder](/Users/darrell/entropy/src/dieharder): DIEHARDER tests

## Attribution

Functions adapted from DIEHARD or DIEHARDER include `# Author` citations in their doc comments. The goal is not to erase provenance behind a Rust rewrite.

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
- Pincus, "Approximate Entropy as a Measure of System Complexity," *PNAS* 88, 1991 — multi-scale ApEn(m) vs. m profile catches correlation length; NIST uses only a single scale.
- Hellekalek and Wegenkittl, "Empirical Evidence Concerning AES," *ACM Trans. Modeling and Computer Simulation* 13(4), 2003 — Walsh-Hadamard spectral test; sensitive to nonlinear Boolean structure in keystream generators.
- Golić, "On the Linear Complexity and Multidimensional Distribution of Decimated m-Sequences," *IEEE Trans. Inf. Theory* 43(3), 1997 — decimated linear complexity; directly relevant to stream ciphers and LFSR-based generators.
- Doganaksoy and Göloglu, "On the Weakness of Non-Dual Bent Functions," *SAC 2005*, LNCS 3897 — L1-norm DFT variant; catches diffuse periodic structure missed by NIST's peak-count statistic.
- Webster and Tavares, "On the Design of S-Boxes," *CRYPTO 1985* — Strict Avalanche Criterion and Bit Independence Criterion; applicable to seeded PRNGs to test differential output behavior.

The source PDFs and papers live under `pubs/`. Full BibTeX entries and implementation notes are in [BIB.md](BIB.md).

---

<p align="center">
  <a href="https://commons.wikimedia.org/wiki/File%3AHet_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg">
    <img src="https://upload.wikimedia.org/wikipedia/commons/d/dd/Het_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg" alt="Extracting the stone of madness" width="360" />
  </a>
</p>

<p align="center">
  <em>Extracting the Stone of Madness</em>, after Hieronymus Bosch. Image source: <a href="https://commons.wikimedia.org/wiki/File%3AHet_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg">Wikimedia Commons</a>.
</p>
