//! Historical DIEHARD tests, run only when asked for.
//!
//! Nothing here is part of [`crate::diehard::run_all`] or of `run_tests`'
//! default battery; [`run_all`] runs the tests on request.  Each result name
//! carries its variant:
//!
//! | Result name | Variant |
//! |---|---|
//! | `diehard_historical::operm5_dieharder` | OPERM5 as corrected in Dieharder 3.31.1 ([`operm5`]) |
//!
//! Each module says what the test is, which reference it follows, where and
//! why it departs from Marsaglia's `fortran/diehard.f`, the calibration
//! evidence behind it and why it stays outside the default battery.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).
//! [pubs/diehard-fortran-1996.tar.gz]

pub mod operm5;
mod operm5_table;

#[cfg(test)]
mod oracle;

use crate::{result::TestResult, rng::Rng};

/// Words [`run_all`] must capture for every historical test to run.
pub const WORDS_NEEDED: usize = operm5::WORDS;

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
    vec![operm5::operm5_dieharder(&words)]
}
