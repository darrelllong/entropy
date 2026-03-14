//! NIST SP 800-22 Rev 1a — all 15 statistical tests.
//!
//! Reference: Rukhin et al., *A Statistical Test Suite for Random and
//! Pseudorandom Number Generators for Cryptographic Applications*,
//! NIST SP 800-22 Rev 1a (2010).  `pubs/NIST-SP-800-22r1a.pdf`
//!
//! Each sub-module corresponds to one section of the document.

pub mod frequency;              // §2.1
pub mod block_frequency;        // §2.2
pub mod runs;                   // §2.3
pub mod longest_run;            // §2.4
pub mod matrix_rank;            // §2.5
pub mod spectral;               // §2.6
pub mod non_overlapping_template; // §2.7
pub mod overlapping_template;   // §2.8
pub mod universal;              // §2.9
pub mod linear_complexity;      // §2.10
pub mod serial;                 // §2.11
pub mod approximate_entropy;    // §2.12
pub mod cumulative_sums;        // §2.13
pub mod random_excursions;      // §2.14
pub mod random_excursions_variant; // §2.15

use crate::{result::TestResult, rng::Rng};

/// Run all 15 NIST SP 800-22 tests and return the results.
///
/// Uses the recommended default parameters from SP 800-22 §2.  The sequence
/// length `n` should be at least 1 000 000 for the full battery; 100 000 is
/// the minimum for most tests.
pub fn run_all(rng: &mut impl Rng, n: usize) -> Vec<TestResult> {
    let bits = rng.collect_bits(n);
    vec![
        frequency::frequency(&bits),
        block_frequency::block_frequency(&bits, 128),
        runs::runs(&bits),
        longest_run::longest_run(&bits),
        matrix_rank::matrix_rank(&bits),
        spectral::spectral(&bits),
        non_overlapping_template::non_overlapping_template(&bits, 9),
        overlapping_template::overlapping_template(&bits, 9),
        universal::universal(&bits),
        linear_complexity::linear_complexity(&bits, 500),
        serial::serial(&bits, 3),
        approximate_entropy::approximate_entropy(&bits, 10),
        cumulative_sums::cumulative_sums_forward(&bits),
        cumulative_sums::cumulative_sums_backward(&bits),
        random_excursions::random_excursions(&bits),
        random_excursions_variant::random_excursions_variant(&bits),
    ]
}
