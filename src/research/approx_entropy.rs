//! Multi-scale approximate entropy profile.
//!
//! The NIST SP 800-22 Rev. 1a §2.12 approximate entropy test computes
//! `ApEn(m) = φ(m) − φ(m+1)` at a single fixed embedding dimension `m = 10`.
//! This module sweeps `m` over a caller-supplied range, producing a profile
//! that reveals at which pattern lengths the sequence departs from randomness.
//!
//! The `φ(m)` statistic is the circular pattern-counting formula defined in
//! NIST SP 800-22 Rev. 1a §2.12.4 (step 3):
//!
//! ```text
//! φ(m) = (1/n) Σ_{pattern p} C_p · ln(C_p / n)
//! ```
//!
//! where `C_p` is the count of occurrences of pattern `p` in the circular
//! bit stream.  A small `ApEn(m)` at a given `m` means the sequence is more
//! regular than expected; a value near `ln 2` indicates randomness at that
//! scale.
//!
//! # References
//! NIST SP 800-22 Rev. 1a, §2.12 — "Approximate Entropy Test", 2010.
//! [pubs/NIST-SP-800-22r1a.pdf]
//!
//! # Author
//! NIST (specification); Darrell Long (Rust implementation).

#[derive(Debug, Clone)]
pub struct ApproxEntropyPoint {
    pub m: usize,
    pub phi_m: f64,
    pub phi_m1: f64,
    pub ap_en: f64,
}

fn phi(bits: &[u8], m: usize) -> f64 {
    let n = bits.len();
    let table_size = 1usize << m;
    let mut counts = vec![0u32; table_size];

    for i in 0..n {
        let mut pattern = 0usize;
        for j in 0..m {
            pattern = (pattern << 1) | bits[(i + j) % n] as usize;
        }
        counts[pattern] += 1;
    }

    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let cf = c as f64;
            cf * (cf / n as f64).ln()
        })
        .sum::<f64>()
        / n as f64
}

/// Compute the NIST SP 800-22 §2.12 ApEn statistic at each value in
/// `m_values`, returning one [`ApproxEntropyPoint`] per valid `m`.
///
/// Values of `m` are silently skipped when `2^m > n/10` (too few samples
/// to populate the pattern table reliably).
pub fn approx_entropy_profile(bits: &[u8], m_values: &[usize]) -> Vec<ApproxEntropyPoint> {
    let n = bits.len();
    m_values
        .iter()
        .copied()
        .filter(|&m| m > 0 && m < 30 && (1usize << m) <= n / 10)
        .map(|m| {
            let phi_m = phi(bits, m);
            let phi_m1 = phi(bits, m + 1);
            ApproxEntropyPoint {
                m,
                phi_m,
                phi_m1,
                ap_en: phi_m - phi_m1,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::approx_entropy_profile;

    #[test]
    fn constant_stream_has_zero_profile() {
        let bits = vec![0u8; 1024];
        let profile = approx_entropy_profile(&bits, &[2, 3, 4]);
        assert_eq!(3, profile.len());
        for point in profile {
            assert!(point.ap_en.abs() < 1e-12);
        }
    }
}
