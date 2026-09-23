//! Mersenne Twister MT19937 pseudorandom number generator.
//!
//! The standard 32-bit PRNG by M. Matsumoto and T. Nishimura, "Mersenne
//! twister: A 623-dimensionally equidistributed uniform pseudo-random number
//! generator", *ACM Transactions on Modeling and Computer Simulation* 8(1),
//! pp. 3–30, January 1998.  Period = 2^19937 − 1.
//!
//! The recurrence and the tempering are those of §2.1 of the paper.  The state
//! is seeded by the authors' 2002 initialisation,
//! mt\[i\] = 1812433253·(mt\[i−1\] ⊕ (mt\[i−1\] ≫ 30)) + i, not the
//! multiplier 69069 of the paper's Appendix C, and all 624 words are twisted
//! at once.  The C++ standard's `std::mt19937` uses the same seeding.
//!
//! # References
//! * M. Matsumoto and T. Nishimura, 1998, as above.
//!   [pubs/matsumoto-nishimura-1998-mersenne-twister.pdf]
//! * ISO/IEC 14882, [rand.predef]: the 10 000th output of a default-seeded
//!   `mt19937`.
//!
//! # Author
//! Makoto Matsumoto and Takuji Nishimura (1998).

use super::{
    jump::{advance_linear, annihilating_polynomial, Poly},
    streams::{Advance, Streams},
    Rng,
};
use std::sync::OnceLock;

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908_B0DF;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7FFF_FFFF;

/// MT19937 32-bit Mersenne Twister.
pub struct Mt19937 {
    mt: [u32; N],
    idx: usize,
}

impl Mt19937 {
    /// Initialise with a 32-bit seed by the authors' 2002 initialisation.
    pub fn new(seed: u32) -> Self {
        let mut rng = Self {
            mt: init_genrand(seed),
            idx: N,
        };
        rng.generate(); // pre-twist so the first output is fully conditioned
        rng
    }

    fn generate(&mut self) {
        twist(&mut self.mt);
        self.idx = 0;
    }
}

/// The recurrence applied to the whole array: the linear map one block of
/// [`N`] outputs advances the state by.
fn twist(mt: &mut [u32; N]) {
    for i in 0..N {
        let x = (mt[i] & UPPER_MASK) | (mt[(i + 1) % N] & LOWER_MASK);
        let xa = if x & 1 == 0 {
            x >> 1
        } else {
            (x >> 1) ^ MATRIX_A
        };
        mt[i] = mt[(i + M) % N] ^ xa;
    }
}

/// The array as 64-bit words for the jump machinery, two array words each.
const PACKED_WORDS: usize = N / 2;

/// Stream segments of MT19937: 2⁶⁴ outputs each.  The period is 2¹⁹⁹³⁷ − 1,
/// so the count of segments is not the limit of anything.
const SEGMENT_LOG2: u32 = 64;

fn pack(mt: &[u32; N]) -> [u64; PACKED_WORDS] {
    std::array::from_fn(|j| u64::from(mt[2 * j]) | u64::from(mt[2 * j + 1]) << 32)
}

fn unpack(packed: &[u64; PACKED_WORDS]) -> [u32; N] {
    std::array::from_fn(|i| (packed[i / 2] >> (32 * (i % 2))) as u32)
}

/// [`twist`] on the packed array: the block step the annihilating polynomial
/// is derived for and applied to.
fn twist_packed(packed: &mut [u64; PACKED_WORDS]) {
    let mut mt = unpack(packed);
    twist(&mut mt);
    *packed = pack(&mt);
}

/// The polynomial annihilating the block step, derived once.  The recurrence
/// has degree 19 937; the remaining 31 bits, the low bits of `mt[0]`, are
/// overwritten by a twist before anything reads them, and the factor
/// x³¹ the derivation adds covers them.
fn annihilator() -> &'static Poly {
    static POLY: OnceLock<Poly> = OnceLock::new();
    POLY.get_or_init(|| annihilating_polynomial(twist_packed))
}

impl Advance for Mt19937 {
    /// The outputs left in the current block are skipped one at a time, whole
    /// blocks by the polynomial, and the remainder one at a time again: at
    /// most 2·[`N`] ordinary steps beside the jump.
    fn advance(&mut self, steps: u128) {
        let in_block = (N - self.idx) as u128;
        if steps <= in_block {
            self.idx += steps as usize;
            return;
        }
        let past_block = steps - in_block;
        let (blocks, rest) = (past_block / N as u128, (past_block % N as u128) as usize);
        self.idx = N;
        if blocks > 0 {
            let mut packed = pack(&self.mt);
            advance_linear(&mut packed, annihilator(), blocks, 0, twist_packed);
            self.mt = unpack(&packed);
        }
        for _ in 0..rest {
            let _ = self.next_u32();
        }
    }
}

impl Streams for Mt19937 {
    /// Segment `index` of 2⁶⁴ outputs from this generator's position.
    fn stream(&self, index: u64) -> Self {
        let mut stream = Self {
            mt: self.mt,
            idx: self.idx,
        };
        stream.advance(u128::from(index) << SEGMENT_LOG2);
        stream
    }
}

/// The state array for a 32-bit seed:
/// mt[i] = 1812433253·(mt[i−1] ⊕ (mt[i−1] ≫ 30)) + i.
fn init_genrand(seed: u32) -> [u32; N] {
    let mut mt = [0u32; N];
    mt[0] = seed;
    for i in 1..N {
        mt[i] = 1_812_433_253_u32
            .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
            .wrapping_add(i as u32);
    }
    mt
}

impl Rng for Mt19937 {
    fn next_u32(&mut self) -> u32 {
        if self.idx >= N {
            self.generate();
        }
        let mut y = self.mt[self.idx];
        self.idx += 1;
        // Tempering, the paper's output transform (Matsumoto and Nishimura §2.2).
        y ^= y >> 11;
        y ^= (y << 7) & 0x9D2C_5680;
        y ^= (y << 15) & 0xEFC6_0000;
        y ^= y >> 18;
        y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An advance is the outputs it stands for, from mid-block and across
    /// many blocks, and a stream is an advance of index times the segment.
    #[test]
    fn advances_match_stepping() {
        for (skip, steps) in [
            (0u32, 0u128),
            (5, 1),
            (0, N as u128),
            (7, 1_000_003),
            (623, 2 * N as u128 + 1),
        ] {
            let mut stepped = Mt19937::new(5489);
            let mut jumped = Mt19937::new(5489);
            for _ in 0..skip {
                let _ = stepped.next_u32();
                let _ = jumped.next_u32();
            }
            for _ in 0..steps {
                let _ = stepped.next_u32();
            }
            jumped.advance(steps);
            let next: Vec<u32> = (0..3).map(|_| stepped.next_u32()).collect();
            let jumped_next: Vec<u32> = (0..3).map(|_| jumped.next_u32()).collect();
            assert_eq!(jumped_next, next, "skip {skip}, {steps} steps");
        }
        let base = Mt19937::new(1);
        let mut walked = Mt19937::new(1);
        walked.advance(3u128 << SEGMENT_LOG2);
        let mut third = base.stream(3);
        assert_eq!(third.next_u32(), walked.next_u32());
        assert_ne!(base.stream(1).next_u32(), base.stream(2).next_u32());
    }

    /// Seed 19650218: outputs of the reference generator, which C++
    /// `std::mt19937(19650218)` also produces.
    #[test]
    fn known_output_seed_19650218() {
        let mut rng = Mt19937::new(19650218);
        let expected = [2325592414u32, 482149846, 4177211283, 3872387439, 1663027210];
        for &exp in &expected {
            assert_eq!(rng.next_u32(), exp, "MT19937 output mismatch");
        }
    }

    /// The default seed 5489 of C++ `std::mt19937`: its first ten outputs, and
    /// the 10 000th output, which the C++ standard ([rand.predef]) requires.
    #[test]
    fn known_output_init_genrand_5489() {
        const DEFAULT_SEED: u32 = 5489;
        const OUTPUT_10000: u32 = 4_123_659_995;
        let expected: [u32; 10] = [
            3_499_211_612,
            581_869_302,
            3_890_346_734,
            3_586_334_585,
            545_404_204,
            4_161_255_391,
            3_922_919_429,
            949_333_985,
            2_715_962_298,
            1_323_567_403,
        ];
        let mut rng = Mt19937::new(DEFAULT_SEED);
        let got = expected.map(|_| rng.next_u32());
        assert_eq!(got, expected);
        for _ in expected.len()..9_999 {
            let _ = rng.next_u32();
        }
        assert_eq!(rng.next_u32(), OUTPUT_10000);
    }
}
