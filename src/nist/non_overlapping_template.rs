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
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.7.
pub fn non_overlapping_template(bits: &[u8], m: usize) -> TestResult {
    // Use 000000001 (first aperiodic template of length m) as a single probe.
    let mut template = vec![0u8; m];
    template[m - 1] = 1;
    non_overlapping_template_raw(bits, &template)
}

/// Raw entry point: caller supplies the template explicitly.
pub fn non_overlapping_template_raw(bits: &[u8], template: &[u8]) -> TestResult {
    let n = bits.len();
    let m = template.len();
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
    use super::aperiodic_templates_9;

    #[test]
    fn aperiodic_9bit_template_count() {
        assert_eq!(aperiodic_templates_9().len(), 148);
    }
}
