//! DIEHARDER byte distribution test.
//!
//! Each trial reads three words and, from each, the bytes at bit offsets 0,
//! 12 and 24: nine byte streams.  Every stream is counted over the 256 byte
//! values, and the Pearson χ² summed over the nine 256-cell tables, df
//! 9 × 255, is the statistic.
//!
//! # Author
//! David Bauer, in Robert G. Brown's *Dieharder: A Random Number Test Suite*
//! (2006).

use crate::{math::igamc, result::TestResult};

const SAMP_PER_WORD: usize = 3;
const WORDS_PER_TRIAL: usize = 3;
const SAMP_TOTAL: usize = WORDS_PER_TRIAL * SAMP_PER_WORD;
const TABLE_SIZE: usize = 256 * SAMP_TOTAL;
/// Bits between the bytes read from one word: offsets 0, 12 and 24.
const BYTE_STEP: usize = (32 - 8) / (SAMP_PER_WORD - 1);

/// Run the byte distribution test.
///
/// # Author
/// David Bauer, Dieharder (2006).
pub fn byte_distribution(words: &[u32]) -> TestResult {
    let tsamples = words.len() / WORDS_PER_TRIAL;
    // Each of the 9 byte streams is chi-squared over 256 cells with expected
    // count tsamples/256; Cochran's rule needs that ≥ 5, i.e. tsamples ≥ 1280.
    if tsamples < 5 * 256 {
        return TestResult::insufficient(
            "dieharder::byte_distribution",
            "need at least 1280 trials (3840 words) for valid χ² expected counts",
        );
    }

    let mut counts = vec![0u32; TABLE_SIZE];
    for trial in words.chunks_exact(WORDS_PER_TRIAL).take(tsamples) {
        for (i, &word) in trial.iter().enumerate() {
            for j in 0..SAMP_PER_WORD {
                let byte = ((word >> (BYTE_STEP * j)) & 0xff) as usize;
                counts[byte * SAMP_TOTAL + i * SAMP_PER_WORD + j] += 1;
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
    .chi_square(chi_sq, (255 * SAMP_TOTAL) as f64)
}

#[cfg(test)]
mod tests {
    use super::{byte_distribution, WORDS_PER_TRIAL};

    /// 1 280 trials is the smallest sample with 5 expected counts per cell.
    const NEEDED: usize = 5 * 256 * WORDS_PER_TRIAL;

    #[test]
    fn short_inputs_skip_and_constant_input_fails() {
        assert!(byte_distribution(&[]).skipped());
        assert!(byte_distribution(&vec![0; NEEDED - 1]).skipped());
        let r = byte_distribution(&vec![0; NEEDED]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
