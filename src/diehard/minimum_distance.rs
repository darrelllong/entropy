//! DIEHARD minimum distance test in two dimensions.
//!
//! Places n = 8 000 uniform points in a square of side 10 000 and finds the
//! smallest distance d between two of them.  With r = d/side, each of the
//! C(n, 2) pairs is within distance d with probability
//! H_2(r) = πr² − (8/3)r³ + r⁴/2 (see `nearest_pair::cube_pair_probability`),
//! and the number of such pairs is approximately Poisson, so
//!
//! u = 1 − exp(−C(n, 2)·H_2(r))
//!
//! is approximately uniform.  100 repetitions give 100 values of u, and a
//! Kolmogorov–Smirnov test of them is the result.  The calibration of this
//! transform is in [`crate::dieharder::minimum_distance_nd`], which generalises
//! the test to 2 to 5 dimensions.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{
    diehard::nearest_pair::{cube_pair_probability, min_squared_distance},
    math::ks_test,
    result::TestResult,
    rng::Rng,
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
    let pairs = n_points as f64 * (n_points as f64 - 1.0) / 2.0;
    let mut p_values = Vec::with_capacity(repeats);

    for _ in 0..repeats {
        let points: Vec<[f64; 2]> = (0..n_points)
            .map(|_| [rng.next_f64() * SQUARE_SIDE, rng.next_f64() * SQUARE_SIDE])
            .collect();

        let r = (min_squared_distance(&points).sqrt() / SQUARE_SIDE).min(1.0);
        p_values.push(-(-pairs * cube_pair_probability(r, 2)).exp_m1());
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
    use super::minimum_distance_2d;
    use crate::rng::ConstantRng;

    /// Every point coincides, so d² = 0 in every repeat.
    #[test]
    fn constant_generator_fails() {
        let r = minimum_distance_2d(&mut ConstantRng::new(0), true);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
