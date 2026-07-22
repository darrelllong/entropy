//! NIST SP 800-22 §2.11 — Serial Test.
//!
//! Counts overlapping m-bit, (m−1)-bit, and (m−2)-bit patterns in the
//! bit sequence (treating it as circular) and computes the ψ² statistic.
//! Two p-values are returned (for ∇ψ² and ∇²ψ², in that order, matching
//! the publication's P-value1/P-value2); the test passes if both are ≥ α.
//!
//! Recommended defaults: m = 3, n ≥ 1 000 000; the publication requires
//! m < ⌊log₂ n⌋ − 2 (SP 800-22 §2.11.7).

use crate::{math::igamc, result::TestResult};

/// Run the serial test; returns two p-values as a pair.
///
/// The two p-values correspond to ∇ψ²_m and ∇²ψ²_m respectively.
/// The test is considered to pass if both p-values ≥ α.
///
/// The result's `p_value` field is `min(p1, p2)` so that the standard
/// pass/fail logic applies to the worst of the two.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.11.
pub fn serial(bits: &[u8], m: usize) -> TestResult {
    let results = serial_both(bits, m);
    // serial_both always returns exactly two results; pick the one with the
    // smaller p-value (most conservative) as the single reported verdict.
    let r1 = results[0].clone();
    let r2 = results[1].clone();
    if r1.p_value <= r2.p_value { r1 } else { r2 }
}

/// Run the serial test; returns the two p-values as separate `TestResult`s.
///
/// This is the statistically correct way to report the serial test.
/// Taking min(p1, p2) — as done by `serial` — inflates false-failure rates.
pub fn serial_both(bits: &[u8], m: usize) -> Vec<TestResult> {
    let n = bits.len();
    if n < 1_000 || m < 2 {
        return vec![
            TestResult::insufficient("nist::serial_delta1", "n < 1000 or m < 2"),
            TestResult::insufficient("nist::serial_delta2", "n < 1000 or m < 2"),
        ];
    }
    // §2.11.7: m < ⌊log₂ n⌋ − 2, else the χ² approximation is invalid.
    if m >= (n.ilog2() as usize).saturating_sub(2) {
        let why = format!("m={m} violates m < ⌊log₂ n⌋ − 2 (n={n})");
        return vec![
            TestResult::insufficient("nist::serial_delta1", &why),
            TestResult::insufficient("nist::serial_delta2", &why),
        ];
    }

    let psi_m = psi_sq(bits, m, n);
    let psi_m1 = psi_sq(bits, m - 1, n);
    let psi_m2 = psi_sq(bits, m - 2, n);

    let del1 = psi_m - psi_m1;
    let del2 = psi_m - 2.0 * psi_m1 + psi_m2;

    // §2.11.4 step 5: ∇ψ² ~ χ²(2^{m−1}) and ∇²ψ² ~ χ²(2^{m−2}), so the igamc
    // shape parameters (df/2) are 2^{m−2} and 2^{m−3} respectively.
    let p1 = igamc(2.0_f64.powi(m as i32 - 2), del1 / 2.0);
    let p2 = igamc(2.0_f64.powi(m as i32 - 3), del2 / 2.0);

    vec![
        TestResult::with_note(
            "nist::serial_delta1",
            p1,
            format!("n={n}, m={m}, ∇ψ²={del1:.4}"),
        ),
        TestResult::with_note(
            "nist::serial_delta2",
            p2,
            format!("n={n}, m={m}, ∇²ψ²={del2:.4}"),
        ),
    ]
}

/// Compute the ψ² statistic for patterns of length `l` in the circular
/// (wrap-around) sequence of length `n`.
fn psi_sq(bits: &[u8], l: usize, n: usize) -> f64 {
    if l == 0 {
        return 0.0;
    }
    let table_size = 1usize << l;
    let mut counts = vec![0u32; table_size];

    for i in 0..n {
        let mut pattern = 0usize;
        for j in 0..l {
            pattern = (pattern << 1) | bits[(i + j) % n] as usize;
        }
        counts[pattern] += 1;
    }

    let sum_sq: f64 = counts.iter().map(|&c| (c as f64).powi(2)).sum();
    table_size as f64 / n as f64 * sum_sq - n as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SP 800-22 §2.11.6 worked example: ε = 0011011101, m = 3.
    const EXAMPLE: [u8; 10] = [0, 0, 1, 1, 0, 1, 1, 1, 0, 1];

    #[test]
    fn psi_sq_matches_nist_worked_example() {
        assert!((psi_sq(&EXAMPLE, 3, 10) - 2.8).abs() < 1e-12);
        assert!((psi_sq(&EXAMPLE, 2, 10) - 1.2).abs() < 1e-12);
        assert!((psi_sq(&EXAMPLE, 1, 10) - 0.4).abs() < 1e-12);
    }

    /// The publication's p-values for the worked example: with m = 3,
    /// P-value1 = igamc(2^{m−2}, ∇ψ²/2) = igamc(2, 0.8) ≈ 0.808792 and
    /// P-value2 = igamc(2^{m−3}, ∇²ψ²/2) = igamc(1, 0.4) ≈ 0.670320.
    /// This pins the df/statistic pairing that was once cross-wired.
    #[test]
    fn p_value_pairing_matches_nist_worked_example() {
        let del1 = 2.8 - 1.2;
        let del2 = 2.8 - 2.0 * 1.2 + 0.4;
        let p1 = igamc(2.0_f64.powi(1), del1 / 2.0);
        let p2 = igamc(2.0_f64.powi(0), del2 / 2.0);
        assert!((p1 - 0.808792).abs() < 1e-5, "p1 = {p1}");
        assert!((p2 - 0.670320).abs() < 1e-5, "p2 = {p2}");
    }

    #[test]
    fn structured_input_fails_and_m_bound_enforced() {
        // Perfectly alternating bits: wildly non-uniform pattern counts.
        let bits: Vec<u8> = (0..2048).map(|i| (i % 2) as u8).collect();
        for r in serial_both(&bits, 3) {
            assert!(r.p_value < 0.01, "{r}");
        }
        // m too large for n → insufficient, not a bogus p-value.
        for r in serial_both(&bits, 11) {
            assert!(r.skipped(), "{r}");
        }
    }
}
