//! NIST SP 800-22 Rev 1a — all 15 statistical tests.
//!
//! Reference: Rukhin et al., *A Statistical Test Suite for Random and
//! Pseudorandom Number Generators for Cryptographic Applications*,
//! NIST SP 800-22 Rev 1a (2010).  `pubs/NIST-SP-800-22r1a.pdf`
//!
//! Each sub-module corresponds to one section of the document.  The unit
//! tests run the publication's worked examples, including those on the first
//! 10⁶ binary digits of e (`tests/data/e_1e6_bits.bin`, which a test
//! recomputes).

pub mod approximate_entropy; // §2.12
pub mod block_frequency; // §2.2
pub mod cumulative_sums; // §2.13
pub mod frequency; // §2.1
pub mod linear_complexity; // §2.10
pub mod longest_run; // §2.4
pub mod matrix_rank; // §2.5
pub mod non_overlapping_template; // §2.7
pub mod overlapping_template; // §2.8
pub mod random_excursions; // §2.14
pub mod random_excursions_variant; // §2.15
pub mod runs; // §2.3
pub mod serial; // §2.11
pub mod spectral; // §2.6
pub mod universal; // §2.9

#[cfg(test)]
mod test_vectors;

use crate::{result::TestResult, rng::Rng};

/// Run all 15 NIST SP 800-22 tests and return the results.
///
/// Always returns 200 results, in the order of the capacity comment in the
/// body; a test whose preconditions fail is skipped in its slot.
///
/// Uses the recommended default parameters from SP 800-22 §2.  The sequence
/// length `n` should be at least 1 000 000 for the full battery; 100 000 is
/// the minimum for most tests.
///
/// # Examples
///
/// ```no_run
/// use entropy::{nist, rng::Mt19937};
///
/// let mut rng = Mt19937::new(5489);
/// for result in nist::run_all(&mut rng, 1_000_000) {
///     println!("{result}");
/// }
/// ```
pub fn run_all(rng: &mut impl Rng, n: usize) -> Vec<TestResult> {
    let bits = rng.collect_bits(n);
    // Capacity: 12 fixed-result tests
    //         + 148 non-overlapping 9-bit templates (SP 800-22 Appendix E)
    //         +   2 serial sub-tests
    //         +  12 Maurer parametric settings (L = 5..=16)
    //         +   8 random_excursions states
    //         +  18 random_excursions_variant states
    //         = 200 total NIST slots
    let mut results = Vec::with_capacity(200);
    results.extend([
        frequency::frequency(&bits),
        block_frequency::block_frequency(&bits, 128),
        runs::runs(&bits),
        longest_run::longest_run(&bits),
        matrix_rank::matrix_rank(&bits),
        spectral::spectral(&bits),
        overlapping_template::overlapping_template(&bits, 9),
        universal::universal(&bits),
        linear_complexity::linear_complexity(&bits, 500),
        approximate_entropy::approximate_entropy(&bits, 10),
        cumulative_sums::cumulative_sums_forward(&bits),
        cumulative_sums::cumulative_sums_backward(&bits),
    ]);
    // Non-overlapping template: all 148 aperiodic 9-bit templates (SP 800-22 §2.7, Appendix E).
    results.extend(non_overlapping_template::non_overlapping_all(&bits));
    // Serial has two p-values; emit both rather than collapsing to min.
    // m = 3 is a fixed, size-independent default (unlike `universal`, which
    // scales L with n): the publication permits m up to ⌊log₂ n⌋ − 2 ≈ 21 at
    // this n, and a larger m would catch some weak linear generators, but a
    // fixed m keeps the slot comparable across runs
    // and sample sizes.  Callers wanting higher power can invoke
    // `serial::serial_both` directly with a larger m.
    results.extend(serial::serial_both(&bits, 3));
    // Maurer's original parametric family is useful beyond the single NIST-picked setting.
    results.extend(universal::universal_parametric_all(&bits));
    // Random excursions has 8 sub-tests; emit all.
    results.extend(random_excursions::random_excursions_all(&bits));
    // Random excursions variant has 18 sub-tests; emit all.
    results.extend(random_excursions_variant::random_excursions_variant_all(
        &bits,
    ));
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nist::test_vectors::e_bits;
    use crate::rng::Mt19937;

    /// SP 800-22 Appendix B, second table: the P-values for the first 10⁶
    /// bits of e, with the parameters the table names.  Appendix B prints the
    /// cumulative sums as 0.669887 and 0.724266, within 10⁻⁶ of the 0.6698865
    /// and 0.7242653 this crate computes.  The other rows are pinned beside their modules' §2.x.8
    /// examples or differ for reasons given there: overlapping template
    /// (§2.8.8), universal and linear complexity (their Appendix B tests),
    /// random excursions for x = +1 (§2.14.8) and the variant for x = −1
    /// (§2.15.8).
    #[test]
    fn matches_appendix_b_e_table() {
        let e = e_bits(1_000_000);
        let rows = [
            (frequency::frequency(&e), 0.953749),
            (block_frequency::block_frequency(&e, 128), 0.211072),
            (cumulative_sums::cumulative_sums_forward(&e), 0.669887),
            (cumulative_sums::cumulative_sums_backward(&e), 0.724266),
            (runs::runs(&e), 0.561917),
            // Appendix B prints 0.718945, from §3.4's four-decimal class
            // probabilities; the exact probabilities give 0.718366.
            (longest_run::longest_run(&e), 0.718366),
            (matrix_rank::matrix_rank(&e), 0.306156),
            (spectral::spectral(&e), 0.847187),
            (
                non_overlapping_template::non_overlapping_template(&e, 9),
                0.078790,
            ),
            (approximate_entropy::approximate_entropy(&e, 10), 0.700073),
            (serial::serial_both(&e, 16).swap_remove(0), 0.766182),
        ];
        for (r, p) in rows {
            assert!((r.p_value - p).abs() < 1e-6, "{r}");
        }
    }

    /// `run_all` keeps its 200-slot layout when the walk is too short for
    /// the excursion tests.
    #[test]
    fn run_all_keeps_200_slots_when_excursions_skip() {
        let results = run_all(&mut Mt19937::new(5489), 20_000);
        let names: Vec<&str> = results.iter().map(|r| r.name).collect();
        assert_eq!(names.len(), 200);
        assert_eq!(
            names[..12],
            [
                "nist::frequency",
                "nist::block_frequency",
                "nist::runs",
                "nist::longest_run",
                "nist::matrix_rank",
                "nist::spectral",
                "nist::overlapping_template",
                "nist::universal",
                "nist::linear_complexity",
                "nist::approximate_entropy",
                "nist::cumulative_sums_forward",
                "nist::cumulative_sums_backward",
            ]
        );
        assert!(names[12..160]
            .iter()
            .all(|&n| n == "nist::non_overlapping_template"));
        assert_eq!(
            names[160..162],
            ["nist::serial_delta1", "nist::serial_delta2"]
        );
        assert!(names[162..174]
            .iter()
            .all(|n| n.starts_with("maurer::universal_l")));
        assert!(names[174..182]
            .iter()
            .all(|&n| n == "nist::random_excursions"));
        assert!(names[182..]
            .iter()
            .all(|&n| n == "nist::random_excursions_variant"));
        assert!(results[174..].iter().all(|r| r.skipped()));
    }
}
