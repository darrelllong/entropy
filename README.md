# entropy

`entropy` is a pure Rust statistical test suite for pseudorandom number generators.

It aims to provide a readable, hackable implementation of the major classic batteries:

- NIST SP 800-22 Rev. 1a
- DIEHARD
- DIEHARDER

Every test is implemented here from its published description and the mathematics of its null distribution, and cites the author of its design. Where a test's reference distribution has no closed form, the crate computes or simulates it and says how. Passing these tests is evidence about statistical quality, not a proof of it.

## Installation

The crate is published on crates.io as [`rng-entropy`](https://crates.io/crates/rng-entropy); the library name is `entropy`:

```sh
cargo add rng-entropy
```

Minimal library use — construct a generator and run the NIST SP 800-22 battery on it:

```rust
use entropy::{nist, rng::Mt19937};

fn main() {
    let mut rng = Mt19937::new(5489);
    for result in nist::run_all(&mut rng, 1_000_000) {
        println!("{result}");
    }
}
```

## Dependency Note

This crate depends on Darrell Long's [`rust-mp`](https://github.com/darrelllong/rump) crate (library name `rump`) for ln Γ, and, through the default `cryptography` feature, on his [`cryptography-rs`](https://crates.io/crates/cryptography-rs) crate (library name `cryptography`, source at [darrelllong/cryptography](https://github.com/darrelllong/cryptography)).  Both are taken from sibling checkouts (`../rump`, `../cryptography`) during development, and from their published versions otherwise.  `cryptography-rs` supplies:

- **Block ciphers** used by the CTR-mode RNGs: Camellia-128, Twofish-128, Serpent-128, SM4, Grasshopper (256-bit key), CAST-128, SEED.
- **Stream ciphers**: Rabbit, Salsa20, Snow3G, ZUC-128.
- **Elliptic-curve primitives**: P-256 scalar multiplication used by Dual_EC_DRBG.
- **DRBG**: AES-256-CTR DRBG (`CtrDrbgAes256`).

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

## Running

### Full run (canonical)

```sh
tests/run_all.sh
```

Runs everything: the NIST/DIEHARD/DIEHARDER battery plus all five
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
tests/run_battery.sh --suite diehard-historical --rng MT19937   # opt-in, see below
```

The 64-bit generators return the high half of each output as a word, so the
default run never reads their low bits.  `--views` adds, from fixed seeds, four
runs of each of PCG64, Xoshiro256, Xoroshiro128, SFC64, JSF64 and Xorshift64:
high half, low half, full word and bit-reversed high half.  The views of one
generator share its outputs, so their results are not independent.
`--alternatives` adds PCG64 with five specified defects (a bit bias of
2^-11, a stuck low bit, repeated 65 536-word blocks, a lag-1 dependence
between bits and a period of 2^20 words), so the results show which tests
detect each.  `tests/run_all.sh` passes `--views` and `--alternatives`.

`--corpus <file>` tests saved output instead of the generators: the file's
bytes, read as little-endian 32-bit words.  Each suite reads its own fixed
range of words (NIST the first 500 000; every default suite fits in the first
68 157 440, 260 MiB), and a suite the file does not cover reports SKIP.
`--json` prints one JSON object per result.

### Auxiliary probes only

```sh
tests/run_aux.sh
```

Runs the five standalone research probes with their default parameters:
`bib_tests` (Knuth permutation/gap, Wald–Wolfowitz runs above/below the median, NIST ApEn profile),
`upstream_tests` (L'Ecuyer–Simard Hamming-weight correlation and independence, and Doty-Humphrey's FPF test),
`testu01_lz` (Lempel-Ziv compressibility), `webster_tavares` (SAC/BIC avalanche),
and `gorilla` (Marsaglia-Tsang Gorilla, with the paper's Anderson-Darling aggregate).
Use the individual binaries for filtered or resized runs:

```sh
cargo run --release --bin bib_tests    -- --rng AES
cargo run --release --bin upstream_tests -- --rng AES
cargo run --release --bin testu01_lz   -- --rng AES --k 27
cargo run --release --bin webster_tavares -- --samples 2048
cargo run --release --bin gorilla      -- --rng AES
```

A further standalone binary, `bitplane_complexity`, measures Berlekamp-Massey
linear complexity on each individual output bit plane across successive
64-bit outputs (`cargo run --release --bin bitplane_complexity -- --rng Xorshift64`);
it is not part of `run_aux.sh`.

### Throughput benchmarks

Throughput is measured with [pilot-bench](https://github.com/darrelllong/pilot-bench),
a statistical benchmarking harness that reports MW/s (10⁶ u32 words/second) with
confidence intervals. Build `pilot_rng` and run the benchmark script:

```sh
tests/run_benchmarks.sh                        # quick preset, skip already-measured
tests/run_benchmarks.sh --preset normal        # tighter CIs (takes longer)
tests/run_benchmarks.sh --force aes_ctr rabbit # re-measure specific generators
```

Results land in `stats/<machine>/*.bench`. After measuring, regenerate the
radar charts with:

```sh
python3 scripts/make_radar.py
```

See [BENCHMARKS.md](BENCHMARKS.md) for the full results table and radar charts.

## What The Runner Exercises

The default runner compares 43 built-in generators across six categories:

**OS entropy**
- `OsRng` (`/dev/urandom`)

**Degenerate (must fail everything)**
- `ConstantRng`, `CounterRng`

**Historical broken generators (negative controls)**
- Unix libc: System V `rand()`, `mrand48()`, BSD `random()` and Linux glibc `rand()/random()` (one generator, seeded as glibc does), FreeBSD `rand_r()` compat
- Windows: CRT `rand()`, VB6/VBA `Rnd()`, `.NET Random` compat
- Classic LCGs: ANSI C, MINSTD, Borland C++

**Quality simulation generators**
- `MT19937`, `Xorshift32`, `Xorshift64`
- `PCG32`, `PCG64`, `Xoshiro256`, `Xoroshiro128`
- `SFC64`, `JSF64`

**Cipher-based CSPRNGs** (block-CTR mode, from the `cryptography` crate)
- AES-128-CTR, Camellia-128-CTR, Twofish-128-CTR, Serpent-128-CTR
- SM4-CTR, Grasshopper-CTR, CAST-128-CTR, SEED-CTR
- Stream ciphers: Rabbit, Salsa20, Snow3G, ZUC-128

**Cryptographic DRBGs**
- `ChaCha20`, `SpongeBob` (SHA3-512), `Squidward` (SHA-256)
- `HmacDrbg`, `HashDrbg`, `CtrDrbgAes256` (AES-256-CTR DRBG)
- `DualEcDrbg` (P-256, known-backdoored — negative control)

That mix makes output useful both for regression testing and for verifying that the batteries correctly punish weak and broken constructions while passing strong ones.

## Implementation Status

| Area | What is implemented and how it is checked |
|------|------|
| NIST SP 800-22: frequency, block_frequency, runs, longest_run, matrix_rank, spectral, serial, approximate_entropy, cumulative_sums, universal, linear_complexity, non_overlapping_template (all 148 aperiodic 9-bit templates), overlapping_template, random_excursions, random_excursions_variant | The statistics of SP 800-22 Rev. 1a; the unit tests run the publication's worked examples, including those on the first 10⁶ binary digits of e, and each module explains any figure the publication prints that does not follow from its own formulas |
| Maurer (1992): parametric universal family `L=5..16` | Added alongside the NIST single setting; runs a setting only when the sample holds the `K ≥ 1000·2^L` test blocks of SP 800-22's §2.9.7 table, so at the battery's 16 Mbit `L=5..10` run and `L=11..16` skip |
| DIEHARD: birthday_spacings, binary_rank (31×31, 32×32, 6×8), bitstream, OPSO/OQSO/DNA, count_ones_stream, parking_lot, minimum_distance_2d, spheres_3d, squeeze, runs, craps | Marsaglia's statistics.  Binary-rank probabilities are exact; squeeze's cell probabilities come from an exact recurrence (`examples/squeeze_table.rs`); the monkey tests use disjoint letter fields and exact iid missing-word moments; summaries over repeated trials are Kolmogorov–Smirnov tests |
| DIEHARD historical tests: OPERM5, overlapping sums, count-the-1s on each byte offset, 6×8 rank on each byte offset | Opt-in suite (`--suite diehard-historical`); see below |
| DIEHARDER: bit_distribution, byte_distribution, dct, fill_tree, gcd, ks_uniform, lagged_sums, minimum_distance_nd, monobit2, permutations | Brown's and Bauer's statistics.  Fill-tree's distribution is exact; the GCD step-count law is estimated by a 10¹²-pair simulation (`examples/gcd_step_table.rs`); χ² cells expecting too few counts are pooled so that every observation is scored once; monobit2 combines its block lengths by Bonferroni's bound |
| Webster–Tavares (1985): strict avalanche / bit-independence probe over seeded RNG families | Research binary (`webster_tavares`); the dependence matrix and avalanche-variable correlations of the paper |
| Knuth TAOCP Vol. 2 §3.3.2 permutation and gap tests, plus the Wald–Wolfowitz (1940) runs test above/below the median | Research binary (`bib_tests`) over uniform `[0,1)` streams |
| NIST SP 800-22 §2.12 ApEn statistic swept over embedding dimensions `m=2..6` | Part of `bib_tests` |
| L'Ecuyer and Simard (2007): Lempel–Ziv compressibility | Research binary (`testu01_lz`); the LZ78 phrase count against its exact distribution for k ≤ 5 and simulated distributions above (`examples/lz78_table.rs`), through a randomized probability-integral transform |
| L'Ecuyer and Simard (1999, 2007): Hamming-weight correlation and independence | Part of `upstream_tests`; asymptotic normal correlation test with a two-sided p-value, and the weight-pair χ² with its corner statistics |
| Doty-Humphrey: floating-point-format frequency (FPF) | Part of `upstream_tests`; disjoint codewords, so the samples are independent, with per-exponent significand G-tests and an exponent-distribution G-test |
| Marsaglia and Tsang (2002): Gorilla | Research binary (`gorilla`); the missing-word counts of all 32 bit positions and the paper's Anderson–Darling aggregate |

## Important Caveats

- Passing these tests does not prove unpredictability, backtracking resistance, or cryptographic suitability.
- A single low p-value is not automatically evidence that a generator is broken.
- Some tests naturally emit families of p-values; the runner reports each member rather than flattening a family into one verdict.
- Results that share input are dependent; a cluster of failures in one family is less evidence than the same number of failures in unrelated tests.
- Open statistical work is listed in [AUDIT.md](AUDIT.md) and [SUGGESTIONS.md](SUGGESTIONS.md).

## Historical DIEHARD Tests

Four DIEHARD tests run only on request, outside the default battery:

```sh
tests/run_battery.sh --suite diehard-historical --rng MT19937
cargo run --release -- --test diehard_historical::overlapping_sums --rng PCG64
```

The modules live in [src/diehard/historical](src/diehard/historical); each documents its statistic, its null distribution and its calibration.

- **OPERM5** (`diehard_historical::operm5`): the 120 orderings of overlapping five-word windows, scored by the quadratic form in the pseudoinverse of their covariance, which the crate builds exactly by enumerating the orderings of up to nine values; df 96.  1 000 005 words.
- **Overlapping sums** (`diehard_historical::overlapping_sums`): 100 decorrelated overlapping sums of uniforms per inner test, corrected by the distribution of Φ(x) pooled over the sums, which the crate computes by inverting their characteristic functions; three Anderson–Darling layers.  199 000 words.
- **Count-the-1s on specific bytes** (`diehard_historical::count_ones_bytes`): the Q5 − Q4 statistic on the byte at each of the 25 bit offsets of a word, each offset on its own 256 004 words; 25 results.
- **6×8 binary rank on every byte offset** (`diehard_historical::rank_6x8_windows` and `…_summary`): the 6×8 rank test at each of the 25 offsets, each on its own 600 000 words, then an Anderson–Darling summary of the 25 p-values.  The summary has little power against a single broken offset; read the offset results first.

## Project Layout

- [src/math.rs](src/math.rs): special functions, KS helper, FFT support
- [src/result.rs](src/result.rs): shared result type and display logic
- [src/rng](src/rng): RNG implementations used by the harness
- [src/nist](src/nist): NIST SP 800-22 tests
- [src/diehard](src/diehard): DIEHARD tests; [src/diehard/historical](src/diehard/historical) holds the opt-in historical DIEHARD suite
- [src/research](src/research): research probes
- [examples](examples): the programs that derive or estimate the crate's reference tables
- [src/dieharder](src/dieharder): DIEHARDER tests

## Attribution

Every test names the author of its design in an `# Author` section of its doc comment, and BIB.md holds the full references.

## Reference Corpus

The standards, manuals and papers the implementations follow are kept under [pubs/](pubs), so that the code can be checked against its published sources rather than summaries.  `pubs/SOURCES.tsv` records where and when each file was retrieved, and BIB.md marks the references that are not there and why: TAOCP and Numerical Recipes are copyrighted books, a few papers are paywalled, and some publishers refuse automated download.

## References

Primary references:

- NIST SP 800-22 Rev. 1a
- George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995)
- Robert G. Brown, *Dieharder: A Random Number Test Suite* (2004–2011)
- Marsaglia and Tsang, "Some Difficult-to-pass Tests of Randomness," *Journal of Statistical Software* 7(3), 2002

Additional suites and tests surveyed (candidates for future implementation):

- L'Ecuyer and Simard, "TestU01: A C Library for Empirical Testing of Random Number Generators," *ACM TOMS* 33(4), 2007 — BigCrush contains ~106 tests including BirthdaySpacings, Gap, CouponCollector, MaxOft, LempelZiv, HammingCorr, RandomWalk, and LinearComplexity profile tests, many of which catch defects invisible to all three batteries here.
- Chris Doty-Humphrey, *PractRand*, 2018 — streaming suite; its BCFN, DC6, FPF, and TMFn tests are designed for small-state generators (xorshift*, PCG) that pass the classic batteries.
- Knuth, *The Art of Computer Programming* Vol. 2 §3.3.2 — classical tests not in NIST/Diehard: Gap, Poker (hand-type), Permutation, and the Serial Correlation Coefficient with exact variance.
- Wald and Wolfowitz, "On a Test Whether Two Samples are from the Same Population," *Annals of Mathematical Statistics* 11(2), 1940 — the runs test above/below the median that `bib_tests` runs; TAOCP's run test scores monotone run lengths instead.
- Maurer, "A Universal Statistical Test for Random Bit Generators," *Journal of Cryptology* 5(2), 1992 — the full parametric form (L=10–16) is substantially more sensitive than the single NIST-selected setting.
- Hellekalek and Wegenkittl, "Empirical Evidence Concerning AES," *ACM Trans. Modeling and Computer Simulation* 13(4), 2003 — Walsh-Hadamard spectral test; sensitive to nonlinear Boolean structure in keystream generators.
- Golić and Živković, "On the Linear Complexity of Nonuniformly Decimated PN-Sequences," *IEEE Trans. Inf. Theory* 34(5), 1988 — decimated linear complexity; directly relevant to stream ciphers and LFSR-based generators.
- Webster and Tavares, "On the Design of S-Boxes," *CRYPTO 1985* — Strict Avalanche Criterion and Bit Independence Criterion; applicable to seeded PRNGs to test differential output behavior.

The papers and manuals live under `pubs/`. Full BibTeX entries are in [BIB.md](BIB.md).

## License

BSD-2-Clause. See [LICENSE](LICENSE).

---

<p align="center">
  <a href="https://commons.wikimedia.org/wiki/File%3AHet_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg">
    <img src="https://upload.wikimedia.org/wikipedia/commons/d/dd/Het_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg" alt="Extracting the stone of madness" width="360" />
  </a>
</p>

<p align="center">
  <em>Extracting the Stone of Madness</em>, after Hieronymus Bosch. Image source: <a href="https://commons.wikimedia.org/wiki/File%3AHet_snijden_van_de_kei._Rijksmuseum_SK-A-1601.jpeg">Wikimedia Commons</a>.
</p>
