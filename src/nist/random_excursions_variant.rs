//! NIST SP 800-22 §2.15 — Random Excursions Variant Test.
//!
//! For each of 18 states x ∈ {±1,…,±9}, tests whether the total number of
//! visits to state x across all cycles follows the expected distribution
//! (approximately normal for large J).
//!
//! Unlike §2.14 (which bins per-cycle visit counts), this test examines the
//! aggregate visit count over the entire walk.
//!
//! Minimum recommended: J ≥ 500.

use crate::{math::erfc, result::TestResult};

/// States tested: x ∈ {-9,-8,…,-1,+1,…,+9}.
const STATES: [i32; 18] = [
    -9, -8, -7, -6, -5, -4, -3, -2, -1, 1, 2, 3, 4, 5, 6, 7, 8, 9,
];

/// Run all 18 random excursions variant sub-tests and report the worst
/// state's p-value with a Bonferroni correction for the 18 states examined
/// (valid under their dependence — all states share one walk).
///
/// Callers that want per-state results should use
/// [`random_excursions_variant_all`] (which `run_all` uses).
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.15.
pub fn random_excursions_variant(bits: &[u8]) -> TestResult {
    let results = random_excursions_variant_all(bits);
    let m = results.len() as f64;
    results
        .into_iter()
        .min_by(|a, b| {
            a.p_value
                .partial_cmp(&b.p_value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|worst| {
            if worst.skipped() {
                worst
            } else {
                TestResult::with_note(
                    "nist::random_excursions_variant",
                    (m * worst.p_value).min(1.0),
                    format!(
                        "Bonferroni over {m} states; worst: {}",
                        worst.note.unwrap_or_default()
                    ),
                )
            }
        })
        .unwrap_or_else(|| TestResult::insufficient("nist::random_excursions_variant", "J < 500"))
}

/// Run all 18 sub-tests and return a result per state.
pub fn random_excursions_variant_all(bits: &[u8]) -> Vec<TestResult> {
    // Build random walk.
    let (walk, j) = build_walk(bits);

    // §2.15.4 (as in sts): J must be at least max(0.005·√n, 500).
    let j_min = (0.005 * (bits.len() as f64).sqrt()).max(500.0);
    if (j as f64) < j_min {
        return vec![TestResult::insufficient(
            "nist::random_excursions_variant",
            &format!("J={j} < {j_min:.0}"),
        )];
    }

    // Count total visits per state across the entire walk (excluding endpoints).
    let mut visit_counts = std::collections::HashMap::new();
    for &s in &walk[1..walk.len() - 1] {
        *visit_counts.entry(s).or_insert(0usize) += 1;
    }

    STATES
        .iter()
        .map(|&x| {
            let count = *visit_counts.get(&x).unwrap_or(&0) as f64;
            let numer = (count - j as f64).abs();
            let denom = (2.0 * j as f64 * (4.0 * x.unsigned_abs() as f64 - 2.0)).sqrt();
            // NIST STS randomexcursionsvariant.c: erfc(|ξ(x)-J|/√(2J(4|x|-2))).
            let p_value = erfc(numer / denom);
            TestResult::with_note(
                "nist::random_excursions_variant",
                p_value,
                format!("x={x}, ξ(x)={count}, J={j}"),
            )
        })
        .collect()
}

/// Build the ±1 random walk and count cycles (returns walk and J).
///
/// The closing zero is appended only when Sₙ ≠ 0; an unconditional append
/// would create a spurious empty cycle when the walk already ends at zero
/// (see the same fix in §2.14).
fn build_walk(bits: &[u8]) -> (Vec<i32>, usize) {
    let mut s = 0i32;
    let mut walk = vec![0i32];
    for &b in bits {
        s += if b == 1 { 1 } else { -1 };
        walk.push(s);
    }
    if s != 0 {
        walk.push(0);
    }
    let j = walk.iter().filter(|&&v| v == 0).count() - 1;
    (walk, j)
}
