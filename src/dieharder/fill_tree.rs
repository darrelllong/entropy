//! DIEHARDER test 207 — dab_filltree.
//!
//! Fills a 32-element sorted binary array (treated as a binary search tree) with
//! random floating-point values.  Each trial records the number of words consumed
//! before a collision (i.e., before the search path reaches a node whose children
//! have already been filled).  A chi-square test compares the observed fill-count
//! distribution against the empirical reference table `TARGET_DATA` from the C
//! source.  A second chi-square tests that the collision positions are uniform.
//!
//! The tree is the 32-element sorted array from `dab_filltree.c`:
//!   - Array size: SIZE = 32, start value index: startVal = SIZE/2 - 1 = 15.
//!   - `insert(x, array, startVal)`: binary search from position 15 with step 8,
//!     halving the step at each level.  Returns the collision position when d=0.
//!
//! # Author
//! David Bauer, *Dieharder* (2006), test `dab_filltree`.
//! Source: `dieharder-3.31.1/libdieharder/dab_filltree.c`

use crate::{math::igamc, result::TestResult};

/// Binary tree array size (ntuple = 32 in the C source).
const SIZE: usize = 32;
/// Number of trials.
const N_TRIALS: usize = 100_000;
/// Number of rotation cycles.
const CYCLES: usize = 4;
/// Start node index: SIZE/2 - 1.
const START_VAL: usize = SIZE / 2 - 1; // = 15

/// Empirical probability P(collision on word i) for i = 0..TARGET_LEN.
/// Source: `dab_filltree.c` `targetData[]`.
const TARGET_DATA: [f64; 20] = [
    0.0,          // i=0: impossible
    0.0,          // i=1: impossible (root always empty on first insert)
    0.0,          // i=2
    0.0,          // i=3
    0.13333333,   // i=4
    0.20000000,   // i=5
    0.20634921,   // i=6
    0.17857143,   // i=7
    0.13007085,   // i=8
    0.08183633,   // i=9
    0.04338395,   // i=10
    0.01851828,   // i=11
    0.00617270,   // i=12
    0.00151193,   // i=13
    0.00023520,   // i=14
    0.00001680,   // i=15
    0.0,          // i=16
    0.0,          // i=17
    0.0,          // i=18
    0.0,          // i=19
];
const TARGET_LEN: usize = TARGET_DATA.len(); // 20

/// Run the fill-tree test and return both reference outputs separately.
///
/// # Author
/// David Bauer, Dieharder (2006), `dab_filltree`.
pub fn fill_tree_both(words: &[u32]) -> Vec<TestResult> {
    // Upper bound: each trial may consume at most SIZE*2 words.
    if words.len() < N_TRIALS * SIZE * 2 {
        return vec![
            TestResult::insufficient("dieharder::fill_tree_count", "not enough words"),
            TestResult::insufficient("dieharder::fill_tree_position", "not enough words"),
        ];
    }

    // Precompute expected counts for fill distribution.
    let n_f = N_TRIALS as f64;
    let expected_fill: Vec<f64> = TARGET_DATA.iter().map(|&p| p * n_f).collect();

    // Find chi-square range: use cells where expected >= 4.
    // Mirrors the C code: start = (last index with expected < 4 before any > 4) + 1,
    // end = last index with expected > 4.
    let mut start_idx = 0usize;
    let mut end_idx = 0usize;
    let mut found_end = false;
    for i in 0..TARGET_LEN {
        if expected_fill[i] < 4.0 {
            if !found_end { start_idx = i; }
        } else if expected_fill[i] > 4.0 {
            end_idx = i;
            found_end = true;
        }
    }
    start_idx += 1; // as in C: `start++`

    // Observed fill counts and position counts.
    let mut fill_counts = vec![0u32; TARGET_LEN];
    let mut position_counts = vec![0u32; SIZE / 2];

    let mut word_idx = 0usize;

    let mut rot_amount = 0u32;
    for j in 0..N_TRIALS {
        let mut array = [0.0f64; SIZE];
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

            // Rotate and normalise to (0, 1).
            let rotated = if rot_amount == 0 { v } else { v.rotate_left(rot_amount) };
            let x = rotated as f64 / u32::MAX as f64;

            if word_count > SIZE * 2 {
                // Should never happen with a non-degenerate RNG.
                break 0;
            }

            if let Some(pos) = tree_insert(x, &mut array) {
                break pos;
            }
        };

        let count_idx = (word_count - 1).min(TARGET_LEN - 1);
        fill_counts[count_idx] += 1;
        position_counts[fail_pos / 2] += 1;

        if j % (N_TRIALS / CYCLES) == 0 {
            rot_amount = (rot_amount + 1) % 32;
        }
    }

    // Chi-square 1: fill-count distribution vs TARGET_DATA.
    let chi_fill: f64 = (start_idx..=end_idx)
        .map(|i| {
            let e = expected_fill[i];
            let o = fill_counts[i] as f64;
            (o - e).powi(2) / e
        })
        .sum();
    let df_fill = (end_idx - start_idx).saturating_sub(1);
    let p_fill = igamc(df_fill as f64 / 2.0, chi_fill / 2.0);

    // Chi-square 2: collision position uniformity over SIZE/2 positions.
    let expected_pos = N_TRIALS as f64 / (SIZE / 2) as f64;
    let chi_pos: f64 = position_counts
        .iter()
        .map(|&c| (c as f64 - expected_pos).powi(2) / expected_pos)
        .sum();
    let df_pos = SIZE / 2 - 1;
    let p_pos = igamc(df_pos as f64 / 2.0, chi_pos / 2.0);

    vec![
        TestResult::with_note(
            "dieharder::fill_tree_count",
            p_fill,
            format!("trials={N_TRIALS}, χ²={chi_fill:.4}, start={start_idx}, end={end_idx}"),
        ),
        TestResult::with_note(
            "dieharder::fill_tree_position",
            p_pos,
            format!("trials={N_TRIALS}, χ²={chi_pos:.4}"),
        ),
    ]
}

/// Backward-compatible single-result wrapper.
pub fn fill_tree(words: &[u32]) -> TestResult {
    let mut results = fill_tree_both(words);
    if results.iter().any(TestResult::skipped) {
        return results.remove(0);
    }
    let p_fill = results[0].p_value;
    let p_pos = results[1].p_value;
    TestResult::with_note(
        "dieharder::fill_tree",
        p_fill.min(p_pos),
        format!("p_fill={p_fill:.4}, p_pos={p_pos:.4}"),
    )
}

/// Binary search-tree insertion into a flat double array.
///
/// Mirrors the C `insert()` from `dab_filltree.c`:
///   - Start at index `START_VAL` with step `(START_VAL+1)/2`.
///   - If slot is empty (0.0), place `x` there and return `None` (success).
///   - Else move left or right, halve the step.
///   - If step reaches 0, return `Some(i)` (collision at position i).
fn tree_insert(x: f64, array: &mut [f64; SIZE]) -> Option<usize> {
    let mut i = START_VAL;
    let mut d = (START_VAL + 1) / 2; // = 8
    while d > 0 {
        if array[i] == 0.0 {
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
