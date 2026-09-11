//! Mersenne Twister MT19937 pseudorandom number generator.
//!
//! The standard 32-bit PRNG by M. Matsumoto and T. Nishimura, "Mersenne
//! twister: A 623-dimensionally equidistributed uniform pseudo-random number
//! generator", *ACM Transactions on Modeling and Computer Simulation* 8(1),
//! pp. 3–30, January 1998.  Period = 2^19937 − 1.
//!
//! The recurrence and the tempering are those of §2.1 of the paper.  Seeding
//! is `init_genrand` from the authors' reference code `mt19937ar.c`, whose
//! initialization, "improved 2002/1/26", replaced the multiplier 69069 of the
//! paper's Appendix C `sgenrand` with 1812433253.  Like `genrand_int32` there,
//! [`Mt19937`] twists all 624 words at once.
//!
//! # References
//! * M. Matsumoto and T. Nishimura, 1998, as above.
//!   [pubs/matsumoto-nishimura-1998-mersenne-twister.pdf]
//! * M. Matsumoto and T. Nishimura, `mt19937ar.c`, and `mt19937ar.out`, the
//!   output its `main` prints.  [pubs/mt19937ar.c] [pubs/mt19937ar.out]
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
    /// Initialise with a 32-bit seed, as `init_genrand` in `mt19937ar.c` does.
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

/// `init_genrand` from `mt19937ar.c`: the state array for a 32-bit seed.
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

    /// `init_genrand(19650218)` then `genrand_int32`, the battery's seed.
    /// 19650218 is the constant `init_by_array` seeds with internally; the
    /// `mt19937ar.out` table shipped with mt19937ar.c is `init_by_array`
    /// output, so it is not the source of these values (the last test below
    /// pins that table).  They first came from an independent replica of
    /// Matsumoto & Nishimura's reference `init_genrand`/`genrand_int32`; the
    /// functions compiled from [pubs/mt19937ar.c] agree for 5000 outputs, and
    /// libc++ `std::mt19937(19650218)` agrees.
    #[test]
    fn known_output_seed_19650218() {
        let mut rng = Mt19937::new(19650218);
        let expected = [2325592414u32, 482149846, 4177211283, 3872387439, 1663027210];
        for &exp in &expected {
            assert_eq!(rng.next_u32(), exp, "MT19937 output mismatch");
        }
    }

    /// The canonical default seed 5489: the seed mt19937ar.c's
    /// `genrand_int32` falls back to when `init_genrand` was never called, and
    /// the default of C++ `std::mt19937`.  The first ten outputs come from an
    /// independent replica of the reference `init_genrand`/`genrand_int32`
    /// (cross-checked by running CPython's C twister from the replica's
    /// initial state, against libc++ `std::mt19937`, and against the functions
    /// compiled from [pubs/mt19937ar.c], which agree for 5000 outputs).  The
    /// 10 000th output is the value the C++ standard ([rand.predef]) requires
    /// of a default-constructed `mt19937`.
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

    /// The check output that ships with the reference code: `main` in
    /// [pubs/mt19937ar.c] seeds with `init_by_array({0x123, 0x234, 0x345,
    /// 0x456}, 4)` and prints 1000 `genrand_int32` outputs, the first block of
    /// [pubs/mt19937ar.out].  `init_by_array` is transcribed below because the
    /// crate does not expose it; `init_genrand`, the twist and the tempering
    /// are the crate's own.
    #[test]
    fn init_by_array_reproduces_mt19937ar_out() {
        const INIT_KEY: [u32; 4] = [0x123, 0x234, 0x345, 0x456];
        const FIRST_FIVE: [u32; 5] = [
            1_067_595_299,
            955_945_823,
            477_289_528,
            4_107_218_783,
            4_228_976_476,
        ];
        const OUTPUTS_996_TO_1000: [u32; 5] = [
            2_643_151_863,
            3_896_204_135,
            2_416_995_901,
            1_397_735_321,
            3_460_025_646,
        ];
        let mut rng = init_by_array(&INIT_KEY);
        let outputs: Vec<u32> = (0..1000).map(|_| rng.next_u32()).collect();
        assert_eq!(outputs[..5], FIRST_FIVE);
        assert_eq!(outputs[995..], OUTPUTS_996_TO_1000);
    }

    /// `init_by_array` from mt19937ar.c, as revised there on 2004/2/26.
    fn init_by_array(key: &[u32]) -> Mt19937 {
        let mut mt = init_genrand(19_650_218);
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..N.max(key.len()) {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1_664_525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..N - 1 {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1_566_083_941))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
        }
        mt[0] = UPPER_MASK; // MSB is 1, assuring a non-zero initial array
        Mt19937 { mt, idx: N }
    }
}
