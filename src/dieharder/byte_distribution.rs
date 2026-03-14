//! DIEHARDER test 205 — dab_bytedistrib.
//!
//! Chi-square test on nine distinct byte streams sampled from groups of three
//! consecutive 32-bit words, following `dab_bytedistrib.c`.
//!
//! # Author
//! David Bauer, *Dieharder* (2006), test `dab_bytedistrib`.

use crate::{math::igamc, result::TestResult};

const SAMP_PER_WORD: usize = 3;
const WORDS_PER_TRIAL: usize = 3;
const SAMP_TOTAL: usize = WORDS_PER_TRIAL * SAMP_PER_WORD;
const TABLE_SIZE: usize = 256 * SAMP_TOTAL;

/// Run the byte distribution test.
///
/// # Author
/// David Bauer, Dieharder (2006), `dab_bytedistrib`.
pub fn byte_distribution(words: &[u32]) -> TestResult {
    let tsamples = words.len() / WORDS_PER_TRIAL;
    if tsamples == 0 {
        return TestResult::insufficient("dieharder::byte_distribution", "not enough words");
    }

    let mut counts = vec![0u32; TABLE_SIZE];
    for t in 0..tsamples {
        for i in 0..WORDS_PER_TRIAL {
            let mut word = words[t * WORDS_PER_TRIAL + i];
            let mut current_shift = 0usize;
            for j in 0..SAMP_PER_WORD {
                let shift_amount = ((j + 1) * (32 - 8)) / (SAMP_PER_WORD - 1);
                let byte = (word & 0xff) as usize;
                counts[byte * SAMP_TOTAL + i * SAMP_PER_WORD + j] += 1;
                word >>= shift_amount - current_shift;
                current_shift = shift_amount;
            }
        }
    }

    let expected = tsamples as f64 / 256.0;
    let chi_sq: f64 = counts
        .iter()
        .map(|&c| (c as f64 - expected).powi(2) / expected)
        .sum();

    let p_value = igamc((255 * SAMP_TOTAL) as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "dieharder::byte_distribution",
        p_value,
        format!("tsamples={tsamples}, streams={SAMP_TOTAL}, expected/cell={expected:.1}, χ²={chi_sq:.4}"),
    )
}
