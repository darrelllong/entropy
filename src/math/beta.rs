//! The regularised incomplete beta function and Student's t quantile.

use super::ln_gamma::{ln_gamma, stirling_remainder};

/// Why a numerical function returned no value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NumericalError {
    /// An argument lies outside the function's domain.
    Domain,
    /// The evaluation could not reach its stated accuracy: an expansion did
    /// not converge within its budget, or its result was not a valid value.
    NotConverged,
}

impl core::fmt::Display for NumericalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Domain => "an argument lies outside the function's domain",
            Self::NotConverged => "the evaluation did not converge to its stated accuracy",
        })
    }
}

impl std::error::Error for NumericalError {}

/// Student's `t` at `probability`, one-sided, on `freedom` degrees of
/// freedom: the point `t` with `P(T ≤ t) = probability`, for a bound on a
/// difference of means from few observations.
///
/// The distribution function is `P(|T| > t) = I_x(ν/2, 1/2)` with
/// `x = ν/(ν + t²)` and `I` the regularised incomplete beta function
/// (DLMF 8.17.1 and §8.18; Abramowitz & Stegun 26.7.1), inverted by
/// bisection on `t` to a part in a billion. A Cornish–Fisher expansion
/// about the normal quantile would be five per cent short at the far points
/// on few degrees of freedom, where its terms have not begun to shrink.
///
/// # Errors
///
/// [`NumericalError::Domain`] if `freedom` is zero or `probability` is not
/// strictly between one half and one; [`NumericalError::NotConverged`] if
/// an incomplete-beta evaluation fails, or the bracket grows past the
/// doubles.
pub fn student_t_quantile(freedom: usize, probability: f64) -> Result<f64, NumericalError> {
    if freedom == 0 || !(probability > 0.5 && probability < 1.0) {
        return Err(NumericalError::Domain);
    }
    let nu = freedom as f64;
    let tail = 2.0 * (1.0 - probability);
    let two_sided_tail = |t: f64| regularized_incomplete_beta(nu / (nu + t * t), nu / 2.0, 0.5);
    /// Relative width at which the bisection stops.
    const BISECTION_TOLERANCE: f64 = 1e-9;
    let (mut low, mut high) = (0.0f64, 1.0f64);
    while two_sided_tail(high)? > tail {
        high *= 2.0;
        if !high.is_finite() {
            return Err(NumericalError::NotConverged);
        }
    }
    while high - low > BISECTION_TOLERANCE * high {
        let middle = 0.5 * (low + high);
        if two_sided_tail(middle)? > tail {
            low = middle;
        } else {
            high = middle;
        }
    }
    Ok(0.5 * (low + high))
}

/// The regularised incomplete beta function `I_x(a, b)` for `a, b > 0` and
/// `x` in `[0, 1]`: the distribution function of the beta distribution,
/// and through it of Student's `t` and the `F` and binomial distributions.
///
/// `I_x(a, b) = x^a (1−x)^b / (a·B(a, b)) · C`, with `C` the continued
/// fraction DLMF 8.17.22 by the modified Lentz method (Lentz, Applied Optics
/// 15 (1976), 668–671; Thompson & Barnett, J. Comput. Phys. 64 (1986),
/// 490–509) in double-double arithmetic. For `x` below `(a+1)/(a+b+2)` it
/// computes `I_x(a, b)`, otherwise `1 − I_x(a, b) = I_{1−x}(b, a)` (DLMF
/// 8.17.4), with `1 − x` held exactly. The prefactor's logarithm is formed in
/// the regime of the shapes so that nothing underflows or cancels; see
/// `incomplete_beta_tail`.
///
/// Accuracy, against 50-digit values over shapes from `10⁻³⁰⁰` to `10⁴` in
/// every pairing (`scripts/incomplete_beta_reference.py`) and by closed forms
/// and symmetry to `10¹⁵`: absolute error below `10⁻¹⁴`; and where the tail
/// the fraction computes is a normal double, relative error within
/// `10⁻¹⁴·(1 + |ln tail|)` of that tail.
///
/// Near the mean the fraction takes about `0.45·max(a, b)^0.32` steps (526
/// at `10⁶`, 192 166 at `10¹⁴`); it is allowed `64 + 64·⌈√max(a, b)⌉`, at
/// most ten million.
///
/// # Errors
///
/// [`NumericalError::Domain`] if `a` or `b` is not a positive finite number
/// or `x` lies outside `[0, 1]`; [`NumericalError::NotConverged`] if the
/// fraction does not settle within its budget or the result lies outside
/// `[0, 1]` by more than `10⁻¹⁴` (a result that close is rounded into the
/// interval).
pub fn regularized_incomplete_beta(x: f64, a: f64, b: f64) -> Result<f64, NumericalError> {
    let positive = |v: f64| v.is_finite() && v > 0.0;
    if !(positive(a) && positive(b) && (0.0..=1.0).contains(&x)) {
        return Err(NumericalError::Domain);
    }
    if x == 0.0 {
        return Ok(0.0);
    }
    if x == 1.0 {
        return Ok(1.0);
    }
    let value = if x < (a + 1.0) / (a + b + 2.0) {
        incomplete_beta_tail(DoubleDouble::from(x), a, b)?
    } else {
        // 1 − x exactly, so no input bit is lost on the complement side.
        let complement = DoubleDouble::from(1.0).add(DoubleDouble::from(-x));
        1.0 - incomplete_beta_tail(complement, b, a)?
    };
    // A probability within the documented absolute error of 0 or 1 can land
    // past it; that is rounding, not failure. Anything farther out is.
    const SLACK: f64 = 1e-14;
    if (-SLACK..=1.0 + SLACK).contains(&value) {
        Ok(value.clamp(0.0, 1.0))
    } else {
        Err(NumericalError::NotConverged)
    }
}

/// `I_p(a, b)` by prefactor and continued fraction, for `p` (held exactly) on
/// the side where the fraction converges: `I_p(a, b) = P·C` with
/// `P = p^a q^b / (a·B(a, b))`, `q = 1 − p` and `C` the fraction. `ln P` is
/// formed in the regime of the shapes, so that no intermediate underflows
/// and no large terms cancel:
///
/// - both shapes at least one: Stirling's form of `ln B` (DLMF 5.11.1).
///   With `p₀ = a/(a+b)`, `r = (p − p₀)/p₀` and `s` likewise for `q`, the
///   linear terms `a·r + b·s` sum to zero, leaving
///   `ln P = a·(ln(1+r) − r) + b·(ln(1+s) − s) + ½ ln(b/(a(a+b))) − ½ ln 2π
///   − δ(a) − δ(b) + δ(a+b)`, with `δ` Stirling's remainder;
/// - one shape below one: `ln Γ(1 + t)` for the small shape `t`, whose
///   Stirling remainder would be large, and Stirling's form for the large
///   shape `u`: `ln Γ(u) − ln Γ(t+u) = −t ln(t+u) − (u − ½) ln(1 + t/u) + t
///   + δ(u) − δ(t+u)`;
/// - both below one: `ln B = ln Γ(1+a) + ln Γ(1+b) − ln Γ(1+a+b) + ln((a+b)/(ab))`.
///
/// The deviation from the mean is formed in double-doubles from the exact
/// mean.
fn incomplete_beta_tail(p: DoubleDouble, a: f64, b: f64) -> Result<f64, NumericalError> {
    let dd = DoubleDouble::from;
    let negate = |v: DoubleDouble| DoubleDouble {
        hi: -v.hi,
        lo: -v.lo,
    };
    let total = dd(a).add(dd(b));
    let q = dd(1.0).add(negate(p));
    let ln_p = log_of_complement_pair(p, q);
    let ln_q = log_of_complement_pair(q, p);
    let ln_prefactor = if a >= 1.0 && b >= 1.0 {
        let (p0, q0) = (dd(a).div(total), dd(b).div(total));
        let deviation = p.add(negate(p0));
        let r = deviation.div(p0);
        let s = negate(deviation).div(q0);
        a * log_one_plus_minus(r, p.div(p0))
            + b * log_one_plus_minus(s, q.div(q0))
            + 0.5 * (b.ln() - a.ln() - (a + b).ln())
            - 0.5 * (2.0 * core::f64::consts::PI).ln()
            - stirling_remainder(a)
            - stirling_remainder(b)
            + stirling_remainder(a + b)
    } else if a < 1.0 && b < 1.0 {
        // ln P = a ln p + b ln q − ln a − ln B, and −ln a − ln B =
        // ln(b/(a+b)) − ln Γ(1+a) − ln Γ(1+b) + ln Γ(1+a+b).
        a * ln_p + b * ln_q - (a / b).ln_1p() - ln_gamma(1.0 + a) - ln_gamma(1.0 + b)
            + ln_gamma(1.0 + a + b)
    } else {
        // The small shape t pairs with its variable; −ln a − ln B collects
        // −ln Γ(1+t) + ln t − ln a + t ln(t+u) + (u − ½) ln(1 + t/u) − t
        // − δ(u) + δ(t+u).
        let (t, u) = if a < b { (a, b) } else { (b, a) };
        // ln t − ln a is 0 or ln(b/a); it is formed before joining the sum,
        // whose other terms can be far smaller than either logarithm.
        let ln_ratio = t.ln() - a.ln();
        a * ln_p + b * ln_q + ln_ratio + t * (t + u).ln() + (u - 0.5) * (t / u).ln_1p()
            - t
            - ln_gamma(1.0 + t)
            - stirling_remainder(u)
            + stirling_remainder(t + u)
    };
    let fraction = beta_continued_fraction(p, a, b)?;
    let value = ln_prefactor.exp() * fraction;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(NumericalError::NotConverged)
    }
}

/// `ln v` for `v` in `(0, 1)` given `v` and `1 − v` both exactly: `ln_1p` of
/// `−(1 − v)` when `v` is near one, where the direct logarithm would lose
/// the complement's digits.
fn log_of_complement_pair(v: DoubleDouble, complement: DoubleDouble) -> f64 {
    /// Below this complement, v is near one and `ln_1p` keeps its digits.
    const NEAR_ONE: f64 = 0.5;
    let c = complement.hi + complement.lo;
    if c < NEAR_ONE {
        (-c).ln_1p()
    } else {
        (v.hi + v.lo).ln()
    }
}

/// `ln(1 + r) − r`, given `r` and `1 + r` both to double-double accuracy: the
/// series `−r²/2 + r³/3 − …` for `|r| < 1/10`, where it needs at most sixteen
/// terms and `ln(1 + r)` and `r` would cancel, and `ln(1 + r) − r` from the
/// ratio itself otherwise.
fn log_one_plus_minus(r: DoubleDouble, one_plus_r: DoubleDouble) -> f64 {
    /// |r| below which the series is summed.
    const SERIES_BELOW: f64 = 0.1;
    /// The series stops once a term is this fraction of ε times the sum…
    const TAIL_EPSILONS: f64 = 0.25;
    /// …or past this power, which |r| < 1/10 never needs.
    const LAST_POWER: f64 = 40.0;
    let r_value = r.hi + r.lo;
    if r_value.abs() < SERIES_BELOW {
        let mut power = r_value * r_value;
        let mut sum = 0.0;
        let mut n = 2.0;
        loop {
            let term = power / n;
            let signed = if (n as u64).is_multiple_of(2) {
                -term
            } else {
                term
            };
            sum += signed;
            if term.abs() <= f64::EPSILON * sum.abs() * TAIL_EPSILONS || n > LAST_POWER {
                return sum;
            }
            power *= r_value;
            n += 1.0;
        }
    }
    let ratio = one_plus_r.hi + one_plus_r.lo;
    ratio.ln() - r_value
}

/// A double-double number `hi + lo` with `|lo| ≤ ulp(hi)/2`: about 106
/// bits, with error-free sums and products (Dekker, *A floating-point
/// technique for extending the available precision*, Numer. Math. 18 (1971),
/// 224–242; the product by fused multiply-add).
#[derive(Clone, Copy, Debug)]
struct DoubleDouble {
    hi: f64,
    lo: f64,
}

impl DoubleDouble {
    fn from(value: f64) -> Self {
        Self { hi: value, lo: 0.0 }
    }

    fn normalized(hi: f64, lo: f64) -> Self {
        let sum = hi + lo;
        Self {
            hi: sum,
            lo: lo - (sum - hi),
        }
    }

    fn add(self, other: Self) -> Self {
        let sum = self.hi + other.hi;
        let virtual_b = sum - self.hi;
        let error = (self.hi - (sum - virtual_b)) + (other.hi - virtual_b);
        Self::normalized(sum, error + self.lo + other.lo)
    }

    fn mul(self, other: Self) -> Self {
        let product = self.hi * other.hi;
        let error = self.hi.mul_add(other.hi, -product);
        Self::normalized(product, error + (self.hi * other.lo + self.lo * other.hi))
    }

    fn div(self, other: Self) -> Self {
        let quotient = self.hi / other.hi;
        let remainder = self.add(Self::from(-quotient).mul(other));
        Self::normalized(quotient, remainder.hi / other.hi)
    }
}

/// The continued fraction DLMF 8.17.22 by the modified Lentz method in
/// double-double arithmetic:
/// `I_x(a, b) = x^a (1−x)^b / (a·B(a, b)) · 1/(1 + d₁/(1 + d₂/(1 + …)))`.
///
/// Near the mean the fraction takes on the order of `√max(a, b)` steps, and in
/// doubles their rounding and the last-place stopping test left an error that
/// grew as `√max(a, b)·ε` (`10⁻¹⁰` at `a = b = 10¹²`). In double-doubles a
/// step is accepted as converged within `10⁻³⁰`. [`NumericalError::NotConverged`]
/// if that does not happen within [`continued_fraction_budget`] steps.
fn beta_continued_fraction(x: DoubleDouble, a: f64, b: f64) -> Result<f64, NumericalError> {
    beta_continued_fraction_within(x, a, b, continued_fraction_budget(a, b))
}

/// Steps allowed the continued fraction: `64 + 64·⌈√max(a, b)⌉`, and never
/// more than ten million (about a second), past which shapes near `10¹⁹`
/// are refused rather than computed slowly.
fn continued_fraction_budget(a: f64, b: f64) -> f64 {
    /// Steps allowed whatever the shapes.
    const BASE: f64 = 64.0;
    /// Further steps per unit of ⌈√max(a, b)⌉.
    const PER_ROOT: f64 = 64.0;
    /// The cap.
    const MOST: f64 = 1e7;
    (BASE + PER_ROOT * a.max(b).sqrt().ceil()).min(MOST)
}

fn beta_continued_fraction_within(
    x: DoubleDouble,
    a: f64,
    b: f64,
    budget: f64,
) -> Result<f64, NumericalError> {
    /// Lentz's floor for a vanishing denominator.
    const TINY: f64 = 1e-300;
    /// A step's factor within this of one is converged.
    const CONVERGED: f64 = 1e-30;
    let dd = DoubleDouble::from;
    let one = dd(1.0);
    let floored = |value: DoubleDouble| {
        if value.hi.abs() < TINY {
            dd(TINY)
        } else {
            value
        }
    };
    // Every coefficient is formed in double-doubles: with a shape such as
    // 10⁻³, `b − m` and `a + 2m` are inexact in doubles, and that rounding
    // compounds over the steps as the product's did.
    let (a_dd, b_dd) = (dd(a), dd(b));
    let negate = |v: DoubleDouble| DoubleDouble {
        hi: -v.hi,
        lo: -v.lo,
    };
    let qab = a_dd.add(b_dd);
    let qap = a_dd.add(one);
    let qam = a_dd.add(dd(-1.0));
    let mut c = one;
    let mut d = one.div(floored(one.add(negate(qab).mul(x).div(qap))));
    let mut h = d;
    let mut m = 1.0;
    while m <= budget {
        let (m_dd, m2) = (dd(m), dd(2.0 * m));
        let even = m_dd
            .mul(b_dd.add(negate(m_dd)))
            .mul(x)
            .div(qam.add(m2).mul(a_dd.add(m2)));
        d = one.div(floored(one.add(even.mul(d))));
        c = floored(one.add(even.div(c)));
        h = h.mul(d).mul(c);
        let odd = negate(a_dd.add(m_dd))
            .mul(qab.add(m_dd))
            .mul(x)
            .div(a_dd.add(m2).mul(qap.add(m2)));
        d = one.div(floored(one.add(odd.mul(d))));
        c = floored(one.add(odd.div(c)));
        let step = d.mul(c);
        h = h.mul(step);
        let change = step.add(dd(-1.0));
        if (change.hi + change.lo).abs() <= CONVERGED {
            let value = h.hi + h.lo;
            return if value.is_finite() {
                Ok(value)
            } else {
                Err(NumericalError::NotConverged)
            };
        }
        m += 1.0;
    }
    Err(NumericalError::NotConverged)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Relative error allowed against three-digit printed tables.
    const PRINTED_TABLE: f64 = 5e-4;
    /// Relative error allowed against 50-digit quantiles: twice the bisection's.
    const QUANTILE: f64 = 2e-9;
    /// The documented absolute error of the incomplete beta.
    const ABSOLUTE: f64 = 1e-14;
    /// The documented relative error per unit of 1 + |ln tail|.
    const PER_LOG: f64 = 1e-14;
    /// Error allowed in I_{1/2}(a, a) = 1/2 and in the symmetry's sum.
    const HALF: f64 = 2e-15;
    const SYMMETRY: f64 = 4e-15;
    /// Relative error allowed against the expm1/ln_1p closed forms, on tails
    /// above `CLOSED_FORM_TAIL_FLOOR`, relative to at least `CLOSED_FORM_SCALE`.
    const CLOSED_FORM: f64 = 1e-12;
    const CLOSED_FORM_TAIL_FLOOR: f64 = 1e-300;
    const CLOSED_FORM_SCALE: f64 = 1e-3;
    /// Rows the reference files must hold.
    const T_ROWS: usize = 1_000;
    const BETA_ROWS: usize = 1_500;

    /// Double-double `1 − x`, as the function forms it.
    fn exact_complement_is_representable(x: f64) -> bool {
        1.0 - (1.0 - x) == x
    }

    /// The quantile is the table's, at the two-sided five, one and a tenth
    /// of a per cent points, from one degree of freedom up.
    #[test]
    fn student_t_is_the_table() {
        let table = [
            (1, [12.706, 63.657, 636.619]),
            (2, [4.303, 9.925, 31.599]),
            (3, [3.182, 5.841, 12.924]),
            (5, [2.571, 4.032, 6.869]),
            (7, [2.365, 3.499, 5.408]),
            (10, [2.228, 3.169, 4.587]),
            (20, [2.086, 2.845, 3.850]),
            (30, [2.042, 2.750, 3.646]),
            (120, [1.980, 2.617, 3.373]),
        ];
        for (freedom, points) in table {
            for (probability, expected) in [0.975, 0.995, 0.9995].into_iter().zip(points) {
                let t = student_t_quantile(freedom, probability).expect("in the domain");
                assert!(
                    (t / expected - 1.0).abs() < PRINTED_TABLE,
                    "{freedom} at {probability}: {t} against {expected}"
                );
            }
        }
    }

    /// Against the 50-digit quantiles of `scripts/incomplete_beta_reference.py`
    /// in the regime of factoring's polynomial race (its degrees of freedom and
    /// Bonferroni-divided probabilities) and at the classical table points: the
    /// bisection stops within a part in a billion.
    #[test]
    fn student_t_matches_high_precision_quantiles() {
        let mut checked = 0;
        for line in include_str!("../../tests/data/student_t.txt").lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let freedom: usize = fields[0].parse().expect("freedom");
            let probability: f64 = fields[1].parse().expect("probability");
            let expected: f64 = fields[2].parse().expect("quantile");
            let t = student_t_quantile(freedom, probability).expect("in the domain");
            assert!(
                (t / expected - 1.0).abs() < QUANTILE,
                "{freedom} at {probability}: {t} against {expected}"
            );
            checked += 1;
        }
        assert!(checked > T_ROWS, "only {checked} quantiles");
    }

    #[test]
    fn student_t_refuses_its_domain_edges() {
        assert_eq!(student_t_quantile(0, 0.9), Err(NumericalError::Domain));
        for probability in [0.5, 1.0, 0.2, f64::NAN] {
            assert_eq!(
                student_t_quantile(3, probability),
                Err(NumericalError::Domain)
            );
        }
    }

    /// Against 50-digit values from `scripts/incomplete_beta_reference.py`
    /// over shapes from `10⁻³⁰⁰` to `10⁴` in every pairing, at the mean, from
    /// 1 to 40 standard deviations either side and either side of the switch
    /// between the two sides of the symmetry: absolute error below `10⁻¹⁴`,
    /// and on the tail the fraction computes, when that tail is a normal
    /// double, relative error within `10⁻¹⁴·(1 + |ln tail|)`.
    #[test]
    fn the_incomplete_beta_function_matches_high_precision_values() {
        let mut checked = 0;
        for line in include_str!("../../tests/data/incomplete_beta.txt").lines() {
            let v: Vec<f64> = line
                .split_whitespace()
                .map(|field| field.parse().expect("number"))
                .collect();
            let (x, a, b, expected) = (v[0], v[1], v[2], v[3]);
            let got = regularized_incomplete_beta(x, a, b).expect("converges");
            let error = (got - expected).abs();
            let what = format!("I_{x:e}({a:e}, {b:e}) = {got:e}, expected {expected:e}");
            assert!(error < ABSOLUTE, "{what}: absolute error {error:e}");
            let direct = x < (a + 1.0) / (a + b + 2.0);
            let tail = if direct { expected } else { 1.0 - expected };
            if (f64::MIN_POSITIVE..=0.5).contains(&tail) {
                let bound = PER_LOG * tail * (1.0 + tail.ln().abs());
                assert!(error <= bound, "{what}: relative error {:e}", error / tail);
            }
            checked += 1;
        }
        assert!(checked > BETA_ROWS, "only {checked} references");
    }

    /// Beyond the references' reach, the closed forms that fix the answer:
    /// `I_{1/2}(a, a) = 1/2` to `a = 10¹⁴`, `I_x(1, b) = 1 − (1 − x)^b` and
    /// `I_x(a, 1) = x^a` by `expm1`/`ln_1p` to shapes of `10¹²`, and the
    /// symmetry `I_x(a, b) + I_{1−x}(b, a) = 1` — the two sides evaluated
    /// separately — for shapes to `10¹⁵`, at complements exact in doubles.
    #[test]
    fn the_incomplete_beta_function_holds_its_identities_at_large_shapes() {
        let least_normal = f64::MIN_POSITIVE;
        for a in [
            5e-324,
            least_normal.next_down(),
            least_normal,
            least_normal.next_up(),
            1e-300,
            1e-200,
            1e-160,
            1e-100,
            1e-20,
            0.5,
            12.0,
            1e4,
            1e6,
            1e8,
            1e10,
            1e12,
            1e14,
        ] {
            let value = regularized_incomplete_beta(0.5, a, a).expect("converges");
            assert!((value - 0.5).abs() < HALF, "I_1/2({a:e}, {a:e}) = {value}");
        }
        for b in [1e-300, 1e-100, 1e-20, 1e-3, 0.5, 2.0, 1e3, 1e6, 1e9, 1e12] {
            for x in [1e-15, 1e-9, 1e-3, 0.1, 0.5, 0.9, 1.0 - 1e-9] {
                let one_minus_power = -(b * f64::ln_1p(-x)).exp_m1();
                let power = x.powf(b);
                for (got, expected) in [
                    (regularized_incomplete_beta(x, 1.0, b), one_minus_power),
                    (regularized_incomplete_beta(x, b, 1.0), power),
                ] {
                    let got = got.expect("converges");
                    let tail = expected.min(1.0 - expected);
                    let error = (got - expected).abs();
                    assert!(
                        error < ABSOLUTE,
                        "x {x:e}, shape {b:e}: {got:e} against {expected:e}"
                    );
                    if tail > CLOSED_FORM_TAIL_FLOOR {
                        assert!(
                            error < CLOSED_FORM * tail.max(CLOSED_FORM_SCALE),
                            "x {x:e}, shape {b:e}: tail"
                        );
                    }
                }
            }
        }
        let unit = 2f64.powi(-53);
        for a in [1e8f64, 1e10, 1e12] {
            for ratio in [1.0, 3.0, 1e3] {
                let b = a * ratio;
                let mean = a / (a + b);
                let sd = (a * b / ((a + b) * (a + b) * (a + b + 1.0))).sqrt();
                for k in [-8.0, -2.0, -0.5, 0.0, 0.5, 2.0, 8.0] {
                    let x = ((mean + k * sd) / unit).round() * unit;
                    assert!(exact_complement_is_representable(x));
                    let p = regularized_incomplete_beta(x, a, b).expect("converges");
                    let q = regularized_incomplete_beta(1.0 - x, b, a).expect("converges");
                    assert!(
                        (p + q - 1.0).abs() < SYMMETRY,
                        "a {a:e} b {b:e} k {k}: {p} + {q}"
                    );
                }
            }
        }
    }

    /// The incomplete beta refuses what is outside its domain, returns the
    /// ends exactly, and reports a continued fraction that has not settled.
    #[test]
    fn the_incomplete_beta_function_reports_failure() {
        for (x, a, b) in [
            (0.5, 0.0, 1.0),
            (0.5, 1.0, -1.0),
            (-0.1, 1.0, 1.0),
            (1.1, 1.0, 1.0),
            (f64::NAN, 1.0, 1.0),
            (0.5, f64::INFINITY, 1.0),
            (0.5, f64::NAN, 1.0),
        ] {
            assert_eq!(
                regularized_incomplete_beta(x, a, b),
                Err(NumericalError::Domain),
                "{x} {a} {b}"
            );
        }
        assert_eq!(regularized_incomplete_beta(0.0, 2.0, 3.0), Ok(0.0));
        assert_eq!(regularized_incomplete_beta(1.0, 2.0, 3.0), Ok(1.0));
        // Near the mean at a = b = 10⁶ the fraction needs hundreds of steps.
        let half = DoubleDouble::from(0.5);
        assert_eq!(
            beta_continued_fraction_within(half, 1e6, 1e6, 3.0),
            Err(NumericalError::NotConverged)
        );
        assert!(beta_continued_fraction_within(
            half,
            1e6,
            1e6,
            continued_fraction_budget(1e6, 1e6)
        )
        .is_ok());
    }
}
