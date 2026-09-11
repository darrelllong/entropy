//! DIEHARD Tests 8 & 9 — Count-the-1's Tests.
//!
//! Each byte is mapped to a letter {A,B,C,D,E} based on its Hamming weight:
//!   0,1,2 → A;  3 → B;  4 → C;  5 → D;  6,7,8 → E.
//!
//! The reference statistic is the **Q5 − Q4 difference**
//! (Marsaglia, DIEHARD 1995; `diehard_count_1s_stream.c`):
//!
//! 1. Collect N = 256 000 overlapping 5-letter words → chi-square Q5 over
//!    3125 = 5⁵ categories using letter-probability-weighted expected counts.
//! 2. Collect the same N overlapping 4-letter words (leading 4 letters of each
//!    5-letter window) → chi-square Q4 over 625 = 5⁴ categories.
//! 3. The test statistic  Z = (Q5 − Q4 − 2500) / √5000  is approximately
//!    standard normal under H₀.  Mean 2500 and σ √5000 are Marsaglia's
//!    (`sknt1s`, `fortran/diehard.f` line 808), which Dieharder's
//!    `diehard_count_1s_stream.c` keeps.
//!
//! Three differences from DIEHARD, all shared with Dieharder.  Marsaglia's
//! `sknt1s` (`fortran/diehard.f` lines 740–826) scores 2 560 000 overlapping
//! five-letter words (`n=100` at line 767, 25 600 × n at lines 779–780) and
//! runs the test twice on successive bytes (`do 888 jk=1,2`, line 772); this
//! module scores 256 000 words once, the size `tests.txt` gives.  `jtbl8`
//! hands out each word's bytes high byte first (lines 214–215); this module
//! takes them low byte first, as Dieharder does.  And DIEHARD reports
//! `phi(z)` (line 817), a CDF value, where this module reports the two-sided
//! erfc(|z|/√2).
//!
//! This crate keeps only the stream variant.  Dieharder rates its byte
//! variant, `diehard_count_1s_byte`, "Good" (`list_tests.c` lines 31–36).  Its
//! author calls that test "LESS stringent than the stream version overall"
//! but says it "might reveal problems with specific offsets ignored by the
//! stream test", and that he "could fix the stream test to cycle through the
//! possible bitlevel offsets and make this test completely obsolete"
//! (`diehard_count_1s_byte.c` lines 60–71).  This crate's own byte variant,
//! since removed, scored Q5 alone with df 3 124, which overlapping words do
//! not support, and was miscalibrated.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).
//! Source: Marsaglia's `fortran/diehard.f`, subroutine `sknt1s`.
//! [pubs/diehard-fortran-1996.tar.gz]

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

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
/// 37, 56, 70, 56 and 37 chances in 256 (Marsaglia, `tests.txt`; `ps[]` in
/// Dieharder's `diehard_count_1s_stream.c`).
const LETTER_PROBS: [f64; ALPHA_SIZE] = [
    37.0 / 256.0,
    56.0 / 256.0,
    70.0 / 256.0,
    56.0 / 256.0,
    37.0 / 256.0,
];

// Reference statistic parameters (Marsaglia's `sknt1s`, diehard.f line 808;
// Dieharder's diehard_count_1s_stream.c).
const QDIFF_MEAN: f64 = 2500.0;
const QDIFF_STDDEV: f64 = 70.710_678; // √5000

/// Count-the-1's test on a stream of all bytes.
///
/// Uses Marsaglia's Q5 − Q4 difference statistic (`sknt1s`), as Dieharder's
/// `diehard_count_1s_stream.c` ports it.
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

/// The letter, 0 (A) to 4 (E), that DIEHARD assigns a byte by its Hamming
/// weight.
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

    // Reference statistic: Z = (Q5 − Q4 − 2500) / √5000.
    let z = q_difference_z(q5, q4);
    let p_value = erfc(z.abs() / SQRT_2);

    TestResult::with_note(
        name,
        p_value,
        format!(
            "n={N_SAMPLES}, Q5={q5:.2}, Q4={q4:.2}, Q5-Q4={:.2}, Z={z:.4}",
            q5 - q4
        ),
    )
}

/// Marsaglia's naive Pearson sums (Q5, Q4) over `N_SAMPLES` overlapping
/// five-letter words and their leading four letters, read from the first
/// [`LETTERS_PER_TEST`] letters of `letters` (0..ALPHA_SIZE each).
pub(crate) fn q5_q4(mut letters: impl Iterator<Item = usize>) -> (f64, f64) {
    let n = N_SAMPLES;

    let lp = LETTER_PROBS;
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

    // Q5: Vtest chi-square on 5-letter words.
    let q5: f64 = counts5
        .iter()
        .enumerate()
        .map(|(w, &c)| {
            let l = [w / 625, (w / 125) % 5, (w / 25) % 5, (w / 5) % 5, w % 5];
            let exp = nf * lp[l[0]] * lp[l[1]] * lp[l[2]] * lp[l[3]] * lp[l[4]];
            if exp < 5.0 {
                return 0.0;
            }
            (c as f64 - exp).powi(2) / exp
        })
        .sum();

    // Q4: Vtest chi-square on 4-letter words.
    let q4: f64 = counts4
        .iter()
        .enumerate()
        .map(|(w, &c)| {
            let l = [w / 125, (w / 25) % 5, (w / 5) % 5, w % 5];
            let exp = nf * lp[l[0]] * lp[l[1]] * lp[l[2]] * lp[l[3]];
            if exp < 5.0 {
                return 0.0;
            }
            (c as f64 - exp).powi(2) / exp
        })
        .sum();

    (q5, q4)
}

/// Z = (Q5 − Q4 − 2500) / √5000, Marsaglia's standardisation (`sknt1s` and
/// `wknt1s`, `fortran/diehard.f` lines 808 and 902).
pub(crate) fn q_difference_z(q5: f64, q4: f64) -> f64 {
    (q5 - q4 - QDIFF_MEAN) / QDIFF_STDDEV
}

#[cfg(test)]
mod tests {
    use super::{count_ones_stream, hamming_letter, ALPHA_SIZE, LETTER_PROBS, N_SAMPLES, WORD_LEN};

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
