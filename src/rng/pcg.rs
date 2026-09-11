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
//! * M. E. O'Neill, "PCG: A Family of Simple Fast Space-Efficient
//!   Statistically Good Algorithms for Random Number Generation", Harvey Mudd
//!   College Technical Report HMC-CS-2014-0905, 2014.
//!   [pubs/oneill-2014-pcg.pdf]  [§6.3.1 defines the PCG-XSH-RR output
//!   function, §6.3.3 PCG-XSL-RR]
//! * M. E. O'Neill, pcg-c, the reference C implementation, commit
//!   83252d9c23df.  <https://github.com/imneme/pcg-c>
//!   [pubs/pcg-c-83252d9c23df.tar.gz]  [In `include/pcg_variants.h`,
//!   `pcg32_random_r` is `pcg_setseq_64_xsh_rr_32_random_r` and
//!   `pcg64_random_r` is `pcg_setseq_128_xsl_rr_64_random_r`, seeded by
//!   `pcg_setseq_64_srandom_r` and `pcg_setseq_128_srandom_r`;
//!   `test-high/expected` holds the known answers pinned below]
//!
//! # Author
//! Melissa E. O'Neill (algorithm); Darrell Long (Rust port).

use super::{OsRng, Rng};

// ── PCG32 (64-bit LCG, XSH-RR output → 32 bits) ─────────────────────────────

// PCG_DEFAULT_MULTIPLIER_64 in pcg_variants.h.
const PCG32_MULT: u64 = 6_364_136_223_846_793_005;

/// 32-bit PCG using a 64-bit LCG with XSH-RR output permutation.
///
/// Period: 2⁶⁴.  2⁶³ selectable streams via the `seq` parameter (the stream
/// increment is `(seq << 1) | 1`, so `seq` and `seq + 2⁶³` coincide).
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
        // Mirror pcg_setseq_64_srandom_r in pcg_variants.h exactly:
        //   rng->state = 0U;  rng->inc = (initseq << 1u) | 1u;
        //   pcg_setseq_64_step_r(rng);  // advance from 0
        //   rng->state += initstate;
        //   pcg_setseq_64_step_r(rng);  // mix in the seed
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
        // XSH-RR (pcg_output_xsh_rr_64_32; O'Neill 2014, §6.3.1) on the state
        // before the advance: xorshift high bits, then rotate right.
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

// PCG_DEFAULT_MULTIPLIER_128 in pcg_variants.h, written there as
// PCG_128BIT_CONSTANT(2549297995355413924ULL, 4865540595714422341ULL): the
// multiplier of pcg64 (XSL-RR).
const PCG64_MULT: u128 = 47_026_247_687_942_121_848_144_207_491_837_523_525;

/// 64-bit PCG using a 128-bit LCG with XSL-RR output permutation.
///
/// Period: 2¹²⁸.  2¹²⁷ selectable streams via the `seq` parameter (the stream
/// increment is `(seq << 1) | 1`, so `seq` and `seq + 2¹²⁷` coincide).
///
/// Each output permutes the state *after* the LCG advance, as O'Neill's
/// reference 128-bit generator (`pcg_setseq_128_xsl_rr_64_random_r` in
/// pcg-c) does; [`Pcg32`] permutes the state before it, as its reference does.
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
        self.state = self.state.wrapping_mul(PCG64_MULT).wrapping_add(self.inc);
        // XSL-RR (pcg_output_xsl_rr_128_64; O'Neill 2014, §6.3.3) on the
        // advanced state: xor the two 64-bit halves, then rotate right by the
        // top 6 bits.
        let state = self.state;
        let xsl = ((state >> 64) as u64) ^ (state as u64);
        let rot = (state >> 122) as u32;
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

    /// pcg-c's published output: `test-high/check-pcg32.c` seeds
    /// `pcg32_srandom_r(&rng, 42u, 54u)`, and round 1 of
    /// `test-high/expected/check-pcg32.out` lists these six values.  That
    /// check, built from [pubs/pcg-c-83252d9c23df.tar.gz], reproduces its
    /// expected file, and the tarball's `pcg32_random_r` agrees with this
    /// generator for 5000 outputs.
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

    /// pcg-c's published output: `test-high/check-pcg64.c` seeds
    /// `pcg64_srandom_r(&rng, 42u, 54u)`, and round 1 of
    /// `test-high/expected/check-pcg64.out` lists these six values.  That
    /// check, built from [pubs/pcg-c-83252d9c23df.tar.gz], reproduces its
    /// expected file, and the tarball's `pcg64_random_r` agrees with this
    /// generator for 5000 outputs at seeds (42, 54) and (1, 1).  Permuting the
    /// state before the advance instead emits one extra value first and shifts
    /// the whole stream by one.
    #[test]
    fn pcg64_reference_sequence() {
        let expected: [u64; 6] = [
            0x86b1_da1d_7206_2b68,
            0x1304_aa46_c985_3d39,
            0xa367_0e9e_0dd5_0358,
            0xf909_0e52_9a7d_ae00,
            0xc85b_9fd8_3799_6f2c,
            0x6061_21f8_e391_9196,
        ];
        let mut rng = Pcg64::new(42, 54);
        assert_eq!(expected.map(|_| rng.next_u64()), expected);
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
