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
//! **Output lengths** (SP 800-90 Table 4):
//! | Curve  | seqlen | outlen |
//! |--------|--------|--------|
//! | P-256  | 256    | 240    |
//! | P-384  | 384    | 368    |
//! | P-521  | 521    | 504    |
//!
//! **Performance note:** Each output block requires two scalar multiplications
//! (s·P and t·Q), making Dual_EC_DRBG orders of magnitude slower than
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
    curve:   CurveParams,
    p:       AffinePoint,   // generator point  (P = G for NIST standard)
    q:       AffinePoint,   // secondary point  (Q from SP 800-90 Appendix A.1)
    s:       BigUint,       // current state scalar
    outlen:  usize,         // output bits per block (multiple of 8)
    buf:     Vec<u8>,       // buffered output bytes
    pos:     usize,         // index of next unread byte in buf
}

impl DualEcDrbg {
    /// Construct with explicit curve, generator P, secondary point Q, seed, and outlen.
    ///
    /// * `curve`  — any short-Weierstrass prime-field curve.
    /// * `p`      — generator point (typically `curve.base_point()`).
    /// * `q`      — secondary point; determines output.  NIST values may be backdoored.
    /// * `seed`   — initial state, interpreted as a big-endian integer.  Should be
    ///              at least `⌈seqlen/8⌉` bytes of high-entropy material.
    /// * `outlen` — output bits per block; must be a positive multiple of 8.
    pub fn new(
        curve:  CurveParams,
        p:      AffinePoint,
        q:      AffinePoint,
        seed:   &[u8],
        outlen: usize,
    ) -> Self {
        assert!(outlen > 0 && outlen % 8 == 0, "outlen must be a positive multiple of 8");
        let s = BigUint::from_be_bytes(seed);
        Self { curve, p, q, s, outlen, buf: Vec::new(), pos: 0 }
    }

    /// P-256 (secp256r1) with NIST SP 800-90 standard Q point.  outlen = 240 bits.
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
        let r_bytes = to_be_padded(&r_point.x, self.curve.coord_len);

        // rightmost(outlen, r): the least-significant outlen bits = last outlen/8 bytes
        let skip = r_bytes.len() - self.outlen / 8;
        self.buf = r_bytes[skip..].to_vec();
        self.pos = 0;
    }
}

impl Rng for DualEcDrbg {
    /// Return the next 32-bit word, generating a new block when needed.
    fn next_u32(&mut self) -> u32 {
        // Ensure at least 4 bytes are available.
        while self.buf.len() - self.pos < 4 {
            // Carry over any partial bytes into the next block's prefix.
            let leftover: Vec<u8> = self.buf[self.pos..].to_vec();
            self.generate_block();
            // Prepend leftover before the new block.
            let new_buf = [leftover, self.buf.clone()].concat();
            self.buf = new_buf;
            self.pos = 0;
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

/// Zero-pad `x` to exactly `len` bytes in big-endian.
fn to_be_padded(x: &BigUint, len: usize) -> Vec<u8> {
    let raw = x.to_be_bytes();
    if raw.len() >= len {
        // Truncate to the least-significant `len` bytes (should never lose information).
        raw[raw.len() - len..].to_vec()
    } else {
        let mut out = vec![0u8; len];
        out[len - raw.len()..].copy_from_slice(&raw);
        out
    }
}

/// Construct an [`AffinePoint`] from two lowercase hex strings (no spaces).
fn point_from_hex(x_hex: &str, y_hex: &str) -> AffinePoint {
    let x = BigUint::from_be_bytes(&decode_hex(x_hex));
    let y = BigUint::from_be_bytes(&decode_hex(y_hex));
    AffinePoint::new(x, y)
}

/// Decode a lowercase hex string (may span multiple `&str` pieces after concat)
/// to a `Vec<u8>`.  Strips whitespace; panics on invalid hex.
fn decode_hex(s: &str) -> Vec<u8> {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(cleaned.len() % 2 == 0, "hex string must have even length");
    (0..cleaned.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16).expect("valid hex digit"))
        .collect()
}
