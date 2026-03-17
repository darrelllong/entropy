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
/// `BLOCK_LEN == 16` the full 128-bit counter is used.
///
/// Keystream bytes are consumed least-significant byte first (LE), consistent
/// with `ChaCha20Rng` and other byte-backed generators in this crate.
pub struct BlockCtrRng<C: BlockCipher> {
    cipher:  C,
    counter: u128,
    buf:     Vec<u8>,   // one encrypted block = C::BLOCK_LEN bytes
    pos:     usize,
}

impl<C: BlockCipher> BlockCtrRng<C> {
    /// Construct from an already-keyed cipher and an initial counter value.
    ///
    /// `counter = 0` is the conventional starting point for CTR mode.
    pub fn new(cipher: C, counter: u128) -> Self {
        let block_len = C::BLOCK_LEN;
        // pos == block_len forces a refill on the first next_u32() call.
        Self { cipher, counter, buf: vec![0u8; block_len], pos: block_len }
    }

    fn refill(&mut self) {
        let block_len = C::BLOCK_LEN;
        let ctr_bytes = self.counter.to_be_bytes(); // always 16 bytes
        // Right-justify the counter into the block: copy the last `copy_len`
        // bytes of ctr_bytes into the last `copy_len` bytes of the block.
        let copy_len = block_len.min(16);
        for b in &mut self.buf { *b = 0; }
        self.buf[block_len - copy_len..]
            .copy_from_slice(&ctr_bytes[16 - copy_len..]);
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
        let w = u32::from_le_bytes(
            self.buf[self.pos..self.pos + 4].try_into().unwrap(),
        );
        self.pos += 4;
        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cryptography::Aes128;

    #[test]
    fn block_ctr_rng_non_constant() {
        // AES-128 with the SP 800-38A test vector key; counter starts at 0.
        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
            0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
        ];
        let cipher = Aes128::new(&key);
        let mut rng = BlockCtrRng::new(cipher, 0);
        let a = rng.next_u32();
        let b = rng.next_u32();
        // Two words from the same block should differ (overwhelming probability).
        assert_ne!(a, b);
    }

    #[test]
    fn block_ctr_rng_advances_across_blocks() {
        let key = [0u8; 16];
        let cipher = Aes128::new(&key);
        let mut rng = BlockCtrRng::new(cipher, 0);
        // Drain one full 16-byte (4-word) block then read from the next.
        let words: Vec<u32> = (0..8).map(|_| rng.next_u32()).collect();
        // Block 0 and block 1 should differ in at least one word.
        assert_ne!(words[..4], words[4..]);
    }
}
