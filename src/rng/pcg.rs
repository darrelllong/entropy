//! PCG (Permuted Congruential Generator) — PCG32 and PCG64.
//!
//! A family of LCG-based generators whose output is passed through a
//! *permutation function* (XSH-RR for 32-bit, XSL-RR for 64-bit) that
//! destroys the linearity visible in the raw LCG stream.  The result passes
//! BigCrush, PractRand, and NIST SP 800-22 with high confidence.
//!
//! PCG generators support multiple independent streams via a second parameter
//! (`seq`) that selects the LCG increment; streams with different `seq` values
//! are statistically independent.
//!
//! # References
//! M. E. O'Neill, "PCG: A Family of Simple Fast Space-Efficient Statistically
//! Good Algorithms for Random Number Generation", Harvey Mudd College
//! Technical Report HMC-CS-2014-0905, 2014.
//! [pubs/oneill-2014-pcg.pdf]
//!
//! # Author
//! Melissa E. O'Neill (algorithm); Darrell Long (Rust port).

use super::{OsRng, Rng};

// ── PCG32 (64-bit LCG, XSH-RR output → 32 bits) ─────────────────────────────

const PCG32_MULT: u64 = 6_364_136_223_846_793_005;

/// 32-bit PCG using a 64-bit LCG with XSH-RR output permutation.
///
/// Period: 2⁶⁴.  Two independent streams via the `seq` parameter.
pub struct Pcg32 {
    state: u64,
    inc: u64, // must be odd; encodes stream selection
}

impl Pcg32 {
    /// Construct from an initial state and stream selector.
    ///
    /// Any `seq` value selects a distinct, non-overlapping stream.
    #[must_use]
    pub fn new(state: u64, seq: u64) -> Self {
        let inc = (seq << 1) | 1;
        let mut rng = Self { state: 0, inc };
        // Mirror the reference C initialiser exactly:
        //   pcg32_random_r(rng);  // advance from 0
        //   rng->state += initstate;
        //   pcg32_random_r(rng);  // mix in the seed
        rng.step();
        rng.state = rng.state.wrapping_add(state);
        rng.step();
        rng
    }

    /// Construct from 128 bits drawn from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        Self::new(os.next_u64(), os.next_u64())
    }

    #[inline]
    fn step(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(PCG32_MULT).wrapping_add(self.inc);
        // XSH-RR: xorshift high bits, then rotate right.
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }
}

impl Default for Pcg32 {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for Pcg32 {
    fn next_u32(&mut self) -> u32 {
        self.step()
    }
}

// ── PCG64 (128-bit LCG, XSL-RR output → 64 bits) ────────────────────────────

// Multiplier from the PCG reference implementation (pcg128_once_insecure).
const PCG64_MULT: u128 = 47_026_247_687_942_121_848_144_207_491_837_523_525;

/// 64-bit PCG using a 128-bit LCG with XSL-RR output permutation.
///
/// Period: 2¹²⁸.  Two independent streams via the `seq` parameter.
pub struct Pcg64 {
    state: u128,
    inc: u128,
}

impl Pcg64 {
    /// Construct from 256 bits of seed material (state + stream selector).
    #[must_use]
    pub fn new(state: u128, seq: u128) -> Self {
        let inc = (seq << 1) | 1;
        let mut rng = Self { state: 0, inc };
        rng.step();
        rng.state = rng.state.wrapping_add(state);
        rng.step();
        rng
    }

    /// Construct from 256 bits drawn from the operating system RNG.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        let s = ((os.next_u64() as u128) << 64) | os.next_u64() as u128;
        let q = ((os.next_u64() as u128) << 64) | os.next_u64() as u128;
        Self::new(s, q)
    }

    #[inline]
    fn step(&mut self) -> u64 {
        let old = self.state;
        self.state = old.wrapping_mul(PCG64_MULT).wrapping_add(self.inc);
        // XSL-RR: xor the two 64-bit halves, then rotate right by top 6 bits.
        let xsl = ((old >> 64) as u64) ^ (old as u64);
        let rot = (old >> 122) as u32;
        xsl.rotate_right(rot)
    }
}

impl Default for Pcg64 {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for Pcg64 {
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

    // Reference values from the PCG32 demo (seed=42, seq=54):
    // https://www.pcg-random.org/using-pcg-c-basic.html
    #[test]
    fn pcg32_reference_sequence() {
        let mut rng = Pcg32::new(42, 54);
        let expected: [u32; 6] = [
            0xa15c02b7, 0x7b47f409, 0xba1d3330, 0x83d2f293, 0xbfa4784b, 0xcbed606e,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u32(), e, "PCG32 reference mismatch");
        }
    }

    #[test]
    fn pcg32_different_streams_differ() {
        let mut a = Pcg32::new(1, 1);
        let mut b = Pcg32::new(1, 2);
        assert_ne!(a.next_u32(), b.next_u32());
    }

    #[test]
    fn pcg64_produces_nonzero_output() {
        let mut rng = Pcg64::from_os_rng();
        let v: u64 = (0..8).map(|_| rng.next_u64()).fold(0, |a, b| a | b);
        assert_ne!(v, 0);
    }

    #[test]
    fn pcg64_next_u32_uses_high_bits_of_u64() {
        // Verify next_u32 returns the upper 32 bits of a native step.
        let mut a = Pcg64::new(1, 1);
        let mut b = Pcg64::new(1, 1);
        assert_eq!(a.next_u32(), (b.next_u64() >> 32) as u32);
    }
}
