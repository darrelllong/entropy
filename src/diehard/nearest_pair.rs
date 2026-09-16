//! Brute-force nearest-pair scan shared by the minimum-distance tests.
//!
//! The minimum distance, 3-D spheres and n-dimensional minimum distance
//! tests each need the smallest Euclidean distance among n points in a
//! square or cube.  All three measure the plain distance inside the box, with
//! no wrap-around.

/// Smallest squared Euclidean distance between two of `points`.
///
/// Every pair is visited once, and each squared distance is summed in
/// coordinate order, (Δ₀² + Δ₁²) + Δ₂² and so on.  Only the value is
/// returned, so which of several tied pairs is met first cannot matter.
/// Callers that need the distance take one square root of the result:
/// correctly rounded `sqrt` is monotone non-decreasing, so that root equals
/// the smallest per-pair root bit for bit.  Returns `f64::MAX` for fewer than
/// two points; every caller passes at least 500.
pub(crate) fn min_squared_distance<const D: usize>(points: &[[f64; D]]) -> f64 {
    let mut min_sq = f64::MAX;
    for (i, p) in points.iter().enumerate() {
        for q in &points[i + 1..] {
            let mut sq = 0.0;
            for (a, b) in p.iter().zip(q) {
                let delta = a - b;
                sq += delta * delta;
            }
            if sq < min_sq {
                min_sq = sq;
            }
        }
    }
    min_sq
}

/// H_d(r), the probability that two independent uniform points in the unit
/// d-cube lie within Euclidean distance r of each other, for 0 ≤ r ≤ 1.
///
/// The difference of two uniform points has density Πᵢ (1 − |zᵢ|) on
/// [−1, 1]^d.  Expanding the product and integrating each monomial over the
/// ball of radius r gives
///
/// H_d(r) = Σ_{k=0..d} (−1)^k C(d, k) π^((d−k)/2) r^(d+k) / Γ(1 + (d+k)/2).
///
/// The k = 0 term is the volume of the ball; the others account for the part
/// of the ball that lies outside the cube near its faces.  Returns NaN for r
/// outside [0, 1].
pub(crate) fn cube_pair_probability(r: f64, d: usize) -> f64 {
    if !(0.0..=1.0).contains(&r) {
        return f64::NAN;
    }
    let mut binomial = 1.0f64;
    let mut sum = 0.0;
    for k in 0..=d {
        let m = (d + k) as f64;
        let term = binomial
            * std::f64::consts::PI.powf((d - k) as f64 / 2.0)
            * r.powf(m)
            * (-crate::math::lgamma(1.0 + m / 2.0)).exp();
        sum += if k % 2 == 0 { term } else { -term };
        binomial *= (d - k) as f64 / (k + 1) as f64;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::{cube_pair_probability, min_squared_distance};
    use crate::rng::{Mt19937, Rng};

    /// H_1(r) = 2r − r² and H_2(r) = πr² − (8/3)r³ + r⁴/2, and H_3(0.4)
    /// matches a Monte Carlo estimate.
    #[test]
    fn cube_pair_probability_closed_forms() {
        for r in [0.0, 0.01, 0.3, 1.0] {
            assert!((cube_pair_probability(r, 1) - (2.0 * r - r * r)).abs() < 1e-14);
            let h2 = std::f64::consts::PI * r * r - 8.0 / 3.0 * r.powi(3) + r.powi(4) / 2.0;
            assert!((cube_pair_probability(r, 2) - h2).abs() < 1e-14, "r = {r}");
        }
        let mut rng = Mt19937::new(99);
        let trials = 400_000;
        let r = 0.4;
        let hits = (0..trials)
            .filter(|_| {
                let d2: f64 = (0..3)
                    .map(|_| (rng.next_f64() - rng.next_f64()).powi(2))
                    .sum();
                d2 <= r * r
            })
            .count();
        let estimate = hits as f64 / trials as f64;
        let exact = cube_pair_probability(r, 3);
        let se = (exact * (1.0 - exact) / trials as f64).sqrt();
        assert!((estimate - exact).abs() < 4.0 * se, "{estimate} vs {exact}");
        assert!(cube_pair_probability(1.5, 2).is_nan());
    }

    /// (0, 0)–(0.5, 0.5) and (3, 4)–(3.5, 4.5) tie at squared distance 0.5.
    #[test]
    fn finds_the_smallest_squared_distance() {
        let points = [[0.0, 0.0], [3.0, 4.0], [0.5, 0.5], [10.0, 10.0], [3.5, 4.5]];
        assert_eq!(min_squared_distance(&points), 0.5);
        assert_eq!(min_squared_distance::<2>(&[]), f64::MAX);
        assert_eq!(min_squared_distance(&[[1.0, 2.0, 3.0]]), f64::MAX);
    }

    /// An independent scan: a root per pair, with the squares summed by an
    /// iterator.
    fn per_pair_root_min<const D: usize>(points: &[[f64; D]]) -> f64 {
        let mut min_dist = f64::MAX;
        for i in 0..points.len() {
            for j in i + 1..points.len() {
                let dist_sq: f64 = points[i]
                    .iter()
                    .zip(&points[j])
                    .map(|(a, b)| (a - b).powi(2))
                    .sum();
                let dist = dist_sq.sqrt();
                if dist < min_dist {
                    min_dist = dist;
                }
            }
        }
        min_dist
    }

    fn root_matches_per_pair_roots<const D: usize>(rng: &mut Mt19937) {
        for n in [2, 3, 300] {
            let fine: Vec<[f64; D]> = (0..n)
                .map(|_| std::array::from_fn(|_| rng.next_f64()))
                .collect();
            // Quarter-step coordinates: many tied and coincident pairs.
            let coarse: Vec<[f64; D]> = (0..n)
                .map(|_| std::array::from_fn(|_| (rng.next_f64() * 4.0).floor() / 4.0))
                .collect();
            for points in [fine, coarse] {
                assert_eq!(
                    min_squared_distance(&points).sqrt().to_bits(),
                    per_pair_root_min(&points).to_bits(),
                    "D = {D}, n = {n}"
                );
            }
        }
    }

    #[test]
    fn root_of_the_minimum_is_the_minimum_root_bit_for_bit() {
        let mut rng = Mt19937::new(5489);
        root_matches_per_pair_roots::<2>(&mut rng);
        root_matches_per_pair_roots::<3>(&mut rng);
        root_matches_per_pair_roots::<4>(&mut rng);
        root_matches_per_pair_roots::<5>(&mut rng);
    }
}
