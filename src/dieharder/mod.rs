//! DIEHARDER tests not covered by NIST SP 800-22 or DIEHARD.
//!
//! Reference: Robert G. Brown, *Dieharder: A Random Number Test Suite*,
//! version 3.31.1.  <https://webhome.phy.duke.edu/~rgb/General/dieharder.php>
//! `pubs/dieharder-3.31.1.tgz`
//!
//! Each function cites its original author.

pub mod bit_distribution;
pub mod byte_distribution;
pub mod dct;
pub mod fill_tree;
pub mod gcd;
pub mod ks_uniform;
pub mod lagged_sums;
pub mod minimum_distance_nd;
pub mod monobit2;
pub mod permutations;

use crate::{result::TestResult, rng::Rng};

/// Run all unique DIEHARDER tests.
/// `quick` reduces the O(n²) geometric test parameters for fast iteration.
pub fn run_all(rng: &mut impl Rng, n_u32: usize, quick: bool) -> Vec<TestResult> {
    let words = rng.collect_u32s(n_u32);
    let mut results = vec![
        minimum_distance_nd::minimum_distance_nd(rng, 5, quick),
        permutations::permutations(rng, 5),
        lagged_sums::lagged_sums(&words, 1),
        lagged_sums::lagged_sums(&words, 100),
        ks_uniform::ks_uniform(&words),
        byte_distribution::byte_distribution(&words),
        dct::dct(&words),
        monobit2::monobit2(&words),
    ];
    results.extend(fill_tree::fill_tree_both(&words));
    // bit_distribution has per-width results; emit each independently.
    results.extend(bit_distribution::bit_distribution_all(&words, 8));
    // gcd has two independent statistics (GCD distribution + step counts).
    results.extend(gcd::gcd_both(rng));
    results
}
