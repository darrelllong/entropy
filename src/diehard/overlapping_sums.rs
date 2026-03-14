//! DIEHARD Test 14 — Overlapping Sums Test.
//!
//! Forms overlapping sums S_i = U_i + U_{i+1} + … + U_{i+99} of 100
//! consecutive uniform variables, then applies the Cholesky-derived linear
//! transformation from DIEHARD to produce approximately N(0,1) values, which
//! are converted to uniforms and tested with Kolmogorov-Smirnov.  The
//! p-values from 10 KS tests are given a final KS test.
//!
//! **Known limitation**: Robert G. Brown (Dieharder 3.31.1) documents that
//! this test "converges to a non-zero p-value of ~0.097 for ALL rngs tested"
//! and is "completely useless."  Results should be interpreted with caution.
//!
//! The linear transformation implemented here follows the original DIEHARD
//! Fortran/C source (Marsaglia 1995), as transcribed in Dieharder 3.31.1
//! `diehard_sums.c`.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::{ks_test, normal_cdf}, result::TestResult};

const WINDOW: usize = 100;    // m: number of overlapping elements
const REPEATS: usize = 10;    // outer repetitions for the final KS test

/// Run the overlapping sums test using the Cholesky linear transformation.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn overlapping_sums(words: &[u32]) -> TestResult {
    // We need REPEATS × (WINDOW + WINDOW) words: WINDOW to fill the initial
    // sum, then WINDOW more for the m rolling-sum updates.
    // The C code uses m floats for initial seed + m new floats per repeat.
    let needed = REPEATS * (2 * WINDOW);
    if words.len() < needed {
        return TestResult::insufficient(
            "diehard::overlapping_sums (Cholesky-transform, known-biased)",
            "not enough words",
        );
    }

    let m = WINDOW;
    let mean = 0.5 * m as f64;
    let std  = (12.0f64).sqrt();

    let floats: Vec<f64> = words.iter().map(|&w| w as f64 / 4_294_967_296.0).collect();

    let mut outer_pvals = Vec::with_capacity(REPEATS);

    for rep in 0..REPEATS {
        let base = rep * 2 * m;
        let chunk = &floats[base..base + 2 * m];

        // Build initial sum S[0] from the first m floats.
        let mut raw_sum: f64 = chunk[..m].iter().sum();

        // y[i] will hold the Cholesky-normalised overlapping sums.
        // We need m values total: y[0] from the initial sum, y[1..m-1] from
        // rolling updates using chunk[m..2m-1].
        let mut y = vec![0.0f64; m];

        // y[0]: raw_sum normalised.
        y[0] = (raw_sum - mean) * std;

        for t in 1..m {
            // Roll: add chunk[m + t - 1], drop chunk[t - 1].
            raw_sum += chunk[m + t - 1] - chunk[t - 1];
            y[t] = (raw_sum - mean) * std;
        }

        // Apply the Cholesky linear transformation to make x[i] ~ N(0,1).
        let mut x = vec![0.0f64; m];
        x[0] = y[0] / (m as f64).sqrt();
        x[1] = -x[0] * (m - 1) as f64 / (2.0 * m as f64 - 1.0).sqrt()
              + y[1] * (m as f64 / (2.0 * m as f64 - 1.0)).sqrt();
        x[0] = normal_cdf(x[0]).clamp(1e-15, 1.0 - 1e-15);
        x[1] = normal_cdf(x[1]).clamp(1e-15, 1.0 - 1e-15);

        for t in 2..m {
            let a = 2.0 * m as f64 + 1.0 - t as f64;
            let b = 2.0 * a - 2.0;
            x[t] = y[t - 2] / (a * b).sqrt()
                  - y[t - 1] * ((a - 1.0) / (b + 2.0)).sqrt()
                  + y[t]     * (a / b).sqrt();
            x[t] = normal_cdf(x[t]).clamp(1e-15, 1.0 - 1e-15);
        }

        outer_pvals.push(ks_test(&mut x));
    }

    let p_value = ks_test(&mut outer_pvals);

    TestResult::with_note(
        "diehard::overlapping_sums (Cholesky-transform, known-biased)",
        p_value,
        format!("window={WINDOW}, repeats={REPEATS}; NOTE: biased ~0.097 for all RNGs per Dieharder"),
    )
}
