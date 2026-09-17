//! The 6×8 binary rank test at each of the 25 byte offsets of a word: 25
//! window results and one Anderson–Darling summary.  Result names:
//! `diehard_historical::rank_6x8_windows`, once per window, then
//! `diehard_historical::rank_6x8_windows_summary`.
//!
//! # The statistic
//!
//! Window b, for b = 1 to 25, takes from each word the byte at bits b to
//! b + 7 counting from the most significant bit, `(w >> (25 − b)) & 255`.
//! Six such bytes from six successive words are the rows of a 6 × 8 matrix
//! over GF(2), whose rank is 6 with probability 0.7731, 5 with probability
//! 0.2174 and at most 4 with probability 0.0094 (exact values from the count
//! of matrices of each rank; see [`crate::diehard::binary_rank`]).  100 000
//! matrices give a Pearson χ² on those three cells, df 2, and the p-value
//! exp(−χ²/2).
//!
//! Window b reads its own block of 600 000 words, the b-th in the input
//! (15 000 000 words in all), so under the null the 25 p-values are
//! independent.  A 26th result, the summary, is the Anderson–Darling
//! statistic A² of the 25 p-values against U(0, 1), reported as
//! 1 − CDF(A²).
//!
//! # Reading the results
//!
//! The summary has little power against one broken window.  A² gives the
//! smallest p-value little weight: with 24 uniform p-values and one 0 it
//! falls below 0.01 in under 10% of cases.  The broken window's own result
//! fails decisively, so read the 25 window results first; the summary asks
//! only whether they are jointly uniform.
//!
//! # Calibration
//!
//! 10 000 streams of 15 000 000 words, each from a separately seeded PCG64
//! generator, gave 250 000 window p-values with p < 0.01 in 0.99% (binomial
//! standard deviation 0.02%) and p < 0.001 in 0.10% (0.006%), and a
//! Kolmogorov–Smirnov p of 0.80.  The summary fell below 0.01 in 1.05%
//! (0.10%) and below 0.001 in 0.09% (0.03%), with a Kolmogorov–Smirnov p of
//! 0.13.  A test below runs a fixed
//! 20-stream version under `cargo test --release`.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use super::anderson_darling_statistic;
use crate::{
    diehard::binary_rank::{
        rank_6x8_chi_square, rank_6x8_counts, RANK_6X8_MATRICES, RANK_6X8_ROWS,
    },
    math::{anderson_darling_cdf, igamc},
    result::TestResult,
};

/// Name of each window's result.
const NAME: &str = "diehard_historical::rank_6x8_windows";
/// Name of the Anderson–Darling summary.
const SUMMARY_NAME: &str = "diehard_historical::rank_6x8_windows_summary";
/// Byte windows: bits b to b + 7 from the most significant, b = 1 to 25.
pub const WINDOWS: usize = 25;
/// Results the test reports: one per window, then the summary.
pub const RESULTS: usize = WINDOWS + 1;
/// Words each window reads: six rows for each of 100 000 matrices.
pub const WORDS_PER_WINDOW: usize = RANK_6X8_ROWS * RANK_6X8_MATRICES;
/// Words the test reads.
pub const WORDS: usize = WINDOWS * WORDS_PER_WINDOW;

/// The 6×8 rank test on each of the 25 byte windows, window b reading the
/// b-th block of [`WORDS_PER_WINDOW`] words: [`RESULTS`] results, the 25
/// windows in order and then their Anderson–Darling summary.
///
/// Reports [`RESULTS`] SKIPs for fewer than [`WORDS`] words.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn rank_6x8_windows(words: &[u32]) -> Vec<TestResult> {
    if words.len() < WORDS {
        let reason = "need 15 000 000 words";
        let mut skipped: Vec<TestResult> = (0..WINDOWS)
            .map(|_| TestResult::insufficient(NAME, reason))
            .collect();
        skipped.push(TestResult::insufficient(SUMMARY_NAME, reason));
        return skipped;
    }
    let mut p_values = Vec::with_capacity(WINDOWS);
    let mut results: Vec<TestResult> = words
        .chunks_exact(WORDS_PER_WINDOW)
        .zip(1..=WINDOWS)
        .map(|(block, b)| {
            let counts = window_counts(block, WINDOWS - b);
            let chi_square = rank_6x8_chi_square(&counts);
            let p_value = igamc(1.0, chi_square / 2.0);
            p_values.push(p_value);
            TestResult::with_note(
                NAME,
                p_value,
                format!(
                    "bits {b} to {}, ranks ≤4/5/6: {}/{}/{}, χ²={chi_square:.4}",
                    b + 7,
                    counts[0],
                    counts[1],
                    counts[2]
                ),
            )
            .chi_square(chi_square, 2.0)
        })
        .collect();
    let a2 = anderson_darling_statistic(&mut p_values);
    results.push(
        TestResult::with_note(
            SUMMARY_NAME,
            1.0 - anderson_darling_cdf(WINDOWS, a2),
            format!("A²={a2:.4} over the 25 window p-values"),
        )
        .anderson_darling(a2, WINDOWS),
    );
    results
}

/// Rank counts (≤ 4, 5, 6) of the matrices whose rows are the bytes
/// `w >> shift` of `block`.
fn window_counts(block: &[u32], shift: usize) -> [usize; 3] {
    rank_6x8_counts(block.iter().map(|&w| (w >> shift) & 0xFF))
}

#[cfg(test)]
mod tests {
    use super::{rank_6x8_windows, NAME, RESULTS, SUMMARY_NAME, WINDOWS, WORDS, WORDS_PER_WINDOW};
    use crate::{
        math::ks_test,
        rng::{Mt19937, Pcg64, Rng},
    };

    /// Sum of the 25 window p-values on a fixed PCG64 stream, pinned.
    const GOLDEN_WINDOW_P_SUM: f64 = 14.214_903_027_983_372;
    /// The summary's p-value on the same stream, pinned.
    const GOLDEN_SUMMARY_P: f64 = 0.279_674_977_667_713_1;

    /// The 26 results on a fixed stream, in order, pinned to
    /// 10⁻¹².
    #[test]
    fn results_on_a_fixed_stream_are_pinned() {
        let words = Pcg64::new(20_260_916, 68).collect_u32s(WORDS);
        let results = rank_6x8_windows(&words);
        assert_eq!(results.len(), RESULTS);
        for (b, r) in (1..=WINDOWS).zip(&results) {
            assert_eq!(r.name, NAME);
            let head = format!("bits {b} to {},", b + 7);
            assert!(r.note.as_deref().unwrap_or("").starts_with(&head), "{r}");
        }
        let sum: f64 = results[..WINDOWS].iter().map(|r| r.p_value).sum();
        assert!((sum - GOLDEN_WINDOW_P_SUM).abs() < 1e-12, "sum {sum:?}");
        let summary = &results[WINDOWS];
        assert_eq!(summary.name, SUMMARY_NAME);
        assert!(
            (summary.p_value - GOLDEN_SUMMARY_P).abs() < 1e-12,
            "{:?}",
            summary.p_value
        );
    }

    /// Zeroing window 8's byte (bits 8 to 15) in window 8's block gives every
    /// matrix there rank 0: window 8's result fails and no other window's
    /// moves.  The summary is not asserted; see the module documentation.
    #[test]
    fn each_window_reads_its_own_bits_and_block() {
        let mut words = Mt19937::new(5489).collect_u32s(WORDS);
        let b = 8;
        let byte = u32::from(u8::MAX) << (WINDOWS - b);
        for w in &mut words[(b - 1) * WORDS_PER_WINDOW..b * WORDS_PER_WINDOW] {
            *w &= !byte;
        }
        let results = rank_6x8_windows(&words);
        for (i, r) in (1..=WINDOWS).zip(&results) {
            if i == b {
                assert!(r.p_value < 1e-10, "{r}");
            } else {
                assert!(r.p_value > 1e-6, "{r}");
            }
        }
        assert_eq!(results[WINDOWS].name, SUMMARY_NAME);
    }

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        for words in [vec![], vec![0; WORDS - 1]] {
            let results = rank_6x8_windows(&words);
            assert_eq!(results.len(), RESULTS);
            assert!(results.iter().all(|r| r.skipped()));
        }
        let results = rank_6x8_windows(&vec![0; WORDS]);
        assert_eq!(results.len(), RESULTS);
        assert!(results.iter().all(|r| !r.skipped() && !r.passed()));
    }

    /// Streams from separately seeded PCG64 generators.
    const SMOKE_STREAMS: u64 = 20;

    /// Over 20 PCG64 streams the 500 window p-values pass a KS test with at
    /// most 12 below 0.01 (Binomial(500, 0.01) exceeds 12 with probability
    /// 0.002), and at most two summaries fall below 0.01 (Binomial(20, 0.01)
    /// exceeds 2 with probability 0.001).
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "twenty 15 million-word streams; runs under cargo test --release"
    )]
    fn null_streams_give_uniform_p_values() {
        let mut window_p = Vec::new();
        let mut summaries_below = 0;
        for i in 0..SMOKE_STREAMS {
            let words = Pcg64::new(u128::from(i), u128::from(SMOKE_STREAMS)).collect_u32s(WORDS);
            let results = rank_6x8_windows(&words);
            window_p.extend(results[..WINDOWS].iter().map(|r| r.p_value));
            summaries_below += usize::from(results[WINDOWS].p_value < 0.01);
        }
        let windows_below = window_p.iter().filter(|&&p| p < 0.01).count();
        assert!(
            windows_below <= 12,
            "{windows_below} of 500 windows below 0.01"
        );
        assert!(
            summaries_below <= 2,
            "{summaries_below} of {SMOKE_STREAMS} summaries below 0.01"
        );
        let ks = ks_test(&mut window_p);
        assert!(ks > 1e-3, "KS p = {ks}");
    }
}
