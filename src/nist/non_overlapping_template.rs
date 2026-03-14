//! NIST SP 800-22 §2.7 — Non-overlapping Template Matching Test.
//!
//! Counts non-overlapping occurrences of a fixed aperiodic m-bit template in
//! each of N blocks of M bits, then tests the counts against a normal
//! approximation.
//!
//! Default: m = 9 (the first of the 148 aperiodic 9-bit templates).
//! Minimum recommended: n ≥ 10^6 for reliable results with m = 9.

use crate::{math::igamc, result::TestResult};

/// First aperiodic template of length 9 from Appendix E of SP 800-22.
const TEMPLATE_9: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 1];

/// Run the non-overlapping template matching test with an m-bit template.
///
/// Uses the first aperiodic template of the given length.  Callers that want
/// to iterate over all 148 templates can call [`non_overlapping_template_raw`]
/// with each template directly.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.7.
pub fn non_overlapping_template(bits: &[u8], m: usize) -> TestResult {
    let template = if m == 9 { TEMPLATE_9 } else { &TEMPLATE_9[..m.min(9)] };
    non_overlapping_template_raw(bits, template)
}

/// Raw entry point: caller supplies the template explicitly.
pub fn non_overlapping_template_raw(bits: &[u8], template: &[u8]) -> TestResult {
    let n = bits.len();
    let m = template.len();
    let big_m = 8 * m; // block size recommended by SP 800-22
    let num_blocks = n / big_m;

    if num_blocks < 8 {
        return TestResult::insufficient(
            "nist::non_overlapping_template",
            "n too small — need ≥ 8 blocks",
        );
    }

    // μ and σ² for the count of non-overlapping occurrences in one block.
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

    TestResult::with_note(
        "nist::non_overlapping_template",
        p_value,
        format!("n={n}, m={m}, M={big_m}, N={num_blocks}, χ²={chi_sq:.4}"),
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
