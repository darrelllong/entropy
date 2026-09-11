//! DIEHARD Test 13 — Squeeze Test.
//!
//! Starting from k = 2 147 483 647 (= 2³¹ − 1), repeatedly applies
//! k = ⌈k · U⌉ where U is drawn from the generator as a float in (0,1).
//! Counts j, the number of steps to reduce k to 1 (or j = 48 if not
//! reached).  Repeats 100 000 times; the distribution of j is tested
//! with chi-square against the theoretical cell probabilities.
//!
//! Cell layout: j ≤ 6 (pooled, index 0), j = 7..=47 (individual, indices 1–41),
//! j ≥ 48 (pooled, index 42).  Total: 43 cells.
//!
//! The chi-square pools weak cells as Dieharder's `Vtest_eval` does (see
//! [`crate::math::vtest_pvalue`]).  At N = 100 000 five cells expect fewer
//! than 5 counts (j ≤ 6, j = 45, 46, 47 and j ≥ 48; together 9.278), so they
//! are scored as one pooled cell next to the 38 strong cells: 39 cells,
//! df = 38.  Dropping those cells instead (df = 37) would never see a
//! generator that over-produces extreme squeeze lengths.
//!
//! DIEHARD itself does neither.  Marsaglia's `sqeez` (`fortran/diehard.f`
//! lines 219–281) scores all 43 cells with no pooling (lines 254–256), five
//! of them expecting 0.98 to 3.27 counts, and reports `chisq(chsq,42)`, the
//! CDF with df = 42 (line 272).  The pooling here is Dieharder's.
//!
//! Cell probabilities from George Marsaglia, DIEHARD (1995) (`ex` in `sqeez`,
//! lines 228–233), as transcribed in Robert G. Brown's Dieharder 3.31.1,
//! `diehard_squeeze.c`.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::vtest_pvalue, result::TestResult, rng::Rng};

const N_TRIALS: usize = 100_000;
const N_CELLS: usize = 43; // j≤6, j=7..47, j≥48
/// Minimum expected count for a cell to be scored on its own
/// (`vtest.cutoff = 5.0` in `diehard_squeeze.c`).
const CUTOFF: f64 = 5.0;

/// Theoretical P(j falls in cell c) for c = 0..43.
/// Source: Marsaglia DIEHARD, as reproduced in Dieharder 3.31.1 diehard_squeeze.c.
const SDATA: [f64; N_CELLS] = [
    0.00002103, 0.00005779, 0.00017554, 0.00046732, 0.00110783, 0.00236784, 0.00460944, 0.00824116,
    0.01362781, 0.02096849, 0.03017612, 0.04080197, 0.05204203, 0.06283828, 0.07205637, 0.07869451,
    0.08206755, 0.08191935, 0.07844008, 0.07219412, 0.06398679, 0.05470931, 0.04519852, 0.03613661,
    0.02800028, 0.02105567, 0.01538652, 0.01094020, 0.00757796, 0.00511956, 0.00337726, 0.00217787,
    0.00137439, 0.00084970, 0.00051518, 0.00030666, 0.00017939, 0.00010324, 0.00005851, 0.00003269,
    0.00001803, 0.00000982, 0.00001121,
];

/// Run the squeeze test.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn squeeze(rng: &mut impl Rng) -> TestResult {
    let mut counts = [0u32; N_CELLS];

    for _ in 0..N_TRIALS {
        let mut k: i64 = 2_147_483_647; // 2^31 − 1
        let mut j: usize = 0;

        while k != 1 && j < 48 {
            // U in (0, 1): shift by 0.5 to avoid U=0 making k drop to 0.
            let u = (rng.next_u32() as f64 + 0.5) / 4_294_967_296.0;
            k = (k as f64 * u).ceil() as i64;
            j += 1;
        }

        // Clamp to cell range: j ≤ 6 → index 0; j = 7..47 → index j-6; j ≥ 48 → index 42.
        let j = j.max(6);
        let idx = (j - 6).min(N_CELLS - 1);
        counts[idx] += 1;
    }

    let (p_value, df, chi_sq) = score(&counts);

    TestResult::with_note(
        "diehard::squeeze",
        p_value,
        format!("trials={N_TRIALS}, cells={N_CELLS}, df={df}, χ²={chi_sq:.4}"),
    )
}

/// Score a squeeze histogram: Pearson chi-square against `N_TRIALS · SDATA`
/// with Dieharder's `Vtest_eval` pooling.  Returns `(p_value, df, chi_sq)`;
/// the NaN arm is unreachable at `N_TRIALS` (39 cells score) and exists only
/// so a future change of sample size degrades to SKIP instead of a panic.
fn score(counts: &[u32; N_CELLS]) -> (f64, usize, f64) {
    let n = N_TRIALS as f64;
    let expected: [f64; N_CELLS] = std::array::from_fn(|i| n * SDATA[i]);
    vtest_pvalue(counts, &expected, CUTOFF).unwrap_or((f64::NAN, 0, f64::NAN))
}

#[cfg(test)]
mod tests {
    use super::{score, squeeze, CUTOFF, N_CELLS, N_TRIALS, SDATA};
    use crate::rng::ConstantRng;

    /// Near-expectation counts with nonzero weak cells (indices 0, 39–42).
    #[rustfmt::skip]
    const NEAR: [u32; N_CELLS] = [
        3, 6, 18, 47, 111, 254, 461, 824, 1363, 2097, 3018, 4080, 5204, 6284, 7206, 7869,
        8167, 8217, 7844, 7219, 6399, 5471, 4520, 3614, 2800, 2106, 1539, 1094, 758, 512, 332, 218,
        137, 85, 52, 31, 18, 10, 6, 4, 1, 2, 0,
    ];

    /// Strong cells at their expectations; the weak cells hold 106 counts
    /// against a pooled expectation of 9.278 (j ≤ 6 and j ≥ 48 over-produced).
    #[rustfmt::skip]
    const EXTREME: [u32; N_CELLS] = [
        40, 6, 18, 47, 111, 237, 461, 824, 1363, 2097, 3018, 4080, 5204, 6284, 7206, 7869,
        8207, 8192, 7844, 7219, 6399, 5471, 4520, 3614, 2800, 2106, 1539, 1094, 758, 512, 338, 218,
        137, 85, 52, 31, 18, 10, 6, 3, 2, 1, 60,
    ];

    #[test]
    fn weak_cells_are_the_documented_five() {
        let n = N_TRIALS as f64;
        let weak: Vec<usize> = (0..N_CELLS).filter(|&i| n * SDATA[i] < CUTOFF).collect();
        assert_eq!(weak, [0, 39, 40, 41, 42]);
        let pooled: f64 = weak.iter().map(|&i| n * SDATA[i]).sum();
        assert!((pooled - 9.278).abs() < 1e-9, "pooled = {pooled}");
    }

    // Reference values: a line-by-line Python replica of Dieharder's
    // `Vtest_eval` (Vtest.c) on the same counts and `tsamples * sdata[i]`
    // expectations.  Dropping the weak cells instead gives χ² = 1.66426…
    // with df = 37 on NEAR, and χ² = 0.04660… (p ≈ 1) on EXTREME.
    #[test]
    fn score_matches_vtest_eval_on_near_expectation_counts() {
        let (p, df, chi) = score(&NEAR);
        assert_eq!(df, 38);
        assert!((chi - 1.7204464916623783).abs() < 1e-12, "χ² = {chi}");
        assert!((p - 0.9999999999999998).abs() < 1e-12, "p = {p}");
    }

    #[test]
    fn score_rejects_over_produced_extreme_lengths() {
        let (p, df, chi) = score(&EXTREME);
        assert_eq!(df, 38);
        assert!((chi - 1008.361461728401).abs() < 1e-9, "χ² = {chi}");
        // Q(19, χ²/2) in closed form: e^(−χ²/2) Σ_{i<19} (χ²/2)^i / i!.
        assert!((p / 7.817315442438951e-187 - 1.0).abs() < 1e-6, "p = {p}");
    }

    /// SDATA is printed to eight decimals, so its 43 entries sum to 1 only
    /// within 43 × 5 × 10⁻⁹.
    #[test]
    fn sdata_sums_to_one() {
        let sum: f64 = SDATA.iter().sum();
        assert!((sum - 1.0).abs() <= 43.0 * 5e-9, "sum = {sum}");
    }

    /// Word 0 gives U ≈ 1.2 × 10⁻¹⁰, which drops k to 1 at once (j ≤ 6 in
    /// every trial); u32::MAX gives U just below 1, which never shrinks k
    /// (j = 48 in every trial).
    #[test]
    fn constant_generators_fail() {
        for value in [0, u32::MAX] {
            let r = squeeze(&mut ConstantRng::new(value));
            assert!(!r.skipped() && !r.passed(), "{value}: {r}");
        }
    }
}
