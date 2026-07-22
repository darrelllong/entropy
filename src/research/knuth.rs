//! Classical Knuth-style tests from TAOCP Vol. 2, §3.3.2.
//!
//! These are implemented over a uniform `[0, 1)` sample stream:
//! - permutation test over non-overlapping windows of size `t`
//! - gap test for a target interval `[alpha, beta)`
//! - Wald-Wolfowitz runs test above/below the sample median
//!
//! The formulas used here are the standard chi-square and conditional
//! runs-test moments for binary classifications.
//!
//! # References
//! * D. E. Knuth, *The Art of Computer Programming, Volume 2: Seminumerical
//!   Algorithms*, 3rd edition, Addison-Wesley, 1997. §3.3.2.
//!   [Permutation test, gap test]
//! * A. Wald and J. Wolfowitz, "On a test whether two samples are from the
//!   same population," *Annals of Mathematical Statistics* 11(2),
//!   pp. 147–162, 1940.  [Conditional moments of the runs-above/below-median
//!   statistic used by `runs_above_below_median_test`]

use crate::{
    math::{chi2_pvalue, erfc},
    result::TestResult,
};
use std::f64::consts::SQRT_2;

/// Summary statistics computed by [`permutation_stats`].
#[derive(Debug, Clone)]
pub struct PermutationStats {
    /// Window size `t`: each non-overlapping block of `t` samples is ranked.
    pub window: usize,
    /// Number of complete non-overlapping blocks examined.
    pub blocks: usize,
    /// Chi-square statistic over the `t!` ordering buckets (`df = t! − 1`).
    pub chi_square: f64,
}

/// Summary statistics computed by [`gap_stats`].
#[derive(Debug, Clone)]
pub struct GapStats {
    /// Lower bound of the target interval (inclusive).
    pub alpha: f64,
    /// Upper bound of the target interval (exclusive).
    pub beta: f64,
    /// Maximum tracked gap length; longer gaps pool into the tail cell.
    pub max_gap: usize,
    /// Number of completed gaps observed (hits after the first hit).
    pub gaps: usize,
    /// Number of chi-square cells after Cochran-rule tail merging
    /// (`df = cells − 1`); at most `max_gap + 1`.
    pub cells: usize,
    /// Chi-square statistic over the merged gap-length cells.
    pub chi_square: f64,
}

/// Summary statistics computed by [`runs_above_below_median_stats`].
#[derive(Debug, Clone)]
pub struct RunsMedianStats {
    /// Sample median; values equal to it are discarded before counting runs.
    pub median: f64,
    /// Count of samples strictly below the median.
    pub below: usize,
    /// Count of samples strictly above the median.
    pub above: usize,
    /// Number of runs (maximal same-side stretches) observed.
    pub runs: usize,
    /// Normal-approximation z-score of the run count (Wald–Wolfowitz moments).
    pub z_score: f64,
}

fn factorial(n: usize) -> usize {
    (1..=n).product::<usize>().max(1)
}

fn permutation_rank(window: &[f64]) -> usize {
    let t = window.len();
    let mut rank = 0usize;
    for i in 0..t {
        let mut less = 0usize;
        for j in (i + 1)..t {
            if window[j] < window[i] {
                less += 1;
            }
        }
        rank = rank * (t - i) + less;
    }
    rank
}

/// Knuth TAOCP §3.3.2 permutation-test statistics: rank each non-overlapping
/// window of `t` samples and chi-square the counts over the `t!` orderings.
///
/// Returns `None` when `t` is outside `2..=8` or there are fewer than
/// `5·t!` complete blocks (Cochran's rule).
pub fn permutation_stats(samples: &[f64], t: usize) -> Option<PermutationStats> {
    if !(2..=8).contains(&t) {
        return None;
    }
    let buckets = factorial(t);
    let blocks = samples.len() / t;
    // Cochran's rule: the chi-square approximation needs expected counts of
    // at least 5 per cell, i.e. blocks ≥ 5·t!.  (`blocks > buckets` alone
    // permitted expected counts barely above 1.)
    if blocks < 5 * buckets {
        return None;
    }

    let mut counts = vec![0usize; buckets];
    for block in 0..blocks {
        let start = block * t;
        let rank = permutation_rank(&samples[start..start + t]);
        counts[rank] += 1;
    }

    let expected = blocks as f64 / buckets as f64;
    let chi_square = counts
        .into_iter()
        .map(|obs| {
            let diff = obs as f64 - expected;
            diff * diff / expected
        })
        .sum();

    Some(PermutationStats {
        window: t,
        blocks,
        chi_square,
    })
}

/// Knuth permutation test as a [`TestResult`] (`knuth::permutation`).
///
/// Returns an insufficient-data result (NaN p-value) when
/// [`permutation_stats`] returns `None`.
pub fn permutation_test(samples: &[f64], t: usize) -> TestResult {
    let Some(stats) = permutation_stats(samples, t) else {
        return TestResult::insufficient(
            "knuth::permutation",
            "need t in 2..=8 and at least 5·t! blocks (Cochran's rule)",
        );
    };
    let df = factorial(t) - 1;
    let p_value = chi2_pvalue(stats.chi_square, df);
    TestResult::with_note(
        "knuth::permutation",
        p_value,
        format!(
            "t={}, blocks={}, χ²={:.4}, df={}",
            stats.window, stats.blocks, stats.chi_square, df
        ),
    )
}

/// Knuth TAOCP §3.3.2 gap-test statistics: chi-square the lengths of gaps
/// between visits to the target interval `[alpha, beta)` against the
/// geometric law, with Cochran-rule tail merging.
///
/// Returns `None` on invalid bounds (`0 ≤ alpha < beta ≤ 1` required),
/// `max_gap == 0`, or too few gaps to form two Cochran-valid cells.
pub fn gap_stats(samples: &[f64], alpha: f64, beta: f64, max_gap: usize) -> Option<GapStats> {
    if !(0.0..1.0).contains(&alpha) || !(0.0..=1.0).contains(&beta) || alpha >= beta {
        return None;
    }
    if max_gap == 0 {
        return None;
    }

    let p = beta - alpha;
    let mut counts = vec![0usize; max_gap + 1];
    let mut seen_first_hit = false;
    let mut current_gap = 0usize;
    let mut gaps = 0usize;

    for &x in samples {
        let hit = alpha <= x && x < beta;
        if hit {
            if seen_first_hit {
                counts[current_gap.min(max_gap)] += 1;
                gaps += 1;
            } else {
                seen_first_hit = true;
            }
            current_gap = 0;
        } else if seen_first_hit {
            current_gap += 1;
        }
    }

    // Cell probabilities: P(gap = r) = p(1−p)^r for r < max_gap; tail pooled.
    let mut probs: Vec<f64> = (0..max_gap)
        .map(|r| p * (1.0 - p).powi(r as i32))
        .collect();
    probs.push((1.0 - p).powi(max_gap as i32));

    // Cochran's rule: every cell needs expected count ≥ 5.  The interior
    // cells p(1−p)^r decrease monotonically in r; the final pooled-tail cell
    // (1−p)^max_gap does NOT follow that ordering (it exceeds its predecessor
    // whenever p < 0.5), but it is always last, so keeping the longest valid
    // prefix and pooling everything past the cut absorbs it correctly — the
    // same prefix-merge fix applied to the R pipeline's gap test
    // (scripts/r_rng_tests.R).
    let g = gaps as f64;
    let cut = probs.iter().take_while(|&&pr| pr * g >= 5.0).count();
    if cut < probs.len() {
        let tail_prob: f64 = probs[cut..].iter().sum();
        if tail_prob * g >= 5.0 {
            probs.truncate(cut);
            probs.push(tail_prob);
        } else if cut == 0 {
            return None;
        } else {
            // Pooled tail still below 5: fold it into the last valid cell.
            probs.truncate(cut);
            *probs.last_mut().unwrap() += tail_prob;
        }
    }
    let cells = probs.len();
    if cells < 2 {
        return None;
    }
    let merged_counts: Vec<usize> = (0..cells)
        .map(|i| {
            if i + 1 < cells {
                counts[i]
            } else {
                counts[i..].iter().sum()
            }
        })
        .collect();

    let mut chi_square = 0.0;
    for (&obs, &prob) in merged_counts.iter().zip(probs.iter()) {
        let expected = g * prob;
        let diff = obs as f64 - expected;
        chi_square += diff * diff / expected;
    }

    Some(GapStats {
        alpha,
        beta,
        max_gap,
        gaps,
        cells,
        chi_square,
    })
}

/// Knuth gap test as a [`TestResult`] (`knuth::gap`).
///
/// Returns an insufficient-data result (NaN p-value) when [`gap_stats`]
/// returns `None`.
pub fn gap_test(samples: &[f64], alpha: f64, beta: f64, max_gap: usize) -> TestResult {
    let Some(stats) = gap_stats(samples, alpha, beta, max_gap) else {
        return TestResult::insufficient(
            "knuth::gap",
            "need 0 <= alpha < beta <= 1, max_gap > 0, and enough gaps for ≥2 Cochran-valid cells",
        );
    };
    let df = stats.cells - 1;
    let p_value = chi2_pvalue(stats.chi_square, df);
    TestResult::with_note(
        "knuth::gap",
        p_value,
        format!(
            "[{:.3},{:.3}) gaps={}, r={}, cells={}, χ²={:.4}, df={}",
            stats.alpha, stats.beta, stats.gaps, stats.max_gap, stats.cells, stats.chi_square, df
        ),
    )
}

/// Wald–Wolfowitz runs statistics above/below the sample median.
///
/// Values equal to the median are discarded.  Returns `None` with fewer
/// than three usable values or when either side of the median is empty.
pub fn runs_above_below_median_stats(samples: &[f64]) -> Option<RunsMedianStats> {
    if samples.len() < 3 {
        return None;
    }

    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    let median = if n.is_multiple_of(2) {
        0.5 * (sorted[n / 2 - 1] + sorted[n / 2])
    } else {
        sorted[n / 2]
    };

    let labels: Vec<bool> = samples
        .iter()
        .filter_map(|&x| {
            if x < median {
                Some(false)
            } else if x > median {
                Some(true)
            } else {
                None
            }
        })
        .collect();

    if labels.len() < 3 {
        return None;
    }

    let above = labels.iter().filter(|&&b| b).count();
    let below = labels.len() - above;
    if above == 0 || below == 0 {
        return None;
    }

    let mut runs = 1usize;
    for i in 1..labels.len() {
        if labels[i] != labels[i - 1] {
            runs += 1;
        }
    }

    let n1 = above as f64;
    let n2 = below as f64;
    let total = n1 + n2;
    let mean = 1.0 + 2.0 * n1 * n2 / total;
    let variance = 2.0 * n1 * n2 * (2.0 * n1 * n2 - total) / (total * total * (total - 1.0));
    let z_score = (runs as f64 - mean) / variance.sqrt();

    Some(RunsMedianStats {
        median,
        below,
        above,
        runs,
        z_score,
    })
}

/// Wald–Wolfowitz runs test as a [`TestResult`] (`knuth::runs_median`),
/// using the two-sided normal approximation of the run count.
///
/// Returns an insufficient-data result (NaN p-value) when
/// [`runs_above_below_median_stats`] returns `None`.
pub fn runs_above_below_median_test(samples: &[f64]) -> TestResult {
    let Some(stats) = runs_above_below_median_stats(samples) else {
        return TestResult::insufficient(
            "knuth::runs_median",
            "need at least three non-median values with both sides represented",
        );
    };
    let p_value = erfc(stats.z_score.abs() / SQRT_2);
    TestResult::with_note(
        "knuth::runs_median",
        p_value,
        format!(
            "median={:.6}, below={}, above={}, runs={}, z={:.4}",
            stats.median, stats.below, stats.above, stats.runs, stats.z_score
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{gap_stats, permutation_rank, permutation_stats, runs_above_below_median_stats};

    #[test]
    fn permutation_rank_orders_three_values_lexicographically() {
        assert_eq!(0, permutation_rank(&[0.1, 0.2, 0.3]));
        assert_eq!(1, permutation_rank(&[0.1, 0.3, 0.2]));
        assert_eq!(5, permutation_rank(&[0.3, 0.2, 0.1]));
    }

    #[test]
    fn permutation_stats_count_non_overlapping_blocks() {
        // 36 blocks of t=3 (Cochran needs ≥ 5·3! = 30): cycle all six orderings.
        let orderings: [[f64; 3]; 6] = [
            [0.1, 0.2, 0.3],
            [0.1, 0.3, 0.2],
            [0.2, 0.1, 0.3],
            [0.2, 0.3, 0.1],
            [0.3, 0.1, 0.2],
            [0.3, 0.2, 0.1],
        ];
        let samples: Vec<f64> = (0..36).flat_map(|i| orderings[i % 6]).collect();
        let stats = permutation_stats(&samples, 3).unwrap();
        assert_eq!(36, stats.blocks);
        // Perfectly balanced counts → χ² = 0.
        assert!(stats.chi_square.abs() < 1e-12);
    }

    #[test]
    fn permutation_stats_reject_sub_cochran_block_counts() {
        // 7 blocks of t=3 < 30 required — must be None, not a bogus χ².
        let samples = vec![0.1; 21];
        assert!(permutation_stats(&samples, 3).is_none());
    }

    #[test]
    fn gap_stats_ignore_prefix_before_first_hit() {
        // Two leading misses, then 41 hits separated by single misses:
        // 40 gaps of length 1; the prefix must not count.
        let mut samples = vec![0.9, 0.8];
        for _ in 0..40 {
            samples.extend_from_slice(&[0.1, 0.7]);
        }
        samples.push(0.1);
        let stats = gap_stats(&samples, 0.0, 0.2, 3).unwrap();
        assert_eq!(40, stats.gaps);
    }

    #[test]
    fn gap_stats_merge_tail_cells_per_cochran() {
        // p = 0.25, 40 gaps: expected counts 10, 7.5, 5.6, 4.2, … — cells
        // r ≥ 3 fall below 5 and must pool into one tail cell (4 cells total).
        let mut samples = vec![];
        for _ in 0..40 {
            samples.extend_from_slice(&[0.1, 0.7]);
        }
        samples.push(0.1);
        let stats = gap_stats(&samples, 0.0, 0.25, 15).unwrap();
        assert_eq!(40, stats.gaps);
        assert_eq!(4, stats.cells);
    }

    #[test]
    fn runs_median_counts_alternation() {
        let samples = vec![0.1, 0.9, 0.2, 0.8, 0.3, 0.7];
        let stats = runs_above_below_median_stats(&samples).unwrap();
        assert_eq!(6, stats.runs);
        assert_eq!(3, stats.below);
        assert_eq!(3, stats.above);
    }
}
