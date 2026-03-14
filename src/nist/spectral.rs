//! NIST SP 800-22 §2.6 — Discrete Fourier Transform (Spectral) Test.
//!
//! Converts the bit sequence to a ±1 sequence, applies the DFT, and checks
//! whether the number of DFT magnitudes that exceed a threshold is consistent
//! with an i.i.d. uniform source.
//!
//! Uses the O(n²) DFT from [`crate::math::dft_magnitudes`]; for the default
//! n = 1 000 this is ~10⁶ operations and takes < 1 ms.
//!
//! Minimum recommended sequence length: n ≥ 1 000.

use crate::{math::{dft_magnitudes, erfc}, result::TestResult};
use std::f64::consts::SQRT_2;

/// Run the spectral (DFT) test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.6.
pub fn spectral(bits: &[u8]) -> TestResult {
    let n = bits.len();
    if n < 1_000 {
        return TestResult::insufficient("nist::spectral", "n < 1000");
    }

    // The O(n²) DFT is only tractable for small n.  NIST §2.6 recommends n = 1 000
    // as the default; we cap at 1 000 so the test runs in bounded time.
    let bits = &bits[..1_000.min(n)];
    let n = bits.len();

    // Convert bits to ±1.
    let x: Vec<f64> = bits.iter().map(|&b| if b == 1 { 1.0 } else { -1.0 }).collect();

    // DFT magnitudes; only the first n/2 are independent.
    let mags = dft_magnitudes(&x);

    // Threshold T such that P(|X_k| < T) = 0.95 under H₀.
    let threshold = (n as f64 * 0.05_f64.ln().abs()).sqrt();

    let n0 = 0.95 * n as f64 / 2.0; // expected count below threshold
    let n1 = mags[..n / 2].iter().filter(|&&m| m < threshold).count() as f64;

    let d = (n1 - n0) / (n as f64 * 0.95 * 0.05 / 4.0).sqrt();
    let p_value = erfc(d.abs() / SQRT_2);

    TestResult::with_note(
        "nist::spectral",
        p_value,
        format!("n={n}, N₀={n0:.1}, N₁={n1}, T={threshold:.4}, d={d:.4}"),
    )
}
