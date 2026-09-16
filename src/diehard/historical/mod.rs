//! Historical DIEHARD tests, run only when asked for.
//!
//! Nothing here is part of [`crate::diehard::run_all`] or of `run_tests`'
//! default battery: `run_tests --suite diehard-historical` runs [`run_all`],
//! and no other selection does, including the default of running every
//! suite.
//!
//! | Result name | Test |
//! |---|---|
//! | `diehard_historical::operm5` | Overlapping 5-permutations, χ² through the pseudoinverse of their covariance ([`operm5`]) |
//! | `diehard_historical::overlapping_sums` | Decorrelated overlapping sums of uniforms, three Anderson–Darling layers ([`overlapping_sums`]) |
//! | `diehard_historical::count_ones_bytes` | Count-the-1s on each of the 25 byte windows of a word, 25 results ([`count_ones_bytes`]) |
//! | `diehard_historical::rank_6x8_windows` | 6×8 binary rank on each of the 25 byte windows of a word, 25 results ([`rank_6x8`]) |
//! | `diehard_historical::rank_6x8_windows_summary` | Anderson–Darling summary of those 25 window p-values ([`rank_6x8`]) |
//!
//! Each module gives its statistic, its null distribution and the evidence
//! that its p-values are calibrated.  They stay outside the default battery
//! because each adds result slots that the battery's other tests largely
//! cover.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

pub mod count_ones_bytes;
pub mod operm5;
pub mod overlapping_sums;
pub mod rank_6x8;

use crate::{result::TestResult, rng::Rng};

/// Words [`run_all`] must capture for every historical test to run.
pub const WORDS_NEEDED: usize = larger(
    larger(operm5::WORDS, overlapping_sums::WORDS),
    larger(count_ones_bytes::WORDS, rank_6x8::WORDS),
);

const fn larger(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

/// Results [`run_all`] returns: one each for OPERM5 and overlapping sums, 25
/// for count-the-1s and 26 for the 6×8 rank test.
pub const RESULTS: usize = 2 + count_ones_bytes::WINDOWS + rank_6x8::RESULTS;

/// Run every historical DIEHARD test on one capture of `n_u32` words,
/// returning [`RESULTS`] results.
///
/// Each test reads from the start of the same capture, as
/// [`crate::diehard::run_all`] does; a test given fewer words than it needs
/// reports SKIP.  [`WORDS_NEEDED`] words run them all.
///
/// # Examples
///
/// ```no_run
/// use entropy::{diehard::historical, rng::Mt19937};
///
/// let mut rng = Mt19937::new(5489);
/// for result in historical::run_all(&mut rng, historical::WORDS_NEEDED) {
///     println!("{result}");
/// }
/// ```
pub fn run_all(rng: &mut impl Rng, n_u32: usize) -> Vec<TestResult> {
    let words = rng.collect_u32s(n_u32);
    let mut results = vec![
        operm5::operm5(&words),
        overlapping_sums::overlapping_sums(&words),
    ];
    results.extend(count_ones_bytes::count_ones_bytes(&words));
    results.extend(rank_6x8::rank_6x8_windows(&words));
    results
}

/// Floor on each product uᵢ(1 − uₙ₋₁₋ᵢ), so that a p-value of exactly 0 or 1
/// adds a large finite term rather than an infinite one.
const AD_PRODUCT_FLOOR: f64 = 1e-20;

/// The Anderson–Darling statistic
/// Aₙ² = −n − (1/n) Σᵢ (2i − 1) ln(uᵢ(1 − uₙ₊₁₋ᵢ)) of `u` against U(0, 1),
/// each product floored at [`AD_PRODUCT_FLOOR`].  Sorts `u` in place.
///
/// # Author
/// T. W. Anderson and D. A. Darling, "A Test of Goodness of Fit", *JASA* 49
/// (1954).
fn anderson_darling_statistic(u: &mut [f64]) -> f64 {
    u.sort_by(f64::total_cmp);
    let n = u.len();
    let log_sum: f64 = (0..n)
        .map(|i| {
            let product = (u[i] * (1.0 - u[n - 1 - i])).max(AD_PRODUCT_FLOOR);
            (2 * i + 1) as f64 * product.ln()
        })
        .sum();
    -(n as f64) - log_sum / n as f64
}

#[cfg(test)]
mod tests {
    use super::{run_all, RESULTS};
    use crate::rng::Mt19937;

    #[test]
    fn run_all_reports_every_result_even_when_it_skips() {
        let results = run_all(&mut Mt19937::new(5489), 0);
        assert_eq!(results.len(), RESULTS);
        assert!(results.iter().all(|r| r.skipped()));
    }
}
