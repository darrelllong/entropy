//! DIEHARD 3-D spheres test.
//!
//! Places n = 4 000 uniform points in a cube of side 1 000 and finds the
//! smallest distance d between two of them.  With r = d/side, each of the
//! C(n, 2) pairs is within distance d with probability H_3(r) (see
//! `nearest_pair::cube_pair_probability`), and the number of such pairs is
//! approximately Poisson, so u = 1 − exp(−C(n, 2)·H_3(r)) is approximately
//! uniform.  20 repetitions give 20 values of u, and a Kolmogorov–Smirnov
//! test of them is the result.  Marsaglia scores r³ against an exponential
//! law, which is the leading term of H_3.
//!
//! # Calibration
//!
//! 20 000 separately seeded PCG64 clouds of 4 000 points give values of u
//! with mean 0.4979, 1.05% below 0.01, 1.00% above 0.99 and a
//! Kolmogorov–Smirnov p-value of 0.47 against uniformity.  The quick size,
//! 500 points, is the three-dimensional case of
//! [`crate::dieharder::minimum_distance_nd`].
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{
    diehard::nearest_pair::{cube_pair_probability, min_squared_distance},
    math::ks_test,
    result::TestResult,
    rng::Rng,
};

const CUBE_SIDE: f64 = 1_000.0;

/// Run the 3D spheres test.
///
/// `quick`: use 500 points and 10 repeats instead of 4 000 × 20 to reduce the
/// O(n²) pairwise-distance cost during development.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn spheres_3d(rng: &mut impl Rng, quick: bool) -> TestResult {
    let n_points = if quick { 500 } else { 4_000 };
    let repeats = if quick { 10 } else { 20 };
    let pairs = n_points as f64 * (n_points as f64 - 1.0) / 2.0;
    let mut p_values = Vec::with_capacity(repeats);

    for _ in 0..repeats {
        let points: Vec<[f64; 3]> = (0..n_points)
            .map(|_| {
                [
                    rng.next_f64() * CUBE_SIDE,
                    rng.next_f64() * CUBE_SIDE,
                    rng.next_f64() * CUBE_SIDE,
                ]
            })
            .collect();

        let r = (min_squared_distance(&points).sqrt() / CUBE_SIDE).min(1.0);
        p_values.push(-(-pairs * cube_pair_probability(r, 3)).exp_m1());
    }

    let p_value = ks_test(&mut p_values);
    let d = crate::math::ks_statistic(&mut p_values);

    TestResult::with_note(
        "diehard::spheres_3d",
        p_value,
        format!("n={n_points}, cube={CUBE_SIDE}, repeats={repeats}"),
    )
    .kolmogorov_smirnov(d, p_values.len())
}

#[cfg(test)]
mod tests {
    use super::spheres_3d;
    use crate::rng::ConstantRng;

    /// Every point coincides, so r = 0 in every repeat.
    #[test]
    fn constant_generator_fails() {
        let r = spheres_3d(&mut ConstantRng::new(0), true);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
