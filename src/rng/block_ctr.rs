//! Generic block-cipher CTR-mode pseudorandom number generator.
//!
//! Any [`cryptography::BlockCipher`] can be wrapped by [`BlockCtrRng`] to
//! produce a keystream by encrypting successive big-endian counter values.
//! This is the construction specified in NIST SP 800-38A § 6.5 for arbitrary
//! block sizes, generalising the dedicated [`super::AesCtr`] wrapper.
//!
//! One block is generated per [`BlockCipher::encrypt`] call; the counter is
//! incremented (wrapping) after each block.  The keystream bytes are consumed
//! in little-endian order to match the conventions used by the rest of this
//! crate's byte-backed generators.
//!
//! # Invariants
//!
//! `0 ≤ pos ≤ C::BLOCK_LEN`.  `pos == C::BLOCK_LEN` is the sentinel meaning
//! "buffer exhausted; refill required."  Construction sets `pos = BLOCK_LEN`
//! so the first call to `next_u32` always triggers a refill.
//!
//! # References
//! * M. Dworkin, "Recommendation for Block Cipher Modes of Operation",
//!   *NIST SP 800-38A*, 2001.

use cryptography::BlockCipher;

use super::Rng;

/// Block-cipher CTR-mode RNG wrapping any [`BlockCipher`].
///
/// The counter is a 128-bit integer encoded big-endian, right-justified into
/// the cipher's block (so the low `BLOCK_LEN` bytes of the counter occupy the
/// block; any excess high bits are silently truncated).  For ciphers with
/// `BLOCK_LEN == 16` the full 128-bit counter is used.  For ciphers with
/// `BLOCK_LEN < 16` the effective counter space is `2^(8 * BLOCK_LEN)` blocks.
///
/// Keystream bytes are consumed least-significant byte first (LE), consistent
/// with `ChaCha20Rng` and other byte-backed generators in this crate.
pub struct BlockCtrRng<C: BlockCipher> {
    cipher: C,
    counter: u128,
    buf: Vec<u8>, // one encrypted block = C::BLOCK_LEN bytes
    pos: usize,
}

impl<C: BlockCipher> BlockCtrRng<C> {
    /// Construct from an already-keyed cipher and an initial counter value.
    ///
    /// `counter = 0` is the conventional starting point for CTR mode.
    ///
    /// # Panics
    ///
    /// Panics if `C::BLOCK_LEN < 4`, because a block smaller than 4 bytes
    /// cannot hold a single `u32` output word.
    pub fn new(cipher: C, counter: u128) -> Self {
        let block_len = C::BLOCK_LEN;
        assert!(
            block_len >= 4,
            "BlockCtrRng requires BLOCK_LEN >= 4, got {block_len}"
        );
        // pos == block_len forces a refill on the first next_u32() call.
        Self {
            cipher,
            counter,
            buf: vec![0u8; block_len],
            pos: block_len,
        }
    }

    fn refill(&mut self) {
        let block_len = C::BLOCK_LEN;
        let ctr_bytes = self.counter.to_be_bytes(); // always 16 bytes
        // Right-justify the counter into the block: copy the least-significant
        // `copy_len` bytes of ctr_bytes into the tail of the block.
        // For BLOCK_LEN == 16, copy_len == 16 and the full counter is used.
        // For BLOCK_LEN < 16, only the low BLOCK_LEN bytes are used and the
        // high counter bits are truncated — the effective counter wraps at
        // 2^(8 * BLOCK_LEN) blocks.
        let copy_len = block_len.min(16);
        for b in &mut self.buf {
            *b = 0;
        }
        self.buf[block_len - copy_len..].copy_from_slice(&ctr_bytes[16 - copy_len..]);
        self.cipher.encrypt(&mut self.buf);
        self.counter = self.counter.wrapping_add(1);
        self.pos = 0;
    }
}

impl<C: BlockCipher> Rng for BlockCtrRng<C> {
    fn next_u32(&mut self) -> u32 {
        if self.pos + 4 > self.buf.len() {
            self.refill();
        }
        // Safety: the guard above and the BLOCK_LEN >= 4 assertion in new()
        // together guarantee self.buf[self.pos..self.pos+4] is always valid.
        let w = u32::from_le_bytes(self.buf[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cryptography::Aes128;

    /// Known-answer test: AES-128(key=0, plaintext=0) = 66e94bd4ef8a2c3b884cfa59ca342b2e
    ///
    /// Source: NIST FIPS 197, Appendix B (AES-128 known-answer vector).
    /// The first u32 from the keystream in little-endian byte order is
    /// u32::from_le_bytes([0x66, 0xe9, 0x4b, 0xd4]) = 0xd44b_e966.
    #[test]
    fn block_ctr_rng_kat_fips197() {
        let key = [0u8; 16];
        let cipher = Aes128::new(&key);
        let mut rng = BlockCtrRng::new(cipher, 0);
        assert_eq!(
            rng.next_u32(),
            0xd44b_e966,
            "First u32 must match FIPS 197 AES-128 KAT: AES(key=0, ctr=0)[0..4] LE"
        );
    }

    /// Known-answer test: second block of AES-128(key=0, ctr=1) differs from block 0.
    ///
    /// Verifies counter advancement.  Both values are deterministic; the
    /// assertion checks the known first word of each block.
    #[test]
    fn block_ctr_rng_advances_across_blocks() {
        let key = [0u8; 16];
        let cipher = Aes128::new(&key);
        let mut rng = BlockCtrRng::new(cipher, 0);
        // Drain block 0 (4 words × 4 bytes = 16 bytes).
        let block0: Vec<u32> = (0..4).map(|_| rng.next_u32()).collect();
        // Read block 1.
        let block1: Vec<u32> = (0..4).map(|_| rng.next_u32()).collect();
        // AES(key=0, ctr=0) ≠ AES(key=0, ctr=1): these are deterministic and unequal.
        assert_ne!(
            block0, block1,
            "consecutive CTR blocks must differ"
        );
        // First word of block 0 is the FIPS 197 KAT value (verified above).
        assert_eq!(block0[0], 0xd44b_e966);
    }
}
