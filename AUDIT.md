# Code Audit — 2026-09-10

Scope: every file under `src/`, `tests/`, `scripts/`, `.github/`, and the
manifest, at commit `40447e3` plus the cleanup recorded below.  Method: full
read of each module by four independent reviewers, formulas and constants
compared against the primary sources in `pubs/` (SP 800-22 Rev 1a text,
SP 800-90A, Marsaglia's `tests.txt` and Diehard sources, the Dieharder
3.31.1 C sources, TestU01's `scomp.c`/`sstring.c`, Marsaglia–Tsang 2002,
Webster–Tavares 1985), and cheap claims executed against the built crate
and against the macOS libc where a reference implementation exists.

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

13. **glibc `random()` seeding differs for seeds >= 2^31** —
    `src/rng/c_stdlib.rs:68-77, 342-348`.  `park_miller31` seeds via
    `u32 -> i64` (always non-negative); glibc runs Schrage on a signed
    `int32_t`, so high seeds diverge.  Seed 1 (the battery) is
    unaffected.  PLAUSIBLE.
    **Fixed:** seeding runs Schrage's step on the signed 32-bit word, pinned
    for seeds 3 000 000 000 and 2^31 − 1. The reference is an independent C
    and Python replica of glibc's `__srandom_r`, not glibc itself; seed-1
    output is unchanged.

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
    reproduces §2.4.8's P = 0.180609. **Left:** M = 128 and M = 10⁴ keep the
    four decimals §3.4 prints. For M = 10⁴, which the battery uses, they
    differ from exact values by up to 1.6 × 10⁻³, but they are the published
    standard's values.

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
    deviation from the paper's ADKS. Anderson–Darling would need its
    distribution's source, which is not in `pubs/`.

18. **Craps dice use low bits** — `src/diehard/craps.rs:157-170`.
    `v % 6` after rejection; Marsaglia and Dieharder use high bits.
    Unbiased but a different bit-plane.  CONFIRMED.
    **Fixed:** each die is GSL's quotient ⌊x / 715 827 882⌋, redrawn when it
    is 6 or more, as `diehard_craps.c` gets through `gsl_rng_uniform_int`;
    the bounded-retry fallback stays. MINSTD, whose words never set bit 31,
    now fails craps. Dieharder scales by each generator's declared range,
    which the crate's `Rng` trait does not carry.

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
    `math::chi2_pvalue` replaces both local p-value helpers. **Left:** the NIST tests still inline `igamc`; the `hex()` test helpers, the `take_bytes` idiom, the three nearest-pair scans and the two `runs_float` run counters remain (the production `decode_hex` is gone). **Fixed
    in DIEHARD/DIEHARDER:** the binomial and Poisson pmfs live in `math.rs`,
    and 3-D spheres compares squared distances; results are identical.
33. **Boilerplate copied across seven binaries**: `Args::parse`, `die`,
    `matches_rng`, and the 14-case RNG list in `bib_tests`, `gorilla`,
    `testu01_lz`, `upstream_tests`, `webster_tavares`,
    `bitplane_complexity`, plus the `dump_rng`/`pilot_rng` dispatch
    tables; `upstream_tests.rs:176-252` repeats each label twice.
    **Left:** the shared CLI boilerplate is unchanged, though `--rng` now
    ignores case in every binary.
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
    **Fixed.**
40. **CI**: `.github/workflows/ci.yml:38-45` checks out the sibling
    crates at unpinned default branches, so any push there changes what
    CI builds; no `cargo fmt --check` step; MSRV job is Ubuntu-only.
    **Fixed in part:** CI runs `cargo fmt --check`. **Left:** sibling
    checkouts stay unpinned and the MSRV job stays Ubuntu-only.
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
    choice, and `monobit2`'s inherited level aliasing.

## Test coverage

Unit tests: 136 in the library, 3 in `dump_rng`, 10 integration.  Gaps:

- No spec known-answer tests for NIST frequency, runs, longest run, or
  cumulative sums even though the §2.x.8 examples pass through the
  public API today; no Berlekamp–Massey KAT; no 10^6-bit e fixture, so
  the matrix-rank, serial and random-excursion examples are unpinned.
  **Addressed in part:** §2.1.8, §2.3.8, §2.4.8, §2.10.4 (Berlekamp–Massey) and
  §2.13.8 are pinned end to end, §2.12.8 through the production χ² and P
  code, §2.5.8's χ² from its printed counts, and §2.9.8's σ and P from its
  printed sum.  The 10⁶-bit e fixture is still missing, so the matrix-rank
  computation and the serial and random-excursion examples remain unpinned.
- Fourteen DIEHARD/DIEHARDER modules have no unit tests at all
  (birthday spacings, count-ones, parking lot, runs, 3-D spheres,
  squeeze, minimum distance, byte distribution, DCT, GCD, KS uniform,
  lagged sums, minimum distance n-D, permutations); no golden p-values
  for a fixed seed; no check that the reference tables sum to 1.
  **Addressed in part:** every reference probability table has a sum-to-one
  test.  Seventeen modules gained edge-input tests: constant input fails in
  each, and empty or short input skips in those with a length gate; the
  three binary-rank tests also pin their one-word-short boundary.  Golden
  p-values for a fixed seed are still missing.
- No KAT for `Rand48`, `Xorshift32/64`, `Pcg64`, or `Lcg32`
  AnsiC/Borland.  (`DualEcDrbg` and the streaming paths of both DRBGs were
  pinned on 2026-09-10; see item C.)  `ChaCha20Rng` has no deterministic constructor, so it cannot be
  pinned to RFC 8439.  `tests/dump_rng.rs:110-142` pins 5 of 44 names.
  **Addressed:** Rand48 (against macOS libc), Xorshift32/64, Pcg64 and Lcg32
  AnsiC/Borland are pinned. `ChaCha20Rng` still has no deterministic
  constructor.
- `src/main.rs` has zero tests; `Args::parse` reads `std::env::args`
  directly.
  **Addressed:** option parsing moved to `Args::parse_from`, with tests for
  it and `group_thousands`.
- Research: `hamming_indep`, `lumped_chi_square`, `truncate_table_bits`,
  `grouped_tail_g_test`, `gorilla_aggregate_ks` untested; the PractRand
  FPF truncation rule is unverified (no source available).
  **Addressed:** each of those now has a test, `hamming_indep` against an
  independent replica. The FPF truncation rule is still unverified.

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

## Status after the 2026-09-10 fixes

Every bug (items 1–6), every issue found while fixing (A–E, B withdrawn),
and most of items 7–41 are fixed; each carries a note above.  What remains
open, by choice or for lack of a source:

- **Kept for fidelity to the cited reference:** the Dieharder fill-tree
  off-by-one (14), SP 800-22's four-decimal longest-run tables for M = 128
  and M = 10⁴ (15), both MSVC `rand()` types (35), and the unused public
  test functions (34).
- **Documented instead of changed, source not in `pubs/`:** TestU01's
  exact Hamming bit packing (12), the Gorilla paper's Anderson–Darling
  aggregate (17), and TAOCP's leading gap in the Knuth gap test (41).
- **Documented, not changed:** the 6x8 binary rank reads only the low byte
  (10), and the serial test keeps its n ≥ 1000 floor (41).
- **Not started:** the NIST tests' inline `igamc` calls, the `hex()` test
  helpers, the `take_bytes` idiom, the nearest-pair scans and the
  `runs_float` run counters (32); shared CLI code for the research
  binaries (33); pinned sibling checkouts and a macOS MSRV job in CI (40);
  `monobit2`'s inherited level aliasing (41).
- **Coverage still missing:** a 10⁶-bit e fixture for the serial and
  random-excursion examples, a deterministic `ChaCha20Rng` constructor,
  golden p-values for a fixed seed in DIEHARD/DIEHARDER, and a source for
  PractRand's FPF truncation rule.

TESTS.md was regenerated from a full battery run of 41330b0 on dyson.  None
of the later review fixes changes a result at the battery's sample size.

Four adversarial reviewers then checked this wave against its sources.  They
found two code defects, both fixed: the ApEn gates had dropped the floor in
§2.12.7, and `serial()` could report a FAIL as SKIP after rounding.  The rest
of what they found was wording in docs, notes and test claims, corrected
above.
