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
//!    standard normal under H₀.  Mean 2500 and σ √5000 are the values given
//!    in Marsaglia's original C source.
//!
//! This crate retains only the stream variant.  The specific-byte-lane
//! variant was retired by Dieharder's author as obsolete compared to the
//! stream form and `rgb_bitdist`.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

const WORD_LEN: usize = 5;
const ALPHA_SIZE: usize = 5;
const N_CATEGORIES5: usize = 3125; // 5^5
const N_CATEGORIES4: usize = 625;  // 5^4
const N_SAMPLES: usize = 256_000;

// Reference statistic parameters (Marsaglia, diehard_count_1s_stream.c).
const QDIFF_MEAN: f64 = 2500.0;
const QDIFF_STDDEV: f64 = 70.710_678; // √5000

/// Count-the-1's test on a stream of all bytes.
///
/// Uses the Q5 − Q4 difference statistic from Marsaglia's reference C source.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn count_ones_stream(words: &[u32]) -> TestResult {
    let bytes_needed = N_SAMPLES + WORD_LEN - 1;
    let words_needed = (bytes_needed + 3) / 4;
    if words.len() < words_needed {
        return TestResult::insufficient("diehard::count_ones_stream", "not enough words");
    }

    let bytes: Vec<u8> = words
        .iter()
        .flat_map(|&w| w.to_le_bytes())
        .take(bytes_needed)
        .collect();

    let letters: Vec<usize> = bytes.iter().map(|&b| hamming_letter(b)).collect();
    count_ones_test(&letters, "diehard::count_ones_stream")
}

fn hamming_letter(b: u8) -> usize {
    match b.count_ones() {
        0 | 1 | 2 => 0, // A
        3          => 1, // B
        4          => 2, // C
        5          => 3, // D
        _          => 4, // E  (6, 7, or 8)
    }
}

fn count_ones_test(letters: &[usize], name: &'static str) -> TestResult {
    let n = N_SAMPLES;

    // Letter marginal probabilities (binomial weights for 8 trials, p=0.5).
    // P(A)=P(E)=37/256, P(B)=P(D)=56/256, P(C)=70/256.
    let lp = [37.0_f64/256.0, 56.0/256.0, 70.0/256.0, 56.0/256.0, 37.0/256.0];
    let nf = n as f64;

    let mut counts5 = [0u32; N_CATEGORIES5];
    let mut counts4 = [0u32; N_CATEGORIES4];

    // Encode the initial (WORD_LEN − 1)-letter prefix.
    let mut word5 = 0usize;
    for &l in &letters[..WORD_LEN - 1] {
        word5 = word5 * ALPHA_SIZE + l;
    }

    // Slide over N complete 5-letter windows.
    for i in (WORD_LEN - 1)..(n + WORD_LEN - 1) {
        word5 = (word5 * ALPHA_SIZE + letters[i]) % N_CATEGORIES5;
        counts5[word5] += 1;
        // The leading 4-letter prefix of this window is word5 / ALPHA_SIZE.
        counts4[word5 / ALPHA_SIZE] += 1;
    }

    // Q5: Vtest chi-square on 5-letter words.
    let q5: f64 = counts5.iter().enumerate().map(|(w, &c)| {
        let l = [w/625, (w/125)%5, (w/25)%5, (w/5)%5, w%5];
        let exp = nf * lp[l[0]] * lp[l[1]] * lp[l[2]] * lp[l[3]] * lp[l[4]];
        if exp < 5.0 { return 0.0; }
        (c as f64 - exp).powi(2) / exp
    }).sum();

    // Q4: Vtest chi-square on 4-letter words.
    let q4: f64 = counts4.iter().enumerate().map(|(w, &c)| {
        let l = [w/125, (w/25)%5, (w/5)%5, w%5];
        let exp = nf * lp[l[0]] * lp[l[1]] * lp[l[2]] * lp[l[3]];
        if exp < 5.0 { return 0.0; }
        (c as f64 - exp).powi(2) / exp
    }).sum();

    // Reference statistic: Z = (Q5 − Q4 − 2500) / √5000.
    let z = (q5 - q4 - QDIFF_MEAN) / QDIFF_STDDEV;
    let p_value = erfc(z.abs() / SQRT_2);

    TestResult::with_note(
        name,
        p_value,
        format!("n={n}, Q5={q5:.2}, Q4={q4:.2}, Q5-Q4={:.2}, Z={z:.4}", q5 - q4),
    )
}
