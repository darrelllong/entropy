//! DIEHARDER bit distribution test.
//!
//! The words are read as one stream of bits, most significant bit first, and
//! cut into n-bit values, which are grouped into blocks of 64.  For each of
//! the 2ⁿ possible values v, the number of times v occurs in a block is
//! Binomial(64, 2⁻ⁿ); over all blocks the histogram of that number is scored
//! with a Pearson χ² against its binomial expectation, the cells expecting
//! fewer than 20 counts pooled (see [`crate::math::chi_square_pooled`]).  Every
//! width from 1 to the requested maximum and every value of each width gives
//! its own result, so a failure is reported with its width and pattern.  The
//! results share their input and are not independent.
//!
//! # Author
//! Robert G. Brown, *Dieharder: A Random Number Test Suite* (2004–2011).

use crate::{
    math::{binomial_pmf, chi_square_pooled},
    result::TestResult,
};

const BSAMPLES: usize = 64;
const MIN_EXPECTED: f64 = 20.0;

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

fn pattern_results(words: &[u32], n: usize) -> Option<Vec<TestResult>> {
    if !(1..=20).contains(&n) {
        return None;
    }

    let value_max = 1usize << n;
    let total_nbit_values = (words.len() * 32) / n;
    let tsamples = total_nbit_values / BSAMPLES;
    // Counts are u32; a block count above u32::MAX (2³⁶ n-bit values, tens of
    // GiB of input) could overflow a cell.
    if tsamples == 0 || tsamples > u32::MAX as usize {
        return None;
    }

    let ntuple_prob = 1.0 / value_max as f64;
    let expected_hist: Vec<f64> = (0..=BSAMPLES)
        .map(|b| tsamples as f64 * binomial_pmf(BSAMPLES, b, ntuple_prob))
        .collect();
    // Pooling needs two cells that expect MIN_EXPECTED blocks, or one and a
    // pool of the rest that does; otherwise no pattern can be scored, and the
    // width is dropped before its tables are allocated.
    let strong: Vec<f64> = expected_hist
        .iter()
        .copied()
        .filter(|&e| e >= MIN_EXPECTED)
        .collect();
    let scoreable = match strong.as_slice() {
        [] => false,
        [only] => tsamples as f64 - only >= MIN_EXPECTED,
        _ => true,
    };
    if !scoreable {
        return None;
    }

    // histograms[v][c]: blocks in which pattern v occurs c ≥ 1 times.  Only
    // the at most 64 patterns a block contains are updated; the zero cell is
    // filled in afterwards as the remainder.
    let mut histograms = vec![[0u32; BSAMPLES + 1]; value_max];
    let mut count = vec![0u8; value_max];
    let mut touched = Vec::with_capacity(BSAMPLES);
    let mut cursor = 0usize;

    for _ in 0..tsamples {
        for _ in 0..BSAMPLES {
            let value = next_n_bits_msb(words, &mut cursor, n)? as usize;
            if count[value] == 0 {
                touched.push(value);
            }
            count[value] += 1;
        }
        for value in touched.drain(..) {
            histograms[value][usize::from(count[value])] += 1;
            count[value] = 0;
        }
    }
    for histogram in &mut histograms {
        let present: u32 = histogram[1..].iter().sum();
        histogram[0] = tsamples as u32 - present;
    }

    let mut results = Vec::with_capacity(value_max);
    for (pattern, histogram) in histograms.iter().enumerate().take(value_max) {
        if let Some((p, df, chi_sq)) = chi_square_pooled(histogram, &expected_hist, MIN_EXPECTED) {
            results.push(TestResult::with_note(
                "dieharder::bit_distribution",
                p,
                format!("width={n}, pattern={pattern}, tsamples={tsamples}, bsamples={BSAMPLES}, df={df}, χ²={chi_sq:.4}"),
            )
            .chi_square(chi_sq, df as f64));
        }
    }
    Some(results)
}

/// The whole family as one result.
///
/// Reports the worst per-pattern p-value with a Bonferroni correction for the
/// number of patterns examined (up to Σ 2ⁿ ≈ 510 at `max_bits = 8`).  A bare
/// minimum would reject a perfect generator with probability
/// 1 − 0.99⁵¹⁰ ≈ 99% at α = 0.01; Bonferroni stays valid under the dependence
/// between pattern counts.  Prefer [`bit_distribution_all`] (used by
/// `run_all`) for the uncollapsed family.
pub fn bit_distribution(words: &[u32], max_bits: usize) -> TestResult {
    let results = bit_distribution_all(words, max_bits);
    let scored: Vec<&TestResult> = results.iter().filter(|r| !r.skipped()).collect();
    if scored.is_empty() {
        return TestResult::insufficient("dieharder::bit_distribution", "not enough data");
    }
    let worst = scored
        .iter()
        .min_by(|a, b| a.p_value.partial_cmp(&b.p_value).unwrap())
        .unwrap();
    let m = scored.len() as f64;
    TestResult::with_note(
        "dieharder::bit_distribution",
        (m * worst.p_value).min(1.0),
        format!(
            "Bonferroni over {} patterns; worst: {}",
            scored.len(),
            worst.note.clone().unwrap_or_default()
        ),
    )
    .with_statistic(
        "smallest pattern p-value",
        worst.p_value,
        None,
        "Bonferroni bound",
    )
}

/// Every per-width, per-pattern result, widths 1 to `max_bits` (at most 20).
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

    /// The sparse update gives the histograms of a direct count.
    #[test]
    fn sparse_histograms_match_direct_counts() {
        use crate::rng::{Mt19937, Rng};
        let words = Mt19937::new(7).collect_u32s(20_000);
        for n in 1..=6 {
            let results = super::pattern_results(&words, n).expect("scoreable");
            assert_eq!(results.len(), 1 << n, "width {n}");
        }
        // Direct count for width 3, pattern 5.
        let tsamples = words.len() * 32 / 3 / 64;
        let mut hist = [0u32; 65];
        let mut cursor = 0;
        for _ in 0..tsamples {
            let c = (0..64)
                .filter(|_| next_n_bits_msb(&words, &mut cursor, 3) == Some(5))
                .count();
            hist[c] += 1;
        }
        let note = super::pattern_results(&words, 3).unwrap()[5]
            .note
            .clone()
            .unwrap();
        let expected: Vec<f64> = (0..=64)
            .map(|b| tsamples as f64 * crate::math::binomial_pmf(64, b, 1.0 / 8.0))
            .collect();
        let (_, df, chi) = crate::math::chi_square_pooled(&hist, &expected, 20.0).unwrap();
        assert!(note.ends_with(&format!("df={df}, χ²={chi:.4}")), "{note}");
    }

    /// A width with too few blocks to score is dropped without allocating.
    #[test]
    fn unscoreable_widths_are_dropped() {
        assert!(super::pattern_results(&[0u32; 64], 20).is_none());
    }

    #[test]
    fn constant_stream_produces_tiny_pattern_pvalues() {
        let words = vec![0u32; 4096];
        let results = bit_distribution_all(&words, 2);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.p_value < 1e-6));
    }
}
