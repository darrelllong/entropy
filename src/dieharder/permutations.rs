//! DIEHARDER test 202 — rgb_permutations.
//!
//! A parameterisable ordering test: tests whether all t! orderings of t
//! consecutive values appear with equal frequency.  This is the cleaner,
//! adjustable version of DIEHARD's OPERM5 (which is fixed at t = 5).
//!
//! # Author
//! Robert G. Brown, *Dieharder* (2006), test `rgb_permutations`.

use crate::{math::igamc, result::TestResult, rng::Rng};

/// Run the permutations test for windows of `t` consecutive floats.
///
/// # Author
/// Robert G. Brown, Dieharder (2006), `rgb_permutations`.
pub fn permutations(rng: &mut impl Rng, t: usize) -> TestResult {
    if !(2..=8).contains(&t) {
        return TestResult::insufficient("dieharder::permutations", "t must be 2..=8");
    }

    let n_perms = factorial(t);
    // Use non-overlapping k-tuples: tsamples independent draws of k values.
    // rgb_permutations.c: for t in 0..tsamples { fill testv[0..k] with k rands; sort; count }
    let n_samples = 100_000.max(n_perms * 30); // tsamples >> 30·k!

    let mut counts = vec![0u32; n_perms];
    let mut testv = vec![0.0f64; t];
    for _ in 0..n_samples {
        for v in testv.iter_mut() {
            *v = rng.next_f64();
        }
        let rank = perm_rank(&testv, t);
        counts[rank] += 1;
    }

    let expected = n_samples as f64 / n_perms as f64;
    let chi_sq: f64 = counts
        .iter()
        .map(|&c| (c as f64 - expected).powi(2) / expected)
        .sum();
    let df = n_perms - 1;

    let p_value = igamc(df as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "dieharder::permutations",
        p_value,
        format!("t={t}, n={n_samples}, χ²={chi_sq:.4}"),
    )
}

/// Lexicographic rank of the ordering permutation of `window`.
fn perm_rank(window: &[f64], t: usize) -> usize {
    let mut order: Vec<usize> = (0..t).collect();
    order.sort_unstable_by(|&a, &b| window[a].partial_cmp(&window[b]).unwrap());

    // Lehmer code → factorial number system rank.
    let mut rank = 0usize;
    let mut available: Vec<usize> = (0..t).collect();
    for (k, &ord) in order.iter().enumerate().take(t) {
        let pos = available.iter().position(|&v| v == ord).unwrap();
        rank += pos * factorial(t - 1 - k);
        available.remove(pos);
    }
    rank
}

fn factorial(n: usize) -> usize {
    (1..=n).product()
}
