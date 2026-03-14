//! NIST SP 800-22 §2.10 — Linear Complexity Test.
//!
//! Divides the sequence into M-bit blocks, computes the linear complexity
//! (shortest LFSR length) of each block via the Berlekamp-Massey algorithm,
//! and tests whether the distribution of complexities is consistent with an
//! i.i.d. Bernoulli(½) source.
//!
//! Recommended defaults: M = 500, n ≥ 10^6.

use crate::{math::igamc, result::TestResult};

/// Run the linear complexity test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.10.
pub fn linear_complexity(bits: &[u8], m: usize) -> TestResult {
    if m < 500 || m > 5000 {
        return TestResult::insufficient("nist::linear_complexity", "M must be in [500, 5000]");
    }

    let n = bits.len();
    let num_blocks = n / m;

    if num_blocks < 200 {
        return TestResult::insufficient(
            "nist::linear_complexity",
            "n too small — need ≥ 200 blocks",
        );
    }

    // Theoretical mean μ = M/2 + (9 + r)/36  where r = M mod 2.
    let r = (m % 2) as f64;
    let mu = m as f64 / 2.0 + (9.0 + r) / 36.0 - (m as f64 / 3.0 + 2.0 / 9.0) / 2f64.powi(m as i32);

    // Category boundaries for T = (−1)^M (L − μ) + 2/9.
    // Six categories: T ≤ −2.5, (−2.5,−1.5], (−1.5,−0.5], (−0.5,0.5],
    //                 (0.5,1.5], (1.5,2.5], T > 2.5  (7 categories total).
    let pi = [0.010417, 0.031250, 0.125000, 0.500000, 0.250000, 0.062500, 0.020833];

    let mut nu = [0usize; 7];
    let sign = if m % 2 == 0 { 1.0 } else { -1.0 };

    for block in bits.chunks_exact(m).take(num_blocks) {
        let l = berlekamp_massey(block) as f64;
        let t = sign * (l - mu) + 2.0 / 9.0;
        let idx = if t <= -2.5 {
            0
        } else if t <= -1.5 {
            1
        } else if t <= -0.5 {
            2
        } else if t <= 0.5 {
            3
        } else if t <= 1.5 {
            4
        } else if t <= 2.5 {
            5
        } else {
            6
        };
        nu[idx] += 1;
    }

    let chi_sq: f64 = nu
        .iter()
        .zip(pi.iter())
        .map(|(&count, &p)| {
            let exp = num_blocks as f64 * p;
            (count as f64 - exp).powi(2) / exp
        })
        .sum();

    let p_value = igamc(3.0, chi_sq / 2.0); // df = 6

    TestResult::with_note(
        "nist::linear_complexity",
        p_value,
        format!("n={n}, M={m}, N={num_blocks}, χ²={chi_sq:.4}"),
    )
}

/// Berlekamp-Massey algorithm: returns the linear complexity (shortest LFSR
/// length) of the binary sequence `s`.
///
/// This is the reference implementation from NIST SP 800-22 Appendix A,
/// transcribed into safe Rust.
pub fn berlekamp_massey(s: &[u8]) -> usize {
    let big_n = s.len();
    let mut c = vec![0u8; big_n + 1];
    let mut b = vec![0u8; big_n + 1];
    c[0] = 1;
    b[0] = 1;
    let mut l = 0usize;
    let mut m: i64 = -1;

    for n in 0..big_n {
        // Discrepancy d = s[n] XOR (Σ c[i]·s[n−i] for i = 1..=L)
        let mut d = s[n];
        for i in 1..=l {
            d ^= c[i] & s[n - i];
        }
        if d == 0 {
            continue;
        }
        let t = c.clone();
        let shift = (n as i64 - m) as usize;
        for i in shift..=big_n {
            c[i] ^= b[i - shift];
        }
        if 2 * l <= n {
            l = n + 1 - l;
            b = t;
            m = n as i64;
        }
    }
    l
}
