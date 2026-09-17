//! DIEHARDER minimum distance test in d = 2 … 5 dimensions.
//!
//! Each repetition draws n uniform points in the unit d-cube and finds the
//! smallest distance r between two of them.  Each of the C(n, 2) pairs lies
//! within r with probability H_d(r), the exact pair probability for the cube
//! (see `nearest_pair::cube_pair_probability`), and the number of such pairs
//! is approximately Poisson, so
//!
//! u = 1 − exp(−C(n, 2)·H_d(r))
//!
//! is approximately uniform.  The values of u over the repetitions get a
//! Kolmogorov–Smirnov test.
//!
//! # Calibration
//!
//! Values of u from separately seeded PCG64 clouds, summarized by their mean
//! and the Kolmogorov–Smirnov p-value against uniformity:
//!
//! | d | 500 points, 40 000 clouds | 8 000 points, 5 000 clouds |
//! |---|---|---|
//! | 2 | mean 0.5010, p = 0.66 | mean 0.4968, p = 0.49 |
//! | 3 | mean 0.4963, p = 0.005 | mean 0.5035, p = 0.14 |
//! | 4 | mean 0.5032, p = 0.02 | mean 0.5015, p = 0.34 |
//! | 5 | mean 0.4999, p = 0.64 | mean 0.4974, p = 0.80 |
//!
//! The Poisson law ignores the dependence between pairs that share a point.
//! At 500 points in three and four dimensions that leaves a bias of a few
//! thousandths in u, which 40 000 clouds resolve and the test's 20-cloud
//! Kolmogorov–Smirnov test does not.  Using the ball volume with Fischler's
//! second-order term in place of H_d gives means of 0.5140 and 0.5249 at 500
//! points in four and five dimensions (KS p < 10⁻⁴).
//!
//! The test's own p-values, from xoshiro256** streams seeded apart, rejected
//! at 0.01 in these fractions of runs (binomial standard deviation 0.07% for
//! 20 000 quick runs, 0.22% for 2 000 full runs):
//!
//! | test | quick | full |
//! |---|---|---|
//! | d = 2 | 1.005% | 1.05% |
//! | d = 3 | 0.950% | 1.20% |
//! | d = 4 | 0.855% | 1.10% |
//! | d = 5 | 1.005% | 1.05% |
//! | DIEHARD minimum distance | 0.980% | 1.25% |
//! | DIEHARD 3-D spheres | 0.910% | 0.85% |
//!
//! # Author
//! Robert G. Brown, *Dieharder: A Random Number Test Suite* (2004–2011).

use crate::{
    diehard::nearest_pair::{cube_pair_probability, min_squared_distance},
    math::ks_test,
    result::TestResult,
    rng::Rng,
};

/// Run the N-dimensional minimum distance test.
///
/// `quick`: use 500 points and 20 repeats instead of 8 000 × 100.
///
/// # Author
/// Robert G. Brown, Dieharder (2006).
pub fn minimum_distance_nd(rng: &mut impl Rng, d: usize, quick: bool) -> TestResult {
    let n_points = if quick { 500 } else { 8_000 };
    let repeats = if quick { 20 } else { 100 };

    // One scan per dimension, each monomorphised over a fixed-size point.
    let mut p_values = match d {
        2 => pair_count_uniforms::<2>(rng, n_points, repeats),
        3 => pair_count_uniforms::<3>(rng, n_points, repeats),
        4 => pair_count_uniforms::<4>(rng, n_points, repeats),
        5 => pair_count_uniforms::<5>(rng, n_points, repeats),
        _ => {
            return TestResult::unsupported("dieharder::minimum_distance_nd", "d must be 2..=5");
        }
    };

    let p_value = ks_test(&mut p_values);
    let ks_d = crate::math::ks_statistic(&mut p_values);

    TestResult::with_note(
        "dieharder::minimum_distance_nd",
        p_value,
        format!("d={d}, n={n_points}, repeats={repeats}"),
    )
    .kolmogorov_smirnov(ks_d, p_values.len())
}

/// One u = 1 − exp(−C(n, 2)·H_d(r)) per repeat, each from `n_points` uniform
/// points in the unit `D`-cube, drawn point by point and coordinate by
/// coordinate.
fn pair_count_uniforms<const D: usize>(
    rng: &mut impl Rng,
    n_points: usize,
    repeats: usize,
) -> Vec<f64> {
    let pairs = n_points as f64 * (n_points as f64 - 1.0) / 2.0;
    // One point buffer, refilled each repeat.
    let mut points = vec![[0.0f64; D]; n_points];
    let mut uniforms = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        for coord in points.iter_mut().flatten() {
            *coord = rng.next_f64();
        }
        // A closest pair farther apart than 1 needs a handful of points; H_d
        // is near 1 there and the formula no longer applies.
        let r = min_squared_distance(&points).sqrt().min(1.0);
        uniforms.push(-(-pairs * cube_pair_probability(r, D)).exp_m1());
    }
    uniforms
}

#[cfg(test)]
mod tests {
    use super::minimum_distance_nd;
    use crate::rng::ConstantRng;

    #[test]
    fn unsupported_dimensions() {
        for d in [0, 1, 6] {
            assert!(minimum_distance_nd(&mut ConstantRng::new(0), d, true).is_unsupported());
        }
    }

    /// Every point coincides, so the minimum distance is zero in every repeat.
    #[test]
    fn constant_generator_fails() {
        for d in 2..=5 {
            let r = minimum_distance_nd(&mut ConstantRng::new(0), d, true);
            assert!(!r.skipped() && !r.passed(), "d = {d}: {r}");
        }
    }
}
