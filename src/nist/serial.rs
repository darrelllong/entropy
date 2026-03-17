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

    let psi_m = psi_sq(bits, m, n);
    let psi_m1 = psi_sq(bits, m - 1, n);
    let psi_m2 = if m >= 2 { psi_sq(bits, m - 2, n) } else { 0.0 };

    let del2 = psi_m - 2.0 * psi_m1 + psi_m2;
    let del1 = psi_m - psi_m1;

    let p1 = igamc(2.0_f64.powi(m as i32 - 2), del2 / 2.0);
    let p2 = igamc(2.0_f64.powi(m as i32 - 1), del1 / 2.0);

    vec![
        TestResult::with_note(
            "nist::serial_delta1",
            p1,
            format!("n={n}, m={m}, ∇²ψ²={del2:.4}"),
        ),
        TestResult::with_note(
            "nist::serial_delta2",
            p2,
            format!("n={n}, m={m}, ∇ψ²={del1:.4}"),
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
