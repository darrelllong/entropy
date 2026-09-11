//! DIEHARDER test 201 — rgb_minimum_distance.
//!
//! Generalised nearest-neighbour test for arbitrary dimension d (2..=5).
//! Each trial generates n_points random d-tuples in the unit d-cube, finds the
//! minimum pairwise distance, and converts it to a p-value using the Fischler
//! formula with second-order correction (matches rgb_minimum_distance.c).
//!
//! P-value formula (Fischler, as in `rgb_minimum_distance.c`):
//!   dvolume = ball_volume(mindist, d)
//!   earg = −n(n−1)·dvolume/2
//!   qarg = 1 + ((2 + Q\[d\])/6)·n³·dvolume²
//!   p = 1 − exp(earg)·qarg
//!
//! Q correction table from C source:
//!   Q = [0.0, 0.0, 0.4135, 0.5312, 0.6202, 1.3789]  (indices 0..5)
//!
//! # Author
//! Robert G. Brown, *Dieharder* (2006), test `rgb_minimum_distance`.
//! Source: `dieharder-3.31.1/libdieharder/rgb_minimum_distance.c`

use crate::{
    diehard::nearest_pair::min_squared_distance, math::ks_test, result::TestResult, rng::Rng,
};

/// Fischler Q correction values indexed by dimension d (Q[d] for d=2..=5).
/// Source: `static double rgb_md_Q[] = {0.0,0.0,0.4135,0.5312,0.6202,1.3789}`.
const Q_CORRECTION: [f64; 6] = [0.0, 0.0, 0.4135, 0.5312, 0.6202, 1.3789];

/// Run the N-dimensional minimum distance test.
///
/// `quick`: use 500 points and 20 repeats instead of 8 000 × 100.
///
/// Both full-size parameters sit below Dieharder's defaults
/// (`rgb_minimum_distance.h`): tsamples = 10 000 points per p-value and
/// psamples = 1 000 p-values under the final KS test; 8 000 points is
/// Marsaglia's choice.  The header, citing Fischler, puts the power trade-off
/// in numbers for d = 2 and n = 8 000: about 2 500 trials resolve a
/// consistent 20% error in the local density, while 100 trials resolve only
/// deviations of around 1.5 times the expected density.  The 100 repeats here
/// sit at that weaker end.
///
/// # Author
/// Robert G. Brown, Dieharder (2006), `rgb_minimum_distance`.
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

        // Fischler formula (rgb_minimum_distance.c):
        //   earg = −n(n−1)·dvolume/2
        //   qarg = 1 + ((2+Q[d])/6)·n³·dvolume²
        //   p = 1 − exp(earg)·qarg
        let earg = -n * (n - 1.0) * dvolume / 2.0;
        let qarg = 1.0 + ((2.0 + Q_CORRECTION[D]) / 6.0) * n.powi(3) * dvolume.powi(2);
        let p = 1.0 - earg.exp() * qarg;

        p_values.push(p.clamp(1e-15, 1.0 - 1e-15));
    }
    p_values
}

/// Volume of a d-ball of radius r.
///
/// V_d(r) = π^(d/2) · r^d / Γ(d/2 + 1).
/// Matches `dvolume` in `rgb_minimum_distance.c` (even/odd dimension cases).
fn ball_volume(r: f64, d: usize) -> f64 {
    use std::f64::consts::PI;
    if d.is_multiple_of(2) {
        // Even d: Γ(d/2+1) = (d/2)!
        let half_d = d / 2;
        let factorial: f64 = (1..=half_d).map(|k| k as f64).product();
        PI.powf(half_d as f64) * r.powi(d as i32) / factorial
    } else {
        // Odd d: formula from C source:
        //   2·(2π)^((d-1)/2)·r^d / d!!
        // where d!! = d · (d-2) · … · 1.
        let half_d_minus1 = (d - 1) / 2;
        let double_factorial: f64 = (1..=d).step_by(2).map(|k| k as f64).product();
        2.0 * (2.0 * PI).powf(half_d_minus1 as f64) * r.powi(d as i32) / double_factorial
    }
}

#[cfg(test)]
mod tests {
    use super::minimum_distance_nd;
    use crate::rng::ConstantRng;

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
