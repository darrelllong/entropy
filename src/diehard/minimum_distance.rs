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

use crate::{
    diehard::nearest_pair::min_squared_distance, math::ks_test, result::TestResult, rng::Rng,
};

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
        format!("n={n_points}, side={SQUARE_SIDE}, repeats={repeats} [BUGGY FORMULA — see diehard_2dsphere.c; use minimum_distance_nd(d=2) instead]"),
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
