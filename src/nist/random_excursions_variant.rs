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
use std::f64::consts::SQRT_2;

/// States tested: x ∈ {-9,-8,…,-1,+1,…,+9}.
const STATES: [i32; 18] = [-9,-8,-7,-6,-5,-4,-3,-2,-1,1,2,3,4,5,6,7,8,9];

/// Run all 18 random excursions variant sub-tests; return the one with the
/// smallest p-value.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.15.
pub fn random_excursions_variant(bits: &[u8]) -> TestResult {
    let results = random_excursions_variant_all(bits);
    results
        .into_iter()
        .min_by(|a, b| a.p_value.partial_cmp(&b.p_value).unwrap())
        .unwrap_or_else(|| TestResult::insufficient("nist::random_excursions_variant", "J < 500"))
}

/// Run all 18 sub-tests and return a result per state.
pub fn random_excursions_variant_all(bits: &[u8]) -> Vec<TestResult> {
    // Build random walk.
    let (walk, j) = build_walk(bits);

    if j < 500 {
        return vec![TestResult::insufficient(
            "nist::random_excursions_variant",
            &format!("J={j} < 500"),
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
            let p_value = erfc(numer / (denom * SQRT_2));
            TestResult::with_note(
                "nist::random_excursions_variant",
                p_value,
                format!("x={x}, ξ(x)={count}, J={j}"),
            )
        })
        .collect()
}

/// Build the ±1 random walk and count cycles (returns walk and J).
fn build_walk(bits: &[u8]) -> (Vec<i32>, usize) {
    let mut s = 0i32;
    let mut walk = vec![0i32];
    for &b in bits {
        s += if b == 1 { 1 } else { -1 };
        walk.push(s);
    }
    walk.push(0);
    let j = walk.iter().filter(|&&v| v == 0).count() - 1;
    (walk, j)
}
