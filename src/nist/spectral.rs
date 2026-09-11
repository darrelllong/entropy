//! NIST SP 800-22 §2.6 — Discrete Fourier Transform (Spectral) Test.
//!
//! Converts the bit sequence to a ±1 sequence, applies the FFT, and checks
//! whether the number of DFT magnitudes that exceed a threshold is consistent
//! with an i.i.d. uniform source.
//!
//! Uses an O(n log n) FFT on the full input sequence via [`crate::math::fft_magnitudes`].
//!
//! The threshold T = √(log(1/0.05)·n) and d = (N₁ − N₀)/√(n·0.95·0.05/4) are
//! the forms SP 800-22 Rev. 1a prints in §2.6.4 and §3.6.  The publication
//! does not say where these forms come from; §3.6 lists Kim, Umeno and
//! Hasegawa's "Corrections of the NIST Statistical Test Suite for Randomness"
//! and Killmann et al.'s note on the DFT test among its references.
//!
//! Minimum recommended sequence length: n ≥ 1 000.
//!
//! # References
//! * A. Rukhin et al., *NIST SP 800-22 Rev. 1a*, 2010, §2.6 and §3.6.
//!   [pubs/NIST-SP-800-22r1a.pdf]
//! * S. Kim, K. Umeno and A. Hasegawa, "Corrections of the NIST Statistical
//!   Test Suite for Randomness," Cryptology ePrint Archive, Report 2004/018,
//!   2004.  [Cited in §3.6]
//! * W. Killmann, J. Schüth, W. Thumser and I. Uludag, "A Note Concerning the
//!   DFT Test in NIST Special Publication 800-22," T-Systems, Systems
//!   Integration, July 2004.  [Cited in §3.6]

use crate::{
    math::{erfc, fft_magnitudes},
    result::TestResult,
};
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

    // Convert bits to ±1.
    let x: Vec<f64> = bits
        .iter()
        .map(|&b| if b == 1 { 1.0 } else { -1.0 })
        .collect();

    // FFT magnitudes; only the first n/2 are independent.
    let mags = fft_magnitudes(&x);

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
