//! DIEHARDER test 17 — marsaglia_tsang_gcd.
//!
//! Tests the distribution of GCD values and the number of steps taken by the
//! Euclidean algorithm applied to pairs of random 32-bit integers.  Weak
//! generators produce GCD distributions that deviate from the theoretical
//! values (which follow from the density of coprime integer pairs = 6/π²).
//!
//! # Author
//! George Marsaglia and Wai Wan Tsang, "Some Difficult-to-pass Tests of
//! Randomness", *Journal of Statistical Software* 7(3), 2002.
//! <https://doi.org/10.18637/jss.v007.i03>

use crate::{math::igamc, rng::Rng, result::TestResult};

const N_PAIRS: usize = 100_000;

// Theoretical probability that gcd(u, v) = k for random u,v is 6/(π²·k²).
// We bin into gcd = 1, 2, 3, 4, ≥5.
// P(gcd=1) ≈ 6/π² ≈ 0.6079, P(gcd=2) ≈ 6/(4π²), etc.

/// Run the GCD distribution test.
///
/// # Author
/// George Marsaglia and Wai Wan Tsang, *Journal of Statistical Software*
/// 7(3), 2002.  doi:10.18637/jss.v007.i03
pub fn gcd(rng: &mut impl Rng) -> TestResult {
    const MAX_GCD_BIN: usize = 5; // bins: 1,2,3,4,≥5
    let mut gcd_counts = [0u32; MAX_GCD_BIN];
    let mut step_counts: Vec<u32> = Vec::new();

    for _ in 0..N_PAIRS {
        let u = rng.next_u32();
        let v = rng.next_u32();
        if u == 0 || v == 0 { continue; }
        let (g, steps) = euclid_gcd_with_steps(u, v);
        let idx = ((g as usize) - 1).min(MAX_GCD_BIN - 1);
        gcd_counts[idx] += 1;
        if step_counts.len() <= steps { step_counts.resize(steps + 1, 0); }
        step_counts[steps] += 1;
    }

    // Theoretical GCD probabilities: P(gcd = k) = 6/(π² k²).
    use std::f64::consts::PI;
    let pi2 = PI * PI;
    let pi_k: Vec<f64> = (1..=MAX_GCD_BIN)
        .map(|k| {
            if k < MAX_GCD_BIN {
                6.0 / (pi2 * (k as f64).powi(2))
            } else {
                // P(gcd ≥ 5) = 1 − Σ_{k=1}^{4} 6/(π²k²)
                1.0 - (1..MAX_GCD_BIN).map(|j| 6.0 / (pi2 * (j as f64).powi(2))).sum::<f64>()
            }
        })
        .collect();

    let chi_sq: f64 = gcd_counts
        .iter()
        .zip(pi_k.iter())
        .filter(|(_, &p)| p * N_PAIRS as f64 >= 5.0)
        .map(|(&c, &p)| {
            let exp = p * N_PAIRS as f64;
            (c as f64 - exp).powi(2) / exp
        })
        .sum();

    let df = gcd_counts
        .iter()
        .zip(pi_k.iter())
        .filter(|(_, &p)| p * N_PAIRS as f64 >= 5.0)
        .count()
        .saturating_sub(1);

    let p_value = igamc(df as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "dieharder::gcd",
        p_value,
        format!("pairs={N_PAIRS}, gcd_counts={gcd_counts:?}, χ²={chi_sq:.4}"),
    )
}

/// Compute gcd(a, b) using the Euclidean algorithm; also return the step count.
fn euclid_gcd_with_steps(mut a: u32, mut b: u32) -> (u32, usize) {
    let mut steps = 0usize;
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
        steps += 1;
    }
    (a, steps)
}
