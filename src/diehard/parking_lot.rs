//! DIEHARD Test 10 — Parking Lot Test.
//!
//! Tries to park 12 000 unit-radius "cars" (circles) in a 100×100 square.
//! A new car is placed at a random (x,y) location; it is rejected if it
//! overlaps any previously parked car.  The number of cars successfully
//! parked should be approximately normal with mean = 3 523, σ = 21.9.
//!
//! The test repeats 10 times; the 10 resulting z-scores are converted to
//! p-values and tested with a Kolmogorov-Smirnov test.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::ks_test, result::TestResult, rng::Rng};

const ATTEMPTS: usize = 12_000;
// Marsaglia's published values for circular unit-radius cars (Rényi, 1958).
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

    TestResult::with_note(
        "diehard::parking_lot",
        p_value,
        format!("attempts={ATTEMPTS}, mean={MEAN}, σ={SIGMA}, repeats={repeats}"),
    )
}

fn simulate(rng: &mut impl Rng) -> usize {
    let mut cars: Vec<(f64, f64)> = Vec::with_capacity(ATTEMPTS / 3);
    let mut parked = 0usize;

    for _ in 0..ATTEMPTS {
        let x = rng.next_f64() * 100.0;
        let y = rng.next_f64() * 100.0;
        // Unit-radius circular cars: no overlap iff Euclidean distance ≥ 2.
        let fits = cars.iter().all(|&(cx, cy)| {
            let dx = cx - x;
            let dy = cy - y;
            dx * dx + dy * dy >= 4.0
        });
        if fits {
            cars.push((x, y));
            parked += 1;
        }
    }
    parked
}
