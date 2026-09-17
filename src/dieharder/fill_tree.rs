//! DIEHARDER fill-tree test.
//!
//! A 32-slot array holds a binary search tree of four levels: the root in slot
//! 15 and, below each slot, children 8, 4, 2 and then 1 slots to either side.
//! Each trial inserts uniform samples until one follows a path whose four
//! slots are all occupied; that sample "collides" at one of the 16 gaps below
//! the bottom level.  Two statistics:
//!
//! 1. Fill count: the number t of samples placed before the collision,
//!    4 ≤ t ≤ 15, scored with a Pearson χ² against its exact distribution.
//! 2. Collision position: which of the 16 gaps, scored with a Pearson χ²
//!    against the uniform distribution, df 15.
//!
//! # The null distribution
//!
//! Inserting independent continuous samples builds a random binary search
//! tree.  For a subtree of h levels let D_h(t, g) be the probability that its
//! own sequence of arrivals places t samples and then collides at gap g.  The
//! first arrival takes the root with value u; later arrivals go left with
//! probability u.  If the left subtree would collide on its (a + 1)-th
//! arrival and the right one on its (b + 1)-th, the left collides first, after
//! r ≤ b right arrivals, with probability C(a + r, r)·u^(a+1)·(1 − u)^r, and
//! integrating over u gives (a + 1)/((a + r + 1)(a + r + 2)).  Hence
//!
//! D_h(1 + a + r, g) = Σ D_{h−1}(a, g) · P(T_{h−1} ≥ r) · (a + 1)/((a + r + 1)(a + r + 2)),
//!
//! with the mirror-image term for the right subtree and D_0(0, 0) = 1.  At
//! h = 4 this gives P(t = 4) = 2/15, P(5) = 1/5, P(6) = 13/63, …,
//! P(15) = 1/59 535, and every gap has probability exactly 1/16.
//!
//! Fill-count cells are pooled from the tail inward until each expects at
//! least 5 trials, so every trial is counted once and df is one less than the
//! number of cells.
//!
//! Samples are words scaled to [0, 1); every 25 000 trials the words are
//! rotated left by one more bit, so the comparisons read different bits.
//!
//! # Author
//! David Bauer, in Robert G. Brown's *Dieharder: A Random Number Test Suite*
//! (2006).

use crate::{
    math::{chi_square_pooled_tails, igamc},
    result::TestResult,
};
use std::sync::OnceLock;

/// Slots in the tree array.
const SIZE: usize = 32;
/// Levels of the tree.
const LEVELS: usize = 4;
/// Gaps below the bottom level, where a sample collides.
const GAPS: usize = 1 << LEVELS;
/// Largest fill count: every slot of the four levels occupied.
const MAX_FILL: usize = GAPS - 1;
/// Number of trials.
const N_TRIALS: usize = 100_000;
/// Number of rotation cycles.
const CYCLES: usize = 4;
/// The root slot.
const START_VAL: usize = SIZE / 2 - 1; // = 15
/// Smallest expected count for a fill-count cell.
const MIN_EXPECTED: f64 = 5.0;

/// Run the fill-tree test, returning the fill-count and the position results.
///
/// # Author
/// David Bauer, Dieharder (2006).
pub fn fill_tree_both(words: &[u32]) -> Vec<TestResult> {
    // Cheap upfront floor: mean consumption is ≈ 7.5 words/trial, so demand
    // 8·N_TRIALS.  (The worst case is 16 words per trial, see `tree_insert`,
    // but requiring that — 1.6 M words — would reject streams that virtually
    // always suffice; the out-of-words check below still handles a stream
    // that truly runs dry.)
    if words.len() < N_TRIALS * 8 {
        return vec![
            TestResult::insufficient("dieharder::fill_tree_count", "not enough words"),
            TestResult::insufficient("dieharder::fill_tree_position", "not enough words"),
        ];
    }

    let mut fill_counts = [0u32; MAX_FILL + 1];
    let mut position_counts = [0u32; GAPS];

    let mut word_idx = 0usize;

    let mut rot_amount = 0u32;
    for j in 0..N_TRIALS {
        // Empty slots are NaN, which no sample in [0, 1) equals.
        let mut array = [f64::NAN; SIZE];
        let mut word_count = 0usize;

        let fail_pos = loop {
            if word_idx >= words.len() {
                return vec![
                    TestResult::insufficient("dieharder::fill_tree_count", "ran out of words"),
                    TestResult::insufficient("dieharder::fill_tree_position", "ran out of words"),
                ];
            }
            let v = words[word_idx];
            word_idx += 1;
            word_count += 1;

            // Rotate and scale to [0, 1); the tree only compares samples, so
            // the scale does not matter.
            let rotated = if rot_amount == 0 {
                v
            } else {
                v.rotate_left(rot_amount)
            };
            let x = rotated as f64 / 4_294_967_296.0;

            if let Some(pos) = tree_insert(x, &mut array) {
                break pos;
            }
        };

        fill_counts[word_count - 1] += 1;
        position_counts[fail_pos / 2] += 1;

        if j % (N_TRIALS / CYCLES) == 0 {
            rot_amount = (rot_amount + 1) % 32;
        }
    }

    let (chi_fill, df_fill, cells) = fill_count_chi_square(&fill_counts);
    let p_fill = igamc(df_fill as f64 / 2.0, chi_fill / 2.0);

    let expected_pos = N_TRIALS as f64 / GAPS as f64;
    let chi_pos: f64 = position_counts
        .iter()
        .map(|&c| (c as f64 - expected_pos).powi(2) / expected_pos)
        .sum();
    let df_pos = GAPS - 1;
    let p_pos = igamc(df_pos as f64 / 2.0, chi_pos / 2.0);

    vec![
        TestResult::with_note(
            "dieharder::fill_tree_count",
            p_fill,
            format!("trials={N_TRIALS}, cells={cells}, χ²={chi_fill:.4}"),
        )
        .chi_square(chi_fill, df_fill as f64),
        TestResult::with_note(
            "dieharder::fill_tree_position",
            p_pos,
            format!("trials={N_TRIALS}, χ²={chi_pos:.4}"),
        )
        .chi_square(chi_pos, df_pos as f64),
    ]
}

/// Fill-tree test as one result.
///
/// The two statistics come from the same trials, so the fold uses a
/// Bonferroni bound (valid under dependence) rather than a bare min,
/// which would double the false-failure rate.
pub fn fill_tree(words: &[u32]) -> TestResult {
    let mut results = fill_tree_both(words);
    if results.iter().any(TestResult::skipped) {
        return results.remove(0);
    }
    let p_fill = results[0].p_value;
    let p_pos = results[1].p_value;
    TestResult::with_note(
        "dieharder::fill_tree",
        (2.0 * p_fill.min(p_pos)).min(1.0),
        format!("p_fill={p_fill:.4}, p_pos={p_pos:.4} (Bonferroni)"),
    )
    .with_statistic(
        "smaller p-value",
        p_fill.min(p_pos),
        None,
        "Bonferroni bound",
    )
}

/// Pearson χ² of the fill counts (indexed by t) against the exact
/// distribution, the tail cells pooled until each expects at least
/// [`MIN_EXPECTED`] trials.  Returns (χ², df, cells).
fn fill_count_chi_square(counts: &[u32; MAX_FILL + 1]) -> (f64, usize, usize) {
    let n = N_TRIALS as f64;
    let law = fill_count_distribution();
    let first = law.iter().position(|&p| p > 0.0).expect("a nonempty law");
    let expected: Vec<f64> = law[first..].iter().map(|&p| n * p).collect();
    let observed: Vec<f64> = counts[first..].iter().map(|&c| f64::from(c)).collect();
    let (chi, df) = chi_square_pooled_tails(&observed, &expected, MIN_EXPECTED)
        .expect("the law has cells expecting at least five trials");
    (chi, df, df + 1)
}

/// P(fill count = t), t = 0 … 15, computed once.
fn fill_count_distribution() -> &'static [f64; MAX_FILL + 1] {
    static LAW: OnceLock<[f64; MAX_FILL + 1]> = OnceLock::new();
    LAW.get_or_init(|| {
        let joint = collision_distribution(LEVELS);
        std::array::from_fn(|t| joint.get(t).map_or(0.0, |row| row.iter().sum()))
    })
}

/// D_h(t, g) of the module documentation, as rows t of gap probabilities.
fn collision_distribution(levels: usize) -> Vec<Vec<f64>> {
    if levels == 0 {
        return vec![vec![1.0]];
    }
    let sub = collision_distribution(levels - 1);
    let half = sub[0].len();
    // P(T ≥ r) for the subtree.
    let at_least: Vec<f64> = (0..=sub.len())
        .map(|r| sub[r.min(sub.len())..].iter().flatten().sum())
        .collect();
    let mut out = vec![vec![0.0; 2 * half]; 2 * sub.len()];
    for (a, row) in sub.iter().enumerate() {
        for (r, &tail) in at_least.iter().enumerate().take(sub.len()) {
            let weight = tail * (a + 1) as f64 / ((a + r + 1) * (a + r + 2)) as f64;
            for (g, &p) in row.iter().enumerate() {
                out[1 + a + r][g] += p * weight;
                out[1 + a + r][half + g] += p * weight;
            }
        }
    }
    out
}

/// Binary search-tree insertion into a flat array whose empty slots are NaN.
///
///   - Start at index `START_VAL` with step `(START_VAL+1)/2`.
///   - If slot is empty (`NaN`), place `x` there and return `None` (success).
///   - Else move left or right, halve the step.
///   - If step reaches 0, return `Some(i)` (collision at even position i).
///
/// The search path reaches only the 15 slots 1, 3, …, 29; every insert that
/// does not collide fills one of them, so a trial ends within 16 inserts.
fn tree_insert(x: f64, array: &mut [f64; SIZE]) -> Option<usize> {
    let mut i = START_VAL;
    let mut d = START_VAL.div_ceil(2); // = 8
    while d > 0 {
        if array[i].is_nan() {
            array[i] = x;
            return None; // success
        }
        if array[i] < x {
            i += d;
        } else {
            i -= d;
        }
        d /= 2;
    }
    Some(i) // collision: return position
}

#[cfg(test)]
mod tests {
    use super::{
        fill_count_chi_square, fill_count_distribution, fill_tree_both, tree_insert, GAPS,
        MAX_FILL, N_TRIALS, SIZE, START_VAL,
    };
    use crate::rng::{Mt19937, Rng};

    /// Inserting 1/16 … 15/16 in level order fills slots 1, 3, …, 29, the
    /// only ones the search path reaches; the next insert must collide.
    #[test]
    fn fifteen_inserts_fill_every_reachable_slot() {
        let mut array = [f64::NAN; SIZE];
        for k in [8, 4, 12, 2, 6, 10, 14, 1, 3, 5, 7, 9, 11, 13, 15] {
            assert_eq!(
                tree_insert(f64::from(k) / 16.0, &mut array),
                None,
                "k = {k}"
            );
        }
        let filled: Vec<usize> = (0..SIZE).filter(|&i| !array[i].is_nan()).collect();
        assert_eq!(filled, (1..=29).step_by(2).collect::<Vec<_>>());
        assert_eq!(tree_insert(0.0, &mut array), Some(0));
        assert_eq!(tree_insert(0.99, &mut array), Some(30));
    }

    /// No trial needs more than 16 words.
    #[test]
    fn every_trial_collides_within_sixteen_words() {
        let mut rng = Mt19937::new(5489);
        let mut longest = 0;
        for _ in 0..N_TRIALS {
            let mut array = [f64::NAN; SIZE];
            let mut words = 1;
            while tree_insert(rng.next_f64(), &mut array).is_none() {
                words += 1;
            }
            longest = longest.max(words);
        }
        assert!(longest <= 16, "longest trial took {longest} words");
    }

    /// On an all-zero stream every trial collides on its fifth word at
    /// position 0, so both statistics are scored and fail.
    #[test]
    fn all_zero_stream_is_scored_and_fails() {
        let words = vec![0u32; N_TRIALS * 8];
        for r in fill_tree_both(&words) {
            assert!(!r.skipped(), "{r}");
            assert!(r.p_value < 1e-10, "{r}");
        }
    }

    /// A sample of exactly 0.0 occupies its slot and is not overwritten.
    #[test]
    fn zero_sample_occupies_its_slot() {
        let mut array = [f64::NAN; SIZE];
        assert_eq!(tree_insert(0.0, &mut array), None); // placed at START_VAL
        assert_eq!(array[START_VAL], 0.0);
        // A larger value routes right (0.0 < 0.5) into a different empty node,
        // so the stored 0.0 is not overwritten.
        assert_eq!(tree_insert(0.5, &mut array), None);
        assert_eq!(array[START_VAL], 0.0, "0.0 slot must remain occupied");
    }

    /// The exact fill-count law: rational values at both ends, total 1, and
    /// support 4 … 15.
    #[test]
    fn fill_count_distribution_is_exact() {
        let law = fill_count_distribution();
        assert!(law[..4].iter().all(|&p| p == 0.0));
        for (t, want) in [
            (4, 2.0 / 15.0),
            (5, 1.0 / 5.0),
            (6, 13.0 / 63.0),
            (7, 5.0 / 28.0),
            (8, 295.0 / 2268.0),
            (11, 1.0 / 54.0),
            (15, 1.0 / 59535.0),
        ] {
            assert!((law[t] - want).abs() < 1e-15, "P({t}) = {}", law[t]);
        }
        assert!((law.iter().sum::<f64>() - 1.0).abs() < 1e-14);
    }

    /// Every gap has probability 1/16, so the position test's uniform law is
    /// exact.
    #[test]
    fn collision_position_is_uniform() {
        let joint = super::collision_distribution(4);
        assert_eq!(joint.len(), MAX_FILL + 1);
        for g in 0..GAPS {
            let p: f64 = joint.iter().map(|row| row[g]).sum();
            assert!((p - 1.0 / 16.0).abs() < 1e-15, "gap {g}: {p}");
        }
    }

    /// Counts rounded from their expectations give χ² near 0; the top two fill counts are
    /// pooled (1.68 expected at t = 15), leaving 11 cells.
    #[test]
    fn fill_count_cells_cover_every_trial() {
        let law = fill_count_distribution();
        let counts: [u32; MAX_FILL + 1] =
            std::array::from_fn(|t| (law[t] * N_TRIALS as f64).round() as u32);
        let (chi, df, cells) = fill_count_chi_square(&counts);
        assert_eq!((cells, df), (11, 10));
        assert!(chi < 0.1, "χ² = {chi}");
    }

    #[test]
    fn short_inputs_skip() {
        let short = vec![0; N_TRIALS * 8 - 1];
        for r in fill_tree_both(&[])
            .into_iter()
            .chain(fill_tree_both(&short))
        {
            assert!(r.skipped(), "{r}");
        }
    }
}
