//! NIST SP 800-22 §2.2 — Frequency Test within a Block.
//!
//! Divides the sequence into M-bit blocks and tests whether the proportion
//! of ones in each block is approximately 1/2.
//!
//! Recommended defaults: M = 128, n ≥ 100.

use crate::{math::igamc, result::TestResult};

/// Run the block-frequency test with block size `m` bits.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.2.
pub fn block_frequency(bits: &[u8], m: usize) -> TestResult {
    let n = bits.len();
    if n < 100 || m < 20 || m > n {
        return TestResult::insufficient("nist::block_frequency", "n < 100 or m out of range");
    }

    let num_blocks = n / m; // N

    // χ² = 4M · Σ (π_j − 0.5)²  where π_j = (ones in block j) / M
    let chi_sq: f64 = bits
        .chunks_exact(m)
        .take(num_blocks)
        .map(|block| {
            let ones: f64 = block.iter().map(|&b| b as f64).sum();
            let pi_j = ones / m as f64;
            (pi_j - 0.5) * (pi_j - 0.5)
        })
        .sum::<f64>()
        * 4.0
        * m as f64;

    let p_value = igamc(num_blocks as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "nist::block_frequency",
        p_value,
        format!("n={n}, M={m}, N={num_blocks}, χ²={chi_sq:.4}"),
    )
}
