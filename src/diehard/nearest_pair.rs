//! Brute-force nearest-pair scan shared by the minimum-distance tests.
//!
//! DIEHARD's minimum distance and 3-D spheres tests (George Marsaglia,
//! *DIEHARD: A Battery of Tests of Randomness*, 1995) and Dieharder's
//! `rgb_minimum_distance` (Robert G. Brown, Dieharder 3.31.1) each need the
//! smallest Euclidean distance among n points in a square or cube.  All three
//! measure the plain distance inside the box, with no wrap-around:
//! `diehard_2dsphere.c` and `rgb_minimum_distance.c` both say they omit
//! periodic boundaries.  `pubs/dieharder-3.31.1.tgz`

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

#[cfg(test)]
mod tests {
    use super::min_squared_distance;
    use crate::rng::{Mt19937, Rng};

    /// (0, 0)–(0.5, 0.5) and (3, 4)–(3.5, 4.5) tie at squared distance 0.5.
    #[test]
    fn finds_the_smallest_squared_distance() {
        let points = [[0.0, 0.0], [3.0, 4.0], [0.5, 0.5], [10.0, 10.0], [3.5, 4.5]];
        assert_eq!(min_squared_distance(&points), 0.5);
        assert_eq!(min_squared_distance::<2>(&[]), f64::MAX);
        assert_eq!(min_squared_distance(&[[1.0, 2.0, 3.0]]), f64::MAX);
    }

    /// The scan `minimum_distance_nd` used before the merge: a root per pair,
    /// with the squares summed by an iterator.
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
