//! NIST SP 800-22 §2.7 — Non-overlapping Template Matching Test.
//!
//! Counts non-overlapping occurrences of a fixed aperiodic m-bit template in
//! each of N blocks of M bits, then tests the counts against a normal
//! approximation.
//!
//! The published test runs all 148 aperiodic 9-bit templates from Appendix E
//! of SP 800-22, yielding 148 p-values.  Use [`non_overlapping_all`] to obtain
//! all 148 results (the canonical form), or [`non_overlapping_template_raw`]
//! to test a single caller-supplied template.
//!
//! Minimum recommended: n ≥ 10^6 for reliable results with m = 9.

use crate::{math::igamc, result::TestResult};

/// Template lengths accepted by the single-template entry points.
///
/// The NIST STS ships aperiodic template files for m = 2..=21 only, and
/// §2.7.7 recommends m = 9 or 10.  The range also keeps `1u64 << m` well
/// away from its overflow at m ≥ 64 and rules out the empty template, which
/// would match at every position without advancing the scan.
const MIN_TEMPLATE_LEN: usize = 2;
const MAX_TEMPLATE_LEN: usize = 21;

/// Reason string shared by the guarded entry points.
const TEMPLATE_LEN_NOTE: &str = "template length m must be in 2..=21";

/// Return all 148 aperiodic 9-bit templates from NIST SP 800-22 Appendix E.
///
/// A template is aperiodic if it has no period p with 1 ≤ p < m such that
/// T[i] = T[i+p] for all i in 0..m-p.  This generates the same set as
/// Appendix E of SP 800-22 Rev 1a (2010).
fn aperiodic_templates_9() -> Vec<Vec<u8>> {
    let m = 9usize;
    (0u16..512)
        .filter(|&bits| {
            let pattern: Vec<u8> = (0..m).map(|i| ((bits >> (m - 1 - i)) & 1) as u8).collect();
            // Aperiodic: no period p in 1..m such that T[i] == T[i+p] for all i.
            !(1..m).any(|p| (0..m - p).all(|i| pattern[i] == pattern[i + p]))
        })
        .map(|bits| (0..m).map(|i| ((bits >> (m - 1 - i)) & 1) as u8).collect())
        .collect()
}

/// Run the non-overlapping template test for all 148 aperiodic 9-bit templates.
///
/// Returns 148 `TestResult`s, one per template, as specified in NIST SP 800-22
/// §2.7 and Appendix E.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.7, Appendix E.
pub fn non_overlapping_all(bits: &[u8]) -> Vec<TestResult> {
    aperiodic_templates_9()
        .into_iter()
        .map(|t| non_overlapping_template_raw(bits, &t))
        .collect()
}

/// Run the non-overlapping template matching test with a single m-bit template.
///
/// Returns a skipped result when `m` is outside 2..=21 (see
/// [`non_overlapping_template_raw`]).
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.7.
pub fn non_overlapping_template(bits: &[u8], m: usize) -> TestResult {
    if !(MIN_TEMPLATE_LEN..=MAX_TEMPLATE_LEN).contains(&m) {
        return TestResult::insufficient("nist::non_overlapping_template", TEMPLATE_LEN_NOTE);
    }
    // Use 000000001 (first aperiodic template of length m) as a single probe.
    let mut template = vec![0u8; m];
    template[m - 1] = 1;
    non_overlapping_template_raw(bits, &template)
}

/// Raw entry point: caller supplies the template explicitly.
///
/// Returns a skipped result when the template length is outside 2..=21 or
/// the template contains a symbol other than 0 or 1 (such a template can
/// never match, so every block count would be 0 and the test would reject
/// any stream).
pub fn non_overlapping_template_raw(bits: &[u8], template: &[u8]) -> TestResult {
    let n = bits.len();
    let m = template.len();
    if !(MIN_TEMPLATE_LEN..=MAX_TEMPLATE_LEN).contains(&m) {
        return TestResult::insufficient("nist::non_overlapping_template", TEMPLATE_LEN_NOTE);
    }
    if template.iter().any(|&b| b > 1) {
        return TestResult::insufficient(
            "nist::non_overlapping_template",
            "template must contain only 0 and 1 symbols",
        );
    }
    // SP 800-22 §2.7 suite-code setup: N = 8 blocks, M = n / N.
    let num_blocks: usize = 8;
    let big_m = n / num_blocks;

    if big_m < m + 1 {
        return TestResult::insufficient(
            "nist::non_overlapping_template",
            "n too small — need M = n/8 > m",
        );
    }

    let pow2m = (1u64 << m) as f64;
    let mu = (big_m - m + 1) as f64 / pow2m;
    let sigma2 = big_m as f64 * (1.0 / pow2m - (2 * m - 1) as f64 / (pow2m * pow2m));

    // `chunks_exact` can yield more than N blocks when n < 64 (n mod 8 ≥
    // n/8); the statistic is defined over exactly N = 8 blocks.
    let chi_sq: f64 = bits
        .chunks_exact(big_m)
        .take(num_blocks)
        .map(|block| {
            let w = count_non_overlapping(block, template);
            (w as f64 - mu).powi(2) / sigma2
        })
        .sum();

    let p_value = igamc(num_blocks as f64 / 2.0, chi_sq / 2.0);

    let tmpl_str: String = template.iter().map(|b| b.to_string()).collect();
    TestResult::with_note(
        "nist::non_overlapping_template",
        p_value,
        format!("B={tmpl_str}, N={num_blocks}, M={big_m}, χ²={chi_sq:.4}"),
    )
}

/// Count non-overlapping occurrences of `template` in `block`.
fn count_non_overlapping(block: &[u8], template: &[u8]) -> usize {
    let m = template.len();
    let mut count = 0usize;
    let mut i = 0usize;
    while i + m <= block.len() {
        if &block[i..i + m] == template {
            count += 1;
            i += m; // skip past the match — non-overlapping
        } else {
            i += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aperiodic_9bit_template_count() {
        assert_eq!(aperiodic_templates_9().len(), 148);
    }

    /// SP 800-22 §2.7.8 worked example: ε = 10100100101110010110 split
    /// into two 10-bit blocks with template B = 001 gives W₁ = 2 and
    /// W₂ = 1.  Only the block counts are checked here; the spec's χ² and
    /// p-value use N = 2 blocks, whereas this implementation fixes N = 8.
    #[test]
    fn count_matches_spec_example() {
        let block1 = [1, 0, 1, 0, 0, 1, 0, 0, 1, 0];
        let block2 = [1, 1, 1, 0, 0, 1, 0, 1, 1, 0];
        let template = [0, 0, 1];
        assert_eq!(count_non_overlapping(&block1, &template), 2);
        assert_eq!(count_non_overlapping(&block2, &template), 1);
        // A match ending on the last bit of the block counts.
        assert_eq!(count_non_overlapping(&[0, 0, 0, 1], &[0, 0, 1]), 1);
        // Two disjoint matches are both counted.
        assert_eq!(count_non_overlapping(&[0, 0, 1, 0, 0, 1], &[0, 0, 1]), 2);
        // The scan resumes after a match, which matters only for a periodic
        // template: 11 sits at two overlapping offsets in 111 but counts
        // once, and counts twice (not three times) in 1111.
        assert_eq!(count_non_overlapping(&[1, 1, 1], &[1, 1]), 1);
        assert_eq!(count_non_overlapping(&[1, 1, 1, 1], &[1, 1]), 2);
    }

    /// Regression: m = 0 indexed `template[m - 1]`, m ≥ 64 overflowed
    /// `1u64 << m`, and an empty raw template never advanced the matcher.
    /// Out-of-range lengths and non-binary symbols must now return a skipped
    /// result instead of panicking, hanging, or fabricating a rejection.
    #[test]
    fn degenerate_templates_are_skipped() {
        let bits = vec![1u8; 1024];
        for m in [0usize, 1, 22, 33, 64] {
            assert!(non_overlapping_template(&bits, m).skipped(), "m = {m}");
        }
        assert!(non_overlapping_template_raw(&bits, &[]).skipped());
        assert!(non_overlapping_template_raw(&bits, &[0, 1, 2]).skipped());
        assert!(non_overlapping_template_raw(&bits, &[2, 2, 2]).skipped());
        for m in [2usize, 9, 21] {
            assert!(!non_overlapping_template(&bits, m).skipped(), "m = {m}");
        }
    }

    /// For n < 64, `chunks_exact(n / 8)` can produce more than eight blocks;
    /// the χ² must be summed over exactly N = 8 of them so that the reported
    /// degrees of freedom match.
    #[test]
    fn exactly_eight_blocks_are_scored_for_short_streams() {
        // n = 23: M = 2, and chunks_exact(2) would yield 11 blocks.
        let bits = vec![1u8; 23];
        let r = non_overlapping_template(&bits, 2);
        assert!(
            r.skipped(),
            "M = 2 cannot hold an m = 2 template plus one bit"
        );
        // n = 31: M = 3, chunks_exact(3) yields 10 blocks; every block is
        // 111, so every W = 0 and χ² = 8·µ²/σ² exactly.
        let bits = vec![1u8; 31];
        let r = non_overlapping_template(&bits, 2);
        assert!(!r.skipped());
        let mu = (3.0 - 2.0 + 1.0) / 4.0;
        let sigma2 = 3.0 * (1.0 / 4.0 - 3.0 / 16.0);
        let expected = 8.0 * mu * mu / sigma2;
        assert!(r.to_string().contains(&format!("χ²={expected:.4}")), "{r}");
    }
}
