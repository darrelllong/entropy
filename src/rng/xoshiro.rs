//! Xoshiro256 and Xoroshiro128 — David Blackman & Sebastiano Vigna (2018).
//!
//! Both generators use a linear engine (xor/shift/rotate recurrence) combined
//! with a non-linear *scrambler* (starstar: multiply–rotate–multiply).
//!
//! | Generator    | State  | Period  | Use case                        |
//! |--------------|--------|---------|---------------------------------|
//! | Xoshiro256   | 256 bit| 2²⁵⁶−1 | General purpose, large period   |
//! | Xoroshiro128 | 128 bit| 2¹²⁸−1 | Tight memory, comparable speed  |
//!
//! Neither is cryptographically secure, and both carry the linear
//! dependencies of their underlying LFSR-style state updates, detectable by
//! linear-complexity tests at extreme depth.
//!
//! # References
//! * D. Blackman and S. Vigna, "Scrambled Linear Pseudorandom Number
//!   Generators", *ACM Transactions on Mathematical Software* 47(4), 2021.
//!   DOI: 10.1145/3460772.  [BIB.md: blackman2021xoshiro]
//!   [pubs/blackman-vigna-2021-scrambled-linear-prngs.pdf] (arXiv:1805.01407v3)
//!   [Figs. 1 and 4 give the xoroshiro128 and xoshiro256 code, Table 2 their
//!   engine parameters (A, B, C) = (24, 16, 37) and (A, B) = (17, 45), and
//!   Table 3 the `**` scrambler's (S, R, T) = (5, 7, 9)]
//!   [the paper's (A, B, C) = (24, 16, 37) replaced the (55, 14, 36) of the
//!   2016 xoroshiro128+; the two parameter sets are different generators]
//!
//! # Author
//! David Blackman and Sebastiano Vigna (algorithm).

use super::{
    jump::{apply, characteristic_polynomial, jump_polynomial},
    OsRng, Rng,
};
use std::sync::OnceLock;

// ── Xoshiro256 ────────────────────────────────────────────────────────────────

/// 256-bit xoshiro256 generator — starstar scrambler, 64-bit output.
///
/// Period: 2²⁵⁶ − 1.  Do not seed with all-zeros.
pub struct Xoshiro256 {
    s: [u64; 4],
}

impl Xoshiro256 {
    /// Construct from four 64-bit seeds (must not all be zero).
    #[must_use]
    pub fn new(s0: u64, s1: u64, s2: u64, s3: u64) -> Self {
        assert!(
            s0 | s1 | s2 | s3 != 0,
            "xoshiro256: all-zero seed forbidden"
        );
        Self {
            s: [s0, s1, s2, s3],
        }
    }

    /// Construct from 256 bits drawn from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        loop {
            let s = [os.next_u64(), os.next_u64(), os.next_u64(), os.next_u64()];
            if s[0] | s[1] | s[2] | s[3] != 0 {
                return Self { s };
            }
        }
    }

    #[inline]
    fn step(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        advance256(&mut self.s);
        result
    }

    /// Advance by 2^`exponent` steps, for `exponent` up to the 256 bits of
    /// state, through the jump polynomial derived in [`super::jump`].
    ///
    /// Jumping is the same forward motion as calling the generator, so a
    /// caller who jumps k times from one seed holds the k-th segment of one
    /// stream, whatever order the segments are taken in.  The cost does not
    /// grow with the exponent.
    ///
    /// # Panics
    /// Panics if `exponent` exceeds 256.
    pub fn jump_pow2(&mut self, exponent: u32) {
        assert!(exponent <= 256, "xoshiro256: the state is 256 bits");
        static CHARACTERISTIC: OnceLock<Vec<u64>> = OnceLock::new();
        let characteristic = CHARACTERISTIC.get_or_init(|| characteristic_polynomial(advance256));
        apply(
            &mut self.s,
            &jump_polynomial(characteristic, exponent),
            advance256,
        );
    }

    /// The generator advanced by `index`·2¹²⁸ steps: stream `index` of the
    /// 2¹²⁸ segments of length 2¹²⁸ this partition defines.
    #[must_use]
    pub fn stream(&self, index: u64) -> Self {
        let mut stream = Self { s: self.s };
        for _ in 0..index {
            stream.jump_pow2(HALF_STATE_BITS);
        }
        stream
    }
}

/// Half the state's bits: the exponent of the default segment length.
const HALF_STATE_BITS: u32 = 128;

/// The linear part of xoshiro256's step, which the jump polynomial is derived
/// from and applied to.
fn advance256(s: &mut [u64; 4]) {
    let t = s[1] << 17;
    s[2] ^= s[0];
    s[3] ^= s[1];
    s[1] ^= s[2];
    s[0] ^= s[3];
    s[2] ^= t;
    s[3] = s[3].rotate_left(45);
}

impl Default for Xoshiro256 {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for Xoshiro256 {
    // Return the upper 32 bits (marginally higher avalanche quality).
    fn next_u32(&mut self) -> u32 {
        (self.step() >> 32) as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.step()
    }
}

// ── Xoroshiro128 ──────────────────────────────────────────────────────────────

/// 128-bit xoroshiro128 generator — starstar scrambler, 64-bit output.
///
/// Period: 2¹²⁸ − 1.  Half the state of xoshiro256; measured somewhat slower in this crate's
/// benchmarks (see BENCHMARKS.md) despite the smaller footprint.
/// Do not seed with all-zeros.
pub struct Xoroshiro128 {
    s: [u64; 2],
}

impl Xoroshiro128 {
    /// Construct from two 64-bit seeds (must not both be zero).
    #[must_use]
    pub fn new(s0: u64, s1: u64) -> Self {
        assert!(s0 | s1 != 0, "xoroshiro128: all-zero seed forbidden");
        Self { s: [s0, s1] }
    }

    /// Construct from 128 bits drawn from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        loop {
            let s = [os.next_u64(), os.next_u64()];
            if s[0] | s[1] != 0 {
                return Self { s };
            }
        }
    }

    #[inline]
    fn step(&mut self) -> u64 {
        let result = self.s[0].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        advance128(&mut self.s);
        result
    }

    /// Advance by 2^`exponent` steps, as [`Xoshiro256::jump_pow2`].
    ///
    /// # Panics
    /// Panics if `exponent` exceeds 128.
    pub fn jump_pow2(&mut self, exponent: u32) {
        assert!(exponent <= 128, "xoroshiro128: the state is 128 bits");
        static CHARACTERISTIC: OnceLock<Vec<u64>> = OnceLock::new();
        let characteristic = CHARACTERISTIC.get_or_init(|| characteristic_polynomial(advance128));
        apply(
            &mut self.s,
            &jump_polynomial(characteristic, exponent),
            advance128,
        );
    }

    /// The generator advanced by `index`·2⁶⁴ steps: stream `index` of the
    /// 2⁶⁴ segments of length 2⁶⁴ this partition defines.
    #[must_use]
    pub fn stream(&self, index: u64) -> Self {
        let mut stream = Self { s: self.s };
        for _ in 0..index {
            stream.jump_pow2(HALF_STATE_BITS_128);
        }
        stream
    }
}

/// Half xoroshiro128's state bits: the exponent of its segment length.
const HALF_STATE_BITS_128: u32 = 64;

/// The linear part of xoroshiro128's step.
fn advance128(s: &mut [u64; 2]) {
    let s1x = s[1] ^ s[0];
    s[0] = s[0].rotate_left(24) ^ s1x ^ (s1x << 16); // a = 24, b = 16
    s[1] = s1x.rotate_left(37); // c = 37
}

impl Default for Xoroshiro128 {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for Xoroshiro128 {
    fn next_u32(&mut self) -> u32 {
        (self.step() >> 32) as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.step()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    /// A jump of 2^k is exactly 2^k steps, checked at k = 20 against a
    /// million steps of the generator itself, for both generators.  The jump
    /// polynomial is derived from the generator, so this checks the whole
    /// derivation: Berlekamp–Massey, the squaring chain and the application.
    #[test]
    fn a_jump_is_the_steps_it_stands_for() {
        const EXPONENT: u32 = 20;
        let mut stepped = xoshiro256_fixed();
        for _ in 0..1u64 << EXPONENT {
            let _ = stepped.next_u64();
        }
        let mut jumped = xoshiro256_fixed();
        jumped.jump_pow2(EXPONENT);
        assert_eq!(jumped.s, stepped.s, "xoshiro256");

        let mut stepped = Xoroshiro128::new(0x1234_5678_9abc_def0, 0x0fed_cba9_8765_4321);
        for _ in 0..1u64 << EXPONENT {
            let _ = stepped.next_u64();
        }
        let mut jumped = Xoroshiro128::new(0x1234_5678_9abc_def0, 0x0fed_cba9_8765_4321);
        jumped.jump_pow2(EXPONENT);
        assert_eq!(jumped.s, stepped.s, "xoroshiro128");
    }

    /// Two jumps of 2^k are one jump of 2^(k+1), at exponents no walk could
    /// reach, and a jump of 2^0 is one step.
    #[test]
    fn jumps_compose_as_powers_of_two() {
        for exponent in [0u32, 1, 63, 64, 127, 128, 255] {
            let mut twice = xoshiro256_fixed();
            twice.jump_pow2(exponent);
            twice.jump_pow2(exponent);
            let mut once = xoshiro256_fixed();
            once.jump_pow2(exponent + 1);
            assert_eq!(twice.s, once.s, "2^{exponent} twice");
        }
        let mut one = xoshiro256_fixed();
        one.jump_pow2(0);
        let mut stepped = xoshiro256_fixed();
        let _ = stepped.next_u64();
        assert_eq!(one.s, stepped.s);
    }

    /// Streams are the segments of one stream: stream k is k jumps, they
    /// differ from one another, and stream 0 is the seed itself.
    #[test]
    fn streams_partition_one_stream() {
        let base = xoshiro256_fixed();
        assert_eq!(base.stream(0).s, base.s);
        let mut jumped = xoshiro256_fixed();
        jumped.jump_pow2(128);
        assert_eq!(base.stream(1).s, jumped.s);
        let heads: Vec<u64> = (0..8).map(|k| base.stream(k).next_u64()).collect();
        let mut sorted = heads.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), heads.len(), "streams repeat: {heads:?}");
        let mut small = Xoroshiro128::new(1, 2);
        assert_eq!(small.stream(0).s, [1, 2]);
        assert_ne!(small.stream(3).s, small.stream(4).s);
        let _ = small.next_u64();
    }

    use super::*;

    /// xoshiro256** at the arbitrary nonzero state the equal-seed tests share.
    fn xoshiro256_fixed() -> Xoshiro256 {
        Xoshiro256::new(0xdead, 0xbeef, 0xcafe, 0xbabe)
    }

    /// xoroshiro128** at the arbitrary nonzero state the equal-seed test uses.
    fn xoroshiro128_fixed() -> Xoroshiro128 {
        Xoroshiro128::new(0xdead_beef, 0xcafe_babe)
    }

    // Known-answer test: first three outputs of xoshiro256** with state
    // {1, 2, 3, 4}.  In decimal they are 11520, 0, 1509978240.
    #[test]
    fn xoshiro256_reference() {
        let mut rng = Xoshiro256::new(1, 2, 3, 4);
        let expected: [u64; 3] = [0x2d00, 0x0000, 0x5a00_7080];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e, "xoshiro256** KAT mismatch");
        }
    }

    // Known-answer test: first three outputs of xoroshiro128** with state
    // {1, 2}.
    #[test]
    fn xoroshiro128_reference() {
        let mut rng = Xoroshiro128::new(1, 2);
        let expected: [u64; 3] = [0x1680, 0x16_c380_4380, 0x86b5_b3ad_0000_4380];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e, "xoroshiro128** KAT mismatch");
        }
    }

    #[test]
    fn xoshiro256_next_u32_high_bits() {
        let (mut a, mut b) = (xoshiro256_fixed(), xoshiro256_fixed());
        assert_eq!(a.next_u32(), (b.next_u64() >> 32) as u32);
    }

    #[test]
    fn zero_seed_rejected() {
        let r = std::panic::catch_unwind(|| Xoshiro256::new(0, 0, 0, 0));
        assert!(r.is_err());
    }

    #[test]
    fn xoshiro256_deterministic() {
        let (mut a, mut b) = (xoshiro256_fixed(), xoshiro256_fixed());
        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn xoroshiro128_deterministic() {
        let (mut a, mut b) = (xoroshiro128_fixed(), xoroshiro128_fixed());
        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
