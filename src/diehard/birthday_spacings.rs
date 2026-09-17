//! DIEHARD birthday spacings test.
//!
//! Each trial chooses m = 512 birthdays from a year of n = 2²⁴ days: 24 bits
//! of each of 512 words.  The birthdays are sorted, their spacings
//! C(1) = B(1), C(i) = B(i) − B(i−1) are sorted, and j counts the i with
//! C(i) = C(i−1), so a spacing value that occurs r times adds r − 1.  j is
//! asymptotically Poisson with λ = m³/(4n) = 2.
//!
//! 500 trials per bit offset give a histogram of j, scored with a Pearson χ²
//! against Poisson(2) in cells of at least 5 expected trials: at 500 trials,
//! j = 0 to 5 each alone and j ≥ 6 pooled, expecting 67.668, 135.335,
//! 135.335, 90.224, 45.112, 18.045 and 8.282 trials, df 6.  Offset o reads
//! bits o to o + 23 of each word, for o = 0 to 8, each offset on its own
//! 256 000 words (2 304 000 in all), so the nine p-values are independent
//! under the null; a Kolmogorov–Smirnov test of the nine is the result.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{
    math::{igamc, ks_test},
    result::TestResult,
};

const M: usize = 512; // birthdays per trial
const YEAR: u32 = 1 << 24; // year size = 2^24 (nbits = 24)
const SAMPLES: usize = 500; // trials per bit window
const LAMBDA: f64 = 2.0; // m³/(4n) = 512³/(4×2^24) = 2
const WINDOWS: usize = 9; // bit offsets 0..=8

/// Run the birthday spacings test.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn birthday_spacings(words: &[u32]) -> TestResult {
    if words.len() < WINDOWS * SAMPLES * M {
        return TestResult::insufficient("diehard::birthday_spacings", "not enough words");
    }

    let (cell_of, expected) = poisson_cells(LAMBDA, SAMPLES);
    let df = expected.len() - 1;
    let mut p_values: Vec<f64> = Vec::with_capacity(WINDOWS);
    let mut word_iter = words.iter().copied();
    let mut birthdays = vec![0u32; M];
    let mut spacings = vec![0u32; M];

    for offset in 0..WINDOWS {
        let mut observed = vec![0u32; expected.len()];

        for _ in 0..SAMPLES {
            for birthday in birthdays.iter_mut() {
                // The length gate guarantees the iterator has enough words.
                let word = word_iter
                    .next()
                    .expect("birthday_spacings: word iterator exhausted (precondition failed)");
                *birthday = (word >> offset) & (YEAR - 1);
            }
            birthdays.sort_unstable();

            // C(1) = B(1), C(i) = B(i) − B(i−1).
            spacings[0] = birthdays[0];
            for (spacing, pair) in spacings[1..].iter_mut().zip(birthdays.windows(2)) {
                *spacing = pair[1] - pair[0];
            }
            spacings.sort_unstable();

            let j = repeated_spacings(&spacings);
            observed[cell_of[j.min(cell_of.len() - 1)]] += 1;
        }

        let chi_sq: f64 = observed
            .iter()
            .zip(&expected)
            .map(|(&obs, &exp)| (f64::from(obs) - exp).powi(2) / exp)
            .sum();
        p_values.push(igamc(df as f64 / 2.0, chi_sq / 2.0));
    }

    // Final KS test on the 9 chi-square p-values.
    let p_value = ks_test(&mut p_values);
    let d = crate::math::ks_statistic(&mut p_values);

    TestResult::with_note(
        "diehard::birthday_spacings",
        p_value,
        format!("m={M}, year=2^24, samples={SAMPLES}"),
    )
    .kolmogorov_smirnov(d, p_values.len())
}

/// The number of i with C(i) = C(i−1) in sorted spacings: a value that
/// occurs r times adds r − 1.
fn repeated_spacings(sorted: &[u32]) -> usize {
    sorted.windows(2).filter(|pair| pair[0] == pair[1]).count()
}

/// χ² cells for a Poisson(`lambda`) count over `trials` trials.
///
/// Returns `cell_of`, the cell of each count up to the first pooled one
/// (larger counts share the last cell), and each cell's expected number of
/// trials.  Counts join the open cell from 0 upward, and the cell closes once
/// it expects at least 5 trials (cell 0 needs more than 5).  As soon as fewer
/// than 5 trials are expected above count i, count i and every larger count
/// join the open cell, which takes the whole remaining expectation.
fn poisson_cells(lambda: f64, trials: usize) -> (Vec<usize>, Vec<f64>) {
    /// Trials a cell must expect before it closes: the conventional lower
    /// bound for a χ² cell (W. G. Cochran, "The χ² test of goodness of fit,"
    /// *Annals of Mathematical Statistics* 23(3), 1952).
    const MIN_EXPECTED: f64 = 5.0;
    /// Standard deviations of the Poisson count the loop covers before the
    /// remaining expectation is pooled: √λ each, so the tail left out is
    /// below 10⁻⁴ of the trials.
    const TAIL_SIGMAS: f64 = 4.0;
    let n = trials as f64;
    let mut p = (-lambda).exp();
    let mut cumulative = p * n;
    let mut open = p * n;
    let mut cell_of = vec![0];
    let mut expected = Vec::new();
    if open > MIN_EXPECTED {
        expected.push(open);
        open = 0.0;
    }
    let last = (lambda + TAIL_SIGMAS * lambda.sqrt()) as usize;
    for i in 1..=last {
        p = lambda * p / i as f64;
        cumulative += p * n;
        open += p * n;
        cell_of.push(expected.len());
        if cumulative > n - MIN_EXPECTED {
            expected.push(open + n - cumulative);
            return (cell_of, expected);
        }
        if open >= MIN_EXPECTED {
            expected.push(open);
            open = 0.0;
        }
    }
    // Not reached at λ = 2 and 500 trials; close the open cell with the
    // remaining expectation.
    expected.push(open + n - cumulative);
    (cell_of, expected)
}

#[cfg(test)]
mod tests {
    use super::{birthday_spacings, poisson_cells, repeated_spacings, LAMBDA, M, SAMPLES, WINDOWS};

    /// A value seen r times adds r − 1.
    #[test]
    fn repeats_count_adjacent_equal_spacings() {
        assert_eq!(repeated_spacings(&[1, 2, 3]), 0);
        assert_eq!(repeated_spacings(&[5, 5, 7, 7]), 2);
        assert_eq!(repeated_spacings(&[5, 5, 5]), 2);
        assert_eq!(repeated_spacings(&[4, 4, 4, 4, 9]), 3);
    }

    /// At λ = 2 and 500 trials counts 0 to 5 are scored alone and 6 and up
    /// pooled: expectations 500·e⁻²·2ʲ/j! and 500·P(j ≥ 6).
    #[test]
    fn cells_at_lambda_two() {
        let (cell_of, expected) = poisson_cells(LAMBDA, SAMPLES);
        assert_eq!(cell_of, [0, 1, 2, 3, 4, 5, 6]);
        let mut term = 500.0 * (-2.0f64).exp();
        let mut head = 0.0;
        for (j, e) in expected[..6].iter().enumerate() {
            if j > 0 {
                term *= 2.0 / j as f64;
            }
            head += term;
            assert!((e - term).abs() < 1e-9, "cell {j}: {e} vs {term}");
        }
        assert!((expected[6] - (500.0 - head)).abs() < 1e-9);
        assert!((expected.iter().sum::<f64>() - 500.0).abs() < 1e-9);
    }

    /// Words needed: 500 trials of 512 birthdays at each of 9 bit offsets.
    const NEEDED: usize = WINDOWS * SAMPLES * M;

    #[test]
    fn short_inputs_skip() {
        assert!(birthday_spacings(&[]).skipped());
        assert!(birthday_spacings(&vec![0; NEEDED - 1]).skipped());
    }

    /// Every birthday is day 0, so every spacing is 0 and every trial lands in
    /// the pooled cell.
    #[test]
    fn constant_input_fails() {
        let r = birthday_spacings(&vec![0; NEEDED]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
