//! AES-128 in counter mode (CTR) pseudorandom number generator.
//!
//! Encrypts successive 128-bit counter values under a fixed AES-128 key to
//! produce a keystream.  Each encrypted block yields four 32-bit output words.
//!
//! The AES-128 T-table implementation is taken verbatim from the cryptography
//! codebase and implements FIPS PUB 197 (2001).  The T-table path is
//! intentionally optimised for throughput, not constant-time behaviour.
//!
//! Counter mode construction follows NIST SP 800-38A § 6.5.
//!
//! # References
//!
//! * National Institute of Standards and Technology, "Advanced Encryption
//!   Standard (AES)", *FIPS PUB 197*, November 2001.
//! * M. Dworkin, "Recommendation for Block Cipher Modes of Operation",
//!   *NIST Special Publication 800-38A*, 2001.
//!
//! # Author
//! Joan Daemen and Vincent Rijmen (Rijndael / AES, 2001);
//! T-table implementation by Darrell Long (UC Santa Cruz).

use super::Rng;

// ─────────────────────────────────────────────────────────────────────────────
// FIPS 197 S-box  (§ 4.2.1)
// ─────────────────────────────────────────────────────────────────────────────

const SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

/// Key schedule round constants — FIPS 197, § 5.2.
const RCON: [u32; 10] = [
    0x0100_0000,
    0x0200_0000,
    0x0400_0000,
    0x0800_0000,
    0x1000_0000,
    0x2000_0000,
    0x4000_0000,
    0x8000_0000,
    0x1b00_0000,
    0x3600_0000,
];

// ─────────────────────────────────────────────────────────────────────────────
// GF(2⁸) arithmetic helpers — used only at compile time to build T-tables.
// ─────────────────────────────────────────────────────────────────────────────

const fn xtime(a: u8) -> u8 {
    (a << 1) ^ (0x1b & 0u8.wrapping_sub(a >> 7))
}
const fn mul3(a: u8) -> u8 {
    xtime(a) ^ a
}

// ─────────────────────────────────────────────────────────────────────────────
// Encryption T-tables — computed at compile time from SBOX.
//
// TE0[a] = [ 2·S[a], S[a], S[a], 3·S[a] ] (big-endian bytes).
// TE1..TE3 are right-rotations of TE0 covering the four output-byte positions.
// ─────────────────────────────────────────────────────────────────────────────

const TE0: [u32; 256] = {
    let mut t = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let s = SBOX[i];
        t[i] =
            ((xtime(s) as u32) << 24) | ((s as u32) << 16) | ((s as u32) << 8) | (mul3(s) as u32);
        i += 1;
    }
    t
};

const TE1: [u32; 256] = {
    let mut t = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        t[i] = TE0[i].rotate_right(8);
        i += 1;
    }
    t
};
const TE2: [u32; 256] = {
    let mut t = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        t[i] = TE0[i].rotate_right(16);
        i += 1;
    }
    t
};
const TE3: [u32; 256] = {
    let mut t = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        t[i] = TE0[i].rotate_right(24);
        i += 1;
    }
    t
};

// ─────────────────────────────────────────────────────────────────────────────
// Key expansion — AES-128 (NK=4, NR=10, 44 round-key words).
// ─────────────────────────────────────────────────────────────────────────────

fn sub_word(w: u32) -> u32 {
    u32::from(SBOX[(w >> 24) as usize]) << 24
        | u32::from(SBOX[((w >> 16) & 0xff) as usize]) << 16
        | u32::from(SBOX[((w >> 8) & 0xff) as usize]) << 8
        | u32::from(SBOX[(w & 0xff) as usize])
}

fn expand_128(key: &[u8; 16]) -> [u32; 44] {
    let mut w = [0u32; 44];
    for i in 0..4 {
        w[i] = u32::from_be_bytes(key[4 * i..4 * i + 4].try_into().unwrap());
    }
    for i in 4..44 {
        let mut t = w[i - 1];
        if i % 4 == 0 {
            t = sub_word(t.rotate_left(8)) ^ RCON[i / 4 - 1];
        }
        w[i] = w[i - 4] ^ t;
    }
    w
}

// ─────────────────────────────────────────────────────────────────────────────
// AES-128 block encryption — FIPS 197 T-table path.
// ─────────────────────────────────────────────────────────────────────────────

fn aes_encrypt(block: &[u8; 16], rk: &[u32; 44]) -> [u8; 16] {
    let mut s0 = u32::from_be_bytes(block[0..4].try_into().unwrap()) ^ rk[0];
    let mut s1 = u32::from_be_bytes(block[4..8].try_into().unwrap()) ^ rk[1];
    let mut s2 = u32::from_be_bytes(block[8..12].try_into().unwrap()) ^ rk[2];
    let mut s3 = u32::from_be_bytes(block[12..16].try_into().unwrap()) ^ rk[3];

    for r in 1..10 {
        let k = 4 * r;
        let t0 = TE0[(s0 >> 24) as usize]
            ^ TE1[((s1 >> 16) & 0xff) as usize]
            ^ TE2[((s2 >> 8) & 0xff) as usize]
            ^ TE3[(s3 & 0xff) as usize]
            ^ rk[k];
        let t1 = TE0[(s1 >> 24) as usize]
            ^ TE1[((s2 >> 16) & 0xff) as usize]
            ^ TE2[((s3 >> 8) & 0xff) as usize]
            ^ TE3[(s0 & 0xff) as usize]
            ^ rk[k + 1];
        let t2 = TE0[(s2 >> 24) as usize]
            ^ TE1[((s3 >> 16) & 0xff) as usize]
            ^ TE2[((s0 >> 8) & 0xff) as usize]
            ^ TE3[(s1 & 0xff) as usize]
            ^ rk[k + 2];
        let t3 = TE0[(s3 >> 24) as usize]
            ^ TE1[((s0 >> 16) & 0xff) as usize]
            ^ TE2[((s1 >> 8) & 0xff) as usize]
            ^ TE3[(s2 & 0xff) as usize]
            ^ rk[k + 3];
        s0 = t0;
        s1 = t1;
        s2 = t2;
        s3 = t3;
    }

    let k = 40;
    let c0 = u32::from(SBOX[(s0 >> 24) as usize]) << 24
        | u32::from(SBOX[((s1 >> 16) & 0xff) as usize]) << 16
        | u32::from(SBOX[((s2 >> 8) & 0xff) as usize]) << 8
        | u32::from(SBOX[(s3 & 0xff) as usize]);
    let c1 = u32::from(SBOX[(s1 >> 24) as usize]) << 24
        | u32::from(SBOX[((s2 >> 16) & 0xff) as usize]) << 16
        | u32::from(SBOX[((s3 >> 8) & 0xff) as usize]) << 8
        | u32::from(SBOX[(s0 & 0xff) as usize]);
    let c2 = u32::from(SBOX[(s2 >> 24) as usize]) << 24
        | u32::from(SBOX[((s3 >> 16) & 0xff) as usize]) << 16
        | u32::from(SBOX[((s0 >> 8) & 0xff) as usize]) << 8
        | u32::from(SBOX[(s1 & 0xff) as usize]);
    let c3 = u32::from(SBOX[(s3 >> 24) as usize]) << 24
        | u32::from(SBOX[((s0 >> 16) & 0xff) as usize]) << 16
        | u32::from(SBOX[((s1 >> 8) & 0xff) as usize]) << 8
        | u32::from(SBOX[(s2 & 0xff) as usize]);

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&(c0 ^ rk[k]).to_be_bytes());
    out[4..8].copy_from_slice(&(c1 ^ rk[k + 1]).to_be_bytes());
    out[8..12].copy_from_slice(&(c2 ^ rk[k + 2]).to_be_bytes());
    out[12..16].copy_from_slice(&(c3 ^ rk[k + 3]).to_be_bytes());
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// AES-128-CTR RNG
// ─────────────────────────────────────────────────────────────────────────────

/// AES-128 in counter mode (NIST SP 800-38A § 6.5).
///
/// Each invocation of `next_u32` returns one 32-bit word from the keystream
/// produced by encrypting a 128-bit big-endian counter.  Four words are
/// dispensed per AES block; the counter is incremented after each block.
///
/// Default key: NIST SP 800-38A Appendix F.5 AES-128-CTR test vector key
/// `2b7e1516 28aed2a6 abf71588 09cf4f3c`.
/// Default counter: all zeros.
pub struct AesCtr {
    rk: [u32; 44],
    counter: u128, // 128-bit counter, incremented after each block
    buf: [u32; 4], // current keystream block, as four big-endian u32s
    pos: usize,    // index of next word to return (0..4); 4 = exhausted
}

impl AesCtr {
    /// Construct an AES-128-CTR RNG from a 16-byte key and a 128-bit counter.
    ///
    /// The counter is interpreted as a big-endian unsigned integer and
    /// incremented (wrapping) after each 16-byte block.
    #[must_use]
    pub fn new(key: &[u8; 16], counter: u128) -> Self {
        Self {
            rk: expand_128(key),
            counter,
            buf: [0u32; 4],
            pos: 4, // force refill on first next_u32()
        }
    }

    /// Construct with the NIST SP 800-38A AES-128-CTR test vector key and
    /// counter = 0.
    ///
    /// Key: `2b7e151628aed2a6abf7158809cf4f3c`
    #[must_use]
    pub fn with_nist_key() -> Self {
        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        Self::new(&key, 0)
    }

    /// Fill `buf` by encrypting the current counter block, then increment.
    fn refill(&mut self) {
        let block = self.counter.to_be_bytes();
        let ct = aes_encrypt(&block, &self.rk);
        self.buf[0] = u32::from_be_bytes(ct[0..4].try_into().unwrap());
        self.buf[1] = u32::from_be_bytes(ct[4..8].try_into().unwrap());
        self.buf[2] = u32::from_be_bytes(ct[8..12].try_into().unwrap());
        self.buf[3] = u32::from_be_bytes(ct[12..16].try_into().unwrap());
        self.counter = self.counter.wrapping_add(1);
        self.pos = 0;
    }
}

impl Rng for AesCtr {
    fn next_u32(&mut self) -> u32 {
        if self.pos >= 4 {
            self.refill();
        }
        let w = self.buf[self.pos];
        self.pos += 1;
        w
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // NIST SP 800-38A Appendix F.5, AES-128 CTR.
    // Key:          2b7e151628aed2a6abf7158809cf4f3c
    // Counter blk1: f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff
    // Plaintext 1:  6bc1bee22e409f96e93d7e117393172a
    // Ciphertext 1: 874d6191b620e3261bef6864990db6ce
    // Keystream 1 = ciphertext XOR plaintext = ec8cdf7398607cb0f2d21675ea9ea1e4
    //
    // Counter blk2: f0f1f2f3f4f5f6f7f8f9fafbfcfdff00
    // Plaintext 2:  ae2d8a571e03ac9c9eb76fac45af8e51
    // Ciphertext 2: 9806f66b7970fdff8617187bb9fffdff
    // Keystream 2 = 362b7c3c6773516318a077d7fc5073ae
    #[test]
    fn nist_sp_800_38a_ctr_f5() {
        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        let ctr0: u128 = 0xf0f1f2f3_f4f5f6f7_f8f9fafb_fcfdfeff_u128;
        let mut rng = AesCtr::new(&key, ctr0);

        // Expected: keystream words = Enc(key, counter_i) as big-endian u32s.
        // keystream_1 = ec8cdf73 98607cb0 f2d21675 ea9ea1e4
        // keystream_2 = 362b7c3c 67735163 18a077d7 fc5073ae
        let expected: [u32; 8] = [
            0xec8cdf73, 0x98607cb0, 0xf2d21675, 0xea9ea1e4, 0x362b7c3c, 0x67735163, 0x18a077d7,
            0xfc5073ae,
        ];
        for &exp in &expected {
            assert_eq!(rng.next_u32(), exp, "AES-CTR NIST vector mismatch");
        }
    }

    #[test]
    fn produces_non_constant_output() {
        let mut rng = AesCtr::with_nist_key();
        let a = rng.next_u32();
        let b = rng.next_u32();
        assert_ne!(a, b);
    }
}
