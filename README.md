# entropy

`entropy` is a pure Rust statistical test suite for pseudorandom number generators.

It aims to provide a readable, hackable implementation of the major classic batteries:

- NIST SP 800-22 Rev. 1a
- DIEHARD
- DIEHARDER

This is a serious audit tool, but it is not a magical oracle. Some tests are fully faithful to the published or reference implementations, some are close ports of the Dieharder source, and a small number are still approximate. The project is strongest when it is explicit about which is which.

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

This crate depends on Darrell Long's [`cryptography-rs`](https://crates.io/crates/cryptography-rs) crate (`cryptography-rs = { version = "0.7", path = "../cryptography" }`; library name `cryptography`, source at [darrelllong/cryptography](https://github.com/darrelllong/cryptography) — the sibling checkout serves local development, the published version takes over otherwise). It supplies:

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
tests/run_battery.sh --suite diehard-historical --rng MT19937   # opt-in, see below
```

### Auxiliary probes only

```sh
tests/run_aux.sh
```

Runs the five standalone research probes with their default parameters:
`bib_tests` (Knuth permutation/gap, Wald–Wolfowitz runs above/below the median, NIST ApEn profile),
`upstream_tests` (TestU01 HammingCorr/HammingIndep + PractRand FPF),
`testu01_lz` (TestU01 Lempel-Ziv), `webster_tavares` (SAC/BIC avalanche),
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
- `WyRand`, `SFC64`, `JSF64`

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

Status here means "how comfortable this repository should be claiming fidelity," not "whether the test compiles."

| Area | Status |
|------|--------|
| NIST SP 800-22: frequency, block_frequency, runs, longest_run, matrix_rank, spectral, serial, approximate_entropy, cumulative_sums, universal, linear_complexity | Faithful or close faithful implementations |
| Maurer (1992): parametric universal family `L=5..16` | Added alongside the NIST single setting (which selects `L` from the sample size over `L=6..16`); emits a result for every `L` but runs a setting only when the sample holds the `K ≥ 1000·2^L` test blocks behind SP 800-22's §2.9.7 table, so at the battery's 16 Mbit `L=5..10` run and `L=11..16` skip |
| NIST SP 800-22: non_overlapping_template | Faithful for all 148 aperiodic 9-bit templates with the standard `N = 8` block setup |
| NIST SP 800-22: random_excursions, random_excursions_variant | Faithful family outputs; runner emits all per-state results |
| DIEHARD: runs_float, binary_rank, birthday_spacings, bitstream, monkey tests, count_ones_stream, craps | Faithful or close to Marsaglia's `diehard.f` or Dieharder's C; each module documents which it follows and where it departs from DIEHARD (runs and birthday spacings count as `diehard.f` does; the monkey tests draw disjoint letter fields and use exact iid missing-word moments; summaries are KS where DIEHARD's are Anderson–Darling) |
| DIEHARD historical tests: OPERM5, overlapping sums, count-the-1s on specific bytes, 6x8 rank over 25 windows | Opt-in suite (`--suite diehard-historical`), never part of the default battery; see the inventory below |
| DIEHARDER: fill_tree, gcd | Faithful; runner emits both underlying sub-results |
| DIEHARDER: bit_distribution | Faithful `rgb_bitdist` core statistic with explicit per-width, per-pattern Vtest outputs instead of Brown's random one-pattern collapse |
| Several geometric / higher-level Dieharder-style tests | Plausible and useful, but still best treated as implementation-reviewed rather than externally validated |
| Webster–Tavares (1985): strict avalanche / bit-independence probe over seeded RNG families | Implemented as a research binary (`webster_tavares`); computes the dependence matrix and avalanche-variable correlations from the paper |
| Knuth TAOCP Vol. 2 §3.3.2 permutation and gap tests, plus the Wald–Wolfowitz (1940) runs test above/below the median | Implemented as a research binary (`bib_tests`) over uniform `[0,1)` streams |
| NIST SP 800-22 §2.12 ApEn statistic swept over multiple embedding dimensions `m=2..6` | Implemented as part of `bib_tests`; reveals at which pattern lengths a sequence departs from randomness beyond the single fixed NIST setting |
| TestU01 1.2.3 (library, 2009; paper 2007): `scomp_LempelZiv` core statistic and official empirical calibration table | Implemented as a research binary (`testu01_lz`); exact per-replication `LZ78` phrase count and TestU01 `μ/σ` normalization, but not yet the full TestU01 goodness-of-fit reporting stack |
| TestU01 1.2.3 (library, 2009; paper 2007): `sstring_HammingCorr` and `sstring_HammingIndep` core statistics | Implemented as part of `upstream_tests`; `unif01_StripB` bit fields packed into `L`-bit blocks as `sstring.c` packs them (for `L ≥ s`, ⌊L/s⌋ fields plus the leading `L mod s` bits of one more word; for `L < s`, ⌊s/L⌋ blocks per field, low bits first), so a block equals the paper's concatenated bit stream only when `s` divides `L`, as at the defaults; asymptotic normal `HammingCorr` with a two-sided p-value; and TestU01's `gofs_MinExpected=10` lumping, including its two-class fallback, for the main `HammingIndep` chi-square.  Statistics and generator calls are pinned against TestU01 1.2.3 itself |
| PractRand pre-0.95: `FPF(4,14,6)` core statistic | Implemented as part of `upstream_tests`; parses disjoint codewords (iid samples — a documented deviation from upstream's 16-bit stride-overlapped windows) with per-platter and cross-exponent G-tests, but without PractRand's empirical calibration tables/suspicion scores |

## Important Caveats

- Passing these tests does not prove unpredictability, backtracking resistance, or cryptographic suitability.
- A single low p-value is not automatically evidence that a generator is broken.
- Some tests naturally emit families of p-values; the runner now preserves many of those families instead of flattening them into one fake verdict.
- A few historically famous tests are themselves weak. In particular, Dieharder explicitly calls out some classic tests as poor discriminators.

## Historical DIEHARD Tests

Commit `3b41af8` (2026-03-14) removed three DIEHARD tests from the default battery, citing Dieharder. A later review against Marsaglia's own Fortran, `diehard.f` of January 1996 built with gfortran, found the tests sound or fixable and the reasons given for removing them inaccurate. They are back as an opt-in suite, labelled with the variant each one is, alongside DIEHARD's 25-window 6x8 rank test. No default run includes the suite, and the default battery is unchanged:

```sh
tests/run_battery.sh --suite diehard-historical --rng MT19937
cargo run --release -- --test diehard_historical::overlapping_sums_fortran --rng PCG64
```

The modules live in [src/diehard/historical](src/diehard/historical). Each module's documentation gives its departures from `diehard.f`, its goldens against the gfortran build, its null calibration and its limitations. The removed files can be read with `git show <revision>:<path>`. Each hash below is the SHA-256 of that output, followed by the git blob id.

### OPERM5: `diehard_historical::operm5_dieharder`

- **Removed:** in `3b41af8`, which deleted `src/diehard/operm5.rs`. Read it as `git show 3b41af8^:src/diehard/operm5.rs`: SHA-256 `82c52f5963eb7492bed020af6edcbb5318f5b207895760cdfc743f9cf2bcd927`, blob `a1dd0a379688b85f01974dc58287407baa278c83`.
- **Reason given then:** Dieharder describes the original overlapping DIEHARD OPERM5 as the broken test that `rgb_operm` was meant to replace.
- **Evaluated:** Dieharder does say Marsaglia's original is broken, and it is miscalibrated: the R block of `operm5d.ata` has 9 negative eigenvalues, and df 99 is wrong because the covariance has rank 96. But the module removed was Dieharder 3.31.1's corrected OPERM5, with Stephen Moenkehues' pseudoinverse and df 96, which `dieharder -l` rates "Good". It is calibrated: C·P·C = C holds against the covariance rebuilt by enumeration, and over 40 000 null streams 0.99% of p-values fell below 0.01.
- **Restored as:** that corrected version, with its computation unchanged.
- **Limitations:** it is not Marsaglia's statistic, which differs in index, matrix, degrees of freedom, number of passes and p-value convention. Words are compared unsigned where Dieharder compares them signed, which leaves the null distribution unchanged. It takes one p-sample where Dieharder takes 100, and needs 1 000 005 words.

### Overlapping sums: `diehard_historical::overlapping_sums_fortran`

- **Removed:** in `3b41af8`, which deleted `src/diehard/overlapping_sums.rs`. Read it as `git show 3b41af8^:src/diehard/overlapping_sums.rs`: SHA-256 `d6715dc7832ebc6052eed5bf0323cc7c84890671c1345756ca39e91352f5ed18`, blob `b6c38e73190be12d1e288f07465ebd69103257fd`.
- **Reason given then:** Dieharder says the test is completely useless, broken and not worth fixing, and explicitly says not to use it.
- **Evaluated:** those defects belong to Dieharder's transcription, `diehard_sums.c`, not to Marsaglia's Fortran. The transcription uses y[t−2] where `diehard.f` uses y(1), which leaves the transformed sums correlated. It also drops Marsaglia's correction table f and replaces his three Anderson–Darling layers with KS tests. The removed module copied that transcription, and 5.0% of its p-values fell below 0.01 over 200 000 null streams. `diehard.f`'s `cdosum` as written is calibrated: 0.997% below 0.01 over 100 000 streams, and 0.977% over another 100 000 on the landing tree.
- **Restored as:** `cdosum`, with y(1), the table f and three Anderson–Darling layers over 199 000 words.
- **Limitations:** arithmetic is double precision rather than `REAL*4`. It uses this crate's Anderson–Darling distribution rather than DIEHARD's older approximation, which differs by up to 0.004 at n = 10. It reports 1 − CDF where DIEHARD prints the CDF.

### Count-the-1s on specific bytes: `diehard_historical::count_ones_bytes_25_fresh`

- **Removed:** in `3b41af8`, which deleted the function `count_ones_specific_bytes` from `src/diehard/count_ones.rs` (`git show 3b41af8 -- src/diehard/count_ones.rs`). Read the file before the removal as `git show 3b41af8^:src/diehard/count_ones.rs`: SHA-256 `c1bdd59adc36aa11ffffab02b8342bee2e61b84cde5caaf069c2dd7f076bd3ab`, blob `0e13bfe60d651d6c9468b9127f9933bf083cafe7`.
- **Reason given then:** Dieharder says this byte-lane variant is effectively obsolete compared with the stream variant and `rgb_bitdist`.
- **Evaluated:** Brown's judgement has two parts (`diehard_count_1s_byte.c` lines 60–71). Unconditionally, he calls the byte test "LESS stringent than the stream version overall" and "vastly less sensitive than rgb_bitdist", which supports removing it from a battery on grounds of power. Conditionally, it "might reveal problems with specific offsets ignored by the stream test", and he "could fix the stream test to cycle through the possible bitlevel offsets and make this test completely obsolete"; Dieharder 3.31.1 still rates `diehard_count_1s_byte` "Good". So the removal reason overstated the obsolescence, but not the loss of power, and the test is valid as DIEHARD scores it. The removed function was not DIEHARD's test either: it read one lane, `w & 0xFF`, and scored Q5 alone as χ²(3124), which overlapping words do not support: over 20 000 null streams, 2.95% of its p-values fell below 0.01. Over 10 000 null streams the restored test's 250 000 window p-values fell below 0.01 in 1.008%, and in 1.042% over another 10 000 on the landing tree.
- **Restored as:** DIEHARD's `wknt1s`, with Q5 − Q4 on all 25 byte windows, bits 1–8 through 25–32: 25 results and no summary.
- **Limitations:** each window reads its own 256 004 words, 6 400 100 in all. DIEHARD instead rereads nearly the same words for every window, which leaves its 25 results weakly dependent: simulated with every window on identical words, as the aligned gfortran build reads them, adjacent windows' Q5 − Q4 correlate at 0.024. The p-value is two-sided where DIEHARD prints Φ(z). The test adds 25 result slots per generator.

### 6x8 binary rank over 25 windows: `diehard_historical::rank_6x8_25_fresh` and `diehard_historical::rank_6x8_25_fresh_summary`

- **Not removed:** the default battery's `diehard::binary_rank_6x8` reads only DIEHARD's last window, bits 25–32, and stays as it is (AUDIT.md item 10). The fidelity review recommended DIEHARD's full sweep.
- **Restored as:** `cdbinrnk` over all 25 windows, each on its own 600 000 words: 25 window results, then one Anderson–Darling summary of their p-values, the layout DIEHARD prints. Over 10 000 null streams on the landing tree, 1.012% of the 250 000 window p-values and 1.05% of the summaries fell below 0.01, and 22.2% of streams had some window below 0.01, as independent windows predict. With every window on identical words, as the aligned gfortran build reads them (DIEHARD's own rereads start 128 to 3 520 words later and shift matrix boundaries by 0, 2 or 4 words), the summary fell below 0.01 in 2.38%.
- **Limitations:** it needs 15 000 000 words and adds 26 result slots. The summary alone has little power against one broken window: an adversarial review found that with one of 25 p-values set to 0 the summary fell below 0.01 in only 7.9% of simulated draws, and in 10.0% of 600 streams whose window 8 was broken. The window results catch it: in 1 000 such streams window 8's own result fell below 10⁻¹⁰ every time, while the summary fell below 0.01 in 8.2%. Read the window results first. Fresh words replace DIEHARD's rereads, and the cell probabilities are exact where DIEHARD's have six digits.

`DIEHARDER`: nothing removed. Deprecated internals such as the Kuiper KS path are intentionally not exposed as active tests in this crate.

## Project Layout

- [src/math.rs](src/math.rs): special functions, KS helper, FFT support
- [src/result.rs](src/result.rs): shared result type and display logic
- [src/rng](src/rng): RNG implementations used by the harness
- [src/nist](src/nist): NIST SP 800-22 tests
- [src/diehard](src/diehard): DIEHARD tests; [src/diehard/historical](src/diehard/historical) holds the opt-in historical DIEHARD suite
- [src/dieharder](src/dieharder): DIEHARDER tests

## Attribution

Functions adapted from DIEHARD or DIEHARDER include `# Author` citations in their doc comments. The goal is not to erase provenance behind a Rust rewrite.

## Reference Corpus

This repository keeps a local reference shelf under [pubs/](pubs) so people can check the implementation work against the actual standards, manuals, source releases, and papers instead of trusting summaries.

Included now:

- standards and specifications: `NIST-SP-800-22r1a.pdf`, `NIST-SP-800-90-2006.pdf`, `NIST-SP-800-90-2007.pdf`, `NIST-SP-800-90Ar1.pdf`, `NIST-SP-800-90B.pdf`, `NIST-SP-800-90C.pdf`, `NIST-FIPS-140-3.pdf`, `NIST-FIPS-197.pdf`, `NIST-SP-800-38A.pdf`, `NIST-FIPS-180-4.pdf`, `NIST-FIPS-202.pdf`, `NIST-CAVP-drbgtestvectors-no_reseed-HMAC_DRBG.rsp`, `rfc4503-rabbit.txt`, `rfc8439-chacha20-poly1305.txt`, `etsi-sage-snow3g-spec-v1.1.pdf`, `etsi-sage-snow3g-testdata-v1.1.doc`, `etsi-sage-zuc-spec-v1.6.pdf`, `etsi-sage-zuc-testdata-v1.1.pdf`
- battery sources and manuals: `Diehard.zip`, `diehard-fortran-1996.tar.gz`, `diehard-f2c-source-1996.tar.gz`, `diehard-c-wang-1998.tar.gz`, `diehard-doc.txt`, `diehard-tests.txt`, `dieharder-3.31.1.tgz`, `dieharder-manual.pdf`, `dieharder-tests.txt`, `NIST-STS-2.1.2-src-and-constants.zip`, `TestU01-2009-57e98bf33880.tar.gz`
- generator reference code: `mt19937ar.c`, `mt19937ar.out`, `vigna-xoshiro256starstar.c`, `vigna-xoshiro256plusplus.c`, `vigna-xoroshiro128plus.c`, `vigna-xoroshiro128starstar.c`, `vigna-xoroshiro128plusplus.c`, `vigna-splitmix64.c`, `pcg-c-83252d9c23df.tar.gz`, `wyhash-e4764a0b637d.tar.gz`, `jenkins-2007-smallprng.html`, `glibc-2.40-random.c`, `glibc-2.40-random_r.c`, `glibc-2.40-rand.c`, `glibc-2.40-COPYING.LIB`, `freebsd-0d022baa047a-random.c`, `freebsd-0d022baa047a-rand.c`, `gsl-2.8-rng-subset.tar.gz`, `v7-unix-programmers-manual-vol1.pdf`
- papers: `lecuyer-simard-2007-testu01.pdf`, `maurer-1992-universal-test.pdf`, `marsaglia-1985-current-view-keynote.pdf`, `marsaglia-zaman-1993-monkey-tests.pdf`, `marsaglia-tsang-2002-difficult-tests.pdf`, `marsaglia-2003-xorshift-rngs.pdf`, `marsaglia-tsang-wang-2003-kolmogorov-distribution.pdf`, `marsaglia-marsaglia-2004-anderson-darling.pdf`, `marsaglia-marsaglia-2004-ADinf.c`, `marsaglia-marsaglia-2004-AnDarl.c`, `marsaglia-2004-normal-distribution.pdf`, `marsaglia-2004-normal-distribution-sources.c`, `marsaglia-tsang-2002-tuftests.c`, `webster-tavares-1985-sbox-design.pdf`, `wald-wolfowitz-1940-runs.pdf`, `matsumoto-nishimura-1998-mersenne-twister.pdf`, `blackman-vigna-2021-scrambled-linear-prngs.pdf`, `oneill-2014-pcg.pdf`, `bernstein-2005-salsa20-spec.pdf`, `bernstein-2008-chacha.pdf`, `rabbit-estream-description.pdf`, `bernstein-lange-niederhagen-2015-dual-ec.pdf`, `kim-umeno-hasegawa-2004-nist-sts-corrections.pdf`, `park-miller-1988-good-ones-hard-to-find.pdf`, `lecuyer-simard-1999-beware-lcg-multipliers.pdf`, `hellekalek-wegenkittl-2003-empirical-evidence-aes.pdf`, `pincus-1991-approximate-entropy.pdf`, `hughes-2022-badrandom-the-effect-and-mitigations-for-low-entropy-random-numbers-in-tls.pdf`

`pubs/SOURCES.tsv` records where and when each file added on 2026-09-11 was retrieved.  BIB.md marks the references that are still not here and why: TAOCP and Numerical Recipes are copyrighted books, a few papers are paywalled, and some publishers refuse automated download.

When the code claims fidelity to a published test, these are the documents the project is expected to match.

## References

Primary references used by the code and audit:

- NIST SP 800-22 Rev. 1a
- George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995)
- Robert G. Brown, *Dieharder* 3.31.x source
- Marsaglia and Tsang, "Some Difficult-to-pass Tests of Randomness," *Journal of Statistical Software* 7(3), 2002

Additional suites and tests surveyed (candidates for future implementation):

- L'Ecuyer and Simard, "TestU01: A C Library for Empirical Testing of Random Number Generators," *ACM TOMS* 33(4), 2007 — the current gold standard; BigCrush contains ~106 tests including BirthdaySpacings, Gap, CouponCollector, MaxOft, LempelZiv, HammingCorr, RandomWalk, and LinearComplexity profile tests, many of which catch defects invisible to all three batteries here.
- Chris Doty-Humphrey (Crow), *PractRand* pre-0.95, 2018 — streaming suite; BCFN, DC6, FPF, and TMFn tests are designed specifically for small-state generators (xorshift*, PCG) that pass all classic batteries.
- Knuth, *The Art of Computer Programming* Vol. 2 §3.3.2 — classical tests not in NIST/Diehard: Gap, Poker (hand-type), Permutation, and the Serial Correlation Coefficient with exact variance.
- Wald and Wolfowitz, "On a Test Whether Two Samples are from the Same Population," *Annals of Mathematical Statistics* 11(2), 1940 — the runs test above/below the median that `bib_tests` runs; TAOCP's run test scores monotone run lengths instead.
- Maurer, "A Universal Statistical Test for Random Bit Generators," *Journal of Cryptology* 5(2), 1992 — the full parametric form (L=10–16) is substantially more sensitive than the single NIST-selected setting.
- Hellekalek and Wegenkittl, "Empirical Evidence Concerning AES," *ACM Trans. Modeling and Computer Simulation* 13(4), 2003 — Walsh-Hadamard spectral test; sensitive to nonlinear Boolean structure in keystream generators.
- Golić and Živković, "On the Linear Complexity of Nonuniformly Decimated PN-Sequences," *IEEE Trans. Inf. Theory* 34(5), 1988 — decimated linear complexity; directly relevant to stream ciphers and LFSR-based generators.
- Webster and Tavares, "On the Design of S-Boxes," *CRYPTO 1985* — Strict Avalanche Criterion and Bit Independence Criterion; applicable to seeded PRNGs to test differential output behavior.

The source PDFs, manuals, and source archives live under `pubs/`. Full BibTeX entries and implementation notes are in [BIB.md](BIB.md).

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
