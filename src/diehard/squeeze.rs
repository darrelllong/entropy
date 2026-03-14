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
//! Cell probabilities from George Marsaglia, DIEHARD (1995), as transcribed
//! in Robert G. Brown's Dieharder 3.31.1, `diehard_squeeze.c`.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::igamc, rng::Rng, result::TestResult};

const N_TRIALS: usize = 100_000;
const N_CELLS:  usize = 43;   // j≤6, j=7..47, j≥48

/// Theoretical P(j falls in cell c) for c = 0..43.
/// Source: Marsaglia DIEHARD, as reproduced in Dieharder 3.31.1 diehard_squeeze.c.
const SDATA: [f64; N_CELLS] = [
    0.00002103, 0.00005779, 0.00017554, 0.00046732, 0.00110783,
    0.00236784, 0.00460944, 0.00824116, 0.01362781, 0.02096849,
    0.03017612, 0.04080197, 0.05204203, 0.06283828, 0.07205637,
    0.07869451, 0.08206755, 0.08191935, 0.07844008, 0.07219412,
    0.06398679, 0.05470931, 0.04519852, 0.03613661, 0.02800028,
    0.02105567, 0.01538652, 0.01094020, 0.00757796, 0.00511956,
    0.00337726, 0.00217787, 0.00137439, 0.00084970, 0.00051518,
    0.00030666, 0.00017939, 0.00010324, 0.00005851, 0.00003269,
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

    let n = N_TRIALS as f64;
    let chi_sq: f64 = counts
        .iter()
        .zip(SDATA.iter())
        .filter(|(_, &p)| p * n >= 5.0)
        .map(|(&c, &p)| (c as f64 - n * p).powi(2) / (n * p))
        .sum();

    let df = counts
        .iter()
        .zip(SDATA.iter())
        .filter(|(_, &p)| p * n >= 5.0)
        .count()
        .saturating_sub(1);

    let p_value = igamc(df as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "diehard::squeeze",
        p_value,
        format!("trials={N_TRIALS}, cells={N_CELLS}, df={df}, χ²={chi_sq:.4}"),
    )
}
