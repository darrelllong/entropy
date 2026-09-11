//! Dual_EC_DRBG — Dual Elliptic Curve Deterministic Random Bit Generator.
//!
//! Implements the Dual_EC_DRBG algorithm from NIST SP 800-90 (June 2006), §9.
//! This DRBG was withdrawn in NIST SP 800-90A Rev. 1 (June 2015) after
//! Bernstein et al. demonstrated that the NSA-specified Q points likely contain
//! a backdoor: knowledge of the discrete logarithm e with Q = e·P allows the
//! entire internal state to be recovered from 30 bytes of output.
//!
//! This implementation supports pluggable curves and Q points:
//! - [`DualEcDrbg::p256`], [`DualEcDrbg::p384`], [`DualEcDrbg::p521`] use the
//!   NIST-specified Q points from SP 800-90 Appendix A.1 (potentially backdoored).
//! - [`DualEcDrbg::new`] accepts any [`CurveParams`] with caller-supplied P and Q,
//!   enabling use of non-NSA Q points on any supported curve.
//!
//! **Algorithm** (SP 800-90 §9, simplified, no prediction resistance):
//! ```text
//! state:  s  (seqlen-bit integer)
//! per block:
//!   t  = x(s · P)            // x-coordinate of scalar multiplication
//!   s  ← x(t · P)            // update state
//!   r  = x(t · Q)            // output value
//!   out = rightmost(outlen, r)  // least-significant outlen bits
//! ```
//!
//! **Output lengths** (the `outlen` SP 800-90 specifies for each curve):
//! | Curve  | seqlen | outlen |
//! |--------|--------|--------|
//! | P-256  | 256    | 240    |
//! | P-384  | 384    | 368    |
//! | P-521  | 521    | 504    |
//!
//! **Performance note:** Each output block requires three scalar
//! multiplications (t = x(s·P), the output from x(t·Q), and the next state
//! x(t·P)), making Dual_EC_DRBG orders of magnitude slower than
//! hash- or cipher-based DRBGs.  This implementation is suitable for research
//! and statistical testing, not high-throughput applications.
//!
//! # References
//! * NIST SP 800-90, June 2006, §9 and Appendix A.1.  *(Original; Dual_EC
//!   was removed in Rev. 1.)*
//! * D. Bernstein, T. Lange, R. Niederhagen, "Dual EC: A Standardized Back
//!   Door," *The New Codebreakers*, LNCS 9100, 2016.
//!
//! # Author
//! Darrell Long (UC Santa Cruz).

use cryptography::vt::{AffinePoint, BigUint, CurveParams};

use super::Rng;

// ── DualEcDrbg ───────────────────────────────────────────────────────────────

/// Dual_EC_DRBG with pluggable elliptic curve and P/Q points.
pub struct DualEcDrbg {
    curve: CurveParams,
    p: AffinePoint, // generator point  (P = G for NIST standard)
    q: AffinePoint, // secondary point  (Q from SP 800-90 Appendix A.1)
    s: BigUint,     // current state scalar
    outlen: usize,  // output bits per block (multiple of 8)
    buf: Vec<u8>,   // buffered output bytes
    pos: usize,     // index of next unread byte in buf
}

impl DualEcDrbg {
    /// Construct with explicit curve, generator P, secondary point Q, seed, and outlen.
    ///
    /// * `curve`  — any short-Weierstrass prime-field curve.
    /// * `p`      — generator point (typically `curve.base_point()`).
    /// * `q`      — secondary point; determines output.  NIST values may be backdoored.
    /// * `seed`   — initial state, interpreted as a big-endian integer.  Should be
    ///   at least `⌈seqlen/8⌉` bytes of high-entropy material.
    /// * `outlen` — output bits per block; a multiple of 8 in
    ///   `32..=max_outlen(curve)`.  The lower bound is what the [`Rng`] path
    ///   needs to assemble 32-bit words across block boundaries.  The upper
    ///   bound is 240 / 368 / 504 bits for P-256 / P-384 / P-521, the
    ///   `max_outlen` SP 800-90 gives for those curves and the `outlen` the
    ///   constructors below use.  It is computed as
    ///   `8·⌊(seedlen − (13 + log₂ h)) / 8⌋` with `seedlen` the bit length
    ///   of the group order; for other curves that rule is an extrapolation.
    ///   The standard drops those leading bits of `x(t·Q)` because a raw
    ///   x-coordinate is not uniformly distributed.  With the trapdoor they
    ///   cost only about 2¹⁶ guesses, which is the backdoor described in the
    ///   module documentation.  No edition of SP 800-90 that contains
    ///   Dual_EC_DRBG is in `pubs/`.
    ///
    /// # Panics
    /// Panics if `outlen` is not a multiple of 8 in `32..=max_outlen`, or if
    /// `seed` is empty or all-zero: `s = 0` is a fixed point of the state
    /// update (`scalar_mul` returns the point at infinity, whose x-coordinate
    /// is 0), so the generator would emit an all-zero stream forever.
    pub fn new(
        curve: CurveParams,
        p: AffinePoint,
        q: AffinePoint,
        seed: &[u8],
        outlen: usize,
    ) -> Self {
        let max_outlen = max_outlen(&curve);
        assert!(
            outlen.is_multiple_of(8) && (32..=max_outlen).contains(&outlen),
            "outlen must be a multiple of 8 in 32..={max_outlen} for this curve"
        );
        let s = BigUint::from_be_bytes(seed);
        assert!(
            !s.is_zero(),
            "Dual_EC_DRBG: seed must be a nonzero big-endian integer \
             (s = 0 is a fixed point that emits an all-zero stream)"
        );
        Self {
            curve,
            p,
            q,
            s,
            outlen,
            buf: Vec::new(),
            pos: 0,
        }
    }

    /// P-256 (secp256r1) with NIST SP 800-90 standard Q point.  outlen = 240 bits.
    ///
    /// **BACKDOORED.** The P-256 Q point in SP 800-90 Appendix A.1 Table A-1
    /// is from the NSA-generated table and is the primary suspect point in the
    /// published Dual_EC analyses.  An adversary holding the discrete-log
    /// trapdoor scalar e where Q = e·P can recover the full internal state
    /// from one output block.  Use this constructor only as a negative
    /// control.  Never use for key material or any security-sensitive purpose.
    ///
    /// Q coordinates from NIST SP 800-90 (June 2006), Appendix A.1, Table A-1.
    pub fn p256(seed: &[u8]) -> Self {
        let curve = cryptography::vt::p256();
        let p = curve.base_point();
        let q = point_from_hex(
            "c97445f45cdef9f0d3e05e1e585fc297235b82b5be8ff3efca67c59852018192",
            "b28ef557ba31dfcbdd21ac46e2a91e3c304f44cb87058ada2cb815151e610046",
        );
        Self::new(curve, p, q, seed, 240)
    }

    /// P-384 (secp384r1) with NIST SP 800-90 standard Q point.  outlen = 368 bits.
    ///
    /// **BACKDOORED.** The P-384 Q point in SP 800-90 Appendix A.1 Table A-2
    /// is from the same NSA-generated table as the P-256 Q point and is equally
    /// suspect.  An adversary holding the discrete-log trapdoor scalar e where
    /// Q = e·P can recover the full internal state from one output block.
    /// Use this constructor only as a negative control.  Never use for key
    /// material or any security-sensitive purpose.
    ///
    /// Q coordinates from NIST SP 800-90 (June 2006), Appendix A.1, Table A-2.
    pub fn p384(seed: &[u8]) -> Self {
        let curve = cryptography::vt::p384();
        let p = curve.base_point();
        let q = point_from_hex(
            "8e722de3125bddb05580164bfe20b8b432216a62926c57502ceede31c47816ed\
             d1e89769124179d0b695106428815065",
            "023b1660dd701d0839fd45eec36f9ee7b32e13b315dc02610aa1b636e346df67\
             1f790f84c5e09b05674dbb7e45c803dd",
        );
        Self::new(curve, p, q, seed, 368)
    }

    /// P-521 (secp521r1) with NIST SP 800-90 standard Q point.  outlen = 504 bits.
    ///
    /// **BACKDOORED.** The P-521 Q point in SP 800-90 Appendix A.1 Table A-3
    /// is from the same NSA-generated table as P-256 and P-384 and carries the
    /// same discrete-log trapdoor risk.  An adversary holding the trapdoor can
    /// recover the full internal state from one output block.
    /// Use this constructor only as a negative control.  Never use for key
    /// material or any security-sensitive purpose.
    ///
    /// Q coordinates from NIST SP 800-90 (June 2006), Appendix A.1, Table A-3.
    pub fn p521(seed: &[u8]) -> Self {
        let curve = cryptography::vt::p521();
        let p = curve.base_point();
        let q = point_from_hex(
            "01b9fa3e518d683c6b65763694ac8efbaec6fab44f2276171a42726507dd08ad\
             d4c3b3f4c1ebc5b1222ddba077f722943b24c3edfa0f85fe24d0c8c01591f0be6f63",
            "01f3bdba585295d9a1110d1df1f9430ef8442c5018976ff3437ef91b81dc0b81\
             32c8d5c39c32d0e004a3092b7d327c0e7a4d26d2c7b69b58f9066652911e457779de",
        );
        Self::new(curve, p, q, seed, 504)
    }

    /// Execute one generate step: update state and buffer one block of output bytes.
    fn generate_block(&mut self) {
        // t = x(s · P)
        let t_point = self.curve.scalar_mul(&self.p, &self.s);
        let t = t_point.x;

        // s ← x(t · P)  — update state using t as scalar
        let s_point = self.curve.scalar_mul(&self.p, &t);
        self.s = s_point.x;

        // r = x(t · Q)  — compute output value
        let r_point = self.curve.scalar_mul(&self.q, &t);

        // rightmost(outlen, r) = r mod 2^outlen, as outlen/8 big-endian bytes
        self.buf = r_point
            .x
            .low_bits(self.outlen)
            .to_be_bytes_padded(self.outlen / 8);
        self.pos = 0;
    }
}

impl Rng for DualEcDrbg {
    /// Return the next 32-bit word, generating a new block when needed.
    ///
    /// Note: words are decoded **big-endian** from the output block, matching
    /// SP 800-90's big-endian integer encoding — a deliberate deviation from
    /// the little-endian convention of the crate's other byte-backed
    /// generators (see [`super::Rng`]).
    fn next_u32(&mut self) -> u32 {
        let remaining = self.buf.len() - self.pos;
        if remaining < 4 {
            // Save the tail bytes (at most 3) without allocating, then
            // generate a new block and assemble the output word across
            // the old/new boundary.
            let mut spill = [0u8; 3];
            spill[..remaining].copy_from_slice(&self.buf[self.pos..]);
            self.generate_block(); // resets self.buf and self.pos
            let mut bytes = [0u8; 4];
            bytes[..remaining].copy_from_slice(&spill[..remaining]);
            bytes[remaining..].copy_from_slice(&self.buf[..4 - remaining]);
            self.pos = 4 - remaining;
            return u32::from_be_bytes(bytes);
        }
        let word = u32::from_be_bytes([
            self.buf[self.pos],
            self.buf[self.pos + 1],
            self.buf[self.pos + 2],
            self.buf[self.pos + 3],
        ]);
        self.pos += 4;
        word
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Largest admissible `outlen`: `8·⌊(seedlen − (13 + log₂ h)) / 8⌋`, with
/// `seedlen` the bit length of the group order `n`.  The rule reproduces
/// the `max_outlen` SP 800-90 gives for P-256, P-384 and P-521 (240, 368,
/// 504); no edition containing Dual_EC_DRBG is in `pubs/`.
fn max_outlen(curve: &CurveParams) -> usize {
    let seedlen = curve.n.bits();
    let hidden = 13 + curve.h.ilog2() as usize;
    8 * (seedlen.saturating_sub(hidden) / 8)
}

/// Construct an [`AffinePoint`] from two hexadecimal coordinate strings,
/// parsed by rump.
fn point_from_hex(x_hex: &str, y_hex: &str) -> AffinePoint {
    let coord = |hex: &str| BigUint::from_str_radix(hex, 16).expect("valid hex coordinate");
    AffinePoint::new(coord(x_hex), coord(y_hex))
}

impl Drop for DualEcDrbg {
    /// Wipe the buffered output and overwrite the state scalar on drop.
    ///
    /// The `BigUint` heap storage is scrubbed by the bignum type itself: the
    /// cryptography crate enables rump's `wipe` feature, so the replaced
    /// state scalar volatile-wipes its limbs when it drops. The explicit
    /// overwrite here is belt-and-suspenders; Dual_EC_DRBG is a research
    /// negative control demonstrating a backdoor, not a construction that
    /// protects live key material.
    fn drop(&mut self) {
        cryptography::zeroize_slice(&mut self.buf);
        self.s = BigUint::zero();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_seed_rejected() {
        // s = 0 is a fixed point (all-zero output stream); new() must panic.
        let r = std::panic::catch_unwind(|| DualEcDrbg::p256(&[0u8; 32]));
        assert!(r.is_err(), "all-zero seed must be rejected");
        let r = std::panic::catch_unwind(|| DualEcDrbg::p256(b""));
        assert!(r.is_err(), "empty seed must be rejected");
    }

    /// P-256 instance with Q = 3·G rather than Q = P, so an output block is
    /// not simply the next state.  Tests built on it check bookkeeping only;
    /// the P and Q roles are pinned by `p256_seed_one_kat`.
    fn p256_with_outlen(outlen: usize) -> DualEcDrbg {
        let curve = cryptography::vt::p256();
        let p = curve.base_point();
        let q = curve.scalar_mul(&p, &BigUint::from_be_bytes(&[3]));
        DualEcDrbg::new(curve, p, q, &[1u8; 32], outlen)
    }

    /// The `max_outlen` values SP 800-90 gives for the three NIST curves.
    #[test]
    fn max_outlen_matches_sp800_90() {
        assert_eq!(max_outlen(&cryptography::vt::p256()), 240);
        assert_eq!(max_outlen(&cryptography::vt::p384()), 368);
        assert_eq!(max_outlen(&cryptography::vt::p521()), 504);
    }

    /// Regression: `new` accepted any positive multiple of 8 for `outlen`,
    /// then `next_u32` panicked on the first draw for `outlen < 32`, and
    /// nothing stopped `outlen` from exceeding the standard's `max_outlen`.
    #[test]
    fn outlen_bounds_enforced_at_construction() {
        for bad in [0usize, 8, 12, 16, 24, 248, 256, 264] {
            let r = std::panic::catch_unwind(|| p256_with_outlen(bad));
            assert!(r.is_err(), "outlen {bad} must be rejected");
        }
        for ok in [32usize, 40, 240] {
            let mut g = p256_with_outlen(ok);
            let _ = g.next_u32();
            let _ = g.next_u32();
        }
    }

    /// `next_u32` splices words across block boundaries.  With 5-byte
    /// blocks the unread-byte count before each draw cycles 0, 1, 2, 3, 4,
    /// so three words in five straddle a boundary, carrying one, two or
    /// three saved bytes, and the word stream must equal the big-endian
    /// parse of the concatenated raw blocks.
    #[test]
    fn next_u32_splices_words_across_block_boundaries() {
        let mut words = p256_with_outlen(40);
        let mut blocks = p256_with_outlen(40);
        let mut raw = Vec::new();
        for _ in 0..8 {
            blocks.generate_block();
            raw.extend_from_slice(&blocks.buf);
        }
        assert_eq!(raw.len(), 40);
        for chunk in raw.chunks_exact(4) {
            let want = u32::from_be_bytes(chunk.try_into().unwrap());
            assert_eq!(words.next_u32(), want);
        }
    }

    /// Known-answer test for the battery's instantiation (P-256, outlen 240,
    /// seed 0x00…01): the first two 30-byte blocks as fifteen big-endian
    /// words, one of which straddles the block boundary.  Values come from an
    /// independent Python replica of the generate step using textbook affine
    /// P-256 arithmetic, so the rump-parsed Q, the scalar multiplications,
    /// and the `rightmost(outlen)` truncation are all pinned.
    #[test]
    fn p256_seed_one_kat() {
        let mut seed = [0u8; 32];
        seed[31] = 1;
        let mut g = DualEcDrbg::p256(&seed);
        let want: [u32; 15] = [
            0x6094_1c3f,
            0x56bc_e106,
            0x3354_0ae7,
            0xf66e_bd30,
            0x89f8_6f37,
            0x7b57_ff7f,
            0x6160_eda7,
            0x69ab_d54f,
            0x68fd_16fd,
            0x3310_f42a,
            0xe84a_0c29,
            0xe8d2_5773,
            0x4686_ceb7,
            0x4975_d10c,
            0xdbcf_afbd,
        ];
        let got: Vec<u32> = want.iter().map(|_| g.next_u32()).collect();
        assert_eq!(got, want);
    }

    /// The SP 800-90 Q literals for all three curves parse (via rump) to
    /// valid points on their curves.
    #[test]
    fn nist_q_points_are_on_their_curves() {
        let seed = [1u8; 66];
        for g in [
            DualEcDrbg::p256(&seed[..32]),
            DualEcDrbg::p384(&seed[..48]),
            DualEcDrbg::p521(&seed),
        ] {
            assert!(!g.q.infinity && g.curve.is_on_curve(&g.q));
        }
    }
}
