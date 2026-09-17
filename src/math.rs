//! Special functions and distributions shared by the test suites.
//!
//! Each function names the mathematics it evaluates and where that comes
//! from.  ln Γ comes from rump; the fast Fourier transform from `rustfft`.

use rustfft::{num_complex::Complex, FftPlanner};
use std::f64::consts::{FRAC_PI_2, PI, SQRT_2};

// ── erfc and the normal distribution ──────────────────────────────────────────

/// Mills' ratio R(z) = (1 − Φ(z))/φ(z) at z = 0, 2, 4, …, 16, each the f64
/// nearest its value.  R(0) = √(π/2); a test recomputes the others from
/// Laplace's continued fraction.
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

/// ln √(2π): φ(x) = exp(−x²/2 − ln √(2π)).
#[allow(clippy::excessive_precision)]
const LN_SQRT_2PI: f64 = 0.91893853320467274178;

/// Largest tabled z; `mills_ratio` sums an asymptotic series beyond it.
const MILLS_TABLE_END: f64 = 16.0;

/// `mills_ratio` stops its Taylor series once a pair of terms, past the
/// order where the terms must shrink, is at most this fraction of the sum.
const MILLS_TAIL: f64 = f64::EPSILON / 8.0;

/// Mills' ratio R(x) = (1 − Φ(x))/φ(x) for x ≥ 0 (not NaN).
///
/// Up to x = 16 this is a Taylor series about a tabled point z, x = z + h,
/// the approach of G. Marsaglia, "Evaluating the Normal Distribution",
/// *Journal of Statistical Software* 11(4), 2004, p. 4: R′ = xR − 1 gives
/// R⁽ᵏ⁺¹⁾ = xR⁽ᵏ⁾ + kR⁽ᵏ⁻¹⁾, and the loop builds the coefficients
/// cₖ = R⁽ᵏ⁾(z)/k! two at a time.
///
/// The expansion point.  The part of a rounding error in R(z) or R′(z) that
/// is not a multiple of R is a multiple of e^{x²/2}, the solution of R′ = xR,
/// and grows by e^{zh + h²/2} on the way to z + h.  So z is the tabled point
/// at or above x (−2 < h ≤ 0), or z = 0 for x < 1, where that factor is at
/// most 1.
///
/// The stopping rule.  The recurrence cₖ₊₁ = (z·cₖ + cₖ₋₁)/(k + 1) runs
/// forward, so rounding in the early coefficients grows like the terms of
/// e^{z|h|} and cancels only over the whole alternating tail; stopping at the
/// first pair of terms that rounds away can stop where that error and the
/// true term cancel.  The recurrence gives
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

/// Upper normal tail 1 − Φ(x) = R(x)·φ(x) for x ≥ 0 (not NaN).  At x = 0
/// the product R(0)·exp(−ln √(2π)) rounds to exactly 0.5.
fn normal_upper_tail(x: f64) -> f64 {
    mills_ratio(x) * (-0.5 * x * x - LN_SQRT_2PI).exp()
}

/// Complementary error function, erfc(x) = 1 − erf(x).
///
/// erfc(x) = 2·(1 − Φ(x√2)) for x ≥ 0 and erfc(x) = 2 − erfc(−x) below 0,
/// with the normal tail from `mills_ratio`.
///
/// Accuracy, as the largest error observed against 70-digit references on
/// 1 121 489 arguments in [−11.3, 26.5] (1 000 000 uniformly random, and
/// 121 489 packed near the expansion points and near arguments where a
/// naive stopping rule errs).  ε is `f64::EPSILON`.
///
/// - From 0 to 26.5, relative error 3.6·(1 + x²)·ε (3.54 at
///   x = 0.1958876234856557): at most 1.2 × 10⁻¹⁵ below x = 1,
///   7.0 × 10⁻¹⁵ below 4, 9.6 × 10⁻¹⁴ below 16 and 3.3 × 10⁻¹³ up to 26.5.
///   The x² comes from exp(−u²/2) at the rounded u = x√2.
/// - Below 0, relative error 2.6·ε (at x = −0.12261875751971507) and
///   absolute error 6.8 × 10⁻¹⁶.
///
/// These are maxima of rounding noise, and denser sampling keeps finding
/// slightly larger values near 0, so the tests allow about twice them.
/// Results are subnormal from x ≈ 26.55, where the relative error grows to
/// order 1, and 0 from x ≈ 27.22.
///
/// erfc(±0) = 1 exactly, 0 ≤ erfc(x) ≤ 1 for x ≥ 0 and 1 ≤ erfc(x) ≤ 2 below,
/// so a two-sided p-value erfc(|z|/√2) never exceeds 1.  erfc(+∞) = 0,
/// erfc(−∞) = 2 and erfc(NaN) = NaN.
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
/// Φ(x) = 1 − Φ(−x) evaluated as the upper tail below 0, and 1 minus the
/// upper tail from 0 up, without [`erfc`]'s x√2 rescaling, so lower-tail
/// values keep relative accuracy.  Against the same references on 1 523 325
/// arguments in [−37.5, 40], the largest relative error observed below 0 is
/// 3.4·(1 + x²/2)·ε, at x = −0.2940544440351558 (3.4 × 10⁻¹⁵ on [−6, 0),
/// 1.6 × 10⁻¹⁴ on [−16, −6) and 9.4 × 10⁻¹⁴ on [−37.5, −16)).  The largest
/// absolute error from 0 up is 3.4 × 10⁻¹⁶ (relative 2.7·ε).  As for
/// [`erfc`], the tests allow about twice these.  Results are subnormal below
/// x ≈ −37.5 and 0 from x ≈ −38.49.
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

/// Standard normal quantile Φ⁻¹(p) for 0 < p < 1.
///
/// Solves ln Φ(x) = ln q for the lower tail q = min(p, 1 − p) by Newton's
/// method, whose derivative φ(x)/Φ(x) is well conditioned far into the tail,
/// inside a bisection bracket on [−39, 0], then reflects for p > 1/2.
/// Φ⁻¹(0) = −∞, Φ⁻¹(1) = +∞, and NaN outside [0, 1].
#[must_use]
pub fn normal_quantile(p: f64) -> f64 {
    if !(0.0..=1.0).contains(&p) {
        return f64::NAN;
    }
    if p == 0.0 {
        return f64::NEG_INFINITY;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }
    let q = p.min(1.0 - p);
    let target = q.ln();
    let (mut lo, mut hi) = (-39.0f64, 0.0f64);
    let mut x = (-(-2.0 * target).sqrt()).max(lo);
    for _ in 0..200 {
        let cdf = normal_cdf(x);
        if cdf.ln() > target {
            hi = x;
        } else {
            lo = x;
        }
        let density = (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt();
        let mut next = x - (cdf.ln() - target) * cdf / density;
        if !(next > lo && next < hi) {
            next = 0.5 * (lo + hi);
        }
        if (next - x).abs() <= 4.0 * f64::EPSILON * x.abs().max(1.0) {
            x = next;
            break;
        }
        x = next;
    }
    if p > 0.5 {
        -x
    } else {
        x
    }
}

// ── ln Γ ──────────────────────────────────────────────────────────────────────

/// Natural logarithm of the gamma function, ln Γ(x), for x > 0: rump's
/// Lanczos evaluation (g = 7, nine terms).  rump measures its error below
/// 5·10⁻¹⁵ absolute on [0.1, 3], where ln Γ passes through zero, and below
/// 2·10⁻¹⁵ relative elsewhere.
#[must_use]
pub fn lgamma(x: f64) -> f64 {
    rump::number_theory::ln_gamma(x)
}

// ── Regularised incomplete gamma ──────────────────────────────────────────────

/// Relative size at which the incomplete-gamma series and continued fraction
/// stop.
const GAMMA_TOLERANCE: f64 = 1e-15;

/// Regularised upper incomplete gamma function Q(a, x) = Γ(a, x)/Γ(a), the
/// survival function of χ² with 2a degrees of freedom at 2x.
///
/// With the prefactor xᵃe⁻ˣ/Γ(a) taken in logarithms:
///
/// - for x < a + 1, Q = 1 − P, with P(a, x) = xᵃe⁻ˣ/Γ(a) · Σₖ xᵏ/(a(a + 1)…(a + k))
///   (DLMF 8.7.1), whose terms decrease once k > x − a;
/// - otherwise the continued fraction Γ(a, x) = xᵃe⁻ˣ ·
///   1/(x + 1 − a − 1·(1 − a)/(x + 3 − a − 2·(2 − a)/(x + 5 − a − …)))
///   (DLMF 8.9.2), evaluated by the modified Lentz algorithm (W. J. Lentz,
///   *Applied Optics* 15 (1976); I. J. Thompson and A. R. Barnett, *J.
///   Computational Physics* 64 (1986));
/// - for a < 1/10 and x < a + 1, where Q is small because a is and 1 − P would
///   lose it to rounding, Q is computed directly from
///   γ(a, x) = xᵃ Σₖ (−x)ᵏ/(k!(a + k)) (DLMF 8.7.3):
///   Q = −expm1(u) − a·eᵘ·Σ_{k≥1} (−x)ᵏ/(k!(a + k)), u = a·ln x − ln Γ(1 + a),
///   with ln Γ(1 + a) from its Taylor series (DLMF 5.7.3), which stays
///   accurate relative to a however small a is.  Neither term cancels: the
///   sum is negative.
///
/// Both need O(√a) terms near x ≈ a, the bulk of a χ² distribution, so the
/// iteration budget grows with √a.  Returns NaN if a is not a positive finite
/// number, x < 0, either is NaN, or the expansion does not converge.
#[must_use]
pub fn igamc(a: f64, x: f64) -> f64 {
    if !(a > 0.0 && a.is_finite() && x >= 0.0) {
        return f64::NAN;
    }
    if x == 0.0 {
        return 1.0;
    }
    if x.is_infinite() {
        return 0.0;
    }
    if a < SMALL_SHAPE && x < a + 1.0 {
        return small_shape_upper_gamma(a, x).unwrap_or(f64::NAN);
    }
    let ln_prefactor = a * x.ln() - x - lgamma(a);
    if x < a + 1.0 {
        lower_gamma_series(a, x, ln_prefactor).map_or(f64::NAN, |p| 1.0 - p)
    } else {
        upper_gamma_fraction(a, x, ln_prefactor).unwrap_or(f64::NAN)
    }
}

/// Terms or convergents the incomplete-gamma expansions may take.
fn gamma_iterations(a: f64) -> u64 {
    500 + (10.0 * a.sqrt()) as u64
}

/// P(a, x) by the series of DLMF 8.7.1, or `None` without convergence.
/// Shapes below which [`igamc`] computes Q directly rather than as 1 − P.
const SMALL_SHAPE: f64 = 0.1;

/// Euler's constant γ.
const EULER_GAMMA: f64 = 0.577_215_664_901_532_9;

/// ζ(k) for k = 2 … `ZETA_TERMS` + 1: the sum of n⁻ᵏ to n = 63 plus the
/// Euler–Maclaurin tail from 64, through its B₈ term (DLMF 2.10.1).
fn zeta_table() -> &'static [f64] {
    static TABLE: std::sync::OnceLock<Vec<f64>> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        const N: f64 = 64.0;
        // B₂ⱼ/(2j)! for j = 1 … 4.
        const BERNOULLI_OVER_FACTORIAL: [f64; 4] =
            [1.0 / 12.0, -1.0 / 720.0, 1.0 / 30_240.0, -1.0 / 1_209_600.0];
        (2..ZETA_TERMS + 2)
            .map(|k| {
                let s = k as f64;
                let head: f64 = (1..64).rev().map(|n| (n as f64).powf(-s)).sum();
                let mut tail = N.powf(1.0 - s) / (s - 1.0) + 0.5 * N.powf(-s);
                // The j-th correction is B₂ⱼ/(2j)! · s(s + 1)…(s + 2j − 2) · N^(−s−2j+1).
                let mut rising = s;
                for (j, b) in BERNOULLI_OVER_FACTORIAL.iter().enumerate() {
                    if j > 0 {
                        rising *= (s + 2.0 * j as f64 - 1.0) * (s + 2.0 * j as f64);
                    }
                    tail += b * rising * N.powf(-s - 2.0 * j as f64 - 1.0);
                }
                head + tail
            })
            .collect()
    })
}

/// Terms of the ln Γ(1 + a) series: aᵏ < 10⁻¹⁷ for a < 1/10.
const ZETA_TERMS: usize = 17;

/// ln Γ(1 + a) = −γa + Σ_{k≥2} (−1)ᵏ ζ(k) aᵏ/k (DLMF 5.7.3), for 0 < a < 1/10.
fn ln_gamma_one_plus_small(a: f64) -> f64 {
    let mut sum = 0.0;
    // (−a)ᵏ, starting from k = 1.
    let mut power = -a;
    for (i, zeta) in zeta_table().iter().enumerate() {
        let k = (i + 2) as f64;
        power *= -a;
        sum += zeta * power / k;
    }
    sum - EULER_GAMMA * a
}

/// Q(a, x) for 0 < a < 1/10 and 0 < x < a + 1; see [`igamc`].
fn small_shape_upper_gamma(a: f64, x: f64) -> Option<f64> {
    let u = a * x.ln() - ln_gamma_one_plus_small(a);
    // Σ_{k≥1} (−x)ᵏ/(k!(a + k)); x < 1.1, so the terms fall at least as fast
    // as xᵏ/k!.
    let mut power = 1.0;
    let mut sum = 0.0;
    for k in 1..=60 {
        let kf = k as f64;
        power *= -x / kf;
        let term = power / (a + kf);
        sum += term;
        if !sum.is_finite() {
            return None;
        }
        if term.abs() <= sum.abs() * GAMMA_TOLERANCE {
            let q = -u.exp_m1() - a * u.exp() * sum;
            return q.is_finite().then_some(q);
        }
    }
    None
}

fn lower_gamma_series(a: f64, x: f64, ln_prefactor: f64) -> Option<f64> {
    let mut term = 1.0 / a;
    let mut sum = term;
    let mut denominator = a;
    for _ in 0..gamma_iterations(a) {
        denominator += 1.0;
        term *= x / denominator;
        sum += term;
        if !sum.is_finite() {
            return None;
        }
        if term.abs() <= sum.abs() * GAMMA_TOLERANCE {
            return Some(sum * ln_prefactor.exp());
        }
    }
    None
}

/// Q(a, x) by the continued fraction of DLMF 8.9.2, or `None` without
/// convergence.
///
/// The fraction is b₀ + a₁/(b₁ + a₂/(b₂ + …)) with b₀ = 0, a₁ = 1,
/// bⱼ = x + 2j − 1 − a and aⱼ₊₁ = −j(j − a).  The modified Lentz algorithm
/// keeps the ratios Cⱼ = fⱼ/fⱼ₋₁ and Dⱼ = qⱼ₋₁/qⱼ of successive numerators
/// and denominators, replacing a zero by a tiny number, and multiplies the
/// convergent by CⱼDⱼ until that factor is 1 to within the tolerance.
fn upper_gamma_fraction(a: f64, x: f64, ln_prefactor: f64) -> Option<f64> {
    const TINY: f64 = 1e-300;
    let nonzero = |v: f64| if v.abs() < TINY { TINY } else { v };
    let mut value = TINY;
    let mut c = value;
    let mut d = 0.0;
    for j in 1..=gamma_iterations(a) {
        let jf = j as f64;
        let numerator = if j == 1 {
            1.0
        } else {
            -(jf - 1.0) * (jf - 1.0 - a)
        };
        let denominator = x + 2.0 * jf - 1.0 - a;
        d = 1.0 / nonzero(denominator + numerator * d);
        c = nonzero(denominator + numerator / c);
        let factor = c * d;
        value *= factor;
        if !value.is_finite() {
            return None;
        }
        if (factor - 1.0).abs() <= GAMMA_TOLERANCE {
            return Some(value * ln_prefactor.exp());
        }
    }
    None
}

// ── Kolmogorov–Smirnov ────────────────────────────────────────────────────────

/// Two-sided Kolmogorov–Smirnov test that `samples` are drawn from U(0, 1):
/// the p-value of D = supₓ |Fₙ(x) − x| by [`ks_pvalue`].
///
/// Returns NaN, the crate's insufficient-data value, if any sample is NaN.
/// Otherwise the slice is sorted in place.
#[must_use]
pub fn ks_test(samples: &mut [f64]) -> f64 {
    ks_pvalue(ks_statistic(samples), samples.len())
}

/// The Kolmogorov–Smirnov distance D = sup |F_n(x) − x| of `samples` from
/// Uniform(0, 1), sorting them in place; NaN if any sample is NaN.
#[must_use]
pub fn ks_statistic(samples: &mut [f64]) -> f64 {
    if samples.iter().any(|x| x.is_nan()) {
        return f64::NAN;
    }
    samples.sort_by(f64::total_cmp);
    let nf = samples.len() as f64;
    samples
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let f_hi = (i + 1) as f64 / nf;
            let f_lo = i as f64 / nf;
            (f_hi - x).abs().max((x - f_lo).abs())
        })
        .fold(0.0_f64, f64::max)
}

/// Largest n for which [`ks_pvalue`] evaluates the exact distribution.
const KS_EXACT_MAX_N: usize = 4_999;

/// P(Dₙ ≥ d), the upper tail of the two-sided Kolmogorov–Smirnov statistic
/// for a sample of n.
///
/// For n ≤ 4 999 this is the exact distribution by Durbin's matrix formula,
/// as G. Marsaglia, W. W. Tsang and J. Wang present it in "Evaluating
/// Kolmogorov's Distribution", *Journal of Statistical Software* 8(18), 2003,
/// with their approximation for the far right tail.  For larger n it is
/// Kolmogorov's limiting series at Stephens' modified argument
/// d(√n + 0.12 + 0.11/√n) (M. A. Stephens, *JASA* 69 (1974)).
///
/// Returns NaN for n = 0.
#[must_use]
pub fn ks_pvalue(d: f64, n: usize) -> f64 {
    if n == 0 {
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

/// Kolmogorov's series 2 Σₖ (−1)^(k−1) e^(−2k²s²) at Stephens' argument s.
fn ks_pvalue_asymptotic(d: f64, n: usize) -> f64 {
    let root_n = (n as f64).sqrt();
    let s = d * (root_n + 0.12 + 0.11 / root_n);
    let mut sum = 0.0_f64;
    for k in 1..=100u32 {
        let kf = f64::from(k);
        let term = (-2.0 * kf * kf * s * s).exp();
        sum += if k % 2 == 1 { term } else { -term };
        // A term that underflows to 0 has converged: for huge s every term
        // vanishes and the p-value is 0.
        if term <= 1e-15 * sum.abs() {
            return (2.0 * sum).clamp(0.0, 1.0);
        }
    }
    // For very small s the terms do not fall within 100 steps; there the
    // p-value is 1 to within rounding.
    1.0
}

/// Binary exponent by which [`ks_pvalue_exact`] rescales its matrix powers.
const KS_SCALE_BITS: i32 = 400;

/// P(Dₙ ≥ d) by Durbin's formula.  With k = ⌊nd⌋ + 1, m = 2k − 1 and
/// h = k − nd, let H be the m × m matrix with entries
/// [i − j + 1 ≥ 0]/(i − j + 1)!, except that the first column subtracts
/// hⁱ⁺¹/(i + 1)!, the last row subtracts h^(m−j)/(m − j)!, and the corner
/// adds (2h − 1)^m/m! when 2h > 1.  Then P(Dₙ < d) = n!/nⁿ · (Hⁿ)ₖₖ.
///
/// For nd² > 7.24, or > 3.76 with n > 99, the tail is instead
/// 2 exp(−(2.000071 + 0.331/√n + 1.409/n)·nd²) (Marsaglia, Tsang and Wang
/// 2003, §3).
fn ks_pvalue_exact(d: f64, n: usize) -> f64 {
    let nf = n as f64;
    let s = d * d * nf;
    if s > 7.24 || (s > 3.76 && n > 99) {
        return 2.0 * (-(2.000_071 + 0.331 / nf.sqrt() + 1.409 / nf) * s).exp();
    }
    let k = (nf * d).floor() as usize + 1;
    let m = 2 * k - 1;
    let h = k as f64 - nf * d;
    let factorial = |r: usize| (1..=r).map(|g| g as f64).product::<f64>();
    let mut matrix = vec![0.0; m * m];
    for i in 0..m {
        for j in 0..=(i + 1).min(m - 1) {
            let mut entry = 1.0;
            if j == 0 {
                entry -= h.powi(i as i32 + 1);
            }
            if i == m - 1 {
                entry -= h.powi((m - j) as i32);
                if j == 0 && 2.0 * h > 1.0 {
                    entry += (2.0 * h - 1.0).powi(m as i32);
                }
            }
            matrix[i * m + j] = entry / factorial(i + 1 - j);
        }
    }
    let (power, exponent) = scaled_matrix_power(&matrix, m, n);
    // n!/nⁿ in logarithms, and the binary exponent of the power.
    let ln_scale = lgamma(nf + 1.0) - nf * nf.ln() + f64::from(exponent) * std::f64::consts::LN_2;
    let below = power[(k - 1) * m + (k - 1)] * ln_scale.exp();
    (1.0 - below).clamp(0.0, 1.0)
}

/// A^power for the m × m matrix `a`, as (matrix, e) with A^power = matrix·2ᵉ;
/// the matrix is rescaled whenever an entry exceeds 2^[`KS_SCALE_BITS`].
fn scaled_matrix_power(a: &[f64], m: usize, power: usize) -> (Vec<f64>, i32) {
    let mut result: Option<(Vec<f64>, i32)> = None;
    let mut base = (a.to_vec(), 0i32);
    let mut remaining = power;
    while remaining > 0 {
        if remaining & 1 == 1 {
            result = Some(match result {
                None => base.clone(),
                Some((r, e)) => rescale(matrix_product(&r, &base.0, m), e + base.1),
            });
        }
        remaining >>= 1;
        if remaining > 0 {
            base = rescale(matrix_product(&base.0, &base.0, m), 2 * base.1);
        }
    }
    result.expect("power is at least 1")
}

/// The m × m product a·b.
fn matrix_product(a: &[f64], b: &[f64], m: usize) -> Vec<f64> {
    let mut c = vec![0.0; m * m];
    for i in 0..m {
        for l in 0..m {
            let ail = a[i * m + l];
            if ail != 0.0 {
                for j in 0..m {
                    c[i * m + j] += ail * b[l * m + j];
                }
            }
        }
    }
    c
}

/// Divides `matrix` by 2^[`KS_SCALE_BITS`] if any entry exceeds it, adding
/// that to the exponent.
fn rescale(mut matrix: Vec<f64>, exponent: i32) -> (Vec<f64>, i32) {
    let limit = 2f64.powi(KS_SCALE_BITS);
    if matrix.iter().any(|x| x.abs() > limit) {
        let factor = 2f64.powi(-KS_SCALE_BITS);
        matrix.iter_mut().for_each(|x| *x *= factor);
        return (matrix, exponent + KS_SCALE_BITS);
    }
    (matrix, exponent)
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

/// f(z, j), the j-th term of the series for ADinf (Marsaglia and Marsaglia
/// 2004, §2, p. 2).
///
/// With t = (4j + 1)²π²/(8z) it sums c₀ + c₁(z/8) + c₂(z/8)²/2! + …, where
/// c₀ = π e⁻ᵗ (2t)^(−1/2), c₁ = π (π/2)^(1/2) erfc(√t) and
/// cₙ₊₁ = ((n − ½ − t)cₙ + t cₙ₋₁)/n.  Terms with t > 150 are below 10⁻⁶⁵
/// and are taken as 0.
fn ad_inf_term(z: f64, j: usize) -> f64 {
    let k = (4 * j + 1) as f64;
    let t = k * k * (PI * PI / 8.0) / z;
    if t > 150.0 {
        return 0.0;
    }
    let mut a = PI / SQRT_2 * (-t).exp() / t.sqrt();
    let mut b = PI * FRAC_PI_2.sqrt() * erfc(t.sqrt());
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
/// about 15 digits (Marsaglia and Marsaglia 2004, §2, pp. 2–3).
///
/// ADinf(z) = (1/z) Σⱼ C(−½, j) (4j + 1) f(z, j).  Below z = 0.01 it returns
/// 0 (ADinf(0.01) ≈ 5.3·10⁻⁵³).  Above `AD_INF_Z_MAX` = 30 it returns 1: the
/// alternating series cancels ever larger terms as z grows, while
/// ADinf(30) = 1 − 1.8·10⁻¹⁴.
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
/// Pr(Aₙ < z): the three polynomials g₁, g₂ and g₃ of Marsaglia and
/// Marsaglia (2004), §3, p. 4.
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
/// and 128 and ±5·10⁻⁴ for other n (p. 4).
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
/// before the tail does.  A simulation of 2·10⁹ samples each put ADinf +
/// errfix up to 1.3·10⁻³ from
/// Pr(Aₙ < z) for n = 4, 1.3·10⁻² for n = 2 and 5.4·10⁻² for n = 1 (against
/// the exact distribution, p. 1), and its tail 15% high for n = 4 just below
/// the switch.
///
/// # Beyond the paper
///
/// - Above the switch the upper tail is the scaled limiting tail described
///   above, and n < 8 returns NaN.
/// - ADinf is taken as 1 for z > 30, where its alternating series loses
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

// ── Pearson chi-square with pooled weak cells ───────────────────────────────

/// Pearson χ² on binned counts, pooling the cells that expect too few.
///
/// - A cell whose expected count is at least `cutoff` is scored on its own.
/// - Every other cell, wherever it lies, joins one pooled cell.
/// - If the pool expects at least `cutoff` it is scored as a cell of its own;
///   otherwise it is merged into the scored cell with the smallest
///   expectation.  Either way every observation is scored exactly once.
/// - df = (number of scored cells) − 1.
///
/// Returns `Some((p_value, df, chi_sq))`, or `None` if the slices differ in
/// length, are empty, or fewer than two cells are scored.
#[must_use]
pub fn chi_square_pooled(
    observed: &[u32],
    expected: &[f64],
    cutoff: f64,
) -> Option<(f64, usize, f64)> {
    if observed.len() != expected.len() || observed.is_empty() {
        return None;
    }
    let mut cells: Vec<(f64, f64)> = Vec::new();
    let (mut pool_expected, mut pool_observed) = (0.0, 0.0);
    for (&o, &e) in observed.iter().zip(expected) {
        if e >= cutoff {
            cells.push((e, f64::from(o)));
        } else {
            pool_expected += e;
            pool_observed += f64::from(o);
        }
    }
    if pool_expected >= cutoff {
        cells.push((pool_expected, pool_observed));
    } else if pool_expected > 0.0 || pool_observed > 0.0 {
        let smallest = cells.iter_mut().min_by(|a, b| a.0.total_cmp(&b.0))?;
        smallest.0 += pool_expected;
        smallest.1 += pool_observed;
    }
    if cells.len() < 2 {
        return None;
    }
    let chi_sq: f64 = cells.iter().map(|&(e, o)| (o - e).powi(2) / e).sum();
    let df = cells.len() - 1;
    Some((chi2_pvalue(chi_sq, df), df, chi_sq))
}

// ── Pearson chi-square with pooled tails ────────────────────────────────────

/// Pearson χ² of `observed` against `expected` for a distribution over
/// ordered cells, pooling each tail inward until the cell at that end
/// expects at least `min_expected`.
///
/// Every observation stays in exactly one cell, and the degrees of freedom
/// are one less than the number of cells left.  Returns `Some((χ², df))`, or
/// `None` if the slices differ in length or fewer than two cells remain.
#[must_use]
pub fn chi_square_pooled_tails(
    observed: &[f64],
    expected: &[f64],
    min_expected: f64,
) -> Option<(f64, usize)> {
    if observed.len() != expected.len() {
        return None;
    }
    let mut cells: Vec<(f64, f64)> = expected
        .iter()
        .copied()
        .zip(observed.iter().copied())
        .collect();
    while cells.len() > 1 && cells[0].0 < min_expected {
        let (e, o) = cells.remove(0);
        cells[0].0 += e;
        cells[0].1 += o;
    }
    while cells.len() > 1 && cells[cells.len() - 1].0 < min_expected {
        let (e, o) = cells.pop().expect("more than one cell");
        let last = cells.len() - 1;
        cells[last].0 += e;
        cells[last].1 += o;
    }
    if cells.len() < 2 {
        return None;
    }
    let chi = cells.iter().map(|&(e, o)| (o - e).powi(2) / e).sum();
    Some((chi, cells.len() - 1))
}

// ── Discrete probability mass functions ─────────────────────────────────────

/// Binomial PMF: P(X = k) for X ~ Binomial(n, p), with `k ≤ n`.
///
/// Evaluated in log space through [`lgamma`], so it stays finite for large n
/// (C(n, k) itself overflows `f64` past n ≈ 1 030).  The degenerate `p ≤ 0`
/// and `p ≥ 1` cases return the exact point masses instead of taking `ln 0`.
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

    #[test]
    fn pooled_chi_square_scores_every_observation_once() {
        // Cells 0 and 3 are weak; their pool (observed 5, expected 2 + 3 = 5)
        // reaches the cutoff, so it is scored as a third cell.
        let (p, df, chi) = chi_square_pooled(&[1, 12, 9, 4], &[2.0, 10.0, 10.0, 3.0], 5.0).unwrap();
        assert_eq!(df, 2);
        assert!((chi - 0.5).abs() < 1e-15, "χ² = {chi}");
        assert!((p - (-0.25f64).exp()).abs() < 1e-12, "p = {p}");

        // The pool expects 2 + 2 = 4, below the cutoff, so it joins the
        // smallest scored cell: (12 + 3 − 14)²/14 + (9 − 10)²/10.
        let (_, df, chi) = chi_square_pooled(&[3, 12, 9, 0], &[2.0, 10.0, 10.0, 2.0], 5.0).unwrap();
        assert_eq!(df, 1);
        let want = 1.0 / 14.0 + 0.1;
        assert!((chi - want).abs() < 1e-15, "χ² = {chi}");

        // One strong cell and a pool below the cutoff leave one cell.
        assert!(chi_square_pooled(&[7, 30, 1], &[3.0, 30.0, 1.0], 5.0).is_none());
        assert!(chi_square_pooled(&[], &[], 5.0).is_none());
    }

    /// Every observation counts once, so the tail pooling rejects a stream
    /// whose excess sits in weak cells whose pool is itself weak.
    #[test]
    fn pooled_chi_square_sees_excess_in_a_weak_pool() {
        let expected = [100.0, 100.0, 1.0, 1.0];
        let (p, _, _) = chi_square_pooled(&[70, 70, 30, 30], &expected, 5.0).unwrap();
        assert!(p < 1e-3, "p = {p}");
    }

    #[test]
    fn pooled_tails_keep_every_observation() {
        let (chi, df) = chi_square_pooled_tails(
            &[1.0, 2.0, 50.0, 49.0, 3.0],
            &[2.0, 3.0, 45.0, 45.0, 5.0],
            5.0,
        )
        .unwrap();
        // Cells: (5, 3), (45, 50), (45, 49), (5, 3) — the left tail pooled.
        assert_eq!(df, 3);
        let want = 4.0 / 5.0 + 25.0 / 45.0 + 16.0 / 45.0 + 4.0 / 5.0;
        assert!((chi - want).abs() < 1e-12, "χ² = {chi}");
        assert!(chi_square_pooled_tails(&[1.0], &[1.0], 5.0).is_none());
    }

    // Reference values are exact rationals C(n,k)·pᵏ·(1−p)ⁿ⁻ᵏ rounded once to
    // f64, independent of the lgamma evaluation.
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
    /// independent 70-digit decimal evaluations at the exact binary argument.
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
    /// point erred.  −0.2848151991652159 and −0.2014286549487616 are among
    /// the largest relative errors a random scan found near 0, 3.00 and
    /// 3.17·(1 + x²/2)·ε.
    const NORMAL_CDF_REFERENCE: [(f64, f64); 33] = [
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
        (-0.2848151991652159, 0.38789286352684327),
        (-0.2014286549487616, 0.42018170547733735),
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
    /// references on the 1 121 489 arguments `erfc`'s documentation
    /// describes is 3.54·(1 + x²)·ε from 0 up (at x = 0.1958876234856557)
    /// and 2.56·ε below 0 (at x = −0.12261875751971507).  This allows
    /// 7·(1 + x²)·ε from 0 up and 7·ε below, about twice as much.
    fn erfc_tolerance(x: f64) -> f64 {
        7.0 * (1.0 + x.max(0.0).powi(2)) * f64::EPSILON
    }

    /// Allowed relative error of `normal_cdf`.  The largest observed on the
    /// 1 523 325 arguments `normal_cdf`'s documentation describes is
    /// 3.36·(1 + x²/2)·ε below 0 (at x = −0.2940544440351558) and 2.65·ε
    /// from 0 up (at x = 0.11809722168467235).  This allows 7·(1 + x²/2)·ε
    /// below 0 and 7·ε from 0 up, about twice as much.
    fn normal_cdf_tolerance(x: f64) -> f64 {
        7.0 * (1.0 + 0.5 * x.min(0.0).powi(2)) * f64::EPSILON
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
    /// Φ's upper half is 1 minus the same tail, so it follows.  Near
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

    // Values of Q(a, x) to 16 digits.
    #[test]
    fn igamc_golden_values() {
        // Series branch (x < a + 1)
        assert!((igamc(3.0, 2.5) - 0.5438131158833297).abs() < 1e-10);
        // Continued-fraction branch (x ≥ a + 1)
        assert!((igamc(0.5, 2.0) - 0.045500263896358445).abs() < 1e-10);
        // Chi-square mapping: P(χ²₅ > 11.0705) ≈ 0.05
        assert!((chi2_pvalue(11.0705, 5) - 0.04999995542804364).abs() < 1e-9);
    }

    // Large shape parameters need O(√a) iterations.
    #[test]
    fn igamc_small_shapes() {
        // R 4.2.0, pgamma(x, a, lower.tail = FALSE).
        for (a, x, q) in [
            (1e-310, 1e-310, 7.132_241_631_632_852e-308),
            (1e-300, 1e-300, 6.901_983_122_332_726e-298),
            (1e-100, 1e-100, 2.296_812_936_345_03e-98),
            (1e-20, 1e-20, 4.547_448_619_497_939e-19),
            (1e-5, 1e-5, 1.093_513_009_563_298_9e-4),
            (0.05, 0.03, 0.139_200_351_040_582_62),
            (0.09, 1.05, 0.019_869_336_725_188_227),
            (1e-8, 0.5, 5.597_735_977_099_587e-9),
            (0.01, 1e-10, 0.201_138_908_566_394_8),
        ] {
            let got = igamc(a, x);
            assert!(
                (got - q).abs() <= 1e-12 * q,
                "Q({a}, {x}) = {got}, want {q}"
            );
        }
        // Q(a, x) → a·E₁(x) as a → 0; E₁(1) = 0.219 383 934 395 520 27.
        let q = igamc(1e-20, 1.0);
        assert!((q / 1e-20 - 0.219_383_934_395_520_27).abs() < 1e-12, "{q}");
        // The branches agree where they meet, at a = 1/10.
        for x in [0.01, 0.5, 1.09] {
            let below = igamc(0.1 * (1.0 - 1e-12), x);
            let at = igamc(0.1, x);
            assert!((below - at).abs() < 1e-11, "x = {x}: {below} vs {at}");
        }
    }

    /// ζ(2) = π²/6 and ζ(4) = π⁴/90.
    #[test]
    fn zeta_table_matches_closed_forms() {
        let pi = std::f64::consts::PI;
        assert!((zeta_table()[0] - pi * pi / 6.0).abs() < 4e-16);
        assert!((zeta_table()[2] - pi.powi(4) / 90.0).abs() < 4e-16);
    }

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

    // Values of the exact two-sided KS distribution.  A triangular H (without
    // its superdiagonal) would give p ≈ 1 for every input.
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

    // The asymptotic series' two extremes: huge s (every term underflows,
    // catastrophic D) gives 0; tiny s (terms stay O(1)) gives 1.
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

    /// A NaN sample makes the test insufficient rather than panicking.
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

    /// ADinf pinned across its range, from 10⁻⁵³ to 1 − 10⁻¹⁴.  The relative
    /// tolerance leaves room for libm's `exp` and `sqrt` to differ by an ulp.
    #[test]
    fn ad_inf_is_pinned_across_its_range() {
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
                "ADinf({z}) = {got}, pinned {want}"
            );
        }
    }

    /// Mills' ratio at the tabled points from Laplace's continued fraction
    /// R(z) = 1/(z + 1/(z + 2/(z + 3/(z + …)))), evaluated from the tail up,
    /// and R(0) = √(π/2).
    #[test]
    fn mills_ratio_table_matches_the_continued_fraction() {
        assert!((MILLS_RATIO_AT_EVEN[0] - (PI / 2.0).sqrt()).abs() <= f64::EPSILON);
        for (j, &r) in MILLS_RATIO_AT_EVEN.iter().enumerate().skip(1) {
            let z = 2.0 * j as f64;
            let mut fraction = z;
            for k in (1..20_000).rev() {
                fraction = z + k as f64 / fraction;
            }
            let want = 1.0 / fraction;
            assert!(
                (r - want).abs() <= 4.0 * f64::EPSILON * want,
                "R({z}) = {r} vs {want}"
            );
        }
    }

    /// errfix pinned on each of its three polynomial pieces for several n.
    /// Above x = 0.8 the terms of g₃ cancel from about 10³ to 10⁻³, so a
    /// fused multiply-add can move the result by up to 5·10⁻¹⁴.
    #[test]
    fn ad_errfix_is_pinned() {
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
                "errfix({n}, {x}) = {got}, pinned {want}"
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

    /// Φ⁻¹ inverts Φ from the centre to p = 10⁻³⁰⁰, and matches
    /// Φ⁻¹(0.975) = 1.959963984540054.
    #[test]
    fn normal_quantile_inverts_the_cdf() {
        assert!((super::normal_quantile(0.975) - 1.959_963_984_540_054).abs() < 1e-14);
        assert!(super::normal_quantile(0.5).abs() < 1e-15);
        for e in [1, 2, 5, 10, 30, 100, 300] {
            let p = 10f64.powi(-e);
            let x = super::normal_quantile(p);
            let back = super::normal_cdf(x);
            assert!(
                (back - p).abs() <= 1e-12 * p,
                "p = {p}: x = {x}, Φ(x) = {back}"
            );
            let upper = 1.0 - p.max(1e-15);
            assert_eq!(
                super::normal_quantile(upper),
                -super::normal_quantile(1.0 - upper)
            );
        }
        assert!(super::normal_quantile(-0.1).is_nan());
        assert_eq!(super::normal_quantile(0.0), f64::NEG_INFINITY);
    }
}
