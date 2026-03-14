//! DIEHARD Tests 8 & 9 — Count-the-1's Tests.
//!
//! Each byte is mapped to a letter {A,B,C,D,E} based on its Hamming weight:
//!   0,1,2 → A;  3 → B;  4 → C;  5 → D;  6,7,8 → E.
//! Overlapping 5-letter words from the stream are counted; the 5⁵ = 3125
//! word frequencies are tested via chi-square.
//!
//! Test 8 uses all bytes from the stream.
//! Test 9 uses only a specific byte lane (byte 0) from each word.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::igamc, result::TestResult};

const WORD_LEN: usize = 5;
const ALPHA_SIZE: usize = 5;
const N_CATEGORIES: usize = 3125; // 5^5
const N_SAMPLES: usize = 256_000;

/// Count-the-1's test on a stream of all bytes.
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

/// Count-the-1's test on a specific byte lane (byte 0 of each word).
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn count_ones_specific_bytes(words: &[u32]) -> TestResult {
    let needed = N_SAMPLES + WORD_LEN - 1;
    if words.len() < needed {
        return TestResult::insufficient(
            "diehard::count_ones_specific_bytes",
            "not enough words",
        );
    }

    let letters: Vec<usize> = words
        .iter()
        .take(needed)
        .map(|&w| hamming_letter((w & 0xFF) as u8))
        .collect();

    count_ones_test(&letters, "diehard::count_ones_specific_bytes")
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
    let mut counts = [0u32; N_CATEGORIES];

    // Encode first window.
    let mut word = 0usize;
    for &l in &letters[..WORD_LEN - 1] {
        word = word * ALPHA_SIZE + l;
    }

    for i in (WORD_LEN - 1)..n + WORD_LEN - 1 {
        word = (word * ALPHA_SIZE + letters[i]) % N_CATEGORIES;
        counts[word] += 1;
    }

    // Letter probabilities: P(weight ∈ {0,1,2})=37/256, P(3)=56/256,
    // P(4)=70/256, P(5)=56/256, P(6,7,8)=37/256.
    // Each letter in a 5-letter word is drawn from an independent byte, so
    // E[count(w)] = n × P(l₁)×…×P(l₅) for word w = (l₁,…,l₅).
    // This replaces the incorrect uniform expected count (n/3125).
    let lp = [37.0_f64/256.0, 56.0/256.0, 70.0/256.0, 56.0/256.0, 37.0/256.0];
    let nf = n as f64;
    let chi_sq: f64 = counts
        .iter()
        .enumerate()
        .map(|(w, &c)| {
            let l = [w/625, (w/125)%5, (w/25)%5, (w/5)%5, w%5];
            let exp = nf * lp[l[0]] * lp[l[1]] * lp[l[2]] * lp[l[3]] * lp[l[4]];
            if exp < 5.0 { return 0.0; }
            (c as f64 - exp).powi(2) / exp
        })
        .sum();

    let df = counts
        .iter()
        .enumerate()
        .filter(|&(w, _)| {
            let l = [w/625, (w/125)%5, (w/25)%5, (w/5)%5, w%5];
            let exp = nf * lp[l[0]] * lp[l[1]] * lp[l[2]] * lp[l[3]] * lp[l[4]];
            exp >= 5.0
        })
        .count()
        .saturating_sub(1);

    let p_value = igamc(df as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(name, p_value, format!("n={n}, χ²={chi_sq:.4}"))
}
