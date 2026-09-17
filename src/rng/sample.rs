//! Exact sampling from a generator's words: integers in a range, Bernoulli
//! trials, floats, shuffles and byte strings.
//!
//! Every method here is exact with respect to the generator's words being
//! independent and uniform: an integer range gets exactly equal
//! probabilities, a Bernoulli trial exactly its probability.  None rounds a
//! modulus or truncates a float.  Each is value-stable: the same words give
//! the same result in every release, and the tests pin known answers.

use super::Rng;

/// Bits in a generator word.
const WORD_BITS: u32 = u64::BITS;

/// Significand bits of a double below its leading 1 (52).
const FRACTION_BITS: u32 = f64::MANTISSA_DIGITS - 1;

/// A double's biased exponent field, once the fraction is shifted out (0x7ff).
const EXPONENT_FIELD: u64 = (1 << (u64::BITS - 1 - FRACTION_BITS)) - 1;

/// −1 074: the binary exponent of the unit of a subnormal's integer significand.
const SUBNORMAL_UNIT_EXPONENT: i32 = f64::MIN_EXP - f64::MANTISSA_DIGITS as i32;

/// A normal double with biased exponent b and integer significand m is
/// m·2^(b − `INTEGER_SIGNIFICAND_BIAS`) (1075).
const INTEGER_SIGNIFICAND_BIAS: i32 = f64::MAX_EXP + FRACTION_BITS as i32 - 1;

/// Leading zero bits of a uniform real U from which U < 2⁻¹⁰⁷⁴ rounds down to
/// 0 (1 074), which is also the number of fractional bits a double can have.
const ZERO_BITS_TO_ZERO: u32 = SUBNORMAL_UNIT_EXPONENT.unsigned_abs();

/// Leading zero bits of U from which U < 2⁻¹⁰²² is subnormal (1 022).
const ZERO_BITS_TO_SUBNORMAL: u32 = (1 - f64::MIN_EXP) as u32;

/// Digit blocks that hold every fractional bit of a double (17).
const PROBABILITY_BLOCKS: u32 = ZERO_BITS_TO_ZERO.div_ceil(WORD_BITS);

/// A uniform integer in [0, `bound`) from `bits`-bit words, by D. Lemire's
/// multiply-and-reject method (D. Lemire, "Fast random integer generation in
/// an interval," *ACM Transactions on Modeling and Computer Simulation*
/// 29(1), Article 3, 2019, Algorithm 5).
///
/// A word x maps to ⌊x·bound / 2^bits⌋, accepted unless the low `bits` bits of
/// x·bound fall below t = (2^bits − bound) mod bound.  Each result then has
/// exactly ⌊2^bits / bound⌋ accepted preimages, so the result is uniform; the
/// test enumerates every word at 12 bits to confirm it.  The remainder is
/// computed only when the low bits fall below `bound`, which for 64-bit words
/// happens with probability below `bound`/2⁶⁴.
///
/// # Panics
/// Panics if `bound` is 0 or `bits` is outside 1 ..= 64.
fn lemire_below(mut next: impl FnMut() -> u64, bound: u64, bits: u32) -> u64 {
    assert!(bound > 0, "empty range");
    assert!((1..=WORD_BITS).contains(&bits), "word width");
    let modulus = 1u128 << bits;
    let low = |m: u128| m & (modulus - 1);
    let bound = u128::from(bound);
    let mut m = u128::from(next()) * bound;
    if low(m) < bound {
        let threshold = (modulus - bound) % bound;
        while low(m) < threshold {
            m = u128::from(next()) * bound;
        }
    }
    (m >> bits) as u64
}

/// The binary digits of a probability p in [0, 1), 64 at a time.  p is
/// m·2^e for its integer significand m < 2⁵³ and e ≤ −53, so its expansion
/// is finite; block i (from 1) is ⌊m·2^(e + 64i)⌋ mod 2⁶⁴.
struct ProbabilityDigits {
    significand: u64,
    exponent: i32,
    taken: i32,
}

impl ProbabilityDigits {
    fn new(p: f64) -> Self {
        let bits = p.to_bits();
        let biased = ((bits >> FRACTION_BITS) & EXPONENT_FIELD) as i32;
        let fraction = bits & ((1u64 << FRACTION_BITS) - 1);
        let (significand, exponent) = if biased == 0 {
            (fraction, SUBNORMAL_UNIT_EXPONENT)
        } else {
            (
                fraction | (1u64 << FRACTION_BITS),
                biased - INTEGER_SIGNIFICAND_BIAS,
            )
        };
        Self {
            significand,
            exponent,
            taken: 0,
        }
    }

    /// The next 64 binary digits of p, as an integer.
    fn next_block(&mut self) -> u64 {
        self.taken += 1;
        let shift = self.exponent + WORD_BITS as i32 * self.taken;
        if shift < 0 {
            if shift.unsigned_abs() >= WORD_BITS {
                0
            } else {
                self.significand >> -shift
            }
        } else if shift.unsigned_abs() >= u128::BITS {
            0
        } else {
            // The bits above the low 64 belong to earlier blocks.
            (u128::from(self.significand) << shift) as u64
        }
    }
}

/// `true` with probability exactly `p`, for `p` in [0, 1].
///
/// A uniform real U < p is decided digit block by digit block: U's next 64
/// bits are compared with p's, and only a tie, of probability 2⁻⁶⁴, reads
/// further.  p's expansion is finite, so an exhausted p means U ≥ p.
///
/// # Panics
/// Panics if `p` is not in [0, 1].
fn bernoulli(rng: &mut (impl Rng + ?Sized), p: f64) -> bool {
    assert!((0.0..=1.0).contains(&p), "probability outside [0, 1]");
    if p == 1.0 {
        return true;
    }
    let mut digits = ProbabilityDigits::new(p);
    for _ in 0..PROBABILITY_BLOCKS {
        let u = rng.next_u64();
        let q = digits.next_block();
        if u != q {
            return u < q;
        }
    }
    // p's digits are exhausted and U's so far equal them, so U ≥ p.
    false
}

/// Sampling methods available on every generator.
///
/// ```
/// use entropy::rng::{Pcg64, Sample, Seedable};
///
/// let mut rng = Pcg64::seed_from_u64(42);
/// let die = rng.range(1, 7);
/// assert!((1..7).contains(&die));
/// let _coin = rng.bernoulli(0.3);
/// let x = rng.unit_f64();
/// assert!((0.0..1.0).contains(&x));
/// let mut deck: Vec<u8> = (0..52).collect();
/// rng.shuffle(&mut deck);
/// deck.sort_unstable();
/// assert_eq!(deck, (0..52).collect::<Vec<u8>>());
/// assert!(rng.normal().is_finite());
/// ```
pub trait Sample: Rng {
    /// A uniform integer in [0, `bound`), exactly.
    ///
    /// # Panics
    /// Panics if `bound` is 0.
    fn below(&mut self, bound: u64) -> u64 {
        lemire_below(|| self.next_u64(), bound, WORD_BITS)
    }

    /// A uniform integer in [`low`, `high`), exactly.
    ///
    /// # Panics
    /// Panics if the range is empty.
    fn range(&mut self, low: i64, high: i64) -> i64 {
        assert!(low < high, "empty range");
        let width = high.wrapping_sub(low) as u64;
        low.wrapping_add(self.below(width) as i64)
    }

    /// `true` with probability exactly `numerator`/`denominator`.
    ///
    /// # Panics
    /// Panics if `denominator` is 0 or `numerator` exceeds it.
    fn ratio(&mut self, numerator: u64, denominator: u64) -> bool {
        assert!(numerator <= denominator, "ratio above 1");
        self.below(denominator) < numerator
    }

    /// `true` with probability exactly `p`, for any `p` in [0, 1].
    ///
    /// # Panics
    /// Panics if `p` is not in [0, 1].
    fn bernoulli(&mut self, p: f64) -> bool {
        bernoulli(self, p)
    }

    /// A uniform double on the 2⁵³ multiples of 2⁻⁵³ in [0, 1), from the top
    /// 53 bits of one `next_u64`.  Unlike `next_f64`, which the geometric
    /// tests read and which has 32 bits, every representable spacing of the
    /// grid is reachable.
    fn unit_f64(&mut self) -> f64 {
        let grid_bits = f64::MANTISSA_DIGITS;
        (self.next_u64() >> (WORD_BITS - grid_bits)) as f64 * (1.0 / (1u64 << grid_bits) as f64)
    }

    /// A uniform real in [0, 1) rounded down to a double, so that every double
    /// x in [0, 1) occurs with probability equal to the gap between x and the
    /// next double: all 2⁶² of them are reachable, down to the smallest
    /// subnormal.
    ///
    /// The real U has 2^−(e+1) ≤ U < 2^−e with probability 2^−(e+1), so e is
    /// the number of leading zero bits of a stream of uniform words; given e,
    /// U's next 52 bits are uniform and are the significand.  Past 1 074 zero
    /// bits U rounds down to 0.
    fn unit_f64_dense(&mut self) -> f64 {
        let mut zeros = 0u32;
        let mut word = self.next_u64();
        while word == 0 {
            zeros += WORD_BITS;
            if zeros >= ZERO_BITS_TO_ZERO {
                return 0.0;
            }
            word = self.next_u64();
        }
        zeros += word.leading_zeros();
        if zeros >= ZERO_BITS_TO_ZERO {
            return 0.0;
        }
        let significand = self.next_u64() >> (WORD_BITS - FRACTION_BITS);
        // U lies in [2^−(zeros+1), 2^−zeros): biased exponent 1022 − zeros.
        if zeros < ZERO_BITS_TO_SUBNORMAL {
            let exponent = u64::from(ZERO_BITS_TO_SUBNORMAL - zeros);
            f64::from_bits((exponent << FRACTION_BITS) | significand)
        } else {
            // Subnormal: value = f · 2^−1074 with f < 2^52, whose leading 1
            // sits at bit 1073 − zeros; the lower bits are uniform.
            let top = ZERO_BITS_TO_ZERO - 1 - zeros;
            let fraction = (1u64 << top) | (significand >> (FRACTION_BITS - top));
            f64::from_bits(fraction)
        }
    }

    /// An exponential variate with mean 1: −ln U for U from
    /// [`Sample::unit_f64_dense`], redrawn when U = 0.  Small U, which the
    /// dense draw resolves down to the smallest subnormal, gives the tail up to
    /// about 744.
    fn exponential(&mut self) -> f64 {
        loop {
            let u = self.unit_f64_dense();
            if u > 0.0 {
                return -u.ln();
            }
        }
    }

    /// A standard normal variate by inversion: a random sign on Φ⁻¹(U/2) for
    /// U from [`Sample::unit_f64_dense`], so both tails reach |z| ≈ 38.49,
    /// the quantile of half the smallest subnormal.  Where halving U would
    /// round (odd subnormal significands, including 2⁻¹⁰⁷⁴ itself, whose half
    /// is 0), U/2 is carried as ln U − ln 2.  Each draw costs a Newton solve,
    /// a few microseconds.
    ///
    /// This samples the continuous normal law through a dense uniform, to the
    /// accuracy of Φ⁻¹; it does not promise any particular rounded law.
    fn normal(&mut self) -> f64 {
        loop {
            let u = self.unit_f64_dense();
            if u > 0.0 {
                let half = 0.5 * u;
                let z = if 2.0 * half == u {
                    crate::math::normal_quantile(half)
                } else {
                    crate::math::normal_quantile_ln(u.ln() - std::f64::consts::LN_2)
                };
                return if self.next_u32() & 1 == 0 { z } else { -z };
            }
        }
    }

    /// Fill `bytes` with generator output, four bytes of each `next_u32`
    /// little-endian; a final partial word supplies its low bytes.
    fn fill_bytes(&mut self, bytes: &mut [u8]) {
        let mut chunks = bytes.chunks_exact_mut(size_of::<u32>());
        for chunk in &mut chunks {
            chunk.copy_from_slice(&self.next_u32().to_le_bytes());
        }
        let rest = chunks.into_remainder();
        if !rest.is_empty() {
            let word = self.next_u32().to_le_bytes();
            rest.copy_from_slice(&word[..rest.len()]);
        }
    }

    /// Shuffle `items` uniformly over all orderings: the Fisher–Yates shuffle
    /// as R. Durstenfeld gave it ("Algorithm 235: Random permutation,"
    /// *Communications of the ACM* 7(7), p. 420, 1964), with exact indices.
    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }

    /// A uniformly chosen element, or `None` for an empty slice.
    fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            Some(&items[self.below(items.len() as u64) as usize])
        }
    }

    /// A uniformly chosen element, mutably, or `None` for an empty slice.
    fn choose_mut<'a, T>(&mut self, items: &'a mut [T]) -> Option<&'a mut T> {
        if items.is_empty() {
            None
        } else {
            let i = self.below(items.len() as u64) as usize;
            Some(&mut items[i])
        }
    }

    /// Put a uniform random ordered sample of `amount` elements (all of them
    /// if `amount` exceeds the length) at the front of `items`, and return it
    /// and the rest.  The first `amount` steps of a forward Fisher–Yates
    /// shuffle.
    fn partial_shuffle<'a, T>(
        &mut self,
        items: &'a mut [T],
        amount: usize,
    ) -> (&'a mut [T], &'a mut [T]) {
        let amount = amount.min(items.len());
        for i in 0..amount {
            let j = i + self.below((items.len() - i) as u64) as usize;
            items.swap(i, j);
        }
        items.split_at_mut(amount)
    }

    /// A uniform integer in [0, `bound`) for a 128-bit bound, by rejection
    /// from the smallest enclosing power of two: exact, and fewer than two
    /// draws of two words on average.
    ///
    /// # Panics
    /// Panics if `bound` is 0.
    fn below_u128(&mut self, bound: u128) -> u128 {
        assert!(bound > 0, "empty range");
        let bits = u128::BITS - (bound - 1).leading_zeros();
        let mask = if bits == u128::BITS {
            u128::MAX
        } else {
            (1u128 << bits) - 1
        };
        loop {
            let high = u128::from(self.next_u64()) << WORD_BITS;
            let v = (high | u128::from(self.next_u64())) & mask;
            if v < bound {
                return v;
            }
        }
    }

    /// `amount` distinct indices from 0 … `length` − 1, every such set equally
    /// likely and in uniformly random order; `None` if `amount` exceeds
    /// `length`.  Uses R. W. Floyd's algorithm (J. Bentley with R. Floyd,
    /// "Programming pearls: A sample of brilliance," *Communications of the
    /// ACM* 30(9), pp. 754–757, 1987) for small samples and a partial
    /// shuffle when the sample is at least half the range, then shuffles the
    /// result.
    fn sample_indices(&mut self, length: usize, amount: usize) -> Option<Vec<usize>> {
        if amount > length {
            return None;
        }
        let mut chosen = if amount * 2 >= length {
            let mut all: Vec<usize> = (0..length).collect();
            self.partial_shuffle(&mut all, amount);
            all.truncate(amount);
            all
        } else {
            let mut set = std::collections::HashSet::with_capacity(amount);
            let mut order = Vec::with_capacity(amount);
            for j in length - amount..length {
                let t = self.below(j as u64 + 1) as usize;
                let pick = if set.contains(&t) { j } else { t };
                set.insert(pick);
                order.push(pick);
            }
            order
        };
        self.shuffle(&mut chosen);
        Some(chosen)
    }

    /// `amount` distinct elements of `items`, uniformly, in random order;
    /// `None` if there are fewer than `amount`.
    fn sample<'a, T>(&mut self, items: &'a [T], amount: usize) -> Option<Vec<&'a T>> {
        self.sample_indices(items.len(), amount)
            .map(|indices| indices.into_iter().map(|i| &items[i]).collect())
    }

    /// `K` distinct elements of `items`, uniformly, in random order; `None` if
    /// there are fewer than `K`.
    fn sample_array<'a, T, const K: usize>(&mut self, items: &'a [T]) -> Option<[&'a T; K]> {
        let indices = self.sample_indices(items.len(), K)?;
        Some(std::array::from_fn(|k| &items[indices[k]]))
    }

    /// An element chosen with probability exactly proportional to its integer
    /// weight; `None` if `items` is empty or every weight is 0.
    fn choose_weighted<'a, T>(
        &mut self,
        items: &'a [T],
        weight: impl Fn(&T) -> u64,
    ) -> Option<&'a T> {
        let total: u128 = items.iter().map(|x| u128::from(weight(x))).sum();
        if total == 0 {
            return None;
        }
        let mut target = self.below_u128(total);
        for item in items {
            let w = u128::from(weight(item));
            if target < w {
                return Some(item);
            }
            target -= w;
        }
        unreachable!("the target lies below the total")
    }

    /// `amount` distinct elements drawn one after another, each with
    /// probability exactly proportional to its integer weight among those not
    /// yet drawn; `None` if fewer than `amount` have positive weight.
    fn sample_weighted<'a, T>(
        &mut self,
        items: &'a [T],
        amount: usize,
        weight: impl Fn(&T) -> u64,
    ) -> Option<Vec<&'a T>> {
        let mut remaining: Vec<(usize, u64)> = items
            .iter()
            .enumerate()
            .map(|(i, x)| (i, weight(x)))
            .filter(|&(_, w)| w > 0)
            .collect();
        if remaining.len() < amount {
            return None;
        }
        let mut chosen = Vec::with_capacity(amount);
        for _ in 0..amount {
            let total: u128 = remaining.iter().map(|&(_, w)| u128::from(w)).sum();
            let mut target = self.below_u128(total);
            let position = remaining
                .iter()
                .position(|&(_, w)| {
                    let w = u128::from(w);
                    if target < w {
                        true
                    } else {
                        target -= w;
                        false
                    }
                })
                .expect("the target lies below the total");
            chosen.push(&items[remaining.swap_remove(position).0]);
        }
        Some(chosen)
    }

    /// A uniformly chosen item of an iterator of unknown length, or `None` if
    /// it is empty: reservoir sampling (J. S. Vitter, "Random sampling with a
    /// reservoir," *ACM Transactions on Mathematical Software* 11(1),
    /// pp. 37–57, 1985, Algorithm R), one exact index per item.
    fn choose_from_iter<I: IntoIterator>(&mut self, iter: I) -> Option<I::Item> {
        let mut chosen = None;
        for (i, item) in iter.into_iter().enumerate() {
            if self.below(i as u64 + 1) == 0 {
                chosen = Some(item);
            }
        }
        chosen
    }

    /// `amount` distinct items of an iterator, uniformly, in random order,
    /// by Algorithm R; fewer if the iterator is shorter.
    fn sample_from_iter<I: IntoIterator>(&mut self, iter: I, amount: usize) -> Vec<I::Item> {
        let mut reservoir = Vec::with_capacity(amount);
        for (i, item) in iter.into_iter().enumerate() {
            if i < amount {
                reservoir.push(item);
            } else {
                let j = self.below(i as u64 + 1) as usize;
                if j < amount {
                    reservoir[j] = item;
                }
            }
        }
        self.shuffle(&mut reservoir);
        reservoir
    }
}

impl<R: Rng + ?Sized> Sample for R {}

#[cfg(test)]
mod tests {
    use super::{
        lemire_below, ProbabilityDigits, Sample, PROBABILITY_BLOCKS, WORD_BITS, ZERO_BITS_TO_ZERO,
    };
    use crate::rng::{CounterRng, Pcg64, Rng};

    /// At 12-bit words every bound from 1 to 4 096 gives each result exactly
    /// ⌊4 096/bound⌋ accepted words: the method is exactly uniform.
    #[test]
    fn lemire_is_exact_over_every_word() {
        const BITS: u32 = 12;
        let words = 1u64 << BITS;
        for bound in 1..=words {
            let mut preimages = vec![0u64; bound as usize];
            for x in 0..words {
                // One word, then a word that is never accepted if needed: a
                // rejected x shows as a second read.
                let mut reads = 0;
                let value = lemire_below(
                    || {
                        reads += 1;
                        if reads == 1 {
                            x
                        } else {
                            words - 1
                        }
                    },
                    bound,
                    BITS,
                );
                if reads == 1 {
                    preimages[value as usize] += 1;
                }
            }
            let each = words / bound;
            assert!(preimages.iter().all(|&c| c == each), "bound {bound}");
        }
    }

    #[test]
    fn ranges_cover_their_ends_only() {
        let mut rng = Pcg64::new(1, 1);
        let mut seen = [false; 7];
        for _ in 0..10_000 {
            let v = rng.range(-3, 4);
            assert!((-3..4).contains(&v));
            seen[(v + 3) as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
        assert_eq!(rng.below(1), 0);
        for _ in 0..1_000 {
            let v = rng.range(i64::MIN, i64::MAX);
            assert!(v < i64::MAX);
        }
    }

    /// The digits of p reassemble p exactly, including subnormals.
    #[test]
    fn probability_digits_reassemble() {
        for p in [
            0.5,
            0.1,
            1.0 / 3.0,
            2.0f64.powi(-60),
            f64::MIN_POSITIVE,
            0.999_999_999_999,
        ] {
            let mut d = ProbabilityDigits::new(p);
            let first = d.next_block();
            let second = d.next_block();
            // Upper bound check: the first two blocks give p to within 2^-128.
            let approx = first as f64 * 2f64.powi(-64) + second as f64 * 2f64.powi(-128);
            assert!((approx - p).abs() <= p * 1e-15 + 2f64.powi(-128), "{p}");
        }
        let mut half = ProbabilityDigits::new(0.5);
        assert_eq!(half.next_block(), 1 << 63);
        assert_eq!(half.next_block(), 0);
    }

    /// A counter's words decide Bernoulli draws by exact comparison.
    #[test]
    fn bernoulli_compares_words_with_the_probability() {
        // next_u64 of CounterRng(0) is 0 << 32 | 1 = 1: below every p > 2^-64.
        assert!(CounterRng::new(0).bernoulli(0.25));
        assert!(!CounterRng::new(0).bernoulli(0.0));
        assert!(CounterRng::new(0).bernoulli(1.0));
        let mut rng = Pcg64::new(7, 7);
        let n = 200_000;
        let hits = (0..n).filter(|_| rng.bernoulli(0.3)).count() as f64;
        let sd = (0.3 * 0.7 / n as f64).sqrt();
        assert!((hits / n as f64 - 0.3).abs() < 5.0 * sd);
        let mut ratios = Pcg64::new(8, 8);
        let hits = (0..n).filter(|_| ratios.ratio(1, 3)).count() as f64;
        assert!((hits / n as f64 - 1.0 / 3.0).abs() < 5.0 * (2.0 / 9.0 / n as f64).sqrt());
    }

    #[test]
    fn unit_f64_uses_53_bits() {
        struct Fixed(u64);
        impl Rng for Fixed {
            fn next_u32(&mut self) -> u32 {
                0
            }
            fn next_u64(&mut self) -> u64 {
                self.0
            }
        }
        assert_eq!(Fixed(u64::MAX).unit_f64(), 1.0 - 2f64.powi(-53));
        assert_eq!(Fixed(1 << 11).unit_f64(), 2f64.powi(-53));
        assert_eq!(Fixed((1 << 11) - 1).unit_f64(), 0.0);
    }

    #[test]
    fn fill_bytes_is_little_endian_words() {
        let mut bytes = [0u8; 6];
        CounterRng::new(0x0403_0201).fill_bytes(&mut bytes);
        assert_eq!(bytes, [1, 2, 3, 4, 2, 2]);
    }

    /// Every ordering of three items appears about equally often.
    #[test]
    fn shuffles_are_uniform_over_orderings() {
        let mut rng = Pcg64::new(3, 3);
        let mut counts = std::collections::HashMap::new();
        let n = 60_000;
        for _ in 0..n {
            let mut v = [0, 1, 2];
            rng.shuffle(&mut v);
            *counts.entry(v).or_insert(0u32) += 1;
        }
        assert_eq!(counts.len(), 6);
        let expected = n as f64 / 6.0;
        let chi: f64 = counts
            .values()
            .map(|&c| (f64::from(c) - expected).powi(2) / expected)
            .sum();
        assert!(chi < 25.0, "chi-square {chi} on 5 df");
        assert_eq!(rng.choose::<u8>(&[]), None);
        assert_eq!(rng.choose(&[9]), Some(&9));
    }

    /// A tie in one block is decided by the next: the draw is the exact
    /// comparison of the words' binary expansion with p's.
    #[test]
    fn bernoulli_ties_read_further() {
        struct Words(Vec<u64>);
        impl Rng for Words {
            fn next_u32(&mut self) -> u32 {
                0
            }
            fn next_u64(&mut self) -> u64 {
                self.0.remove(0)
            }
        }
        // 0.3·2⁻²⁰ has significant digits in both of its first two blocks.
        let p = 0.3 * 2f64.powi(-20);
        let mut d = ProbabilityDigits::new(p);
        let (q1, q2) = (d.next_block(), d.next_block());
        assert!(q2 > 0);
        assert!(Words(vec![q1, q2 - 1]).bernoulli(p));
        assert!(!Words(vec![q1, q2, 1]).bernoulli(p));
        assert!(!Words(vec![q1 + 1]).bernoulli(p));
        assert!(Words(vec![q1 - 1]).bernoulli(p));
        // Past p's last digit every block of p is 0, so U ≥ p.
        let last = ProbabilityDigits::new(2f64.powi(-1074));
        assert_eq!((last.exponent, last.significand), (-1074, 1));
    }

    /// A generator that replays `next_u64` words and answers every
    /// `next_u32` with a fixed word (the normal's sign).
    struct Words(Vec<u64>, u32);

    impl Rng for Words {
        fn next_u32(&mut self) -> u32 {
            self.1
        }
        fn next_u64(&mut self) -> u64 {
            self.0.remove(0)
        }
    }

    /// Words for [`Sample::unit_f64_dense`]: `zeros` zero bits, a one, then
    /// `significand_word` (whose top 52 bits are the significand).
    fn dense_words(zeros: u32, significand_word: u64) -> Vec<u64> {
        let mut words = vec![0u64; (zeros / WORD_BITS) as usize];
        words.push(1 << (WORD_BITS - 1 - zeros % WORD_BITS));
        words.push(significand_word);
        words
    }

    /// Leading zero bits that make U the smallest subnormal, 2⁻¹⁰⁷⁴.
    const SMALLEST_SUBNORMAL_ZEROS: u32 = ZERO_BITS_TO_ZERO - 1;

    /// A significand word whose top bit, the first significand bit, is set.
    const FIRST_SIGNIFICAND_BIT: u64 = 1 << (WORD_BITS - 1);

    /// Dense floats: exact on crafted words, including subnormals, and
    /// uniform in distribution.
    #[test]
    fn dense_floats_follow_the_word_stream() {
        const SAMPLES: usize = 200_000;
        /// Standard errors allowed for the sample means.
        const SIGMAS: f64 = 5.0;
        /// Allowed error of the normal's sample variance.
        const VARIANCE_TOLERANCE: f64 = 0.02;
        let dense = |zeros, word| Words(dense_words(zeros, word), 0).unit_f64_dense();
        // A leading 1 bit: U in [1/2, 1), significand from the second word.
        let grid = 2f64.powi(-(f64::MANTISSA_DIGITS as i32));
        assert_eq!(dense(0, u64::MAX), 1.0 - grid);
        assert_eq!(dense(0, 0), 0.5);
        // Three zero bits then a one: [1/16, 1/8).
        assert_eq!(dense(3, 0), 0.0625);
        assert_eq!(dense(SMALLEST_SUBNORMAL_ZEROS, 0), f64::from_bits(1));
        // 1 041 zero bits: U in [2^−1042, 2^−1041), whose floor is 2^−1042.
        let zeros = 1_041;
        assert_eq!(dense(zeros, 0), 2f64.powi(-(zeros as i32) - 1));
        let all_zero = vec![0; PROBABILITY_BLOCKS as usize];
        assert_eq!(Words(all_zero, 0).unit_f64_dense(), 0.0);

        let mut rng = Pcg64::new(11, 11);
        let n = SAMPLES as f64;
        let uniform_variance = 1.0 / 12.0;
        let mean: f64 = (0..SAMPLES).map(|_| rng.unit_f64_dense()).sum::<f64>() / n;
        assert!((mean - 0.5).abs() < SIGMAS * (uniform_variance / n).sqrt());
        let e: f64 = (0..SAMPLES).map(|_| rng.exponential()).sum::<f64>() / n;
        assert!((e - 1.0).abs() < SIGMAS / n.sqrt());
        let z: Vec<f64> = (0..SAMPLES).map(|_| rng.normal()).collect();
        let zm = z.iter().sum::<f64>() / n;
        let zv = z.iter().map(|x| (x - zm).powi(2)).sum::<f64>() / n;
        assert!(
            zm.abs() < SIGMAS / n.sqrt() && (zv - 1.0).abs() < VARIANCE_TOLERANCE,
            "{zm} {zv}"
        );
    }

    /// Both tails of the normal and the exponential's far tail are reachable,
    /// down to the smallest subnormal U.
    #[test]
    fn variates_reach_their_far_tails() {
        /// Relative tolerance where Φ is still a normal double.
        const MODERATE_TAIL: f64 = 1e-9;
        /// Tolerance on Φ⁻¹ against 50-digit references (mpmath).
        const FAR_TAIL: f64 = 2e-13;
        /// (significand f, Φ⁻¹(f·2⁻¹⁰⁷⁵)), the normal for U = f·2⁻¹⁰⁷⁴.  f = 1
        /// and 3 cannot be halved exactly; f = 2 can.
        const SUBNORMAL_QUANTILES: [(u64, f64); 3] = [
            (1, -38.485_408_335_567_34),
            (2, -38.467_405_617_144_35),
            (3, -38.456_870_800_437_05),
        ];
        let sign = |negative: bool| u32::from(negative);

        // 200 zero bits: U = 2^−201, so the exponential gives 201·ln 2 and the
        // normal ±Φ⁻¹(2^−202).
        let zeros = 200;
        let exponent = f64::from(zeros + 1);
        let tiny = || dense_words(zeros, 0);
        let e = Words(tiny(), 0).exponential();
        assert!(
            (e - exponent * std::f64::consts::LN_2).abs() < MODERATE_TAIL,
            "{e}"
        );
        let low = Words(tiny(), sign(false)).normal();
        let high = Words(tiny(), sign(true)).normal();
        let q = crate::math::normal_cdf(low);
        assert!(
            (q / 2f64.powf(-exponent - 1.0) - 1.0).abs() < MODERATE_TAIL,
            "{low}: {q}"
        );
        assert_eq!(high, -low);

        // U = f·2⁻¹⁰⁷⁴: a leading 1 at 1 073 − ⌊log₂ f⌋ zeros and f's lower
        // bit, if any, as the first significand bit.
        for (f, want) in SUBNORMAL_QUANTILES {
            let top = f.ilog2();
            let significand_word = if f & 1 == 1 && top > 0 {
                FIRST_SIGNIFICAND_BIT
            } else {
                0
            };
            let words = || dense_words(SMALLEST_SUBNORMAL_ZEROS - top, significand_word);
            assert_eq!(Words(words(), 0).unit_f64_dense(), f64::from_bits(f));
            let z = Words(words(), sign(false)).normal();
            assert!((z - want).abs() < FAR_TAIL * want.abs(), "f = {f}: {z}");
            assert_eq!(Words(words(), sign(true)).normal(), -z);
        }
    }

    /// Every 2-subset of 5 appears about equally often, in both orders.
    #[test]
    fn sampled_indices_are_uniform_ordered_subsets() {
        let mut rng = Pcg64::new(21, 21);
        for amount in [2usize, 4] {
            let mut counts = std::collections::HashMap::new();
            let n = 60_000;
            for _ in 0..n {
                let s = rng.sample_indices(5, amount).unwrap();
                let mut sorted = s.clone();
                sorted.sort_unstable();
                sorted.dedup();
                assert_eq!(sorted.len(), amount);
                *counts.entry(s).or_insert(0u32) += 1;
            }
            // Ordered selections: 5·4 = 20 for 2, 5·4·3·2 = 120 for 4.
            let cells = if amount == 2 { 20 } else { 120 };
            assert_eq!(counts.len(), cells, "amount {amount}");
            let e = n as f64 / cells as f64;
            let chi: f64 = counts
                .values()
                .map(|&c| (f64::from(c) - e).powi(2) / e)
                .sum();
            assert!(
                chi < (cells as f64) + 6.0 * (2.0 * cells as f64).sqrt(),
                "{chi}"
            );
        }
        assert!(rng.sample_indices(3, 4).is_none());
        assert_eq!(rng.sample_indices(0, 0), Some(vec![]));
        let items = [10, 20, 30];
        let picked: [&i32; 3] = rng.sample_array(&items).unwrap();
        let mut values: Vec<i32> = picked.iter().map(|&&v| v).collect();
        values.sort_unstable();
        assert_eq!(values, items);
        assert_eq!(rng.sample(&items, 2).unwrap().len(), 2);
    }

    #[test]
    fn weighted_choices_follow_their_weights() {
        let mut rng = Pcg64::new(31, 31);
        let items = [(1u64, 'a'), (0, 'b'), (3, 'c')];
        let n = 80_000;
        let c = (0..n)
            .filter(|_| rng.choose_weighted(&items, |x| x.0).unwrap().1 == 'c')
            .count() as f64;
        assert!((c / n as f64 - 0.75).abs() < 5.0 * (0.1875 / n as f64).sqrt());
        assert!(rng.choose_weighted(&items[1..2], |x| x.0).is_none());
        let both = rng.sample_weighted(&items, 2, |x| x.0).unwrap();
        assert!(both.iter().all(|x| x.1 != 'b') && both[0].1 != both[1].1);
        assert!(rng.sample_weighted(&items, 3, |x| x.0).is_none());
        // Huge weights sum in 128 bits.
        let heavy = [u64::MAX, u64::MAX];
        assert!(rng.choose_weighted(&heavy, |&w| w).is_some());
    }

    #[test]
    fn iterator_choices_are_uniform() {
        let mut rng = Pcg64::new(41, 41);
        let mut counts = [0u32; 6];
        let n = 60_000;
        for _ in 0..n {
            counts[rng.choose_from_iter(0..6).unwrap()] += 1;
        }
        let e = n as f64 / 6.0;
        let chi: f64 = counts.iter().map(|&c| (f64::from(c) - e).powi(2) / e).sum();
        assert!(chi < 25.0, "{chi}");
        assert_eq!(rng.choose_from_iter(std::iter::empty::<u8>()), None);
        let s = rng.sample_from_iter(0..100, 10);
        let mut d = s.clone();
        d.sort_unstable();
        d.dedup();
        assert_eq!(d.len(), 10);
        assert_eq!(rng.sample_from_iter(0..3, 10).len(), 3);
        let mut v = [1, 2, 3, 4, 5];
        let (front, back) = rng.partial_shuffle(&mut v, 2);
        assert_eq!((front.len(), back.len()), (2, 3));
        *rng.choose_mut(&mut v).unwrap() = 0;
        assert!(v.contains(&0));
        for _ in 0..1_000 {
            assert!(rng.below_u128(u128::MAX) < u128::MAX);
            assert!(rng.below_u128(3) < 3);
        }
    }
}
