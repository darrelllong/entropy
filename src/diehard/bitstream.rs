//! DIEHARD Test 6 — Bitstream Test.
//!
//! Views the output as a stream of bits and counts missing 20-bit words
//! in 2²¹ overlapping 20-bit windows.  For a truly random stream the count
//! of missing words should be approximately normal with
//! mean = 141 909 and σ = 428.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

const WINDOW: usize = 20;
const STREAM_LEN: usize = 1 << 21;  // 2^21 overlapping words
const TOTAL_WORDS: usize = 1 << 20; // 2^20 possible 20-bit words
const EXPECTED_MISSING: f64 = 141_909.0;
const SIGMA: f64 = 428.0;
const REPEATS: usize = 20;

/// Run the bitstream test (20-bit monkey-at-a-typewriter test).
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn bitstream(words: &[u32]) -> TestResult {
    // Each run needs STREAM_LEN/32 words of input to get STREAM_LEN bits.
    let bits_needed = STREAM_LEN + WINDOW - 1; // 2^21 windows over 2^21+19 bits
    let words_needed = (bits_needed + 31) / 32;

    if words.len() < REPEATS * words_needed {
        return TestResult::insufficient("diehard::bitstream", "not enough words");
    }

    let mut p_values = Vec::with_capacity(REPEATS);

    for rep in 0..REPEATS {
        let chunk = &words[rep * words_needed..(rep + 1) * words_needed];
        let bits = words_to_bits_msb(chunk, bits_needed);
        let missing = count_missing_20bit_words(&bits);
        let z = (missing as f64 - EXPECTED_MISSING) / SIGMA;
        let p = erfc(z.abs() / SQRT_2);
        p_values.push(p);
    }

    // Kolmogorov-Smirnov test on REPEATS p-values.
    let p_value = crate::math::ks_test(&mut p_values);

    TestResult::with_note(
        "diehard::bitstream",
        p_value,
        format!("window=20-bit, stream=2^21, repeats={REPEATS}"),
    )
}

fn words_to_bits_msb(words: &[u32], n: usize) -> Vec<u8> {
    let mut bits = Vec::with_capacity(n);
    for &w in words {
        for i in 0..32 {
            bits.push(((w >> (31 - i)) & 1) as u8);
            if bits.len() == n { return bits; }
        }
    }
    bits
}

fn count_missing_20bit_words(bits: &[u8]) -> usize {
    let mut seen = vec![false; TOTAL_WORDS];
    let mask = TOTAL_WORDS - 1; // 0xFFFFF
    let mut pattern = 0usize;

    // Build initial window.
    for &b in &bits[..WINDOW] {
        pattern = ((pattern << 1) | b as usize) & mask;
    }
    seen[pattern] = true;

    for &b in &bits[WINDOW..] {
        pattern = ((pattern << 1) | b as usize) & mask;
        seen[pattern] = true;
    }

    seen.iter().filter(|&&s| !s).count()
}

#[cfg(test)]
mod tests {
    use super::{count_missing_20bit_words, words_to_bits_msb, STREAM_LEN, WINDOW};

    #[test]
    fn extracts_bits_msb_first() {
        let bits = words_to_bits_msb(&[0x8000_0001], 4);
        assert_eq!(bits, vec![1, 0, 0, 0]);
    }

    #[test]
    fn uses_exact_number_of_overlapping_windows() {
        let bits_needed = STREAM_LEN + WINDOW - 1;
        assert_eq!(STREAM_LEN, bits_needed - WINDOW + 1);
    }

    #[test]
    fn all_zero_stream_has_all_but_one_missing_word() {
        let bits = vec![0u8; WINDOW + 50];
        assert_eq!((1 << 20) - 1, count_missing_20bit_words(&bits));
    }
}
