//! DIEHARDER test 209 — dab_monobit2.
//!
//! An enhanced high-sensitivity monobit test.  Tests the balance between 0s
//! and 1s at very fine granularity: runs many independent monobit checks on
//! short sub-sequences and aggregates via chi-square.  Described as "the best
//! single test in Dieharder by far" for detecting generator bias.
//!
//! # Author
//! David Bauer, *Dieharder* (2006), test `dab_monobit2`.

use crate::{math::igamc, result::TestResult};

const BLOCK_SIZE: usize = 65_536;  // 2^16 bits per block
const N_BLOCKS: usize = 200;

/// Run the enhanced monobit test.
///
/// # Author
/// David Bauer, Dieharder (2006), `dab_monobit2`.
pub fn monobit2(words: &[u32]) -> TestResult {
    let bits_per_block = BLOCK_SIZE;
    let words_per_block = bits_per_block / 32;
    let words_needed = N_BLOCKS * words_per_block;

    if words.len() < words_needed {
        return TestResult::insufficient("dieharder::monobit2", "not enough words");
    }

    // For each block: count ones, compute χ² contribution.
    // Under H₀, ones ~ Bin(N, 0.5) ≈ N(N/2, N/4).
    // We use chi-square across all blocks simultaneously.
    let expected = bits_per_block as f64 / 2.0;
    let variance = bits_per_block as f64 / 4.0;

    let chi_sq: f64 = (0..N_BLOCKS)
        .map(|b| {
            let start = b * words_per_block;
            let ones: u32 = words[start..start + words_per_block]
                .iter()
                .map(|&w| w.count_ones())
                .sum();
            (ones as f64 - expected).powi(2) / variance
        })
        .sum();

    // Under H₀, chi_sq ~ χ²_{N_BLOCKS}.
    let p_value = igamc(N_BLOCKS as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "dieharder::monobit2",
        p_value,
        format!("blocks={N_BLOCKS}, block_size={BLOCK_SIZE}, χ²={chi_sq:.4}"),
    )
}
