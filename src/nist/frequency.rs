//! NIST SP 800-22 §2.1 — Frequency (Monobit) Test.
//!
//! Tests whether the proportion of ones in the sequence is approximately 1/2.
//! A biased generator produces significantly more 0s or 1s than expected.
//!
//! Minimum recommended sequence length: n ≥ 100.

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

/// Run the frequency (monobit) test on the given bit sequence.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.1.
pub fn frequency(bits: &[u8]) -> TestResult {
    let n = bits.len();
    if n < 100 {
        return TestResult::insufficient("nist::frequency", "n < 100");
    }

    // S_n = Σ (2·b_i − 1), so +1 for each 1-bit and −1 for each 0-bit.
    let s_n: i64 = bits
        .iter()
        .map(|&b| if b == 1 { 1i64 } else { -1i64 })
        .sum();

    let s_obs = (s_n.unsigned_abs() as f64) / (n as f64).sqrt();
    let p_value = erfc(s_obs / SQRT_2);

    TestResult::with_note(
        "nist::frequency",
        p_value,
        format!("n={n}, S_n={s_n}, s_obs={s_obs:.4}"),
    )
}
