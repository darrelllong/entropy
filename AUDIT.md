# Code Audit — 2026-09-10

Scope: every file under `src/`, `tests/`, `scripts/`, `.github/`, and the
manifest, at commit `40447e3` plus the cleanup recorded below.  Method: full
read of each module by four independent reviewers, formulas and constants
compared against the primary sources in `pubs/` (SP 800-22 Rev 1a text,
SP 800-90A, Marsaglia's `tests.txt`, the Dieharder 3.31.1 C sources,
Marsaglia–Tsang 2002, Webster–Tavares 1985) and against TestU01's
`scomp.c`/`sstring.c`, which were not in `pubs/` then (item 12), and cheap
claims executed against the built crate and against the macOS libc where a
reference implementation exists.

On 2026-09-10 `pubs/` held no DIEHARD source.  `pubs/Diehard.zip` holds DOS
binaries, data and documentation only, so the DIEHARD findings of that date
rest on `tests.txt` and on strings extracted from `diehard.exe`, not on
Marsaglia's code.  His Fortran (`diehard.f`, January 1996), its f2c
translation and Dagang Wang's 1998 C were recovered on 2026-09-11 from the
Internet Archive's copy of `stat.fsu.edu/pub/diehard` (f5ecb2a);
`pubs/SOURCES.tsv` records each archive's origin and sha256 (ffb3aee).  The
comparison of the DIEHARD modules with that Fortran is recorded in
"Follow-up — 2026-09-11".

Baseline state: `cargo clippy --all-targets -D warnings` clean,
`cargo fmt --check` clean, `cargo test --release` 153/153, no `unsafe`,
no debug output, no TODO/FIXME markers, no dead files in `pubs/` or
`stats/`.

Every finding cites `file:line`.  **CONFIRMED** means it was reproduced by
running code or checked digit by digit against the source; **PLAUSIBLE**
means it rests on code reading alone.

## Cleanup performed

| Removed | Reason |
|---|---|
| `tests/build_radar_svg.py` | March 2026 one-chart radar script, never referenced; superseded by `scripts/make_radar.py`, which produces all three SVGs in `assets/`. |
| `src/bench_rngs.rs` and its `[[bin]]` entry | Self-described legacy in-process benchmark over 18 generators with drifting labels and a stale "43 generators" count; undocumented in README/USAGE; the canonical path is `pilot_rng` + `scripts/bench_rngs.sh`. |

Kept after inspection: `scripts/r_report_analysis.py`,
`scripts/r_report_summary.py`, and `scripts/r_gap_test_diagnostic.R` are
not wired into `run_r_report.sh` but were deliberately maintained in the
July 2026 hardening commit and parse old and new reports.

## Bugs (reachable panics, hangs, or wrong verdicts)

All six items below were fixed on 2026-09-10; each fix carries a regression
test or a reproducible CLI check, and the patch went through repeated rounds of
adversarial review whose findings are folded into the notes below.  The line numbers
are those of the audited revision.

1. **HMAC_DRBG panics on legal input lengths** — `src/rng/hmac_drbg.rs:169-183`.
   `drbg_update` copies `provided_data` into a fixed 128-byte stack buffer
   guarded only by `debug_assert!`.  A personalization string over 47 bytes
   (`from_entropy`, line 92) or additional input over 95 bytes (`generate`,
   line 118) panics in release with an out-of-range slice.  SP 800-90A
   allows 2^35 bits.  CONFIRMED.
   **Fixed:** messages up to 81 bytes still use a stack scratch (so the patch adds no allocation of its own on the streaming path; the HMAC computations inside cryptography-rs still allocate); longer ones use a heap buffer sized to the input, and both are wiped after use.  A known-answer test pins the output for a 200-byte personalization string and 1 KiB of additional input against an independent from-spec replica, so a fix that silently truncated would fail.

2. **Non-overlapping template entry points unguarded on `m`** —
   `src/nist/non_overlapping_template.rs:54, 73-75, 100-106`.
   `non_overlapping_template(bits, 0)` indexes `template[m-1]`; `m >= 64`
   overflows `1u64 << m` (masked shift in release gives a silently wrong
   chi-square); `non_overlapping_template_raw(bits, &[])` underflows
   `2*m-1` and in release the matcher never advances, an infinite loop.
   CONFIRMED (panics reproduced; loop by reading).
   **Fixed:** both entry points return a skipped result unless `m` is in 2..=21 (the template lengths the NIST STS ships) and the template contains only 0/1 symbols (a non-binary template can never match and previously rejected any stream).  The χ² is now summed over exactly N = 8 blocks; for n < 64 `chunks_exact` could produce up to 10 blocks at the template lengths now accepted.  Tests cover the length bounds, bad symbols, the short-stream block count, and the §2.7.8 count example.

3. **Dual_EC accepts `outlen` below 32 bits, then panics on first draw** —
   `src/rng/dual_ec.rs:83-88, 200-210`.  `new` admits any positive multiple
   of 8 while `next_u32` assumes a block holds at least 4 bytes.
   CONFIRMED (`outlen = 8` panics).
   **Fixed:** `new` requires a multiple of 8 in `32..=max_outlen`, with `max_outlen` 240 / 368 / 504 for P-256 / P-384 / P-521 (the values SP 800-90 gives and the constructors already used), computed as `8·⌊(seedlen − 13 − log₂h)/8⌋`, a rule that reproduces them.  Both SP 800-90 editions are now in `pubs/`, and the values match Table 4 of the March 2007 revision.  Tests pin the three table values, reject 0, 8, 12, 16, 24, 248, 256 and 264 on P-256 with Q ≠ P, and check that `next_u32` splices words correctly across 5-byte blocks.

4. **`upstream_tests` CLI has no range validation** —
   `src/bin/upstream_tests.rs:35-110`.  `--hi-d 9` or `--hi-n 10` abort
   with a library panic (exit 101) instead of the documented exit 1;
   `--hc-l 0` silently yields a NaN correlation (`testu01_hamming.rs:165`).
   The sibling binaries validate their flags.  CONFIRMED.
   **Fixed:** every flag is range-checked before the library is called (exit 1 with the flag's name), with `r` checked on its own before `r + s` so the check itself cannot overflow (the same guard was added to the library asserts and to `testu01_lz`).  `hamming_indep` now bounds `L` to 1..=4096 (the pair table is `(L+1)²` words; an unbounded `L` was killed by the OOM killer) and computes the binomial weights in log space (`2⁻ᴸ` underflowed to 0 at `L ≥ 1075`, which zeroed every expectation and fabricated a rejection).  The help text lists all ten flags with defaults and constraints.

5. **SAC probe misdeclares seed widths** — `src/bin/webster_tavares.rs:143-202`.
   ANSI C LCG, MINSTD and VB6 are declared 32-bit but `Lcg32::new` keeps
   31 bits, MINSTD reduces mod 2^31-1, and VB6 keeps 24 bits.  The
   "input bits exceed seed width" warning therefore never fires, and
   the dead input bits are reported as non-avalanching.  CONFIRMED.
   **Fixed:** ANSI C is declared 31 bits (masked), VB6 24 (masked), `mrand48` 32 (srand48 fills only the high 32 bits of the state), and MINSTD 64: its `mod 2³¹−1` reduction folds every seed bit into the state rather than discarding any, so the original audit statement that MINSTD truncates was wrong.  The comment now states the criterion: a bit is dead only when it is discarded, not when the seed map is many-to-one.  The warning fires for ANSI C and VB6 at the default 32-bit input.

6. **Craps can loop forever** — `src/diehard/craps.rs:138-147`.
   `play_craps` has no throw cap; a stream that sets a point and never
   rolls that point or 7 hangs the battery.  DIEHARD and Dieharder share
   the flaw.  PLAUSIBLE (not triggered by any registry generator).
   **Fixed:** a game is cut off at 1 000 throws and scored as a loss in the ≥22 cell (an honest generator reaches the cap with probability about 2.6 × 10⁻¹²⁶ per game); a rigged-stream test pins the cap and checks that a full run on that stream terminates.  That run is also rejected, but because its other games are all two-throw wins, not because of the cap.

### Found while fixing

A. **Hash_DRBG hand-rolled its 440-bit arithmetic** —
   `src/rng/hash_drbg.rs:242-273`.  The Hashgen counter, `V + w`, and
   `V + H + C + reseed_counter` were byte-wise carry loops.  **Fixed:** all
   three go through rump's `BigUint` (`+=`, `low_bits(440)`,
   `to_be_bytes_padded`).  New tests pin carries across limb boundaries,
   truncation of the partial top limb, mixed-width addends, and the Hashgen
   counter wrap.  Dual_EC's byte padding, `rightmost(outlen)` truncation,
   and hex decoding were moved onto rump as well.  Built against the
   committed sibling crates, streaming Hash_DRBG runs at about 87% of the
   hand-rolled baseline's throughput on this machine, because each rump
   conversion allocates.

B. **Withdrawn: "state wiping was compiled out."**  This was diagnosed
   against uncommitted edits in the sibling cryptography checkout, which at
   the time gated both `zeroize_slice` and rump's limb scrubbing behind a
   new opt-in `wipe` feature.  Committed cryptography-rs 0.7 (342989a, the
   revision CI checks out) has no such feature: `zeroize_slice` is an
   unconditional volatile write and the crate always enables
   `rust-mp/wipe`, so the manifest comment and `DualEcDrbg`'s drop docs
   were correct.  Enabling the feature here broke dependency resolution
   against the committed crate and was reverted.  A test now fails if
   `zeroize_slice` stops clearing memory.  The same uncommitted work later
   made `zeroize_slice` unconditional again but still left rump's limb
   scrubbing opt-in.  If it lands that way, this crate will need
   `features = ["wipe"]`, and no test here would notice: the scrubbing
   itself cannot be observed from safe code, and no test inspects the
   resolved dependency features.

C. **Streaming paths and Dual_EC output were unpinned** — the DRBG
   known-answer tests reached only `generate`, and no Dual_EC output was
   tested.  **Fixed:** 4096-word streaming goldens for Hash_DRBG and
   HMAC_DRBG, and a fifteen-word P-256 known-answer test for the battery's
   Dual_EC seed, all from independent Python replicas.  The three Dual_EC Q
   literals are checked to lie on their curves.

D. **DRBG module docs denied backtracking resistance** —
   `src/rng/hash_drbg.rs:27-33`, `src/rng/hmac_drbg.rs:30-34`.  Both said a
   memory compromise exposes all past output.  SP 800-90A §8.8 designs
   every DRBG mechanism for backtracking resistance, and both
   implementations run the one-way update after each generate step, so a
   compromise exposes future output and only the already-generated bytes
   still in the output buffer.  What they lack is prediction resistance,
   because nothing reseeds.  CONFIRMED against the PDF.  **Fixed:** both
   module docs now say so.  The Hash_DRBG doc also no longer describes
   Hashgen as hashing a counter concatenated with V, and the Dual_EC citations name Table 4 and Appendices A.1.1–A.1.3 of the March 2007 edition, now in `pubs/` (item 21).

E. **Pcg64 did not match O'Neill's reference** — `src/rng/pcg.rs`.  It
   permuted the state before the LCG step, while pcg-c's 128-bit generator
   permutes the advanced state, so the stream carried one extra leading
   value.  **Fixed:** pinned to pcg-c's published `check-pcg64.out` for
   seed (42, 54).  Pcg64's output changes; the battery seeds it from the OS,
   so battery results are unaffected.  R-REPORT.md's PCG64 section was measured from the old stream and now says so.

Items F–N were found in the 2026-09-11 follow-up ("Follow-up — 2026-09-11").
Their line numbers are those of 229e104 unless another revision is named.

F. **Runs up/down counted only one of the two final runs** —
   `src/diehard/runs_float.rs:148-154, 188-194` at f098126.  At the end of a
   sequence the counter closed only the down-run when the last word exceeded
   the first, and only the up-run otherwise, which is Dieharder's rule
   (`diehard_runs.c:132-143`).  Marsaglia's `udruns` counts both
   (`diehard.f:529-530`), so every word lies in one up-run and one down-run,
   as AS 157's covariance assumes; one missing count moves the nearly
   singular quadratic form by O(1).  CONFIRMED: a gfortran build of
   `diehard.f` on the same words gives identical in-loop counts and one
   different final count, and 200 000 simulated null sequences put the
   up-run statistic's mean at 7.14 and its variance at 23.6, against 6
   and 12.  Over 48 000 null calls on MT19937 streams, the ten-sequence KS
   p-value fell below 0.01 in 2.72% (up) and 2.54% (down) of calls, and
   below 0.001 in 0.43% and 0.37%.
   **Fixed:** both final runs are counted, following `diehard.f:529-530`
   (3f7fdc0; now `runs_float.rs:202-203`).  The same 48 000 calls give 1.02%
   and 0.97% below 0.01, and 0.11% and 0.09% below 0.001 (standard errors
   0.045% and 0.014%).  The reviewer's independent port of `udruns` matched
   the counts bit for bit on 19 streams.  The doc adds that DIEHARD runs the
   block of ten sequences twice, and that its single-precision `REAL`
   comparison merges nearby words into ties, which it counts as falls: the
   `REAL`s of w and w + 1 are equal for 96.5% of 2²⁶ random words (3f3f2a3,
   39060f6).  Output changes for `diehard::runs_up` and `diehard::runs_down`.

G. **Linear complexity took the sign of its mean from STS, not SP 800-22** —
   `src/nist/linear_complexity.rs:83`.  The code added (9 + (−1)^M)/36 and
   its comment said SP 800-22 prints that.  §2.10.4 prints
   (9 + (−1)^(M+1))/36, which is §3.10's (4 + M mod 2)/18 and gives the
   worked μ = 6.777222 for M = 13; STS 2.1.2's `linearComplexity.c` has the
   other sign.  CONFIRMED against the PDF and the STS source.  The sign never
   changes a class: under the printed mean every Tᵢ lies within
   (M/3 + 2/9)/2^M of an integer, the other sign lowers it by 1/18, and the
   class boundaries are half-integers.
   **Fixed:** the mean follows §2.10.4 (8cf4be3), and tests check μ and Tᵢ
   for M = 13 and for the battery's M = 500 (f281a34).  f64 checks found no
   class that differs over every M in 500..=5000 (the implementer's) or
   M = 1..5000 (the reviewer's), and p-values on e, MT19937 streams and
   `run_all` are bit-identical.

H. **System.Random panicked on overflow in debug builds** —
   `src/rng/c_stdlib.rs:258-300` (`WindowsDotNetRandom::new`).  C# evaluates
   `int` arithmetic unchecked, but the port subtracted with checked `i32`
   arithmetic, so a debug build panicked with "attempt to subtract with
   overflow" for large-magnitude seeds, including in the `webster_tavares`
   probe, which seeds the generator across the whole `i32` range.  Release
   builds wrapped and already matched .NET.  CONFIRMED (panic reproduced).
   **Fixed:** f8ae1d9 wrapped the seed rounds and `InternalSample`.  Review
   then brute-forced every seed magnitude: only the correction rounds'
   subtraction `seed_array[i] − seed_array[1 + n]` ever leaves `i32`, from
   magnitude 161 844 078 up, for 1 269 681 342 of the 2³¹ magnitudes.
   d3b614b keeps a single `wrapping_sub` there (`:300`) and makes the other
   steps plain arithmetic again, so a debug build still panics if that
   analysis ever stops holding.  An exhaustive sweep of the fixed generator
   over every seed magnitude with overflow checks found nothing else out of
   range.  A test pins five raw samples for four overflowing seeds against a
   replica of the .NET Framework reference source (`random.cs`) alone, since
   no .NET runtime was available; the replica reproduces the seed-1 prefix
   and the widely cited first outputs for seeds 0 and 42.

I. **ChaCha20Rng let its block counter wrap** —
   `src/rng/chacha20_rng.rs:140-144`.  RFC 8439 §2.3 limits one key and nonce
   to 2³² blocks.  The generator let `cryptography::ChaCha20` wrap its 32-bit
   counter to block 0 and repeat keystream, and the test added with
   `ChaCha20Rng::new` (24e8afe) pinned that wrap.  Committed cryptography
   (342989a) wraps, but the cryptography tree in preparation panics there,
   which would fail that test and the sibling's job that builds this crate.
   CONFIRMED against both trees.  The battery draws nowhere near 2³² blocks.
   **Fixed:** the generator counts the blocks it takes and panics with its
   own message before it asks for a block past counter 2³² − 1, so it behaves
   the same whichever sibling cryptography version it builds against
   (8c548a8).  The wrap test became two: `new(.., u32::MAX)` serves exactly
   the cipher's block at that counter, and the seventeenth read panics.  With
   the check removed, the panic test fails against both trees.

J. **The universal-constants test repeated the table's own rounding** —
   `src/nist/universal.rs:359` (`constants_match_maurer_series`).  Added in
   2cca76f, the test summed Maurer's series first to last in plain f64, which
   reproduces the L = 6..16 table entries to about 2.5 × 10⁻¹², so it showed
   only that the table matched itself; the module doc and 2cca76f's message
   then said the entries equal the series to 10⁻¹¹.  CONFIRMED: a Neumaier
   compensated sum, which agrees with a 40-digit decimal evaluation to
   3.1 × 10⁻¹⁴, puts the table 4.0 × 10⁻¹¹ from the series in μ and
   5.6 × 10⁻¹⁰ in σ², both at L = 16, and σ² more than 10⁻¹¹ away at every
   L ≥ 11.
   **Fixed:** the test's reference is the compensated sum, itself checked
   against the decimal values (a3f907c), and each row is held to twice its
   own measured gap, never below 10⁻¹³: from 2.4 × 10⁻¹³ (σ², L = 7) to
   1.14 × 10⁻⁹ (σ², L = 16) (37ad21a).  Adding 5 × 10⁻¹⁰ to the L = 10 σ²
   entry now fails.  The table is unchanged, so no result changes.

K. **`monobit2` inherits a calibration defect from `chisq_binomial`** —
   `src/dieharder/monobit2.rs:40, 62-68`.  Dieharder's `chisq_binomial`
   (`chisq.c:166`) scores a cell only when its observed count exceeds 10, so
   each block size's p-value is not uniform under H₀.  The earlier doc
   (bde3347) called the reported p-value "somewhat heavy near zero" and put
   the cause elsewhere.  CONFIRMED by null simulation on MT19937: level 0
   alone, before the Šidák step, fell below 0.01 in 1.44% of 20 000 trials at
   2 000 words, and the reported p-value in 1.27–1.50% of trials: 1.30% of
   20 000 at 2 000 words (the review measured 1.26% there), 1.27% of 20 000
   at 10 000, 1.50% of 5 000 at 100 000 and 1.50% of 1 000 at 1 000 000.
   The shared cells of item 41 can also turn a
   pass into a fail: 100 000 MT19937 words with six all-ones level-0 blocks
   and six all-zeros level-1 blocks written in score p = 0 at seeds 1, 2, 3
   and 5489, against 0.345, 0.435, 0.140 and 0.848 with separate histograms.
   Such a stream is far from random, so that is extra sensitivity outside the
   null, not a false alarm under it.
   **Fixed:** the doc states both (5bd2bb6), and a second golden at 1 140
   words, the fewest with two block sizes, covers the flat layout's second
   segment (7424f0f).  Both behaviours are kept for fidelity, as the
   fill-tree off-by-one (14) is.

L. **DIEHARD departures were undocumented or misstated** — `src/diehard/`.
   Checked against `diehard.f`, several modules departed from DIEHARD without
   saying so, or described it wrongly:
   - `bitstream.rs`: DIEHARD feeds each word low bit first into one continuous
     stream and prints 20 `phi` values with no summary (`diehard.f:122-138`);
     the module reads each word high bit first, as Dieharder does, over 20
     disjoint chunks with a KS summary.
   - `count_ones.rs`: `sknt1s` scores 2 560 000 five-letter words, twice, and
     takes each word's bytes high byte first (`diehard.f:214-215, 767-780`);
     the module scores 256 000 words once, low byte first, as `tests.txt` and
     Dieharder do.  Its doc credited μ and σ to a C source; they are
     Marsaglia's Fortran (`diehard.f:808`).
   - Summaries and p-values: DIEHARD's `KSTEST` is Marsaglia's
     Anderson–Darling statistic (`diehard.f:1668-1709`), and DIEHARD reports
     CDF values throughout, where the modules report KS summaries and upper
     or two-sided tails.
   - `monkey.rs`: DIEHARD sweeps the letter's bit field over 23, 28 and 31
     positions for OPSO, OQSO and DNA (`diehard.f:689`), and Dieharder rates
     all three "Suspect".
   - `parking_lot.rs`: DIEHARD makes 12 001 attempts (`diehard.f:304-315`),
     the module and Dieharder 12 000, which moved the mean of 3 000 simulated
     lots by 0.08 cars against σ = 21.9.
   - `craps.rs` (item 18), `squeeze.rs` (item 8) and `birthday_spacings.rs`
     (item 27).
   - `minimum_distance.rs` quoted a Dieharder sentence, "The formula used
     here is WRONG.", that Dieharder does not contain, and the 32×32 comment
     in `binary_rank.rs` described NIST's three cells where the code, DIEHARD
     and Dieharder use four.
   - Window reuse: the 6×8 doc (3b63d1b) and the fidelity review (its §5 and
     §6.3) said DIEHARD rereads the same words for every window, and the
     birthday doc (2611fd7) said `cdbday` rewinds for each.  Neither holds.
     `jkreset` (`diehard.f:425-427`) resets the record counter but keeps the
     buffer index, so a window that starts mid-record first reads out the
     rest of that 4 096-word record and only then rereads the file from
     word 1.  An instrumented gfortran build starts 6×8 window 2 at word
     600 001 and window 25 at word 596 481, and count-the-1s on specific
     bytes has leftovers of 1 933 to 4 086 words.  `cdbday`'s 256 000 words
     end mid-record, so its windows alternate: five read words 1 to 256 000,
     and four read the 2 048 words after them and then words 1 to 253 952.
     The windows overlap in most of their words, so their p-values are
     dependent.

   CONFIRMED against gfortran 16.2 builds of `diehard.f` on the same
   4 000 000 words, with bit and byte order aligned and bit 31 flipped for
   the float tests (DIEHARD floats the signed word): 31×31 and 32×32 χ² 0.397
   and 6.849, 6×8 χ² 2.011, 141 957 missing bitstream words, count-the-1s
   Q5 − Q4 = 2520.02, 3 519 parked cars, all 20 minimum-distance d² to four
   decimals, all 20 3-D sphere r³ to three, and craps' 98 570 wins and 21
   throw cells agree.  The tables agree as well: squeeze's cell probabilities
   with `diehard.f` to 10⁻¹², the runs matrix exactly, and the exact 31×31
   rank probabilities with DIEHARD's table to 4.2 × 10⁻¹¹.
   **Fixed:** each module now documents its departures, and the misstatements
   are corrected (f31f192, c413022, d1684fb, ea40a1e, 3f3f2a3, 39060f6).  No
   output changes.

M. **The March removal of three DIEHARD tests rested on wrong reasons** —
   `README.md:200-209`; the removed modules at `3b41af8^:src/diehard/`
   (`operm5.rs`, `overlapping_sums.rs`, `count_ones.rs`).  Commit 3b41af8
   (2026-03-14) removed OPERM5, overlapping sums and count-the-1s on specific
   bytes, and README cites Dieharder's verdict on each.  The fidelity review
   evaluated all three against `diehard.f`, Dieharder 3.31.1 and simulation:
   - **OPERM5:** the removed module ported Dieharder's rewritten
     `diehard_operm5.c`, whose matrix is the exact pseudoinverse of the
     120-pattern covariance C (rank 96 = 5! − 4!; Dieharder's table equals
     pinv(C) to 4.5 × 10⁻¹⁰), with df 96.  That is Dieharder's calibrated
     correction, rated "Good" in `list_tests.c`, not the defunct original
     README describes.  Marsaglia's published OPERM5 is the miscalibrated
     one: its R matrix is indefinite and its df 99 exceeds the rank, and
     `diehard.f` on a good generator gave P(p > 0.99) = 4.25% over 800
     statistics.
   - **Overlapping sums:** Marsaglia's transformation is exact (M T Mᵀ = I)
     and his f table is the empirical CDF of φ(x), so `diehard.f` is
     calibrated at the resolution it reports (600 runs, final KS p = 0.81).
     Dieharder's transcription uses `y[t-2]` for `y[0]` and drops the f
     table; with its default 100 psamples it rejects 54% of perfect
     generators at 0.01.  The removed module copied that transcription and
     rejected 4.5% at 0.01.  The test was broken only by the transcription.
   - **Count-the-1s on specific bytes:** the removed module scored Q5 alone
     with df 3 124, which overlapping words do not support (in 1 000 null
     streams its mean was 3 115 and its variance 9 071 against 6 248, and
     P(p < 0.01) = 2.6%), so it was miscalibrated as this crate wrote it.
     DIEHARD's `wknt1s` scores Q5 − Q4, which the same simulation calibrates
     (two-sided P(p < 0.01) = 0.8%).  Dieharder rates its own byte test
     "Good", and its author's remark that he could make it obsolete is
     conditional.

   CONFIRMED by computation and by `diehard.f` itself run on a good
   generator (`jtbl` replaced by gfortran's RNG).
   **Fixed:** the `count_ones` doc no longer says Dieharder retired the byte
   variant (c413022).  README.md:200-209 still gives the old reasons.
   <!-- PENDING historical suite -->

N. **`math::erfc` was good only to about 10⁻⁷** — `src/math.rs:12-41`.
   `erfc`, and `normal_cdf` through it, was Numerical Recipes' `erfcc`,
   whose stated fractional error is below 1.2 × 10⁻⁷, with a transcription
   slip: its last coefficient was 0.17087294, where published copies of
   Numerical Recipes print 0.17087277.  At 0 the code returned the
   exponential of its coefficients' sum, 1.0000002 where the book's gives
   1.00000003, so a two-sided p-value erfc(|z|/√2) exceeded 1 for a zero
   statistic; HammingCorr printed p = 1.0000002 for this reason.  CONFIRMED
   by evaluating both coefficient sets and against public copies of the
   book's `erfcc`.
   **Fixed:** `erfc` and `normal_cdf` follow `cPhi` from Marsaglia's
   "Evaluating the Normal Distribution" (2004, in `pubs/` since be5c958):
   the upper tail is Mills' ratio R times the normal density, with R summed
   as a Taylor series about his tabled values at z = 0, 2, …, 16 (6806a53).
   `mills_ratio` documents three departures that f64 needs.  The series
   expands about the tabled point at or above x, not the nearest one; it
   stops only once the terms it leaves out are provably at most ε·R/4; and
   past the table an asymptotic series carries it to underflow.  The
   stopping rule came from review.  6806a53 kept `cPhi`'s stop at the first
   pair of terms that rounds away, which in f64 can be a pair where rounding
   error and the true term cancel: 1 154 of 20 million arguments in [0, 16)
   were more than 4 ulp off, up to 1.5 × 10⁻¹⁰ relative error at
   u = 14.886 (erfc at x = 10.526), and monotonicity reversed for z ≥ 8,
   with Φ rising by 1.5 × 10⁻¹⁰ between neighbouring floats near −14.886.
   6f2af66 added the provable stopping rule and reference rows for x√2 in
   (6, 16]; reverting the rule or the expansion point now fails the tests.
   Against a 70-digit decimal oracle on 2.4 million random arguments
   (e11df66), the largest errors are 3.539·(1 + x²)·ε for erfc from 0 up,
   2.560·ε below 0, 3.351·(1 + x²/2)·ε for Φ below 0 and 2.646·ε from 0 up;
   the test tolerances are 7 in the same units, about twice those.
   erfc(0) = 1 exactly and erfc(x) ≤ 1 for x ≥ 0, so a two-sided p-value no
   longer exceeds 1.  The eight goldens that pass through `erfc` or
   `normal_cdf` moved by exactly the old `erfcc` error at their arguments,
   at most 4.67 × 10⁻⁸, and were re-pinned (f015ebf); the largest move among
   the SP 800-22 pins is 1.05 × 10⁻⁷, in §2.15.8.  Merged in 89eb68b,
   83a841c and 93e6621.  **Left:** the research probes' two-sided p-values
   stay capped at 1 (6754bf8), but the comments at
   `testu01_hamming.rs:63-64, 253, 572`, `knuth.rs:358` and
   `testu01_lz.rs:247` still say `erfc` exceeds 1 by about 10⁻⁷ near 0.

## Correctness risks (statistic differs from the cited reference)

7. **Universal test sigma uses the Coron–Naccache constant while citing
   NIST** — `src/nist/universal.rs:165-171`.  Implements
   `c = 0.7 - 0.8/L + (1.6 + 12.8/L) K^(-4/L)`; SP 800-22 §2.9.4 uses
   `(4 + 32/L) K^(-3/L) / 15`, and §3.9 says the other form is not in
   the suite.  Negligible at the battery's K (p 0.282887 vs STS 0.282568
   on 10^6 bits of e) but a 40 % sigma difference at L=16, K=1000, which
   `universal_parametric_all` (line 125) permits.  The unit test
   `uses_nist_correction_factor` (line 191) pins the non-NIST value.
   CONFIRMED.
   **Fixed:** `universal_sigma` uses §2.9.4's c(L,K), which is also Maurer's
   eq. (13). A test reproduces §2.9.8's σ = 0.002703 and P = 0.427733 from the example's printed sum and table values (the Coron–Naccache form gives 0.427991), and checks that the shipped 12-digit constants give P = 0.427772 for the same sum.

8. **Squeeze drops sub-cutoff cells instead of pooling** —
   `src/diehard/squeeze.rs:59-71`.  Dieharder's `Vtest_eval` pools
   cells with expectation under 5 into a tail cell scored when the pool
   reaches 5.  At N = 100 000 the five weak cells sum to 9.28, so
   Dieharder scores 38 cells; this code scores 37 and never sees
   over-production of extreme squeeze lengths.  The same drop pattern in
   birthday spacings, binary rank, GCD and craps is numerically
   equivalent to the C at their fixed sample sizes.  CONFIRMED.
   **Fixed:** weak cells are pooled and scored as one cell exactly as
   `Vtest_eval` does (39 cells, df 38), through the Vtest routine moved
   unchanged from `bit_distribution` to `math::vtest_pvalue`. A test with
   over-produced extreme lengths now rejects (p ≈ 8 × 10⁻¹⁸⁷, where the old
   scoring gave p ≈ 1).
   The finding's 38 cells are the strong ones; with the pool the code
   scores 39.  Against `diehard.f` the pooling is Dieharder's, not DIEHARD's:
   Marsaglia's `sqeez` scores all 43 cells with no pooling, five of them
   expecting 0.98 to 3.27 counts, and reports `chisq(chsq,42)`, the CDF with
   df 42 (`diehard.f:254-272`).  The doc now says so (ea40a1e); the pooling
   stays.

9. **31x31 binary rank tests the low 31 bits** —
   `src/diehard/binary_rank.rs:127-137`.  Marsaglia specifies the
   leftmost 31 bits (`tests.txt:35-36`).  Verdicts will differ from
   DIEHARD on generators with weak low bits.  CONFIRMED.
   **Fixed:** rows are the leftmost 31 bits. Dieharder 3.31.1 has no 31x31
   test, which the doc now says. ANSI C and MINSTD, which emit zero-extended
   31-bit words, now fail this test.

10. **6x8 binary rank uses only byte 0** — `src/diehard/binary_rank.rs:56-88`.
    DIEHARD sweeps 25 byte positions and KS-combines; Dieharder uses the
    low byte.  Bytes 1–3 are never rank-tested and the doc says "a
    specified byte position".  CONFIRMED vs Dieharder.
    **Documented, not changed:** the doc says the test reads the low byte, as Dieharder's `rank.c` does, where DIEHARD repeats it over 25 overlapping 8-bit windows (bits 1–8 through 25–32, not 25 byte positions as this finding says), and that Dieharder also scores rank 3 as its own cell.
    On 2026-09-11 the doc was checked against Marsaglia's `cdbinrnk`
    (`diehard.f:920-1001`; 3b63d1b, d1684fb).  The low byte read here is
    DIEHARD's last window (kr = 0, "bits 25 to 32"), and a gfortran build on
    the same words gives the same χ² (2.011).  The doc now also says that
    DIEHARD pools ranks ≤ 4 as this test does but with six-digit cell
    probabilities, reports each window's lower tail 1 − exp(−χ²/2),
    summarizes the 25 with Anderson–Darling, and starts each window where
    `jkreset` leaves it (item L).  Sweeping all 25 windows, each on fresh
    words, belongs to the historical suite.
    <!-- PENDING historical suite -->

11. **Webster–Tavares BIC masks degenerate linear maps** —
    `src/research/webster_tavares.rs:50-54`.  A never/always-flipping
    avalanche variable forces rho = 0, so Xorshift32/64 print
    `BICmax = 0.0000`, the ideal value, precisely because they are
    GF(2)-linear.  CONFIRMED (ran binary).
    **Fixed:** pairs whose avalanche variable never or always flips are
    counted in a new `BICdegen` column and left out of BICmax and BICmean,
    which print NaN when no pair has a defined correlation. Xorshift32 no
    longer reports an ideal BIC.

12. **Hamming bit extraction is not TestU01's** —
    `src/research/testu01_hamming.rs:46-61`.  Takes the low
    `min(remaining, s)` bits of a fresh chunk per call; TestU01 packs
    `s/L` blocks per word for `L < s` and strips top bits for the tail.
    Valid statistic, but README line 190's "faithful TestU01 bit
    extraction" overstates it.  CONFIRMED against `sstring.c`.
    **Documented, not changed:** the audit read TestU01's `sstring.c` online, but the fixing pass had no copy to implement from, so the extraction stands. The README and module docs now
    describe it exactly: a block equals the paper's concatenated bit stream
    only when `s` divides `L`.
    **Fixed on 2026-09-11:** TestU01's source entered `pubs/`
    (`TestU01-2009-57e98bf33880.tar.gz`, f098126), and the blocks are packed
    exactly as `sstring.c`'s `HammingCorr_L`/`_S` and `HammingIndep_L`/`_S`
    pack them: for L ≥ s, ⌊L/s⌋ fields and then the leading L mod s bits of
    one more word; for L < s, ⌊s/L⌋ blocks from each field, low bits first.
    Where lumping leaves a single class, HammingIndep splits the pair table
    into two columns as `sstring_HammingIndep` does, instead of reporting NaN
    (e7c2a96).  Statistics and generator calls are pinned against TestU01
    1.2.3 built from `pubs/`, and the reviewer matched 25 more cases.  Output
    changes only when s does not divide L or lumping leaves one class, not at
    the `upstream_tests` defaults (s = 10, L = 300).  FPF reads the same
    stream after both Hamming tests, so non-default Hamming parameters change
    the words FPF sees and every FPF line; in one such run all eight FPF lines
    changed for MT19937, Xorshift32 and AES-128-CTR (`upstream_tests.rs:14-24`,
    2d42290).

13. **glibc `random()` seeding differs for seeds >= 2^31** —
    `src/rng/c_stdlib.rs:68-77, 342-348`.  `park_miller31` seeds via
    `u32 -> i64` (always non-negative); glibc runs Schrage on a signed
    `int32_t`, so high seeds diverge.  Seed 1 (the battery) is
    unaffected.  PLAUSIBLE.
    **Fixed:** seeding runs Schrage's step on the signed 32-bit word, pinned
    for seeds 3 000 000 000 and 2^31 − 1, and seed-1 output is unchanged.  The
    pins came from an independent C and Python replica of glibc's
    `__srandom_r`.  On 2026-09-11 glibc 2.40's own `random_r.c`, now in
    `pubs/`, was compiled: `__initstate_r` and `__random_r` match `BsdRandom`
    for 2000 outputs at each of the seeds 0, 1, 2, 12345, 2³¹ − 1, 2³¹,
    2³¹ + 1, 3 000 000 000 and 2³² − 1 (6c851e6).  The macOS libc differs at
    0, 2³¹ − 1 and 2³¹ + 1.  Current FreeBSD `random()` differs at all nine,
    because its `srandom_r` fills the table through `parkmiller32`;
    `BsdRandom` follows glibc, and its docs and README say so.
    `LcgVariant::AnsiC` equals glibc's TYPE_0 generator for seeds whose low
    32 bits are nonzero, checked at nine `u64` seeds (10cce8d).

14. **Fill-tree reproduces a Dieharder off-by-one** —
    `src/dieharder/fill_tree.rs:156-163`.  Cell 14 (expected 23.5) is
    excluded from the chi-square as in `dab_filltree.c:71`, and the
    truncated multinomial is not renormalised.  The comment acknowledges
    it; flagged as a known-wrong statistic kept for fidelity.  The
    `word_count > SIZE*2` bail-out (lines 132-135) is unreachable.
    CONFIRMED.
    **Fixed:** the bail-out is removed. The search path reaches only 15
    slots, so every trial collides by its 16th word (200 000 replica trials
    agree). The Dieharder-faithful off-by-one stays.

15. **Matrix-rank and longest-run probabilities truncated to 4 digits** —
    `src/nist/matrix_rank.rs:20-22`, `src/nist/longest_run.rs:23-31`.
    STS uses the exact values; on 10^5 bits of e the code reproduces the
    spec's counts exactly but reports p 0.531905 vs 0.532069.  CONFIRMED.
    **Fixed for matrix rank and for longest run at M = 8:** matrix-rank
    probabilities come from the §3.5 formula and reproduce §2.5.8's χ² =
    1.2619656 and P = 0.532069; longest run uses exact k/256 for M = 8 and
    reproduces §2.4.8's P = 0.180609. **Kept:** M = 128 and M = 10⁴ keep the
    four decimals §3.4 prints.  STS 2.1.2's `longestRunOfOnes.c`, in `pubs/`
    since 2026-09-11, uses the same four decimals for M = 10⁴, so the battery,
    which uses M = 10⁴, matches STS, although those values differ from exact
    ones by up to 1.6 × 10⁻³.  For M = 128 STS uses a 10-digit table within
    4 × 10⁻¹⁰ of exact, where §3.4's decimals are up to 6.4 × 10⁻⁵ off, so
    only 6 272 ≤ n < 750 000 gives p-values slightly different from STS's.  A
    test computes the exact distribution and checks both gaps (2cca76f).

16. **Approximate-entropy m gate looser than NIST** —
    `src/research/approx_entropy.rs:69-76` admits `2^m <= n/10`;
    §2.12.7 requires `m < log2(n) - 5`, i.e. `2^m < n/32`.
    `src/nist/approximate_entropy.rs:29` allows equality where the spec
    is strict.  CONFIRMED.
    **Fixed:** both gates require n ≥ 2^(m+6), following §2.12.7's "m < ⌊log2 n⌋ − 5".  A first pass worked from a text extraction that drops the floor and admitted n up to 2^(m+6) − 1; review caught it.

17. **Gorilla aggregate is KS, paper uses Anderson–Darling** —
    `src/research/marsaglia_tsang.rs:14, 113-123`.  Doc says "the
    second-stage aggregate check described in Marsaglia and Tsang".
    CONFIRMED (paper p. 6).
    **Documented, not changed:** the docs call the aggregate a KS test and a
    deviation from the paper's ADKS. Anderson–Darling needed its
    distribution's source, which was not in `pubs/` on 2026-09-10.
    **Fixed on 2026-09-11:** `pubs/` gained Marsaglia and Marsaglia's 2004
    paper on the Anderson–Darling distribution with its `ADinf.c` and
    `AnDarl.c`, and the `tuftests.c` attached to Marsaglia–Tsang 2002
    (f098126, e018c34).  `tuftests.c` shows that ADKS is the
    Anderson–Darling statistic A₃₂ of the 32 per-bit p-values, each product
    floored at 10⁻³⁰, printed as Pr(A₃₂ < A).  `gorilla_aggregate_ad`
    computes it and converts it with `math::anderson_darling_cdf`, a port of
    the paper's ADinf + errfix pinned to the paper and to the attached C
    (3f38169).  From the per-bit values printed on p. 6 it reproduces the
    ADKS printed for KISS, SHR3 (only with the floor), LFIB4 and both
    congruential generators, to within the four-decimal rounding of those
    inputs.  LFIB4 tells the conversions apart: the paper prints 0.724, as
    AD(32, ·) gives, where `tuftests.c`'s fit `ad32()` gives 0.727 and a KS
    test 0.587.  `anderson_darling_cdf` returns NaN for n < 8, where the
    method is off by up to 1.3 × 10⁻³ at n = 4 (0c52658).  errfix does not
    vanish as ADinf → 1, so past ADinf = 0.9995 (z > 6.6127) the upper tail
    is the limiting tail scaled by errfix's relative size at the switch,
    which meets the body continuously; the rule is empirical and documented
    as such.  Against `examples/anderson_darling_tail.rs`, a seeded
    simulator whose output does not depend on the thread count (08f0ef1;
    4 × 10⁹ samples for n = 8, 10⁹ for 16 and 32, 2 × 10⁸ for 64 and 128),
    the body is within 9.2 × 10⁻⁵ for z ≤ 4, and the tail errors carry signs
    and standard errors.  They are mostly positive, so p-values are mostly
    conservative, and largest just past the switch: +8.46 ± 0.07% (n = 8),
    +4.62 ± 0.14% (n = 16) and +2.46 ± 0.14% (n = 32) at z = 6.62.  The only
    resolved understatement is −0.27 ± 0.02% for n = 8 at z = 4.41, and the
    extremes past z ≈ 10 are within one run's sampling noise (833cef4,
    a54b1f2, 414a5b0, 7d39dc8).  None of this can move a verdict at
    α = 0.01, whose upper tail sits near z = 3.9.  `gorilla_all` reads all 32
    bit positions from the same words, where `tuftests.c` draws fresh words
    for each; the module documents why the aggregate's null distribution is
    unchanged, since the bits of an iid uniform word are independent, and a
    240 000-replicate simulation agrees.  For |z| > 5.4 the module also
    departs from `tuftests.c`, which stores Φ in single precision (f6f191e).
    `gorilla_aggregate_ks` stays, deprecated (bd8f749), and the `gorilla`
    binary prints `agg_ad_A` and `agg_ad_p` in place of `agg_ks_p`.

18. **Craps dice use low bits** — `src/diehard/craps.rs:157-170`.
    `v % 6` after rejection; Marsaglia and Dieharder use high bits.
    Unbiased but a different bit-plane.  CONFIRMED.
    **Fixed:** each die is GSL's quotient ⌊x / 715 827 882⌋, redrawn when it
    is 6 or more, as `diehard_craps.c` gets through `gsl_rng_uniform_int`;
    the bounded-retry fallback stays. MINSTD, whose words never set bit 31,
    now fails craps. Dieharder scales by each generator's declared range,
    which the crate's `Rng` trait does not carry.
    GSL 2.8's `gsl_rng_uniform_int`, now in `pubs/`, confirms the quotient
    and the redraw (`rng/gsl_rng.h:189-212`).  Marsaglia's `craptest` uses
    another high-bit map (`diehard.f:555-556`): it reads each word as a
    signed integer x and takes int(6x/2³² + 3).  In double precision that
    face is GSL's + 3 (mod 6) on every word GSL keeps except twelve next to
    GSL's face boundaries, which get + 2.  In single precision, as gfortran
    evaluates `REAL`, the 191 words just below 2³¹ roll a 7, and 834 kept
    words in all get + 4.  No word gets the same face from both maps (checked
    over all 2³² words).  The doc had said the two maps agree on all but a
    dozen words; it now gives these counts and says the wins p-value is
    two-sided where `craptest` reports the one-sided `phi(t)` (f31f192,
    c413022).  Both maps read the high bits, so the choice stands.

19. **Universal parametric path emits p-values for any K >= 1** —
    `src/nist/universal.rs:122-131`.  Spec wants K near 1000·2^L; below
    that the normal approximation is meaningless yet no SKIP is issued.
    PLAUSIBLE.
    **Fixed:** a setting runs only when K ≥ 1000·2^L, the K behind every row
    of §2.9.7's table. At the battery's 16 Mbit, L = 11..16 now report SKIP.

20. **GCD reports SKIP on an all-zero stream** —
    `src/dieharder/gcd.rs:64-66, 79, 118, 137`.  Every pair is skipped,
    df is 0, `igamc(0,0)` is NaN, both results SKIP rather than FAIL.
    CONFIRMED by reading.
    **Fixed:** a statistic left with no degrees of freedom reports p = 0
    with a note, so an all-zero stream fails.

## Documentation drift and citation gaps

21. `src/rng/dual_ec.rs:3, 15, 39` cite "SP 800-90 (June 2006) §9"; the
    final places Dual_EC in §10.3, and no version containing Dual_EC is
    in `pubs/` (project rule: keep `pubs/` current).  Lines 44-45 credit
    only the port, not NIST.  Lines 171-189 apply the step-13 backtrack
    update after every block, i.e. a sequence of one-block Generate
    calls; undocumented deviation from multi-block Generate.  CONFIRMED.
    **Fixed:** citations follow the March 2007 revision section by section
    (§10.3.1, §10.3.1.4, Table 4, Appendices A.1.1–A.1.3 and E.2), both
    editions are now in `pubs/`, NIST is credited, and the doc explains the
    per-block state update (the backtracking update that the 2007 revision inserted as step 14).
22. `src/rng/spongebob.rs:18-20` claims SHA3-512 dispatches to aarch64
    intrinsics; the sibling crate gates that behind the opt-in
    `arm-sha3` feature, which this crate never enables.  CONFIRMED.
    **Fixed:** the module doc and BENCHMARKS.md say SHA3-512 runs the
    portable Keccak.
23. `scripts/parse_battery.py:301` says Dual_EC costs two scalar
    multiplications per block; the code and `src/main.rs:384-386` say
    three.  Regenerated TESTS.md carries the wrong number.  CONFIRMED.
    **Fixed:** "three" in the script, TESTS.md, USAGE.md, `tests/registry.rs`, `scripts/run_r_report.sh` and R-REPORT.md.
24. `src/rng/mt19937.rs:74-76` attributes the seed-19650218 vector to the
    `mt19937ar.c` output table, which is actually `init_by_array`
    output; the values are correct but the provenance is wrong, and the
    canonical `init_genrand(5489)` vector is untested.  CONFIRMED.
    **Fixed:** provenance corrected, and `init_genrand(5489)` is pinned
    (first ten outputs and the 10 000th, 4123659995) from an independent
    replica.
25. `src/nist/universal.rs:7-9, 25-26`: header L/Q thresholds disagree
    with the §2.9.7 table (L=7 starts at 904 960, L=15 at 496 435 200);
    the 12-digit variances are attributed to tables that give 3–4
    digits.  CONFIRMED.
    **Fixed:** thresholds follow §2.9.7. The doc says the digits beyond the
    printed tables come from an uncited source; they agree with Maurer's
    eqs. (16)–(17) to 6 × 10⁻¹⁰.
    Item J confirms that bound: against an accurate evaluation of the series
    the largest gaps are 4.0 × 10⁻¹¹ in μ and 5.6 × 10⁻¹⁰ in σ², both at
    L = 16.
26. `src/nist/serial.rs:13-19` doc says `serial()` returns a pair with
    `p_value = min(p1,p2)`; it returns one `TestResult`.  CONFIRMED.
    **Fixed:** the doc matches what `serial()` returns.  Review then found that
    rounding could leave ∇²ψ² just below zero, where `igamc` returns NaN, so
    `serial_delta2` skipped on valid input and `serial()` returned that skip
    even when `serial_delta1` failed (324 of 20 000 MT19937 seeds at n = 1040,
    m = 2).  Both statistics are now clamped at zero, and a skipped entry
    never masks a scored one.
27. `src/diehard/birthday_spacings.rs:9-13` claims Dieharder excludes
    tail bins under 5; `chisq_poisson` keeps all bins with df 7.
    CONFIRMED.
    **Fixed:** the doc says the code scores j = 0..6 with df 6 where `chisq_poisson` scores every cell (df 7 at 500 trials, df 5 at Dieharder's default of 100), that the repeat count follows Marsaglia's definition rather than the C's loop, which skips an interval after each run, and that the nine bit windows and the final KS are DIEHARD's.
    **Corrected on 2026-09-11:** the premise that the repeat count followed
    Marsaglia was wrong.  It followed `tests.txt`'s wording, the number of
    values that occur more than once.  Marsaglia's `cdbday` counts the i with
    C(i) = C(i−1) in the sorted spacings (`diehard.f:1277-1283`), so a value
    seen three times adds 2, and his `CHSQTS` scores j = 0 to 5 alone and
    pools j ≥ 6, df 6 (`diehard.f:1312-1375`), where the code dropped j = 7
    and discarded larger j.  The count and the cells now follow `diehard.f`
    (2611fd7).  Both rules are calibrated: over 12 000 null calls on MT19937
    the p-value fell below 0.01 in 0.94% of calls before and 0.96% after.
    Each of the nine windows still reads its own 256 000 words, where
    DIEHARD's windows share most of theirs (item L); the doc says so, and that
    DIEHARD's summary is Anderson–Darling, not KS.  Output changes for
    `diehard::birthday_spacings`.
28. `src/diehard/monkey.rs:7-14` says letter extraction deviates from
    Dieharder for all three; OPSO and OQSO extraction are identical, only
    DNA differs.  CONFIRMED.
    **Fixed:** only DNA extraction is described as differing from Dieharder, the null moments are described separately (OPSO and OQSO differ from Dieharder only in μ and σ), and the Author section credits Marsaglia and the 1993 Marsaglia–Zaman monkey-test paper.
29. Missing author citations (project rule): `src/diehard/monkey.rs`
    (no Author anywhere), `src/rng/stream_rng.rs` (no References or
    Author), `src/rng/block_ctr.rs:21-23` (no Author),
    `src/research/practrand_fpf.rs:1-5` (no Doty-Humphrey),
    `src/research/testu01_hamming.rs:1-5` and `testu01_lz.rs:1-5` (no
    L'Ecuyer and Simard), `src/bin/bitplane_complexity.rs:1` (no
    Berlekamp–Massey reference), `src/bin/upstream_tests.rs` and
    `src/bin/testu01_lz.rs` (nothing), `src/nist/overlapping_template.rs:49`
    (Hamano–Kaneko corrected pi uncited), `src/nist/spectral.rs` (Kim et
    al. corrected T and d uncited).  `src/research/knuth.rs:1-9, 332`
    names the above/below-median Wald–Wolfowitz test as TAOCP's runs
    test, which is runs-up/down.  CONFIRMED.
    **Fixed outside DIEHARD:** PractRand, TestU01 and Berlekamp–Massey are
    credited; the stream-cipher, CTR and ChaCha20 adapters have Author and
    References sections; the overlapping-template π values and spectral
    corrections cite what SP 800-22 names; and the median runs test is
    credited to Wald and Wolfowitz in its code, result label, README, BIB.md, TESTS.md and scripts.  `monkey.rs` credits Marsaglia and Zaman.  Review found two new Author lines misattributing CTR mode and Rabbit; they now credit the SP 800-38A recommendation and Rabbit's FSE 2003 designers.
30. `src/bin/upstream_tests.rs:128-141` help lists 3 of 10 accepted
    flags and advertises "FPF(4,14,6)" though the stride-4 overlap is
    dropped (`practrand_fpf.rs:14-21`).  `src/main.rs:22-24` places the
    `--help` line after the exit-code paragraph.  `src/rng/mod.rs:79-87`
    endianness list omits `BlockCtrRng`/`StreamRng` and `AesCtr`.
    CONFIRMED.
    **Fixed:** the `--help` line sits with the other options, and the byte-
    order list was rebuilt from each generator's code.
31. Sample sizes far below Dieharder defaults with no doc note:
    `gcd.rs:26` 10^5 vs 10^7 pairs; `fill_tree.rs:24` 10^5 vs 1.5×10^7;
    `dct.rs:26` 5 000 vs 50 000 blocks; `minimum_distance_nd.rs:41-42`
    8 000×100 vs 10 000×1 000.  CONFIRMED.
    **Fixed:** each sample-size constant states Dieharder's header default
    and the power it gives up.

## Code quality

32. **Duplicated helpers that belong in `math.rs`**: `chi_square_pvalue`
    in `practrand_fpf.rs:50` and `testu01_hamming.rs:63` re-implement
    `math::chi2_pvalue`; every NIST test inlines `igamc(df/2, x/2)`
    instead of calling it; `binomial_pmf` (`bit_distribution.rs:25`) and
    `binomial_pdf` (`monobit2.rs:133`); `poisson_pmf`
    (`birthday_spacings.rs:133`); `strip_b` in `testu01_hamming.rs:25`
    and `testu01_lz.rs:74`; three O(n^2) nearest-pair scans
    (`minimum_distance.rs:70`, `spheres_3d.rs:56` takes sqrt per pair,
    `minimum_distance_nd.rs:105`); two run counters in
    `runs_float.rs:119-197`; a `hex()` test helper in `hash_drbg.rs:332` and
    `hmac_drbg.rs:265`, duplicated by the production `decode_hex` in
    `dual_ec.rs:249` (replaced by rump's `from_str_radix` on 2026-09-10); the `take_bytes` refill idiom
    in five generators.
    **Fixed in the research probes:** one shared `strip_b`, and
    `math::chi2_pvalue` replaces both local p-value helpers. **Fixed
    in DIEHARD/DIEHARDER:** the binomial and Poisson pmfs live in `math.rs`,
    and 3-D spheres compares squared distances; results are identical.
    **Fixed on 2026-09-11:** the nine NIST tests that inlined `igamc` call
    `math::chi2_pvalue` (124ed00), with p-values bit-identical over 2 453
    values and 653 more that the reviewer checked.  A private `ByteBuffered`
    trait holds `take_bytes` once for `ChaCha20Rng`, `HashDrbg`, `HmacDrbg`,
    `SpongeBob`, `Squidward` and `StreamRng`, and one test-only `hex()`
    serves both DRBGs (88fe957); output is byte-identical (1 847 106 lines
    compared in review) and throughput unchanged.  The three nearest-pair
    scans share one `min_squared_distance`, and `runs_float`'s two entry
    points one run counter (a14efeb), with byte-identical output.
33. **Boilerplate copied across seven binaries**: `Args::parse`, `die`,
    `matches_rng`, and the 14-case RNG list in `bib_tests`, `gorilla`,
    `testu01_lz`, `upstream_tests`, `webster_tavares`,
    `bitplane_complexity`, plus the `dump_rng`/`pilot_rng` dispatch
    tables; `upstream_tests.rs:176-252` repeats each label twice.
    `--rng` ignores case in every binary (2026-09-10).  **Fixed on
    2026-09-11:** `src/bin/common/cli.rs` holds the argument cursor, the
    `--rng` filter and the usage exit, with unit tests, and
    `src/bin/common/family.rs` holds the seeded generator family, visited
    lazily so `--rng` still decides what is constructed.  `bib_tests`,
    `bitplane_complexity`, `gorilla`, `testu01_lz`, `upstream_tests` and
    `webster_tavares` share them, and `upstream_tests` no longer writes each
    label twice (d4a1cd1).  Options, help text, error messages, exit codes
    and output are unchanged: each binary's help and usage-error runs and a
    small release run were captured before and after and diffed.  `dump_rng`
    and `pilot_rng` keep their dispatch tables, whose name lists
    `tests/registry.rs` locks together, because `pilot_rng`'s timing loops
    are monomorphised per generator.
34. **Dead public API** (no callers in `src/` or `tests/`):
    `nist::serial::serial`, `nist::random_excursions::random_excursions`,
    `nist::random_excursions_variant::random_excursions_variant`,
    `nist::non_overlapping_template::non_overlapping_template`,
    `diehard::craps::craps`, `diehard::runs_float::runs_float`,
    `dieharder::fill_tree::fill_tree`, `dieharder::gcd::gcd`,
    `dieharder::bit_distribution::bit_distribution`, the `CRand` alias
    (`c_stdlib.rs:120`), `RngResults.nist_n` (`main.rs:232, 419, 436`,
    always `NIST_N`), and the `(31,31)` match arm in
    `binary_rank.rs:190-196` (identical to the generic arm).
    **Fixed:** `RngResults.nist_n` is gone.  `CRand` was removed, then restored as a deprecated alias after review, because it was public in 0.5.0. **Left:** the
    unused public test functions stay, because removing them breaks the
    public API. The redundant `(31,31)` match arm and `gf2_rank_6x8` wrapper
    are gone.
35. **Same generator twice**: `LcgVariant::Msvc` (`lcg.rs:33-35, 95-102`)
    and `WindowsMsvcRand` (`c_stdlib.rs:131-160`) each pin the same
    41, 18467, … vector; `msvc_lcg` exists only in the two binaries, so
    the binaries carry 44 names and the battery 43.
    **Documented:** both public types stay; they cross-reference each other,
    share one known-answer constant, and a test checks their streams agree.
36. **Variable-length result vectors**: `random_excursions_all`
    (`random_excursions.rs:84-87`) and the variant (`:63-66`) return a
    single SKIP entry instead of 8/18 when the walk is too short, so
    `run_all`'s documented 200-slot layout (`nist/mod.rs:45-52`) shrinks
    to 176.
    **Fixed:** both functions always return 8 and 18 entries, so `run_all`
    keeps 200 NIST slots.
37. **Exit-code drift**: `dump_rng` uses 2 for usage errors
    (`dump_rng.rs:167,177,193`), `pilot_rng` uses 1
    (`pilot_rng.rs:146,201`), research binaries use 1 via `die` but 101
    on panics.  `--rng` matching (`main.rs:186`) is case-sensitive, so
    `--rng dual_ec` matches nothing; `--suite diehard --rng Dual_EC`
    exits 0 silently.
    **Fixed:** `--rng` ignores case in `run_tests` and every research
    binary, `run_tests` exits 1 with a message when nothing runs, and
    `dump_rng` usage errors exit 1 like `pilot_rng`. Correction: `--rng
    dual_ec` already exited 1 with a message; the silent exit 0 was `--suite
    diehard --rng Dual_EC`.
38. **Memory**: `testu01_lz.rs:84` reserves `n_bits/4 + 1` trie nodes
    (256 MiB at k=25, 2 GiB at k=28) against about 56 MiB used.
    `random_excursions_variant.rs:70` uses a `HashMap` for 18 fixed
    states.  `linear_complexity.rs:122` clones the connection vector on
    every discrepancy.
    **Fixed:** the LZ trie reserves from `LZ_MU` (about 61 MiB at k = 25),
    the variant counts visits in a fixed array, and Berlekamp–Massey reuses
    one scratch buffer. Outputs are unchanged.
39. **Repeated hex literals** (project rule): `src/rng/aes_ctr.rs:270-273`
    and `:333-336` inline the NIST F.5 key twice.
    **Fixed.**  The item missed other repeats, found on 2026-09-11: the
    DRBGVS entropy input and nonce written out in two HMAC_DRBG tests
    (`hmac_drbg.rs:303-304, 346-347` at 229e104); wyhash's first prime in
    `wyrand.rs` and `seed.rs`; the VB6 `Rnd` and `rand48` parameters;
    Jenkins's `raninit` word and the JSF64 test seed; an AES-128 zero-key
    keystream word in `block_ctr.rs`; xoshiro test states; and the Constant
    word that `run_tests`, `dump_rng` and `pilot_rng` each wrote out, with
    the jsf64 seed of the last two.  **Fixed on 2026-09-11:** each is one
    named constant (590ca29, 25cac16).  The generator parameters are public
    associated constants, so docs.rs shows them:
    `WindowsVb6Rnd::{MULTIPLIER, INCREMENT, MASK}`,
    `Rand48::{MULTIPLIER, INCREMENT, MODULUS}`, `Jsf64::INITIAL_A` and
    `WyRand::{INCREMENT, MIX}` (19efccd).  `seed_material` XORs its own
    `SEED_MATERIAL_MASK` rather than `WyRand::INCREMENT`, and
    `seed_material_pinned_bytes` pins its output (7a529ed).  A test ties the
    printed "Constant (0xDEAD_DEAD)" label to `CONSTANT_RNG_WORD`, and
    `tests/dump_rng.rs` gains a jsf64 pin (43a1ee8).  The HMAC_DRBG test
    cites its CAVP record, whose file is now in `pubs/`: `HMAC_DRBG.rsp`
    from `drbgvectors_no_reseed.zip` (CAVS 14.3), in the first of four
    sections with the same header, at line 4104, record `COUNT = 0` at line
    4112 (69741ff, 5b7e460).  Output is bit-identical: `dump_rng <name> 4096`
    matches 229e104 for all 40 deterministic names.  Merged in 1e26fd5.  Two
    values repeat by design.  0xa0761d6478bd642f is both `WyRand::INCREMENT`
    and `SEED_MATERIAL_MASK`, one value in two roles.  0xdeadbeef is five
    distinct choices: `JSF64_PROBE_SEED`, the JSF64 known-answer seed, a
    `bit_distribution` test word, an xoroshiro test state, and the `strip_b`
    test word at `research/mod.rs:35`, picked for its nibbles 0xD and 0xE.
40. **CI**: `.github/workflows/ci.yml:38-45` checks out the sibling
    crates at unpinned default branches, so any push there changes what
    CI builds; no `cargo fmt --check` step; MSRV job is Ubuntu-only.
    **Fixed:** CI runs `cargo fmt --check` (2026-09-10).  Both sibling
    checkouts are pinned by full SHA to cryptography 342989a and rump
    3ff885c, the commits this crate is verified against, and the 1.87 MSRV
    job also runs on macos-latest (277e3aa).  The comment beside the pins
    says what pinning costs: the siblings' downstream jobs build this crate
    but run only `cargo test` on ubuntu-latest stable, so moving the pins
    means first running the whole matrix locally against the new sibling
    commits (ee554d1).  The stable jobs also run
    `cargo test --release -- --include-ignored`, so the three tests too slow
    for debug builds (the `dct` golden and two linear-complexity pins on e)
    now run in CI (003e8d7, merged in 3a2bf85).
41. **Nits**: `hash_drbg.rs:140,171` and `hmac_drbg.rs:120,142` refuse
    the last permitted call (`>=` vs spec `>`); `serial.rs:42` gates
    n >= 1000 without spec basis; `craps.rs:201-203` tail-mass comment
    off by eight orders of magnitude; `bitstream.rs:101-108`
    `msb_first_ordering` asserts nothing; `monobit2.rs:46-56` flat
    layout aliases adjacent levels (inherited from `dab_monobit2.c`);
    `knuth.rs:278` `partial_cmp().unwrap()` panics on NaN input;
    `knuth.rs:169-186` skips the leading gap that TAOCP Algorithm G
    counts (PLAUSIBLE).
    **Fixed:** the DRBGs refuse only above the reseed interval; the median
    sort uses `total_cmp`, and `math::ks_test` returns NaN for a NaN sample
    instead of panicking; the gap test documents its leading gap;
    `msb_first_ordering` asserts a real count; and the craps tail comment is
    corrected.  This item's own figure was wrong too: the mass beyond 200
    throws is 2.3 × 10⁻²⁶, and 4.4 × 10⁻³² is (25/36)¹⁹⁸, the chance a set 6
    or 8 stays unresolved for 198 more throws.  The serial m gate already
    matched §2.11.7 exactly (m < ⌊log₂ n⌋ − 2); only its docs misquoted the
    rule.  **Left:** the serial n ≥ 1000 floor, documented as a project
    choice.  **Kept for fidelity:** `monobit2`'s inherited level aliasing,
    now documented with its block phase (bde3347): level j's all-ones cell is
    level j + 1's all-zeros cell, and level j's first block closes after
    2^j + 1 words.  No shared cell held a count in 46 300 null MT19937
    trials, but a stream that fills one can flip a verdict (item K).

## Test coverage

Unit tests: 136 in the library, 3 in `dump_rng`, 10 integration.  Gaps:

- No spec known-answer tests for NIST frequency, runs, longest run, or
  cumulative sums even though the §2.x.8 examples pass through the
  public API today; no Berlekamp–Massey KAT; no 10^6-bit e fixture, so
  the matrix-rank, serial and random-excursion examples are unpinned.
  **Addressed in part:** §2.1.8, §2.3.8, §2.4.8, §2.10.4 (Berlekamp–Massey) and
  §2.13.8 are pinned end to end, §2.12.8 through the production χ² and P
  code, §2.5.8's χ² from its printed counts, and §2.9.8's σ and P from its
  printed sum.  **Addressed on 2026-09-11:** `tests/data/e_1e6_bits.bin`
  packs the first 10⁶ bits of STS 2.1.2's `data.e`, checked against the
  symbol counts §2.11.8 prints and, for its first 2¹⁴ bits, against e summed
  with rump's `BigUint` (5417cae).  §2.5.8, §2.8.8, §2.10.8, §2.11.8, §2.14.8,
  §2.15.8 and Appendix B's e table are pinned on it, §2.10.8 and Appendix
  B's M = 500 row in release builds only; §2.6.8, which runs on 100 bits of
  π below the spectral gate, is pinned through the statistic's helper
  (1766dff).  Where the printed figures differ, the cause is reproduced:
  §2.8.8's χ² uses §3.8's compound-Poisson probabilities, which STS keeps;
  §2.10.8 uses STS's π₀ = 0.01047 where §2.10.4 prints 0.010417; §2.14.8's rows
  for x > 0 drop the final excursion's visits.  In §2.6.8 STS counts
  N₁ = 48, as the module does, not the printed 46, and nothing found
  explains the 46.  Random excursions match STS on e (J = 1490 and all eight
  χ² and P; dc00343); linear complexity is scored in debug builds on 100 000
  bits of e at M = 500, with STS's class counts (f281a34); and serial's
  statistic-to-df pairing is tested through the code `serial_both` uses
  (405b199).  **Still missing:** §2.9.8's input is unavailable, so only its
  σ and P are pinned, from the printed sum.
- Fourteen DIEHARD/DIEHARDER modules have no unit tests at all
  (birthday spacings, count-ones, parking lot, runs, 3-D spheres,
  squeeze, minimum distance, byte distribution, DCT, GCD, KS uniform,
  lagged sums, minimum distance n-D, permutations); no golden p-values
  for a fixed seed; no check that the reference tables sum to 1.
  **Addressed in part:** every reference probability table has a sum-to-one
  test.  Seventeen modules gained edge-input tests: constant input fails in
  each, and empty or short input skips in those with a length gate; the
  three binary-rank tests also pin their one-word-short boundary.
  **Addressed on 2026-09-11:** `tests/diehard_goldens.rs` pins the p-value of
  every DIEHARD and DIEHARDER test function to 10⁻¹², and its note exactly,
  on MT19937 seeded with 5489 (aa1ac68).  `binary_rank_31x31` alone is held
  to 10⁻⁸, because shifting every libm result by one ulp moved it by up to
  3.7 × 10⁻¹⁰, and a second `monobit2` golden covers two block sizes
  (7424f0f).  The goldens pass in debug and release builds on aarch64 and
  x86_64 macOS and on x86_64 Linux (moore, glibc, rustc 1.95); the `dct`
  golden runs only in release builds.  They are this code's own output, not
  reference values from the C.  The eight goldens that pass through `erfc`
  or `normal_cdf` were re-pinned when it changed (item N).
- No KAT for `Rand48`, `Xorshift32/64`, `Pcg64`, or `Lcg32`
  AnsiC/Borland.  (`DualEcDrbg` and the streaming paths of both DRBGs were
  pinned on 2026-09-10; see item C.)  `ChaCha20Rng` has no deterministic constructor, so it cannot be
  pinned to RFC 8439.  `tests/dump_rng.rs:110-142` pins 5 of 44 names.
  **Addressed:** Rand48 (against macOS libc), Xorshift32/64, Pcg64 and Lcg32
  AnsiC/Borland are pinned.  **Addressed on 2026-09-11:**
  `ChaCha20Rng::new(key, nonce, counter)` (24e8afe) is pinned to RFC 8439's
  §2.3.2 block, its §2.4.2 keystream across a block boundary and the five
  Appendix A.1 blocks.  `StreamRng` reproduces all three RFC 4503 Appendix
  A.2 Rabbit vectors and both Salsa20 §9 expansion examples (dfec5eb); all
  15 SNOW 3G key/IV sets in ETSI/SAGE Document 3, the 4 of §3, the 5 UEA2
  sets of §4 compared to LENGTH bits and the 6 UIA2 sets of §5; and all 4
  ZUC-128 sets of §3, the only keystream its Document 3 prints (cf2d696,
  a4f8932).  A test pins `mt19937ar.out` through a transcribed
  `init_by_array` (2f60c0c).  Compiled in scratch from the reference C now
  in `pubs/`, pcg-c's pcg32 and pcg64, `init_genrand` and `genrand_int32`
  from `mt19937ar.c`, Vigna's `next()`, Jenkins' 64-bit `ranval`, wyrand
  from wyhash's `wyhash_final2.h` and `wyhash_final4.h`, and Marsaglia's
  `xor()` and `xor64()` each agree with the crate for 5000 outputs
  (2f60c0c); glibc's generators are checked as item 13 describes, and
  FreeBSD's `rand_r` matches `BsdRandCompat` at five seeds (6c851e6).  That
  pass also corrected docs that claimed more than their sources: the V7
  `rand(3)` page prints no LCG parameters, wyhash has no "final version 3",
  and Jenkins states no period for JSF64; the xorshift docs now note that
  the paper's printed `xor()` drops the xor from its middle step.  **Still
  missing:** SFC64 is checked only against a Python replica, because
  PractRand's source is not in `pubs/`.
- `src/main.rs` has zero tests; `Args::parse` reads `std::env::args`
  directly.
  **Addressed:** option parsing moved to `Args::parse_from`, with tests for
  it and `group_thousands`.
- Research: `hamming_indep`, `lumped_chi_square`, `truncate_table_bits`,
  `grouped_tail_g_test`, `gorilla_aggregate_ks` untested; the PractRand
  FPF truncation rule is unverified (no source available).
  **Addressed:** each of those now has a test, `hamming_indep` against an
  independent replica.  **Addressed on 2026-09-11:** HammingCorr and
  HammingIndep statistics and generator calls, and multi-replication LZ
  phrase counts, are pinned against TestU01 1.2.3 built from `pubs/`
  (e7c2a96, 27b0b39; item 12), and `anderson_darling_cdf` against the 2004
  paper, its C and the in-repo tail simulation (item 17).  **Still
  missing:** the FPF truncation rule is unverified, because PractRand's
  source is not in `pubs/`.

## Follow-up — 2026-09-11

The follow-up rechecked the open items against the sources that entered
`pubs/` that day, compared the DIEHARD modules with Marsaglia's Fortran, and
fixed or documented what it found on topic branches.  The findings are
folded into items 8–41, F–N and the coverage notes above.  This section
records how the work was checked, what `pubs/` gained, and what the
neighbouring repositories' audits say about this crate.

**How the work was checked.**

- *Adversarial review.*  Every branch went to an adversarial reviewer, and
  each finding was fixed on the branch and sent back.  The NIST
  (`audit-nist`), generator (`audit-rng`), .NET (`fix-dotnet`) and DIEHARD
  (`audit-diehard`) branches were merged only after their reviewers conceded
  every finding.  On the research (`audit-research`), erfc (`math-erfc`),
  hex-literal (`fix-hex-literals`) and TESTS.md (`docs-tests-theory`)
  branches the reviewers conceded every finding up to a last round of
  documentation and tolerance fixes, which were checked against the
  reviewers' figures before merging.  `ci-release-tests` adds one CI step,
  whose command the Linux runs below execute.
- *Staging.*  Each merge was made on `merge-staging` and verified there
  against `git archive` copies of the committed sibling crates, cryptography
  342989a and rump 3ff885c, which is what CI builds: `cargo fmt --check`,
  clippy with `-D warnings`, `cargo test`, `cargo test --release --
  --include-ignored`, rustdoc with `-D missing_docs`, and a 1.87 check.  At
  229e104 all pass, with 346 tests in the debug run (3 ignored) and 349 in
  the release run, and at main 93e6621 with 797 across the two runs.
- *Linux.*  On moore (x86-64, glibc, rustc 1.95), every run passed in debug
  and in release with `--include-ignored`: 229e104 with 346 and 349 tests,
  83a841c (erfc and research) with 395 and 398, which shows `erfc`'s
  ε-scaled test tolerances hold on glibc's libm, and main 93e6621 with 397
  and 400.  The earlier staging commit 489dc5a passed with 387 and 390.
  fd6df95, main's tip, changes only TESTS.md.
- *DIEHARD fidelity review.*  A read-only review compared every DIEHARD
  module with `diehard.f`.  It ran gfortran 16.2 builds of the Fortran as an
  oracle on 4 000 000 words, built with `-fno-automatic`, without which
  gfortran clobbers `jtbl`'s record buffer between calls, and simulated null
  distributions in C, NumPy and the Fortran itself.  Its findings are items
  8, 10, 18, 27, F, L and M; its claim that DIEHARD rereads the same words
  for every window was wrong (item L).

**Additions to `pubs/`** (f098126, f5ecb2a, ffb3aee, e018c34, be5c958,
69741ff).  Fifty files, each listed with its origin and retrieval date in
`pubs/SOURCES.tsv`, which also gives sha256 for the DIEHARD archives and for
the files repacked or extracted from larger downloads:

- DIEHARD: the Fortran, f2c and Wang archives of the method paragraph, and
  PDFs of Marsaglia's extract of the Marsaglia–Zaman 1993 monkey-test paper
  and of his 1984 keynote, converted from the PostScript in the f2c archive.
- Test suites: NIST STS 2.1.2's sources and constants (repacked without its
  generator outputs and experiments, keeping `data.e`, `data.pi`, `data.sqrt2` and
  `data.sqrt3`), the TestU01 2009 tree, and Kim–Umeno–Hasegawa 2004.
- Library generators: glibc 2.40's `random_r.c`, `random.c` and `rand.c`
  with its license, FreeBSD's `rand.c` and `random.c`, and GSL 2.8's `rng/`.
- Generator references: Matsumoto–Nishimura 1998 with `mt19937ar.c` and
  `mt19937ar.out`; O'Neill 2014 and pcg-c; Blackman–Vigna with six of
  Vigna's C files; wyhash; Jenkins' small-PRNG page; Marsaglia's xorshift
  paper; the V7 manual.
- Stream ciphers: the ChaCha and Salsa20 papers, RFC 8439, RFC 4503 and the
  eSTREAM Rabbit description, and ETSI/SAGE's SNOW 3G and ZUC specifications
  with their Document 3 test data.
- Statistics: Marsaglia and Marsaglia 2004 on the Anderson–Darling
  distribution with `ADinf.c` and `AnDarl.c`; the `tuftests.c` attached to
  Marsaglia–Tsang 2002; Marsaglia 2004 on the normal distribution with its
  `sources.c`; Marsaglia–Tsang–Wang 2003 on the Kolmogorov distribution; and
  Wald–Wolfowitz 1940.
- DRBGs: Bernstein–Lange–Niederhagen 2015 on Dual_EC, and the CAVP
  `HMAC_DRBG.rsp` from NIST's no-reseed DRBG test vectors (item 39).

PractRand, Hamano–Kaneko 2007, Numerical Recipes and TAOCP are still not in
`pubs/`.

**Cross-repository findings.**

- The cryptography session's audit left an untracked `AUDIT.md` and
  `SUGGESTIONS.md` in `../cryptography`.  Every factual claim they make about
  this crate was checked and holds, among them that `src/diehard` at
  f098126 has eleven test modules and `mod.rs`, that `pubs/Diehard.zip`
  (613 818 bytes) holds DOS binaries, data and documentation rather than a
  source distribution, and that 3b41af8 deleted `operm5.rs` (249 907 bytes)
  and `overlapping_sums.rs` (3 783 bytes), both still readable from its
  parent.  Its suggestions hand DIEHARD preservation and a reusable
  battery-input adapter to this crate.
- This crate's own `SUGGESTIONS.md`, untracked and written by another
  session against ffb3aee, confirms the source recovery and asks for two
  things.  The first is an inventory of the historical DIEHARD
  implementations, with each one's revision and the reason it was disabled,
  and an evaluation of any disputed statistic before a test is restored;
  item M is that evaluation.
  <!-- PENDING historical suite -->
  The second is a finite byte-corpus adapter with an explicit contract for
  word endianness, bit order, partial words, consumption and short input,
  verified against a generated stream.  A design exists: sequential and
  rewind modes, little-endian words by default, a step that runs out of
  input reported as SKIP with exit code 2 rather than cycled or padded, and
  the corpus sha256 and per-step consumption in the output, about 172 MB at
  the default sizes.  It is being built.
  <!-- PENDING corpus adapter -->
- `wipe-opt-in` (41f93ca) turns on cryptography-rs's opt-in `wipe` feature,
  which that crate is introducing for rump's limb scrubbing (item B).  It
  passes fmt, clippy, tests, docs and 1.87 against a copy of cryptography's
  uncommitted tree with rump 3ff885c.  The committed 342989a has no such
  feature, so the manifest cannot resolve against it; the branch waits until
  cryptography publishes the feature, and the CI pins (item 40) move with it.
- An uncommitted rump change (F3 in the cryptography audit) alters
  `to_be_bytes_padded`; the `store_mod_seedlen` doc in `hash_drbg.rs` needs
  updating when it lands.

## Verified correct

Every generator known-answer vector in the tree is genuine, and several
generators without one were checked as well: MT19937, PCG32,
SFC64, JSF64, wyrand, xorshift, rand48, Hash_DRBG, HMAC_DRBG and Dual_EC
P-256 output were reproduced from independent replicas of the
reference algorithms, and `mrand48`/`random`/`rand` were cross-checked against the macOS libc.  (The
audit's PCG64 replica shared the crate's pre-advance output, so that check
missed the mismatch item E fixed.)  NIST frequency, block frequency, runs, longest
run, spectral, approximate entropy, cumulative sums, serial, both random
excursion tests and the non-overlapping template match STS on 10^6 bits
of e to 5–6 digits.  DIEHARD/DIEHARDER tables (`SDATA`, `KPROB`,
`TARGET_DATA`, Fischler Q, runs A/B, rank probabilities, parking lot,
bitstream, birthday, craps, count-the-1's, byte distribution, DCT,
monobit2) match the C digit for digit.  TestU01 LZ mu/sigma tables and
end-phrase rule, HammingCorr/HammingIndep statistics, Gorilla constants,
SAC/BIC definitions and Knuth chi-square degrees of freedom all match
their sources.  `#![forbid(unsafe_code)]` holds, all-zero seeds are
rejected where required, wrapping arithmetic is used throughout, CTR
counters wrap, `OsRng` uses `read_exact`, and the documented 0/1/2 exit
contract is implemented in `run_tests`.

## Status after the 2026-09-11 follow-up

Items 1–6, A–L and N are fixed or documented (B withdrawn), and so are most
of items 7–41; each carries a note above.  Item M waits on the historical
DIEHARD suite.

- **Merged** (main at fd6df95): `audit-nist` (79f48b4), `audit-rng`
  (fd36369), `fix-dotnet` (601ab91), `kat-etsi` (ccf18e7), `audit-diehard`
  (e410409, 229e104), `fix-hex-literals` (1e26fd5), `math-erfc` with the
  re-pinned goldens (89eb68b, 83a841c, 93e6621), `audit-research` (956976b,
  d6555c8, 72580dc, 00c2a2c), `docs-tests-theory` (0a417eb),
  `ci-release-tests` (3a2bf85), and the regenerated TESTS.md (fd6df95).
- **Kept for fidelity to the cited reference:** the Dieharder fill-tree
  off-by-one (14); SP 800-22's four-decimal longest-run tables for M = 128
  and M = 10⁴, the latter also STS's (15); both MSVC `rand()` types (35);
  `monobit2`'s shared cells and inherited calibration defect (41, K); and
  Dieharder's squeeze pooling (8), dice (18), bit and byte orders and sample
  sizes (L).
- **Kept by choice and documented:** fresh words for each birthday window
  (27); KS summaries with upper-tail or two-sided p-values where DIEHARD
  reports Anderson–Darling and CDF values (L); the 6×8 test's single window
  (10); the Gorilla aggregate's scaled upper tail and shared words (17); the
  unused public test functions (34); the serial test's n ≥ 1000 floor and
  TAOCP's leading gap in the Knuth gap test (41); and SKIP where STS writes
  P = 0 below the random-excursion cycle gate.
- **TESTS.md:** Theory By Test was corrected and reviewed (0a417eb).  It
  gives each DIEHARD test's p-value convention and summary, DIEHARD's
  Anderson–Darling `KSTEST`, the approximate uniformity of the CDF values
  that parking lot, minimum distance and 3-D spheres feed their KS tests,
  the Gorilla aggregate's AD(32, ·) body and this crate's tail rule,
  monobit2's rejection rates with the word counts they were measured at,
  and the word counts and N behind the runs and gcd figures.  It also
  carries the departures in items 8, 18, 27, F and L, and corrects these
  prose errors: `minimum_distance_nd` runs only d = 5; `gcd_step_counts`
  scores k = 6–32 with df 26; `gcd_distribution` leaves g = 1 unscored and
  pools g ≥ 23; universal's σ is c(L,K)·√(var/K); ApEn's bound is
  m < ⌊log₂ n⌋ − 5; the Gorilla σ = 4170 came from simulation; Lempel–Ziv
  reports two p-values with no outer KS; Webster–Tavares reads the low 32
  bits of the first `next_u64`; `craps_throws` has df 21 where DIEHARD pools
  from 21 up with df 20; and the gap test folds a tail pool that still
  expects fewer than 5 into the last kept cell.
  `tests/run_all.sh` then ran on dyson on 2026-09-11, in 9 min 53 s, at
  33dc35c, whose default battery is main's: it differs only by the opt-in
  historical suite and by erfc test and doc changes.
  `scripts/parse_battery.py` rebuilt the header, summary table, Failure
  Highlights and Auxiliary Probes, and left Theory By Test byte-identical
  (fd6df95).
- **Battery outputs that changed** in that run, at the fixed seeds: the runs
  fix (F) removed Camellia-128-CTR's `diehard::runs_up` failure, leaving 11,
  all `bit_distribution`, and Twofish-128-CTR and CAST-128-CTR each fail
  `diehard::runs_down` once, about the one such failure a calibrated test is
  expected to give across the battery at α = 0.01; with the runs and
  birthday-spacings (27) changes VB6 `Rnd()` goes from 529 to 528 failures;
  and the Gorilla probe prints `agg_ad_A` and `agg_ad_p` in place of
  `agg_ks_p` (17).  p-values that pass through `erfc` move by at most about
  10⁻⁷ (N).  OS-seeded generators differ from the previous run by chance,
  and Failure Highlights stay one line per generator.
- **Open:**
  - The historical DIEHARD suite: the three removed tests (M), DIEHARD's
    25-window 6×8 sweep (10) and an inventory in place of README's
    "Removed On Purpose".  `diehard-historical` is under review fixes.
    <!-- PENDING historical suite -->
  - The finite byte-corpus input (Follow-up), which is being built.
    <!-- PENDING corpus adapter -->
  - Comments in three research modules still give `erfc`'s old error as
    the reason for their p-value caps (N).
  - The `wipe` feature waits on cryptography (Follow-up), and BIB.md dates
    wyhash 2022 where the `pubs/` snapshot is a March 2026 commit.
  - Coverage: SFC64 and PractRand's FPF truncation rule are unverified, and
    §2.9.8's input is unavailable (Test coverage).
