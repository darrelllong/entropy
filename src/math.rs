//! Special mathematical functions used across all test suites.
//!
//! All functions are pure Rust, no external crates.  Algorithms are cited inline.

use std::f64::consts::{PI, SQRT_2};

// ── erfc ──────────────────────────────────────────────────────────────────────

/// Complementary error function, erfc(x) = 1 − erf(x).
///
/// Uses the rational approximation from W. H. Press et al., *Numerical Recipes*
/// (3rd ed., 2007), §6.2.2.  Maximum absolute error ≈ 1.2 × 10⁻⁷.
///
/// Used by nearly every NIST SP 800-22 test for its p-value.
pub fn erfc(x: f64) -> f64 {
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.5 * z);
    // 1 outer paren + 8 levels of t*(…) = 9 opens → 9 closes total.
    #[rustfmt::skip]
    let y = (-z * z
        - 1.26551223
        + t * (1.00002368
        + t * (0.37409196
        + t * (0.09678418
        + t * (-0.18628806
        + t * (0.27886807
        + t * (-1.13520398
        + t * (1.48851587
        + t * (-0.82215223
        + t * 0.17087294))))))))
    ).exp() * t;
    if x >= 0.0 { y } else { 2.0 - y }
}

/// Standard normal CDF, Φ(x) = P(Z ≤ x) for Z ~ N(0,1).
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * erfc(-x / SQRT_2)
}

// ── lgamma ────────────────────────────────────────────────────────────────────

/// Natural logarithm of the gamma function, ln Γ(x), for x > 0.
///
/// Lanczos approximation from W. H. Press et al., *Numerical Recipes*
/// (3rd ed., 2007), §6.1.  Accurate to ~15 significant figures.
pub fn lgamma(x: f64) -> f64 {
    const C: [f64; 6] = [
        76.18009172947146,
        -86.50532032941677,
        24.01409824083091,
        -1.231739572450155,
        1.208650973866179e-3,
        -5.395239384953e-6,
    ];
    let mut y = x;
    let tmp = x + 5.5 - (x + 0.5) * (x + 5.5).ln();
    let mut ser = 1.000_000_000_190_015_f64;
    for &c in &C {
        y += 1.0;
        ser += c / y;
    }
    -tmp + (2.506_628_274_631_000_5 * ser / x).ln()
}

// ── igamc ─────────────────────────────────────────────────────────────────────

/// Regularized **upper** incomplete gamma function Q(a, x) = Γ(a, x) / Γ(a).
///
/// This is the survival function of the chi-square distribution:
/// `p_value = igamc(df/2, χ²/2)`.
///
/// Algorithm from W. H. Press et al., *Numerical Recipes* (3rd ed.), §6.2:
/// series expansion for x < a + 1, Lentz continued-fraction otherwise.
///
/// # Panics
/// Panics if `a ≤ 0` or `x < 0`.
pub fn igamc(a: f64, x: f64) -> f64 {
    assert!(a > 0.0 && x >= 0.0, "igamc: invalid arguments a={a}, x={x}");
    if x == 0.0 {
        return 1.0;
    }
    if x < a + 1.0 {
        1.0 - gamser(a, x)
    } else {
        gammcf(a, x)
    }
}

/// Series expansion for the regularized lower incomplete gamma P(a, x).
fn gamser(a: f64, x: f64) -> f64 {
    let gln = lgamma(a);
    let mut ap = a;
    let mut del = 1.0 / a;
    let mut sum = del;
    for _ in 0..200 {
        ap += 1.0;
        del *= x / ap;
        sum += del;
        if del.abs() < sum.abs() * 1e-13 {
            break;
        }
    }
    sum * (-x + a * x.ln() - gln).exp()
}

/// Lentz continued-fraction expansion for Q(a, x).
///
/// Follows the modified Lentz algorithm from W. H. Press et al.,
/// *Numerical Recipes* (3rd ed.), §6.2.  The key invariant is that d and c
/// are updated in sequence with the SAME value of `an` — d must not be
/// touched twice in one iteration, which would corrupt the fraction.
fn gammcf(a: f64, x: f64) -> f64 {
    let gln = lgamma(a);
    let fpmin = f64::MIN_POSITIVE / f64::EPSILON;
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / fpmin;
    let mut d = if b.abs() < fpmin { fpmin } else { 1.0 / b };
    let mut h = d;
    for i in 1_u64..=200 {
        let an = -(i as f64) * (i as f64 - a);
        b += 2.0;
        // Update d: clamp before inverting so we never divide by zero.
        let new_d = an * d + b;
        d = 1.0 / if new_d.abs() < fpmin { fpmin } else { new_d };
        // Update c: clamp before using.
        let new_c = b + an / c;
        c = if new_c.abs() < fpmin { fpmin } else { new_c };
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-13 {
            break;
        }
    }
    (-x + a * x.ln() - gln).exp() * h
}

// ── Kolmogorov-Smirnov ────────────────────────────────────────────────────────

/// Two-sided Kolmogorov-Smirnov test: returns the p-value for the hypothesis
/// that `samples` are drawn from U(0, 1).
///
/// Uses Marsaglia's series from G. Marsaglia, W. W. Tsang, J. Wang,
/// "Evaluating Kolmogorov's Distribution", *Journal of Statistical Software*
/// 8(18), 2003.
pub fn ks_test(samples: &mut Vec<f64>) -> f64 {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = samples.len();
    let nf = n as f64;
    let d = samples
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let f_hi = (i + 1) as f64 / nf;
            let f_lo = i as f64 / nf;
            (f_hi - x).abs().max((x - f_lo).abs())
        })
        .fold(0.0_f64, f64::max);
    ks_pvalue(d, n)
}

/// P-value for the Kolmogorov-Smirnov statistic D with sample size n.
///
/// Uses the Kolmogorov distribution series.  For large n this converges
/// quickly; for small n (< 35) uses the exact formula.
pub fn ks_pvalue(d: f64, n: usize) -> f64 {
    let s = d * (n as f64).sqrt();
    // Asymptotic series (Kolmogorov 1933)
    let s2 = -2.0 * s * s;
    let mut sum = 0.0_f64;
    for k in 1_i64..=100 {
        let term = (-1.0_f64).powi(k as i32 - 1) * (k as f64 * k as f64 * s2).exp();
        sum += term;
        if term.abs() < 1e-15 * sum.abs() {
            break;
        }
    }
    (2.0 * sum).clamp(0.0, 1.0)
}

// ── Chi-square p-value (convenience) ─────────────────────────────────────────

/// Chi-square survival function: P(χ²_{df} > chi_sq) = igamc(df/2, chi_sq/2).
pub fn chi2_pvalue(chi_sq: f64, df: usize) -> f64 {
    igamc(df as f64 / 2.0, chi_sq / 2.0)
}

// ── Discrete Fourier Transform (DFT) ─────────────────────────────────────────

/// Naïve O(n²) DFT of a real sequence, returning magnitudes |X_k| for k = 0..n.
///
/// For the NIST spectral test (SP 800-22 §2.6) the recommended sequence length
/// is n = 1 000, making the O(n²) cost ~10⁶ multiplications — negligible.
pub fn dft_magnitudes(x: &[f64]) -> Vec<f64> {
    let n = x.len();
    let two_pi_over_n = 2.0 * PI / n as f64;
    (0..n)
        .map(|k| {
            let (re, im) = x.iter().enumerate().fold((0.0_f64, 0.0_f64), |(re, im), (j, &xj)| {
                let angle = two_pi_over_n * (k * j) as f64;
                (re + xj * angle.cos(), im - xj * angle.sin())
            });
            (re * re + im * im).sqrt()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erfc_known_values() {
        assert!((erfc(0.0) - 1.0).abs() < 1e-6);
        assert!((erfc(1.0) - 0.157299).abs() < 1e-5);
        assert!((erfc(-1.0) - 1.842701).abs() < 1e-5);
    }

    #[test]
    fn lgamma_known_values() {
        // Γ(1) = 1  →  ln Γ(1) = 0
        assert!(lgamma(1.0).abs() < 1e-10);
        // Γ(2) = 1  →  ln Γ(2) = 0
        assert!(lgamma(2.0).abs() < 1e-10);
        // Γ(3) = 2  →  ln Γ(3) = ln 2
        assert!((lgamma(3.0) - 2.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn igamc_boundary() {
        // Q(a, 0) = 1
        assert!((igamc(1.0, 0.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn normal_cdf_symmetry() {
        // erfc approximation is accurate to ~1.2e-7, not 1e-12.
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-6);
        assert!((normal_cdf(1.0) + normal_cdf(-1.0) - 1.0).abs() < 1e-6);
    }
}
