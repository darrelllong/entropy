//! DIEHARD Test 7 — Monkey Tests: OPSO, OQSO, DNA.
//!
//! Each test counts missing "words" in a sparse alphabet generated from
//! overlapping subfields of the 32-bit output.  Missing word counts should
//! be approximately normal.
//!
//! | Test | Alphabet | Word length | Expected missing | σ |
//! |------|----------|-------------|-----------------|---|
//! | OPSO | 1024 (10-bit) | 2 | 141 909 | 290 |
//! | OQSO | 32 (5-bit)   | 4 | 141 909 | 295 |
//! | DNA  | 4 (2-bit)    | 10 | 141 909 | 339 |
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

const STREAM: usize = 1 << 21; // 2^21 overlapping words

/// Overlapping Pairs Sparse Occupancy (OPSO).
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn opso(words: &[u32]) -> TestResult {
    monkey_test(
        words,
        10,  // bits per letter
        2,   // letters per word
        290.0,
        "diehard::opso",
    )
}

/// Overlapping Quadruples Sparse Occupancy (OQSO).
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn oqso(words: &[u32]) -> TestResult {
    monkey_test(
        words,
        5,   // bits per letter
        4,   // letters per word
        295.0,
        "diehard::oqso",
    )
}

/// DNA test: 4-letter alphabet {C,G,A,T}, 10-letter words.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn dna(words: &[u32]) -> TestResult {
    monkey_test(
        words,
        2,   // bits per letter
        10,  // letters per word
        339.0,
        "diehard::dna",
    )
}

/// Generic monkey test.
///
/// Extracts overlapping words of `word_len` letters from the bit stream,
/// where each letter is `bits_per_letter` bits.
fn monkey_test(
    words: &[u32],
    bits_per_letter: usize,
    word_len: usize,
    sigma: f64,
    name: &'static str,
) -> TestResult {
    let letter_bits = bits_per_letter;
    let alphabet_size = 1usize << letter_bits;
    let total_possible = alphabet_size.pow(word_len as u32);

    // Total bits needed: STREAM letters of `letter_bits` each + overlap.
    let bits_needed = STREAM * letter_bits + letter_bits * (word_len - 1);
    let words_needed = (bits_needed + 31) / 32;

    if words.len() < words_needed {
        return TestResult::insufficient(name, "not enough words");
    }

    // Unpack bits.
    let mut bits = Vec::with_capacity(bits_needed);
    'outer: for &w in words {
        for i in 0..32 {
            bits.push(((w >> i) & 1) as u8);
            if bits.len() >= bits_needed { break 'outer; }
        }
    }

    // Build letters from bit stream.
    let _letter_mask = alphabet_size - 1;
    let word_mask = total_possible - 1;
    let mut seen = vec![false; total_possible];

    let mut current_word = 0usize;

    // Initialise the first (word_len − 1) letters.
    for i in 0..word_len - 1 {
        let mut letter = 0usize;
        for j in 0..letter_bits {
            letter |= (bits[i * letter_bits + j] as usize) << j;
        }
        current_word = (current_word * alphabet_size + letter) & word_mask;
    }

    // Slide over STREAM letters.
    for i in (word_len - 1)..(word_len - 1 + STREAM) {
        let mut letter = 0usize;
        for j in 0..letter_bits {
            let idx = i * letter_bits + j;
            if idx < bits.len() {
                letter |= (bits[idx] as usize) << j;
            }
        }
        current_word = (current_word * alphabet_size + letter) & word_mask;
        seen[current_word] = true;
    }

    let missing = seen.iter().filter(|&&s| !s).count();
    let expected = 141_909.0_f64;
    let z = (missing as f64 - expected) / sigma;
    let p_value = erfc(z.abs() / SQRT_2);

    TestResult::with_note(name, p_value, format!("missing={missing}, z={z:.4}"))
}
