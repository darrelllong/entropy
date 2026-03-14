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
- classic LCG-style generators
- Blum Blum Shub
- Blum-Micali
- AES-128-CTR as a deterministic keystream source
- intentionally bad generators like a constant stream and a counter

That makes the output useful both for regression testing and for sanity-checking that the batteries still punish obviously bad constructions.

## Implementation Status

Status here means "how comfortable this repository should be claiming fidelity," not "whether the test compiles."

| Area | Status |
|------|--------|
| NIST SP 800-22: frequency, block_frequency, runs, longest_run, matrix_rank, spectral, serial, approximate_entropy, cumulative_sums, universal, linear_complexity | Faithful or close faithful implementations |
| NIST SP 800-22: non_overlapping_template | Faithful for all 148 aperiodic 9-bit templates with the standard `N = 8` block setup |
| NIST SP 800-22: random_excursions, random_excursions_variant | Faithful family outputs; runner emits all per-state results |
| DIEHARD: operm5, runs_float, binary_rank, birthday_spacings, bitstream, monkey tests, count_ones, craps | Faithful or close to the Dieharder reference implementation |
| DIEHARD: squeeze, overlapping_sums | Close to the Dieharder source, but Dieharder itself documents these as weak or broken tests |
| DIEHARDER: fill_tree, gcd | Faithful; runner emits both underlying sub-results |
| DIEHARDER: bit_distribution | Approximate aggregate pattern-frequency test, not Brown's `rgb_bitdist` |
| Several geometric / higher-level Dieharder-style tests | Plausible and useful, but still best treated as implementation-reviewed rather than externally validated |

## Important Caveats

- Passing these tests does not prove unpredictability, backtracking resistance, or cryptographic suitability.
- A single low p-value is not automatically evidence that a generator is broken.
- Some tests naturally emit families of p-values; the runner now preserves many of those families instead of flattening them into one fake verdict.
- A few historically famous tests are themselves weak. In particular, Dieharder explicitly calls out some classic tests as poor discriminators.

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
- Marsaglia and Tsang, "Some Difficult-to-pass Tests of Randomness"

The source PDFs and papers live under `pubs/`.

---

<p align="center">
  <a href="https://commons.wikimedia.org/wiki/File%3AHet_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg">
    <img src="https://upload.wikimedia.org/wikipedia/commons/d/dd/Het_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg" alt="Extracting the stone of madness" width="360" />
  </a>
</p>

<p align="center">
  <em>Extracting the Stone of Madness</em>, after Hieronymus Bosch. Image source: <a href="https://commons.wikimedia.org/wiki/File%3AHet_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg">Wikimedia Commons</a>.
</p>
