//! Special mathematical functions used across all test suites.
//!
//! All functions are pure Rust, no external crates.  Algorithms are cited inline.

use rustfft::{num_complex::Complex, FftPlanner};
use std::f64::consts::{PI, SQRT_2};

// ── erfc and the normal distribution ──────────────────────────────────────────

/// R(z) = cPhi(z)/φ(z), the upper normal tail over the density (Mills'
/// ratio), at z = 0, 2, 4, …, 16.
///
/// The digits are those of the `R[9]` initializer in `cPhi`, G. Marsaglia,
/// "Evaluating the Normal Distribution", *Journal of Statistical Software*
/// 11(4), 2004, pp. 2 and 9; each rounds to the nearest f64.
/// [pubs/marsaglia-2004-normal-distribution.pdf]
#[allow(clippy::excessive_precision)]
const MILLS_RATIO_AT_EVEN: [f64; 9] = [
    1.25331413731550025,
    0.421369229288054473,
    0.236652382913560671,
    0.162377660896867462,
    0.123131963257932296,
    0.0990285964717319214,
    0.0827662865013691773,
    0.0710695805388521071,
    0.0622586659950261958,
];

/// ln √(2π), written as `.91893853320467274178L` in `Phi` and `cPhi` of
/// Marsaglia (2004), pp. 1, 2 and 9: φ(x) = exp(−x²/2 − ln √(2π)).
#[allow(clippy::excessive_precision)]
const LN_SQRT_2PI: f64 = 0.91893853320467274178;

/// Largest tabled z; `mills_ratio` sums an asymptotic series beyond it.
const MILLS_TABLE_END: f64 = 16.0;

/// `mills_ratio` stops its Taylor series once a pair of terms, past the
/// order where the terms must shrink, is at most this fraction of the sum.
const MILLS_TAIL: f64 = f64::EPSILON / 8.0;

/// Mills' ratio R(x) = cPhi(x)/φ(x) for x ≥ 0 (not NaN).
///
/// Up to x = 16 this is the Taylor series of Marsaglia (2004, p. 4) about a
/// tabled z, x = z + h: R′ = xR − 1 gives R⁽ᵏ⁺¹⁾ = xR⁽ᵏ⁾ + kR⁽ᵏ⁻¹⁾, and the
/// loop builds the coefficients cₖ = R⁽ᵏ⁾(z)/k! two at a time as his `cPhi`
/// does.  It departs from `cPhi`, which computes in 80-bit `long double`, in
/// two places.
///
/// The expansion point.  `cPhi` takes the nearest tabled z (|h| ≤ 1).  The
/// part of a rounding error in R(z) or R′(z) that is not a multiple of R is
/// a multiple of e^{x²/2}, the solution of R′ = xR, and grows by
/// e^{zh + h²/2} on the way to z + h; in f64 with h near +1 that measured
/// 1.2 × 10⁻¹⁰ relative error near x = 15.  Here z is the tabled point at
/// or above x (−2 < h ≤ 0), or z = 0 for x < 1.
///
/// The stopping rule.  The recurrence cₖ₊₁ = (z·cₖ + cₖ₋₁)/(k + 1) runs
/// forward, so rounding in the early coefficients grows like the terms of
/// e^{z|h|} and cancels only over the whole alternating tail.  `cPhi` stops
/// at the first pair of terms that rounds away, which can be a pair where
/// that error and the true term cancel; in f64 that left 1.5 × 10⁻¹⁰
/// relative error at x = 14.886.  The recurrence gives
/// |cₖ₊₁hᵏ⁺¹| ≤ ((z|h| + h²)/(k + 1))·max(|cₖhᵏ|, |cₖ₋₁hᵏ⁻¹|) for whatever
/// values rounding left in cₖ and cₖ₋₁, so once k + 1 ≥ 2(z|h| + h²) each
/// term is at most half the larger of the two before it, and everything
/// after a pair of terms sums to at most twice that pair's magnitude.  The
/// loop stops at the first pair past that order whose magnitude is at most
/// ε·R/8, so the terms left out change R by at most ε·R/4.
///
/// Past the table, solving R′ = xR − 1 for R and substituting repeatedly
/// gives the asymptotic series R(x) ~ x⁻¹ Σₖ (−1)ᵏ (2k − 1)!! x⁻²ᵏ.  Its terms
/// fall until k ≈ x²/2 > 128, the sum stops changing by k ≈ 12, and the
/// truncation error is below the first omitted term.
fn mills_ratio(x: f64) -> f64 {
    if x > MILLS_TABLE_END {
        let w = 1.0 / (x * x);
        let (mut term, mut s, mut k) = (1.0, 1.0, 1.0);
        loop {
            term *= -(2.0 * k - 1.0) * w;
            let t = s;
            s = t + term;
            if s == t {
                return s / x;
            }
            k += 1.0;
        }
    }
    let j = if x < 1.0 {
        0
    } else {
        (0.5 * x).ceil() as usize
    };
    let z = 2.0 * j as f64;
    let h = x - z;
    let q = h * h;
    let settled = 2.0 * (z * h.abs() + q);
    let mut a = MILLS_RATIO_AT_EVEN[j];
    let mut b = a * z - 1.0;
    let mut pwr = 1.0;
    let mut s = a + h * b;
    let mut i = 2.0;
    loop {
        a = (a + z * b) / i;
        b = (b + z * a) / (i + 1.0);
        pwr *= q;
        s += pwr * (a + h * b);
        if i >= settled && pwr * (a.abs() + (h * b).abs()) <= s * MILLS_TAIL {
            return s;
        }
        i += 2.0;
    }
}

/// Upper normal tail cPhi(x) = 1 − Φ(x) = R(x)·φ(x) for x ≥ 0 (not NaN),
/// the last step of Marsaglia's `cPhi`.  At x = 0 the product
/// R(0)·exp(−ln √(2π)) rounds to exactly 0.5.
fn normal_upper_tail(x: f64) -> f64 {
    mills_ratio(x) * (-0.5 * x * x - LN_SQRT_2PI).exp()
}

/// Complementary error function, erfc(x) = 1 − erf(x).
///
/// erfc(x) = 2·cPhi(x√2) for x ≥ 0 and erfc(x) = 2 − erfc(−x) below 0, with
/// cPhi(u) = 1 − Φ(u) evaluated by the method of G. Marsaglia, "Evaluating
/// the Normal Distribution", *Journal of Statistical Software* 11(4), 2004,
/// pp. 2–5 and 9, with the table point and tail changes f64 needs (see
/// `mills_ratio` in the source).  [pubs/marsaglia-2004-normal-distribution.pdf]
///
/// Accuracy, as the largest error observed against 70-digit references
/// from two independent Python `decimal` oracles (they agree to 10⁻⁶¹; see
/// the tests) on 121 489 arguments in [−11.3, 26.5], packed near the
/// expansion points and near the arguments where stopping the series at
/// the first negligible pair erred; ε is `f64::EPSILON`:
///
/// - from 0 to 26.5, relative error 2.8·(1 + x²)·ε: at most 9.4 × 10⁻¹⁶
///   below x = 1, 6.5 × 10⁻¹⁵ below 4, 9.3 × 10⁻¹⁴ below 16 and
///   3.2 × 10⁻¹³ up to 26.5, the x² coming from exp(−u²/2) at the rounded
///   u = x√2;
/// - below 0, relative error 2.3·ε and absolute error 5.5 × 10⁻¹⁶.
///
/// The tests allow about twice these.  Results are subnormal from
/// x ≈ 26.55, where the relative error grows to order 1, and 0 from
/// x ≈ 27.22.
///
/// erfc(±0) = 1 exactly, 0 ≤ erfc(x) ≤ 1 for x ≥ 0 and 1 ≤ erfc(x) ≤ 2 below,
/// so a two-sided p-value erfc(|z|/√2) never exceeds 1.  erfc(+∞) = 0,
/// erfc(−∞) = 2 and erfc(NaN) = NaN.
///
/// Used by nearly every NIST SP 800-22 test for its p-value.
#[must_use]
pub fn erfc(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    let y = 2.0 * normal_upper_tail(x.abs() * SQRT_2);
    if x >= 0.0 {
        y
    } else {
        2.0 - y
    }
}

/// Standard normal CDF, Φ(x) = P(Z ≤ x) for Z ~ N(0,1).
///
/// Φ(x) = cPhi(−x) below 0 and 1 − cPhi(x) from 0 up, with cPhi as in
/// [`erfc`] but without its x√2 rescaling, so lower-tail values keep relative
/// accuracy.  Against the same references on 123 323 arguments in
/// [−37.5, 40], the largest relative error observed below 0 is
/// 2.5·(1 + x²/2)·ε (3.4 × 10⁻¹⁵ on [−6, 0), 1.6 × 10⁻¹⁴ on [−16, −6) and
/// 9.4 × 10⁻¹⁴ on [−37.5, −16)), and the largest absolute error from 0 up is
/// 3.0 × 10⁻¹⁶ (relative 2.3·ε).  The tests allow about twice these.
/// Results are subnormal below x ≈ −37.5 and 0 from x ≈ −38.49.
///
/// Marsaglia's table-free `Phi` (2004, p. 1) is not used.  Evaluated as
/// printed with f64 throughout, it exceeds 1 by up to 1.11 × 10⁻¹⁵ at 284
/// points of a 10⁻⁴ grid on [7, 9], and its relative error is 1.39 × 10⁻⁹
/// at x = −5 and 9.0 × 10⁻⁷ at x = −6.
///
/// Φ(±0) = 0.5 exactly and 0 ≤ Φ(x) ≤ 1.  Φ(−∞) = 0, Φ(+∞) = 1 and
/// Φ(NaN) = NaN.
#[must_use]
pub fn normal_cdf(x: f64) -> f64 {
    if x.is_nan() {
        x
    } else if x < 0.0 {
        normal_upper_tail(-x)
    } else {
        1.0 - normal_upper_tail(x)
    }
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

    // ── erfc and normal_cdf ───────────────────────────────────────────────────

    /// erfc(x) at the f64 arguments shown, rounded once to f64 from two
    /// independent Python `decimal` evaluations at the exact binary argument.
    /// One sums erf's series (2/√π)·e^{−x²}·Σ 2ⁿx²ⁿ⁺¹/(1·3·…·(2n+1)) below
    /// x = 8, with digits added for the cancellation in 1 − erf, and the
    /// continued fraction erfc(x) = (e^{−x²}/√π)/(x + ½/(x + 1/(x + 3⁄2/(x + …))))
    /// from 8 up, at 70 digits.  The other sums the alternating Maclaurin
    /// series and Legendre's continued fraction for Γ(½, x²).  Every row
    /// rounds to the same f64 under both.  The rows run from the negative
    /// tail through 0 and the expansion-point changes at x√2 = 1 and 16 to
    /// the last non-subnormal values near x = 26.5.  The rows with x√2
    /// between 6 and 16 that are not whole numbers are where stopping the
    /// Taylor loop at the first pair that rounds away, or expanding about the
    /// nearest tabled point, erred by 10⁻¹³ to 10⁻¹⁰.
    const ERFC_REFERENCE: [(f64, f64); 40] = [
        (-6.0, 2.0),
        (-3.0, 1.9999779095030015),
        (-1.5, 1.9661051464753108),
        (-1.0, 1.8427007929497148),
        (-0.5, 1.5204998778130465),
        (-1e-3, 1.0011283787909693),
        (1e-8, 0.9999999887162083),
        (1e-3, 0.9988716212090307),
        (0.1, 0.887537083981715),
        (0.3, 0.6713732405408726),
        (0.5, 0.4795001221869535),
        (1.0 / SQRT_2, 0.31731050786291415),
        (1.0, 0.15729920705028513),
        (1.5, 0.033894853524689274),
        (2.0, 0.004677734981047266),
        (2.5, 0.0004069520174449589),
        (3.0, 2.209049699858544e-5),
        (3.5, 7.430983723414128e-7),
        (4.0, 1.541725790028002e-8),
        (4.2487959500800585, 1.8701121117354613e-9),
        (5.0, 1.537459794428035e-12),
        (5.80383063394864, 2.251730418521088e-16),
        (5.909482993094504, 6.418574097208344e-17),
        (6.0, 2.1519736712498913e-17),
        (7.0, 4.183825607779414e-23),
        (7.575785717799614, 8.770697888645215e-27),
        (8.0, 1.1224297172982926e-29),
        (9.10163796029539, 6.49903280844363e-38),
        (9.15703281636579, 2.349540636368718e-38),
        (10.0, 2.088487583762545e-45),
        (10.526128824768495, 4.051868761500087e-50),
        (10.603831571143823, 7.788256497690057e-51),
        (10.81915099945942, 7.576656310913164e-53),
        (16.0 / SQRT_2, 1.2777508801076465e-57),
        (12.0, 1.3562611692059042e-64),
        (15.0, 7.212994172451207e-100),
        (20.0, 5.395865611607901e-176),
        (25.0, 8.300172571196523e-274),
        (26.0, 5.663192408856143e-296),
        (26.5, 2.2109076642637343e-307),
    ];

    /// Φ(x) from the same two evaluations (erfc(−x/√2)/2 with x/√2 carried to
    /// working precision, and the tail Γ(½, x²/2)/(2√π)), rounded once to f64.
    /// Φ(−37) is the last row above the subnormal range; the rows between
    /// −16 and −6 that are not whole numbers are the arguments of the erfc
    /// table's flagged rows, times −√2, or where the nearest expansion
    /// point erred.
    const NORMAL_CDF_REFERENCE: [(f64, f64); 31] = [
        (-37.0, 5.725571222524577e-300),
        (-35.0, 1.1249107064724062e-268),
        (-30.0, 4.906713927148187e-198),
        (-20.0, 2.7536241186062337e-89),
        (-16.0, 6.388754400538087e-58),
        (-14.999392327022141, 3.704728464016343e-51),
        (-14.886194612848115, 2.0259201558278829e-50),
        (-14.886194143273975, 2.0259343807499975e-50),
        (-12.95, 1.1747703181843633e-38),
        (-12.871659843259536, 3.2495164042217336e-38),
        (-12.0, 1.776482112077679e-33),
        (-10.713778907744608, 4.3853489443225774e-27),
        (-10.0, 7.619853024160525e-24),
        (-8.357270995447399, 3.209287048604171e-17),
        (-8.207855996246606, 1.1258652092605346e-16),
        (-8.0, 6.220960574271784e-16),
        (-6.008704856359099, 9.350560558677275e-10),
        (-6.0, 9.86587645037698e-10),
        (-5.0, 2.866515718791939e-7),
        (-3.0, 0.0013498980316300946),
        (-2.0, 0.02275013194817921),
        (-1.0, 0.15865525393145705),
        (-0.5, 0.3085375387259869),
        (-1e-3, 0.49960105778608893),
        (1e-3, 0.500398942213911),
        (0.5, 0.6914624612740131),
        (1.0, 0.8413447460685429),
        (2.0, 0.9772498680518208),
        (3.0, 0.9986501019683699),
        (5.0, 0.9999997133484281),
        (8.0, 0.9999999999999993),
    ];

    /// Allowed relative error of `erfc`.  The largest observed against these
    /// references on the 121 489 arguments `erfc`'s documentation describes
    /// is 2.8·(1 + x²)·ε from 0 up and 2.3·ε below 0; this allows
    /// 6·(1 + x²)·ε from 0 up and 6·ε below, about twice as much.
    fn erfc_tolerance(x: f64) -> f64 {
        6.0 * (1.0 + x.max(0.0).powi(2)) * f64::EPSILON
    }

    /// Allowed relative error of `normal_cdf`.  The largest observed on the
    /// 123 323 arguments `normal_cdf`'s documentation describes is
    /// 2.5·(1 + x²/2)·ε below 0 and 2.3·ε from 0 up; this allows
    /// 5·(1 + x²/2)·ε below 0 and 5·ε from 0 up, about twice as much.
    fn normal_cdf_tolerance(x: f64) -> f64 {
        5.0 * (1.0 + 0.5 * x.min(0.0).powi(2)) * f64::EPSILON
    }

    #[test]
    fn erfc_matches_reference_values() {
        for (x, want) in ERFC_REFERENCE {
            let got = erfc(x);
            let rel = ((got - want) / want).abs();
            assert!(
                rel <= erfc_tolerance(x),
                "erfc({x}) = {got:e}, want {want:e}"
            );
        }
    }

    #[test]
    fn normal_cdf_matches_reference_values() {
        for (x, want) in NORMAL_CDF_REFERENCE {
            let got = normal_cdf(x);
            let rel = ((got - want) / want).abs();
            assert!(
                rel <= normal_cdf_tolerance(x),
                "Φ({x}) = {got:e}, want {want:e}"
            );
        }
    }

    #[test]
    fn erfc_and_normal_cdf_exact_values_and_limits() {
        assert_eq!(erfc(0.0), 1.0);
        assert_eq!(erfc(-0.0), 1.0);
        assert_eq!(normal_cdf(0.0), 0.5);
        assert_eq!(normal_cdf(-0.0), 0.5);
        assert_eq!(erfc(f64::INFINITY), 0.0);
        assert_eq!(erfc(f64::NEG_INFINITY), 2.0);
        assert_eq!(normal_cdf(f64::NEG_INFINITY), 0.0);
        assert_eq!(normal_cdf(f64::INFINITY), 1.0);
        assert!(erfc(f64::NAN).is_nan());
        assert!(normal_cdf(f64::NAN).is_nan());
        // Underflow: erfc is subnormal from x ≈ 26.55 and 0 from x ≈ 27.22;
        // Φ is subnormal below x ≈ −37.5 and 0 from x ≈ −38.49.
        assert_eq!(erfc(28.0), 0.0);
        assert_eq!(erfc(-28.0), 2.0);
        assert_eq!(erfc(f64::MAX), 0.0);
        assert_eq!(normal_cdf(-39.0), 0.0);
        assert_eq!(normal_cdf(39.0), 1.0);
        assert_eq!(normal_cdf(-f64::MAX), 0.0);
    }

    /// erfc(−x) = 2 − erfc(x), erfc(x) = 2Φ(−x√2) and Φ(x) + Φ(−x) = 1.
    #[test]
    fn erfc_and_normal_cdf_symmetry() {
        for i in 0..=40_000 {
            let x = f64::from(i) * 1e-3;
            assert_eq!(erfc(-x), 2.0 - erfc(x), "x = {x}");
            assert_eq!(erfc(x), 2.0 * normal_cdf(-x * SQRT_2), "x = {x}");
            let sum = normal_cdf(x) + normal_cdf(-x);
            assert!((sum - 1.0).abs() <= f64::EPSILON, "Φ(±{x}) sum to {sum}");
        }
    }

    #[test]
    fn erfc_and_normal_cdf_monotone_on_grid() {
        let mut prev = erfc(-6.0);
        for i in 1..=340_000 {
            let x = -6.0 + f64::from(i) * 1e-4;
            let y = erfc(x);
            assert!(y <= prev, "erfc rises at x = {x}");
            prev = y;
        }
        let mut prev = normal_cdf(-40.0);
        for i in 1..=800_000 {
            let x = -40.0 + f64::from(i) * 1e-4;
            let y = normal_cdf(x);
            assert!(y >= prev, "Φ falls at x = {x}");
            prev = y;
        }
    }

    /// Where the Taylor expansion point changes (x√2 = 2, 4, …, 14, and 16,
    /// where the asymptotic series takes over; for Φ at x = −2, …, −16) the
    /// two sides must not step backwards, even between adjacent floats.
    /// Φ's upper half is 1 − cPhi at the same points, so it follows.  Near
    /// x√2 = 1 and x = ±1 the functions move about an ulp per float and
    /// rounding reverses neighbours by up to 3 ulp, as libm's erfc does near
    /// x = 0.8; the grid test covers those.
    #[test]
    fn erfc_and_normal_cdf_monotone_across_expansion_points() {
        for k in 1..=8 {
            let x0 = f64::from(2 * k) / SQRT_2;
            let mut x = f64::from_bits(x0.to_bits() - 1_000);
            let mut prev = erfc(x);
            for _ in 0..2_000 {
                x = f64::from_bits(x.to_bits() + 1);
                let y = erfc(x);
                assert!(y <= prev, "erfc rises at x = {x:e}");
                prev = y;
            }
            let t0 = f64::from(2 * k);
            let mut t = f64::from_bits(t0.to_bits() - 1_000);
            let mut prev = normal_cdf(-t);
            for _ in 0..2_000 {
                t = f64::from_bits(t.to_bits() + 1);
                let y = normal_cdf(-t);
                assert!(y <= prev, "Φ rises at x = {:e}", -t);
                prev = y;
            }
        }
    }

    /// Arguments where stopping the Taylor loop at the first pair that rounds
    /// away erred most (x = −14.886194612848115 made Φ rise by 1.5 × 10⁻¹⁰
    /// over its neighbour).  Neither function may step backwards between
    /// neighbouring floats there.
    #[test]
    fn erfc_and_normal_cdf_monotone_where_early_stopping_erred() {
        const FLAGGED: [f64; 7] = [
            6.008704856359099,
            8.207855996246606,
            8.357270995447399,
            10.713778907744608,
            12.871659843259536,
            14.886194143273975,
            14.886194612848117,
        ];
        for u0 in FLAGGED {
            let mut u = f64::from_bits(u0.to_bits() - 10_000);
            let mut prev = normal_cdf(-u);
            for _ in 0..20_000 {
                u = f64::from_bits(u.to_bits() + 1);
                let y = normal_cdf(-u);
                assert!(y <= prev, "Φ rises at x = {:e}", -u);
                prev = y;
            }
            let x0 = u0 / SQRT_2;
            let mut x = f64::from_bits(x0.to_bits() - 10_000);
            let mut prev = erfc(x);
            for _ in 0..20_000 {
                x = f64::from_bits(x.to_bits() + 1);
                let y = erfc(x);
                assert!(y <= prev, "erfc rises at x = {x:e}");
                prev = y;
            }
        }
    }

    /// Two-sided p-values erfc(|z|/√2) rely on erfc(x) ≤ 1 for x ≥ 0.
    #[test]
    fn erfc_at_most_one_for_nonnegative_arguments() {
        let mut xs = vec![0.0, f64::from_bits(1), f64::MIN_POSITIVE, f64::EPSILON];
        // Every power of two from the smallest subnormal to 2, with neighbours.
        let mut p = f64::from_bits(1);
        while p <= 2.0 {
            xs.extend([
                p,
                f64::from_bits(p.to_bits() - 1),
                f64::from_bits(p.to_bits() + 1),
            ]);
            p *= 2.0;
        }
        xs.extend((1..=10_000_u64).map(f64::from_bits));
        xs.extend((0..=100_000).map(|i| f64::from(i) * 1e-8));
        xs.extend((0..=200_000).map(|i| f64::from(i) * 1e-5));
        for x in xs {
            assert!(erfc(x) <= 1.0, "erfc({x:e}) = {:e}", erfc(x));
            assert!(erfc(-x) >= 1.0, "erfc({:e}) = {:e}", -x, erfc(-x));
        }
    }

    #[test]
    fn normal_cdf_within_unit_interval() {
        let mut xs = vec![
            0.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            f64::EPSILON,
            f64::MAX,
        ];
        xs.push(f64::INFINITY);
        let mut p = f64::from_bits(1);
        while p.is_finite() {
            xs.push(p);
            p *= 2.0;
        }
        xs.extend((0..=40_000).map(|i| f64::from(i) * 1e-3));
        for x in xs {
            for v in [normal_cdf(x), normal_cdf(-x)] {
                assert!((0.0..=1.0).contains(&v), "Φ(±{x:e}) = {v:e}");
            }
        }
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
}
