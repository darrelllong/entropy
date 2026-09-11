//! DIEHARD Test 12 — 3D Spheres Test.
//!
//! Places 4 000 random points in a 1 000×1 000×1 000 cube and finds the
//! point with the nearest neighbour.  The radius r of a sphere centred there
//! that just touches its nearest neighbour satisfies: r³ ~ Exp(mean = 30).
//! Thus 1 − exp(−r³/30) ~ U(0,1).  Repeats 20 times; p-values are combined
//! with a Kolmogorov-Smirnov test.
//!
//! DIEHARD's `d3sphere` (`fortran/diehard.f` lines 147–200) combines the 20
//! values with Marsaglia's Anderson–Darling statistic, which `tests.txt`
//! calls a KS test (`KSTEST`, lines 1668–1709), and reports a CDF value; this
//! module applies a Kolmogorov–Smirnov test and reports its upper tail.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).
//! Source: Marsaglia's `fortran/diehard.f`, subroutine `d3sphere`.
//! [pubs/diehard-fortran-1996.tar.gz]

use crate::{
    diehard::nearest_pair::min_squared_distance, math::ks_test, result::TestResult, rng::Rng,
};

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
        let points: Vec<[f64; 3]> = (0..n_points)
            .map(|_| {
                [
                    rng.next_f64() * CUBE_SIDE,
                    rng.next_f64() * CUBE_SIDE,
                    rng.next_f64() * CUBE_SIDE,
                ]
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

/// Cube of the smallest pairwise distance, r³ = (√min r²)³.
///
/// `min_squared_distance` compares squared distances; one square root is
/// taken at the end.  Correctly rounded `sqrt` and the rounded product `r·r·r`
/// are both monotone non-decreasing, so the cube of the root of the smallest
/// r² equals the smallest per-pair cube bit for bit.  Needs at least two
/// points (callers use 500 or 4 000).
fn min_dist_cubed(points: &[[f64; 3]]) -> f64 {
    let r = min_squared_distance(points).sqrt();
    r * r * r
}

#[cfg(test)]
mod tests {
    use super::{min_dist_cubed, spheres_3d};
    use crate::rng::{ConstantRng, Mt19937, Rng};

    /// Per-pair square root and cube, the scan as first written.
    fn per_pair_cube_min(points: &[[f64; 3]]) -> f64 {
        let mut min_r3 = f64::MAX;
        for i in 0..points.len() {
            for j in i + 1..points.len() {
                let dx = points[i][0] - points[j][0];
                let dy = points[i][1] - points[j][1];
                let dz = points[i][2] - points[j][2];
                let r = (dx * dx + dy * dy + dz * dz).sqrt();
                min_r3 = min_r3.min(r * r * r);
            }
        }
        min_r3
    }

    /// The closest pair is (100.5, 200.25, 300.125)–(101, 201, 301) with
    /// r² = 1.578125; r³ comes from a Python replica taking √ then r·r·r.
    #[test]
    fn min_dist_cubed_on_fixed_points() {
        let points = [
            [0.0, 0.0, 0.0],
            [3.0, 4.0, 0.0],
            [10.0, 10.0, 10.0],
            [10.0, 10.0, 12.0],
            [100.5, 200.25, 300.125],
            [101.0, 201.0, 301.0],
            [999.0, 1.0, 500.0],
        ];
        assert_eq!(
            min_dist_cubed(&points).to_bits(),
            1.9824949955726756_f64.to_bits()
        );
    }

    #[test]
    fn squared_scan_matches_per_pair_cubes_bit_for_bit() {
        let mut rng = Mt19937::new(5489);
        for n in [2, 3, 50, 1_000] {
            let mut points: Vec<[f64; 3]> = (0..n)
                .map(|_| {
                    [
                        rng.next_f64() * 1e3,
                        rng.next_f64() * 1e3,
                        rng.next_f64() * 1e3,
                    ]
                })
                .collect();
            let want = per_pair_cube_min(&points);
            assert_eq!(min_dist_cubed(&points).to_bits(), want.to_bits(), "n = {n}");
            // A duplicated point makes the minimum exactly zero.
            points.push(points[0]);
            assert_eq!(min_dist_cubed(&points), 0.0, "n = {n} with a duplicate");
        }
    }

    /// Every point coincides, so r³ = 0 in every repeat.
    #[test]
    fn constant_generator_fails() {
        let r = spheres_3d(&mut ConstantRng::new(0), true);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
