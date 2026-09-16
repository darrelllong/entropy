//! DIEHARDER monobit2 test: bit counts over blocks of increasing length.
//!
//! Level j splits the words into complete, non-overlapping blocks of 2^(j+1)
//! words.  The number of one bits in a block of b = 32·2^(j+1) bits is
//! Binomial(b, ½) under the null, so the histogram of block counts is scored
//! with a Pearson χ² against its binomial expectation, the tail cells pooled
//! until each expects at least 50 blocks.  The pooling matters in the far
//! upper tail, which the combination below reaches: over 40 000 null trials of
//! 100 000 words, pooling only to 5 put 1.6 times the nominal rate of level
//! p-values below 10⁻³, and 20 put 1.2 times.  Level j is used only while its most
//! likely cell expects at least 20 blocks, at most 16 levels.
//!
//! Each level's p-value p is folded two-sided, 2·min(p, 1 − p), so that a fit
//! too good to be true fails as well as a bad one.  The levels read the same
//! words and are dependent, so they are combined by Bonferroni's bound: the
//! result is min(1, L·min fold) over the L levels, valid whatever their
//! dependence.  The bound makes the result conservative and its null
//! distribution not uniform: the rate below 0.01 was 0.97% over 40 000 null
//! trials of 100 000 words, 1.00% over 10 000 of 1 000 000 words and 0.67%
//! over 300 of 16 000 000 words.
//!
//! # Author
//! David Bauer, in Robert G. Brown's *Dieharder: A Random Number Test Suite*
//! (2006).

use crate::{
    math::{binomial_pmf, chi_square_pooled_tails, igamc, lgamma},
    result::TestResult,
};

/// Most levels considered: blocks of 2 to 2¹⁶ words.
const MAX_LEVELS: usize = 16;
/// Bits in a word.
const WORD_BITS: usize = 32;
/// Smallest expected count of the most likely cell for a level to be used.
const MIN_CENTRE_EXPECTED: f64 = 20.0;
/// Smallest expected count of a cell after pooling.
const MIN_CELL_EXPECTED: f64 = 50.0;

/// Run the monobit2 test.
///
/// # Author
/// David Bauer, Dieharder (2006).
pub fn monobit2(words: &[u32]) -> TestResult {
    let levels = level_count(words.len());
    if levels == 0 {
        return TestResult::insufficient(
            "dieharder::monobit2",
            "not enough words for any block length",
        );
    }

    // Ones in each block of the current level, starting with pairs of words.
    let mut block_ones: Vec<u64> = words
        .chunks_exact(2)
        .map(|pair| u64::from(pair[0].count_ones() + pair[1].count_ones()))
        .collect();
    let mut folds = Vec::with_capacity(levels);
    for j in 0..levels {
        if j > 0 {
            block_ones = block_ones
                .chunks_exact(2)
                .map(|pair| pair[0] + pair[1])
                .collect();
        }
        let bits = WORD_BITS * (2 << j);
        if let Some(p) = level_p_value(&block_ones, bits) {
            folds.push(2.0 * p.min(1.0 - p));
        }
    }
    if folds.is_empty() {
        return TestResult::insufficient("dieharder::monobit2", "no level had two cells");
    }
    let smallest = folds.iter().copied().fold(f64::INFINITY, f64::min);
    let p_value = (folds.len() as f64 * smallest).min(1.0);
    TestResult::with_note(
        "dieharder::monobit2",
        p_value,
        format!(
            "tsamples={}, levels={}, block_sizes=2..{}",
            words.len(),
            folds.len(),
            2usize << (levels - 1)
        ),
    )
}

/// Levels whose most likely cell expects at least [`MIN_CENTRE_EXPECTED`]
/// blocks, for `words` words.
fn level_count(words: usize) -> usize {
    (0..MAX_LEVELS)
        .take_while(|&j| {
            let block = 2usize << j;
            let bits = WORD_BITS * block;
            let blocks = (words / block) as f64;
            let half = bits / 2;
            let ln_centre = lgamma((bits + 1) as f64)
                - 2.0 * lgamma((half + 1) as f64)
                - bits as f64 * std::f64::consts::LN_2;
            blocks * ln_centre.exp() >= MIN_CENTRE_EXPECTED
        })
        .count()
}

/// The χ² p-value of one level's block counts, each a count of ones among
/// `bits` bits, or `None` if pooling leaves fewer than two cells.
fn level_p_value(block_ones: &[u64], bits: usize) -> Option<f64> {
    let mut histogram = vec![0.0f64; bits + 1];
    for &ones in block_ones {
        histogram[ones as usize] += 1.0;
    }
    let blocks = block_ones.len() as f64;
    let expected: Vec<f64> = (0..=bits)
        .map(|k| blocks * binomial_pmf(bits, k, 0.5))
        .collect();
    let (chi, df) = chi_square_pooled_tails(&histogram, &expected, MIN_CELL_EXPECTED)?;
    Some(igamc(df as f64 / 2.0, chi / 2.0))
}

#[cfg(test)]
mod tests {
    use super::{level_count, level_p_value, monobit2};
    use crate::{
        math::ks_test,
        rng::{ConstantRng, Mt19937, Pcg64, Rng},
    };

    #[test]
    fn level_count_grows_with_the_sample() {
        assert_eq!(level_count(0), 0);
        assert!(level_count(2_000) >= 1);
        assert!(level_count(16_000_000) > level_count(2_000));
    }

    /// A constant stream concentrates every block's count in one cell; that
    /// must fail, not skip and not pass.
    #[test]
    fn monobit2_fails_constant_stream() {
        let words = ConstantRng::new(0).collect_u32s(1_000_000);
        let result = monobit2(&words);
        assert!(!result.skipped(), "{result}");
        assert!(result.p_value < 1e-10, "{result}");
    }

    /// Too few words for any level report SKIP.
    #[test]
    fn short_input_skips() {
        assert!(monobit2(&[]).skipped());
        assert!(monobit2(&[1, 2, 3]).skipped());
    }

    /// Blocks whose counts follow the binomial exactly in shape give a large
    /// level p-value.
    #[test]
    fn level_p_value_accepts_a_binomial_sample() {
        let words = Mt19937::new(5489).collect_u32s(200_000);
        let ones: Vec<u64> = words
            .chunks_exact(2)
            .map(|p| u64::from(p[0].count_ones() + p[1].count_ones()))
            .collect();
        let p = level_p_value(&ones, 64).expect("cells");
        assert!(p > 1e-4, "{p}");
    }

    /// Trials from separately seeded PCG64 streams.
    const NULL_TRIALS: u64 = 2_000;

    /// Level p-values are uniform under the null: 2 000 level-0 p-values pass
    /// a KS test, and the combined result falls below 0.01 at no more than its
    /// nominal rate plus three binomial standard deviations.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "2 000 trials of 10 000 words; runs under cargo test --release"
    )]
    fn null_trials_are_calibrated() {
        let mut level0 = Vec::new();
        let mut below = 0;
        for i in 0..NULL_TRIALS {
            let words = Pcg64::new(u128::from(i), 0x6d6f_6e6f).collect_u32s(10_000);
            let ones: Vec<u64> = words
                .chunks_exact(2)
                .map(|p| u64::from(p[0].count_ones() + p[1].count_ones()))
                .collect();
            level0.push(level_p_value(&ones, 64).expect("cells"));
            below += usize::from(monobit2(&words).p_value < 0.01);
        }
        let ks = ks_test(&mut level0);
        assert!(ks > 1e-3, "KS p = {ks}");
        let n = NULL_TRIALS as f64;
        let limit = n * 0.01 + 3.0 * (n * 0.01 * 0.99).sqrt();
        assert!(
            (below as f64) <= limit,
            "{below} of {NULL_TRIALS} below 0.01"
        );
    }
}
