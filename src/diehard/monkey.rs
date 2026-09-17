//! DIEHARD monkey tests: OPSO, OQSO and DNA.
//!
//! Each test forms 2²¹ twenty-bit "words" from letters of a small alphabet
//! and counts how many of the 2²⁰ possible words never appear.
//!
//! - OPSO: two 10-bit letters.  Each pair of 32-bit words gives two samples,
//!   bits 0–9 of both words and then bits 10–19.
//! - OQSO: four 5-bit letters.  Each group of four words gives six samples,
//!   from bits 0–4, 5–9, …, 25–29 of all four.
//! - DNA: ten 2-bit letters.  Each group of ten words gives sixteen samples,
//!   from bits 0–1, 2–3, …, 30–31 of all ten.
//!
//! Every sample is built from its own bit fields, so under the null the 2²¹
//! samples are independent and uniform over the 2²⁰ cells.  For m independent
//! samples over k cells the number of empty cells has mean k(1 − 1/k)^m and
//! variance k(1 − 1/k)^m + k(k − 1)(1 − 2/k)^m − k²(1 − 1/k)^(2m): at m = 2²¹
//! and k = 2²⁰, μ ≈ 141 909.19 and σ ≈ 290.33.  The p-value is two-sided,
//! erfc(|z|/√2).
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995), and
//! G. Marsaglia and A. Zaman, "Monkey tests for random number generators",
//! *Computers & Mathematics with Applications* 26(9) (1993).

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

const STREAM: usize = 1 << 21;
const WORD_SPACE: usize = 1 << 20;
const BITSET_BYTES: usize = WORD_SPACE / 8;

// Empty-cell moments for m = 2²¹ independent samples over k = 2²⁰ cells
// (see the module documentation).
const MONKEY_MEAN: f64 = 141_909.194_619_809_6;
const MONKEY_SIGMA: f64 = 290.333_061_196_0;

#[inline]
fn mark_seen(seen: &mut [u8], index: usize) {
    seen[index >> 3] |= 1u8 << (index & 7);
}

fn missing_count(seen: &[u8]) -> usize {
    seen.iter().map(|byte| 8 - byte.count_ones() as usize).sum()
}

fn monkey_result(name: &'static str, missing: usize, mean: f64, sigma: f64) -> TestResult {
    let z = (missing as f64 - mean) / sigma;
    let p_value = erfc(z.abs() / SQRT_2);
    TestResult::with_note(name, p_value, format!("missing={missing}, z={z:.4}")).normal(z)
}

/// Overlapping Pairs Sparse Occupancy (OPSO).
///
/// # Author
/// George Marsaglia and Arif Zaman (1993).
pub fn opso(words: &[u32]) -> TestResult {
    if words.len() < STREAM {
        return TestResult::insufficient("diehard::opso", "not enough words");
    }

    let mut seen = vec![0u8; BITSET_BYTES];
    let mut pair = 0usize;
    while pair < STREAM / 2 {
        let j0 = words[2 * pair];
        let k0 = words[2 * pair + 1];

        let low = (((j0 & 0x03ff) as usize) << 10) | ((k0 & 0x03ff) as usize);
        let high = ((((j0 >> 10) & 0x03ff) as usize) << 10) | (((k0 >> 10) & 0x03ff) as usize);

        mark_seen(&mut seen, low);
        mark_seen(&mut seen, high);
        pair += 1;
    }

    monkey_result(
        "diehard::opso",
        missing_count(&seen),
        MONKEY_MEAN,
        MONKEY_SIGMA,
    )
}

/// Overlapping Quadruples Sparse Occupancy (OQSO).
///
/// # Author
/// George Marsaglia and Arif Zaman (1993).
pub fn oqso(words: &[u32]) -> TestResult {
    let words_needed = (STREAM / 6) * 4 + if STREAM.is_multiple_of(6) { 0 } else { 4 };
    if words.len() < words_needed {
        return TestResult::insufficient("diehard::oqso", "not enough words");
    }

    let mut seen = vec![0u8; BITSET_BYTES];
    let mut word_idx = 0usize;
    let mut boffset = 0u32;
    let mut i0 = 0u32;
    let mut j0 = 0u32;
    let mut k0 = 0u32;
    let mut l0 = 0u32;

    for t in 0..STREAM {
        if t % 6 == 0 {
            i0 = words[word_idx];
            j0 = words[word_idx + 1];
            k0 = words[word_idx + 2];
            l0 = words[word_idx + 3];
            word_idx += 4;
            boffset = 0;
        }

        let i = ((i0 >> boffset) & 0x1f) as usize;
        let j = ((j0 >> boffset) & 0x1f) as usize;
        let k = ((k0 >> boffset) & 0x1f) as usize;
        let l = ((l0 >> boffset) & 0x1f) as usize;
        let index = (((i << 5) | j) << 10) | ((k << 5) | l);
        mark_seen(&mut seen, index);
        boffset += 5;
    }

    monkey_result(
        "diehard::oqso",
        missing_count(&seen),
        MONKEY_MEAN,
        MONKEY_SIGMA,
    )
}

/// DNA test.
///
/// # Author
/// George Marsaglia and Arif Zaman (1993).
pub fn dna(words: &[u32]) -> TestResult {
    // Each group of 10 words yields 16 samples at boffset ∈ {0,2,4,...,30}:
    // disjoint 2-bit fields, hence independent samples.
    let groups = STREAM.div_ceil(16);
    let words_needed = groups * 10;
    if words.len() < words_needed {
        return TestResult::insufficient("diehard::dna", "not enough words");
    }

    let mut seen = vec![0u8; BITSET_BYTES];
    let mut word_idx = 0usize;
    let mut boffset = 0u32;
    let mut group = [0u32; 10];

    for t in 0..STREAM {
        if t % 16 == 0 {
            group.copy_from_slice(&words[word_idx..word_idx + 10]);
            word_idx += 10;
            boffset = 0;
        }

        let mut index = 0usize;
        for word in group {
            index = (index << 2) | (((word >> boffset) & 0x3) as usize);
        }
        mark_seen(&mut seen, index);
        boffset += 2;
    }

    monkey_result(
        "diehard::dna",
        missing_count(&seen),
        MONKEY_MEAN,
        MONKEY_SIGMA,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        dna, mark_seen, missing_count, monkey_result, opso, oqso, BITSET_BYTES, MONKEY_MEAN,
        STREAM, WORD_SPACE,
    };

    #[test]
    fn missing_count_tracks_bitset_holes() {
        let mut seen = vec![0u8; BITSET_BYTES];
        mark_seen(&mut seen, 0);
        mark_seen(&mut seen, 7);
        mark_seen(&mut seen, 8);
        assert_eq!(WORD_SPACE - 3, missing_count(&seen));
    }

    #[test]
    fn insufficient_data_returns_skip() {
        let words = vec![0u32; 10];
        assert!(opso(&words).skipped());
        assert!(oqso(&words).skipped());
        assert!(dna(&words).skipped());
    }

    /// The constants against the exact formula, and its Poisson limit.
    #[test]
    fn monkey_moments_match_iid_theory() {
        let (k, m) = (WORD_SPACE as f64, STREAM as f64);
        let q1 = (m * (-1.0 / k).ln_1p()).exp();
        let q2 = (m * (-2.0 / k).ln_1p()).exp();
        let var = k * q1 + k * (k - 1.0) * q2 - k * k * q1 * q1;
        assert!((MONKEY_MEAN - k * q1).abs() < 1e-6, "{}", k * q1);
        assert!(
            (super::MONKEY_SIGMA - var.sqrt()).abs() < 1e-6,
            "{}",
            var.sqrt()
        );
        // Asymptotic check: μ ≈ k·e^{−λ}, σ² ≈ k·e^{−λ}(1 − (1+λ)e^{−λ}), λ = 2.
        let k = WORD_SPACE as f64;
        let lambda = 2.0f64;
        let mu_asym = k * (-lambda).exp();
        let var_asym = k * (-lambda).exp() * (1.0 - (1.0 + lambda) * (-lambda).exp());
        assert!((MONKEY_MEAN - mu_asym).abs() < 0.2);
        assert!((super::MONKEY_SIGMA - var_asym.sqrt()).abs() < 0.01);
    }

    #[test]
    fn constant_stream_fails_hard() {
        let words = vec![0u32; STREAM * 10];
        assert!(opso(&words).p_value < 1e-10);
        assert!(oqso(&words).p_value < 1e-10);
        assert!(dna(&words).p_value < 1e-10);
    }

    #[test]
    fn monkey_result_formats_reasonable_note() {
        let result = monkey_result("test", 123_456, MONKEY_MEAN, 290.0);
        assert_eq!(result.name, "test");
        assert!(result.note.unwrap().contains("missing=123456"));
    }
}
