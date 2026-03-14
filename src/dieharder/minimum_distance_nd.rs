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
//!   qarg = 1 + ((2 + Q[d])/6)·n³·dvolume²
//!   p = 1 − exp(earg)·qarg
//!
//! Q correction table from C source:
//!   Q = [0.0, 0.0, 0.4135, 0.5312, 0.6202, 1.3789]  (indices 0..5)
//!
//! # Author
//! Robert G. Brown, *Dieharder* (2006), test `rgb_minimum_distance`.
//! Source: `dieharder-3.31.1/libdieharder/rgb_minimum_distance.c`

use crate::{math::ks_test, rng::Rng, result::TestResult};

/// Fischler Q correction values indexed by dimension d (Q[d] for d=2..=5).
/// Source: `static double rgb_md_Q[] = {0.0,0.0,0.4135,0.5312,0.6202,1.3789}`.
const Q_CORRECTION: [f64; 6] = [0.0, 0.0, 0.4135, 0.5312, 0.6202, 1.3789];

/// Run the N-dimensional minimum distance test.
///
/// `quick`: use 500 points and 20 repeats instead of 8 000 × 100.
///
/// # Author
/// Robert G. Brown, Dieharder (2006), `rgb_minimum_distance`.
pub fn minimum_distance_nd(rng: &mut impl Rng, d: usize, quick: bool) -> TestResult {
    if d < 2 || d > 5 {
        return TestResult::insufficient(
            "dieharder::minimum_distance_nd",
            "d must be 2..=5 (Fischler Q table only covers these dimensions)",
        );
    }

    let n_points = if quick { 500 } else { 8_000 };
    let repeats  = if quick {  20 } else {   100 };

    let mut p_values = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let points: Vec<Vec<f64>> = (0..n_points)
            .map(|_| (0..d).map(|_| rng.next_f64()).collect())
            .collect();

        let mindist = min_dist_nd(&points, d);

        // Volume of a d-ball of radius mindist.
        let dvolume = ball_volume(mindist, d);

        // Fischler formula (rgb_minimum_distance.c):
        //   earg = −n(n−1)·dvolume/2
        //   qarg = 1 + ((2+Q[d])/6)·n³·dvolume²
        //   p = 1 − exp(earg)·qarg
        let n = n_points as f64;
        let earg = -n * (n - 1.0) * dvolume / 2.0;
        let qarg = 1.0 + ((2.0 + Q_CORRECTION[d]) / 6.0) * n.powi(3) * dvolume.powi(2);
        let p = 1.0 - earg.exp() * qarg;

        p_values.push(p.clamp(1e-15, 1.0 - 1e-15));
    }

    let p_value = ks_test(&mut p_values);

    TestResult::with_note(
        "dieharder::minimum_distance_nd",
        p_value,
        format!("d={d}, n={n_points}, repeats={repeats}"),
    )
}

/// Volume of a d-ball of radius r.
///
/// V_d(r) = π^(d/2) · r^d / Γ(d/2 + 1).
/// Matches `dvolume` in `rgb_minimum_distance.c` (even/odd dimension cases).
fn ball_volume(r: f64, d: usize) -> f64 {
    use std::f64::consts::PI;
    if d % 2 == 0 {
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

/// Minimum Euclidean distance among all pairs of d-dimensional points.
fn min_dist_nd(points: &[Vec<f64>], d: usize) -> f64 {
    let mut min_dist = f64::MAX;
    for i in 0..points.len() {
        for j in i + 1..points.len() {
            let dist: f64 = (0..d)
                .map(|k| (points[i][k] - points[j][k]).powi(2))
                .sum::<f64>()
                .sqrt();
            if dist < min_dist { min_dist = dist; }
        }
    }
    min_dist
}
