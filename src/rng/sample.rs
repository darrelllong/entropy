//! Exact sampling from a generator's words: integers in a range, Bernoulli
//! trials, floats, shuffles and byte strings.
//!
//! Every method here is exact with respect to the generator's words being
//! independent and uniform: an integer range gets exactly equal
//! probabilities, a Bernoulli trial exactly its probability.  None rounds a
//! modulus or truncates a float.  Each is value-stable: the same words give
//! the same result in every release, and the tests pin known answers.

use super::Rng;

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
    assert!((1..=64).contains(&bits), "word width");
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
        let biased = ((bits >> 52) & 0x7ff) as i32;
        let fraction = bits & ((1u64 << 52) - 1);
        let (significand, exponent) = if biased == 0 {
            (fraction, -1074)
        } else {
            (fraction | (1u64 << 52), biased - 1075)
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
        let shift = self.exponent + 64 * self.taken;
        if shift < 0 {
            if -shift >= 64 {
                0
            } else {
                self.significand >> -shift
            }
        } else if shift >= 128 {
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
    for _ in 0..18 {
        let u = rng.next_u64();
        let q = digits.next_block();
        if u != q {
            return u < q;
        }
    }
    // Eighteen blocks cover all 1 074 fractional bits of a double.
    false
}

/// Sampling methods available on every generator.
pub trait Sample: Rng {
    /// A uniform integer in [0, `bound`), exactly.
    ///
    /// # Panics
    /// Panics if `bound` is 0.
    fn below(&mut self, bound: u64) -> u64 {
        lemire_below(|| self.next_u64(), bound, 64)
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
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
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
            zeros += 64;
            if zeros >= 1_074 {
                return 0.0;
            }
            word = self.next_u64();
        }
        zeros += word.leading_zeros();
        if zeros >= 1_074 {
            return 0.0;
        }
        let significand = self.next_u64() >> 12;
        // U lies in [2^−(zeros+1), 2^−zeros): biased exponent 1022 − zeros.
        if zeros < 1_022 {
            let exponent = u64::from(1_022 - zeros);
            f64::from_bits((exponent << 52) | significand)
        } else {
            // Subnormal: value = f · 2^−1074 with f < 2^52, whose leading 1
            // sits at bit 1073 − zeros; the lower bits are uniform.
            let top = 1073 - zeros;
            let fraction = (1u64 << top) | (significand >> (52 - top));
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
    /// U from [`Sample::unit_f64_dense`], so both tails reach the extremes a
    /// double probability allows (|z| near 38).  Each draw costs a Newton
    /// solve, a few microseconds.
    fn normal(&mut self) -> f64 {
        loop {
            let u = self.unit_f64_dense();
            if u > 0.0 {
                let z = crate::math::normal_quantile(0.5 * u);
                return if self.next_u32() & 1 == 0 { z } else { -z };
            }
        }
    }

    /// Fill `bytes` with generator output, four bytes of each `next_u32`
    /// little-endian; a final partial word supplies its low bytes.
    fn fill_bytes(&mut self, bytes: &mut [u8]) {
        let mut chunks = bytes.chunks_exact_mut(4);
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
}

impl<R: Rng + ?Sized> Sample for R {}

#[cfg(test)]
mod tests {
    use super::{lemire_below, ProbabilityDigits, Sample};
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

    /// Dense floats: exact on crafted words, including subnormals, and
    /// uniform in distribution.
    #[test]
    fn dense_floats_follow_the_word_stream() {
        struct Words(Vec<u64>);
        impl Rng for Words {
            fn next_u32(&mut self) -> u32 {
                0
            }
            fn next_u64(&mut self) -> u64 {
                self.0.remove(0)
            }
        }
        // A leading 1 bit: U in [1/2, 1), significand from the second word.
        assert_eq!(
            Words(vec![1 << 63, u64::MAX]).unit_f64_dense(),
            1.0 - 2f64.powi(-53)
        );
        assert_eq!(Words(vec![1 << 63, 0]).unit_f64_dense(), 0.5);
        // Three zero bits then a one: [1/16, 1/8).
        assert_eq!(Words(vec![1 << 60, 0]).unit_f64_dense(), 0.0625);
        // 1 073 zero bits (16 words and 49 bits): the smallest subnormal.
        let mut v = vec![0; 16];
        v.push(1 << 14);
        v.push(0);
        assert_eq!(Words(v).unit_f64_dense(), f64::from_bits(1));
        // 1 041 zero bits: U in [2^−1042, 2^−1041), whose floor is 2^−1042.
        let mut v = vec![0; 16];
        v.push(1 << 46);
        v.push(0);
        assert_eq!(Words(v).unit_f64_dense(), 2f64.powi(-1042));
        assert_eq!(Words(vec![0; 17]).unit_f64_dense(), 0.0);

        let mut rng = Pcg64::new(11, 11);
        let n = 200_000;
        let mean: f64 = (0..n).map(|_| rng.unit_f64_dense()).sum::<f64>() / n as f64;
        assert!((mean - 0.5).abs() < 5.0 * (1.0 / 12.0 / n as f64).sqrt());
        let e: f64 = (0..n).map(|_| rng.exponential()).sum::<f64>() / n as f64;
        assert!((e - 1.0).abs() < 5.0 / (n as f64).sqrt());
        let z: Vec<f64> = (0..n).map(|_| rng.normal()).collect();
        let zm = z.iter().sum::<f64>() / n as f64;
        let zv = z.iter().map(|x| (x - zm).powi(2)).sum::<f64>() / n as f64;
        assert!(
            zm.abs() < 5.0 / (n as f64).sqrt() && (zv - 1.0).abs() < 0.02,
            "{zm} {zv}"
        );
    }

    /// Both tails of the normal and the exponential's far tail are reachable.
    #[test]
    fn variates_reach_their_far_tails() {
        struct Words(Vec<u64>, u32);
        impl Rng for Words {
            fn next_u32(&mut self) -> u32 {
                self.1
            }
            fn next_u64(&mut self) -> u64 {
                self.0.remove(0)
            }
        }
        // 200 zero bits: U = 2^−201, so the exponential gives 201·ln 2 and the
        // normal ±Φ⁻¹(2^−202).
        let tiny = || {
            let mut v = vec![0u64; 3];
            v.push(1 << 55);
            v.push(0);
            v
        };
        let e = Words(tiny(), 0).exponential();
        assert!((e - 201.0 * std::f64::consts::LN_2).abs() < 1e-9, "{e}");
        let low = Words(tiny(), 0).normal();
        let high = Words(tiny(), 1).normal();
        let q = crate::math::normal_cdf(low);
        assert!((q / 2f64.powi(-202) - 1.0).abs() < 1e-9, "{low}: {q}");
        assert_eq!(high, -low);
    }
}
