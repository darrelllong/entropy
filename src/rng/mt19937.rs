//! Mersenne Twister MT19937 pseudorandom number generator.
//!
//! The standard 32-bit PRNG by M. Matsumoto and T. Nishimura, "Mersenne
//! twister: A 623-dimensionally equidistributed uniform pseudo-random number
//! generator", *ACM Transactions on Modeling and Computer Simulation* 8(1),
//! pp. 3–30, January 1998.  Period = 2^19937 − 1.
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
    /// Initialise with a 32-bit seed.
    pub fn new(seed: u32) -> Self {
        let mut mt = [0u32; N];
        mt[0] = seed;
        for i in 1..N {
            mt[i] = 1_812_433_253_u32
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        let mut rng = Self { mt, idx: N };
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

    // Reference output from the original mt19937ar.c (Matsumoto & Nishimura).
    // Uses the single-integer seed path: `init_genrand(19650218)`.
    // Expected values taken directly from the mt19937ar.c reference output table.
    #[test]
    fn known_output_seed_19650218() {
        let mut rng = Mt19937::new(19650218);
        // First 5 known outputs from mt19937ar.c with seed=19650218
        let expected = [2325592414u32, 482149846, 4177211283, 3872387439, 1663027210];
        for &exp in &expected {
            assert_eq!(rng.next_u32(), exp, "MT19937 output mismatch");
        }
    }
}
