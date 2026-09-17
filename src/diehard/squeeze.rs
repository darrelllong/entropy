//! DIEHARD squeeze test.
//!
//! Starting from k = 2³¹ − 1, each step replaces k by ⌈k·U⌉, with U a word
//! scaled to (0, 1), and j counts the steps until k = 1, stopping at 48.
//! 100 000 values of j are counted in 43 cells, j ≤ 6, j = 7 … 47 and
//! j = 48, and scored with a Pearson χ².
//!
//! # Cell probabilities
//!
//! For k ≥ 2 and U uniform, ⌈k·U⌉ is uniform on 1 … k.  With f_k(j) the
//! probability of j steps from k,
//!
//! f_1(0) = 1,   f_k(j) = (1/k) Σ_{m=1..k} f_m(j − 1),
//!
//! which [`step_count_distribution`] evaluates with running sums in O(k·j)
//! time.  [`CELL_PROBABILITIES`] holds its values at k = 2³¹ − 1, computed by
//! `examples/squeeze_table.rs` in about six minutes; the tests check the
//! recurrence against direct evaluation.  The 32-bit resolution of U is
//! ignored.
//!
//! # Pooling
//!
//! At 100 000 trials five cells expect fewer than 5 counts (j ≤ 6 and
//! j = 45 … 48, together 9.278); they are scored as one pooled cell beside the
//! 38 others, so df = 38.  Pooling rather than dropping them keeps every
//! trial in the statistic, so a generator that over-produces extreme lengths
//! is still seen (see [`crate::math::chi_square_pooled`]).
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::chi_square_pooled, result::TestResult, rng::Rng};

const N_TRIALS: usize = 100_000;
const N_CELLS: usize = 43; // j≤6, j=7..47, j≥48
/// Minimum expected count for a cell to be scored on its own.
const CUTOFF: f64 = 5.0;

/// P(j falls in cell c) for c = 0 … 42, from [`step_count_distribution`]
/// at k = 2³¹ − 1 (see the module documentation).
pub const CELL_PROBABILITIES: [f64; N_CELLS] = [
    2.10325190895967e-05,
    5.779251312082994e-05,
    0.0001755378334850173,
    0.0004673230959129956,
    0.0011078273104774378,
    0.0023678432149366367,
    0.004609445216858303,
    0.008241166504847597,
    0.013627817807304099,
    0.02096850148149553,
    0.030176128758310442,
    0.040801978276716964,
    0.05204203868805697,
    0.06283828810232171,
    0.07205637981069637,
    0.07869451498632746,
    0.08206755521806683,
    0.08191934712014019,
    0.07844007520114707,
    0.0721941089264247,
    0.0639867843840994,
    0.05470930136919189,
    0.045198513605491035,
    0.036136598319086964,
    0.028000268738051812,
    0.021055667399313208,
    0.015386517949261403,
    0.010940197995838235,
    0.007577958033015017,
    0.005119562263246218,
    0.003377256475537947,
    0.0021778643255639424,
    0.0013743850510540042,
    0.0008496976042414834,
    0.0005151815757911927,
    0.00030665731846050385,
    0.00017938960624225574,
    0.00010323895443623727,
    5.8511581104414977e-05,
    3.269186524932054e-05,
    1.8025309183084338e-05,
    9.817782106971244e-06,
    1.1209908696852011e-05,
];

/// Step at which the squeeze stops counting.
const STEP_CAP: usize = 48;

/// P(j = 0), …, P(j = `cap` − 1) steps to reduce `start` to 1 by
/// k ← ⌈k·U⌉ with U uniform on (0, 1), by the recurrence in the module
/// documentation.  The running sums reach `start` in size while each term is
/// below 1, so they are accumulated with Kahan–Babuška compensation.
#[must_use]
pub fn step_count_distribution(start: u64, cap: usize) -> Vec<f64> {
    // sums[j] = (Σ_{m<k} f_m(j), compensation); f[j] = f_k(j).
    let mut sums = vec![(0.0f64, 0.0f64); cap];
    let mut f = vec![0.0f64; cap];
    if cap == 0 {
        return f;
    }
    f[0] = 1.0;
    sums[0].0 = 1.0;
    for k in 2..=start {
        let kf = k as f64;
        f[0] = 0.0;
        for j in 1..cap {
            let (sum, error) = sums[j - 1];
            f[j] = (sum + error + f[j - 1]) / kf;
        }
        for ((sum, error), &x) in sums.iter_mut().zip(&f) {
            let t = *sum + x;
            *error += if sum.abs() >= x.abs() {
                (*sum - t) + x
            } else {
                (x - t) + *sum
            };
            *sum = t;
        }
    }
    f
}

/// The 43 cells of [`step_count_distribution`]: j ≤ 6, 7 … 47, and the
/// remainder for j = 48.
#[must_use]
pub fn cells_from_steps(steps: &[f64]) -> [f64; N_CELLS] {
    let mut cells = [0.0f64; N_CELLS];
    for (j, &p) in steps.iter().enumerate().take(STEP_CAP) {
        cells[j.saturating_sub(6).min(N_CELLS - 1)] += p;
    }
    cells[N_CELLS - 1] = 1.0 - steps.iter().take(STEP_CAP).sum::<f64>();
    cells
}

/// Run the squeeze test.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn squeeze(rng: &mut impl Rng) -> TestResult {
    let mut counts = [0u32; N_CELLS];

    for _ in 0..N_TRIALS {
        let mut k: i64 = 2_147_483_647; // 2^31 − 1
        let mut j: usize = 0;

        while k != 1 && j < STEP_CAP {
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
    .chi_square(chi_sq, df as f64)
}

/// Score a squeeze histogram: Pearson χ² against
/// `N_TRIALS · CELL_PROBABILITIES` with the weak cells pooled.  Returns `(p_value, df, chi_sq)`;
/// the NaN arm is unreachable at `N_TRIALS` (39 cells score) and exists only
/// so a future change of sample size degrades to SKIP instead of a panic.
fn score(counts: &[u32; N_CELLS]) -> (f64, usize, f64) {
    let n = N_TRIALS as f64;
    let expected: [f64; N_CELLS] = std::array::from_fn(|i| n * CELL_PROBABILITIES[i]);
    chi_square_pooled(counts, &expected, CUTOFF).unwrap_or((f64::NAN, 0, f64::NAN))
}

#[cfg(test)]
mod tests {
    /// DIEHARD quotes the pooled χ² of its example to three decimals.
    const PUBLISHED_CHI: f64 = 1e-3;

    /// A sum of cell probabilities that is exactly 1.
    const PROBABILITY_SUM: f64 = 1e-14;

    /// A χ² and p-value recomputed by the same arithmetic.
    const CLOSED_FORM: f64 = 1e-12;

    /// A χ² recomputed from pooled cells, which round.
    const RECOMPUTED_CHI: f64 = 1e-9;

    /// The p-value of that χ², compared relatively.
    const RELATIVE_P: f64 = 1e-6;

    use super::{
        cells_from_steps, score, squeeze, step_count_distribution, CELL_PROBABILITIES, CUTOFF,
        N_CELLS, N_TRIALS,
    };
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
        let weak: Vec<usize> = (0..N_CELLS)
            .filter(|&i| n * CELL_PROBABILITIES[i] < CUTOFF)
            .collect();
        assert_eq!(weak, [0, 39, 40, 41, 42]);
        let pooled: f64 = weak.iter().map(|&i| n * CELL_PROBABILITIES[i]).sum();
        assert!((pooled - 9.278).abs() < PUBLISHED_CHI, "pooled = {pooled}");
    }

    /// χ² with the 38 strong cells scored alone and the five weak cells pooled,
    /// and Q(19, χ²/2) = e^(−χ²/2) Σ_{i<19} (χ²/2)^i / i!.
    fn by_hand(counts: &[u32; N_CELLS]) -> (f64, f64) {
        let n = N_TRIALS as f64;
        let (mut chi, mut pooled_e, mut pooled_o) = (0.0, 0.0, 0.0);
        for (&c, &p) in counts.iter().zip(&CELL_PROBABILITIES) {
            let e = n * p;
            if e < CUTOFF {
                pooled_e += e;
                pooled_o += f64::from(c);
            } else {
                chi += (f64::from(c) - e).powi(2) / e;
            }
        }
        chi += (pooled_o - pooled_e).powi(2) / pooled_e;
        let x = chi / 2.0;
        let (mut term, mut sum) = (1.0f64, 1.0f64);
        for i in 1..19 {
            term *= x / f64::from(i);
            sum += term;
        }
        (chi, (-x).exp() * sum)
    }

    #[test]
    fn score_on_near_expectation_counts() {
        let (p, df, chi) = score(&NEAR);
        let (want_chi, want_p) = by_hand(&NEAR);
        assert_eq!(df, 38);
        assert!(
            (chi - want_chi).abs() < CLOSED_FORM,
            "χ² = {chi} vs {want_chi}"
        );
        assert!((p - want_p).abs() < CLOSED_FORM, "p = {p} vs {want_p}");
    }

    /// Over-produced extreme lengths reach the pooled cell and are rejected.
    #[test]
    fn score_rejects_over_produced_extreme_lengths() {
        let (p, df, chi) = score(&EXTREME);
        let (want_chi, want_p) = by_hand(&EXTREME);
        assert_eq!(df, 38);
        assert!(chi > 1000.0);
        assert!(
            (chi - want_chi).abs() < RECOMPUTED_CHI,
            "χ² = {chi} vs {want_chi}"
        );
        assert!((p / want_p - 1.0).abs() < RELATIVE_P, "p = {p} vs {want_p}");
    }

    /// The table is a distribution: positive cells summing to 1.
    #[test]
    fn cell_probabilities_sum_to_one() {
        assert!(CELL_PROBABILITIES.iter().all(|&p| p > 0.0));
        let sum: f64 = CELL_PROBABILITIES.iter().sum();
        assert!((sum - 1.0).abs() < PROBABILITY_SUM, "sum = {sum}");
    }

    /// f_k(j) by the recurrence with sums taken afresh for every k.
    fn direct_distribution(start: usize, cap: usize) -> Vec<f64> {
        let mut f = vec![vec![0.0f64; cap]; start + 1];
        f[1][0] = 1.0;
        for k in 2..=start {
            for j in 1..cap {
                f[k][j] = (1..=k).map(|m| f[m][j - 1]).sum::<f64>() / k as f64;
            }
        }
        f.swap_remove(start)
    }

    /// The running-sum evaluation agrees with the direct one, and from k = 2
    /// the step count is geometric: P(j) = 2⁻ʲ.
    #[test]
    fn running_sums_match_direct_recurrence() {
        for start in [1, 2, 3, 10, 300] {
            let fast = step_count_distribution(start, 30);
            let slow = direct_distribution(start as usize, 30);
            for (j, (a, b)) in fast.iter().zip(&slow).enumerate() {
                assert!(
                    (a - b).abs() < PROBABILITY_SUM,
                    "start {start}, j {j}: {a} vs {b}"
                );
            }
        }
        let two = step_count_distribution(2, 10);
        for (j, &p) in two.iter().enumerate().skip(1) {
            assert_eq!(p, 0.5f64.powi(j as i32));
        }
    }

    /// Cells from a million-start distribution land near the table at the
    /// short step counts, which change only slowly with ln k.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "10⁸ steps; runs under cargo test --release"
    )]
    fn cells_are_consistent_with_a_smaller_start() {
        let cells = cells_from_steps(&step_count_distribution(1 << 21, 48));
        assert!((cells.iter().sum::<f64>() - 1.0).abs() < PROBABILITY_SUM);
        // ln(2³¹)/ln(2²¹) ≈ 1.48: the smaller start needs fewer steps, so its
        // mass sits in lower cells.
        let mean = |c: &[f64]| c.iter().enumerate().map(|(i, p)| i as f64 * p).sum::<f64>();
        assert!(mean(&cells) < mean(&CELL_PROBABILITIES));
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
