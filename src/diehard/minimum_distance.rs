//! DIEHARD Test 11 — Minimum Distance Test (2D).
//!
//! Places 8 000 random points in a 10 000×10 000 square and finds the
//! minimum pairwise distance d.  The quantity d² should be exponentially
//! distributed with mean 0.995.  Repeats 100 times; 100 p-values are
//! tested with a Kolmogorov-Smirnov test.
//!
//! # ⚠ Known-Buggy Formula
//!
//! The original DIEHARD formula `1 − exp(−d²/λ)` is **acknowledged as buggy
//! and obsolete** by the Dieharder maintainer (see `diehard_2dsphere.c`:
//! "This test is OBSOLETE. ... The formula used here is WRONG.").  The
//! corrected version is the Fischler formula implemented in
//! [`crate::dieharder::minimum_distance_nd`] with `d = 2`.
//!
//! This module preserves the original buggy formula solely for historical
//! comparison with legacy DIEHARD output.  **Do not use this result for
//! genuine randomness assessment.**
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::ks_test, result::TestResult, rng::Rng};

const SQUARE_SIDE: f64 = 10_000.0;
const LAMBDA: f64 = 0.995; // expected mean of d²

/// Run the 2D minimum distance test (legacy buggy formula — see module docs).
///
/// `quick`: use 500 points and 20 repeats instead of 8 000 × 100 to avoid the
/// O(n²) cost during development.
///
/// # ⚠ Buggy Formula
/// Uses `1 − exp(−d²/λ)`, which the Dieharder maintainer explicitly marks as
/// wrong.  Use [`crate::dieharder::minimum_distance_nd`] for a correct result.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn minimum_distance_2d(rng: &mut impl Rng, quick: bool) -> TestResult {
    let n_points = if quick { 500 } else { 8_000 };
    let repeats = if quick { 20 } else { 100 };
    // With fewer points the nearest pair is farther apart, so λ scales as (n_ref/n)².
    // For n=8000, side=10000: λ ≈ 0.995.  Reference: LAMBDA × (8000/n)².
    let lambda = LAMBDA * (8_000.0 / n_points as f64).powi(2);
    let mut p_values = Vec::with_capacity(repeats);

    for _ in 0..repeats {
        let points: Vec<(f64, f64)> = (0..n_points)
            .map(|_| (rng.next_f64() * SQUARE_SIDE, rng.next_f64() * SQUARE_SIDE))
            .collect();

        let d_sq = min_dist_squared(&points);
        let u = 1.0 - (-d_sq / lambda).exp();
        p_values.push(u.clamp(1e-15, 1.0 - 1e-15));
    }

    let p_value = ks_test(&mut p_values);

    TestResult::with_note(
        "diehard::minimum_distance_2d",
        p_value,
        format!("n={n_points}, side={SQUARE_SIDE}, repeats={repeats} [BUGGY FORMULA — see diehard_2dsphere.c; use minimum_distance_nd(d=2) instead]"),
    )
}

/// Find the minimum squared Euclidean distance among all pairs.
///
/// Uses a naïve O(n²) scan; for n = 8 000 this is 32 million comparisons —
/// acceptable for a test suite.
fn min_dist_squared(points: &[(f64, f64)]) -> f64 {
    let mut min_sq = f64::MAX;
    for i in 0..points.len() {
        for j in i + 1..points.len() {
            let dx = points[i].0 - points[j].0;
            let dy = points[i].1 - points[j].1;
            let sq = dx * dx + dy * dy;
            if sq < min_sq {
                min_sq = sq;
            }
        }
    }
    min_sq
}
