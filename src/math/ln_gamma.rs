//! ln Γ, and the remainder of Stirling's series that the large-shape
//! incomplete gamma and beta functions share.

/// The largest double whose `ln Γ` rounds to a finite double.
const LN_GAMMA_FINITE_BELOW: f64 = 2.559_983_327_851_638_3e305;

/// `ln Γ(x)` for `x > 0`, by Lanczos's approximation (Lanczos, *A
/// precision approximation of the gamma function*, J. SIAM Numer. Anal.
/// B 1 (1964), 86–96) with `g = 7` and nine terms.
///
/// The coefficients make the approximation exact at `Γ(1), …, Γ(9)`, each
/// rounded to the nearest double; `scripts/lanczos_coefficients.py`
/// derives them, confirms the table bit for bit, and separates the error
/// of the approximation from that of the rounded coefficients. Below
/// `x = 1/2` the recurrence `ln Γ(x) = ln Γ(1 + x) − ln x` (DLMF 5.5.1)
/// carries the argument into the approximation's range; it has no
/// intermediate that overflows, so the result is finite down to the least
/// subnormal.
///
/// From `x = 10⁷` the Stirling series `ln Γ(x) = x(ln x − 1) − ½ ln x +
/// ½ ln 2π + 1/(12x) + R` is used (DLMF 5.11.1), whose remainder is bounded
/// by the first omitted term, `1/(360x³)` (DLMF §5.11(ii)), below `10⁻²³`
/// here. It is evaluated at half scale and doubled, so no intermediate
/// overflows. `ln Γ` increases past 2, so the answer is finite exactly for
/// `x` up to the largest double whose `ln Γ` rounds to a finite value,
/// `0x1.754d9278b51a7p+1014` (about `2.56·10³⁰⁵`, found and checked by
/// `scripts/lanczos_coefficients.py`); there the true value is within one
/// unit in the last place of `f64::MAX`, so a result that rounds past it is
/// returned as `f64::MAX`, and above it the result is `+∞`.
///
/// Measured over 54 000 arguments against 40-digit values (the ignored
/// test `the_log_gamma_sweep_matches_high_precision`): absolute error below
/// `5·10⁻¹⁵` for `0.1 ≤ x ≤ 3`, the interval holding both zeros of `ln Γ`
/// and the switch at `1/2`, and relative error below `2·10⁻¹⁵` elsewhere.
/// The approximation itself, with the rounded coefficients, is good to
/// `10⁻¹⁵`; the rest is cancellation in floating evaluation. `NaN` for
/// `x ≤ 0` or `NaN`; `+∞` for `+∞`.
///
/// The normalisation of the gamma, chi-squared, beta and Student's `t`
/// densities; [`regularized_incomplete_beta`] is built on it.
#[must_use]
pub fn ln_gamma(x: f64) -> f64 {
    /// Where the Stirling series takes over from Lanczos's approximation.
    const STIRLING_FROM: f64 = 1e7;
    /// Below this the recurrence carries x up into the approximation's range.
    const RECURRENCE_BELOW: f64 = 0.5;
    /// Lanczos's shift g.
    const G: f64 = 7.0;
    const COEFFICIENTS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x.is_nan() || x <= 0.0 {
        return f64::NAN;
    }
    if x.is_infinite() {
        return f64::INFINITY;
    }
    if x < RECURRENCE_BELOW {
        return ln_gamma(1.0 + x) - x.ln();
    }
    if x >= STIRLING_FROM {
        if x > LN_GAMMA_FINITE_BELOW {
            return f64::INFINITY;
        }
        let ln_x = x.ln();
        let half = 0.5 * x * (ln_x - 1.0) - 0.25 * ln_x
            + 0.25 * (2.0 * core::f64::consts::PI).ln()
            + 1.0 / (24.0 * x);
        let value = 2.0 * half;
        return if value.is_finite() { value } else { f64::MAX };
    }
    let z = x - 1.0;
    let t = z + G + 0.5;
    let mut series = COEFFICIENTS[0];
    for (k, &c) in COEFFICIENTS.iter().enumerate().skip(1) {
        series += c / (z + k as f64);
    }
    0.5 * (2.0 * core::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + series.ln()
}

/// `δ(t) = ln Γ(t) − (t − ½) ln t + t − ½ ln 2π`, the remainder of Stirling's
/// approximation: its asymptotic series (DLMF 5.11.1) from `t = 10`, where the
/// first omitted term is below `10⁻¹⁷`, and [`ln_gamma`] below.
pub(crate) fn stirling_remainder(t: f64) -> f64 {
    /// Where the asymptotic series takes over from ln Γ.
    const SERIES_FROM: f64 = 10.0;
    if t < SERIES_FROM {
        return ln_gamma(t) - (t - 0.5) * t.ln() + t - 0.5 * (2.0 * core::f64::consts::PI).ln();
    }
    // B₂ₖ / (2k(2k − 1)) for k = 1..8.
    const COEFFICIENTS: [f64; 8] = [
        1.0 / 12.0,
        -1.0 / 360.0,
        1.0 / 1260.0,
        -1.0 / 1680.0,
        1.0 / 1188.0,
        -691.0 / 360_360.0,
        1.0 / 156.0,
        -3617.0 / 122_400.0,
    ];
    let inverse_square = 1.0 / (t * t);
    let mut sum = 0.0;
    for &c in COEFFICIENTS.iter().rev() {
        sum = sum * inverse_square + c;
    }
    sum / t
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Where ln Γ passes through its zeros and the switch at ½, errors are
    /// measured absolutely; elsewhere relatively.
    const ABSOLUTE_RANGE: std::ops::RangeInclusive<f64> = 0.1..=3.0;
    /// Absolute error allowed in `ABSOLUTE_RANGE`.
    const ABSOLUTE: f64 = 5e-15;
    /// Relative error allowed elsewhere.
    const RELATIVE: f64 = 2e-15;
    /// Error allowed against closed forms computed in doubles.
    const CLOSED_FORM: f64 = 1e-12;
    /// Reference pairs the sweep file must hold.
    const SWEEP_PAIRS: usize = 54_000;

    /// Against 50-digit values from `scripts/lanczos_coefficients.py
    /// --reference`: subnormal and tiny arguments through the recurrence,
    /// both sides of the switch at 1/2, the zeros at 1 and 2 and their
    /// neighbours, the minimum near 1.4616, and large arguments up to
    /// where the value approaches `f64::MAX`.
    #[test]
    fn the_log_gamma_is_accurate_from_the_least_subnormal_up() {
        let reference: [(f64, f64); 43] = [
            (5e-324, 744.4400719213812),
            (1e-310, 713.8013788281542),
            (2.2250738585072014e-308, 708.3964185322641),
            (1e-300, 690.7755278982137),
            (1e-100, 230.25850929940458),
            (1e-20, 46.051701859880914),
            (1e-10, 23.025850929882736),
            (1e-05, 11.512919692895826),
            (0.001, 6.907178885383853),
            (0.1, 2.252712651734206),
            (0.25, 1.2880225246980774),
            (0.4999999999999999, 0.5723649429247003),
            (0.5, 0.5723649429247001),
            (0.5000000000000001, 0.5723649429246999),
            (0.75, 0.20328095143129538),
            (0.9999999999999998, 1.2816762426960017e-16),
            (1.0, 0.0),
            (1.0000000000000002, -1.2816762426960008e-16),
            (1.00000001, -5.772156531688512e-09),
            (1.4616321449683622, -0.12148629053584961),
            (1.5, -0.12078223763524522),
            (1.999999, -4.227840125965854e-07),
            (1.9999999999999996, -1.8775396131086232e-16),
            (2.0, 0.0),
            (2.0000000000000004, 1.8775396131086244e-16),
            (2.000001, 4.2278465762452923e-07),
            (2.5, 0.2846828704729192),
            (3.0, core::f64::consts::LN_2),
            (7.5, 7.534364236758733),
            (10.0, 12.801827480081469),
            (33.3, 82.60372358165495),
            (100.0, 359.1342053695754),
            (171.5, 709.1431630309282),
            (1000.0, 5905.220423209181),
            (100000.0, 1051287.7089736569),
            (10000000000.0, 220258509288.81058),
            (1000000000000000.0, 3.3538776394910668e+16),
            (1e+100, 2.2925850929940456e+102),
            (1e+300, 6.897755278982137e+302),
            (2.5e+305, 1.7555118602376452e+308),
            (2.557e+305, 1.7955951755681237e+308),
            (2.558e+305, 1.7962984030516992e+308),
            (2.559e+305, 1.7970016309262054e+308),
        ];
        for (x, expected) in reference {
            let got = ln_gamma(x);
            let error = (got - expected).abs();
            if ABSOLUTE_RANGE.contains(&x) {
                assert!(
                    error < ABSOLUTE,
                    "ln Γ({x:e}) = {got:e}, expected {expected:e}, absolute error {error:e}"
                );
            } else {
                let relative = error / expected.abs();
                assert!(
                    relative < RELATIVE,
                    "ln Γ({x:e}) = {got:e}, expected {expected:e}, relative error {relative:e}"
                );
            }
        }
        // The finite limit exactly: the largest double whose ln Γ rounds to a
        // finite double, and its neighbours.
        let limit = LN_GAMMA_FINITE_BELOW;
        assert_eq!(ln_gamma(limit), f64::MAX, "ln Γ at the finite limit");
        assert!(
            ln_gamma(limit.next_down()).is_finite(),
            "below the finite limit"
        );
        assert_eq!(
            ln_gamma(limit.next_up()),
            f64::INFINITY,
            "past the finite limit"
        );
        assert_eq!(ln_gamma(f64::MAX), f64::INFINITY);
        assert!(ln_gamma(0.0).is_nan());
        assert!(ln_gamma(-1.5).is_nan());
        assert!(ln_gamma(f64::NAN).is_nan());
        assert_eq!(ln_gamma(f64::INFINITY), f64::INFINITY);
    }

    /// The dense sweep: `scripts/lanczos_coefficients.py --sweep FILE`
    /// writes 54 000 pairs `x ln Γ(x)` (seed 20260916, 40 digits) covering
    /// the least subnormal to the finite limit near `2.56·10³⁰⁵`, with dense
    /// bands around the switches at 1/2 and `10⁷` and the zeros at 1 and 2;
    /// run with
    /// `ENTROPY_LN_GAMMA_SWEEP=FILE cargo test --release -- --ignored
    /// the_log_gamma_sweep`.
    #[test]
    #[ignore = "needs the reference file from scripts/lanczos_coefficients.py --sweep"]
    fn the_log_gamma_sweep_matches_high_precision() {
        let path = std::env::var("ENTROPY_LN_GAMMA_SWEEP")
            .expect("ENTROPY_LN_GAMMA_SWEEP names the reference file");
        let text = std::fs::read_to_string(path).expect("the reference file is readable");
        let mut checked = 0usize;
        for line in text.lines() {
            let (x, expected) = line.split_once(' ').expect("each line is `x value`");
            let (x, expected): (f64, f64) =
                (x.parse().expect("x"), expected.parse().expect("value"));
            let error = (ln_gamma(x) - expected).abs();
            if ABSOLUTE_RANGE.contains(&x) {
                assert!(error < ABSOLUTE, "ln Γ({x:e}): absolute error {error:e}");
            } else {
                assert!(
                    error < RELATIVE * expected.abs(),
                    "ln Γ({x:e}): relative error {:e}",
                    error / expected.abs()
                );
            }
            checked += 1;
        }
        assert!(checked > SWEEP_PAIRS, "only {checked} reference pairs");
    }

    #[test]
    fn the_log_gamma_matches_factorials_and_the_half_integers() {
        for n in 1..=20u32 {
            let factorial: f64 = (1..n).map(f64::from).product();
            let expected = factorial.ln();
            assert!(
                (ln_gamma(f64::from(n)) - expected).abs() < CLOSED_FORM * expected.abs().max(1.0),
                "Γ({n})"
            );
        }
        // Γ(1/2) = √π, Γ(3/2) = √π/2.
        let root_pi = core::f64::consts::PI.sqrt();
        assert!((ln_gamma(0.5) - root_pi.ln()).abs() < CLOSED_FORM);
        assert!((ln_gamma(1.5) - (root_pi / 2.0).ln()).abs() < CLOSED_FORM);
        // Γ(1/4) to 16 digits.
        assert!((ln_gamma(0.25) - 3.625_609_908_221_908f64.ln()).abs() < CLOSED_FORM);
    }
}
