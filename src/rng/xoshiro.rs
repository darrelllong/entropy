//! Xoshiro256 and Xoroshiro128 — David Blackman & Sebastiano Vigna (2018).
//!
//! Both generators use a linear engine (xor/shift/rotate recurrence) combined
//! with a non-linear *scrambler* (starstar: multiply–rotate–multiply).
//! They pass BigCrush, PractRand > 32 TiB, and NIST SP 800-22.
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
//! D. Blackman and S. Vigna, "Scrambled Linear Pseudorandom Number
//! Generators", *ACM Transactions on Mathematical Software* 47(4), 2021.
//! DOI: 10.1145/3460772.  [BIB.md: blackman2021xoshiro]
//!
//! # Author
//! David Blackman, Sebastiano Vigna (algorithm); Darrell Long (Rust port).

use super::{OsRng, Rng};

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
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }
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
        let s0 = self.s[0];
        let s1 = self.s[1];
        let result = s0.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let s1x = s1 ^ s0;
        self.s[0] = s0.rotate_left(24) ^ s1x ^ (s1x << 16); // a=24, b=16
        self.s[1] = s1x.rotate_left(37); // c=37
        result
    }
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
    use super::*;

    // Known-answer test: first three outputs of xoshiro256** with state
    // {1, 2, 3, 4}, cross-checked against an independent Python replica of
    // Vigna's C reference implementation.  (11520, 0, 1509978240 is the
    // well-known published prefix for this state.)
    #[test]
    fn xoshiro256_reference() {
        let mut rng = Xoshiro256::new(1, 2, 3, 4);
        let expected: [u64; 3] = [0x2d00, 0x0000, 0x5a00_7080];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e, "xoshiro256** KAT mismatch");
        }
    }

    // Known-answer test: first three outputs of xoroshiro128** with state
    // {1, 2}, cross-checked against an independent Python replica of Vigna's
    // C reference implementation.
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
        let mut a = Xoshiro256::new(0xdead, 0xbeef, 0xcafe, 0xbabe);
        let mut b = Xoshiro256::new(0xdead, 0xbeef, 0xcafe, 0xbabe);
        assert_eq!(a.next_u32(), (b.next_u64() >> 32) as u32);
    }

    #[test]
    fn zero_seed_rejected() {
        let r = std::panic::catch_unwind(|| Xoshiro256::new(0, 0, 0, 0));
        assert!(r.is_err());
    }

    #[test]
    fn xoshiro256_deterministic() {
        let mut a = Xoshiro256::new(0xdead, 0xbeef, 0xcafe, 0xbabe);
        let mut b = Xoshiro256::new(0xdead, 0xbeef, 0xcafe, 0xbabe);
        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn xoroshiro128_deterministic() {
        let mut a = Xoroshiro128::new(0xdead_beef, 0xcafe_babe);
        let mut b = Xoroshiro128::new(0xdead_beef, 0xcafe_babe);
        for _ in 0..10 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
