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
    jump::{advance_linear, annihilating_polynomial, Poly},
    streams::{Advance, Streams},
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
}

/// Stream segments of xoshiro256: 2¹²⁸ steps each, half the period's bits,
/// so there are 2¹²⁸ of them.
const SEGMENT_LOG2_256: u32 = 128;

/// The polynomial annihilating xoshiro256's update, derived once.
fn annihilator256() -> &'static Poly {
    static POLY: OnceLock<Poly> = OnceLock::new();
    POLY.get_or_init(|| annihilating_polynomial(advance256))
}

impl Advance for Xoshiro256 {
    /// Through the derived polynomial: a few hundred ordinary steps whatever
    /// `steps` is.
    fn advance(&mut self, steps: u128) {
        advance_linear(&mut self.s, annihilator256(), steps, 0, advance256);
    }
}

impl Streams for Xoshiro256 {
    /// Segment `index` of 2¹²⁸ steps from this generator's position: one
    /// exponentiation, however large the index.
    fn stream(&self, index: u64) -> Self {
        let mut stream = Self { s: self.s };
        advance_linear(
            &mut stream.s,
            annihilator256(),
            u128::from(index),
            SEGMENT_LOG2_256,
            advance256,
        );
        stream
    }
}

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
}

/// Stream segments of xoroshiro128: 2⁶⁴ steps each, so 2⁶⁴ of them.
const SEGMENT_LOG2_128: u32 = 64;

/// The polynomial annihilating xoroshiro128's update, derived once.
fn annihilator128() -> &'static Poly {
    static POLY: OnceLock<Poly> = OnceLock::new();
    POLY.get_or_init(|| annihilating_polynomial(advance128))
}

impl Advance for Xoroshiro128 {
    fn advance(&mut self, steps: u128) {
        advance_linear(&mut self.s, annihilator128(), steps, 0, advance128);
    }
}

impl Streams for Xoroshiro128 {
    /// Segment `index` of 2⁶⁴ steps from this generator's position.
    fn stream(&self, index: u64) -> Self {
        let mut stream = Self { s: self.s };
        advance_linear(
            &mut stream.s,
            annihilator128(),
            u128::from(index),
            SEGMENT_LOG2_128,
            advance128,
        );
        stream
    }
}

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
    /// An advance is exactly the steps it stands for, checked against a
    /// million and more steps of the generator itself, for both generators.
    /// The polynomial is derived from the generator, so this checks the whole
    /// derivation: Berlekamp–Massey, the reciprocal, the exponentiation and
    /// the application.
    #[test]
    fn an_advance_is_the_steps_it_stands_for() {
        for steps in [0u128, 1, 2, 1_000_003, 1 << 20] {
            let mut stepped = xoshiro256_fixed();
            for _ in 0..steps {
                let _ = stepped.next_u64();
            }
            let mut jumped = xoshiro256_fixed();
            jumped.advance(steps);
            assert_eq!(jumped.s, stepped.s, "xoshiro256, {steps} steps");

            let seed = (0x1234_5678_9abc_def0, 0x0fed_cba9_8765_4321);
            let mut stepped = Xoroshiro128::new(seed.0, seed.1);
            for _ in 0..steps {
                let _ = stepped.next_u64();
            }
            let mut jumped = Xoroshiro128::new(seed.0, seed.1);
            jumped.advance(steps);
            assert_eq!(jumped.s, stepped.s, "xoroshiro128, {steps} steps");
        }
    }

    /// Advances compose: 2^k twice is 2^(k+1) once, at counts no walk could
    /// reach, and a stream's start is an advance of index times the segment.
    #[test]
    fn advances_compose_and_streams_are_segments() {
        for exponent in [63u32, 64, 100, 126] {
            let mut twice = xoshiro256_fixed();
            twice.advance(1 << exponent);
            twice.advance(1 << exponent);
            let mut once = xoshiro256_fixed();
            once.advance(1 << (exponent + 1));
            assert_eq!(twice.s, once.s, "2^{exponent} twice");
        }
        let base = xoshiro256_fixed();
        assert_eq!(base.stream(0).s, base.s);
        // Segment 3 of xoroshiro128 is 3·2⁶⁴ steps, which fits a u128.
        let small = Xoroshiro128::new(1, 2);
        let mut walked = Xoroshiro128::new(1, 2);
        walked.advance(3u128 << SEGMENT_LOG2_128);
        assert_eq!(small.stream(3).s, walked.s);
        // Consecutive segments of xoshiro256 differ from one another.
        let heads: Vec<u64> = (0..8).map(|k| base.stream(k).next_u64()).collect();
        let mut sorted = heads.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), heads.len(), "streams repeat: {heads:?}");
        // And segment 2 is segment 1 advanced by one segment.
        let mut second = base.stream(1);
        second.advance(1u128 << 127);
        second.advance(1u128 << 127);
        assert_eq!(second.s, base.stream(2).s);
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
