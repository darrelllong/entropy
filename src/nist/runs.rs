//! NIST SP 800-22 §2.3 — Runs Test.
//!
//! A *run* is a maximal unbroken subsequence of identical bits.
//! The test checks whether the number of runs of 0s and 1s is consistent
//! with what a truly random sequence should produce.
//!
//! Pre-requisite: |π − 0.5| < 2/√n, otherwise the sequence fails the
//! prerequisite and p-value is set to 0.
//!
//! Minimum recommended sequence length: n ≥ 100.

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

/// Run the runs test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.3.
pub fn runs(bits: &[u8]) -> TestResult {
    let n = bits.len();
    if n < 100 {
        return TestResult::insufficient("nist::runs", "n < 100");
    }

    let ones: f64 = bits.iter().map(|&b| b as f64).sum();
    let pi = ones / n as f64;

    // Pre-test: if |π − 0.5| ≥ 2/√n the frequency prerequisite fails.
    if (pi - 0.5).abs() >= 2.0 / (n as f64).sqrt() {
        return TestResult::with_note(
            "nist::runs",
            0.0,
            format!("pre-test failed: π={pi:.4}"),
        );
    }

    // Count runs: V_n = number of transitions + 1.
    let v_n: usize = bits.windows(2).filter(|w| w[0] != w[1]).count() + 1;

    let numer = (v_n as f64 - 2.0 * n as f64 * pi * (1.0 - pi)).abs();
    let denom = 2.0 * (2.0 * n as f64).sqrt() * pi * (1.0 - pi);
    let p_value = erfc(numer / (denom * SQRT_2));

    TestResult::with_note(
        "nist::runs",
        p_value,
        format!("n={n}, V_n={v_n}, π={pi:.4}"),
    )
}
