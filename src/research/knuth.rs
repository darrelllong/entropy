//! Classical Knuth-style tests from TAOCP Vol. 2, §3.3.2.
//!
//! These are implemented over a uniform `[0, 1)` sample stream:
//! - permutation test over non-overlapping windows of size `t`
//! - gap test for a target interval `[alpha, beta)`
//! - Wald-Wolfowitz runs test above/below the sample median
//!
//! The formulas used here are the standard chi-square and conditional
//! runs-test moments for binary classifications.

use crate::{
    math::{chi2_pvalue, erfc},
    result::TestResult,
};
use std::f64::consts::SQRT_2;

#[derive(Debug, Clone)]
pub struct PermutationStats {
    pub window: usize,
    pub blocks: usize,
    pub chi_square: f64,
}

#[derive(Debug, Clone)]
pub struct GapStats {
    pub alpha: f64,
    pub beta: f64,
    pub max_gap: usize,
    pub gaps: usize,
    pub chi_square: f64,
}

#[derive(Debug, Clone)]
pub struct RunsMedianStats {
    pub median: f64,
    pub below: usize,
    pub above: usize,
    pub runs: usize,
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
            if window[j] < window[i] || (window[j] == window[i] && j < i) {
                less += 1;
            }
        }
        rank = rank * (t - i) + less;
    }
    rank
}

pub fn permutation_stats(samples: &[f64], t: usize) -> Option<PermutationStats> {
    if !(2..=8).contains(&t) {
        return None;
    }
    let buckets = factorial(t);
    let blocks = samples.len() / t;
    if blocks <= buckets {
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

pub fn permutation_test(samples: &[f64], t: usize) -> TestResult {
    let Some(stats) = permutation_stats(samples, t) else {
        return TestResult::insufficient(
            "knuth::permutation",
            "need t in 2..=8 and more blocks than permutation classes",
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

    if gaps <= max_gap + 1 {
        return None;
    }

    let mut chi_square = 0.0;
    for (r, &obs) in counts.iter().enumerate() {
        let prob = if r < max_gap {
            p * (1.0 - p).powi(r as i32)
        } else {
            (1.0 - p).powi(max_gap as i32)
        };
        let expected = gaps as f64 * prob;
        let diff = obs as f64 - expected;
        chi_square += diff * diff / expected;
    }

    Some(GapStats {
        alpha,
        beta,
        max_gap,
        gaps,
        chi_square,
    })
}

pub fn gap_test(samples: &[f64], alpha: f64, beta: f64, max_gap: usize) -> TestResult {
    let Some(stats) = gap_stats(samples, alpha, beta, max_gap) else {
        return TestResult::insufficient(
            "knuth::gap",
            "need 0 <= alpha < beta <= 1, max_gap > 0, and enough observed gaps",
        );
    };
    let df = stats.max_gap;
    let p_value = chi2_pvalue(stats.chi_square, df);
    TestResult::with_note(
        "knuth::gap",
        p_value,
        format!(
            "[{:.3},{:.3}) gaps={}, r={}, χ²={:.4}, df={}",
            stats.alpha, stats.beta, stats.gaps, stats.max_gap, stats.chi_square, df
        ),
    )
}

pub fn runs_above_below_median_stats(samples: &[f64]) -> Option<RunsMedianStats> {
    if samples.len() < 3 {
        return None;
    }

    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    let median = if n % 2 == 0 {
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
    use super::{
        gap_stats, permutation_rank, permutation_stats, runs_above_below_median_stats,
    };

    #[test]
    fn permutation_rank_orders_three_values_lexicographically() {
        assert_eq!(0, permutation_rank(&[0.1, 0.2, 0.3]));
        assert_eq!(1, permutation_rank(&[0.1, 0.3, 0.2]));
        assert_eq!(5, permutation_rank(&[0.3, 0.2, 0.1]));
    }

    #[test]
    fn permutation_stats_count_non_overlapping_blocks() {
        let samples = vec![
            0.1, 0.2, 0.3,
            0.3, 0.2, 0.1,
            0.2, 0.1, 0.3,
            0.1, 0.3, 0.2,
            0.2, 0.3, 0.1,
            0.3, 0.1, 0.2,
            0.1, 0.2, 0.3,
        ];
        let stats = permutation_stats(&samples, 3).unwrap();
        assert_eq!(7, stats.blocks);
    }

    #[test]
    fn gap_stats_ignore_prefix_before_first_hit() {
        let samples = vec![
            0.9, 0.8, 0.1,
            0.7, 0.1,
            0.6, 0.1,
            0.5, 0.1,
            0.4, 0.1,
            0.3, 0.1,
        ];
        let stats = gap_stats(&samples, 0.0, 0.2, 3).unwrap();
        assert_eq!(5, stats.gaps);
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
