//! TestU01 bit-string Hamming tests from `sstring.c` / `sstring.tex`.
//!
//! # References
//! * P. L'Ecuyer and R. Simard, "TestU01: A C Library for Empirical Testing
//!   of Random Number Generators," *ACM Transactions on Mathematical
//!   Software* 33(4), Article 22, 2007, §5.2.1,
//!   "Hamming weights", pp. 19–20 (`lecuyer2007testu01` in BIB.md).
//!   [pubs/lecuyer-simard-2007-testu01.pdf]
//! * P. L'Ecuyer and R. Simard, "Beware of linear congruential generators
//!   with multipliers of the form a = ±2^q ± 2^r," *ACM Transactions on
//!   Mathematical Software* 25(3), pp. 367–374, 1999.  [The Hamming
//!   independence test; cited from the 2007 paper's reference list.]
//! * TestU01 1.2.3, `testu01/sstring.c`, and the user's guide
//!   `testu01/sstring.tex` (not in `pubs/`).
//!
//! # Author
//! Pierre L'Ecuyer and Richard Simard (TestU01); Darrell Long (Rust port).
//!
//! This module implements the core single-replication statistics for:
//! - `sstring_HammingCorr`
//! - `sstring_HammingIndep`
//!
//! Bit extraction.  Each generator call yields one 32-bit word, from which the
//! `unif01_StripB` rule keeps bits `r + 1 ..= r + s`, counted from the most
//! significant end, as an `s`-bit field (L'Ecuyer and Simard 2007, p. 22).
//! An `L`-bit block is filled from ⌈L/s⌉ successive fields.  When `s` divides
//! `L`, the block's Hamming weight is that of the next `L` bits of the
//! concatenated field stream, which is how the paper describes TestU01's bit
//! tests (§5, p. 16).  Otherwise the block's last field contributes only its
//! `L mod s` least significant bits, the rest of that field is discarded, and
//! the next block starts on a fresh word; in particular, every block with
//! `L < s` reads a single word.  That case is not claimed to match TestU01's
//! own packing, since `sstring.c` is not in `pubs/`.  The `upstream_tests`
//! defaults (`s = 10`, `L = 300`) take the divisible path.
//!
//! The main Hamming-independence chi-square lumps cells by TestU01's
//! `gofs_MinExpected = 10.0` rule.

use super::strip_b;
use crate::{
    math::{chi2_pvalue, erfc},
    result::TestResult,
    rng::Rng,
};
use std::f64::consts::{LN_2, SQRT_2};

const GOFS_MIN_EXPECTED: f64 = 10.0;

fn bit_chunks(rng: &mut impl Rng, r: usize, s: usize) -> impl Iterator<Item = (u32, usize)> + '_ {
    std::iter::from_fn(move || Some((strip_b(rng.next_u32(), r, s), s)))
}

/// Largest block length accepted by [`hamming_indep`].
///
/// The test holds two `(L + 1)²`-entry tables at once (observed `u64`
/// counts and `f64` expectations), about 256 MiB together at this bound.
pub const HAMMING_INDEP_MAX_L: usize = 4096;

/// `P(weight = k)` for an `l`-bit block of fair bits, k = 0..=l.
///
/// Accumulated in log space: `2⁻ˡ` underflows to 0 for l ≥ 1075, which
/// would zero every probability and fabricate a rejection.  Tail terms that
/// genuinely underflow come out as 0 and are lumped by the caller.
fn binomial_probs(l: usize) -> Vec<f64> {
    let mut log_p = -(l as f64) * LN_2;
    let mut probs = vec![0.0; l + 1];
    probs[0] = log_p.exp();
    for (k, p) in probs.iter_mut().enumerate().skip(1) {
        log_p += ((l + 1 - k) as f64 / k as f64).ln();
        *p = log_p.exp();
    }
    probs
}

fn next_block_weight(blocks: &mut impl Iterator<Item = (u32, usize)>, l: usize) -> Option<usize> {
    let mut remaining = l;
    let mut weight = 0usize;
    while remaining > 0 {
        let (chunk, width) = blocks.next()?;
        let take = remaining.min(width);
        let mask = if take == 32 {
            u32::MAX
        } else {
            (1u32 << take) - 1
        };
        weight += (chunk & mask).count_ones() as usize;
        remaining -= take;
    }
    Some(weight)
}

fn chi_square(expected: &[f64], observed: &[u64]) -> f64 {
    expected
        .iter()
        .zip(observed)
        .filter(|(e, _)| **e > 0.0)
        .map(|(e, &o)| {
            let d = o as f64 - *e;
            d * d / *e
        })
        .sum()
}

fn lumped_chi_square(expected: &[f64], observed: &[u64], min_expected: f64) -> (f64, usize, usize) {
    let mut kept_expected = Vec::new();
    let mut kept_observed = Vec::new();
    let mut lumped_expected = 0.0;
    let mut lumped_observed = 0u64;
    let mut lumped_cells = 0usize;

    for (&e, &o) in expected.iter().zip(observed) {
        if e >= min_expected {
            kept_expected.push(e);
            kept_observed.push(o);
        } else {
            lumped_expected += e;
            lumped_observed += o;
            lumped_cells += 1;
        }
    }

    if lumped_expected >= min_expected || kept_expected.is_empty() {
        // Pool the weak cells into their own class.  If NO cell met the
        // threshold this yields a single class → dof 0 → NaN p-value, i.e.
        // an honest insufficient-data signal rather than an arbitrary
        // low-df fallback.
        kept_expected.push(lumped_expected);
        kept_observed.push(lumped_observed);
    } else {
        let last = kept_expected.len() - 1;
        kept_expected[last] += lumped_expected;
        kept_observed[last] += lumped_observed;
    }

    let classes = kept_expected.len();
    let dof = classes.saturating_sub(1);
    (
        chi_square(&kept_expected, &kept_observed),
        dof,
        lumped_cells,
    )
}

/// Outcome of a single `sstring_HammingCorr` replication.
#[derive(Debug, Clone)]
pub struct HammingCorrSummary {
    /// Number of L-bit blocks examined.
    pub n: usize,
    /// Leading bits dropped from each 32-bit word (TestU01 `r`).
    pub r: usize,
    /// Bits kept per word after the drop (TestU01 `s`).
    pub s: usize,
    /// Block length in bits (TestU01 `L`).
    pub l: usize,
    /// Estimated correlation between successive block Hamming weights.
    pub rho_hat: f64,
    /// Normal z-score, `rho_hat · √(n − 1)`.
    pub z_score: f64,
    /// Two-sided normal p-value of the z-score.
    pub p_value: f64,
}

/// TestU01 `sstring_HammingCorr`: serial correlation between the Hamming
/// weights of `n` successive `l`-bit blocks drawn from `rng`.
///
/// # Panics
/// Panics if `n < 2`, `s` is outside `1..=32`, `r + s > 32`, or `l == 0`
/// (a zero-length block has no Hamming weight and the correlation would be
/// 0/0).
pub fn hamming_corr(
    rng: &mut impl Rng,
    n: usize,
    r: usize,
    s: usize,
    l: usize,
) -> HammingCorrSummary {
    assert!(n >= 2, "n must be at least 2");
    assert!(s > 0 && s <= 32, "s must be in 1..=32");
    assert!(r <= 32 && r + s <= 32, "r + s must be <= 32");
    assert!(l > 0, "L must be positive");
    let mut chunks = bit_chunks(rng, r, s);
    let mut prev = next_block_weight(&mut chunks, l).expect("insufficient stream");
    let mut sum = 0.0f64;
    let center = l as f64 / 2.0;
    for _ in 1..n {
        let cur = next_block_weight(&mut chunks, l).expect("insufficient stream");
        sum += (prev as f64 - center) * (cur as f64 - center);
        prev = cur;
    }
    let rho_hat = 4.0 * sum / ((n - 1) as f64 * l as f64);
    let z_score = rho_hat * ((n - 1) as f64).sqrt();
    let p_value = erfc(z_score.abs() / SQRT_2);
    HammingCorrSummary {
        n,
        r,
        s,
        l,
        rho_hat,
        z_score,
        p_value,
    }
}

/// Package a [`HammingCorrSummary`] as a [`TestResult`] named
/// `testu01::hamming_corr`.
pub fn hamming_corr_result(summary: &HammingCorrSummary) -> TestResult {
    TestResult::with_note(
        "testu01::hamming_corr",
        summary.p_value,
        format!(
            "n={}, r={}, s={}, L={}, rho_hat={:.6}, z={:.4}",
            summary.n, summary.r, summary.s, summary.l, summary.rho_hat, summary.z_score
        ),
    )
}

/// Outcome of a single `sstring_HammingIndep` replication.
#[derive(Debug, Clone)]
pub struct HammingIndepSummary {
    /// Number of (X, Y) block pairs examined.
    pub n: usize,
    /// Leading bits dropped from each 32-bit word (TestU01 `r`).
    pub r: usize,
    /// Bits kept per word after the drop (TestU01 `s`).
    pub s: usize,
    /// Block length in bits (TestU01 `L`).
    pub l: usize,
    /// Number of corner-block statistics computed (TestU01 `d`).
    pub d: usize,
    /// Chi-square over the (L+1)×(L+1) weight-pair table after lumping.
    pub main_chi_square: f64,
    /// Degrees of freedom of the main chi-square (classes − 1).
    pub main_dof: usize,
    /// Survival p-value of the main chi-square; NaN when `main_dof == 0`.
    pub main_p_value: f64,
    /// Number of low-expectation cells pooled by the
    /// `gofs_MinExpected = 10` lumping rule.
    pub lumped_cells: usize,
    /// Corner-block chi-square statistic for each `k` in `1..=d`.
    pub block_chi_square: Vec<f64>,
    /// Degrees of freedom (1 or 2) for each corner-block statistic.
    pub block_dof: Vec<usize>,
    /// Survival p-value for each corner-block statistic.
    pub block_p_value: Vec<f64>,
}

/// TestU01 `sstring_HammingIndep`: independence of the Hamming weights of
/// successive block pairs — the main lumped chi-square over the weight-pair
/// table plus `d` corner-block statistics.
///
/// # Panics
/// Panics if `n < 20`, `s` is outside `1..=32`, `r + s > 32`, `l` is
/// outside `1..=`[`HAMMING_INDEP_MAX_L`], `d` is outside `1..=8`, or
/// `d > (l + 1) / 2`.
pub fn hamming_indep(
    rng: &mut impl Rng,
    n: usize,
    r: usize,
    s: usize,
    l: usize,
    d: usize,
) -> HammingIndepSummary {
    assert!(n as f64 >= 2.0 * GOFS_MIN_EXPECTED, "n must be >= 20");
    assert!(s > 0 && s <= 32, "s must be in 1..=32");
    assert!(r <= 32 && r + s <= 32, "r + s must be <= 32");
    assert!(
        (1..=HAMMING_INDEP_MAX_L).contains(&l),
        "L must be in 1..={HAMMING_INDEP_MAX_L}"
    );
    assert!((1..=8).contains(&d), "d must be in 1..=8");
    assert!(d <= l.div_ceil(2), "d must be <= (L + 1) / 2");

    let probs = binomial_probs(l);
    let width = l + 1;
    let mut counts = vec![0u64; width * width];
    let mut chunks = bit_chunks(rng, r, s);
    for _ in 0..n {
        let x = next_block_weight(&mut chunks, l).expect("insufficient stream");
        let y = next_block_weight(&mut chunks, l).expect("insufficient stream");
        counts[x * width + y] += 1;
    }

    let mut expected = vec![0.0f64; width * width];
    for i in 0..=l {
        for j in 0..=l {
            expected[i * width + j] = n as f64 * probs[i] * probs[j];
        }
    }
    let (main_chi_square, main_dof, lumped_cells) =
        lumped_chi_square(&expected, &counts, GOFS_MIN_EXPECTED);
    // dof 0 (a single class after lumping) yields NaN: igamc rejects a = 0.
    let main_p_value = chi2_pvalue(main_chi_square, main_dof);

    let l2 = l / 2;
    let mut l1 = l / 2;
    if l % 2 == 1 {
        l1 += 1;
    }
    let mut block_chi_square = Vec::with_capacity(d);
    let mut block_dof = Vec::with_capacity(d);
    let mut block_p_value = Vec::with_capacity(d);
    for k in 1..=d {
        let mut xd0 = 0u64;
        let mut xd1 = 0u64;
        for i in 0..=l1 - k {
            for j in 0..=l1 - k {
                xd0 += counts[i * width + j];
            }
        }
        for i in l2 + k..=l {
            for j in l2 + k..=l {
                xd0 += counts[i * width + j];
            }
        }
        for i in 0..=l1 - k {
            for j in l2 + k..=l {
                xd1 += counts[i * width + j];
            }
        }
        for i in l2 + k..=l {
            for j in 0..=l1 - k {
                xd1 += counts[i * width + j];
            }
        }

        let tail_mass: f64 = probs[..=l1 - k].iter().sum();
        let nb_moyen = tail_mass * tail_mass * n as f64 * 2.0;
        let expected_block = [nb_moyen, nb_moyen, n as f64 - 2.0 * nb_moyen];
        let observed_block = [xd0, xd1, n as u64 - xd0 - xd1];
        let chi = chi_square(&expected_block, &observed_block);
        let dof = if (l % 2 == 1) && k == 1 { 1 } else { 2 };
        block_chi_square.push(chi);
        block_dof.push(dof);
        block_p_value.push(chi2_pvalue(chi, dof));
    }

    HammingIndepSummary {
        n,
        r,
        s,
        l,
        d,
        main_chi_square,
        main_dof,
        main_p_value,
        lumped_cells,
        block_chi_square,
        block_dof,
        block_p_value,
    }
}

/// Package the main lumped chi-square from `summary` as a [`TestResult`]
/// named `testu01::hamming_indep_main`.
pub fn hamming_indep_main_result(summary: &HammingIndepSummary) -> TestResult {
    TestResult::with_note(
        "testu01::hamming_indep_main",
        summary.main_p_value,
        format!(
            "n={}, r={}, s={}, L={}, dof={}, lumped_cells={}, chi2={:.4}",
            summary.n,
            summary.r,
            summary.s,
            summary.l,
            summary.main_dof,
            summary.lumped_cells,
            summary.main_chi_square
        ),
    )
}

/// Package the `k`-th corner-block statistic (`k` in `1..=d`) as a
/// [`TestResult`] named `testu01::hamming_indep_block`.
///
/// # Panics
/// Panics if `k` is 0 or exceeds `summary.d` (index out of range).
pub fn hamming_indep_block_result(summary: &HammingIndepSummary, k: usize) -> TestResult {
    let idx = k - 1;
    TestResult::with_note(
        "testu01::hamming_indep_block",
        summary.block_p_value[idx],
        format!(
            "n={}, r={}, s={}, L={}, d={}, dof={}, chi2={:.4}",
            summary.n,
            summary.r,
            summary.s,
            summary.l,
            k,
            summary.block_dof[idx],
            summary.block_chi_square[idx]
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{binomial_probs, bit_chunks, hamming_corr, hamming_indep, next_block_weight};
    use crate::rng::{ConstantRng, Rng, Xorshift32};

    /// Replays a fixed list of words.
    struct SequenceRng {
        words: Vec<u32>,
        next: usize,
    }

    impl Rng for SequenceRng {
        fn next_u32(&mut self) -> u32 {
            let word = self.words[self.next % self.words.len()];
            self.next += 1;
            word
        }
    }

    /// Top-aligns 4-bit fields so that `strip_b(word, 0, 4)` returns them.
    fn fields_as_words(fields: &[u32]) -> SequenceRng {
        SequenceRng {
            words: fields.iter().map(|f| f << 28).collect(),
            next: 0,
        }
    }

    /// Pins the extraction the module docs describe when `s` does not divide
    /// `L`: a block keeps only the low `L mod s` bits of its last field and
    /// the next block starts on a fresh word.
    #[test]
    fn block_tail_keeps_low_field_bits_and_discards_the_rest() {
        // s = 4, L = 6.  Blocks: 1111 + (00)11 = 6, then 1100 + (00)01 = 3.
        // Consecutive 6-bit blocks of the concatenated stream
        // 1111 0011 1100 0001 would weigh 4 and 4.
        let mut rng = fields_as_words(&[0b1111, 0b0011, 0b1100, 0b0001]);
        let mut chunks = bit_chunks(&mut rng, 0, 4);
        assert_eq!(Some(6), next_block_weight(&mut chunks, 6));
        assert_eq!(Some(3), next_block_weight(&mut chunks, 6));

        // s = 4, L = 2: one word per block, low two bits.  Blocks: (11)00 = 0,
        // then (00)11 = 2; the concatenated stream 1100 0011 would give 2, 0.
        let mut rng = fields_as_words(&[0b1100, 0b0011]);
        let mut chunks = bit_chunks(&mut rng, 0, 4);
        assert_eq!(Some(0), next_block_weight(&mut chunks, 2));
        assert_eq!(Some(2), next_block_weight(&mut chunks, 2));
    }

    /// Marsaglia's example xorshift32 seed; any non-zero seed would do.
    const XORSHIFT_SEED: u32 = 2_463_534_242;

    /// With every weight-pair cell below `gofs_MinExpected` the lumping
    /// leaves a single class, so the main chi-square has no degrees of
    /// freedom and must report NaN (insufficient data), not a verdict.
    #[test]
    fn hamming_indep_with_one_class_reports_nan() {
        // L = 7, n = 20: the largest cell expects 20 · (35/128)² ≈ 1.5.
        let mut rng = Xorshift32::new(XORSHIFT_SEED);
        let summary = hamming_indep(&mut rng, 20, 0, 32, 7, 1);
        assert_eq!(0, summary.main_dof);
        assert!(summary.main_p_value.is_nan());
    }

    /// Regression: the direct recurrence started from `2^-L`, which
    /// underflows to 0 for L ≥ 1075 and zeroed every probability.  The
    /// log-space accumulation must still sum to 1 and stay symmetric up to
    /// the cap, and must agree with the direct recurrence where that is exact.
    #[test]
    fn binomial_probs_survive_large_l() {
        for l in [1075usize, super::HAMMING_INDEP_MAX_L] {
            let probs = binomial_probs(l);
            let sum: f64 = probs.iter().sum();
            assert!((sum - 1.0).abs() < 1e-9, "L = {l}: sum = {sum}");
            let k = l / 2 - 10;
            assert!(probs[k] > 0.0, "L = {l}: central mass underflowed");
            assert!(
                (probs[k] - probs[l - k]).abs() <= 1e-9 * probs[k],
                "L = {l}: asymmetric"
            );
        }
        let l = 300usize;
        let mut direct = Vec::with_capacity(l + 1);
        let mut p = 0.5f64.powi(l as i32);
        direct.push(p);
        for k in 1..=l {
            p *= (l + 1 - k) as f64 / k as f64;
            direct.push(p);
        }
        for (k, (&x, &y)) in binomial_probs(l).iter().zip(&direct).enumerate() {
            assert!((x - y).abs() <= 1e-9 * y, "L = 300, k = {k}: {x} vs {y}");
        }
    }

    #[test]
    fn binomial_probs_sum_to_one() {
        let probs = binomial_probs(12);
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn hamming_corr_rejects_constant_stream() {
        let mut rng = ConstantRng::new(0);
        let summary = hamming_corr(&mut rng, 128, 0, 8, 16);
        assert!(summary.p_value < 1e-6);
    }

    /// `gofs_MinExpected` lumping, derived by hand at a threshold of 10.
    #[test]
    fn lumped_chi_square_pools_weak_cells() {
        use super::lumped_chi_square;
        // Weak cells 5 and 3 pool to 8 < 10 and join the last kept cell
        // (12 → 20, observed 14 → 22): χ² = 2²/20 + 2²/20 = 0.4, dof 1.
        let (chi, dof, lumped) = lumped_chi_square(&[20.0, 5.0, 3.0, 12.0], &[18, 7, 1, 14], 10.0);
        assert!((chi - 0.4).abs() < 1e-12, "{chi}");
        assert_eq!((1, 2), (dof, lumped));
        // Weak cells 6 and 4 pool to 10 and form a class of their own:
        // χ² = 5²/20 + 3²/12 + 4²/10 = 3.6 over three classes, dof 2.
        let (chi, dof, lumped) = lumped_chi_square(&[20.0, 6.0, 4.0, 12.0], &[25, 4, 2, 9], 10.0);
        assert!((chi - 3.6).abs() < 1e-12, "{chi}");
        assert_eq!((2, 2), (dof, lumped));
        // No cell reaches 10: everything pools into one class, dof 0.
        let (chi, dof, lumped) = lumped_chi_square(&[4.0, 3.0], &[5, 2], 10.0);
        assert!(chi.abs() < 1e-12, "{chi}");
        assert_eq!((0, 2), (dof, lumped));
    }

    /// A small deterministic HammingIndep case with `L mod s ≠ 0`, checked
    /// against an independent Python replica of the documented procedure
    /// (exact binomial cell probabilities, the same lumping and corner-block
    /// rules), with p-values from R 4.2.0 `pchisq(x, dof, lower.tail = FALSE)`.
    #[test]
    fn hamming_indep_matches_independent_replica() {
        let mut rng = Xorshift32::new(XORSHIFT_SEED);
        let summary = hamming_indep(&mut rng, 1000, 2, 5, 7, 3);
        let close = |got: f64, want: f64| (got - want).abs() <= 1e-9 * want.abs().max(1.0);
        assert!(
            close(summary.main_chi_square, 18.333_335_077_917_695),
            "{}",
            summary.main_chi_square
        );
        assert_eq!(24, summary.main_dof);
        assert_eq!(40, summary.lumped_cells);
        assert!(
            close(summary.main_p_value, 0.786_545_459_781_390_7),
            "{}",
            summary.main_p_value
        );
        let blocks = [
            (0.144, 1, 0.704_336_413_488_451_9),
            (2.393_458_040_406_143_6, 2, 0.302_181_025_137_528_76),
            (2.642_285_714_285_714_3, 2, 0.266_830_178_867_156_5),
        ];
        for (k, &(chi, dof, p)) in blocks.iter().enumerate() {
            assert!(close(summary.block_chi_square[k], chi), "d = {}", k + 1);
            assert_eq!(dof, summary.block_dof[k], "d = {}", k + 1);
            assert!(close(summary.block_p_value[k], p), "d = {}", k + 1);
        }
    }
}
