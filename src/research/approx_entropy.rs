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

/// One row of the ApEn profile: the φ statistics and their difference at a
/// single embedding dimension `m`.
#[derive(Debug, Clone)]
pub struct ApproxEntropyPoint {
    /// Embedding dimension (pattern length in bits).
    pub m: usize,
    /// φ(m): the circular pattern-counting statistic at length `m`.
    pub phi_m: f64,
    /// φ(m+1): the same statistic at length `m + 1`.
    pub phi_m1: f64,
    /// ApEn(m) = φ(m) − φ(m+1); ≈ ln 2 for a random sequence, smaller
    /// when the sequence is more regular than chance.
    pub ap_en: f64,
}

fn phi(bits: &[u8], m: usize) -> f64 {
    let n = bits.len();
    let table_size = 1usize << m;
    // u64: a u32 count would wrap once a pattern occurs 2^32 times.
    let mut counts = vec![0u64; table_size];

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
/// Values of `m` are silently skipped unless `0 < m < 30` and `m` meets the
/// §2.12.7 input size recommendation, "Choose m and n such that
/// m < ⌊log2 n⌋ − 5", i.e. `n ≥ 2^(m+6)`.
pub fn approx_entropy_profile(bits: &[u8], m_values: &[usize]) -> Vec<ApproxEntropyPoint> {
    let n = bits.len();
    m_values
        .iter()
        .copied()
        .filter(|&m| m > 0 && m < 30 && (1u64 << (m + 6)) <= n as u64)
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
    use super::{approx_entropy_profile, phi};

    /// §2.12.7 reads "m < ⌊log2 n⌋ − 5", so n = 2^(m+6) − 1 is the last length
    /// that must be skipped (n = 129 rules out m = 2).
    #[test]
    fn m_gate_follows_sp800_22_input_size_recommendation() {
        for m in [2usize, 3, 6] {
            let boundary = 1usize << (m + 6);
            assert!(
                approx_entropy_profile(&vec![0u8; boundary - 1], &[m]).is_empty(),
                "m = {m}: n = 2^(m+6) - 1 must be skipped"
            );
            assert_eq!(
                1,
                approx_entropy_profile(&vec![0u8; boundary], &[m]).len(),
                "m = {m}: n = 2^(m+6) must run"
            );
        }
        assert!(approx_entropy_profile(&[0u8; 129], &[2]).is_empty());
    }

    /// SP 800-22 §2.12.4 worked example: ε = 0100110101 gives
    /// φ(3) = −1.64341772 and φ(4) = −1.83437197.
    #[test]
    fn phi_matches_sp800_22_worked_example() {
        let bits = [0u8, 1, 0, 0, 1, 1, 0, 1, 0, 1];
        assert!((phi(&bits, 3) + 1.643_417_72).abs() < 1e-8);
        assert!((phi(&bits, 4) + 1.834_371_97).abs() < 1e-8);
    }

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
