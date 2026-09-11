//! DIEHARD Test 6 — Bitstream Test.
//!
//! Views the output as a stream of bits and counts missing 20-bit words
//! in 2²¹ overlapping 20-bit windows.  For a truly random stream the count
//! of missing words should be approximately normal with
//! mean = 141 909 and σ = 428.
//!
//! Two differences from DIEHARD.  Marsaglia's `cdbitst` (`fortran/diehard.f`
//! lines 66–146) feeds each word into the stream from bit 0 upward
//! (`j=lshift(and(j,2**19-1),1)+and(num,1)`, lines 122–123); this module
//! reads each word from bit 31 down, as Dieharder's `diehard_bitstream.c`
//! does.  And DIEHARD runs its 20 samples on one continuous stream and prints
//! 20 `phi` values (lines 136–138) with no summary; this module scores 20
//! disjoint chunks of 2²¹ windows two-sided and reports one
//! Kolmogorov–Smirnov test over them.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).
//! Source: Marsaglia's `fortran/diehard.f`, subroutine `cdbitst`.
//! [pubs/diehard-fortran-1996.tar.gz]

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

const WINDOW: usize = 20;
const STREAM_LEN: usize = 1 << 21; // 2^21 overlapping words
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
    let words_needed = bits_needed.div_ceil(32);

    if words.len() < REPEATS * words_needed {
        return TestResult::insufficient("diehard::bitstream", "not enough words");
    }

    let mut p_values = Vec::with_capacity(REPEATS);

    for rep in 0..REPEATS {
        let chunk = &words[rep * words_needed..(rep + 1) * words_needed];
        let missing = count_missing_20bit_words_streaming(chunk, bits_needed);
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

/// Count missing 20-bit words by feeding the rolling window directly from
/// the MSB-first word stream, without materializing a full bit vector.
fn count_missing_20bit_words_streaming(words: &[u32], bits_needed: usize) -> usize {
    let mut seen = vec![false; TOTAL_WORDS];
    let mask = TOTAL_WORDS - 1; // 0x000F_FFFF
    let mut pattern = 0usize;
    let mut bits_fed = 0usize;

    'outer: for &w in words {
        for i in (0..32).rev() {
            let bit = ((w >> i) & 1) as usize;
            pattern = ((pattern << 1) | bit) & mask;
            bits_fed += 1;
            if bits_fed >= WINDOW {
                seen[pattern] = true;
            }
            if bits_fed == bits_needed {
                break 'outer;
            }
        }
    }

    seen.iter().filter(|&&s| !s).count()
}

#[cfg(test)]
mod tests {
    use super::{
        bitstream, count_missing_20bit_words_streaming, REPEATS, STREAM_LEN, TOTAL_WORDS, WINDOW,
    };

    #[test]
    fn uses_exact_number_of_overlapping_windows() {
        let bits_needed = STREAM_LEN + WINDOW - 1;
        assert_eq!(STREAM_LEN, bits_needed - WINDOW + 1);
    }

    #[test]
    fn all_zero_stream_has_all_but_one_missing_word() {
        // An all-zero word stream produces only the all-zeros 20-bit pattern (0).
        // All other 2^20 - 1 patterns are missing.
        let bits_needed = WINDOW + 50;
        let words = vec![0u32; bits_needed.div_ceil(32)];
        assert_eq!(
            TOTAL_WORDS - 1,
            count_missing_20bit_words_streaming(&words, bits_needed)
        );
    }

    #[test]
    fn msb_first_ordering() {
        // Bits read MSB-first from [0xF000_0000, 0] start 1111 then zeros, so
        // the five 20-bit windows ending at bits 20..=24 are 0xF0000, 0xE0000,
        // 0xC0000, 0x80000 and 0: five words seen.  An LSB-first reader would
        // see 24 zero bits and one word.  (Counts replicated in Python.)
        let words = [0xF000_0000u32, 0u32];
        assert_eq!(
            TOTAL_WORDS - 5,
            count_missing_20bit_words_streaming(&words, WINDOW + 4)
        );
    }

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        let needed = REPEATS * (STREAM_LEN + WINDOW - 1).div_ceil(32);
        assert!(bitstream(&[]).skipped());
        assert!(bitstream(&vec![0; needed - 1]).skipped());
        let r = bitstream(&vec![0; needed]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
