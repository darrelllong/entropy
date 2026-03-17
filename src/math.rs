//! Special mathematical functions used across all test suites.
//!
//! All functions are pure Rust, no external crates.  Algorithms are cited inline.

use std::f64::consts::{PI, SQRT_2};
use rustfft::{num_complex::Complex, FftPlanner};

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
    if x >= 0.0 {
        y
    } else {
        2.0 - y
    }
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
/// Returns `f64::NAN` if `a ≤ 0` or `x < 0`; callers treat `NAN` as an
/// insufficient-data result rather than a statistical verdict.
pub fn igamc(a: f64, x: f64) -> f64 {
    if !(a > 0.0 && x >= 0.0) {
        return f64::NAN;
    }
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
    for _ in 0..500 {
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
    for i in 1_u64..=500 {
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
/// Uses the exact/speedup hybrid from Dieharder's `kstest.c`, which in turn
/// ports G. Marsaglia, W. W. Tsang, J. Wang, "Evaluating Kolmogorov's
/// Distribution", *Journal of Statistical Software* 8(18), 2003.
///
/// # Preconditions
/// All elements must be finite and non-NaN.  The slice is sorted in place.
pub fn ks_test(samples: &mut [f64]) -> f64 {
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

const KS_EXACT_MAX_N: usize = 4_999;

/// P-value for the Kolmogorov-Smirnov statistic D with sample size n.
///
/// For moderate sample sizes, this uses the exact/speedup hybrid matrix method
/// from Dieharder's `p_ks_new()` (Marsaglia-Tsang-Wang 2003).  For large `n`
/// it falls back to the Stephens-corrected asymptotic Kolmogorov series.
///
/// Reference:
/// - Brown, R.G., `libdieharder/kstest.c`, Dieharder 3.31.1.
/// - Marsaglia, G., Tsang, W.W., Wang, J. (2003). Evaluating Kolmogorov's
///   Distribution. *Journal of Statistical Software* 8(18).
/// - Stephens, M.A. (1974). EDF Statistics for Goodness of Fit and Some
///   Comparisons. *JASA* 69(347), 730-737.
/// - Kolmogorov, A.N. (1933). Sulla determinazione empirica di una legge di
///   distribuzione. *Giornale dell'Istituto Italiano degli Attuari* 4, 83-91.
pub fn ks_pvalue(d: f64, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    if d <= 0.0 {
        return 1.0;
    }
    if d >= 1.0 {
        return 0.0;
    }
    if n <= KS_EXACT_MAX_N {
        return ks_pvalue_exact(d, n).clamp(0.0, 1.0);
    }
    ks_pvalue_asymptotic(d, n)
}

fn ks_pvalue_asymptotic(d: f64, n: usize) -> f64 {
    let nf = n as f64;
    // Stephens (1974) corrected argument: reduces error from O(1/√n) to O(1/n).
    let s = d * (nf.sqrt() + 0.12 + 0.11 / nf.sqrt());
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

fn ks_pvalue_exact(d: f64, n: usize) -> f64 {
    let nf = n as f64;
    let s = d * d * nf;
    // Dieharder's fast right-tail fallback inside the "exact" path.
    if s > 7.24 || (s > 3.76 && n > 99) {
        return 2.0 * (-(2.000_071 + 0.331 / nf.sqrt() + 1.409 / nf) * s).exp();
    }

    let k = (nf * d).floor() as usize + 1;
    let m = 2 * k - 1;
    let h = k as f64 - nf * d;

    let mut hmat = vec![0.0; m * m];
    for i in 0..m {
        for j in 0..m {
            if i + 1 >= j + 1 {
                hmat[i * m + j] = 1.0;
            }
        }
    }

    for i in 0..m {
        hmat[i * m] -= h.powi((i + 1) as i32);
        hmat[(m - 1) * m + i] -= h.powi((m - i) as i32);
    }
    if 2.0 * h - 1.0 > 0.0 {
        hmat[(m - 1) * m] += (2.0 * h - 1.0).powi(m as i32);
    }

    for i in 0..m {
        for j in 0..m {
            let span = i as isize - j as isize + 1;
            if span > 0 {
                let mut denom = 1.0;
                for g in 1..=span as usize {
                    denom *= g as f64;
                }
                hmat[i * m + j] /= denom;
            }
        }
    }

    let (q, mut exponent) = matrix_power_scaled(&hmat, m, n);
    let idx = (k - 1) * m + (k - 1);
    let mut prob = q[idx];
    for i in 1..=n {
        prob *= i as f64 / nf;
        if prob < 1e-140 {
            prob *= 1e140;
            exponent -= 140;
        }
    }
    prob *= 10f64.powi(exponent);
    (1.0 - prob).clamp(0.0, 1.0)
}

fn matrix_power_scaled(a: &[f64], m: usize, power: usize) -> (Vec<f64>, i32) {
    if power == 1 {
        return (a.to_vec(), 0);
    }

    let (half_power, half_exp) = matrix_power_scaled(a, m, power / 2);
    let mut squared = matrix_multiply(&half_power, &half_power, m);
    let mut exponent = 2 * half_exp;

    if power % 2 == 1 {
        squared = matrix_multiply(a, &squared, m);
    }

    renormalize_matrix(&mut squared, &mut exponent);
    (squared, exponent)
}

fn matrix_multiply(a: &[f64], b: &[f64], m: usize) -> Vec<f64> {
    let mut c = vec![0.0; m * m];
    for i in 0..m {
        for j in 0..m {
            let mut sum = 0.0;
            for k in 0..m {
                sum += a[i * m + k] * b[k * m + j];
            }
            c[i * m + j] = sum;
        }
    }
    c
}

fn renormalize_matrix(v: &mut [f64], exponent: &mut i32) {
    if !v.iter().any(|x| x.abs() > 1.0e140) {
        return;
    }
    for x in v.iter_mut() {
        *x *= 1.0e-140;
    }
    *exponent += 140;

    if v.iter().all(|&x| x == 0.0) {
        return;
    }
    if v.iter().all(|x| x.abs() < 1.0e-140) {
        v.iter_mut().for_each(|x| *x *= 1.0e140);
        *exponent -= 140;
    }
}

// ── Chi-square p-value (convenience) ─────────────────────────────────────────

/// Chi-square survival function: P(χ²_{df} > chi_sq) = igamc(df/2, chi_sq/2).
pub fn chi2_pvalue(chi_sq: f64, df: usize) -> f64 {
    igamc(df as f64 / 2.0, chi_sq / 2.0)
}

// ── Discrete Fourier Transform (DFT) ─────────────────────────────────────────

/// FFT for a real input of arbitrary length n.
/// Returns magnitudes |X_k| for k = 0..n.
///
/// Uses `rustfft` so the NIST spectral test can analyze the full sequence
/// length instead of truncating to a radix-2 prefix.
pub fn fft_magnitudes(x: &[f64]) -> Vec<f64> {
    let n = x.len();
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(n);
    let mut buffer: Vec<Complex<f64>> = x.iter().map(|&re| Complex { re, im: 0.0 }).collect();
    fft.process(&mut buffer);
    buffer.into_iter().map(|c| c.norm()).collect()
}

/// Naïve O(n²) DFT of a real sequence, returning magnitudes |X_k| for k = 0..n.
///
/// For the NIST spectral test (SP 800-22 §2.6) the recommended sequence length
/// is n = 1 000, making the O(n²) cost ~10⁶ multiplications — negligible.
pub fn dft_magnitudes(x: &[f64]) -> Vec<f64> {
    let n = x.len();
    let two_pi_over_n = 2.0 * PI / n as f64;
    (0..n)
        .map(|k| {
            let (re, im) = x
                .iter()
                .enumerate()
                .fold((0.0_f64, 0.0_f64), |(re, im), (j, &xj)| {
                    let angle = two_pi_over_n * (k * j) as f64;
                    (re + xj * angle.cos(), im - xj * angle.sin())
                });
            (re * re + im * im).sqrt()
        })
        .collect()
}

// ── GF(2) rank ────────────────────────────────────────────────────────────────

/// GF(2) rank of a binary matrix via Gaussian elimination, shared between
/// the NIST SP 800-22 matrix-rank test and the DIEHARD binary-rank test.
///
/// `matrix` is a slice of `rows` packed u32 row-words; bit `c` of `matrix[r]`
/// is the entry at row `r`, column `c`.  Only the low `cols` bits of each word
/// are used (caller must mask if needed).
///
/// Time: O(rows × cols × min(rows,cols)).
pub fn gf2_rank(matrix: &[u32], rows: usize, cols: usize) -> usize {
    let mut m = matrix.to_vec();
    let mut rank = 0usize;
    let mut pivot_row = 0usize;

    for col in 0..cols {
        let found = (pivot_row..rows).find(|&r| (m[r] >> col) & 1 == 1);
        if let Some(r) = found {
            m.swap(pivot_row, r);
            rank += 1;
            let pivot = m[pivot_row];
            for (r, row) in m.iter_mut().enumerate().take(rows) {
                if r != pivot_row && (*row >> col) & 1 == 1 {
                    *row ^= pivot;
                }
            }
            pivot_row += 1;
        }
    }
    rank
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

    #[test]
    fn ks_pvalue_respects_boundaries() {
        assert_eq!(ks_pvalue(0.0, 10), 1.0);
        assert_eq!(ks_pvalue(1.0, 10), 0.0);
    }
}
