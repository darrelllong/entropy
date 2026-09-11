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
//! * TestU01 1.2.3 source (`testu01-source` in BIB.md):
//!   `testu01/sstring.c` (`sstring_HammingCorr`, `HammingCorr_L`,
//!   `HammingCorr_S`, `sstring_HammingIndep`, `HammingIndep_L`,
//!   `HammingIndep_S`, `CountBlocks`), `testu01/unif01.c`
//!   (`unif01_StripB`), `probdist/gofs.c` (`gofs_Chi2`), `probdist/gofw.c`
//!   (`gofw_ActiveTests0`), and the user's guide `testu01/sstring.tex`.
//!   [pubs/TestU01-2009-57e98bf33880.tar.gz]
//!
//! # Author
//! Pierre L'Ecuyer and Richard Simard (TestU01); Darrell Long (Rust port).
//!
//! This module implements the core single-replication (`N = 1`) statistics
//! for:
//! - `sstring_HammingCorr`
//! - `sstring_HammingIndep`
//!
//! Bit extraction follows `sstring.c`.  Each generator call yields one
//! 32-bit word, and `unif01_StripB(gen, r, t)` keeps its bits
//! `r + 1 ..= r + t`, counted from the most significant end, as a `t`-bit
//! integer.  When `L ≥ s` (`HammingCorr_L`, `HammingIndep_L`), a block adds
//! the weights of ⌊L/s⌋ successive `s`-bit fields and, when `s` does not
//! divide `L`, the weight of `unif01_StripB(gen, r, L mod s)` from one more
//! word: the most significant `L mod s` bits of that word's window, with the
//! rest of the word unused.  When `L < s` (`HammingCorr_S`,
//! `HammingIndep_S`), each `s`-bit field supplies ⌊s/L⌋ blocks, taken from
//! its least significant end, and its `s mod L` most significant bits are
//! unused; a run of blocks that ends part-way through a field has still drawn
//! that field.  A block therefore equals the next `L` bits of the
//! concatenated field stream that the paper describes (§5, p. 16) only when
//! `s` divides `L`, as at the `upstream_tests` defaults (`s = 10`,
//! `L = 300`).
//!
//! The main Hamming-independence chi-square lumps cells as
//! `sstring_HammingIndep` does, with `gofs_MinExpected = 10`: the cells that
//! expect fewer than 10 pairs are pooled, and the pool forms a class of its
//! own if it expects at least 10 or otherwise joins the last kept cell.  If
//! that leaves a single class, the pair table is split instead into columns
//! `j ≤ ⌊L/2⌋` and `j > ⌊L/2⌋`, a chi-square with one degree of freedom.
//! Cell probabilities come from a log-space binomial recurrence rather than
//! TestU01's `fmass_BinomialTerm2`, so statistics agree with TestU01's to
//! rounding.
//!
//! P-values.  For `N = 1`, `gofw_ActiveTests0` reports the right tail
//! `1 − F(x)` of the statistic's distribution and TestU01's reports flag
//! p-values below `gofw_Suspectp = 0.001` or above `1 − gofw_Suspectp`.  For
//! the chi-squares that right tail is what [`hamming_indep`] reports.  For
//! HammingCorr, [`hamming_corr`] reports the two-sided
//! `erfc(|z|/√2) = 2·min(Φ(z), 1 − Φ(z))` instead of TestU01's `1 − Φ(z)`,
//! so that a small value flags either an excess or a deficit of
//! correlation, as this crate's convention requires.

use super::strip_b;
use crate::{
    math::{chi2_pvalue, erfc},
    result::TestResult,
    rng::Rng,
};
use std::f64::consts::{LN_2, SQRT_2};

const GOFS_MIN_EXPECTED: f64 = 10.0;

/// Hamming weights of successive `L`-bit blocks, packed from
/// `unif01_StripB` fields as `sstring.c` packs them (see the module docs).
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
            // `HammingCorr_L` / `HammingIndep_L`: ⌊L/s⌋ whole fields, then the
            // leading `L mod s` bits of one more word's window.
            let mut weight = 0;
            for _ in 0..l / s {
                weight += strip_b(self.rng.next_u32(), r, s).count_ones() as usize;
            }
            if l % s > 0 {
                weight += strip_b(self.rng.next_u32(), r, l % s).count_ones() as usize;
            }
            weight
        } else {
            // `HammingCorr_S` / `HammingIndep_S`: ⌊s/L⌋ blocks per field,
            // least significant first.  `l < s <= 32`, so both shifts fit.
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
        // threshold, that is the only class (dof 0); `hamming_indep` then
        // splits the table in two, as `sstring_HammingIndep` does.
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
    /// Survival p-value of the main chi-square.
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
        lumped_chi_square(&expected, &counts, GOFS_MIN_EXPECTED);
    if main_dof == 0 {
        // `sstring_HammingIndep`: "Everything has been put in a single class;
        // separate all in two classes", columns j <= L/2 against j > L/2.
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

    // The reference values in the next two tests come from TestU01 1.2.3
    // itself: the library built from pubs/TestU01-2009-57e98bf33880.tar.gz,
    // with this Xorshift32 stream supplied through
    // `unif01_CreateExternGenBits`, `N = 1`, and the generator calls counted.

    /// `sstring_HammingCorr` across every packing path.  TestU01 reports
    /// `1 − Φ(z)`; this crate reports `2·min(Φ(z), 1 − Φ(z))`.
    #[test]
    fn hamming_corr_matches_testu01() {
        // (n, r, s, L, statistic z, TestU01 p-value, generator calls)
        #[rustfmt::skip]
        let cases = [
            // s | L, the upstream_tests packing: 30 fields per block.
            (2000, 20, 10, 300, 0.829_937_801_243_275, 0.203_286_975_527_249_06, 60_000),
            // L > s, s ∤ L: two fields and five leading bits of a third word,
            // or one field and two leading bits of a second.
            (1500, 3, 10, 25, -0.296_511_178_967_733_4, 0.616_580_134_484_778_9, 4500),
            (1200, 4, 5, 7, -0.144_397_745_564_476_96, 0.557_406_801_533_543_4, 2400),
            // L = s: one field per block.
            (700, 5, 16, 16, 1.172_527_685_431_962_4, 0.120_492_631_751_374_37, 700),
            // L < s: four blocks per field, the last field partly used
            // (n mod 4 = 1, 3), with two unused top bits (s = 30) and none.
            (1001, 2, 30, 7, 0.930_613_139_992_408_8, 0.176_026_857_485_671_26, 251),
            (999, 0, 32, 8, 0.838_842_842_628_616_9, 0.200_778_752_807_635_02, 250),
            // Ten blocks from a single word.
            (10, 0, 31, 3, -0.777_777_777_777_777_8, 0.781_649_984_638_621_1, 1),
        ];
        for (n, r, s, l, z, testu01_p, calls) in cases {
            let testu01_p: f64 = testu01_p;
            let label = format!("n = {n}, r = {r}, s = {s}, L = {l}");
            let mut rng = Xorshift32::new(XORSHIFT_SEED);
            let summary = hamming_corr(&mut rng, n, r, s, l);
            assert!(
                close(summary.z_score, z, 1e-12),
                "{label}: z = {}",
                summary.z_score
            );
            let two_sided = 2.0 * testu01_p.min(1.0 - testu01_p);
            assert!(
                (summary.p_value - two_sided).abs() < 1e-6,
                "{label}: p = {}",
                summary.p_value
            );
            assert_words_drawn(&mut rng, calls);
        }
    }

    /// `sstring_HammingIndep` across every packing path, including the
    /// single-class split of the main chi-square (n = 20, L = 7).
    #[test]
    fn hamming_indep_matches_testu01() {
        type Case = (
            (usize, usize, usize, usize, usize),
            (f64, usize, f64),
            &'static [(f64, usize, f64)],
            usize,
        );
        // ((n, r, s, L, d), main (χ², dof, p), blocks (χ², dof, p), calls)
        #[rustfmt::skip]
        let cases: [Case; 6] = [
            // s | L: three fields per block.
            (
                (1000, 20, 10, 30, 2),
                (39.092_789_508_824_36, 37, 0.375_970_730_429_785_1),
                &[
                    (0.352_874_353_373_417_6, 2, 0.838_251_439_233_810_5),
                    (0.773_536_440_273_037_6, 2, 0.679_248_512_810_353_3),
                ],
                6000,
            ),
            // L > s, s ∤ L: one field and the two leading bits of a second.
            (
                (1000, 2, 5, 7, 3),
                (12.441_371_382_255_028, 24, 0.974_463_559_923_630_2),
                &[
                    (0.016, 1, 0.899_343_188_561_366_3),
                    (1.779_438_469_673_474, 2, 0.410_771_066_769_133_8),
                    (0.512, 2, 0.774_141_968_792_248_4),
                ],
                4000,
            ),
            // L = s + 1, odd L, d = 4.
            (
                (500, 6, 12, 13, 4),
                (7.642_670_489_877_611, 16, 0.958_799_359_780_123_9),
                &[
                    (0.072, 1, 0.788_446_734_264_471),
                    (0.759_575_820_020_642_7, 2, 0.684_006_464_753_447_7),
                    (7.195_100_398_516_558_5, 2, 0.027_390_742_181_789_83),
                    (3.773_189_579_359_362, 2, 0.151_587_116_987_310_53),
                ],
                2000,
            ),
            // L < s: seven blocks per 31-bit field, 1998 blocks, so the last
            // field supplies three.
            (
                (999, 1, 31, 4, 2),
                (28.758_425_091_758_426, 21, 0.119_923_101_246_930_17),
                &[
                    (0.796_859_423_526_090_2, 2, 0.671_373_468_590_986_6),
                    (2.279_628_835_184_390_3, 2, 0.319_878_380_108_191_36),
                ],
                286,
            ),
            // L < s: six blocks per field, two unused top bits.
            (
                (300, 0, 32, 5, 3),
                (13.700_571_428_571_429, 12, 0.320_236_529_589_327_36),
                &[
                    (0.333_333_333_333_333_3, 1, 0.563_702_861_650_773_5),
                    (2.355_393_939_393_939_4, 2, 0.307_987_226_368_857_2),
                    (0.878_640_522_875_817, 2, 0.644_474_346_294_407_9),
                ],
                100,
            ),
            // Every weight-pair cell expects under 10, so lumping leaves one
            // class and TestU01 splits the table into columns j <= 3 and j > 3.
            (
                (20, 0, 32, 7, 1),
                (3.2, 1, 0.073_638_270_120_393_15),
                &[(0.2, 1, 0.654_720_846_018_577_2)],
                10,
            ),
        ];
        for ((n, r, s, l, d), (chi, dof, p), blocks, calls) in cases {
            let label = format!("n = {n}, r = {r}, s = {s}, L = {l}, d = {d}");
            let mut rng = Xorshift32::new(XORSHIFT_SEED);
            let summary = hamming_indep(&mut rng, n, r, s, l, d);
            assert!(
                close(summary.main_chi_square, chi, 1e-10),
                "{label}: main χ² = {}",
                summary.main_chi_square
            );
            assert_eq!(dof, summary.main_dof, "{label}: main dof");
            assert!(
                (summary.main_p_value - p).abs() < 1e-9,
                "{label}: main p = {}",
                summary.main_p_value
            );
            assert_eq!(blocks.len(), summary.block_chi_square.len(), "{label}");
            for (k, &(chi, dof, p)) in blocks.iter().enumerate() {
                assert!(
                    close(summary.block_chi_square[k], chi, 1e-10),
                    "{label}: block {} χ² = {}",
                    k + 1,
                    summary.block_chi_square[k]
                );
                assert_eq!(dof, summary.block_dof[k], "{label}: block {} dof", k + 1);
                assert!(
                    (summary.block_p_value[k] - p).abs() < 1e-9,
                    "{label}: block {} p = {}",
                    k + 1,
                    summary.block_p_value[k]
                );
            }
            assert_words_drawn(&mut rng, calls);
        }
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
}
