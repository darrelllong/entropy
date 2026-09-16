//! DIEHARD minimum distance test in two dimensions.
//!
//! Places n = 8 000 uniform points in a square of side 10 000 and finds the
//! smallest distance d between two of them.  Each of the C(n, 2) pairs is
//! within distance d with probability close to πd²/A, A the area, and the
//! number of such pairs is approximately Poisson, so
//!
//! P(minimum > d) ≈ exp(−C(n, 2)·πd²/A),
//!
//! and u = 1 − exp(−C(n, 2)·πd²/A) is approximately uniform.  The mean of d²
//! is then A/(π·C(n, 2)) ≈ 0.995.  100 repetitions give 100 values of u, and a
//! Kolmogorov–Smirnov test of them is the result.  The boundary of the square
//! changes the pair probability by a relative O(d/side) ≈ 10⁻⁴, below the
//! resolution of 100 repetitions.
//!
//! [`crate::dieharder::minimum_distance_nd`] generalises the test to 2 to 5
//! dimensions.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{
    diehard::nearest_pair::min_squared_distance, math::ks_test, result::TestResult, rng::Rng,
};

const SQUARE_SIDE: f64 = 10_000.0;

/// Run the 2D minimum distance test.
///
/// `quick`: use 500 points and 20 repeats instead of 8 000 × 100 to avoid the
/// O(n²) cost during development.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn minimum_distance_2d(rng: &mut impl Rng, quick: bool) -> TestResult {
    let n_points = if quick { 500 } else { 8_000 };
    let repeats = if quick { 20 } else { 100 };
    // Mean of d² under the Poisson approximation: A / (π·C(n, 2)).
    let pairs = n_points as f64 * (n_points as f64 - 1.0) / 2.0;
    let lambda = SQUARE_SIDE * SQUARE_SIDE / (std::f64::consts::PI * pairs);
    let mut p_values = Vec::with_capacity(repeats);

    for _ in 0..repeats {
        let points: Vec<[f64; 2]> = (0..n_points)
            .map(|_| [rng.next_f64() * SQUARE_SIDE, rng.next_f64() * SQUARE_SIDE])
            .collect();

        let d_sq = min_squared_distance(&points);
        let u = 1.0 - (-d_sq / lambda).exp();
        p_values.push(u.clamp(1e-15, 1.0 - 1e-15));
    }

    let p_value = ks_test(&mut p_values);

    TestResult::with_note(
        "diehard::minimum_distance_2d",
        p_value,
        format!("n={n_points}, side={SQUARE_SIDE}, repeats={repeats}"),
    )
}

#[cfg(test)]
mod tests {
    use super::{minimum_distance_2d, SQUARE_SIDE};
    use crate::rng::ConstantRng;

    /// At 8 000 points the Poisson mean of d² is A/(π·C(n, 2)) ≈ 0.995.
    #[test]
    fn mean_squared_distance_at_full_size() {
        let pairs = 8_000.0 * 7_999.0 / 2.0;
        let mean = SQUARE_SIDE * SQUARE_SIDE / (std::f64::consts::PI * pairs);
        assert!((mean - 0.995).abs() < 5e-4, "{mean}");
    }

    /// Every point coincides, so d² = 0 in every repeat.
    #[test]
    fn constant_generator_fails() {
        let r = minimum_distance_2d(&mut ConstantRng::new(0), true);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
