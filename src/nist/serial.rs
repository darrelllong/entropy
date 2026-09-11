//! NIST SP 800-22 §2.11 — Serial Test.
//!
//! Counts overlapping m-bit, (m−1)-bit, and (m−2)-bit patterns in the
//! bit sequence (treating it as circular) and computes the ψ² statistic.
//! Two p-values are returned (for ∇ψ² and ∇²ψ², in that order, matching
//! the publication's P-value1/P-value2); the test passes if both are ≥ α.
//!
//! Recommended defaults: m = 3, n ≥ 1 000 000.  SP 800-22 §2.11.7 asks for
//! m < ⌊log₂ n⌋ − 2, which this module enforces exactly.

use crate::{math::chi2_pvalue, result::TestResult};

/// Run the serial test and return one result: whichever of the two
/// [`serial_both`] entries has the smaller p-value, unchanged.
///
/// The entry keeps its own name and note: `nist::serial_delta1` for ∇ψ²_m or
/// `nist::serial_delta2` for ∇²ψ²_m, with a tie going to ∇ψ²_m.  The
/// publication's test passes only if both p-values are ≥ α, which is the
/// verdict this entry's pass/fail check gives.  A skipped entry is returned
/// only when both are skipped (the preconditions failed), and then it is the
/// `nist::serial_delta2` one.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.11.
pub fn serial(bits: &[u8], m: usize) -> TestResult {
    let results = serial_both(bits, m);
    // serial_both always returns exactly two results; pick the one with the
    // smaller p-value (most conservative) as the single reported verdict.
    let r1 = results[0].clone();
    let r2 = results[1].clone();
    // A skipped (NaN) entry never masks a scored one.
    match (r1.skipped(), r2.skipped()) {
        (false, true) => r1,
        (true, false) => r2,
        _ if r1.p_value <= r2.p_value => r1,
        _ => r2,
    }
}

/// Run the serial test; returns the two p-values as separate `TestResult`s.
///
/// This is the statistically correct way to report the serial test.
/// Taking min(p1, p2) — as done by `serial` — inflates false-failure rates.
pub fn serial_both(bits: &[u8], m: usize) -> Vec<TestResult> {
    let n = bits.len();
    // The n ≥ 1000 floor is a project choice, not an SP 800-22 rule (§2.11.7
    // bounds only m); the battery runs at millions of bits, so it never
    // applies there.  m ≥ 2 keeps the unsigned m − 2 below valid.
    if n < 1_000 || m < 2 {
        return vec![
            TestResult::insufficient("nist::serial_delta1", "n < 1000 or m < 2"),
            TestResult::insufficient("nist::serial_delta2", "n < 1000 or m < 2"),
        ];
    }
    // §2.11.7: "Choose m and n such that m < ⌊log2 n⌋ − 2".
    if m >= (n.ilog2() as usize).saturating_sub(2) {
        let why = format!("m={m} violates m < ⌊log₂ n⌋ − 2 (n={n})");
        return vec![
            TestResult::insufficient("nist::serial_delta1", &why),
            TestResult::insufficient("nist::serial_delta2", &why),
        ];
    }

    let [(del1, p1), (del2, p2)] = statistics(bits, m);

    vec![
        TestResult::with_note(
            "nist::serial_delta1",
            p1,
            format!("n={n}, m={m}, ∇ψ²={del1:.4}"),
        ),
        TestResult::with_note(
            "nist::serial_delta2",
            p2,
            format!("n={n}, m={m}, ∇²ψ²={del2:.4}"),
        ),
    ]
}

/// ∇ψ²ₘ and ∇²ψ²ₘ of `bits`, each with its P-value, without the gates of
/// [`serial_both`] (m ≥ 2 is still required).
fn statistics(bits: &[u8], m: usize) -> [(f64, f64); 2] {
    let n = bits.len();
    let psi_m = psi_sq(bits, m, n);
    let psi_m1 = psi_sq(bits, m - 1, n);
    let psi_m2 = psi_sq(bits, m - 2, n);

    // ∇ψ² is a sum of squares and ∇²ψ² is non-negative too, but the
    // subtractions can leave rounding residue just below zero, where igamc
    // returns NaN and a scored entry would be reported as skipped.
    let del1 = (psi_m - psi_m1).max(0.0);
    let del2 = (psi_m - 2.0 * psi_m1 + psi_m2).max(0.0);

    // §2.11.4 step 5: ∇ψ² ~ χ²(2^{m−1}) and ∇²ψ² ~ χ²(2^{m−2}), which the
    // publication writes as igamc(2^{m−2}, ∇ψ²/2) and igamc(2^{m−3}, ∇²ψ²/2).
    [
        (del1, chi2_pvalue(del1, 1 << (m - 1))),
        (del2, chi2_pvalue(del2, 1 << (m - 2))),
    ]
}

/// Compute the ψ² statistic for patterns of length `l` in the circular
/// (wrap-around) sequence of length `n`.
fn psi_sq(bits: &[u8], l: usize, n: usize) -> f64 {
    if l == 0 {
        return 0.0;
    }
    let table_size = 1usize << l;
    let mut counts = vec![0u32; table_size];

    for i in 0..n {
        let mut pattern = 0usize;
        for j in 0..l {
            pattern = (pattern << 1) | bits[(i + j) % n] as usize;
        }
        counts[pattern] += 1;
    }

    let sum_sq: f64 = counts.iter().map(|&c| (c as f64).powi(2)).sum();
    table_size as f64 / n as f64 * sum_sq - n as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nist::test_vectors::e_bits;
    use crate::rng::{Mt19937, Rng};

    /// SP 800-22 §2.11.6 worked example: ε = 0011011101, m = 3.
    const EXAMPLE: [u8; 10] = [0, 0, 1, 1, 0, 1, 1, 1, 0, 1];

    #[test]
    fn psi_sq_matches_nist_worked_example() {
        assert!((psi_sq(&EXAMPLE, 3, 10) - 2.8).abs() < 1e-12);
        assert!((psi_sq(&EXAMPLE, 2, 10) - 1.2).abs() < 1e-12);
        assert!((psi_sq(&EXAMPLE, 1, 10) - 0.4).abs() < 1e-12);
    }

    /// The publication's P-values for the worked example (n = 10, m = 3):
    /// P-value1 = igamc(2^{m−2}, ∇ψ²/2) = igamc(2, 0.8) = 0.808792 and
    /// P-value2 = igamc(2^{m−3}, ∇²ψ²/2) = igamc(1, 0.4) = 0.670320.  The
    /// example is below `serial_both`'s n ≥ 1000 floor, so it runs through
    /// `statistics`, the code that pairs each statistic with its degrees of
    /// freedom for `serial_both`.  Swapping the pairing gives 0.449329 and
    /// 0.938448, so this pins the pairing that was once cross-wired.
    #[test]
    fn p_value_pairing_matches_nist_worked_example() {
        let [(del1, p1), (del2, p2)] = statistics(&EXAMPLE, 3);
        assert!((del1 - 1.6).abs() < 1e-12, "∇ψ² = {del1}");
        assert!((del2 - 0.8).abs() < 1e-12, "∇²ψ² = {del2}");
        assert!((p1 - 0.808792).abs() < 1e-6, "p1 = {p1}");
        assert!((p2 - 0.670320).abs() < 1e-6, "p2 = {p2}");
    }

    /// SP 800-22 §2.11.8 on 10⁶ bits of e with m = 2: ψ²₂ = 0.343128,
    /// ψ²₁ = 0.003364 and ψ²₀ = 0, so ∇ψ²₂ = 0.339764 and ∇²ψ²₂ = 0.336400,
    /// with P-value1 = 0.843764 and P-value2 = 0.561915.
    #[test]
    fn matches_section_2_11_8_example() {
        let e = e_bits(1_000_000);
        let (psi_2, psi_1) = (psi_sq(&e, 2, e.len()), psi_sq(&e, 1, e.len()));
        assert!((psi_2 - 0.343128).abs() < 1e-6, "ψ²₂ = {psi_2}");
        assert!((psi_1 - 0.003364).abs() < 1e-6, "ψ²₁ = {psi_1}");
        assert!((psi_2 - psi_1 - 0.339764).abs() < 1e-6);
        assert!((psi_2 - 2.0 * psi_1 - 0.336400).abs() < 1e-6);
        let both = serial_both(&e, 2);
        assert!((both[0].p_value - 0.843764).abs() < 1e-6, "{}", both[0]);
        assert!((both[1].p_value - 0.561915).abs() < 1e-6, "{}", both[1]);
    }

    #[test]
    fn structured_input_fails_and_m_bound_enforced() {
        // Perfectly alternating bits: wildly non-uniform pattern counts.
        let bits: Vec<u8> = (0..2048).map(|i| (i % 2) as u8).collect();
        for r in serial_both(&bits, 3) {
            assert!(r.p_value < 0.01, "{r}");
        }
        // m too large for n → insufficient, not a bogus p-value.
        for r in serial_both(&bits, 11) {
            assert!(r.skipped(), "{r}");
        }
    }

    /// `serial` hands back one `serial_both` entry, name and note included:
    /// the one with the smaller p-value, or the ∇²ψ² entry when both skip.
    #[test]
    fn serial_returns_one_serial_both_entry_unchanged() {
        let bits = Mt19937::new(5489).collect_bits(4096);
        let both = serial_both(&bits, 3);
        assert!(!both[0].skipped() && !both[1].skipped());
        let smaller = if both[0].p_value <= both[1].p_value {
            &both[0]
        } else {
            &both[1]
        };
        let one = serial(&bits, 3);
        assert_eq!(one.name, smaller.name);
        assert_eq!(one.p_value.to_bits(), smaller.p_value.to_bits());
        assert_eq!(one.note, smaller.note);

        let skipped = serial(&bits[..999], 3);
        assert_eq!(skipped.name, "nist::serial_delta2");
        assert!(skipped.skipped(), "{skipped}");
    }

    /// Regression: rounding could leave ∇²ψ² a hair below zero, where
    /// `igamc` returns NaN, so the entry skipped on valid input and
    /// `serial()` returned that skip even when ∇ψ² failed.  This 12-bit
    /// periodic pattern makes ∇²ψ² exactly 0, so P-value2 must be 1; MT19937
    /// seed 37 at n = 1040 hit the same skip.
    #[test]
    fn rounding_below_zero_does_not_skip_a_scored_entry() {
        let pattern: Vec<u8> = (0..12).map(|i| ((0b101001u32 >> i) & 1) as u8).collect();
        let bits: Vec<u8> = pattern.iter().cycle().take(1000).copied().collect();
        let both = serial_both(&bits, 2);
        assert!(!both[1].skipped(), "{}", both[1]);
        assert!((both[1].p_value - 1.0).abs() < 1e-12, "{}", both[1]);
        assert!(!both[0].passed(), "{}", both[0]);
        let one = serial(&bits, 2);
        assert!(!one.skipped() && !one.passed(), "{one}");

        use crate::rng::{Mt19937, Rng};
        let bits = Mt19937::new(37).collect_bits(1040);
        let both = serial_both(&bits, 2);
        assert!(
            both.iter().all(|r| !r.skipped()),
            "{} | {}",
            both[0],
            both[1]
        );
    }
}
