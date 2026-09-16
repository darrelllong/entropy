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
//!
//! # References
//! * A. Rukhin et al., *NIST SP 800-22 Rev. 1a*, 2010, §2.15 and §3.15.
//!   [pubs/NIST-SP-800-22r1a.pdf]

use crate::{math::erfc, result::TestResult};

/// Largest |x| among the tested states.
const MAX_STATE: i32 = 9;

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
///
/// Always returns 18 results, for x = −9, …, −1, +1, …, +9 in that order.
/// When the walk has fewer than max(0.005·√n, 500) cycles every entry is a
/// skipped `nist::random_excursions_variant` result, so the vector keeps its
/// length.
pub fn random_excursions_variant_all(bits: &[u8]) -> Vec<TestResult> {
    // Build random walk.
    let (walk, j) = build_walk(bits);

    // §2.15.4 (as in sts): J must be at least max(0.005·√n, 500).
    let j_min = (0.005 * (bits.len() as f64).sqrt()).max(500.0);
    if (j as f64) < j_min {
        let why = format!("J={j} < {j_min:.0}");
        return STATES
            .iter()
            .map(|_| TestResult::insufficient("nist::random_excursions_variant", &why))
            .collect();
    }

    // Count total visits per state across the entire walk (excluding
    // endpoints) in a fixed array indexed by x + MAX_STATE.  States beyond
    // ±MAX_STATE are not tested, so their visits are not counted.
    let mut visit_counts = [0usize; 2 * MAX_STATE as usize + 1];
    for &s in &walk[1..walk.len() - 1] {
        if (-MAX_STATE..=MAX_STATE).contains(&s) {
            visit_counts[(s + MAX_STATE) as usize] += 1;
        }
    }

    STATES
        .iter()
        .map(|&x| {
            let count = visit_counts[(x + MAX_STATE) as usize] as f64;
            let numer = (count - j as f64).abs();
            let denom = (2.0 * j as f64 * (4.0 * x.unsigned_abs() as f64 - 2.0)).sqrt();
            // §2.15.4 step (5): erfc(|ξ(x) − J|/√(2J(4|x| − 2))).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nist::test_vectors::e_bits;
    use crate::rng::{Mt19937, Rng};
    use std::collections::HashMap;

    /// A walk with J = 1 still yields one skipped entry per state, and the
    /// Bonferroni wrapper passes the skip through.
    #[test]
    fn short_walk_yields_one_skipped_entry_per_state() {
        let bits = [1u8; 1000];
        let results = random_excursions_variant_all(&bits);
        assert_eq!(results.len(), STATES.len());
        for r in &results {
            assert_eq!(r.name, "nist::random_excursions_variant");
            assert!(r.skipped(), "{r}");
        }
        let family = random_excursions_variant(&bits);
        assert_eq!(family.name, "nist::random_excursions_variant");
        assert!(family.skipped(), "{family}");
    }

    /// Per-state results computed independently, with the visit counts in a
    /// `HashMap`.
    fn per_state_with_hashmap(bits: &[u8]) -> Vec<TestResult> {
        let (walk, j) = build_walk(bits);
        let mut visit_counts = HashMap::new();
        for &s in &walk[1..walk.len() - 1] {
            *visit_counts.entry(s).or_insert(0usize) += 1;
        }
        STATES
            .iter()
            .map(|&x| {
                let count = *visit_counts.get(&x).unwrap_or(&0) as f64;
                let numer = (count - j as f64).abs();
                let denom = (2.0 * j as f64 * (4.0 * x.unsigned_abs() as f64 - 2.0)).sqrt();
                TestResult::with_note(
                    "nist::random_excursions_variant",
                    erfc(numer / denom),
                    format!("x={x}, ξ(x)={count}, J={j}"),
                )
            })
            .collect()
    }

    /// The fixed-array counts give bit-identical results to the `HashMap`
    /// counts on 10⁶ bits of Mt19937 seed 1, whose walk has J = 1302 and
    /// spans −875 to +261, so every tested state and the out-of-range branch
    /// run.
    #[test]
    fn fixed_array_matches_hashmap_counts() {
        let bits = Mt19937::new(1).collect_bits(1_000_000);
        let results = random_excursions_variant_all(&bits);
        let reference = per_state_with_hashmap(&bits);
        assert_eq!(results.len(), reference.len());
        for (r, want) in results.iter().zip(&reference) {
            assert!(!r.skipped(), "{r}");
            assert_eq!(r.p_value.to_bits(), want.p_value.to_bits(), "{r}");
            assert_eq!(r.note, want.note);
        }
    }

    /// SP 800-22 §2.15.8 on 10⁶ bits of e: J = 1490 and, for each state, the
    /// printed number of visits and P-value.
    #[test]
    fn matches_section_2_15_8_example() {
        const PRINTED: [(i32, usize, f64); 18] = [
            (-9, 1450, 0.858946),
            (-8, 1435, 0.794755),
            (-7, 1380, 0.576249),
            (-6, 1366, 0.493417),
            (-5, 1412, 0.633873),
            (-4, 1475, 0.917283),
            (-3, 1480, 0.934708),
            (-2, 1468, 0.816012),
            (-1, 1502, 0.826009),
            (1, 1409, 0.137861),
            (2, 1369, 0.200642),
            (3, 1396, 0.441254),
            (4, 1479, 0.939291),
            (5, 1599, 0.505683),
            (6, 1628, 0.445935),
            (7, 1619, 0.512207),
            (8, 1620, 0.538635),
            (9, 1610, 0.593930),
        ];
        let results = random_excursions_variant_all(&e_bits(1_000_000));
        for (r, (x, visits, p)) in results.iter().zip(PRINTED) {
            let note = r.note.as_deref().unwrap();
            assert!(
                note.contains(&format!("x={x}, ξ(x)={visits}, J=1490")),
                "{r}"
            );
            assert!((r.p_value - p).abs() < 1e-6, "{r}");
        }
    }
}
