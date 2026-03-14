//! NIST SP 800-22 §2.14 — Random Excursions Test.
//!
//! Converts bits to a ±1 random walk, finds complete cycles (excursions from
//! zero back to zero), and for each of the 8 non-zero states x ∈ {±1,±2,±3,±4}
//! tests whether the number of cycles that visit state x exactly k times
//! (for k = 0..5) follows the theoretical distribution.
//!
//! Minimum recommended: the number of cycles J ≥ 500 (requires n ≈ 10^6).

use crate::{math::igamc, result::TestResult};

/// States tested: x ∈ {-4,-3,-2,-1,+1,+2,+3,+4}.
const STATES: [i32; 8] = [-4, -3, -2, -1, 1, 2, 3, 4];

/// Run all 8 random excursions sub-tests and return the one with the
/// smallest p-value (most likely to fail).
///
/// Callers that want per-state results should use [`random_excursions_all`].
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.14.
pub fn random_excursions(bits: &[u8]) -> TestResult {
    let results = random_excursions_all(bits);
    results
        .into_iter()
        .min_by(|a, b| a.p_value.partial_cmp(&b.p_value).unwrap())
        .unwrap_or_else(|| TestResult::insufficient("nist::random_excursions", "J < 500"))
}

/// Run all 8 sub-tests and return a result for each state.
pub fn random_excursions_all(bits: &[u8]) -> Vec<TestResult> {
    // Build the random walk partial sums S.
    let walk: Vec<i32> = {
        let mut s = 0i32;
        let mut w = vec![0i32];
        for &b in bits {
            s += if b == 1 { 1 } else { -1 };
            w.push(s);
        }
        w.push(0);
        w
    };

    // Count cycles: runs between consecutive zeros in the walk.
    // A "cycle" is the subsequence between two successive zero-crossings.
    let zero_positions: Vec<usize> =
        walk.iter().enumerate().filter(|(_, &v)| v == 0).map(|(i, _)| i).collect();

    let j = zero_positions.len() - 1; // number of complete cycles

    if j < 500 {
        return vec![TestResult::insufficient(
            "nist::random_excursions",
            &format!("J={j} < 500"),
        )];
    }

    // For each state x, count how many cycles visit x exactly k times, k=0..=5.
    STATES
        .iter()
        .map(|&x| {
            let mut nu = [0usize; 6]; // nu[k] = cycles with exactly k visits to x (k ≥ 5 → nu[5])
            for cycle_idx in 0..j {
                let start = zero_positions[cycle_idx];
                let end = zero_positions[cycle_idx + 1];
                let visits = walk[start + 1..=end].iter().filter(|&&v| v == x).count();
                nu[visits.min(5)] += 1;
            }
            let chi_sq = chi_sq_for_state(x, &nu, j);
            let p_value = igamc(2.5, chi_sq / 2.0); // df = 5
            TestResult::with_note(
                "nist::random_excursions",
                p_value,
                format!("x={x}, J={j}, χ²={chi_sq:.4}"),
            )
        })
        .collect()
}

/// Theoretical probability π_k(x) for exactly k visits to state x in a cycle.
///
/// From SP 800-22 §2.14.3, Table 7.
fn pi_k(x: i32, k: usize) -> f64 {
    let ax = x.unsigned_abs() as f64;
    match k {
        0 => 1.0 - 1.0 / (2.0 * ax),
        1..=4 => (1.0 / (4.0 * ax * ax)) * (1.0 - 1.0 / (2.0 * ax)).powi(k as i32 - 1),
        _ => {
            // k ≥ 5: P(J ≥ 5) = (1 − 1/2|x|)^4 · (1/2|x|)
            let q = 1.0 - 1.0 / (2.0 * ax);
            q.powi(4) * (1.0 / (2.0 * ax))
        }
    }
}

fn chi_sq_for_state(x: i32, nu: &[usize; 6], j: usize) -> f64 {
    (0..6)
        .map(|k| {
            let expected = j as f64 * pi_k(x, k);
            (nu[k] as f64 - expected).powi(2) / expected
        })
        .sum()
}
