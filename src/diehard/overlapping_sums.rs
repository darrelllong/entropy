//! DIEHARD Test 14 — Overlapping Sums Test.
//!
//! Forms overlapping sums S_i = U_i + U_{i+1} + … + U_{i+W−1} of W=100
//! consecutive floats in [0,1).  Each S_i is approximately Normal(W/2, W/12).
//!
//! Consecutive sums overlap in W−1 elements, giving correlation ρ = (W−1)/W.
//! The AR(1) innovation D_i = (S_{i+1} − ρ·S_i − μ(1−ρ)) / (σ√(1−ρ²)) is
//! approximately N(0,1), but D_i and D_{i+k} retain a small residual correlation
//! (W−k)(1−ρ)/(W(1+ρ)) for k < W that biases a naive Kolmogorov-Smirnov test.
//!
//! To obtain truly independent samples we subsample every W-th innovation:
//! Corr(D_i, D_{i+W}) = 0 exactly (no shared uniform inputs).  This yields
//! N_SUMS/W ≈ 100 independent standard-normal deviates per repeat, which are
//! converted to U(0,1) via the normal CDF and tested with
//! Kolmogorov-Smirnov.  The test repeats 10 times; the 10 p-values are
//! given a final Kolmogorov-Smirnov test.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::{ks_test, normal_cdf}, result::TestResult};

const WINDOW: usize = 100;
const N_SUMS: usize = 20_000;   // 200 independent subsampled D values per repeat
const REPEATS: usize = 10;

/// Run the overlapping sums test.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn overlapping_sums(words: &[u32]) -> TestResult {
    let needed = REPEATS * (N_SUMS + WINDOW - 1);
    if words.len() < needed {
        return TestResult::insufficient("diehard::overlapping_sums", "not enough words");
    }

    let floats: Vec<f64> = words.iter().map(|&w| w as f64 / 4_294_967_296.0).collect();

    let mu    = WINDOW as f64 / 2.0;
    let sigma = (WINDOW as f64 / 12.0).sqrt();
    let rho   = (WINDOW - 1) as f64 / WINDOW as f64;
    let scale = sigma * (1.0 - rho * rho).sqrt();

    let mut outer_pvals = Vec::with_capacity(REPEATS);

    for rep in 0..REPEATS {
        let start = rep * (N_SUMS + WINDOW - 1);
        let chunk = &floats[start..start + N_SUMS + WINDOW - 1];

        // Compute rolling overlapping sums S_0 … S_{N_SUMS−1}.
        let mut rolling: f64 = chunk[..WINDOW].iter().sum();
        let mut sums = Vec::with_capacity(N_SUMS);
        sums.push(rolling);
        for i in WINDOW..N_SUMS + WINDOW - 1 {
            rolling += chunk[i] - chunk[i - WINDOW];
            sums.push(rolling);
        }

        // AR(1) innovation: D_i = (S_{i+1} − ρ·S_i − μ(1−ρ)) / (σ√(1−ρ²)).
        // Corr(D_i, D_{i+k}) = (W−k)(1−ρ)/(W(1+ρ)) for k < W; = 0 for k ≥ W.
        // Subsample every W-th innovation to obtain independent deviates.
        let mut uniforms: Vec<f64> = (0..N_SUMS - 1)
            .step_by(WINDOW)
            .map(|i| {
                let d = (sums[i + 1] - rho * sums[i] - mu * (1.0 - rho)) / scale;
                normal_cdf(d).clamp(1e-15, 1.0 - 1e-15)
            })
            .collect();

        outer_pvals.push(ks_test(&mut uniforms));
    }

    let p_value = ks_test(&mut outer_pvals);

    TestResult::with_note(
        "diehard::overlapping_sums",
        p_value,
        format!("window={WINDOW}, n_sums={N_SUMS}, subsample={WINDOW}, repeats={REPEATS}"),
    )
}
