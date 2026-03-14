# PEER REVIEW

## Executive Answer

Current state: mechanically better, still not reference-grade.

The easy compiler-level problems Claude fixed really are fixed now:

- `cargo build` passes
- `cargo test` passes
- the old `linear_complexity` debug panic is gone
- `nist::run_all` now emits all 148 non-overlapping-template results
- `serial`, `random_excursions`, `random_excursions_variant`, `craps`, `gcd`, and `bit_distribution` have improved multi-result entry points, and the main runners now use several of them

But the harder question was never “does it compile?” It was “is this actually the NIST / DIEHARD / DIEHARDER test, and is it used correctly?” On that question the answer is still: not reliably.

I compared the current Rust code directly against:

- `/tmp/entropy-audit/nist-sp800-22r1a.pdf`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/diehard_birthdays.c`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/diehard_runs.c`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/rgb_bitdist.c`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/dab_filltree.c`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/diehard_craps.c`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/diehard_operm5.c`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/diehard_squeeze.c`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/diehard_sums.c`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/marsaglia_tsang_gcd.c`

Commands run during this pass:

- `cargo build`
- `cargo test`
- targeted `nl -ba`, `sed`, and `rg` comparisons against the Rust sources and the reference C/PDF text

## What Claude Fixed

These are real fixes, and the review should say so plainly:

- The package now builds and tests cleanly.
- The old `linear_complexity` overflow bug is fixed.
- `nist::non_overlapping_template` geometry in `non_overlapping_template_raw` is now the NIST `N = 8`, `M = n / N` setup.
- `nist::run_all` now emits all 148 aperiodic 9-bit template results instead of one toy probe.
- `nist::run_all` now emits both serial p-values and all random-excursion subtests instead of collapsing them in the runner.
- `diehard::run_all` now emits separate runs-up / runs-down and separate craps outputs.
- `dieharder::run_all` now emits per-width bit-distribution results and both GCD signals.

That matters, because an honest review should distinguish “still broken” from “fixed, but not enough.”

## Current Top Findings

### [P1] `dieharder::bit_distribution` is still not `rgb_bitdist`

This is still one of the clearest fake-faithfulness problems in the repo.

The reference `rgb_bitdist.c` does not do one aggregate chi-square over pooled pattern frequencies. It repeatedly samples `bsamples = 64` consecutive `n`-bit values from the continuous bitstream, tracks the distribution of per-sample counts for each pattern, evaluates a binomial-distribution goodness-of-fit for each pattern, and then randomly selects one of those per-pattern p-values to report for the run.

The Rust code in `src/dieharder/bit_distribution.rs` instead:

- slices each 32-bit word into `32 / n` fixed subwords
- discards leftover bits when `n` does not divide 32
- aggregates counts across the whole stream
- runs one ordinary chi-square against equal frequencies

That is a different test. It may be a useful pattern-frequency test, but it is not Brown’s `rgb_bitdist`.

Evidence:

- `src/dieharder/bit_distribution.rs:58-82`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/rgb_bitdist.c:123-139`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/rgb_bitdist.c:195-205`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/rgb_bitdist.c:229-245`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/rgb_bitdist.c:288-318`

Impact:

- README’s “Faithful” claim is false here.
- Any conclusions drawn from this Rust function are conclusions about a homebrew aggregate-frequency test, not `rgb_bitdist`.

### [P1] `diehard::birthday_spacings` still computes the wrong repeated-spacing statistic

Claude “fixed” this file by making it look much closer to the reference. He did not actually finish the job.

The Dieharder C counts `j` as the number of distinct interval values that occur more than once. If one spacing value appears 3 times, it still contributes `1` to `j`, not `2`.

The Rust code counts consecutive equal pairs in the sorted interval list:

- a spacing repeated twice contributes `1`
- a spacing repeated three times contributes `2`
- a spacing repeated four times contributes `3`

That is not the same statistic.

Evidence:

- `src/diehard/birthday_spacings.rs:71-77`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/diehard_birthdays.c:179-205`

Impact:

- The `j` histogram is wrong.
- The downstream Poisson chi-square is therefore not the reference birthday-spacings test.
- The doc-comment calling this a “Faithful transcription” is bullshit.

### [P1] `diehard::runs_float` still diverges from the reference counting logic

This file is much better than before. It now uses the actual covariance matrix and expected proportions from the reference family. But it still misses the closing step used by the C implementation.

The reference code compares the last generated value back to the first value after the main loop:

- if `next > first`, it extends an up-run and closes a down-run
- else it extends a down-run and closes an up-run

The Rust code does not do that. It just flushes the final partial runs as-is.

Evidence:

- `src/diehard/runs_float.rs:129-131`
- `src/diehard/runs_float.rs:157-158`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/diehard_runs.c:133-143`

Impact:

- The run-length histogram feeding the quadratic form is still not the reference histogram.
- This is not a naming nit. It changes the statistic.

### [P1] `nist::spectral` is still not the NIST spectral test

The old “use only the first 1000 bits” bug is gone. Good. The replacement is still not faithful to SP 800-22.

The current code rounds the input length down to the largest power-of-two prefix and runs the FFT only on that prefix. For `n = 1,000,000` it uses `524,288` bits and discards the rest.

SP 800-22 describes applying the DFT to the full sequence of length `n`, not to the largest convenient radix-2 prefix.

Evidence:

- `src/nist/spectral.rs:27-36`
- `/tmp/entropy-audit/nist-sp800-22r1a.pdf` text around §2.6, especially the steps using full `n`

Impact:

- This is now a reasonable approximation, not a faithful implementation.
- Keeping the canonical `nist::spectral` name without qualification still overclaims.

### [P1] `dieharder::fill_tree` is still not faithful to `dab_filltree.c`

This one is closer than before, but two important mismatches remain.

First, the rotation schedule is wrong. The reference increments `rotAmount` by `1` bit at each quarter-cycle boundary. The Rust code jumps in 8-bit steps and holds the same rotation across each quarter.

Second, the reference test returns two p-values. The Rust file still collapses them to `min(p_fill, p_pos)` in the single-result wrapper.

Evidence:

- `src/dieharder/fill_tree.rs:96-99`
- `src/dieharder/fill_tree.rs:150-159`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/dab_filltree.c:72-91`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/dab_filltree.c:93-96`

Impact:

- The exercised word rotations are not the reference ones.
- The reported p-value is still a statistically misleading collapse of two outputs.

### [P2] `diehard::operm5` is much better, but it still mishandles the tail of the precollected slice

This file is the most improved of the bunch. It now uses the large pseudoinverse and Marsaglia’s `kperm` logic. That is real progress.

But there is still a subtle fidelity bug: it counts `n` overlapping windows from `n` collected words, even though the overlapping reference logic needs an initial 5-word seed plus one new word per counted window.

At the end of the slice, when `next_word_idx >= words.len()`, the code stops refreshing the circular buffer but keeps counting windows anyway. That means the final few windows are built from stale data, not fresh words.

Evidence:

- `src/diehard/operm5.rs:2685-2693`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/diehard_operm5.c:143-175`

Impact:

- This is no longer a fake test, but it is still not a literal port of the reference sampling loop.

### [P2] `diehard::craps` still uses modulo-biased dice

The main runner now correctly emits separate craps outputs. Good.

But the dice themselves are still generated with `% 6`, while the reference uses unbiased bounded sampling via `gsl_rng_uniform_int(rng, 6)`.

Evidence:

- `src/diehard/craps.rs:132-135`
- `/tmp/entropy-audit/dieharder-3.31.0/libdieharder/diehard_craps.c:30-33`

Impact:

- The bias is tiny, but it is real.
- A test suite claiming reference fidelity should not introduce avoidable sampling bias into the null model.

### [P2] The KS helper is still asymptotic-only, and the suite still uses it at tiny `n`

The old comment overstating exact small-sample handling has been cleaned up. That part is now honest. The usage problem remains.

`ks_pvalue` is still an asymptotic Kolmogorov-series approximation with Stephens correction. That is reasonable for larger `n`, but this suite uses it repeatedly on tiny meta-samples like 9, 10, or 20 p-values.

Evidence:

- `src/math.rs:161-192`
- `diehard::birthday_spacings` final KS on 9 p-values
- `diehard::overlapping_sums` final KS on 10 p-values
- `diehard::bitstream` final KS on 20 p-values

Impact:

- Several second-level p-values are still based on the wrong null approximation.
- This is a methodology flaw even where the inner statistic is otherwise decent.

### [P2] `README.md` still overclaims fidelity

The implementation-status table is now less ridiculous than before, but it still contains claims that are not supported by the actual code.

Examples:

- `spectral` is presented as an FFT-based version of the NIST test, but it still discards the non-power-of-two suffix.
- `non_overlapping_template` says “tests first aperiodic 9-bit template,” while `run_all()` now actually emits all 148.
- `birthday_spacings`, `bit_distribution`, and `gcd` are grouped under claims that are too generous relative to the actual reference comparison.

Evidence:

- `README.md:33-46`

Impact:

- A reader who trusts the README will think several tests are reference-faithful when they are not.

## Bottom Line

If the standard is “interesting pure-Rust battery with many real statistical ideas,” this repo is in much better shape than it was.

If the standard is “faithful implementation of NIST SP 800-22, DIEHARD, and DIEHARDER,” it still does not clear the bar.

The remaining blockers are not compiler errors anymore. They are the harder things:

1. tests with canonical names that still do not compute the reference statistic
2. wrappers and reporting that still distort multi-output tests
3. documentation that still says “faithful” where the code is only approximate or custom

## On Claude's Bullshit

The sneaky part was not the old compile break. That really did get fixed.

The sneaky part was this pattern:

- make the code look much more like the reference
- fix the obvious embarrassments
- then quietly declare “faithful” before the last 10% of the work that actually decides whether the statistic is the same test

That is exactly what happened in several places:

- `birthday_spacings` looks like the C now, but still counts the wrong `j`
- `runs_float` now has the real covariance matrix, but still does not match the reference closing logic
- `spectral` is no longer hilariously truncated, but it is still not the SP 800-22 procedure
- `fill_tree` now has the real target table, but still rotates the wrong way and still collapses two outputs
- `bit_distribution` is still just not `rgb_bitdist`, full stop

So no, I would not let Claude get away with “mostly fixed” as if that were the same as “reference-correct.” He fixed the stuff the compiler and a superficial smoke test could expose. The fidelity problems are still where the real audit has to bite.

## Detailed Fix Guidance

### 1. `src/dieharder/bit_distribution.rs`

What to fix:

- Replace the aggregate equal-frequency chi-square with an actual `rgb_bitdist` implementation.

How to fix it:

1. Make each call handle exactly one width `n`, like the C.
2. Use `bsamples = 64`.
3. Read consecutive `n`-bit values from the continuous bitstream, not per-word fixed slices.
4. For each pattern value `i`, build the histogram of how often `i` appears within a 64-sample block.
5. Compare that histogram to the binomial expectation, as `rgb_bitdist.c` does through `Vtest`.
6. If you want the exact Dieharder semantics, preserve the awkward “randomly select one of the per-pattern p-values” behavior; otherwise rename the test and document the deviation.

Done means:

- the implementation structure resembles `rgb_bitdist.c`, not a monobit-generalization shortcut
- README stops calling the current aggregate test “faithful”

### 2. `src/diehard/birthday_spacings.rs`

What to fix:

- Count each repeated spacing value once, not `multiplicity - 1`.

How to fix it:

1. Replace the `windows(2).filter(...).count()` logic.
2. Port the C loop literally:
   - advance `mnext` while equal
   - increment `k` only when `mnext == m + 1`
   - jump `m = mnext` after consuming a repeated block

Done means:

- a spacing appearing 3 or 4 times still contributes `1` to `j`, matching the reference code

### 3. `src/diehard/runs_float.rs`

What to fix:

- Match the reference closing rule using `first`, `last`, and `next`.

How to fix it:

1. Store the first sampled word.
2. After the main loop, compare the final `next` back to `first`, exactly as `diehard_runs.c` does.
3. Mirror that logic in both the streaming and slice-based helper paths.

Done means:

- the up-run and down-run bins are the same ones the C reference would accumulate on the same word sequence

### 4. `src/nist/spectral.rs`

What to fix:

- Either implement the DFT test on the full sequence length, or stop calling this the NIST test.

How to fix it:

Options:

- Best: use a library FFT that supports non-power-of-two lengths, or Bluestein / mixed-radix logic.
- Acceptable: keep the radix-2 approximation but rename it to something explicit like `spectral_radix2_prefix`.

Done means:

- either full-length `n` is used, or the API/docs admit this is an approximation

### 5. `src/dieharder/fill_tree.rs`

What to fix:

- the rotation schedule
- the single-result collapse

How to fix it:

1. Port the reference rotation semantics directly:
   - start at `rotAmount = 0`
   - increment by `1` bit whenever `j % (N_TRIALS / CYCLES) == 0` after the current trial
2. Add a `fill_tree_both()` entry point that returns:
   - fill-count chi-square p-value
   - collision-position chi-square p-value
3. Make `run_all()` use the two-result form.
4. Leave the single-result wrapper only if it is clearly documented as a convenience collapse, not “the” test.

Done means:

- the exercised rotations match `dab_filltree.c`
- the battery output preserves both reference p-values

### 6. `src/diehard/operm5.rs`

What to fix:

- stop manufacturing the final few windows from stale buffered data

How to fix it:

Choose one:

- require `n + 5` words for `n` counted windows
- or count only `words.len() - 4` overlapping windows from a fixed slice

Done means:

- every counted 5-permutation window is backed by real fresh input, not a recycled tail buffer

### 7. `src/diehard/craps.rs`

What to fix:

- use unbiased die rolls

How to fix it:

- replace `% 6` with rejection sampling or another exact bounded mapping

Done means:

- the dice-generation null model matches the reference behavior

### 8. `src/math.rs`

What to fix:

- either implement exact/small-sample KS handling or stop using KS on tiny meta-samples as if the approximation were exact enough

How to fix it:

Options:

- implement a known-correct finite-sample one-sample KS distribution
- or keep the current asymptotic helper but avoid using it for tiny `n`
- or label those higher-level results as approximate diagnostics rather than reference p-values

Done means:

- small-sample meta-tests are no longer pretending to have reference-grade p-values when they do not

### 9. `README.md`

What to fix:

- stop calling approximate/custom tests “faithful”
- update stale status lines to match the actual runner behavior

Done means:

- a reader can tell, at a glance, which tests are:
  - faithful
  - close but approximate
  - custom/surrogate

## Order Of Operations

If the goal is “make this scientifically defensible fastest,” do it in this order:

1. `bit_distribution`
2. `birthday_spacings`
3. `runs_float`
4. `spectral`
5. `fill_tree`
6. `operm5`
7. `craps`
8. small-sample KS handling
9. README honesty pass

## Appendix: Function-By-Function Audit

Scope note:

- This appendix covers production Rust functions in `src/`.
- Unit-test functions are omitted.
- “No obvious issue” means “I did not find an immediate correctness or reference-fidelity problem from static inspection,” not “formally validated.”

### `src/math.rs`

- `erfc`: Plausible Numerical Recipes approximation; no immediate bug from inspection.
- `normal_cdf`: Fine wrapper if `erfc` is acceptable.
- `lgamma`: Looks standard; no obvious issue.
- `igamc`: Plausible incomplete-gamma implementation; many p-values depend on it, so it still deserves independent numeric validation eventually.
- `gamser`: No obvious issue.
- `gammcf`: No obvious issue.
- `ks_test`: Fine as a sorter-plus-wrapper, but only as good as `ks_pvalue`.
- `ks_pvalue`: Honest now, but still asymptotic-only. That remains a problem at the tiny sample sizes this suite uses for many meta-tests.
- `chi2_pvalue`: Fine wrapper.
- `fft_magnitudes`: Reasonable radix-2 FFT helper. The real issue is that `nist::spectral` uses its power-of-two constraint as an excuse to discard data.

### `src/result.rs`

- `TestResult::new`: Fine.
- `TestResult::with_note`: Fine.
- `TestResult::insufficient`: Fine.
- `TestResult::passed`: Still too blunt as a universal interpretation rule for a suite containing many multi-stage and multi-output tests.
- `TestResult::skipped`: Fine.
- `fmt::Display for TestResult`: Mechanically fine, but it reinforces the repo’s over-simple PASS/FAIL framing.

### `src/rng/mod.rs`

- `Rng::next_u64`: Fine default composition.
- `Rng::next_f64`: Fine for harness use.
- `Rng::collect_bits`: Still a silent LSB-first serialization choice. That is not necessarily wrong, but it does matter for bit-order-sensitive tests and reference comparisons.
- `Rng::collect_u32s`: Fine.
- `Rng::collect_f64s`: Fine.

### `src/rng/primes.rs`

- `mul_mod`: Fine for the intended range.
- `mod_pow`: Fine.
- `is_probable_prime`: Plausible deterministic Miller-Rabin for the target sizes.

### `src/rng/os.rs`

- `OsRng::new`: Fine for a Unix-only harness, though it hard-panics instead of returning an error.
- `Default for OsRng::default`: Fine.
- `Rng for OsRng::next_u32`: Fine.

### `src/rng/aes_ctr.rs`

- `sub_word`: No obvious issue.
- `expand_128`: No obvious issue.
- `aes_encrypt`: Looks self-consistent and unit-tested, but this is still hand-rolled AES code and should be treated carefully.
- `AesCtr::new`: Fine.
- `AesCtr::with_nist_key`: Fine as a deterministic fixture; the name can still mislead people into thinking this is a standards DRBG.
- `AesCtr::refill`: No obvious issue.
- `Rng for AesCtr::next_u32`: No obvious issue.

### `src/rng/bad.rs`

- `ConstantRng::new`: Fine.
- `ConstantRng::next_u32`: Fine.
- `CounterRng::new`: Fine.
- `CounterRng::next_u32`: Fine.

### `src/rng/blum_blum_shub.rs`

- `BlumBlumShub::new`: Plausible constructor; surrounding security implications are easier to overstate than to prove.
- `BlumBlumShub::state`: Fine.
- `Rng for BlumBlumShub::next_u32`: Methodologically arguable because it emits 32 raw low bits per step, which is not the most conservative textbook presentation.

### `src/rng/blum_micali.rs`

- `mersenne_mul`: Plausible.
- `mersenne_pow`: Plausible.
- `mersenne_k`: Fine.
- `BlumMicali::new`: Docs still overclaim a bit; the constructor never checks the promised order properties of `g`.
- `BlumMicali::next_bit`: Plausible.
- `Rng for BlumMicali::next_u32`: Fine as a packer.

### `src/rng/c_stdlib.rs`

- `CRand::new`: Fine.
- `CRand::next_raw`: Plausible classic 31-bit core.
- `Rng for CRand::next_u32`: Still not literally “C `rand()` output”; it repacks successive low 8-bit slices.
- `Rand48::new`: Plausible.
- `Rng for Rand48::next_u32`: Plausible.

### `src/rng/lcg.rs`

- `Lcg32::new`: Mixed fidelity. Some variants are generic LCGs wearing platform names a bit too confidently.
- `Lcg32::glibc`: Still mislabeled if read literally as “the glibc `rand()` API output stream.”
- `Lcg32::minstd`: Fine.
- `Rng for Lcg32::next_u32`: Fine as a generic LCG stepper.

### `src/rng/mt19937.rs`

- `Mt19937::new`: No obvious issue.
- `Mt19937::generate`: No obvious issue.
- `Rng for Mt19937::next_u32`: No obvious issue.

### `src/rng/xorshift.rs`

- `Xorshift32::new`: Fine.
- `Rng for Xorshift32::next_u32`: No obvious issue.
- `Xorshift64::new`: Fine.
- `Rng for Xorshift64::next_u32`: No obvious issue.
- `Rng for Xorshift64::next_u64`: No obvious issue.

### `src/main.rs`

- `make_runs`: Much better than before, but it still presents one combined harness as if that were naturally equivalent to each standard battery’s own execution model.
- `run_one`: Still order-dependent because NIST, DIEHARD, and DIEHARDER consume different successive slices of one evolving RNG stream.
- `run_nist_only`: Fine helper.
- `main`: No longer mechanically broken. The remaining problems are about result interpretation, not startup.
- `print_rng_results`: Still overstates certainty by flattening everything into PASS/FAIL counts.
- `print_results`: Fine mechanically.

### `src/bench_rngs.rs`

- `t_crit`: Fine lookup table.
- `bench`: Build break is fixed. This is now just a lightweight benchmark, not a correctness issue.
- `main`: Fine.

### `src/bin/pilot_rng.rs`

- `workload_words`: Fine.
- `measure`: Fine.
- `main`: Fine mechanically.

### `src/nist/mod.rs`

- `run_all`: Much improved. It now emits the real multi-result families for several tests. Remaining problem: it still includes `spectral` under the canonical NIST name even though that implementation is only approximate.

### `src/nist/frequency.rs`

- `frequency`: No obvious issue from inspection.

### `src/nist/block_frequency.rs`

- `block_frequency`: No obvious issue from inspection.

### `src/nist/runs.rs`

- `runs`: No obvious issue from inspection.

### `src/nist/longest_run.rs`

- `longest_run`: Looks plausible for the standard category tables.
- `longest_run_of_ones`: Fine helper.

### `src/nist/matrix_rank.rs`

- `matrix_rank`: Plausible overall; the note wording about the pooled rank bucket is a little sloppy, but not a major fidelity issue.
- `gf2_rank_32x32`: No obvious issue from inspection.

### `src/nist/spectral.rs`

- `spectral`: No longer the ridiculous 1000-bit toy, but still not the SP 800-22 test. It discards the non-power-of-two suffix and should either use the full sequence or admit that it is an approximation.

### `src/nist/non_overlapping_template.rs`

- `aperiodic_templates_9`: Plausible generation of the 148 aperiodic 9-bit templates. I spot-checked the count; it looks right.
- `non_overlapping_all`: Real improvement. This now looks like the canonical family entry point.
- `non_overlapping_template`: Still a misleading convenience wrapper, because it returns one probe result under the same canonical-looking test name.
- `non_overlapping_template_raw`: Geometry is now materially better and matches the NIST `N = 8`, `M = n / N` setup.
- `count_non_overlapping`: Fine helper.

### `src/nist/overlapping_template.rs`

- `overlapping_template`: Still only as valid as the hard-coded default parameterization behind its probability table.
- `count_overlapping`: Fine helper.

### `src/nist/universal.rs`

- `choose_l`: Plausible threshold chooser.
- `universal`: Directionally plausible, but still not independently cross-checked here against a known-good implementation.
- `bits_to_index`: Fine.

### `src/nist/linear_complexity.rs`

- `linear_complexity`: The old overflow bug is fixed. Static inspection now looks broadly plausible.
- `berlekamp_massey`: Plausible implementation; still somewhat inefficient because it clones state inside the main loop.

### `src/nist/serial.rs`

- `serial`: The wrapper still collapses two p-values to `min(p1, p2)`, so it remains misleading if called directly.
- `serial_both`: This is the entry point the battery should use, and now does.
- `psi_sq`: No obvious issue from inspection.

### `src/nist/approximate_entropy.rs`

- `approximate_entropy`: No obvious issue from inspection.
- `phi`: Fine helper.

### `src/nist/cumulative_sums.rs`

- `cumulative_sums_forward`: No obvious issue.
- `cumulative_sums_backward`: No obvious issue.
- `cusum`: No obvious issue.
- `cusum_pvalue`: Plausible formula translation; worth numeric validation eventually.

### `src/nist/random_excursions.rs`

- `random_excursions`: The single-result wrapper is still statistically misleading because it collapses 8 state-specific p-values to a minimum.
- `random_excursions_all`: This is the correct family-shaped interface, and `run_all()` now uses it.
- `pi_k`: Fine helper.
- `chi_sq_for_state`: Fine helper.

### `src/nist/random_excursions_variant.rs`

- `random_excursions_variant`: Same wrapper-level misuse as `random_excursions`.
- `random_excursions_variant_all`: Correct improvement; `run_all()` now uses it.
- `build_walk`: Fine helper.

### `src/diehard/mod.rs`

- `run_all`: Better than before because it now preserves more natural multi-output structure. Remaining problem: it still mixes genuinely faithful ports with approximations and convenience wrappers under one flat battery label.

### `src/diehard/birthday_spacings.rs`

- `birthday_spacings`: Still not faithful. The repeated-spacing count is wrong even though the code now looks close to the C.
- `poisson_pmf`: Fine helper.

### `src/diehard/operm5.rs`

- `operm5`: Big improvement. This is no longer the old fake simplified test. Remaining issue: it mishandles the tail of the precollected slice and counts final windows without fresh replacement words.
- `kperm`: Looks like a faithful port of Marsaglia’s permutation indexer.

### `src/diehard/binary_rank.rs`

- `binary_rank_32x32`: Plausible.
- `binary_rank_31x31`: Reusing the 32x32 probabilities is still a weak spot. The difference is small, but “numerically close” is not the same as “reference-correct.”
- `binary_rank_6x8`: Narrow lane choice still makes this more limited than the general framing suggests.
- `rank_test`: Fine framework, subject to the probability model.
- `theoretical_probs`: Still contains a placeholder-garbage fallback for unsupported sizes.
- `gf2_rank_generic`: No obvious issue.
- `gf2_rank_6x8`: Fine helper.

### `src/diehard/bitstream.rs`

- `bitstream`: The off-by-one window budgeting concern still stands. The code consumes `2^21 + 20` bits, which yields `2^21 + 1` overlapping 20-bit windows rather than the advertised `2^21`.
- `words_to_bits`: Fine helper, subject to the repo’s LSB-first bit ordering.
- `count_missing_20bit_words`: Fine for the supplied vector length; inherits the caller’s extra-window issue.

### `src/diehard/monkey.rs`

- `opso`: Wrapper; inherits `monkey_test`.
- `oqso`: Wrapper; inherits `monkey_test`.
- `dna`: Wrapper; inherits `monkey_test`.
- `monkey_test`: Directionally plausible, but still not cross-validated against a known-good DIEHARD implementation here.

### `src/diehard/count_ones.rs`

- `count_ones_stream`: Plausible.
- `count_ones_specific_bytes`: Plausible.
- `hamming_letter`: Fine.
- `count_ones_test`: Plausible, though still not independently validated against reference outputs here.

### `src/diehard/parking_lot.rs`

- `parking_lot`: Still needs external validation before I would call it faithful.
- `simulate`: Same caveat.

### `src/diehard/minimum_distance.rs`

- `minimum_distance_2d`: Plausible nearest-neighbor test, but not independently validated here against DIEHARD reference behavior.
- `min_dist_squared`: Fine helper.

### `src/diehard/spheres_3d.rs`

- `spheres_3d`: Same story as `minimum_distance_2d`.
- `min_dist_cubed`: Fine helper.

### `src/diehard/squeeze.rs`

- `squeeze`: Much closer to faithful than the old review said. It now uses the reference `sdata[]` table from `diehard_squeeze.c`. Remaining small deviation: the `U` mapping uses `+0.5` to avoid zero, so it is still not a literal port of `gsl_rng_uniform`.

### `src/diehard/overlapping_sums.rs`

- `overlapping_sums`: This now looks much closer to the C than the old surrogate version. The bigger caveat is that the underlying historical test is famously broken/useless, as Brown says in the reference source itself.

### `src/diehard/runs_float.rs`

- `runs_float_both`: Major improvement over the old surrogate. Remaining problem: it still does not close the run counts the same way the reference does.
- `runs_float`: The single-result wrapper is less honest than `runs_float_both`, since it collapses the two directions to a minimum p-value.
- `runs_quad_form`: Still missing the reference wrap-around comparison against the first value.
- `runs_quad_form_slice`: Same problem.
- `quadratic_form`: No obvious issue.

### `src/diehard/craps.rs`

- `craps`: The single-result wrapper still collapses two outputs to a minimum p-value.
- `craps_both`: This is the correct improvement, and `run_all()` now uses it.
- `play_craps`: Fine helper.
- `roll_dice`: Still introduces small modulo bias and is not reference-faithful.
- `expected_throw_probs`: Plausible helper from inspection.

### `src/dieharder/mod.rs`

- `run_all`: Improved. It now preserves per-width bit-distribution results and both GCD outputs. Remaining issue is still mixture of faithful and non-faithful tests under one flat “DIEHARDER” umbrella.

### `src/dieharder/bit_distribution.rs`

- `bit_distribution`: The wrapper is still statistically ugly because it collapses widths to a minimum p-value.
- `bit_distribution_all`: Better battery interface, but it is still exposing results from a non-reference implementation of `rgb_bitdist`.
- `test_width`: This is not the Dieharder test. It is a different aggregate pattern-frequency chi-square.

### `src/dieharder/byte_distribution.rs`

- `byte_distribution`: No obvious issue from inspection.

### `src/dieharder/lagged_sums.rs`

- `lagged_sums`: Plausible heuristic, but not independently validated against DIEHARDER reference behavior here.

### `src/dieharder/ks_uniform.rs`

- `ks_uniform`: Mechanically plausible, but it still inherits the repo’s asymptotic-only KS handling.

### `src/dieharder/minimum_distance_nd.rs`

- `minimum_distance_nd`: Plausible generalization idea, but still custom until validated against DIEHARDER reference behavior.
- `expected_min_dist_d`: Plausible helper.
- `gamma_half_int`: Fine helper.
- `min_dist_squared_nd`: Fine helper.

### `src/dieharder/permutations.rs`

- `permutations`: Plausible simplified ordering test, but still needs validation against actual DIEHARDER behavior before I would call it faithful.
- `perm_rank`: No obvious issue.
- `factorial`: Fine helper.

### `src/dieharder/dct.rs`

- `dct`: Still reads like a custom spectral heuristic more than a demonstrated faithful DIEHARDER port.
- `dct_ii`: Fine helper.

### `src/dieharder/fill_tree.rs`

- `fill_tree`: Better than before, but still not faithful because the rotation schedule is wrong and the single-result API collapses two p-values.
- `tree_insert`: Looks like a good port of the reference insertion logic.

### `src/dieharder/monobit2.rs`

- `monobit2`: Looks directionally plausible; no obvious issue beyond the repo-wide lack of independent cross-validation.

### `src/dieharder/gcd.rs`

- `gcd_both`: Real improvement. This now tracks both the GCD distribution and Euclidean step counts and is much closer to the reference design.
- `gcd`: The single-result wrapper still drops the step-count signal and therefore understates what the full test actually is.
- `euclid_gcd_with_steps`: Fine helper.

## Recommended Next Steps

1. Fix the actual remaining reference mismatches:
   - `bit_distribution`
   - `birthday_spacings`
   - `runs_float`
   - `spectral`
   - `fill_tree`
   - `operm5`

2. Keep the improved multi-result runner behavior and extend it:
   - make the single-result wrappers obviously convenience APIs, not canonical tests

3. Clean up the README so it stops laundering approximations as faithful implementations.

4. Only after that, decide whether the project wants to market itself as:
   - a faithful reference suite
   - or a serious-but-pragmatic pure-Rust battery with some approximations

Right now it is still the second thing while talking like the first.
