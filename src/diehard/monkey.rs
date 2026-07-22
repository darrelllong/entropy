//! DIEHARD Test 7 — Monkey Tests: OPSO, OQSO, DNA.
//!
//! Letter extraction mirrors the Dieharder C (`diehard_opso.c`,
//! `diehard_oqso.c`, `diehard_dna.c`): letter fields come from fixed bit
//! positions inside separate 32-bit words, not a unified continuous bitstream.
//!
//! Deliberate deviation from both DIEHARD and Dieharder: every sample here is
//! built from *disjoint* bit fields, so the 2²¹ samples are mutually
//! independent, and the missing-words statistic is calibrated with the exact
//! iid moments (μ ≈ 141 909.19, σ ≈ 290.33) for all three tests.  Canonical
//! OPSO/OQSO/DNA use overlapping letter words, whose σs are 290/295/339 —
//! applying those to independent samples would misstate the null distribution
//! (cf. the comment in Dieharder's `bitstream.c`: "If you use non-overlapping
//! samples, sigma is 290, not 428").

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

const STREAM: usize = 1 << 21;
const WORD_SPACE: usize = 1 << 20;
const BITSET_BYTES: usize = WORD_SPACE / 8;

// Exact iid missing-words moments for m = 2²¹ independent samples over
// k = 2²⁰ cells:  μ = k(1−1/k)^m,
// σ² = k(1−1/k)^m + k(k−1)(1−2/k)^m − k²(1−1/k)^{2m}.
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
    TestResult::with_note(name, p_value, format!("missing={missing}, z={z:.4}"))
}

/// Overlapping Pairs Sparse Occupancy (OPSO).
///
/// Reference: `diehard_opso.c`
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

    monkey_result("diehard::opso", missing_count(&seen), MONKEY_MEAN, MONKEY_SIGMA)
}

/// Overlapping Quadruples Sparse Occupancy (OQSO).
///
/// Reference: `diehard_oqso.c`
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

    monkey_result("diehard::oqso", missing_count(&seen), MONKEY_MEAN, MONKEY_SIGMA)
}

/// DNA test.
///
/// Reference: `diehard_dna.c`
pub fn dna(words: &[u32]) -> TestResult {
    // Each group of 10 words yields 16 samples at boffset ∈ {0,2,4,...,30} —
    // disjoint 2-bit fields, hence independent samples (see module docs).
    // Dieharder instead refreshes every 32 samples, stepping boffset by 1 with
    // cyclic wraparound (`get_bit_ntuple_from_uint`); its overlapping scheme
    // requires the σ = 337/339 calibration this module deliberately does not use.
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

    monkey_result("diehard::dna", missing_count(&seen), MONKEY_MEAN, MONKEY_SIGMA)
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

    #[test]
    fn monkey_moments_match_iid_theory() {
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
