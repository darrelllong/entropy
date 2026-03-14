//! NIST SP 800-22 §2.13 — Cumulative Sums (Cusum) Test.
//!
//! Converts bits to a ±1 sequence, forms the running sum (random walk), and
//! tests whether the maximum excursion from zero is too large or too small
//! relative to what is expected for a truly random bit sequence.
//!
//! Two variants are tested: forward (left-to-right) and backward
//! (right-to-left).
//!
//! Minimum recommended: n ≥ 100.

use crate::{math::normal_cdf, result::TestResult};

/// Run the forward (mode=0) cumulative sums test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.13.
pub fn cumulative_sums_forward(bits: &[u8]) -> TestResult {
    cusum(bits, false, "nist::cumulative_sums_forward")
}

/// Run the backward (mode=1) cumulative sums test.
pub fn cumulative_sums_backward(bits: &[u8]) -> TestResult {
    cusum(bits, true, "nist::cumulative_sums_backward")
}

fn cusum(bits: &[u8], reverse: bool, name: &'static str) -> TestResult {
    let n = bits.len();
    if n < 100 {
        return TestResult::insufficient(name, "n < 100");
    }

    let seq: Vec<i64> = if reverse {
        bits.iter().rev().map(|&b| if b == 1 { 1 } else { -1 }).collect()
    } else {
        bits.iter().map(|&b| if b == 1 { 1 } else { -1 }).collect()
    };

    // Maximum absolute value of the running sum.
    let mut running = 0i64;
    let z = seq
        .iter()
        .map(|&x| {
            running += x;
            running.unsigned_abs()
        })
        .max()
        .unwrap_or(0) as f64;

    let p_value = cusum_pvalue(z, n);

    TestResult::with_note(name, p_value, format!("n={n}, z={z}"))
}

/// P-value formula from SP 800-22 §2.13.4.
fn cusum_pvalue(z: f64, n: usize) -> f64 {
    let nf = n as f64;
    let sqrt_n = nf.sqrt();

    // Sum 1: k from floor((−n/z+1)/4) to floor((n/z−1)/4)
    let k1_lo = ((-nf / z + 1.0) / 4.0).floor() as i64;
    let k1_hi = ((nf / z - 1.0) / 4.0).floor() as i64;

    let sum1: f64 = (k1_lo..=k1_hi)
        .map(|k| {
            let kf = k as f64;
            normal_cdf((4.0 * kf + 1.0) * z / sqrt_n)
                - normal_cdf((4.0 * kf - 1.0) * z / sqrt_n)
        })
        .sum();

    // Sum 2: k from floor((−n/z−3)/4) to floor((n/z−1)/4)
    let k2_lo = ((-nf / z - 3.0) / 4.0).floor() as i64;
    let k2_hi = ((nf / z - 1.0) / 4.0).floor() as i64;

    let sum2: f64 = (k2_lo..=k2_hi)
        .map(|k| {
            let kf = k as f64;
            normal_cdf((4.0 * kf + 3.0) * z / sqrt_n)
                - normal_cdf((4.0 * kf + 1.0) * z / sqrt_n)
        })
        .sum();

    1.0 - sum1 + sum2
}
