//! Special mathematical functions used across all test suites.
//!
//! All functions are pure Rust, no external crates.  Algorithms are cited inline.

use rustfft::{num_complex::Complex, FftPlanner};
use std::f64::consts::{PI, SQRT_2};

// ── erfc ──────────────────────────────────────────────────────────────────────

/// Complementary error function, erfc(x) = 1 − erf(x).
///
/// Uses the Chebyshev-fitted rational approximation (`erfcc`) from W. H. Press
/// et al., *Numerical Recipes in C* (2nd ed., 1992), §6.2.  Fractional error
/// below 1.2 × 10⁻⁷ per NR; measured absolute error ≤ 2 × 10⁻⁷ near x = 0.
/// Ample for α = 0.01 verdicts, but very small p-values carry only ~7 digits.
///
/// Used by nearly every NIST SP 800-22 test for its p-value.
#[must_use]
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
#[must_use]
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * erfc(-x / SQRT_2)
}

// ── lgamma ────────────────────────────────────────────────────────────────────

/// Natural logarithm of the gamma function, ln Γ(x), for x > 0.
///
/// Six-coefficient Lanczos approximation (`gammln`) from W. H. Press et al.,
/// *Numerical Recipes in C* (2nd ed., 1992), §6.1.  Relative error in ln Γ
/// is below 2 × 10⁻¹⁰ per NR (~10 significant figures in Γ).
#[must_use]
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
/// Returns `f64::NAN` if `a ≤ 0`, `x < 0`, or the expansion fails to converge
/// (astronomically large `a`); callers treat `NAN` as an insufficient-data
/// result rather than a statistical verdict.
#[must_use]
pub fn igamc(a: f64, x: f64) -> f64 {
    if !(a > 0.0 && x >= 0.0) {
        return f64::NAN;
    }
    if x == 0.0 {
        return 1.0;
    }
    if x.is_infinite() {
        return 0.0;
    }
    if x < a + 1.0 {
        gamser(a, x).map_or(f64::NAN, |p| 1.0 - p)
    } else {
        gammcf(a, x).unwrap_or(f64::NAN)
    }
}

/// Iteration budget for the igamc expansions.  Both the series and the
/// continued fraction need O(√a) terms when x ≈ a (the chi-square bulk),
/// so a fixed cap silently loses accuracy for large df: at a = 10⁵ a
/// 500-iteration cap yields ~11% relative error.  `None` on non-convergence.
fn igamc_max_iter(a: f64) -> u64 {
    500 + (10.0 * a.sqrt()) as u64
}

/// Series expansion for the regularized lower incomplete gamma P(a, x).
fn gamser(a: f64, x: f64) -> Option<f64> {
    let gln = lgamma(a);
    let mut ap = a;
    let mut del = 1.0 / a;
    let mut sum = del;
    for _ in 0..igamc_max_iter(a) {
        ap += 1.0;
        del *= x / ap;
        sum += del;
        if del.abs() < sum.abs() * 1e-13 {
            return Some(sum * (-x + a * x.ln() - gln).exp());
        }
    }
    None
}

/// Lentz continued-fraction expansion for Q(a, x).
///
/// Follows the modified Lentz algorithm from W. H. Press et al.,
/// *Numerical Recipes* (3rd ed.), §6.2.  The key invariant is that d and c
/// are updated in sequence with the SAME value of `an` — d must not be
/// touched twice in one iteration, which would corrupt the fraction.
fn gammcf(a: f64, x: f64) -> Option<f64> {
    let gln = lgamma(a);
    let fpmin = f64::MIN_POSITIVE / f64::EPSILON;
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / fpmin;
    // Lentz convention: clamp the DENOMINATOR to fpmin, then invert.
    // (Unreachable here — the x ≥ a + 1 branch guarantees b ≥ 2.)
    let mut d = 1.0 / if b.abs() < fpmin { fpmin } else { b };
    let mut h = d;
    for i in 1_u64..=igamc_max_iter(a) {
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
            return Some((-x + a * x.ln() - gln).exp() * h);
        }
    }
    None
}

// ── Kolmogorov-Smirnov ────────────────────────────────────────────────────────

/// Two-sided Kolmogorov-Smirnov test: returns the p-value for the hypothesis
/// that `samples` are drawn from U(0, 1).
///
/// Uses the exact/speedup hybrid from Dieharder's `kstest.c`, which in turn
/// ports G. Marsaglia, W. W. Tsang, J. Wang, "Evaluating Kolmogorov's
/// Distribution", *Journal of Statistical Software* 8(18), 2003.
///
/// Returns NaN, the crate's insufficient-data value, if any sample is NaN:
/// such a sample has no place in the empirical distribution.  Otherwise the
/// slice is sorted in place.
#[must_use]
pub fn ks_test(samples: &mut [f64]) -> f64 {
    if samples.iter().any(|x| x.is_nan()) {
        return f64::NAN;
    }
    samples.sort_by(f64::total_cmp);
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
#[must_use]
pub fn ks_pvalue(d: f64, n: usize) -> f64 {
    if n == 0 {
        // No sample → no verdict.  NaN follows the crate's insufficient-data
        // convention (a 0.0 here would report a hard FAIL on empty input).
        return f64::NAN;
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
    let mut converged = false;
    for k in 1_i64..=100 {
        let term = (-1.0_f64).powi(k as i32 - 1) * (k as f64 * k as f64 * s2).exp();
        sum += term;
        // `<=` so that a term underflowing to exactly 0 (huge s: every term
        // vanishes, sum stays 0, true p ≈ 0) counts as converged; the strict
        // `<` never fired there (0 < 0) and misrouted huge-s inputs into the
        // small-s fallback below, reporting p = 1 for catastrophic D.
        if term.abs() <= 1e-15 * sum.abs() {
            converged = true;
            break;
        }
    }
    if !converged {
        // Terms decay like exp(−2k²s²); for very small s (near-superuniform
        // samples) the terms stay O(1) and 100 terms are not enough — but
        // there the true p ≈ 1.
        return 1.0;
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

    // H[i][j] = 1 wherever i − j + 1 ≥ 0: lower triangle PLUS the first
    // superdiagonal (j = i + 1, where the later (i − j + 1)! divisor is 0! = 1).
    // Omitting the superdiagonal makes H triangular and collapses the exact
    // p-value to 1 − n!/nⁿ for every D.
    let mut hmat = vec![0.0; m * m];
    for i in 0..m {
        for j in 0..m {
            if i + 1 >= j {
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
}

// ── Anderson–Darling ──────────────────────────────────────────────────────────

/// Above this `z`, `ad_inf` takes the limiting Anderson–Darling distribution
/// to be exactly 1.
const AD_INF_Z_MAX: f64 = 30.0;

/// Once ADinf(z) exceeds this (z > 6.6127), [`anderson_darling_cdf`] stops
/// evaluating errfix and scales the limiting upper tail instead (see its
/// docs).
const AD_TAIL_SWITCH: f64 = 0.9995;

/// Smallest `n` for which [`anderson_darling_cdf`] returns a probability.
const AD_MIN_N: usize = 8;

/// Upper normal tail `cPhi(x) = ∫ₓ^∞ φ(t) dt`, to 13–15 digits for |x| < 16
/// by its author's account.
///
/// Port of `cPhi` from `ADinf.c`, the code attached to G. Marsaglia and
/// J. C. W. Marsaglia, "Evaluating the Anderson-Darling Distribution,"
/// *Journal of Statistical Software* 9(2), 2004 (`marsaglia2004anderson` in
/// BIB.md).  [pubs/marsaglia-marsaglia-2004-ADinf.c]  It stores the Mills ratio
/// R(x) = cPhi(x)/φ(x) at x = 0, 2, …, 16 and reaches |x| by a Taylor series
/// in h = |x| − 2j.  Two gaps in the C are closed here.  The C reads past its
/// nine-entry table once |x| ≥ 17, which `ADf` reaches for 144.5 < t ≤ 150;
/// this port clamps to R(16) instead, still within 2·10⁻⁸ relative at
/// x = 17.2, and any term it feeds there is below 10⁻⁶⁰.  The C also has no
/// return if the
/// series has not converged after 49 terms; this port keeps the last partial
/// sum.
fn c_phi(x: f64) -> f64 {
    const MILLS_RATIO_AT_EVEN_X: [f64; 9] = [
        1.253_314_137_315_500_3,
        0.421_369_229_288_054_5,
        0.236_652_382_913_560_67,
        0.162_377_660_896_867_45,
        0.123_131_963_257_932_3,
        0.099_028_596_471_731_93,
        0.082_766_286_501_369_18,
        0.071_069_580_538_852_11,
        0.062_258_665_995_026_2,
    ];
    let j = (((x.abs() + 1.0) / 2.0) as usize).min(MILLS_RATIO_AT_EVEN_X.len() - 1);
    let mut a = MILLS_RATIO_AT_EVEN_X[j];
    let z = (2 * j) as f64;
    let h = x.abs() - z;
    let mut b = z * a - 1.0;
    let mut pwr = 1.0;
    let mut s = a + h * b;
    for i in (2..100).step_by(2) {
        a = (a + z * b) / i as f64;
        b = (b + z * a) / (i + 1) as f64;
        pwr *= h * h;
        let t = s;
        s += pwr * (a + h * b);
        if s == t {
            break;
        }
    }
    // ln √(2π), as in the C.
    s *= (-0.5 * x * x - 0.918_938_533_204_672_8).exp();
    if x > 0.0 {
        s
    } else {
        1.0 - s
    }
}

/// f(z, j), the j-th term of the series for ADinf (Marsaglia and Marsaglia
/// 2004, §2, p. 2): `ADf` in `ADinf.c`.  [pubs/marsaglia-marsaglia-2004-ADinf.c]
///
/// With t = (4j + 1)²π²/(8z) it sums c₀ + c₁(z/8) + c₂(z/8)²/2! + …, where
/// c₀ = π e⁻ᵗ (2t)^(−1/2), c₁ = π (π/2)^(1/2) erfc(√t) and
/// cₙ₊₁ = ((n − ½ − t)cₙ + t cₙ₋₁)/n.
fn ad_inf_term(z: f64, j: usize) -> f64 {
    let k = (4 * j + 1) as f64;
    // π²/8, truncated as in the C.
    let t = k * k * 1.233_700_550_136_17 / z;
    if t > 150.0 {
        return 0.0;
    }
    // π/√2 and π√(π/2), truncated as in the C; 2·cPhi(√(2t)) = erfc(√t).
    let mut a = 2.221_441_469_079_18 * (-t).exp() / t.sqrt();
    let mut b = 3.937_402_486_430_6 * 2.0 * c_phi((2.0 * t).sqrt());
    let mut r = z * 0.125;
    let mut f = a + b * r;
    for i in 1..200 {
        let c = ((i as f64 - 0.5 - t) * b + t * a) / i as f64;
        a = b;
        b = c;
        r *= z / (8 * i + 8) as f64;
        if r.abs() < 1e-40 || c.abs() < 1e-40 {
            return f;
        }
        let f_new = f + c * r;
        if f == f_new {
            return f;
        }
        f = f_new;
    }
    f
}

/// Limiting Anderson–Darling distribution ADinf(z) = lim Pr(Aₙ < z), to
/// about 15 digits: `ADinf` in `ADinf.c` (Marsaglia and Marsaglia 2004, §2,
/// pp. 2–3).  [pubs/marsaglia-marsaglia-2004-ADinf.c]
///
/// ADinf(z) = (1/z) Σⱼ C(−½, j) (4j + 1) f(z, j).  Below z = 0.01 it returns 0,
/// as the C does (ADinf(0.01) ≈ 5.3·10⁻⁵³).  Above `AD_INF_Z_MAX` = 30 this
/// port returns 1, departing from the C, whose series cancels ever larger
/// terms as z grows: `ADinf.c` returns 1 + 3·10⁻¹² at z = 100, −1.8·10³⁶ at
/// z = 1000 and NaN at z = 10⁶, while ADinf(30) = 1 − 1.8·10⁻¹⁴.
fn ad_inf(z: f64) -> f64 {
    if z.is_nan() {
        return f64::NAN;
    }
    if z < 0.01 {
        return 0.0;
    }
    if z > AD_INF_Z_MAX {
        return 1.0;
    }
    let mut r = 1.0 / z;
    let mut ad = r * ad_inf_term(z, 0);
    for j in 1..100 {
        r *= (0.5 - j as f64) / j as f64;
        let ad_new = ad + (4 * j + 1) as f64 * r * ad_inf_term(z, j);
        if ad == ad_new {
            return ad;
        }
        ad = ad_new;
    }
    ad
}

/// errfix(n, x), the fitted correction that turns x = ADinf(z) into
/// Pr(Aₙ < z) (Marsaglia and Marsaglia 2004, §3, p. 4): `errfix` in
/// `AnDarl.c`.  [pubs/marsaglia-marsaglia-2004-AnDarl.c]
///
/// The C squares an `int` n, which overflows for n ≥ 46341; this port squares
/// a float.
fn ad_errfix(n: usize, x: f64) -> f64 {
    let n = n as f64;
    if x > 0.8 {
        // g₃(x)/n
        return (-130.2137
            + (745.2337 - (1705.091 - (1950.646 - (1116.360 - 255.7844 * x) * x) * x) * x) * x)
            / n;
    }
    let c = 0.01265 + 0.1757 / n;
    if x < c {
        // (.0037/n³ + .00078/n² + .00006/n) g₁(x/c), g₁(t) = √t (1 − t)(49t − 102)
        let t = x / c;
        let g1 = t.sqrt() * (1.0 - t) * (49.0 * t - 102.0);
        return g1 * (0.0037 / (n * n) + 0.00078 / n + 0.00006) / n;
    }
    // (.04213/n + .01365/n²) g₂((x − c)/(.8 − c))
    let t = (x - c) / (0.8 - c);
    let g2 = -0.00022633 + (6.54034 - (14.6538 - (14.458 - (8.259 - 1.91864 * t) * t) * t) * t) * t;
    g2 * (0.04213 + 0.01365 / n) / n
}

/// Anderson–Darling distribution function Pr(Aₙ < z), where
/// Aₙ = −n − (1/n) Σᵢ (2i − 1) ln(xᵢ (1 − xₙ₊₁₋ᵢ)) for `n` iid U(0, 1)
/// samples sorted as x₁ ≤ … ≤ xₙ.
///
/// G. Marsaglia and J. C. W. Marsaglia, "Evaluating the Anderson-Darling
/// Distribution," *Journal of Statistical Software* 9(2), 2004, §§2–4,
/// pp. 2–5 (`marsaglia2004anderson` in BIB.md).
/// [pubs/marsaglia-marsaglia-2004-anderson-darling.pdf]  In their summary
/// (p. 5), Pr(Aₙ < z) = ADinf(z) + errfix(n, ADinf(z)): ADinf is the limiting
/// distribution, evaluated by the series of §2, and errfix a correction
/// fitted to simulation, stated good to about ±5·10⁻⁵ for n = 8, 16, 32, 64
/// and 128 and ±5·10⁻⁴ for other n (p. 4).  The code ports the article's
/// attached `ADinf.c` (`ADinf`, `ADf`, `cPhi`) and `AnDarl.c` (`errfix`).
/// [pubs/marsaglia-marsaglia-2004-ADinf.c]
/// [pubs/marsaglia-marsaglia-2004-AnDarl.c]  `AnDarl.c` warns that the test
/// is not well suited to n < 7, where accuracy may drop to three digits.
///
/// # Accuracy
///
/// The figures here come from `examples/anderson_darling_tail.rs` with its
/// default seed (4·10⁹ samples for n = 8, 10⁹ for n = 16 and 32, 2·10⁸ for
/// n = 64 and 128), compared with this function at every z in 0.01 steps.
///
/// Body.  For z ≤ 4 the result is within 9.2·10⁻⁵ of the simulation for all
/// those n (5.3·10⁻⁵ for n = 8), the largest errors near z = 0.4.  That is
/// somewhat wider than the paper's ±5·10⁻⁵.
///
/// Tail.  errfix is an absolute correction that does not vanish as
/// ADinf(z) → 1 (errfix(n, 1) ≈ −6·10⁻⁴/n), so it adds a near-constant to
/// the upper tail Pr(Aₙ ≥ z).  For n = 32, ADinf + errfix gives a tail 14%
/// above the simulation at z = 8, 2.3 times it at z = 10 and 11.5 times it
/// at z = 12.  ADinf alone errs the other way, and more for small n: 12–18%
/// below the simulation for n = 8 at 8 ≤ z ≤ 12.  So once ADinf(z) > x* =
/// 0.9995 (z > 6.6127), this function takes the upper tail as
/// (1 − ADinf(z))·(1 − errfix(n, x*)/(1 − x*)): the limiting tail, scaled by
/// the relative size errfix has at the switch, which meets ADinf + errfix
/// there continuously.  That rule is empirical, chosen from simulation, and
/// is not in the paper.  The worst relative errors of the upper tail against
/// the simulation are:
///
/// | n   | 4 < z ≤ 6.61, ADinf + errfix | 6.61 < z ≤ 12, scaled tail |
/// |-----|------------------------------|----------------------------|
/// | 8   | −0.3% to +8.4%               | −1.1% to +8.5%             |
/// | 16  | −0.1% to +4.6%               | −0.6% to +4.6%             |
/// | 32  | −0.0% to +2.5%               | +0.9% to +3.5%             |
/// | 64  | −0.2% to +0.6%               | −0.7% to +5.8%             |
/// | 128 | −0.3% to +0.6%               | −5.0% to +2.1%             |
///
/// The errors are mostly positive, overstating the tail and so giving
/// conservative p-values.  The largest reliably resolved errors are just
/// past the switch: at z = 6.62 the tail is +8.46 ± 0.07% for n = 8,
/// +4.62 ± 0.14% for n = 16 and +2.46 ± 0.14% for n = 32.  Below the switch
/// the table's lowest values, with their standard errors, are −0.27 ± 0.02%
/// for n = 8 at z = 4.41 (−0.20 ± 0.02% at z = 4), −0.12 ± 0.04% for n = 16
/// at z = 4.44, −0.04 ± 0.04% for n = 32 at z = 4.32, −0.16 ± 0.17% for
/// n = 64 at z = 5.48 and −0.29 ± 0.16% for n = 128 at z = 5.32.  Only the
/// n = 8 understatement is resolved by more than three standard errors; the
/// others, each the lowest of 261 values of z, are within three standard
/// errors of zero.  Past z ≈ 10 the simulated tails carry standard errors of 1%
/// to 5%, and the extremes there, of either sign, lie within one run's
/// sampling noise, so their sign is unresolved.  The negative ones run down
/// to −5.0%: −1.07 ± 1.10% for n = 8 at z = 12, −0.58 ± 1.54% for n = 16 at
/// z = 11.24, and the table's −0.7 ± 3.3% and −5.0 ± 4.2% for n = 64 and 128
/// near z = 11.1 and 11.6.  The positive ones likewise: for n = 32,
/// +1.69 ± 1.41% at z = 11 and +1.32 ± 2.37% at z = 12, and the table's +0.9%
/// and +3.5% (standard errors about 1.3% and 1.5%, at z = 10.82 and 11.15), so
/// the 3.5% is not a resolved error larger than the one at the switch; nor is
/// the +5.8 ± 5.0% for n = 64 near z = 11.8.  None of this can move a verdict
/// at α = 0.01, whose upper tail sits near z = 3.9, inside the body bound.
///
/// Minimum n.  For n < 8 this function returns NaN.  The method fails there
/// before the tail does.  A simulation of 2·10⁹ samples each, made before
/// this minimum was imposed, put ADinf + errfix up to 1.3·10⁻³ from
/// Pr(Aₙ < z) for n = 4, 1.3·10⁻² for n = 2 and 5.4·10⁻² for n = 1 (against
/// the exact distribution, p. 1), and its tail 15% high for n = 4 just below
/// the switch.
///
/// # Departures from the attachments
///
/// - `AnDarl.c`'s `AD(n, z)` feeds errfix the authors' short approximation
///   `adinf(z)`.  This port feeds it the full series, as the paper's formula
///   reads.  `adinf` differs from ADinf by up to 2·10⁻⁵ (near z = 0.97),
///   more than the 2·10⁻⁶ the paper states, so the two results differ by
///   up to that much.
/// - Above the switch the upper tail is the scaled limiting tail described
///   above, and n < 8 returns NaN.
/// - ADinf is taken as 1 for z > 30, where the attachment's series loses
///   accuracy; ADinf(30) = 1 − 1.8·10⁻¹⁴.
/// - The result is clamped to [0, 1].  errfix is negative for small x, so the
///   unclamped sum dips below 0 for small z (at z = 0.1 when n = 10).
///
/// Returns NaN for `n` < 8 or a NaN `z`.
#[must_use]
pub fn anderson_darling_cdf(n: usize, z: f64) -> f64 {
    if n < AD_MIN_N || z.is_nan() {
        return f64::NAN;
    }
    ad_cdf_given_limit(n, ad_inf(z))
}

/// Pr(Aₙ < z) from x = ADinf(z).  Up to the switch this is x + errfix(n, x).
/// Above it the upper tail 1 − x is scaled by errfix's relative size at the
/// switch, 1 − errfix(n, x*)/(1 − x*), which meets the lower branch at x*.
fn ad_cdf_given_limit(n: usize, x: f64) -> f64 {
    if x > AD_TAIL_SWITCH {
        let scale = ad_errfix(n, AD_TAIL_SWITCH) / (1.0 - AD_TAIL_SWITCH);
        return (x + (1.0 - x) * scale).clamp(0.0, 1.0);
    }
    (x + ad_errfix(n, x)).clamp(0.0, 1.0)
}

// ── Chi-square p-value (convenience) ─────────────────────────────────────────

/// Chi-square survival function: P(χ²_{df} > chi_sq) = igamc(df/2, chi_sq/2).
#[must_use]
pub fn chi2_pvalue(chi_sq: f64, df: usize) -> f64 {
    igamc(df as f64 / 2.0, chi_sq / 2.0)
}

// ── Pearson chi-square with Dieharder's tail pooling ────────────────────────

/// Pearson chi-square on binned counts, pooling weak cells exactly as Robert
/// G. Brown's `Vtest_eval` (`Vtest.c`, Dieharder 3.31.1) does.
///
/// - A cell whose expected count is at least `cutoff` is scored on its own.
/// - Every other cell, adjacent or not, is merged into one pooled cell (the C
///   accumulates them at the index of the first weak cell), so weak cells at
///   both ends of a histogram share a single pooled cell.
/// - The pooled cell is scored once, after the scan, and only if its summed
///   expectation also reaches `cutoff`; otherwise its counts are dropped.
/// - df = (number of scored cells) − 1.
///
/// Returns `Some((p_value, df, chi_sq))`, or `None` if the slices differ in
/// length, are empty, or fewer than two cells are scored.  `Vtest.c` has no
/// such guard: with one scored cell it evaluates Q(0, χ²/2), and with none its
/// unsigned `ndof − 1` wraps around; neither is a test.
#[must_use]
pub fn vtest_pvalue(observed: &[u32], expected: &[f64], cutoff: f64) -> Option<(f64, usize, f64)> {
    if observed.len() != expected.len() || observed.is_empty() {
        return None;
    }

    let mut chisq = 0.0;
    let mut ndof_terms = 0usize;
    let mut tail_index: Option<usize> = None;
    let mut tail_obs = 0.0;
    let mut tail_exp = 0.0;

    for i in 0..observed.len() {
        let obs = observed[i] as f64;
        let exp = expected[i];
        if exp >= cutoff {
            let diff = obs - exp;
            chisq += diff * diff / exp;
            ndof_terms += 1;
        } else if tail_index.is_none() {
            tail_index = Some(i);
            tail_obs += obs;
            tail_exp += exp;
        } else {
            tail_obs += obs;
            tail_exp += exp;
        }
    }

    if tail_index.is_some() && tail_exp >= cutoff {
        let diff = tail_obs - tail_exp;
        chisq += diff * diff / tail_exp;
        ndof_terms += 1;
    }

    if ndof_terms <= 1 {
        return None;
    }
    let df = ndof_terms - 1;
    Some((chi2_pvalue(chisq, df), df, chisq))
}

// ── Discrete probability mass functions ─────────────────────────────────────

/// Binomial PMF: P(X = k) for X ~ Binomial(n, p), with `k ≤ n`.
///
/// Evaluated in log space through [`lgamma`], so it stays finite for the
/// large `n` of Dieharder's bit-count tests (C(n, k) itself overflows `f64`
/// past n ≈ 1 030).  Relative error is a few × 10⁻¹⁰, inherited from the three
/// `lgamma` terms.  The degenerate `p ≤ 0` and `p ≥ 1` cases return the exact
/// point masses instead of taking `ln 0`.
///
/// Stands in for GSL's `gsl_ran_binomial_pdf`, which Robert G. Brown's
/// Dieharder 3.31.1 calls in `rgb_bitdist.c`, `dab_monobit2.c` and
/// `chisq_binomial` (`chisq.c`); used here by
/// [`crate::dieharder::bit_distribution`] and [`crate::dieharder::monobit2`].
#[must_use]
pub fn binomial_pmf(n: usize, k: usize, p: f64) -> f64 {
    debug_assert!(k <= n, "binomial_pmf: k = {k} exceeds n = {n}");
    if p <= 0.0 {
        return if k == 0 { 1.0 } else { 0.0 };
    }
    if p >= 1.0 {
        return if k == n { 1.0 } else { 0.0 };
    }
    let q = 1.0 - p;
    let log_comb = lgamma((n + 1) as f64) - lgamma((k + 1) as f64) - lgamma((n - k + 1) as f64);
    (log_comb + (k as f64) * p.ln() + ((n - k) as f64) * q.ln()).exp()
}

/// Poisson PMF: P(X = k) = e^(−λ) λᵏ / k! for X ~ Poisson(λ).
///
/// Computed as the running product e^(−λ) · ∏ᵢ₌₁ᵏ (λ / i), which never forms
/// λᵏ or k! separately, so it cannot overflow for the small k the chi-square
/// histograms use.
///
/// Stands in for GSL's `gsl_ran_poisson_pdf`, which Robert G. Brown's
/// Dieharder 3.31.1 calls in `diehard_birthdays.c` and `chisq_poisson`
/// (`chisq.c`); used here by [`crate::diehard::birthday_spacings`].
#[must_use]
pub fn poisson_pmf(k: usize, lambda: f64) -> f64 {
    let mut term = (-lambda).exp();
    for i in 1..=k {
        term *= lambda / i as f64;
    }
    term
}

// ── Discrete Fourier Transform (DFT) ─────────────────────────────────────────

/// FFT for a real input of arbitrary length n.
/// Returns magnitudes |X_k| for k = 0..n.
///
/// Uses `rustfft` so the NIST spectral test can analyze the full sequence
/// length instead of truncating to a radix-2 prefix.
#[must_use]
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
/// Reference implementation kept to cross-check [`fft_magnitudes`] (which the
/// NIST spectral test uses); the two agree to machine precision (see tests).
/// Prefer [`fft_magnitudes`] for anything beyond a few thousand points.
#[must_use]
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
/// are used (caller must mask if needed); `cols` must be ≤ 32.
///
/// Time: O(rows × cols × min(rows,cols)).
#[must_use]
pub fn gf2_rank(matrix: &[u32], rows: usize, cols: usize) -> usize {
    debug_assert!(
        cols <= 32,
        "gf2_rank rows are packed u32: cols must be <= 32"
    );
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

    // Reference: a line-by-line Python replica of `Vtest_eval` (Vtest.c).
    #[test]
    fn vtest_pools_weak_cells_from_both_ends() {
        // Cells 0 and 3 are weak; their pool (x = 5, y = 2 + 3 = 5) reaches the
        // cutoff, so it is scored as a third cell.
        let (p, df, chi) = vtest_pvalue(&[1, 12, 9, 4], &[2.0, 10.0, 10.0, 3.0], 5.0).unwrap();
        assert_eq!(df, 2);
        assert!((chi - 0.5).abs() < 1e-15, "χ² = {chi}");
        assert!((p - 0.7788007830714049).abs() < 1e-12, "p = {p}");

        // Pooled expectation 2 + 2 = 4 misses the cutoff: the pool is dropped.
        let (p, df, chi) = vtest_pvalue(&[3, 12, 9, 0], &[2.0, 10.0, 10.0, 2.0], 5.0).unwrap();
        assert_eq!(df, 1);
        assert!((chi - 0.5).abs() < 1e-15, "χ² = {chi}");
        assert!((p - 0.4795001221869535).abs() < 1e-9, "p = {p}");

        // One strong cell and an unscored pool leave no degrees of freedom.
        assert!(vtest_pvalue(&[7, 30, 1], &[3.0, 30.0, 1.0], 5.0).is_none());
        assert!(vtest_pvalue(&[], &[], 5.0).is_none());
    }

    // Reference values are exact rationals C(n,k)·pᵏ·(1−p)ⁿ⁻ᵏ rounded once to
    // f64 (Python `fractions`), independent of the lgamma evaluation.
    #[test]
    fn binomial_pmf_known_values() {
        let cases = [
            (64, 0, 0.5, 5.421010862427522e-20),
            (64, 32, 0.5, 0.09934675374796689),
            (64, 1, 1.0 / 1024.0, 0.058768915016183754),
            (64, 3, 1.0 / 16.0, 0.19844898179736012),
            (8, 4, 0.5, 0.2734375),
            (512, 256, 0.5, 0.03524463548583874),
        ];
        for (n, k, p, want) in cases {
            let got = binomial_pmf(n, k, p);
            assert!(
                ((got - want) / want).abs() < 1e-9,
                "B({n},{p}) at {k}: {got} vs {want}"
            );
        }
    }

    #[test]
    fn binomial_pmf_degenerate_probabilities() {
        assert_eq!(binomial_pmf(10, 0, 0.0), 1.0);
        assert_eq!(binomial_pmf(10, 1, 0.0), 0.0);
        assert_eq!(binomial_pmf(10, 10, 1.0), 1.0);
        assert_eq!(binomial_pmf(10, 9, 1.0), 0.0);
    }

    // Reference: e⁻² (as an f64) · 2ᵏ / k! evaluated exactly, then rounded once.
    #[test]
    fn poisson_pmf_known_values() {
        let cases = [
            (0, 0.1353352832366127),
            (1, 0.2706705664732254),
            (2, 0.2706705664732254),
            (7, 0.0034370865583901638),
            (20, 5.832924198269276e-14),
        ];
        for (k, want) in cases {
            let got = poisson_pmf(k, 2.0);
            assert!(
                ((got - want) / want).abs() < 1e-14,
                "Poisson(2) at {k}: {got} vs {want}"
            );
        }
    }

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
        // Q(a, ∞) = 0
        assert_eq!(igamc(1.0, f64::INFINITY), 0.0);
        // Domain errors → NaN
        assert!(igamc(0.0, 1.0).is_nan());
        assert!(igamc(1.0, -1.0).is_nan());
        assert!(igamc(f64::NAN, 1.0).is_nan());
    }

    // Golden values from scipy.special.gammaincc / scipy.stats.chi2.sf.
    #[test]
    fn igamc_golden_values() {
        // Series branch (x < a + 1)
        assert!((igamc(3.0, 2.5) - 0.5438131158833297).abs() < 1e-10);
        // Continued-fraction branch (x ≥ a + 1)
        assert!((igamc(0.5, 2.0) - 0.045500263896358445).abs() < 1e-10);
        // Chi-square mapping: P(χ²₅ > 11.0705) ≈ 0.05
        assert!((chi2_pvalue(11.0705, 5) - 0.04999995542804364).abs() < 1e-9);
    }

    // Large shape parameters need O(√a) iterations; a fixed 500-iteration cap
    // silently returned ~11% relative error at a = 10⁵.
    #[test]
    fn igamc_large_shape_parameter() {
        assert!((igamc(20_000.0, 20_000.0) - 0.49905968376625065).abs() < 1e-6);
        assert!((igamc(100_000.0, 100_000.0) - 0.4995794778896348).abs() < 1e-6);
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
        // Empty sample → no verdict, not a hard FAIL.
        assert!(ks_pvalue(0.5, 0).is_nan());
    }

    // Golden values from scipy.stats.kstwo.sf (the exact two-sided KS law).
    // These exercise the Marsaglia-Tsang-Wang matrix path, which a triangular
    // H-matrix bug once collapsed to p ≈ 1 for every input.
    #[test]
    fn ks_pvalue_exact_golden_values() {
        assert!((ks_pvalue(0.1, 100) - 0.2526927570063875).abs() < 1e-5);
        assert!((ks_pvalue(0.15, 100) - 0.01983924212564203).abs() < 1e-6);
        assert!((ks_pvalue(0.409, 10) - 0.05022340810547443).abs() < 1e-6);
        assert!((ks_pvalue(0.05, 1000) - 0.013012074781090332).abs() < 1e-6);
    }

    // Asymptotic branch (n > 4999): Stephens-corrected series, ~1e-3 class
    // accuracy in the far tail — assert ballpark, not digits.
    #[test]
    fn ks_pvalue_asymptotic_golden_value() {
        let p = ks_pvalue(0.02, 10_000);
        assert!((p - 6.616848639387309e-4).abs() < 2e-4, "p = {p}");
    }

    // The two asymptotic-series non-convergence modes must route oppositely:
    // huge s (all terms underflow, catastrophic D) → 0; tiny s (terms stay
    // O(1), near-superuniform sample) → 1.  A constant stream once PASSed
    // ks_uniform because huge-s fell into the tiny-s fallback.
    #[test]
    fn ks_pvalue_asymptotic_extremes() {
        assert_eq!(ks_pvalue(0.99, 16_000_000), 0.0);
        assert_eq!(ks_pvalue(1e-9, 100_000), 1.0);
    }

    #[test]
    fn lgamma_golden_values() {
        assert!((lgamma(10.0) - 12.801827480081467).abs() < 1e-8);
        assert!((lgamma(0.5) - 0.5723649429247004).abs() < 1e-10);
    }

    #[test]
    fn erfc_golden_value_tail() {
        assert!((erfc(2.0) - 0.004677734981047266).abs() < 5e-7);
    }

    #[test]
    fn dft_and_fft_agree() {
        let x: Vec<f64> = (0..64).map(|i| ((i * 37 + 11) % 17) as f64 - 8.0).collect();
        let slow = dft_magnitudes(&x);
        let fast = fft_magnitudes(&x);
        assert_eq!(slow.len(), fast.len());
        for (s, f) in slow.iter().zip(&fast) {
            assert!((s - f).abs() < 1e-9, "dft {s} vs fft {f}");
        }
    }

    #[test]
    fn gf2_rank_known_matrices() {
        // 3×3 identity → rank 3
        assert_eq!(gf2_rank(&[0b001, 0b010, 0b100], 3, 3), 3);
        // Dependent rows: r2 = r0 ^ r1 → rank 2
        assert_eq!(gf2_rank(&[0b011, 0b101, 0b110], 3, 3), 2);
        // Zero matrix → rank 0
        assert_eq!(gf2_rank(&[0, 0, 0], 3, 3), 0);
    }

    /// Regression: the sort used `partial_cmp().unwrap()`, so one NaN sample
    /// panicked.  It now reports NaN (insufficient data).
    #[test]
    fn ks_test_with_nan_sample_is_insufficient() {
        let mut with_nan = vec![0.1, f64::NAN, 0.7];
        assert!(crate::math::ks_test(&mut with_nan).is_nan());
        let mut clean = vec![0.1, 0.4, 0.7];
        assert!(crate::math::ks_test(&mut clean).is_finite());
    }

    /// Values printed in Marsaglia and Marsaglia (2004): ADinf(9) and
    /// ADinf(10) from their 30-digit Maple evaluation (p. 3), and the 90, 95
    /// and 99 percent points of the limiting distribution to 20 places (p. 2).
    #[test]
    fn ad_inf_matches_values_printed_in_the_paper() {
        let cases = [
            (9.0, 0.999_960_465_988_612_4),
            (10.0, 0.999_986_184_964_589_4),
            (1.932_957_832_741_593_7, 0.90),
            (2.492_367_160_049_409_5, 0.95),
            (3.878_125_021_605_395, 0.99),
        ];
        for (z, want) in cases {
            let got = ad_inf(z);
            assert!(
                (got - want).abs() < 1e-14,
                "ADinf({z}) = {got}, paper {want}"
            );
        }
    }

    /// `ADinf` and `cPhi` against the attachment `ADinf.c`
    /// [pubs/marsaglia-marsaglia-2004-ADinf.c], compiled unchanged
    /// except for renaming its interactive `main`.  The relative tolerance
    /// leaves room for libm's `exp` and `sqrt` to differ by an ulp.
    #[test]
    fn ad_inf_and_c_phi_match_the_attached_c() {
        let ad_inf_cases = [
            (0.01, 5.280028041431955e-53),
            (0.05, 1.731492268016011e-10),
            (0.1, 2.8078105126362928e-5),
            (0.2, 0.009_587_452_750_205_868),
            (0.5, 0.253_185_626_469_655_03),
            (1.0, 0.642_733_326_785_979_9),
            (1.5, 0.823_524_627_157_948_7),
            (2.0, 0.908_163_225_058_746_4),
            (3.0, 0.972_635_211_665_972_2),
            (5.0, 0.997_125_578_695_412_5),
            (8.0, 0.999_886_185_844_272_1),
            (12.0, 0.999_998_289_712_999),
            (15.0, 0.999_999_923_667_766_7),
            (20.0, 0.999_999_999_553_491_5),
            (30.0, 0.999_999_999_999_982_2),
        ];
        for (z, want) in ad_inf_cases {
            let got = ad_inf(z);
            assert!(
                (got - want).abs() <= 1e-13 * want,
                "ADinf({z}) = {got}, C {want}"
            );
        }
        let c_phi_cases = [
            (0.0, 0.5),
            (0.5, 0.308_537_538_725_987),
            (1.0, 0.158_655_253_931_457_05),
            (1.5, 0.066_807_201_268_858_09),
            (2.75, 0.002_979_763_235_054_556),
            (3.3, 0.000_483_424_142_383_777_33),
            (5.0, 2.8665157187919407e-7),
            (8.0, 6.220960574271776e-16),
            (15.9, 3.168237665379637e-57),
        ];
        for (x, want) in c_phi_cases {
            let got = c_phi(x);
            assert!(
                (got - want).abs() <= 1e-13 * want,
                "cPhi({x}) = {got}, C {want}"
            );
            assert_eq!(1.0 - got, c_phi(-x), "cPhi(-{x})");
        }
        // Past the C's table (|x| >= 17) this port clamps to R(16), which
        // still agrees with libm's erfc(x/√2)/2 to 2·10⁻⁸ relative at 17.2.
        let want = 1.3276575042717985e-66;
        let got = c_phi(17.2);
        assert!(
            (got - want).abs() <= 1e-7 * want,
            "cPhi(17.2) = {got}, erfc {want}"
        );
    }

    /// `errfix` against the attachment `AnDarl.c`
    /// [pubs/marsaglia-marsaglia-2004-AnDarl.c], compiled unchanged except
    /// for renaming its interactive `main`, and with `-ffp-contract=off`.
    /// Clang otherwise fuses multiply-adds, and above x = 0.8, where the terms
    /// of g₃ cancel from about 10³ to 10⁻³, that moves the result by up to
    /// 5·10⁻¹⁴.
    #[test]
    fn ad_errfix_matches_the_attached_c() {
        let cases = [
            (1, 1.0e-6, -0.001_067_013_216_633_883),
            (1, 0.01, -0.098_460_080_594_883_3),
            (1, 0.05, -0.152_905_821_563_227_85),
            (1, 0.3, 0.043_760_865_899_933_34),
            (1, 0.79, 0.002_782_201_881_838_179),
            (1, 0.8, 0.000_220_535_712_599_942_1),
            (1, 0.81, -0.001_216_175_685_414_100_4),
            (1, 0.999, -0.001_065_211_887_748_773_7),
            (1, 1.0, -0.000_599_999_999_820_966),
            (8, 1.0e-6, -1.4755175138691444e-5),
            (8, 0.01, -0.000_903_636_990_841_514_6),
            (8, 0.05, 0.000_687_436_397_468_982_7),
            (8, 0.3, 0.005_472_533_410_294_906),
            (8, 0.79, 0.000_223_154_174_538_196_43),
            (8, 0.8, 2.166425831718181e-5),
            (8, 0.81, -0.000_152_021_960_676_762_55),
            (8, 0.999, -0.000_133_151_485_968_596_72),
            (8, 1.0, -7.499999997762075e-5),
            (32, 1.0e-6, -2.0821569497825443e-6),
            (32, 0.01, -6.869903814898384e-5),
            (32, 0.05, 0.000_323_037_688_188_530_7),
            (32, 0.3, 0.001_333_498_931_632_664_5),
            (32, 0.79, 5.3137477126165396e-5),
            (32, 0.8, 5.257956389354088e-6),
            (32, 0.81, -3.8005490169190637e-5),
            (32, 0.999, -3.328787149214918e-5),
            (32, 1.0, -1.8749999994405186e-5),
            (128, 1.0e-6, -4.4624254664453196e-7),
            (128, 0.01, -8.416673652423299e-6),
            (128, 0.05, 8.902644718728321e-5),
            (128, 0.3, 0.000_331_101_741_613_059_63),
            (128, 0.79, 1.3122728471196247e-5),
            (128, 0.8, 1.304607335467187e-6),
            (128, 0.81, -9.501372542297659e-6),
            (128, 0.999, -8.321967873037295e-6),
            (128, 1.0, -4.687499998601297e-6),
        ];
        for (n, x, want) in cases {
            let got = ad_errfix(n, x);
            assert!(
                (got - want).abs() <= 1e-15 * want.abs(),
                "errfix({n}, {x}) = {got}, C {want}"
            );
        }
    }

    /// [`anderson_darling_cdf`] against `AnDarl.c`'s `AD(n, z)`
    /// [pubs/marsaglia-marsaglia-2004-AnDarl.c], which feeds
    /// errfix the short approximation `adinf` in place of ADinf; on a 0.0005
    /// grid over [0.01, 12] the two differ by at most 1.95·10⁻⁵, at z = 0.97.
    /// The cases stay below the tail switch, where this port drops errfix.
    /// `AnDarl.c`'s own `ADtest` examples, two samples of 10, are pinned both
    /// to that function and to ADinf + errfix evaluated by the attachments.
    #[test]
    fn anderson_darling_cdf_tracks_the_attached_c() {
        let cases = [
            (10, 0.362_1, 0.117_111_610_941_818_05),
            (10, 0.5, 0.257_365_994_233_906_2),
            (10, 1.0, 0.644_937_032_601_438_6),
            (10, 1.5, 0.823_210_291_017_505_2),
            (10, 2.0, 0.906_935_349_212_242_4),
            (10, 3.0, 0.971_694_963_675_239_7),
            (10, 5.0, 0.996_944_065_399_267_3),
            (32, 0.362_1, 0.115_490_941_908_271_44),
            (32, 0.5, 0.254_474_502_581_219_9),
            (32, 1.0, 0.643_384_778_120_492_8),
            (32, 1.5, 0.823_427_577_900_760_5),
            (32, 2.0, 0.907_780_216_932_811_9),
            (32, 3.0, 0.972_342_045_084_779_1),
            (32, 5.0, 0.997_074_639_071_479_5),
        ];
        for (n, z, want) in cases {
            let got = anderson_darling_cdf(n, z);
            assert!((got - want).abs() < 2e-5, "AD({n}, {z}) = {got}, C {want}");
        }
        let u: [f64; 10] = [
            0.0392, 0.0884, 0.260, 0.310, 0.454, 0.644, 0.797, 0.813, 0.921, 0.960,
        ];
        let w: [f64; 10] = [
            0.0015, 0.0078, 0.0676, 0.0961, 0.106, 0.107, 0.835, 0.861, 0.948, 0.992,
        ];
        for (x, statistic, ad_test, full) in [
            (
                u,
                0.363_203_962_367_370_2,
                0.118_164_224_906_405_5,
                0.118_167_210_503_083_64,
            ),
            (
                w,
                4.231_606_537_078_047,
                0.992_927_546_852_983_1,
                0.992_924_491_393_447_5,
            ),
        ] {
            let log_sum: f64 = (0..10)
                .map(|i| (2 * i + 1) as f64 * (x[i] * (1.0 - x[9 - i])).ln())
                .sum();
            let a = -10.0 - log_sum / 10.0;
            assert!((a - statistic).abs() < 1e-12, "A = {a}");
            let p = anderson_darling_cdf(10, a);
            assert!(
                (p - full).abs() < 1e-12,
                "Pr(A < {a}) = {p}, ADinf+errfix {full}"
            );
            assert!(
                (p - ad_test).abs() < 2e-5,
                "Pr(A < {a}) = {p}, ADtest {ad_test}"
            );
        }
    }

    #[test]
    fn anderson_darling_cdf_edges() {
        assert!(anderson_darling_cdf(8, f64::NAN).is_nan());
        assert_eq!(0.0, anderson_darling_cdf(10, 0.0));
        // ADinf(0.1) + errfix(10, ·) = −2.6·10⁻⁵ before the clamp.
        assert!(ad_inf(0.1) + ad_errfix(10, ad_inf(0.1)) < 0.0);
        assert_eq!(0.0, anderson_darling_cdf(10, 0.1));
        // Fewer than eight samples: no probability.
        for n in 0..AD_MIN_N {
            assert!(anderson_darling_cdf(n, 1.0).is_nan(), "n = {n}");
        }
        assert!(anderson_darling_cdf(AD_MIN_N, 1.0).is_finite());
        // The tail branch meets ADinf + errfix at the switch.
        let just_above = f64::next_up(AD_TAIL_SWITCH);
        for n in [8, 16, 32, 128, 1000] {
            let at = ad_cdf_given_limit(n, AD_TAIL_SWITCH);
            assert_eq!(AD_TAIL_SWITCH + ad_errfix(n, AD_TAIL_SWITCH), at);
            assert!(
                (ad_cdf_given_limit(n, just_above) - at).abs() < 1e-15,
                "n = {n}"
            );
        }
        // ADinf(6.61) = 0.999498547 is below the switch and ADinf(6.62) =
        // 0.999503901 above it; Pr(A < z) rises by little more than ADinf.
        let (below, above) = (6.61, 6.62);
        assert!(ad_inf(below) <= AD_TAIL_SWITCH && ad_inf(above) > AD_TAIL_SWITCH);
        let step = anderson_darling_cdf(32, above) - anderson_darling_cdf(32, below);
        assert!(step > 0.0 && step < 1e-5, "{step}");
        // Past z = 30 ADinf, and so the result, is 1.
        for z in [30.5, 1e3, 1e6, f64::INFINITY] {
            assert_eq!(1.0, anderson_darling_cdf(32, z), "z = {z}");
        }
        assert!(1.0 - anderson_darling_cdf(32, 30.0) < 2e-14);
    }

    /// Upper tails Pr(Aₙ ≥ z) from `examples/anderson_darling_tail.rs` with its
    /// default seed (4·10⁹ samples for n = 8, 10⁹ for n = 16 and 32, 2·10⁸ for
    /// n = 128), with their standard errors.  Each case allows |p − tail|
    /// errfix's stated ±5·10⁻⁵ for z ≤ 4 and, beyond that, the worst relative
    /// error the [`anderson_darling_cdf`] docs give for that n, plus three
    /// standard errors.
    #[test]
    fn anderson_darling_upper_tail_matches_simulation() {
        // (n, z, simulated tail, standard error)
        #[rustfmt::skip]
        let cases = [
            (8, 2.0, 9.334754e-2, 4.60e-6),
            (8, 4.0, 9.26695675e-3, 1.52e-6),
            (8, 6.0, 1.0666525e-3, 5.16e-7),
            (8, 6.61, 5.58675e-4, 3.74e-7),
            (8, 6.62, 5.528135e-4, 3.72e-7),
            (8, 8.0, 1.3008775e-4, 1.80e-7),
            (8, 10.0, 1.622675e-5, 6.37e-8),
            (8, 12.0, 2.08925e-6, 2.29e-8),
            (16, 2.0, 9.2573499e-2, 9.17e-6),
            (16, 4.0, 8.989914e-3, 2.98e-6),
            (16, 6.0, 1.015447e-3, 1.01e-6),
            (16, 6.61, 5.29267e-4, 7.27e-7),
            (16, 6.62, 5.23638e-4, 7.23e-7),
            (16, 8.0, 1.21445e-4, 3.48e-7),
            (16, 10.0, 1.494e-5, 1.22e-7),
            (16, 12.0, 1.854e-6, 4.31e-8),
            (32, 2.0, 9.2206127e-2, 9.15e-6),
            (32, 4.0, 8.852596e-3, 2.96e-6),
            (32, 6.0, 9.92166e-4, 9.96e-7),
            (32, 6.61, 5.14924e-4, 7.17e-7),
            (32, 6.62, 5.0945e-4, 7.14e-7),
            (32, 8.0, 1.17943e-4, 3.43e-7),
            (32, 10.0, 1.429e-5, 1.20e-7),
            (32, 12.0, 1.776e-6, 4.21e-8),
            (128, 2.0, 9.1922825e-2, 2.04e-5),
            (128, 4.0, 8.75464e-3, 6.59e-6),
            (128, 6.0, 9.7526e-4, 2.21e-6),
            (128, 6.61, 5.05715e-4, 1.59e-6),
            (128, 8.0, 1.1595e-4, 7.61e-7),
            (128, 10.0, 1.4065e-5, 2.65e-7),
        ];
        for (n, z, tail, se) in cases {
            let (relative, absolute) = match (n, z <= 4.0) {
                (_, true) => (0.0, 5e-5),
                (8, false) => (0.09, 0.0),
                (16, false) => (0.05, 0.0),
                (32, false) => (0.035, 0.0),
                _ => (0.05, 0.0),
            };
            let p = 1.0 - anderson_darling_cdf(n, z);
            assert!(
                (p - tail).abs() <= relative * tail + absolute + 3.0 * se,
                "n = {n}, z = {z}: p = {p}, simulation {tail} ± {se}"
            );
        }
        // Why the tail is scaled: at z = 10, ADinf + errfix would put n = 32
        // at over twice the simulated tail, and ADinf alone would put n = 8
        // more than 10% below it.
        let x = ad_inf(10.0);
        assert!(1.0 - (x + ad_errfix(32, x)) > 2.0 * 1.429e-5);
        assert!(1.0 - x < 0.9 * 1.622675e-5);
    }
}
