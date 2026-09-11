//! NIST SP 800-22 §2.14 — Random Excursions Test.
//!
//! Converts bits to a ±1 random walk, finds complete cycles (excursions from
//! zero back to zero), and for each of the 8 non-zero states x ∈ {±1,±2,±3,±4}
//! tests whether the number of cycles that visit state x exactly k times
//! (for k = 0..5) follows the theoretical distribution.
//!
//! Minimum recommended: the number of cycles J ≥ 500 (requires n ≈ 10^6).
//!
//! # References
//! * A. Rukhin et al., *NIST SP 800-22 Rev. 1a*, 2010, §2.14 and §3.14.
//!   [pubs/NIST-SP-800-22r1a.pdf]
//! * NIST, *Statistical Test Suite* 2.1.2, `src/randomExcursions.c`.
//!   [pubs/NIST-STS-2.1.2-src-and-constants.zip]  [Same cycles, J gate and
//!   probabilities; below the gate it writes P-value 0 for every state, where
//!   this module reports a skip]

use crate::{math::chi2_pvalue, result::TestResult};

/// States tested: x ∈ {-4,-3,-2,-1,+1,+2,+3,+4}.
const STATES: [i32; 8] = [-4, -3, -2, -1, 1, 2, 3, 4];

/// Run all 8 random excursions sub-tests and report the worst state's
/// p-value with a Bonferroni correction for the 8 states examined
/// (valid under their dependence — all states share one walk).
///
/// Callers that want per-state results should use [`random_excursions_all`]
/// (which `run_all` uses).
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.14.
pub fn random_excursions(bits: &[u8]) -> TestResult {
    let results = random_excursions_all(bits);
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
                    "nist::random_excursions",
                    (m * worst.p_value).min(1.0),
                    format!(
                        "Bonferroni over {m} states; worst: {}",
                        worst.note.unwrap_or_default()
                    ),
                )
            }
        })
        .unwrap_or_else(|| TestResult::insufficient("nist::random_excursions", "J < 500"))
}

/// Run all 8 sub-tests and return a result for each state.
///
/// Always returns 8 results, for x = −4, …, −1, +1, …, +4 in that order.
/// When the walk has fewer than max(0.005·√n, 500) cycles every entry is a
/// skipped `nist::random_excursions` result, so the vector keeps its length.
pub fn random_excursions_all(bits: &[u8]) -> Vec<TestResult> {
    match state_chi_squares(bits) {
        Err(why) => STATES
            .iter()
            .map(|_| TestResult::insufficient("nist::random_excursions", &why))
            .collect(),
        Ok((j, chi_squares)) => STATES
            .iter()
            .zip(chi_squares)
            .map(|(&x, chi_sq)| {
                let p_value = chi2_pvalue(chi_sq, 5);
                TestResult::with_note(
                    "nist::random_excursions",
                    p_value,
                    format!("x={x}, J={j}, χ²={chi_sq:.4}"),
                )
            })
            .collect(),
    }
}

/// The number of cycles J and the χ² of each state in `STATES`, in order, or
/// the reason to skip when J < max(0.005·√n, 500).
fn state_chi_squares(bits: &[u8]) -> Result<(usize, [f64; 8]), String> {
    // Build the random walk partial sums S' = 0, S₁, …, Sₙ, 0.  Append the
    // closing zero only when Sₙ ≠ 0: if the walk already ends at zero, an
    // unconditional append would create a spurious empty cycle, inflating J
    // and every state's ν₀ by 1 relative to the STS reference.
    let walk: Vec<i32> = {
        let mut s = 0i32;
        let mut w = vec![0i32];
        for &b in bits {
            s += if b == 1 { 1 } else { -1 };
            w.push(s);
        }
        if s != 0 {
            w.push(0);
        }
        w
    };

    // Count cycles: runs between consecutive zeros in the walk.
    // A "cycle" is the subsequence between two successive zero-crossings.
    let zero_positions: Vec<usize> = walk
        .iter()
        .enumerate()
        .filter(|(_, &v)| v == 0)
        .map(|(i, _)| i)
        .collect();

    let j = zero_positions.len() - 1; // number of complete cycles

    // §2.14.4 step 3 (as in sts): J must be at least max(0.005·√n, 500).
    let j_min = (0.005 * (bits.len() as f64).sqrt()).max(500.0);
    if (j as f64) < j_min {
        return Err(format!("J={j} < {j_min:.0}"));
    }

    // For each state x, count how many cycles visit x exactly k times, k=0..=5.
    let mut chi_squares = [0.0; 8];
    for (chi_sq, &x) in chi_squares.iter_mut().zip(&STATES) {
        let mut nu = [0usize; 6]; // nu[k] = cycles with exactly k visits to x (k ≥ 5 → nu[5])
        for cycle_idx in 0..j {
            let start = zero_positions[cycle_idx];
            let end = zero_positions[cycle_idx + 1];
            let visits = walk[start + 1..=end].iter().filter(|&&v| v == x).count();
            nu[visits.min(5)] += 1;
        }
        *chi_sq = chi_sq_for_state(x, &nu, j);
    }
    Ok((j, chi_squares))
}

/// Theoretical probability π_k(x) for exactly k visits to state x in a cycle.
///
/// The formulas of SP 800-22 §3.14, which also prints them to four decimals
/// for x = 1..7.  STS 2.1.2's `randomExcursions.c` tabulates |x| = 1..4 to
/// ten digits, and these formulas reproduce that table.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nist::test_vectors::e_bits;

    /// A walk with J = 1 still yields one skipped entry per state, and the
    /// Bonferroni wrapper passes the skip through.
    #[test]
    fn short_walk_yields_one_skipped_entry_per_state() {
        let bits = [1u8; 1000];
        let results = random_excursions_all(&bits);
        assert_eq!(results.len(), STATES.len());
        for r in &results {
            assert_eq!(r.name, "nist::random_excursions");
            assert!(r.skipped(), "{r}");
        }
        let family = random_excursions(&bits);
        assert_eq!(family.name, "nist::random_excursions");
        assert!(family.skipped(), "{family}");
    }

    /// Alternating bits close a cycle every two steps: n = 1000 gives
    /// J = 500, the smallest J that runs, and n = 998 gives J = 499.
    #[test]
    fn cycle_count_gate_keeps_eight_entries() {
        let alternating: Vec<u8> = (0..1000).map(|i| (i % 2) as u8).collect();
        let at = random_excursions_all(&alternating);
        let below = random_excursions_all(&alternating[..998]);
        assert_eq!(at.len(), 8);
        assert_eq!(below.len(), 8);
        assert!(at.iter().all(|r| !r.skipped()));
        assert!(below.iter().all(TestResult::skipped));
    }

    /// STS 2.1.2's χ² and P-value for x = −4, …, −1, +1, …, +4 on 10⁶ bits of e,
    /// where J = 1490.  The x < 0 rows are also the ones SP 800-22 §2.14.8
    /// prints, and Appendix B prints the x = +1 P-value.
    const STS_ON_E: [(f64, f64); 8] = [
        (3.835698, 0.573306),
        (7.318707, 0.197996),
        (7.861927, 0.164011),
        (15.692617, 0.007779),
        (2.430872, 0.786868),
        (4.798906, 0.440912),
        (2.357041, 0.797854),
        (2.488767, 0.778186),
    ];

    /// On 10⁶ bits of e this module's J, eight χ² and eight P-values are those
    /// STS 2.1.2's `randomExcursions.c` prints for the same bits.
    #[test]
    fn matches_sts_on_e() {
        let e = e_bits(1_000_000);
        let Ok((j, chi_squares)) = state_chi_squares(&e) else {
            panic!("the walk on e has enough cycles");
        };
        assert_eq!(j, 1490);
        let results = random_excursions_all(&e);
        for ((r, chi_sq), (sts_chi_sq, sts_p)) in results.iter().zip(chi_squares).zip(STS_ON_E) {
            assert!((chi_sq - sts_chi_sq).abs() < 1e-6, "{r}: χ² = {chi_sq}");
            assert!((r.p_value - sts_p).abs() < 1e-6, "{r}");
            assert!(r.note.as_deref().unwrap().contains("J=1490"), "{r}");
        }
    }

    /// Explains SP 800-22 §2.14.8's rows for x = +1, …, +4, which print
    /// χ² = 2.485906, 5.429381, 2.404171 and 2.393928 (P-values 0.778616,
    /// 0.365752, 0.790853 and 0.792378) where this module and STS give the
    /// values in `STS_ON_E`.  The walk on e returns to zero for the last time
    /// at step 991 028, after 1 489 cycles, and ends at S_n = +58.  Counting
    /// that final excursion toward J but dropping its visits reproduces the
    /// printed rows; the negative states cannot tell, because the excursion
    /// stays above zero.  The module is not changed to match them.
    #[test]
    fn explains_section_2_14_8_rows_for_positive_states() {
        const PRINTED_POSITIVE: [(f64, f64); 4] = [
            (2.485906, 0.778616),
            (5.429381, 0.365752),
            (2.404171, 0.790853),
            (2.393928, 0.792378),
        ];
        let e = e_bits(1_000_000);

        // Visit counts for x = +1, …, +4 over the cycles that close.
        let mut nu = [[0usize; 6]; 4];
        let mut visits = [0usize; 4];
        let (mut s, mut closed, mut last_return) = (0i32, 0usize, 0usize);
        for (step, &b) in (1..).zip(&e) {
            s += if b == 1 { 1 } else { -1 };
            if (1..=4).contains(&s) {
                visits[s as usize - 1] += 1;
            } else if s == 0 {
                for (row, v) in nu.iter_mut().zip(&mut visits) {
                    row[(*v).min(5)] += 1;
                    *v = 0;
                }
                closed += 1;
                last_return = step;
            }
        }
        assert_eq!((closed, last_return, s), (1489, 991_028, 58));
        for ((x, row), (chi_sq, p)) in (1..=4).zip(&mut nu).zip(PRINTED_POSITIVE) {
            row[0] += 1; // the final excursion, scored as visiting no state
            let got = chi_sq_for_state(x, row, closed + 1);
            assert!((got - chi_sq).abs() < 1e-6, "x = {x}: χ² = {got}");
            assert!((chi2_pvalue(got, 5) - p).abs() < 1e-6, "x = {x}");
        }
    }
}
