//! DIEHARDER minimum distance test in d = 2 … 5 dimensions.
//!
//! Each repetition draws n uniform points in the unit d-cube and finds the
//! smallest distance r between two of them.  With V the volume of a d-ball
//! of radius r, Fischler's approximation to the distribution of the minimum,
//!
//! P(minimum ≤ r) ≈ 1 − exp(−n(n − 1)V/2)·(1 + ((2 + Q_d)/6)·n³V²),
//!
//! includes a second-order term, with coefficients Q_2 … Q_5 = 0.4135,
//! 0.5312, 0.6202 and 1.3789, for the dependence between pairs sharing a
//! point.  Its values over 100 repetitions of 8 000 points get a
//! Kolmogorov–Smirnov test.  The formula takes each pair's probability of
//! being within r as V, ignoring the boundary of the cube.
//!
//! # Author
//! Robert G. Brown, *Dieharder: A Random Number Test Suite* (2004–2011).
//! Mark Fischler, "Distribution of Minimum Distance among N Random Points in
//! d Dimensions", Fermi National Accelerator Laboratory (2002), for the
//! approximation and Q_d.

use crate::{
    diehard::nearest_pair::min_squared_distance, math::ks_test, result::TestResult, rng::Rng,
};

/// Fischler's second-order coefficients Q_d, indexed by dimension d = 2 … 5.
const Q_CORRECTION: [f64; 6] = [0.0, 0.0, 0.4135, 0.5312, 0.6202, 1.3789];

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
        2 => fischler_pvalues::<2>(rng, n_points, repeats),
        3 => fischler_pvalues::<3>(rng, n_points, repeats),
        4 => fischler_pvalues::<4>(rng, n_points, repeats),
        5 => fischler_pvalues::<5>(rng, n_points, repeats),
        _ => {
            return TestResult::insufficient(
                "dieharder::minimum_distance_nd",
                "d must be 2..=5 (Fischler Q table only covers these dimensions)",
            );
        }
    };

    let p_value = ks_test(&mut p_values);

    TestResult::with_note(
        "dieharder::minimum_distance_nd",
        p_value,
        format!("d={d}, n={n_points}, repeats={repeats}"),
    )
}

/// One Fischler p-value per repeat, each from `n_points` uniform points in
/// the unit `D`-cube, drawn point by point and coordinate by coordinate.
fn fischler_pvalues<const D: usize>(
    rng: &mut impl Rng,
    n_points: usize,
    repeats: usize,
) -> Vec<f64> {
    let n = n_points as f64;
    // One point buffer, refilled each repeat.
    let mut points = vec![[0.0f64; D]; n_points];
    let mut p_values = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        for coord in points.iter_mut().flatten() {
            *coord = rng.next_f64();
        }

        let mindist = min_squared_distance(&points).sqrt();

        // Volume of a d-ball of radius mindist.
        let dvolume = ball_volume(mindist, D);

        // p = 1 − exp(−n(n−1)·V/2)·(1 + ((2 + Q_d)/6)·n³·V²).
        let earg = -n * (n - 1.0) * dvolume / 2.0;
        let qarg = 1.0 + ((2.0 + Q_CORRECTION[D]) / 6.0) * n.powi(3) * dvolume.powi(2);
        let p = 1.0 - earg.exp() * qarg;

        p_values.push(p.clamp(1e-15, 1.0 - 1e-15));
    }
    p_values
}

/// Volume of a d-ball of radius r.
///
/// V_d(r) = π^(d/2) · r^d / Γ(d/2 + 1), with Γ(d/2 + 1) = (d/2)! for even d
/// and V_d(r) = 2(2π)^((d−1)/2) r^d / d!! for odd d.
fn ball_volume(r: f64, d: usize) -> f64 {
    use std::f64::consts::PI;
    if d.is_multiple_of(2) {
        // Even d: Γ(d/2+1) = (d/2)!
        let half_d = d / 2;
        let factorial: f64 = (1..=half_d).map(|k| k as f64).product();
        PI.powf(half_d as f64) * r.powi(d as i32) / factorial
    } else {
        // Odd d: 2·(2π)^((d−1)/2)·r^d / d!!, with d!! = d·(d − 2)·…·1.
        let half_d_minus1 = (d - 1) / 2;
        let double_factorial: f64 = (1..=d).step_by(2).map(|k| k as f64).product();
        2.0 * (2.0 * PI).powf(half_d_minus1 as f64) * r.powi(d as i32) / double_factorial
    }
}

#[cfg(test)]
mod tests {
    use super::minimum_distance_nd;
    use crate::rng::ConstantRng;

    /// The ball volume against V_d(1) = π^(d/2)/Γ(d/2 + 1) for d = 2 … 5.
    #[test]
    fn ball_volumes_of_unit_radius() {
        use std::f64::consts::PI;
        let want = [PI, 4.0 * PI / 3.0, PI * PI / 2.0, 8.0 * PI * PI / 15.0];
        for (d, w) in (2..=5).zip(want) {
            assert!((super::ball_volume(1.0, d) - w).abs() < 1e-14, "d = {d}");
        }
    }

    #[test]
    fn dimensions_outside_the_q_table_skip() {
        for d in [0, 1, 6] {
            assert!(minimum_distance_nd(&mut ConstantRng::new(0), d, true).skipped());
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
