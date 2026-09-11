//! Historical DIEHARD tests, run only when asked for.
//!
//! Nothing here is part of [`crate::diehard::run_all`] or of `run_tests`'
//! default battery; [`run_all`] runs the tests on request.  Each result name
//! carries its variant:
//!
//! | Result name | Variant |
//! |---|---|
//! | `diehard_historical::operm5_dieharder` | OPERM5 as corrected in Dieharder 3.31.1 ([`operm5`]) |
//! | `diehard_historical::overlapping_sums_fortran` | Overlapping sums as Marsaglia's `diehard.f` computes them ([`overlapping_sums`]) |
//! | `diehard_historical::count_ones_bytes_25_fresh` | Count-the-1s on DIEHARD's 25 byte windows, fresh words per window, 25 results ([`count_ones_bytes`]) |
//!
//! Each module says what the test is, which reference it follows, where and
//! why it departs from Marsaglia's `fortran/diehard.f`, the calibration
//! evidence behind it and why it stays outside the default battery.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).
//! [pubs/diehard-fortran-1996.tar.gz]

pub mod count_ones_bytes;
pub mod operm5;
mod operm5_table;
pub mod overlapping_sums;

#[cfg(test)]
mod oracle;

use crate::{result::TestResult, rng::Rng};

/// Words [`run_all`] must capture for every historical test to run.
pub const WORDS_NEEDED: usize = larger(
    larger(operm5::WORDS, overlapping_sums::WORDS),
    count_ones_bytes::WORDS,
);

const fn larger(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

/// Run every historical DIEHARD test on one capture of `n_u32` words.
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
        operm5::operm5_dieharder(&words),
        overlapping_sums::overlapping_sums_fortran(&words),
    ];
    results.extend(count_ones_bytes::count_ones_bytes_25_fresh(&words));
    results
}

/// Floor on each product uᵢ(1 − uₙ₊₁₋ᵢ) in Marsaglia's `KSTEST`
/// (`fortran/diehard.f` line 1690).
const AD_PRODUCT_FLOOR: f64 = 1e-20;

/// The Anderson–Darling statistic
/// Aₙ² = −n − (1/n) Σᵢ (2i − 1) ln(uᵢ(1 − uₙ₊₁₋ᵢ)) of `u` against U(0, 1),
/// as Marsaglia's `KSTEST` computes it (`fortran/diehard.f` lines
/// 1668–1709, which `tests.txt` calls a Kolmogorov–Smirnov test), each
/// product floored at 10⁻²⁰.  Sorts `u` in place.
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
