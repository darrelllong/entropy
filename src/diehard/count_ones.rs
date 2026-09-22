//! DIEHARD count-the-1s test on a stream of bytes.
//!
//! Each byte becomes a letter by its Hamming weight: 0, 1 or 2 is A, 3 is B,
//! 4 is C, 5 is D, and 6, 7 or 8 is E.  A uniform byte's weight is
//! Binomial(8, ½), so the letters have probabilities 37, 56, 70, 56 and 37 in
//! 256.
//!
//! Over n = 256 000 overlapping five-letter words, Q5 is the Pearson sum of
//! the 5⁵ = 3 125 word counts against their expected counts n·Πp, and Q4 the
//! same sum over the 5⁴ = 625 counts of each word's leading four letters.
//! Overlapping words make neither sum χ²-distributed, but their difference
//! is asymptotically χ² with 5⁵ − 5⁴ = 2 500 degrees of freedom (Marsaglia
//! 1985).  The p-value is two-sided in that distribution,
//! 2·min(P(χ² ≤ Q5 − Q4), P(χ² ≥ Q5 − Q4)): a difference that is too small
//! fails as well as one that is too large.
//!
//! Bytes are taken from each word low byte first.  256 004 bytes are read: a
//! four-letter prefix, then one letter per counted word.
//!
//! 20 000 separately seeded PCG64 streams gave p < 0.01 in 0.990% (binomial
//! standard deviation 0.070%) and p < 0.001 in 0.085%; a Kolmogorov–Smirnov
//! test of the p-values gave 0.46.
//!
//! [`crate::diehard::historical::count_ones_bytes`] applies the same
//! statistic to one byte of each word, at each of the 25 bit offsets.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995), and
//! "A Current View of Random Number Generators", *Computer Science and
//! Statistics: 16th Symposium on the Interface* (1985), for the overlapping
//! χ² difference.

use crate::{math::igamc, result::TestResult};

const WORD_LEN: usize = 5;
const ALPHA_SIZE: usize = 5;
const N_CATEGORIES5: usize = 3125; // 5^5
const N_CATEGORIES4: usize = 625; // 5^4
const N_SAMPLES: usize = 256_000;

/// Letters one Q5 − Q4 statistic reads: a four-letter prefix, then one letter
/// for each of the `N_SAMPLES` overlapping five-letter words it counts.
pub(crate) const LETTERS_PER_TEST: usize = N_SAMPLES + WORD_LEN - 1;

/// Letter probabilities P(A) … P(E).  A uniform byte's Hamming weight is
/// Binomial(8, ½), so the weight groups {0, 1, 2}, 3, 4, 5 and {6, 7, 8} have
/// 37, 56, 70, 56 and 37 chances in 256.
const LETTER_PROBS: [f64; ALPHA_SIZE] = [
    37.0 / 256.0,
    56.0 / 256.0,
    70.0 / 256.0,
    56.0 / 256.0,
    37.0 / 256.0,
];

/// Degrees of freedom of Q5 − Q4: 5⁵ − 5⁴.
const QDIFF_DF: f64 = (N_CATEGORIES5 - N_CATEGORIES4) as f64;

/// The variance of Q5 − Q4 under the null, as a multiple of the 2·df of the
/// χ² it approximates.
///
/// Measured (`examples/statistic_scale.rs`, `stats/statistic-scale-moore.txt`)
/// over 5 000 000 byte windows of 256 004 letters from PCG64, xoshiro256**
/// and SFC64 null streams: the mean is 2 500.00 and the standard deviation
/// 71.07 against √5000 = 70.71, the same in every one of the 25 windows, so
/// the ratio is 1.0103 with a standard error of 0.0006.  The excess is the
/// finite-sample variance of Pearson's statistic where cells are sparse: the
/// rarest of the 3 125 five-letter cells expects only 256 000·(37/256)⁵ ≈ 16
/// counts.  With the χ² law as it stood the test rejected 1.04% of null
/// windows at the 1% level; with this ratio, on 5 000 000 further windows
/// the calibration never saw (`stats/statistic-validate.txt`), it rejects
/// 4.999%, 0.995% and 0.098% at 0.05, 0.01 and 0.001.
const QDIFF_VARIANCE_RATIO: f64 = 1.0103;

/// Count-the-1s test on a stream of all bytes, by the Q5 − Q4 statistic in
/// the module documentation.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn count_ones_stream(words: &[u32]) -> TestResult {
    let bytes_needed = LETTERS_PER_TEST;
    let words_needed = bytes_needed.div_ceil(4);
    if words.len() < words_needed {
        return TestResult::insufficient("diehard::count_ones_stream", "not enough words");
    }

    // Feed bytes → letters directly into the rolling-window accumulator
    // without materialising a Vec<u8> or Vec<usize>.
    let letter_iter = words
        .iter()
        .flat_map(|&w| w.to_le_bytes())
        .take(bytes_needed)
        .map(hamming_letter);
    count_ones_test(letter_iter, "diehard::count_ones_stream")
}

/// The letter, 0 (A) to 4 (E), of a byte by its Hamming weight.
pub(crate) fn hamming_letter(b: u8) -> usize {
    match b.count_ones() {
        0..=2 => 0, // A
        3 => 1,     // B
        4 => 2,     // C
        5 => 3,     // D
        _ => 4,     // E  (6, 7, or 8)
    }
}

/// Core Q5−Q4 test, driven by an iterator of letter indices (0..ALPHA_SIZE).
///
/// Accepts any `Iterator<Item = usize>` so the caller can stream bytes→letters
/// directly without materialising intermediate Vecs.
fn count_ones_test(letters: impl Iterator<Item = usize>, name: &'static str) -> TestResult {
    let (q5, q4) = q5_q4(letters);

    TestResult::with_note(
        name,
        q_difference_p_value(q5, q4),
        format!(
            "n={N_SAMPLES}, Q5={q5:.2}, Q4={q4:.2}, Q5-Q4={:.2}",
            q5 - q4
        ),
    )
    .with_statistic("Q5 - Q4", q5 - q4, Some(QDIFF_DF), QDIFF_NULL)
}

/// The Pearson sums (Q5, Q4) over `N_SAMPLES` overlapping
/// five-letter words and their leading four letters, read from the first
/// [`LETTERS_PER_TEST`] letters of `letters` (0..ALPHA_SIZE each).
pub(crate) fn q5_q4(mut letters: impl Iterator<Item = usize>) -> (f64, f64) {
    let n = N_SAMPLES;

    let nf = n as f64;

    let mut counts5 = [0u32; N_CATEGORIES5];
    let mut counts4 = [0u32; N_CATEGORIES4];

    // Encode the initial (WORD_LEN − 1)-letter prefix.
    let mut word5 = 0usize;
    for l in letters.by_ref().take(WORD_LEN - 1) {
        word5 = word5 * ALPHA_SIZE + l;
    }

    // Slide N complete 5-letter windows directly from the iterator.
    for l in letters.take(n) {
        word5 = (word5 * ALPHA_SIZE + l) % N_CATEGORIES5;
        counts5[word5] += 1;
        // The leading 4-letter prefix of this window is word5 / ALPHA_SIZE.
        counts4[word5 / ALPHA_SIZE] += 1;
    }

    (pearson_sum(&counts5, nf), pearson_sum(&counts4, nf))
}

/// Σ (c − e)²/e over the counts of words of 5ᵏ possible values, word w's k
/// letters being its base-5 digits and e = n·Π P(letter).  Every expected
/// count is at least n·P(A)⁵ ≈ 16.
fn pearson_sum(counts: &[u32], n: f64) -> f64 {
    counts
        .iter()
        .enumerate()
        .map(|(mut word, &c)| {
            let mut expected = n;
            for _ in 0..counts.len().ilog(ALPHA_SIZE) {
                expected *= LETTER_PROBS[word % ALPHA_SIZE];
                word /= ALPHA_SIZE;
            }
            (f64::from(c) - expected).powi(2) / expected
        })
        .sum()
}

/// The two-sided p-value of Q5 − Q4: min(1, 2·min(P(X ≤ Q5 − Q4),
/// P(X ≥ Q5 − Q4))) for X the gamma law with the statistic's mean, 2 500,
/// and its measured variance, 2·2 500·[`QDIFF_VARIANCE_RATIO`]: shape
/// df/(2ρ) and scale 2ρ, which is χ²(2 500) itself at ρ = 1.
pub(crate) fn q_difference_p_value(q5: f64, q4: f64) -> f64 {
    q_difference_p_value_of(q5 - q4)
}

/// The name the count-ones results record for their null law; see
/// [`q_difference_p_value_of`].
pub const QDIFF_NULL: &str = "gamma of measured variance, two-sided";

/// The two-sided p-value of a Q5 − Q4 value in its null law, for a reader
/// that recomputes p-values from recorded statistics.
#[must_use]
pub fn q_difference_p_value_of(q_difference: f64) -> f64 {
    let scale = 2.0 * QDIFF_VARIANCE_RATIO;
    let upper = igamc(QDIFF_DF / scale, q_difference.max(0.0) / scale);
    (2.0 * upper.min(1.0 - upper)).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::{
        count_ones_stream, hamming_letter, q_difference_p_value, ALPHA_SIZE, LETTER_PROBS,
        N_SAMPLES, WORD_LEN,
    };

    /// At the law's median the two-sided p-value is 1, it falls symmetrically
    /// in probability toward either tail, and the law is wider than χ²(2 500)
    /// by the measured ratio: at 2 700 the corrected p-value exceeds the χ²
    /// one.
    #[test]
    fn q_difference_p_value_is_two_sided_and_widened() {
        assert!(q_difference_p_value(2_499.33, 0.0) > 0.99);
        let low = q_difference_p_value(2_300.0, 0.0);
        let high = q_difference_p_value(2_700.0, 0.0);
        assert!(low < 0.01 && high < 0.01, "{low} {high}");
        let chi_square = 2.0 * crate::math::igamc(super::QDIFF_DF / 2.0, 2_700.0 / 2.0);
        assert!(high > chi_square, "{high} against χ² {chi_square}");
        // One standard deviation out, the corrected law is the χ² law
        // stretched by √1.0103: the p-values agree there to a few percent.
        let ratio = super::QDIFF_VARIANCE_RATIO.sqrt();
        let stretched = q_difference_p_value(2_500.0 + 70.71 * ratio, 0.0);
        let plain = 2.0 * crate::math::igamc(super::QDIFF_DF / 2.0, (2_500.0 + 70.71) / 2.0);
        assert!(
            (stretched - plain).abs() < 0.02 * plain,
            "{stretched} vs {plain}"
        );
    }

    /// Counting the 256 bytes by letter must reproduce the table, and the
    /// dyadic entries sum to exactly 1.
    #[test]
    fn letter_probabilities_match_byte_weights_and_sum_to_one() {
        let mut per_letter = [0u32; ALPHA_SIZE];
        for byte in 0..=u8::MAX {
            per_letter[hamming_letter(byte)] += 1;
        }
        assert_eq!(per_letter, [37, 56, 70, 56, 37]);
        for (p, n) in LETTER_PROBS.iter().zip(per_letter) {
            assert_eq!(*p, f64::from(n) / 256.0);
        }
        assert_eq!(LETTER_PROBS.iter().sum::<f64>(), 1.0);
    }

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        let needed = (N_SAMPLES + WORD_LEN - 1).div_ceil(4);
        assert!(count_ones_stream(&[]).skipped());
        assert!(count_ones_stream(&vec![0; needed - 1]).skipped());
        let r = count_ones_stream(&vec![0; needed]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
