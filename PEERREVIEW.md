# Peer Review
*2026-03-16*

This pass replaces the prior review with a fresh audit of:

- the shipped `run_tests` battery
- the generated `TESTS.md` / `BENCHMARKS.md` reporting layer
- the standalone research and upstream probes under `src/research/` and `src/bin/`

The focus of this review is correctness of:

- mathematical formulas
- fidelity to the original source or paper when claimed
- comments and documentation
- whether the extra tests are actually used in the repository's workflow

`cargo test` passed locally (`94` tests). I also spot-ran:

- `cargo run --quiet --bin bib_tests -- --rng MT19937`
- `cargo run --quiet --bin upstream_tests -- --rng MT19937`
- `cargo run --quiet --bin testu01_lz -- --rng MT19937 --replications 2 --k 20`
- `cargo run --quiet --bin webster_tavares -- --rng MT19937 --samples 128`

I did not find a new high-severity defect in the core NIST / DIEHARD / DIEHARDER formulas I spot-checked against the bundled references, but I did find several real issues in the reporting layer and in the research/upstream surface.

---

## Findings

### P2 — Published battery counts and excursion math are stale

**Files**

- `scripts/parse_battery.py`
- `TESTS.md`
- `src/main.rs`

**Problem**

The published battery-count explanation no longer matches the live runner.

The current runner emits:

- `199` NIST results when excursions run
- `17` DIEHARD results
- `522` DIEHARDER results

That is `738` full-battery results, not `742`.

The stale text also says:

- `random_excursions_variant` contributes `16` per-state results, but the implementation emits `18`
- the skip path adds `24` skips, but the runner now emits only `2` skipped family results when `J < 500`
- the expected excursion count uses `J ≈ 0.564√n`, which gives `2256` at `n = 16,000,000`; that constant is too small by a factor of `√2` for the simple symmetric-walk return-to-zero asymptotic

**Evidence**

- `src/nist/random_excursions_variant.rs` defines `18` states `{-9..-1,1..9}`
- `src/nist/mod.rs` extends the result list with all `18` variant subtests
- a spot run of `run_tests --suite nist --rng "MT19937"` showed `199` NIST results
- spot runs showed `17` DIEHARD and `522` DIEHARDER results for the same generator
- when excursions are skipped, the runner reports:
  - one skipped `nist::random_excursions`
  - one skipped `nist::random_excursions_variant`

**Impact**

Any regeneration of `TESTS.md` from the current template bakes incorrect math and incorrect skip semantics back into the published results.

---

### P2 — Radar charts can fabricate missing benchmark points from fallback values

**Files**

- `scripts/make_radar.py`
- `BENCHMARKS.md`

**Problem**

The radar-chart generator silently substitutes hard-coded fallback throughput values whenever a machine is missing some `stats/<machine>/*.bench` files.

If a machine has even one measured bench row, the script still renders a polygon and labels for the whole chart, filling any missing generators with embedded reference values.

That conflicts with the documentation claim that results in `stats/<name>/` are simply "picked up automatically" by chart regeneration.

**Evidence**

- `scripts/make_radar.py` explicitly says it "falls back to the reference values"
- `load_machine_data()` returns fallback values whenever a bench file is absent
- I verified this by generating a chart from a temporary stats tree containing only `wyrand.bench`; the output still rendered fallback labels such as `JSF64 1314 MW/s`

**Impact**

Partial benchmark data can be published as if it were measured machine data.

---

### P2 — Gorilla implementation is missing the paper's aggregate 32-p-value check

**Files**

- `src/research/marsaglia_tsang.rs`
- `src/bin/gorilla.rs`

**Problem**

The repository implements the 32 per-bit Gorilla p-values, but stops there.

Marsaglia and Tsang describe a second-stage aggregate test over those 32 p-values:

- produce one p-value for each bit position
- then run an Anderson-Darling / KS-style uniformity check on those 32 values

The current code computes only the per-bit values and the CLI reduces them to:

- `min_p`
- `max_p`
- `worst_bit`
- `worst_|z|`

**Evidence**

- `src/research/marsaglia_tsang.rs` returns only `Vec<GorillaBitResult>`
- `src/bin/gorilla.rs` summarizes with extrema only
- the bundled Marsaglia-Tsang paper explicitly states that the 32 p-values are then subjected to an aggregate ADKS-style check

**Impact**

This is a partial Gorilla implementation. It can miss generators whose problem is collective non-uniformity across bit positions rather than one spectacularly bad bit.

---

### P3 — Mixed-width buffered reads silently discard bytes in several RNG adapters

**Files**

- `src/rng/squidward.rs`
- `src/rng/chacha20_rng.rs`
- `src/rng/hmac_drbg.rs`
- `src/rng/hash_drbg.rs`

**Problem**

These adapters refill as soon as `offset + N` would cross the current buffer boundary, instead of first draining the remaining bytes.

That means mixed-width access can silently discard tail bytes from the current buffer.

Example shape:

- read several `u32`s
- then request one `u64`
- the adapter discards the remaining bytes in the current digest/block and refills early

This behavior is already documented honestly in `SpongeBob`, but the same pattern still exists in the four modules above without equivalent caveats.

**Impact**

- comments overstate "sequential byte stream" behavior for general API use
- DRBG / stream-cipher adapters can advance state earlier than the prose suggests

**Non-impact**

The test battery and benchmark harness are currently unaffected because the shared collectors use `next_u32()` only.

---

### P3 — `webster_tavares` can score input bits the RNG never consumes

**File**

- `src/bin/webster_tavares.rs`

**Problem**

The CLI accepts `--input-bits` up to `64`, but several test closures immediately truncate the seed to `u32` or `i32`.

For those RNGs:

- flipping high input bits above the true seed width has no effect
- the report then interprets the unchanged output as catastrophic avalanche failure

This is not the paper's question; it is a mismatch between the declared input width and the actual seeded function under test.

**Examples**

- `MT19937`
- `SystemVRand`
- `BsdRandom`
- `WindowsMsvcRand`
- `WindowsVb6Rnd`
- `WindowsDotNetRandom`

all collapse the sampled seed width below the CLI's possible `input_bits`.

**Impact**

The probe is misleading for configurations where `input_bits` exceeds the selected RNG's actual seed interface.

---

### P3 — The local Pincus reference file is not a PDF

**File**

- `pubs/pincus-1991-approximate-entropy.pdf`

**Problem**

This file is not a paper copy at all. It is a Cloudflare "Just a moment..." HTML page saved with a `.pdf` name.

**Evidence**

- `file pubs/pincus-1991-approximate-entropy.pdf` reports `HTML document text`
- the file contents begin with HTML markup, not a PDF header

**Impact**

The repository cannot honestly claim that the Pincus implementation was audited against the local source shelf, because the cited local source is broken.

---

## Notes On The Extra Tests

### Knuth tests

The current Knuth implementations in `src/research/knuth.rs` are reasonable and I did not find a new formula defect in:

- permutation test
- gap test
- Wald-Wolfowitz runs-above/below-median test

The important limitation is usage, not immediate arithmetic: they live only in `bib_tests`, not in `run_tests`, so they are not part of the standard repository battery.

### TestU01 Hamming and Lempel-Ziv

I compared the implemented cores against the official TestU01 mirror sources:

- `sstring.c` for `HammingCorr` / `HammingIndep`
- `scomp.c` for `LempelZiv`

The important pieces matched:

- `StripB`-style bit selection
- `gofs_MinExpected = 10` lumping for the main `HammingIndep` chi-square
- the published `LZMu` / `LZSigma` tables

These remain correctly described as core-statistic ports, not full TestU01 reporting-stack reproductions.

### PractRand FPF

The current implementation is still best described as:

- faithful core counting / bucket logic
- asymptotic chi-square post-processing
- not a reproduction of PractRand's empirical calibration tables or suspicion scoring

That limitation is already documented clearly enough.

### Webster-Tavares

The dependence-matrix / BIC machinery itself looks fine for the sampled Boolean-transform problem it actually computes.

The issue is the CLI surface described above: it can mis-specify the effective input width of the tested RNG family.

### Marsaglia-Tsang Gorilla

The per-bit missing-word statistic, mean, and standard deviation are consistent with the paper.

The gap is the missing aggregate uniformity test over the 32 resulting p-values.

---

## Workflow Gap

The user specifically asked that the Knuth and other beyond-NIST/DIEHARD tests be checked **and used**.

Right now they are checked only as separate binaries:

- `bib_tests`
- `upstream_tests`
- `testu01_lz`
- `webster_tavares`
- `gorilla`

They are **not** part of:

- `tests/run_battery.sh`
- `run_tests`
- `TESTS.md`

So today the repository's standard published battery still excludes them.

That is a workflow/design gap rather than a single bad formula, but it matters: the extra tests exist, they run, and the repository does not yet integrate them into the default audit path.

---

## Bottom Line

The core classical batteries appear materially stronger than the reporting/documentation layer around them.

The main issues from this pass are:

1. stale published battery math in `TESTS.md` generation
2. benchmark charts that can silently mix real and fallback numbers
3. a partial Gorilla implementation
4. a misleading `webster_tavares` input-width surface
5. mixed-width buffered-output bugs in several RNG adapters
6. a broken local Pincus reference file

If the goal is a serious end-to-end audit tool, the next step is not just fixing prose. It is also integrating the extra Knuth / TestU01 / PractRand / Gorilla probes into the repository's standard audit workflow so they are actually used, not merely available.
