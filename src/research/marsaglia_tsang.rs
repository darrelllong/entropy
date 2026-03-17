//! Marsaglia–Tsang "difficult tests" research implementations.
//!
//! This module currently implements the Gorilla test described in:
//!
//! George Marsaglia and Wai Wan Tsang, "Some difficult-to-pass tests of
//! randomness", Journal of Statistical Software 7(3), 2002.
//!
//! The paper's Gorilla test:
//! - selects one bit position from each 32-bit output word
//! - forms a bit stream of length 2^26 + 25
//! - counts how many 26-bit words are missing from the 2^26 overlapping windows
//! - compares the missing-word count to a normal approximation with
//!   mean 24,687,971 and standard deviation 4,170
//! - then runs a KS uniformity check on the resulting 32 per-bit p-values
//!   to catch generators whose problem is collective non-uniformity across
//!   bit positions rather than one spectacularly bad bit

use crate::math::{ks_test, normal_cdf};

const GORILLA_WORD_BITS: usize = 26;
const GORILLA_WINDOWS: usize = 1 << GORILLA_WORD_BITS;
const GORILLA_STREAM_BITS: usize = GORILLA_WINDOWS + GORILLA_WORD_BITS - 1;
const GORILLA_MISSING_MEAN: f64 = 24_687_971.0;
const GORILLA_MISSING_STDDEV: f64 = 4_170.0;

fn bit_is_set(words: &[u32], word_index: usize, bit_position_from_msb: usize) -> bool {
    let shift = 31usize.saturating_sub(bit_position_from_msb);
    ((words[word_index] >> shift) & 1) != 0
}

fn set_seen(seen: &mut [u64], value: usize) -> bool {
    let idx = value / 64;
    let bit = value % 64;
    let mask = 1u64 << bit;
    let was_set = (seen[idx] & mask) != 0;
    if !was_set {
        seen[idx] |= mask;
    }
    !was_set
}

fn missing_words_for_bit(words: &[u32], bit_position_from_msb: usize, word_bits: usize) -> usize {
    assert!((1..=31).contains(&word_bits), "word_bits must be in 1..=31");
    let windows = 1usize << word_bits;
    let stream_bits = windows + word_bits - 1;
    assert!(
        words.len() >= stream_bits,
        "not enough source words for Gorilla stream"
    );
    assert!(bit_position_from_msb < 32, "bit position must be in 0..32");

    let mut seen = vec![0u64; windows.div_ceil(64)];
    let mask = windows - 1;
    let mut rolling = 0usize;
    let mut seen_count = 0usize;

    for idx in 0..stream_bits {
        let bit = usize::from(bit_is_set(words, idx, bit_position_from_msb));
        rolling = ((rolling << 1) & mask) | bit;
        if idx + 1 >= word_bits && set_seen(&mut seen, rolling) {
            seen_count += 1;
        }
    }
    windows - seen_count
}

/// One Marsaglia–Tsang Gorilla result for a single bit position.
#[derive(Debug, Clone)]
pub struct GorillaBitResult {
    pub bit_position: usize,
    pub missing_words: usize,
    pub z_score: f64,
    pub p_value: f64,
}

/// Run the full 32-bit-position Gorilla test.
///
/// Bit positions are numbered 0..31 from most-significant to least-significant,
/// matching the paper.
pub fn gorilla_all(words: &[u32]) -> Vec<GorillaBitResult> {
    assert!(
        words.len() >= GORILLA_STREAM_BITS,
        "gorilla_all requires at least 2^26 + 25 source words"
    );
    (0..32)
        .map(|bit_position| {
            let missing_words = missing_words_for_bit(words, bit_position, GORILLA_WORD_BITS);
            let z_score = (missing_words as f64 - GORILLA_MISSING_MEAN) / GORILLA_MISSING_STDDEV;
            GorillaBitResult {
                bit_position,
                missing_words,
                z_score,
                p_value: 1.0 - normal_cdf(z_score),
            }
        })
        .collect()
}

/// Run a KS uniformity check on the 32 per-bit p-values from [`gorilla_all`].
///
/// Returns a single aggregate p-value.  A value near 0 means the per-bit
/// p-values are not uniformly distributed on [0, 1], which indicates
/// systematic non-randomness spread across bit positions rather than an
/// isolated bad bit.  This is the second-stage aggregate check described in
/// Marsaglia and Tsang (2002).
pub fn gorilla_aggregate_ks(results: &[GorillaBitResult]) -> f64 {
    let mut pvals: Vec<f64> = results.iter().map(|r| r.p_value).collect();
    ks_test(&mut pvals)
}

#[cfg(test)]
mod tests {
    use super::{
        missing_words_for_bit, GorillaBitResult, GORILLA_MISSING_MEAN, GORILLA_MISSING_STDDEV,
    };
    use crate::math::normal_cdf;

    #[test]
    fn alternating_bit_stream_misses_all_but_two_patterns_for_small_word_size() {
        let words = (0..20)
            .map(|i| if i % 2 == 0 { 0xaaaa_aaaa } else { 0x5555_5555 })
            .collect::<Vec<_>>();
        let missing = missing_words_for_bit(&words, 0, 3);
        assert_eq!(6, missing);
    }

    #[test]
    fn constant_zero_stream_hits_exactly_one_pattern_for_small_word_size() {
        let words = vec![0u32; 20];
        let missing = missing_words_for_bit(&words, 5, 4);
        assert_eq!(15, missing);
    }

    #[test]
    fn gorilla_p_value_uses_upper_tail_for_excess_missing_words() {
        let z_score = 3.0;
        let result = GorillaBitResult {
            bit_position: 0,
            missing_words: (GORILLA_MISSING_MEAN + 3.0 * GORILLA_MISSING_STDDEV) as usize,
            z_score,
            p_value: 1.0 - normal_cdf(z_score),
        };
        assert!(
            result.p_value < 0.01,
            "excess missing words should yield a small p-value"
        );
    }
}
