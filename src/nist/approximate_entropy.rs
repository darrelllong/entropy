//! NIST SP 800-22 §2.12 — Approximate Entropy Test.
//!
//! Computes the approximate entropy ApEn(m) = φ(m) − φ(m+1), where φ(m)
//! is the average log-count of overlapping m-bit patterns in the circular
//! sequence.  A small ApEn indicates the sequence is more regular than
//! expected for random data.
//!
//! Recommended defaults: m = 10 for n ≥ 10^6 (SP 800-22 §2.12.7).
//!
//! # References
//! * A. Rukhin et al., *NIST SP 800-22 Rev. 1a*, 2010, §2.12.
//!   [pubs/NIST-SP-800-22r1a.pdf]
//! * S. M. Pincus, "Approximate entropy as a measure of system complexity,"
//!   *Proceedings of the National Academy of Sciences* 88(6), pp. 2297–2301,
//!   March 1991. DOI: 10.1073/pnas.88.6.2297.
//!   [Original ApEn statistic definition]

use crate::{math::chi2_pvalue, result::TestResult};
use std::f64::consts::LN_2;

/// Run the approximate entropy test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.12.
pub fn approximate_entropy(bits: &[u8], m: usize) -> TestResult {
    let n = bits.len();
    // §2.12.7: "Choose m and n such that m < ⌊log2 n⌋ − 5", i.e. n ≥ 2^{m+6}.
    // (The φ(m+1) table has 2^{m+1} cells, so this also keeps both pattern
    // tables well populated.)
    if m >= 30 {
        return TestResult::unsupported("nist::approximate_entropy", "m must be below 30");
    }
    if n == 0 || n < (1usize << (m + 6)) {
        return TestResult::insufficient(
            "nist::approximate_entropy",
            "m violates m < ⌊log₂ n⌋ − 5 (§2.12.7)",
        );
    }

    let phi_m = phi(bits, m, n);
    let phi_m1 = phi(bits, m + 1, n);

    let ap_en = phi_m - phi_m1;
    let (chi_sq, p_value) = chi_square_and_p(ap_en, n, m);

    TestResult::with_note(
        "nist::approximate_entropy",
        p_value,
        format!("n={n}, m={m}, ApEn={ap_en:.6}, χ²={chi_sq:.4}"),
    )
}

/// §2.12.4 steps 6–7: χ² = 2n(ln 2 − ApEn(m)) and P = igamc(2^{m−1}, χ²/2),
/// the χ² tail with 2^m degrees of freedom.
fn chi_square_and_p(ap_en: f64, n: usize, m: usize) -> (f64, f64) {
    let chi_sq = 2.0 * n as f64 * (LN_2 - ap_en);
    (chi_sq, chi2_pvalue(chi_sq, 1 << m))
}

/// Compute φ(m) = (1/n) Σ_{all patterns p} C_m(p) · ln(C_m(p)/n)
/// where C_m(p) is the count of overlapping occurrences of pattern p
/// in the circular sequence.
fn phi(bits: &[u8], m: usize, n: usize) -> f64 {
    let table_size = 1usize << m;
    let mut counts = vec![0u32; table_size];

    for i in 0..n {
        let mut pattern = 0usize;
        for j in 0..m {
            pattern = (pattern << 1) | bits[(i + j) % n] as usize;
        }
        counts[pattern] += 1;
    }

    let sum: f64 = counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let cf = c as f64;
            cf * (cf / n as f64).ln()
        })
        .sum();

    sum / n as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nist::test_vectors::{bits, EPSILON_100};

    /// §2.12.7 reads "m < ⌊log₂ n⌋ − 5", so n = 2^{m+6} − 1 is the largest
    /// length that must be skipped and n = 2^{m+6} the smallest that runs.
    #[test]
    fn m_gate_follows_the_floor_in_section_2_12_7() {
        for m in [2usize, 10] {
            let boundary = 1usize << (m + 6);
            let stream: Vec<u8> = bits(EPSILON_100)
                .into_iter()
                .cycle()
                .take(boundary)
                .collect();
            let below = approximate_entropy(&stream[..boundary - 1], m);
            let at = approximate_entropy(&stream, m);
            assert!(below.skipped(), "m = {m}, n = {}: {below}", boundary - 1);
            assert!(!at.skipped(), "m = {m}, n = {boundary}: {at}");
        }
    }

    /// SP 800-22 §2.12.8: m = 2, n = 100 gives ApEn(2) = 0.665393,
    /// χ² = 5.550792 and P-value = 0.235301.  The example itself is outside
    /// the §2.12.7 gate (2 ≥ ⌊log₂ 100⌋ − 5 = 1), so the statistic is checked
    /// through φ and the same χ² and P code the test runs.
    #[test]
    fn phi_reproduces_section_2_12_8_example() {
        let eps = bits(EPSILON_100);
        let n = eps.len();
        assert!(approximate_entropy(&eps, 2).skipped());
        let ap_en = phi(&eps, 2, n) - phi(&eps, 3, n);
        let (chi_sq, p) = chi_square_and_p(ap_en, n, 2);
        assert!((ap_en - 0.665393).abs() < 1e-6, "ApEn = {ap_en}");
        assert!((chi_sq - 5.550792).abs() < 1e-6, "χ² = {chi_sq}");
        assert!((p - 0.235301).abs() < 1e-6, "p = {p}");
    }
}
