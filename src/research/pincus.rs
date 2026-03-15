//! Pincus-style approximate entropy profile.
//!
//! Pincus (1991) treats approximate entropy as a family `ApEn(m, r, N)` rather
//! than a single fixed setting. For binary RNG streams the natural adaptation is
//! to sweep the embedding dimension `m` while keeping the exact circular
//! pattern-counting definition used by NIST for `r = 0`.

#[derive(Debug, Clone)]
pub struct ApproximateEntropyPoint {
    pub m: usize,
    pub phi_m: f64,
    pub phi_m1: f64,
    pub approximate_entropy: f64,
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

pub fn approximate_entropy_profile(bits: &[u8], m_values: &[usize]) -> Vec<ApproximateEntropyPoint> {
    let n = bits.len();
    m_values
        .iter()
        .copied()
        .filter(|&m| m > 0 && m < 30 && (1usize << m) <= n / 10)
        .map(|m| {
            let phi_m = phi(bits, m);
            let phi_m1 = phi(bits, m + 1);
            ApproximateEntropyPoint {
                m,
                phi_m,
                phi_m1,
                approximate_entropy: phi_m - phi_m1,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::approximate_entropy_profile;

    #[test]
    fn constant_stream_has_zero_profile() {
        let bits = vec![0u8; 1024];
        let profile = approximate_entropy_profile(&bits, &[2, 3, 4]);
        assert_eq!(3, profile.len());
        for point in profile {
            assert!(point.approximate_entropy.abs() < 1e-12);
        }
    }
}
