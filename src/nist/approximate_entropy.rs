//! NIST SP 800-22 §2.12 — Approximate Entropy Test.
//!
//! Computes the approximate entropy ApEn(m) = φ(m) − φ(m+1), where φ(m)
//! is the average log-count of overlapping m-bit patterns in the circular
//! sequence.  A small ApEn indicates the sequence is more regular than
//! expected for random data.
//!
//! Recommended defaults: m = 10 for n ≥ 10^6 (SP 800-22 §2.12.7).

use crate::{math::igamc, result::TestResult};
use std::f64::consts::LN_2;

/// Run the approximate entropy test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.12.
pub fn approximate_entropy(bits: &[u8], m: usize) -> TestResult {
    let n = bits.len();
    if m >= 30 || (1usize << m) > n / 10 {
        return TestResult::insufficient("nist::approximate_entropy", "m too large for n");
    }

    let phi_m  = phi(bits, m,     n);
    let phi_m1 = phi(bits, m + 1, n);

    let ap_en = phi_m - phi_m1;

    let chi_sq = 2.0 * n as f64 * (LN_2 - ap_en);
    let p_value = igamc(2.0_f64.powi(m as i32 - 1), chi_sq / 2.0);

    TestResult::with_note(
        "nist::approximate_entropy",
        p_value,
        format!("n={n}, m={m}, ApEn={ap_en:.6}, χ²={chi_sq:.4}"),
    )
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
