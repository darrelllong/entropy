//! DIEHARD Test 12 — 3D Spheres Test.
//!
//! Places 4 000 random points in a 1 000×1 000×1 000 cube and finds the
//! point with the nearest neighbour.  The radius r of a sphere centred there
//! that just touches its nearest neighbour satisfies: r³ ~ Exp(mean = 30).
//! Thus 1 − exp(−r³/30) ~ U(0,1).  Repeats 20 times; p-values are combined
//! with a Kolmogorov-Smirnov test.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::ks_test, result::TestResult, rng::Rng};

const CUBE_SIDE: f64 = 1_000.0;
const MEAN_R3: f64 = 30.0;

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
    // λ = n(n−1)/2 × (4π/3)/L³, so mean r³ = 1/λ ∝ 1/(n²).
    let mean_r3 = MEAN_R3 * (4_000.0 / n_points as f64).powi(2);
    let mut p_values = Vec::with_capacity(repeats);

    for _ in 0..repeats {
        let points: Vec<(f64, f64, f64)> = (0..n_points)
            .map(|_| {
                (
                    rng.next_f64() * CUBE_SIDE,
                    rng.next_f64() * CUBE_SIDE,
                    rng.next_f64() * CUBE_SIDE,
                )
            })
            .collect();

        let min_r3 = min_dist_cubed(&points);
        let u = 1.0 - (-min_r3 / mean_r3).exp();
        p_values.push(u.clamp(1e-15, 1.0 - 1e-15));
    }

    let p_value = ks_test(&mut p_values);

    TestResult::with_note(
        "diehard::spheres_3d",
        p_value,
        format!("n={n_points}, cube={CUBE_SIDE}, repeats={repeats}"),
    )
}

fn min_dist_cubed(points: &[(f64, f64, f64)]) -> f64 {
    let mut min_r3 = f64::MAX;
    for i in 0..points.len() {
        for j in i + 1..points.len() {
            let dx = points[i].0 - points[j].0;
            let dy = points[i].1 - points[j].1;
            let dz = points[i].2 - points[j].2;
            let r2 = dx * dx + dy * dy + dz * dz;
            let r = r2.sqrt();
            let r3 = r * r * r;
            if r3 < min_r3 {
                min_r3 = r3;
            }
        }
    }
    min_r3
}
