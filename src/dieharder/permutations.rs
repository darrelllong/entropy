//! DIEHARDER permutations test.
//!
//! Draws non-overlapping windows of t uniforms and counts which of the t!
//! orderings each window is in.  The windows are independent and every
//! ordering has probability 1/t!, so a Pearson χ² on the counts, df t! − 1,
//! is the statistic.  Unlike DIEHARD's OPERM5, which uses overlapping
//! windows, no covariance correction is needed.
//!
//! # Author
//! Robert G. Brown, *Dieharder: A Random Number Test Suite* (2004–2011).

use crate::{math::igamc, result::TestResult, rng::Rng};

/// Run the permutations test for windows of `t` consecutive floats.
///
/// # Author
/// Robert G. Brown, Dieharder (2006).
pub fn permutations(rng: &mut impl Rng, t: usize) -> TestResult {
    if !(2..=8).contains(&t) {
        return TestResult::unsupported("dieharder::permutations", "t must be 2..=8");
    }

    let n_perms = factorial(t);
    // Non-overlapping windows: at least 30 expected per ordering.
    let n_samples = 100_000.max(n_perms * 30);

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
    .chi_square(chi_sq, df as f64)
}

/// Lexicographic rank of the ordering permutation of `window`.
fn perm_rank(window: &[f64], t: usize) -> usize {
    let mut order: Vec<usize> = (0..t).collect();
    // The permutation test assumes continuous i.i.d. uniforms with no ties, but
    // 32-bit words can collide exactly.  Break equal values by index so the
    // ordering is a stable total order (deterministic rank) rather than
    // depending on sort implementation details.
    order.sort_unstable_by(|&a, &b| {
        window[a]
            .partial_cmp(&window[b])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(&b))
    });

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

#[cfg(test)]
mod tests {
    use super::permutations;
    use crate::rng::ConstantRng;

    #[test]
    fn window_sizes_outside_the_range_are_unsupported() {
        for t in [0, 1, 9] {
            assert!(permutations(&mut ConstantRng::new(0), t).is_unsupported());
        }
    }

    /// Ties keep their index order, so every window has rank 0.
    #[test]
    fn constant_generator_fails() {
        let r = permutations(&mut ConstantRng::new(0), 5);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
