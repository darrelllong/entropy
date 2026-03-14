//! NIST SP 800-22 §2.11 — Serial Test.
//!
//! Counts overlapping m-bit, (m−1)-bit, and (m−2)-bit patterns in the
//! bit sequence (treating it as circular) and computes the ψ² statistic.
//! Two p-values are returned (for ∇²ψ² and ∇ψ²); the test passes if
//! both are ≥ α.
//!
//! Recommended defaults: m = 3, n ≥ 1 000 000 (SP 800-22 §2.11.7).

use crate::{math::igamc, result::TestResult};

/// Run the serial test; returns two p-values as a pair.
///
/// The two p-values correspond to ∇²ψ²_m and ∇ψ²_m respectively.
/// The test is considered to pass if both p-values ≥ α.
///
/// The result's `p_value` field is `min(p1, p2)` so that the standard
/// pass/fail logic applies to the worst of the two.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.11.
pub fn serial(bits: &[u8], m: usize) -> TestResult {
    let n = bits.len();
    if n < 1_000 || m < 2 {
        return TestResult::insufficient("nist::serial", "n < 1000 or m < 2");
    }

    let psi_m = psi_sq(bits, m, n);
    let psi_m1 = psi_sq(bits, m - 1, n);
    let psi_m2 = if m >= 2 { psi_sq(bits, m - 2, n) } else { 0.0 };

    let del2 = psi_m - 2.0 * psi_m1 + psi_m2;
    let del1 = psi_m - psi_m1;

    let p1 = igamc(2.0_f64.powi(m as i32 - 2), del2 / 2.0);
    let p2 = igamc(2.0_f64.powi(m as i32 - 1), del1 / 2.0);

    let p_min = p1.min(p2);

    TestResult::with_note(
        "nist::serial",
        p_min,
        format!("n={n}, m={m}, p1={p1:.4}, p2={p2:.4}"),
    )
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
