//! DIEHARD battery tests not already covered by NIST SP 800-22.
//!
//! Reference: George Marsaglia, *DIEHARD: A Battery of Tests of Randomness*,
//! Florida State University, 1995.  `pubs/Diehard.zip`
//!
//! Every function in this module cites Marsaglia as the original author.

pub mod birthday_spacings;
pub mod operm5;
pub mod binary_rank;
pub mod bitstream;
pub mod monkey;
pub mod count_ones;
pub mod parking_lot;
pub mod minimum_distance;
pub mod spheres_3d;
pub mod squeeze;
pub mod overlapping_sums;
pub mod runs_float;
pub mod craps;

use crate::{result::TestResult, rng::Rng};

/// Run all unique DIEHARD tests and return results.
///
/// `n_u32` is the number of 32-bit words to consume; 10 000 is sufficient
/// for most tests; 1 000 000 is recommended for the full battery.
/// `quick` reduces the O(n²) geometric test parameters for fast iteration.
pub fn run_all(rng: &mut impl Rng, n_u32: usize, quick: bool) -> Vec<TestResult> {
    let words = rng.collect_u32s(n_u32);
    let mut results = vec![
        birthday_spacings::birthday_spacings(&words),
        operm5::operm5(&words),
        binary_rank::binary_rank_32x32(&words),
        binary_rank::binary_rank_31x31(&words),
        binary_rank::binary_rank_6x8(&words),
        bitstream::bitstream(&words),
        monkey::opso(&words),
        monkey::oqso(&words),
        monkey::dna(&words),
        count_ones::count_ones_stream(&words),
        count_ones::count_ones_specific_bytes(&words),
        parking_lot::parking_lot(rng, quick),
        minimum_distance::minimum_distance_2d(rng, quick),
        spheres_3d::spheres_3d(rng, quick),
        squeeze::squeeze(rng),
        overlapping_sums::overlapping_sums(&words),
    ];
    // runs_float_both returns two results: one for up-runs, one for down-runs.
    results.extend(runs_float::runs_float_both(rng));
    // craps has two independent statistics; emit both rather than collapsing to min.
    results.extend(craps::craps_both(rng));
    results
}
