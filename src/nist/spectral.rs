//! NIST SP 800-22 §2.6 — Discrete Fourier Transform (Spectral) Test.
//!
//! Converts the bit sequence to a ±1 sequence, applies the FFT, and checks
//! whether the number of DFT magnitudes that exceed a threshold is consistent
//! with an i.i.d. uniform source.
//!
//! Uses an O(n log n) FFT on the full input sequence via [`crate::math::fft_magnitudes`].
//!
//! The threshold T = √(log(1/0.05)·n) and d = (N₁ − N₀)/√(n·0.95·0.05/4) are
//! the forms SP 800-22 Rev. 1a prints in §2.6.4 and §3.6.  Both are the
//! corrections of Kim, Umeno and Hasegawa, whom §3.6 lists among its
//! references.  Their §3.1 (eqs. (9)–(12), p. 9) derives
//! T = √(2.995732274·n) from the exponential tail of |Sⱼ|²/n in place of
//! √(3n).  Their §3.2 (p. 10) replaces the variance n·0.95·0.05/2 with
//! n·0.95·0.05/4, since the n/2 peaks are not independent trials.  This module
//! uses both, and counts the zero frequency and the next n/2 − 1 magnitudes.
//!
//! Minimum recommended sequence length: n ≥ 1 000.
//!
//! # Calibration
//!
//! On 200 000 null streams of PCG64, xoshiro256** and SFC64 at each of
//! 16 000 000, 4 194 304 and 1 048 576 bits (`examples/statistic_scale.rs`,
//! `stats/statistic-scale.txt`) d has mean 0 to within 0.002 and standard
//! deviation 1.024, 1.027 and 1.027, with standard error 0.0016, so the
//! variance n·0.95·0.05/4 is short by a factor of about 1.05 at every size,
//! and scored by it the test rejected 1.2% of null streams at the 1% level.
//! The statistic the test records, and takes its p-value from, is therefore
//! d divided by [`SIGMA_RATIO`], the mean of the three; d itself, printed in
//! the note and checked against the publication's example, keeps the
//! standard's form.  On streams the calibration never saw
//! (`stats/statistic-validate.txt`: 200 000 at each of 1 048 576 and
//! 4 194 304 bits, 50 000 at 16 000 000) the recorded statistic has standard
//! deviation 1.002, 1.001 and 0.999 and the test rejects 0.98%, 0.98% and
//! 0.93% at the 1% level.
//!
//! # References
//! * A. Rukhin et al., *NIST SP 800-22 Rev. 1a*, 2010, §2.6 and §3.6.
//!   [pubs/NIST-SP-800-22r1a.pdf]
//! * S. Kim, K. Umeno and A. Hasegawa, "Corrections of the NIST Statistical
//!   Test Suite for Randomness," Cryptology ePrint Archive, Report 2004/018,
//!   2004.  [pubs/kim-umeno-hasegawa-2004-nist-sts-corrections.pdf]
//!   [§3.1 threshold, §3.2 variance; cited in §3.6]
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
    .normal(d / SIGMA_RATIO)
}

/// The fraction of peaks below the threshold under H₀, §2.6.4 step (5): T is
/// the height that 95% of the |Sⱼ| fall under.
const BELOW_THRESHOLD: f64 = 0.95;

/// The measured standard deviation of d, whose printed law gives 1: the mean
/// over the three sample sizes of the calibration in the module
/// documentation.  The recorded statistic is d divided by it.
const SIGMA_RATIO: f64 = 1.026;

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
    /// erfc(|d|/(SIGMA_RATIO·√2)).
    p_value: f64,
}

/// §2.6.4 steps (1)–(8), without `spectral`'s length gate.
fn dft_statistic(bits: &[u8]) -> Dft {
    let n = bits.len();

    // The DFT of the ±1 sequence; a bit's zero mean under H₀ is what makes
    // |Sⱼ|²/n exponential.
    let x: Vec<f64> = bits
        .iter()
        .map(|&b| if b == 1 { 1.0 } else { -1.0 })
        .collect();

    // FFT magnitudes; only the first n/2 are independent.
    let mags = fft_magnitudes(&x);

    // |Sⱼ|²/n is exponential under H₀, so P(|Sⱼ| < T) = BELOW_THRESHOLD at
    // T² = −n·ln(1 − BELOW_THRESHOLD).
    let threshold = (-(n as f64) * (1.0 - BELOW_THRESHOLD).ln()).sqrt();

    let n0 = BELOW_THRESHOLD * n as f64 / 2.0;
    let n1 = mags[..n / 2].iter().filter(|&&m| m < threshold).count();

    let variance = n as f64 * BELOW_THRESHOLD * (1.0 - BELOW_THRESHOLD) / 4.0;
    let d = (n1 as f64 - n0) / variance.sqrt();
    let p_value = erfc(d.abs() / (SIGMA_RATIO * SQRT_2));

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
    /// A count this test computes exactly, up to floating rounding.
    const CLOSED_FORM: f64 = 1e-12;

    /// SP 800-22 quotes its example values to six decimals.
    const PUBLISHED: f64 = 1e-6;

    use super::*;
    use crate::nist::test_vectors::{bits, EPSILON_100};

    /// SP 800-22 §2.6.8: n = 100 and N₀ = 47.5.  The publication prints
    /// N₁ = 46, d = −1.376494 and P-value = 0.168669, but 48 of the first 50
    /// magnitudes lie below T = 17.308 on this input, giving d = 0.458831 and
    /// P-value = 0.646355.  The count is also 48 over frequencies 1 to 50
    /// instead of 0 to 49, with ≤ for <, and with the threshold √(3n) that
    /// predates §2.6.4's, so none of those explains the printed 46.  The
    /// printed d and P-value do follow from 46 by steps (7) and (8).  The
    /// example is below `spectral`'s n ≥ 1000 gate, so it runs through the
    /// statistic directly; the P-values here are step (8)'s, before the
    /// calibration.
    #[test]
    fn section_2_6_8_example_counts() {
        let dft = dft_statistic(&bits(EPSILON_100));
        assert!((dft.n0 - 47.5).abs() < CLOSED_FORM, "N₀ = {}", dft.n0);
        assert_eq!(dft.n1, 48);
        assert!((dft.d - 0.458831).abs() < PUBLISHED, "d = {}", dft.d);
        let step_8_p = erfc(dft.d.abs() / SQRT_2);
        assert!((step_8_p - 0.646355).abs() < PUBLISHED, "p = {step_8_p}");
        assert!((dft.p_value - erfc(dft.d.abs() / (SIGMA_RATIO * SQRT_2))).abs() < CLOSED_FORM);
        let printed_d =
            (46.0 - dft.n0) / (100.0 * BELOW_THRESHOLD * (1.0 - BELOW_THRESHOLD) / 4.0).sqrt();
        assert!((printed_d + 1.376494).abs() < PUBLISHED, "d = {printed_d}");
        assert!((erfc(printed_d.abs() / SQRT_2) - 0.168669).abs() < PUBLISHED);
    }
}
