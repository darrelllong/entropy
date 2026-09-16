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

use super::Rng;

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
        for i in 0..N {
            let x = (self.mt[i] & UPPER_MASK) | (self.mt[(i + 1) % N] & LOWER_MASK);
            let xa = if x & 1 == 0 {
                x >> 1
            } else {
                (x >> 1) ^ MATRIX_A
            };
            self.mt[i] = self.mt[(i + M) % N] ^ xa;
        }
        self.idx = 0;
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
        // Tempering
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
