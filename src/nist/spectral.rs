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

    let Dft {
        threshold,
        n0,
        n1,
        d,
        p_value,
    } = dft_statistic(bits);

    TestResult::with_note(
        "nist::spectral",
        p_value,
        format!("n={n}, N₀={n0:.1}, N₁={n1}, T={threshold:.4}, d={d:.4}"),
    )
}

/// The quantities of §2.6.4 steps (4)–(8).
struct Dft {
    /// Peak-height threshold T.
    threshold: f64,
    /// Expected number of peaks below T, N₀.
    n0: f64,
    /// Observed number of peaks below T, N₁.
    n1: usize,
    /// Normalised difference d.
    d: f64,
    /// erfc(|d|/√2).
    p_value: f64,
}

/// §2.6.4 steps (1)–(8), without `spectral`'s length gate.
fn dft_statistic(bits: &[u8]) -> Dft {
    let n = bits.len();

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
    let n1 = mags[..n / 2].iter().filter(|&&m| m < threshold).count();

    let d = (n1 as f64 - n0) / (n as f64 * 0.95 * 0.05 / 4.0).sqrt();
    let p_value = erfc(d.abs() / SQRT_2);

    Dft {
        threshold,
        n0,
        n1,
        d,
        p_value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nist::test_vectors::{bits, EPSILON_100};

    /// SP 800-22 §2.6.8: n = 100 and N₀ = 47.5.  The publication prints
    /// N₁ = 46, d = −1.376494 and P-value = 0.168669, but STS 2.1.2 finds 48
    /// of the first 50 magnitudes below T = 17.308 on this input and prints
    /// d = 0.458831 and P-value = 0.646355, as this code does.  The count is
    /// also 48 over frequencies 1 to 50 instead of 0 to 49, with ≤ for <, and
    /// with the threshold √(3n) that predates §2.6.4's, so none of those
    /// explains the printed 46.  The printed d and P-value do follow from 46
    /// by steps (7) and (8).  The example is below `spectral`'s n ≥ 1000
    /// gate, so it runs through the statistic directly.
    #[test]
    fn section_2_6_8_example_counts_as_sts_does() {
        let dft = dft_statistic(&bits(EPSILON_100));
        assert!((dft.n0 - 47.5).abs() < 1e-12, "N₀ = {}", dft.n0);
        assert_eq!(dft.n1, 48);
        assert!((dft.d - 0.458831).abs() < 1e-6, "d = {}", dft.d);
        assert!((dft.p_value - 0.646355).abs() < 1e-6, "p = {}", dft.p_value);
        let printed_d = (46.0 - dft.n0) / (100.0 * 0.95 * 0.05 / 4.0_f64).sqrt();
        assert!((printed_d + 1.376494).abs() < 1e-6, "d = {printed_d}");
        assert!((erfc(printed_d.abs() / SQRT_2) - 0.168669).abs() < 1e-6);
    }
}
