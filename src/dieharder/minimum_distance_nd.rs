//! DIEHARDER test 201 — rgb_minimum_distance.
//!
//! Generalises the DIEHARD 2D and 3D nearest-neighbour tests to arbitrary
//! dimension `d`.  In d dimensions, d² ~ Exp(λ_d) where λ_d depends on d;
//! the exact value is 1/Γ(1 + 1/d) · (Γ(d/2 + 1) / π^(d/2))^(1/d).
//!
//! # Author
//! Robert G. Brown, *Dieharder* (2006), test `rgb_minimum_distance`.

use crate::{math::ks_test, rng::Rng, result::TestResult};

/// Run the N-dimensional minimum distance test.
///
/// `quick`: use 500 points and 20 repeats instead of 8 000 × 100.
///
/// # Author
/// Robert G. Brown, Dieharder (2006), `rgb_minimum_distance`.
pub fn minimum_distance_nd(rng: &mut impl Rng, d: usize, quick: bool) -> TestResult {
    if d < 2 || d > 10 {
        return TestResult::insufficient("dieharder::minimum_distance_nd", "d must be 2..=10");
    }

    let n_points = if quick { 500 } else { 8_000 };
    let repeats  = if quick {  20 } else {   100 };
    let lambda = expected_min_dist_d(d, n_points);

    let mut p_values = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let points: Vec<Vec<f64>> = (0..n_points)
            .map(|_| (0..d).map(|_| rng.next_f64()).collect())
            .collect();

        let min_d2 = min_dist_squared_nd(&points, d);
        // D_min^d ~ Exp(λ) where λ = 2 / (n(n−1) × V_d).
        // Use D_min^d = (D_min²)^(d/2) so the transform gives Exp(1).
        let min_dd = min_d2.powf(d as f64 / 2.0);
        let u = 1.0 - (-min_dd / lambda).exp();
        p_values.push(u.clamp(1e-15, 1.0 - 1e-15));
    }

    let p_value = ks_test(&mut p_values);

    TestResult::with_note(
        "dieharder::minimum_distance_nd",
        p_value,
        format!("d={d}, n={n_points}, repeats={repeats}"),
    )
}

/// Mean of D_min^d for n points in the unit d-cube.
///
/// P(D_min ≤ t) ≈ 1 − exp(−n(n−1)/2 · V_d · t^d), so D_min^d ~ Exp(λ) with
/// λ = 2 / (n(n−1) · V_d).  V_d = π^(d/2) / Γ(d/2 + 1) is the unit d-ball volume.
fn expected_min_dist_d(d: usize, n: usize) -> f64 {
    use std::f64::consts::PI;
    // V_d = π^(d/2) / Γ(d/2 + 1)
    let d2 = d as f64 / 2.0;
    let gamma_d2_plus1 = gamma_half_int(d + 2); // Γ(d/2 + 1)
    let v_d = PI.powf(d2) / gamma_d2_plus1;
    2.0 / ((n * (n - 1)) as f64 * v_d)
}

/// Γ(k/2) for small positive integers k.
fn gamma_half_int(k: usize) -> f64 {
    // Γ(1/2) = √π, Γ(1) = 1, Γ(3/2) = √π/2, Γ(2) = 1, ...
    use std::f64::consts::PI;
    let mut g = if k % 2 == 0 {
        1.0f64 // Γ(1) = 1
    } else {
        PI.sqrt() // Γ(1/2)
    };
    let mut n = if k % 2 == 0 { 2usize } else { 1usize };
    while n < k {
        g *= (n as f64) / 2.0;
        n += 2;
    }
    g
}

fn min_dist_squared_nd(points: &[Vec<f64>], d: usize) -> f64 {
    let mut min_sq = f64::MAX;
    for i in 0..points.len() {
        for j in i + 1..points.len() {
            let sq: f64 = (0..d).map(|k| (points[i][k] - points[j][k]).powi(2)).sum();
            if sq < min_sq { min_sq = sq; }
        }
    }
    min_sq
}
