//! DIEHARDER test 200 — rgb_bitdist.
//!
//! Faithful core port of Brown's `rgb_bitdist.c`:
//! - consume a continuous MSB-first bitstream from 32-bit words
//! - partition the stream into `tsamples` blocks of `bsamples = 64` consecutive
//!   `n`-bit values
//! - for each pattern value, build the histogram of how often that pattern
//!   occurs 0..64 times inside a block
//! - compare that histogram to the exact binomial expectation with the same
//!   Vtest tail-bundling rule used by Dieharder
//!
//! Brown's original runner stores one randomly chosen pattern p-value per
//! width because the dieharder harness only has a single p-value slot per run.
//! This Rust crate instead emits every per-pattern p-value explicitly, which is
//! more transparent and avoids hiding failures behind a random pick.

use crate::{
    math::{chi2_pvalue, lgamma},
    result::TestResult,
};

const BSAMPLES: usize = 64;
const VTEST_CUTOFF: f64 = 20.0;

fn binomial_pmf(n: usize, k: usize, p: f64) -> f64 {
    if p <= 0.0 {
        return if k == 0 { 1.0 } else { 0.0 };
    }
    if p >= 1.0 {
        return if k == n { 1.0 } else { 0.0 };
    }
    let q = 1.0 - p;
    let log_comb =
        lgamma((n + 1) as f64) - lgamma((k + 1) as f64) - lgamma((n - k + 1) as f64);
    (log_comb + (k as f64) * p.ln() + ((n - k) as f64) * q.ln()).exp()
}

fn next_n_bits_msb(words: &[u32], bit_cursor: &mut usize, nbits: usize) -> Option<u32> {
    let total_bits = words.len() * 32;
    if *bit_cursor + nbits > total_bits {
        return None;
    }
    let mut value = 0u32;
    for _ in 0..nbits {
        let idx = *bit_cursor / 32;
        let offset = *bit_cursor % 32;
        let bit = (words[idx] >> (31 - offset)) & 1;
        value = (value << 1) | bit;
        *bit_cursor += 1;
    }
    Some(value)
}

fn vtest_pvalue(observed: &[f64], expected: &[f64], cutoff: f64) -> Option<(f64, usize, f64)> {
    if observed.len() != expected.len() || observed.is_empty() {
        return None;
    }

    let mut chisq = 0.0;
    let mut ndof_terms = 0usize;
    let mut tail_index: Option<usize> = None;
    let mut tail_obs = 0.0;
    let mut tail_exp = 0.0;

    for i in 0..observed.len() {
        let obs = observed[i];
        let exp = expected[i];
        if exp >= cutoff {
            let diff = obs - exp;
            chisq += diff * diff / exp;
            ndof_terms += 1;
        } else if tail_index.is_none() {
            tail_index = Some(i);
            tail_obs += obs;
            tail_exp += exp;
        } else {
            tail_obs += obs;
            tail_exp += exp;
        }
    }

    if tail_index.is_some() && tail_exp >= cutoff {
        let diff = tail_obs - tail_exp;
        chisq += diff * diff / tail_exp;
        ndof_terms += 1;
    }

    if ndof_terms <= 1 {
        return None;
    }
    let df = ndof_terms - 1;
    Some((chi2_pvalue(chisq, df), df, chisq))
}

fn pattern_results(words: &[u32], n: usize) -> Option<Vec<TestResult>> {
    if !(1..=20).contains(&n) {
        return None;
    }

    let value_max = 1usize << n;
    let total_nbit_values = (words.len() * 32) / n;
    let tsamples = total_nbit_values / BSAMPLES;
    if tsamples == 0 {
        return None;
    }

    let ntuple_prob = 1.0 / value_max as f64;
    let expected_hist: Vec<f64> = (0..=BSAMPLES)
        .map(|b| tsamples as f64 * binomial_pmf(BSAMPLES, b, ntuple_prob))
        .collect();

    let mut histograms = vec![vec![0f64; BSAMPLES + 1]; value_max];
    let mut count = vec![0usize; value_max];
    let mut cursor = 0usize;

    for _ in 0..tsamples {
        count.fill(0);
        for _ in 0..BSAMPLES {
            let value = next_n_bits_msb(words, &mut cursor, n)? as usize;
            count[value] += 1;
        }
        for pattern in 0..value_max {
            histograms[pattern][count[pattern]] += 1.0;
        }
    }

    let mut results = Vec::with_capacity(value_max);
    for pattern in 0..value_max {
        if let Some((p, df, chi_sq)) = vtest_pvalue(&histograms[pattern], &expected_hist, VTEST_CUTOFF)
        {
            results.push(TestResult::with_note(
                "dieharder::bit_distribution",
                p,
                format!("width={n}, pattern={pattern}, tsamples={tsamples}, bsamples={BSAMPLES}, df={df}, χ²={chi_sq:.4}"),
            ));
        }
    }
    Some(results)
}

/// Convenience collapse retained for ad hoc callers.
pub fn bit_distribution(words: &[u32], max_bits: usize) -> TestResult {
    let results = bit_distribution_all(words, max_bits);
    if results.is_empty() {
        return TestResult::insufficient("dieharder::bit_distribution", "not enough data");
    }
    let worst = results
        .iter()
        .min_by(|a, b| a.p_value.partial_cmp(&b.p_value).unwrap())
        .unwrap();
    TestResult::with_note(
        "dieharder::bit_distribution",
        worst.p_value,
        worst.note.clone().unwrap_or_default(),
    )
}

/// Emit the full per-width, per-pattern `rgb_bitdist` family.
pub fn bit_distribution_all(words: &[u32], max_bits: usize) -> Vec<TestResult> {
    let mut results = Vec::new();
    for n in 1..=max_bits.min(20) {
        if let Some(mut family) = pattern_results(words, n) {
            results.append(&mut family);
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::{bit_distribution_all, next_n_bits_msb};

    #[test]
    fn extracts_bits_msb_first_across_word_boundaries() {
        let words = [0xDEAD_BEEF, 0x0123_4567];
        let mut cursor = 0usize;
        assert_eq!(Some(0b1101), next_n_bits_msb(&words, &mut cursor, 4));
        assert_eq!(Some(0b1110), next_n_bits_msb(&words, &mut cursor, 4));
        cursor = 28;
        assert_eq!(Some(0b1111_0000), next_n_bits_msb(&words, &mut cursor, 8));
    }

    #[test]
    fn constant_stream_produces_tiny_pattern_pvalues() {
        let words = vec![0u32; 4096];
        let results = bit_distribution_all(&words, 2);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.p_value < 1e-6));
    }
}
