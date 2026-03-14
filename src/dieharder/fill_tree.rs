//! DIEHARDER test 207/208 — dab_filltree / dab_filltree2.
//!
//! Inserts random bit sequences as paths in a binary trie (prefix tree) and
//! tests the fill statistics.  Bit-level correlations cause the tree to fill
//! faster or slower than expected for independent bits.
//!
//! # Author
//! David Bauer, *Dieharder* (2006), tests `dab_filltree` and `dab_filltree2`.

use crate::{math::igamc, result::TestResult};

const TREE_DEPTH: usize = 24;   // 2^24 leaves
const N_INSERTIONS: usize = 1 << 22; // 4M insertions
const REPEATS: usize = 5;

/// Run the fill-tree test.
///
/// # Author
/// David Bauer, Dieharder (2006), `dab_filltree`.
pub fn fill_tree(words: &[u32]) -> TestResult {
    let bits_per_insertion = TREE_DEPTH;
    let words_per_rep = (N_INSERTIONS * bits_per_insertion + 31) / 32;

    if words.len() < REPEATS * words_per_rep {
        return TestResult::insufficient("dieharder::fill_tree", "not enough words");
    }

    // Under the null hypothesis (i.i.d. bits), after n insertions into a trie
    // of depth d, the expected number of distinct prefixes at depth k is
    // E_k = 2^k · (1 − (1 − 2^−k)^n).
    // We measure the number of distinct leaves actually reached and compare.
    //
    // For the chi-square test we compare observed vs expected new-leaf counts
    // at each depth level.

    let mut total_chi = 0.0f64;
    let mut total_df = 0usize;
    let mut word_offset = 0usize;

    for _ in 0..REPEATS {
        let chunk = &words[word_offset..word_offset + words_per_rep];
        word_offset += words_per_rep;

        let bits = chunk
            .iter()
            .flat_map(|&w| (0..32u32).map(move |i| ((w >> i) & 1) as u8))
            .take(N_INSERTIONS * bits_per_insertion)
            .collect::<Vec<u8>>();

        let (observed_new, df) = measure_fill(&bits, TREE_DEPTH, N_INSERTIONS);
        let expected_new = expected_fill(TREE_DEPTH, N_INSERTIONS);

        // Chi-square comparing observed vs expected "occupancy" counts at depth boundaries.
        let chi: f64 = observed_new
            .iter()
            .zip(expected_new.iter())
            .filter(|(_, &e)| e >= 5.0)
            .map(|(&o, &e)| (o as f64 - e).powi(2) / e)
            .sum();

        total_chi += chi;
        total_df += df;
    }

    let p_value = igamc(total_df as f64 / 2.0, total_chi / 2.0);

    TestResult::with_note(
        "dieharder::fill_tree",
        p_value,
        format!("depth={TREE_DEPTH}, insertions={N_INSERTIONS}, χ²={total_chi:.4}"),
    )
}

/// Measure the number of new leaves discovered at each depth checkpoint.
fn measure_fill(bits: &[u8], depth: usize, n_insertions: usize) -> (Vec<u32>, usize) {
    // Use a flat hash set to track visited paths (only the full-depth leaves).
    // Check points: every n_insertions/10 insertions.
    let checkpoints = 10usize;
    let step = n_insertions / checkpoints;

    let mut visited = std::collections::HashSet::new();
    let mut counts = vec![0u32; checkpoints];

    for i in 0..n_insertions {
        let start = i * depth;
        if start + depth > bits.len() { break; }
        let path: u32 = (0..depth).fold(0u32, |acc, j| (acc << 1) | bits[start + j] as u32);
        visited.insert(path);

        if (i + 1) % step == 0 {
            let cp = (i + 1) / step - 1;
            if cp < checkpoints {
                counts[cp] = visited.len() as u32;
            }
        }
    }

    let df = checkpoints - 1;
    (counts, df)
}

/// Expected number of distinct leaves after n insertions into a depth-d trie.
fn expected_fill(depth: usize, n: usize) -> Vec<f64> {
    let total_leaves = (1u64 << depth) as f64;
    let checkpoints = 10usize;
    let step = n / checkpoints;
    (0..checkpoints)
        .map(|i| {
            let k = ((i + 1) * step) as f64;
            total_leaves * (1.0 - (-k / total_leaves).exp())
        })
        .collect()
}
