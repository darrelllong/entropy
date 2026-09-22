//! DIEHARD parking lot test.
//!
//! Makes 12 000 attempts to park a car, a square of side 1, at a uniform
//! position in a 100 × 100 lot.  An attempt fails if the new car overlaps a
//! parked one, max(|Δx|, |Δy|) < 1.  The number parked is approximately normal
//! with mean 3 523 and standard deviation 21.9, values Marsaglia determined
//! by simulation; 2 000 000 lots here (`examples/statistic_scale.rs`,
//! `stats/statistic-scale-moore.txt`) give 3 523.49 and 21.86, and 0.98% of
//! them beyond his two-sided 1% point, so his values stand.  Ten lots give
//! ten values of Φ(z), and a Kolmogorov–Smirnov test of them is the result;
//! over 200 000 null runs that test rejects 1.09% at the 1% level, an excess
//! the count's law does not account for.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::ks_test, result::TestResult, rng::Rng};

/// Attempts to park per lot.
pub const ATTEMPTS: usize = 12_000;
/// Mean and standard deviation of the number parked (Marsaglia, DIEHARD).
const MEAN: f64 = 3_523.0;
const SIGMA: f64 = 21.9;

/// Run the parking lot test.
///
/// `quick`: use 5 repeats instead of 10 to reduce the O(n²) collision-check cost.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn parking_lot(rng: &mut impl Rng, quick: bool) -> TestResult {
    let repeats = if quick { 5 } else { 10 };
    let mut p_values = Vec::with_capacity(repeats);

    for _ in 0..repeats {
        let parked = simulate(rng);
        let z = (parked as f64 - MEAN) / SIGMA;
        // Convert z to uniform via normal CDF.
        let p = crate::math::normal_cdf(z);
        p_values.push(p.clamp(1e-15, 1.0 - 1e-15));
    }

    let p_value = ks_test(&mut p_values);
    let d = crate::math::ks_statistic(&mut p_values);

    TestResult::with_note(
        "diehard::parking_lot",
        p_value,
        format!("attempts={ATTEMPTS}, mean={MEAN}, σ={SIGMA}, repeats={repeats}"),
    )
    .kolmogorov_smirnov(d, p_values.len())
}

/// The number of cars parked in one lot of [`ATTEMPTS`] attempts: the
/// quantity whose null mean and standard deviation the test standardises by.
pub fn parked(rng: &mut impl Rng) -> usize {
    simulate(rng)
}

fn simulate(rng: &mut impl Rng) -> usize {
    let mut cars: Vec<(f64, f64)> = Vec::with_capacity(ATTEMPTS / 3);
    let mut parked = 0usize;

    for _ in 0..ATTEMPTS {
        let x = rng.next_f64() * 100.0;
        let y = rng.next_f64() * 100.0;
        // L∞ collision criterion: no overlap iff max(|dx|, |dy|) ≥ 1 (square cars of side 1).
        let fits = cars
            .iter()
            .all(|&(cx, cy)| (cx - x).abs() >= 1.0 || (cy - y).abs() >= 1.0);
        if fits {
            cars.push((x, y));
            parked += 1;
        }
    }
    parked
}

#[cfg(test)]
mod tests {
    use super::parking_lot;
    use crate::rng::ConstantRng;

    /// Every car lands on the first, so one car parks per repeat.
    #[test]
    fn constant_generator_fails() {
        let r = parking_lot(&mut ConstantRng::new(0), true);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
