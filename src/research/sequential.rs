//! A sequential test of fair independent bits with an error guarantee at every
//! sample size.
//!
//! A predictor gives, before each bit x_t, a probability q_t that the bit is 1,
//! computed from earlier bits only.  Its wealth
//!
//! E_t = E_{t−1} · 2 · q_t^{x_t} · (1 − q_t)^{1−x_t},  E_0 = 1,
//!
//! has conditional expectation E_{t−1} when each bit is fair and independent of
//! the past, whatever q_t is, so E_t is a nonnegative martingale with mean 1.
//! An average of such martingales with fixed weights is again one.  Ville's
//! inequality bounds the chance that a nonnegative martingale starting at 1
//! ever reaches 1/α by α, so p = min(1, 1/sup_t E_t) is a valid p-value however
//! long the bits are watched and wherever watching stops.  It is conservative:
//! under the null it is smaller than α with probability at most α, not exactly
//! α, so it must not be fed to tests that assume uniform p-values.
//!
//! The predictors here are Markov models of orders 0 … K.  The order-k model
//! predicts from the counts of zeros and ones seen after the same k preceding
//! bits, with the Krichevsky–Trofimov estimate q = (n₁ + ½)/(n₀ + n₁ + 1).
//! Bits before the first k are read as zeros.  The test's wealth is the
//! uniform average of the K + 1 models' wealth, checked after every word.  A
//! bias, a short-range dependence or a short period grows some model's wealth
//! exponentially; a fair source leaves every model's wealth below 1 in
//! expectation.
//!
//! # References
//! * J. Ville, *Étude critique de la notion de collectif*, Gauthier-Villars,
//!   1939.  [The maximal inequality for nonnegative martingales]
//! * S. R. Howard, A. Ramdas, J. McAuliffe and J. Sekhon, "Time-uniform,
//!   nonparametric, nonasymptotic confidence sequences," *Annals of
//!   Statistics* 49(2), pp. 1055–1080, 2021.
//! * R. E. Krichevsky and V. K. Trofimov, "The performance of universal
//!   encoding," *IEEE Transactions on Information Theory* 27(2), pp. 199–207,
//!   1981.  [The estimator (n₁ + ½)/(n + 1)]
//! * F. M. J. Willems, Y. M. Shtarkov and T. J. Tjalkens, "The context-tree
//!   weighting method: basic properties," *IEEE Transactions on Information
//!   Theory* 41(3), pp. 653–664, 1995.  [Mixtures of context models]

use crate::{result::TestResult, rng::Rng};
use std::f64::consts::LN_2;

/// The most bits a [`MarkovMixture`] accepts, 2⁵³: every count stays an
/// integer that `f64` represents exactly.
pub const MAX_BITS: u64 = 1 << 53;

/// Markov models of orders 0 … `max_order` and their log-wealth.
pub struct MarkovMixture {
    /// counts[k][context] = [zeros, ones] seen after that k-bit context.
    counts: Vec<Vec<[u64; 2]>>,
    log_wealth: Vec<f64>,
    history: u64,
    bits: u64,
}

impl MarkovMixture {
    /// Orders 0 … `max_order`, at most 24.
    ///
    /// # Panics
    /// Panics if `max_order` exceeds 24.
    #[must_use]
    pub fn new(max_order: usize) -> Self {
        assert!(max_order <= 24, "max_order must be at most 24");
        Self {
            counts: (0..=max_order).map(|k| vec![[0, 0]; 1 << k]).collect(),
            log_wealth: vec![0.0; max_order + 1],
            history: 0,
            bits: 0,
        }
    }

    /// Update every model with one bit.
    ///
    /// # Panics
    /// Panics after [`MAX_BITS`] bits.
    pub fn push(&mut self, bit: bool) {
        assert!(self.bits < MAX_BITS, "more than 2^53 bits");
        self.bits += 1;
        let x = usize::from(bit);
        for (k, (table, log_w)) in self.counts.iter_mut().zip(&mut self.log_wealth).enumerate() {
            let context = (self.history & ((1u64 << k) - 1)) as usize;
            let cell = &mut table[context];
            let n = (cell[0] + cell[1]) as f64;
            let q = (cell[x] as f64 + 0.5) / (n + 1.0);
            *log_w += LN_2 + q.ln();
            cell[x] += 1;
        }
        self.history = (self.history << 1) | x as u64;
    }

    /// Natural log of the mixture's wealth: the log of the average of the
    /// models' wealth.
    #[must_use]
    pub fn log_wealth(&self) -> f64 {
        let top = self
            .log_wealth
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let sum: f64 = self.log_wealth.iter().map(|w| (w - top).exp()).sum();
        top + (sum / self.log_wealth.len() as f64).ln()
    }

    /// The order whose wealth is largest.
    #[must_use]
    pub fn leading_order(&self) -> usize {
        self.log_wealth
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(k, _)| k)
    }
}

/// Run the mixture of orders 0 … `max_order` over `words` words of `rng`, bits
/// least significant first, checking its wealth after every word, and report
/// p = min(1, 1/sup E) as `sequential::markov_mixture`.  The p-value is valid
/// for stopping after any word; wealth is not inspected inside a word.
///
/// Log-wealth is a sum of one rounded logarithm per model per bit, so its
/// floating error grows with the number of bits; the tests compare order 0
/// with its closed form over 10⁵ bits.
#[must_use]
pub fn markov_mixture(rng: &mut impl Rng, words: usize, max_order: usize) -> TestResult {
    if words as u64 > MAX_BITS / 32 {
        return TestResult::unsupported("sequential::markov_mixture", "more than 2^48 words");
    }
    let mut mixture = MarkovMixture::new(max_order);
    let mut sup_log = 0.0f64;
    let mut sup_at = 0usize;
    for w in 0..words {
        let word = rng.next_u32();
        for i in 0..32 {
            mixture.push((word >> i) & 1 == 1);
        }
        let log_wealth = mixture.log_wealth();
        if log_wealth > sup_log {
            sup_log = log_wealth;
            sup_at = w + 1;
        }
    }
    let p = (-sup_log).exp().min(1.0);
    TestResult::with_note(
        "sequential::markov_mixture",
        p,
        format!(
            "words={words}, orders=0..={max_order}, ln sup E={sup_log:.3} after {sup_at} words, \
             final ln E={:.3}, leading order {}; anytime-valid, conservative",
            mixture.log_wealth(),
            mixture.leading_order()
        ),
    )
    .with_statistic("ln sup E", sup_log, None, "Ville's inequality, p = 1/sup E")
}

#[cfg(test)]
mod tests {
    use super::{markov_mixture, MarkovMixture};
    use crate::rng::{alternatives::Biased, ConstantRng, Pcg64};

    /// Averaged over all 2¹² bit strings, each model's final wealth and the
    /// mixture's are exactly 1: the martingale property, by enumeration.
    #[test]
    fn wealth_averages_to_one_over_every_string() {
        let n = 12;
        let mut model_sums = [0.0f64; 4];
        let mut mixture_sum = 0.0f64;
        for s in 0u32..1 << n {
            let mut m = MarkovMixture::new(3);
            for i in 0..n {
                m.push((s >> i) & 1 == 1);
            }
            for (sum, log_w) in model_sums.iter_mut().zip(&m.log_wealth) {
                *sum += log_w.exp();
            }
            mixture_sum += m.log_wealth().exp();
        }
        let strings = f64::from(1u32 << n);
        for (k, sum) in model_sums.iter().enumerate() {
            assert!(
                (sum / strings - 1.0).abs() < 1e-12,
                "order {k}: {}",
                sum / strings
            );
        }
        assert!((mixture_sum / strings - 1.0).abs() < 1e-12);
    }

    /// Order 0's wealth has the closed form
    /// E = 2ⁿ·Γ(n₀ + ½)·Γ(n₁ + ½) / (π·Γ(n + 1)), 2ⁿ times the KT probability
    /// of the string; the running sum of logarithms agrees over 10⁵ bits.
    #[test]
    fn order_zero_wealth_matches_its_closed_form() {
        use crate::{math::lgamma, rng::Rng};
        let mut rng = Pcg64::new(9, 9);
        let mut m = MarkovMixture::new(0);
        let (mut n0, mut n1) = (0u64, 0u64);
        for i in 0..100_000 {
            let bit = i % 3 == 0 || rng.next_u32() & 1 == 1;
            m.push(bit);
            if bit {
                n1 += 1;
            } else {
                n0 += 1;
            }
        }
        let n = (n0 + n1) as f64;
        let exact = n * std::f64::consts::LN_2 + lgamma(n0 as f64 + 0.5) + lgamma(n1 as f64 + 0.5)
            - std::f64::consts::PI.ln()
            - lgamma(n + 1.0);
        assert!(
            (m.log_wealth[0] - exact).abs() < 1e-7,
            "{} vs {exact}",
            m.log_wealth[0]
        );
    }

    /// A count past u32::MAX neither wraps nor panics.
    #[test]
    fn counts_pass_the_32_bit_boundary() {
        let mut m = MarkovMixture::new(0);
        m.counts[0][0] = [u64::from(u32::MAX), 0];
        let before = m.log_wealth[0];
        m.push(false);
        assert_eq!(m.counts[0][0], [1u64 << 32, 0]);
        // q = (2³² − ½)/2³², so the wealth nearly doubles.
        assert!((m.log_wealth[0] - before - std::f64::consts::LN_2).abs() < 1e-9);
    }

    #[test]
    fn a_fair_generator_is_not_rejected_and_defects_are() {
        let fair = markov_mixture(&mut Pcg64::new(1, 1), 50_000, 8);
        assert!(fair.p_value > 0.01, "{fair}");
        let biased = markov_mixture(&mut Biased::new(Pcg64::new(1, 1), 6), 50_000, 8);
        assert!(biased.p_value < 1e-6, "{biased}");
        let constant = markov_mixture(&mut ConstantRng::new(0x1234_5678), 100, 8);
        assert!(constant.p_value < 1e-30, "{constant}");
        assert!(constant.note.unwrap().contains("leading order"));
    }
}
