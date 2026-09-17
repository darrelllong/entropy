//! Hamming-weight tests of L'Ecuyer and Simard.
//!
//! P. L'Ecuyer and R. Simard, "TestU01: A C Library for Empirical Testing of
//! Random Number Generators," *ACM Transactions on Mathematical Software*
//! 33(4), Article 22, 2007, §5.2.1, "Hamming weights", pp. 19–20
//! (`lecuyer2007testu01` in BIB.md).  [pubs/lecuyer-simard-2007-testu01.pdf]
//! The independence test is from P. L'Ecuyer and R. Simard, "Beware of
//! linear congruential generators with multipliers of the form
//! a = ±2^q ± 2^r," *ACM Transactions on Mathematical Software* 25(3),
//! pp. 367–374, 1999.
//!
//! Both tests read successive L-bit blocks and their Hamming weights, which
//! are Binomial(L, ½) and independent under the null.
//!
//! - [`hamming_corr`] estimates the correlation of successive weights,
//!   ρ̂ = 4 Σ (Xᵢ − L/2)(Xᵢ₊₁ − L/2) / ((n − 1)L), and scores
//!   z = ρ̂√(n − 1) as standard normal, two-sided, so that an excess and a
//!   deficit of correlation both fail.
//! - [`hamming_indep`] counts n pairs of successive weights (X, Y) in the
//!   (L + 1) × (L + 1) table and scores it with a Pearson χ² against
//!   n·P(X = i)·P(Y = j).  Cells expecting fewer than 10 pairs are pooled;
//!   the pool is a cell of its own if it expects at least 10, and otherwise
//!   joins the last kept cell.  If no cell expects 10, the table is split
//!   into the columns j ≤ ⌊L/2⌋ and j > ⌊L/2⌋, one degree of freedom.  For
//!   k = 1 … d it also counts the pairs in the corners of the table: both
//!   weights at least k below the middle or both at least k above it, and
//!   one of each, each expecting 2n·P(X ≤ ⌈L/2⌉ − k)² by symmetry, with the
//!   rest in a third cell.  For odd L and k = 1 the corners cover the whole
//!   table, and the statistic has one degree of freedom instead of two.
//!
//! Bit extraction.  Each generator call yields one 32-bit word, of which the
//! `s` bits after the leading `r` form a field.  When
//! `L ≥ s`, a block adds the weights of ⌊L/s⌋ successive fields and, when `s`
//! does not divide `L`, the weight of the leading `L mod s` bits of one more
//! word's window.  When `L < s`, each field supplies ⌊s/L⌋ blocks, taken from
//! its least significant end, and its `s mod L` most significant bits are
//! unused.  A block therefore equals the next `L` bits of the concatenated
//! field stream only when `s` divides `L`, as at the `upstream_tests`
//! defaults (`s = 10`, `L = 300`).

use super::strip_b;
use crate::{
    math::{chi2_pvalue, erfc},
    result::TestResult,
    rng::Rng,
};
use std::f64::consts::{LN_2, SQRT_2};

/// Smallest expected count of a cell in the independence test's main χ².
const MIN_EXPECTED: f64 = 10.0;

/// Hamming weights of successive `L`-bit blocks, packed from fields as the
/// module documentation describes.
struct BlockWeights<'a, R: Rng> {
    rng: &'a mut R,
    r: usize,
    s: usize,
    l: usize,
    /// The field still supplying blocks (`L < s` only), shifted so that its
    /// next block sits in the low `L` bits.
    field: u32,
    /// Blocks left in `field` (`L < s` only).
    blocks_left: usize,
}

impl<'a, R: Rng> BlockWeights<'a, R> {
    /// Callers guarantee `1 <= s <= 32`, `r + s <= 32` and `l >= 1`.
    fn new(rng: &'a mut R, r: usize, s: usize, l: usize) -> Self {
        Self {
            rng,
            r,
            s,
            l,
            field: 0,
            blocks_left: 0,
        }
    }

    fn next_weight(&mut self) -> usize {
        let (r, s, l) = (self.r, self.s, self.l);
        if l >= s {
            // ⌊L/s⌋ whole fields, then the leading `L mod s` bits of one more
            // word's window.
            let mut weight = 0;
            for _ in 0..l / s {
                weight += strip_b(self.rng.next_u32(), r, s).count_ones() as usize;
            }
            if l % s > 0 {
                weight += strip_b(self.rng.next_u32(), r, l % s).count_ones() as usize;
            }
            weight
        } else {
            // ⌊s/L⌋ blocks per field, least significant first.  `l < s <= 32`,
            // so both shifts fit.
            if self.blocks_left == 0 {
                self.field = strip_b(self.rng.next_u32(), r, s);
                self.blocks_left = s / l;
            }
            let weight = (self.field & ((1u32 << l) - 1)).count_ones() as usize;
            self.field >>= l;
            self.blocks_left -= 1;
            weight
        }
    }
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
        // The pooled cells form a class of their own.  If no cell met the
        // threshold, that is the only class (dof 0), and `hamming_indep`
        // splits the table in two.
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

/// Outcome of one Hamming-correlation run.
#[derive(Debug, Clone)]
pub struct HammingCorrSummary {
    /// Number of L-bit blocks examined.
    pub n: usize,
    /// Leading bits dropped from each 32-bit word.
    pub r: usize,
    /// Bits kept per word after the drop.
    pub s: usize,
    /// Block length in bits.
    pub l: usize,
    /// Estimated correlation between successive block Hamming weights.
    pub rho_hat: f64,
    /// Normal z-score, `rho_hat · √(n − 1)`.
    pub z_score: f64,
    /// Two-sided normal p-value of the z-score.
    pub p_value: f64,
}

/// Serial correlation between the Hamming
/// weights of `n` successive `l`-bit blocks drawn from `rng`, reported with a
/// two-sided p-value (see the module docs).
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
    let mut blocks = BlockWeights::new(rng, r, s, l);
    let mut prev = blocks.next_weight();
    let mut sum = 0.0f64;
    let center = l as f64 / 2.0;
    for _ in 1..n {
        let cur = blocks.next_weight();
        sum += (prev as f64 - center) * (cur as f64 - center);
        prev = cur;
    }
    let rho_hat = 4.0 * sum / ((n - 1) as f64 * l as f64);
    let z_score = rho_hat * ((n - 1) as f64).sqrt();
    // A guard only: `math::erfc` never exceeds 1 for a non-negative argument.
    let p_value = erfc(z_score.abs() / SQRT_2).min(1.0);
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
    .normal(summary.z_score)
}

/// Outcome of one Hamming-independence run.
#[derive(Debug, Clone)]
pub struct HammingIndepSummary {
    /// Number of (X, Y) block pairs examined.
    pub n: usize,
    /// Leading bits dropped from each 32-bit word.
    pub r: usize,
    /// Bits kept per word after the drop.
    pub s: usize,
    /// Block length in bits.
    pub l: usize,
    /// Number of corner-block statistics computed.
    pub d: usize,
    /// Chi-square over the (L+1)×(L+1) weight-pair table after lumping.
    pub main_chi_square: f64,
    /// Degrees of freedom of the main chi-square (classes − 1).
    pub main_dof: usize,
    /// Survival p-value of the main chi-square.
    pub main_p_value: f64,
    /// Number of cells expecting fewer than 10 pairs, pooled.
    pub lumped_cells: usize,
    /// Corner-block chi-square statistic for each `k` in `1..=d`.
    pub block_chi_square: Vec<f64>,
    /// Degrees of freedom (1 or 2) for each corner-block statistic.
    pub block_dof: Vec<usize>,
    /// Survival p-value for each corner-block statistic.
    pub block_p_value: Vec<f64>,
}

/// Independence of the Hamming weights of
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
    assert!(n as f64 >= 2.0 * MIN_EXPECTED, "n must be >= 20");
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
    let mut blocks = BlockWeights::new(rng, r, s, l);
    for _ in 0..n {
        let x = blocks.next_weight();
        let y = blocks.next_weight();
        counts[x * width + y] += 1;
    }

    let mut expected = vec![0.0f64; width * width];
    for i in 0..=l {
        for j in 0..=l {
            expected[i * width + j] = n as f64 * probs[i] * probs[j];
        }
    }
    let (mut main_chi_square, mut main_dof, lumped_cells) =
        lumped_chi_square(&expected, &counts, MIN_EXPECTED);
    if main_dof == 0 {
        // Every cell pooled into one class: split the table into the columns
        // j ≤ L/2 and j > L/2 instead.
        let mut half_expected = [0.0f64; 2];
        let mut half_observed = [0u64; 2];
        for i in 0..=l {
            for j in 0..=l {
                let half = usize::from(j > l / 2);
                half_expected[half] += expected[i * width + j];
                half_observed[half] += counts[i * width + j];
            }
        }
        main_chi_square = chi_square(&half_expected, &half_observed);
        main_dof = 1;
    }
    let main_p_value = chi2_pvalue(main_chi_square, main_dof);

    // Corners at distance k from the middle: weights w ≤ ⌈L/2⌉ − k are "low"
    // and w ≥ ⌊L/2⌋ + k "high"; the two tails have equal probability.
    let low_end = l.div_ceil(2);
    let high_start = l / 2;
    let mut block_chi_square = Vec::with_capacity(d);
    let mut block_dof = Vec::with_capacity(d);
    let mut block_p_value = Vec::with_capacity(d);
    for k in 1..=d {
        let low = 0..=low_end - k;
        let high = high_start + k..=l;
        let sum = |rows: &std::ops::RangeInclusive<usize>,
                   cols: &std::ops::RangeInclusive<usize>| {
            rows.clone()
                .map(|i| cols.clone().map(|j| counts[i * width + j]).sum::<u64>())
                .sum::<u64>()
        };
        let same_side = sum(&low, &low) + sum(&high, &high);
        let opposite = sum(&low, &high) + sum(&high, &low);
        let tail: f64 = probs[low.clone()].iter().sum();
        let corner_expected = 2.0 * n as f64 * tail * tail;
        let expected_cells = [
            corner_expected,
            corner_expected,
            n as f64 - 2.0 * corner_expected,
        ];
        let observed_cells = [same_side, opposite, n as u64 - same_side - opposite];
        let chi = chi_square(&expected_cells, &observed_cells);
        let dof = if l % 2 == 1 && k == 1 { 1 } else { 2 };
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
    .chi_square(summary.main_chi_square, summary.main_dof as f64)
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
    .chi_square(summary.block_chi_square[idx], summary.block_dof[idx] as f64)
}

#[cfg(test)]
mod tests {
    /// A statistic this test computes exactly from small counts.
    const CLOSED_FORM: f64 = 1e-12;

    /// A sum or symmetry of binomial probabilities, which round as they accumulate.
    const SUMMED_PROBABILITY: f64 = 1e-9;

    use super::{binomial_probs, hamming_corr, hamming_indep, BlockWeights};
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

    /// Words whose leading bits are `fields`, each `width` bits wide, so that
    /// `strip_b(word, 0, width)` returns them.
    fn fields_as_words(fields: &[u32], width: u32) -> SequenceRng {
        SequenceRng {
            words: fields.iter().map(|f| f << (32 - width)).collect(),
            next: 0,
        }
    }

    /// `L > s` with `s ∤ L` (`HammingCorr_L`): a block adds ⌊L/s⌋ whole
    /// fields, then the leading `L mod s` bits of one more word's window.
    #[test]
    fn ragged_block_tail_reads_the_leading_bits_of_one_more_word() {
        // s = 4, L = 6.  Block 1: 1111, then the top two bits of 0011: 4 + 0.
        // Block 2: 1100, then the top two bits of 1001: 2 + 1.  Reading the
        // low two bits instead would give 6 and 3.
        let mut rng = fields_as_words(&[0b1111, 0b0011, 0b1100, 0b1001], 4);
        {
            let mut blocks = BlockWeights::new(&mut rng, 0, 4, 6);
            assert_eq!(4, blocks.next_weight());
            assert_eq!(3, blocks.next_weight());
        }
        assert_eq!(4, rng.next, "two words per block");

        // The same rule inside an r = 4 window: the whole field 1111 from
        // bits 5..=8, then bits 5..=6 of the next word, 11 from 1100: 4 + 2.
        let mut rng = SequenceRng {
            words: vec![0b1111 << 24, 0b1100 << 24],
            next: 0,
        };
        assert_eq!(6, BlockWeights::new(&mut rng, 4, 4, 6).next_weight());
    }

    /// `L < s` (`HammingCorr_S`): each field yields ⌊s/L⌋ blocks from its
    /// least significant end, and its top `s mod L` bits go unused.
    #[test]
    fn short_blocks_share_a_field_low_bits_first() {
        // s = 5, L = 2.  Field 10110 gives 10 (1), then 01 (1), and its top
        // bit is unused; field 00011 gives 11 (2), then 00 (0).
        let mut rng = fields_as_words(&[0b10110, 0b00011, 0b11111], 5);
        {
            let mut blocks = BlockWeights::new(&mut rng, 0, 5, 2);
            let weights: Vec<usize> = (0..4).map(|_| blocks.next_weight()).collect();
            assert_eq!(vec![1, 1, 2, 0], weights);
        }
        assert_eq!(2, rng.next, "two blocks per word");

        // A fifth block draws a third field whole.
        let mut rng = fields_as_words(&[0b10110, 0b00011, 0b11111], 5);
        {
            let mut blocks = BlockWeights::new(&mut rng, 0, 5, 2);
            let weights: Vec<usize> = (0..5).map(|_| blocks.next_weight()).collect();
            assert_eq!(vec![1, 1, 2, 0, 2], weights);
        }
        assert_eq!(3, rng.next);
    }

    /// Marsaglia's example xorshift32 seed; any non-zero seed would do.
    const XORSHIFT_SEED: u32 = 2_463_534_242;

    /// Checks that `rng`, seeded with [`XORSHIFT_SEED`], has drawn exactly
    /// `calls` words.
    fn assert_words_drawn(rng: &mut Xorshift32, calls: usize) {
        let mut fresh = Xorshift32::new(XORSHIFT_SEED);
        for _ in 0..calls {
            fresh.next_u32();
        }
        assert_eq!(fresh.next_u32(), rng.next_u32(), "expected {calls} calls");
    }

    fn close(got: f64, want: f64, tolerance: f64) -> bool {
        (got - want).abs() <= tolerance * want.abs().max(1.0)
    }

    /// Every packing path draws the number of words the module documentation
    /// gives, and the statistics are pinned on a fixed stream.
    #[test]
    fn hamming_corr_packing_and_pins() {
        // (n, r, s, L, z, generator calls)
        #[rustfmt::skip]
        let cases = [
            // s | L, the upstream_tests packing: 30 fields per block.
            (2000, 20, 10, 300, 0.829_937_801_243_275, 60_000),
            // L > s, s ∤ L: two fields and five leading bits of a third word,
            // or one field and two leading bits of a second.
            (1500, 3, 10, 25, -0.296_511_178_967_733_4, 4500),
            (1200, 4, 5, 7, -0.144_397_745_564_476_96, 2400),
            // L = s: one field per block.
            (700, 5, 16, 16, 1.172_527_685_431_962_4, 700),
            // L < s: four blocks per field, the last field partly used
            // (n mod 4 = 1, 3), with two unused top bits (s = 30) and none.
            (1001, 2, 30, 7, 0.930_613_139_992_408_8, 251),
            (999, 0, 32, 8, 0.838_842_842_628_616_9, 250),
            // Ten blocks from a single word.
            (10, 0, 31, 3, -0.777_777_777_777_777_8, 1),
        ];
        for (n, r, s, l, z, calls) in cases {
            let label = format!("n = {n}, r = {r}, s = {s}, L = {l}");
            let mut rng = Xorshift32::new(XORSHIFT_SEED);
            let summary = hamming_corr(&mut rng, n, r, s, l);
            assert!(
                close(summary.z_score, z, 1e-12),
                "{label}: z = {}",
                summary.z_score
            );
            let two_sided = crate::math::erfc(z.abs() / std::f64::consts::SQRT_2);
            assert!((summary.p_value - two_sided).abs() < CLOSED_FORM, "{label}");
            assert_words_drawn(&mut rng, calls);
        }
    }

    /// Words drawn by each packing path, the single-class split (n = 20,
    /// L = 7), and the corner statistics by hand on a small table.
    #[test]
    fn hamming_indep_packing_and_split() {
        // (n, r, s, L, d, generator calls)
        let cases = [
            (1000, 20, 10, 30, 2, 6000),
            (1000, 2, 5, 7, 3, 4000),
            (500, 6, 12, 13, 4, 2000),
            (999, 1, 31, 4, 2, 286),
            (300, 0, 32, 5, 3, 100),
            (20, 0, 32, 7, 1, 10),
        ];
        for (n, r, s, l, d, calls) in cases {
            let label = format!("n = {n}, r = {r}, s = {s}, L = {l}, d = {d}");
            let mut rng = Xorshift32::new(XORSHIFT_SEED);
            let summary = hamming_indep(&mut rng, n, r, s, l, d);
            assert!(summary.main_dof >= 1, "{label}");
            assert!(summary.main_p_value.is_finite(), "{label}");
            assert_eq!(d, summary.block_chi_square.len(), "{label}");
            for (k, &dof) in summary.block_dof.iter().enumerate() {
                let want = if l % 2 == 1 && k == 0 { 1 } else { 2 };
                assert_eq!(want, dof, "{label}: block {}", k + 1);
            }
            assert_words_drawn(&mut rng, calls);
        }
        // n = 20 pairs of 7-bit weights: no cell expects 10, so the main
        // statistic is the one-degree-of-freedom column split.
        let summary = hamming_indep(&mut Xorshift32::new(XORSHIFT_SEED), 20, 0, 32, 7, 1);
        assert_eq!(1, summary.main_dof);
    }

    /// With L = 2 the weights are 0, 1, 2 with probabilities ¼, ½, ¼.  For
    /// k = 1 the low corner is w = 0 and the high corner w = 2, so each corner
    /// cell expects 2n/16.
    #[test]
    fn corner_statistic_by_hand() {
        // Blocks of two bits from 2-bit fields: pairs (0, 0), (2, 2), (0, 2),
        // (1, 1) repeated five times.
        let fields: Vec<u32> = [0b00, 0b00, 0b11, 0b11, 0b00, 0b11, 0b01, 0b10].repeat(5);
        let mut rng = fields_as_words(&fields, 2);
        let summary = hamming_indep(&mut rng, 20, 0, 2, 2, 1);
        // same side 10, opposite 5, middle 5; expected 2.5, 2.5, 15.
        let want = 7.5f64.powi(2) / 2.5 + 2.5f64.powi(2) / 2.5 + 10.0f64.powi(2) / 15.0;
        assert!((summary.block_chi_square[0] - want).abs() < CLOSED_FORM);
        assert_eq!(2, summary.block_dof[0]);
    }

    /// 2^−L underflows to 0 for L ≥ 1075, so the probabilities are accumulated
    /// in logarithms: they must still sum to 1 and stay symmetric up to the
    /// cap, and agree with the direct recurrence where that is exact.
    #[test]
    fn binomial_probs_survive_large_l() {
        for l in [1075usize, super::HAMMING_INDEP_MAX_L] {
            let probs = binomial_probs(l);
            let sum: f64 = probs.iter().sum();
            assert!(
                (sum - 1.0).abs() < SUMMED_PROBABILITY,
                "L = {l}: sum = {sum}"
            );
            let k = l / 2 - 10;
            assert!(probs[k] > 0.0, "L = {l}: central mass underflowed");
            assert!(
                (probs[k] - probs[l - k]).abs() <= SUMMED_PROBABILITY * probs[k],
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
            assert!(
                (x - y).abs() <= SUMMED_PROBABILITY * y,
                "L = 300, k = {k}: {x} vs {y}"
            );
        }
    }

    #[test]
    fn binomial_probs_sum_to_one() {
        let probs = binomial_probs(12);
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < CLOSED_FORM);
    }

    /// A zero correlation gives p = erfc(0) = 1 exactly.
    #[test]
    fn hamming_corr_p_value_is_at_most_one() {
        // r = 0, s = 4, L = 4: the first block weighs 2 = L/2, so the one
        // product in the correlation sum is 0 and z = 0.
        let mut rng = fields_as_words(&[0b1100, 0b0111], 4);
        let summary = hamming_corr(&mut rng, 2, 0, 4, 4);
        assert_eq!(0.0, summary.z_score);
        assert_eq!(1.0, summary.p_value);
    }

    #[test]
    fn hamming_corr_rejects_constant_stream() {
        let mut rng = ConstantRng::new(0);
        let summary = hamming_corr(&mut rng, 128, 0, 8, 16);
        assert!(summary.p_value < 1e-6);
    }

    /// Lumping at a threshold of 10, derived by hand.
    #[test]
    fn lumped_chi_square_pools_weak_cells() {
        use super::lumped_chi_square;
        // Weak cells 5 and 3 pool to 8 < 10 and join the last kept cell
        // (12 → 20, observed 14 → 22): χ² = 2²/20 + 2²/20 = 0.4, dof 1.
        let (chi, dof, lumped) = lumped_chi_square(&[20.0, 5.0, 3.0, 12.0], &[18, 7, 1, 14], 10.0);
        assert!((chi - 0.4).abs() < CLOSED_FORM, "{chi}");
        assert_eq!((1, 2), (dof, lumped));
        // Weak cells 6 and 4 pool to 10 and form a class of their own:
        // χ² = 5²/20 + 3²/12 + 4²/10 = 3.6 over three classes, dof 2.
        let (chi, dof, lumped) = lumped_chi_square(&[20.0, 6.0, 4.0, 12.0], &[25, 4, 2, 9], 10.0);
        assert!((chi - 3.6).abs() < CLOSED_FORM, "{chi}");
        assert_eq!((2, 2), (dof, lumped));
        // No cell reaches 10: everything pools into one class, dof 0.
        let (chi, dof, lumped) = lumped_chi_square(&[4.0, 3.0], &[5, 2], 10.0);
        assert!(chi.abs() < CLOSED_FORM, "{chi}");
        assert_eq!((0, 2), (dof, lumped));
    }
}
