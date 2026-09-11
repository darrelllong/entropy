//! ChaCha20-based CSPRNG.
//!
//! Wraps the `cryptography::ChaCha20` stream cipher as a pseudorandom byte
//! source.  [`ChaCha20Rng::from_os_rng`] draws a 256-bit key and a 96-bit
//! nonce from `OsRng` and starts at block counter 0; [`ChaCha20Rng::new`]
//! takes the key, nonce and initial block counter explicitly, so a stream can
//! be reproduced and pinned to RFC 8439.  The keystream is consumed in 64-byte
//! blocks via `ChaCha20::keystream_block()`.
//!
//! This is structurally identical to how Linux's `/dev/urandom` and macOS's
//! `arc4random` work internally (both use ChaCha20 today).  Each 64-byte
//! block costs one ChaCha20 core invocation (20 rounds over a 4×4 word
//! state), so throughput scales with the cipher's speed.
//!
//! # Byte and word order
//! Each 64-byte block is the ChaCha state after the final addition, serialized
//! as RFC 8439 §2.3 prescribes: word 0 first, each word little-endian.
//! `next_u32` reads four bytes little-endian, so it returns those state words
//! unchanged and in order: words 0 to 15 of the block at the initial counter,
//! then words 0 to 15 of the next block.  `next_u64` reads eight bytes
//! little-endian, so its k-th call within a block returns
//! `word[2k] | word[2k + 1] << 32`.
//!
//! For uniform-width access (all `next_u32` or all `next_u64`) all 512 bits
//! per block are used; mixing widths at a refill boundary silently discards
//! up to 7 trailing bytes before refilling.
//!
//! # Security
//! Output is computationally indistinguishable from random under the
//! assumption that ChaCha20 is a secure PRF.  [`ChaCha20Rng::from_os_rng`]
//! draws the key and nonce from the OS entropy source; a stream built with
//! [`ChaCha20Rng::new`] is only as secret as the key passed in.
//!
//! **Output limit.** ChaCha20 uses a 32-bit block counter; with 64 bytes per
//! block the keystream repeats after 2³² × 64 = **256 GiB** of output, the
//! limit RFC 8439 §2.3 states.  `cryptography::ChaCha20` wraps the counter
//! from 2³² − 1 to 0, so a stream started at counter `c` reaches block 0 after
//! 2³² − `c` blocks.  A long-running process that exhausts this limit will
//! silently wrap and repeat output.  No reseed or counter-exhaustion check is
//! implemented here; for applications that may produce more than a few GiB
//! from a single key, either reseed manually by constructing a fresh
//! `ChaCha20Rng::from_os_rng()` or use the OS CSPRNG directly.
//!
//! **Backtracking resistance.** No forward secrecy is provided.  Compromising
//! the process memory reveals the cipher state, which determines all future
//! output.  Correct for a test harness; do not copy this design into
//! applications that require prediction resistance.
//!
//! # References
//! * D. J. Bernstein, "ChaCha, a variant of Salsa20", Workshop Record of
//!   SASC 2008.  <https://cr.yp.to/chacha/chacha-20080128.pdf>
//!   [pubs/bernstein-2008-chacha.pdf]  [The ChaCha20 algorithm; §3.2 fills
//!   the matrix with constants, key, block counter and nonce]
//! * Y. Nir and A. Langley, "ChaCha20 and Poly1305 for IETF Protocols,"
//!   RFC 8439, June 2018.  [pubs/rfc8439-chacha20-poly1305.txt]  [§2.3: the
//!   96-bit-nonce, 32-bit-counter layout and little-endian serialization that
//!   `cryptography::ChaCha20` follows, where the original ChaCha had a 64-bit
//!   nonce and a 64-bit counter; §2.3.2, §2.4.2 and Appendix A.1 supply the
//!   known-answer tests below]
//!
//! # Author
//! Daniel J. Bernstein (ChaCha20 algorithm); Darrell Long (this adapter over
//! `cryptography::ChaCha20`, which implements the cipher).

use cryptography::ChaCha20;

use super::{ByteBuffered, OsRng, Rng};

const BLOCK_BYTES: usize = 64;

/// ChaCha20 stream cipher used as a CSPRNG.
///
/// Generates 64 bytes per ChaCha20 core invocation.
pub struct ChaCha20Rng {
    cipher: ChaCha20,
    buf: [u8; BLOCK_BYTES],
    offset: usize,
}

impl ChaCha20Rng {
    /// Construct from an explicit 256-bit key, 96-bit nonce and initial block
    /// counter, the inputs of RFC 8439 §2.3 and §2.4.
    ///
    /// The first `next_u32` returns word 0 of the block at `counter`; the
    /// module docs give the full byte-to-word order.  The same inputs always
    /// give the same stream, so this is the constructor for reproducible runs
    /// and known-answer tests.
    #[must_use]
    pub fn new(key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> Self {
        Self {
            cipher: ChaCha20::with_counter(key, nonce, counter),
            buf: [0u8; BLOCK_BYTES],
            offset: BLOCK_BYTES, // force a refill on first use
        }
    }

    /// Construct with a fresh key and nonce from the operating system RNG,
    /// starting at block counter 0.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        for chunk in key.chunks_exact_mut(4) {
            chunk.copy_from_slice(&os.next_u32().to_le_bytes());
        }
        for chunk in nonce.chunks_exact_mut(4) {
            chunk.copy_from_slice(&os.next_u32().to_le_bytes());
        }
        Self::new(&key, &nonce, 0)
    }
}

impl ByteBuffered<BLOCK_BYTES> for ChaCha20Rng {
    fn buffer(&self) -> &[u8; BLOCK_BYTES] {
        &self.buf
    }

    fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }

    fn refill(&mut self) {
        self.buf = self.cipher.keystream_block();
    }
}

impl Default for ChaCha20Rng {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for ChaCha20Rng {
    fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take_bytes::<4>())
    }
    fn next_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take_bytes::<8>())
    }
}

impl Drop for ChaCha20Rng {
    /// Wipe the buffered keystream on drop.  (The ChaCha20 key/nonce live
    /// inside `cipher`, whose own storage is managed by the cryptography crate.)
    fn drop(&mut self) {
        cryptography::zeroize_slice(&mut self.buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::hex;
    use crate::seed::K32;

    /// RFC 8439 §2.3.2's nonce.
    const NONCE_2_3_2: &str = "000000090000004a00000000";
    /// RFC 8439 §2.4.2's nonce.
    const NONCE_2_4_2: &str = "000000000000004a00000000";

    fn nonce(digits: &str) -> [u8; 12] {
        hex(digits).try_into().unwrap()
    }

    /// The next `n` words, each as the four little-endian bytes it was read
    /// from.
    fn word_bytes(rng: &mut ChaCha20Rng, n: usize) -> Vec<u8> {
        (0..n).flat_map(|_| rng.next_u32().to_le_bytes()).collect()
    }

    #[test]
    fn chacha20_rng_nonzero() {
        let mut rng = ChaCha20Rng::from_os_rng();
        let v: u64 = (0..8).map(|_| rng.next_u64()).fold(0, |a, b| a | b);
        assert_ne!(v, 0);
    }

    #[test]
    fn chacha20_rng_advances() {
        let mut rng = ChaCha20Rng::from_os_rng();
        let v0 = rng.next_u64();
        let v1 = rng.next_u64();
        assert_ne!(v0, v1);
    }

    #[test]
    fn chacha20_rng_refills_across_block_boundary() {
        let mut rng = ChaCha20Rng::from_os_rng();
        // Drain one full 64-byte block via u32 (16 calls) then read across boundary.
        for _ in 0..16 {
            let _ = rng.next_u32();
        }
        let v = rng.next_u32(); // triggers refill
        assert_ne!(v, 0xffff_ffff); // trivially non-constant
    }

    /// RFC 8439 §2.3.2: key 00:01:…:1f, nonce 00:00:00:09:00:00:00:4a:00:00:00:00,
    /// block count 1.  Sixteen `next_u32` calls return the printed "ChaCha
    /// state at the end of the ChaCha20 operation" in order, from `e4e7f110`
    /// to `4e3c50a2`, and their little-endian bytes are the printed serialized
    /// block.
    #[test]
    fn rfc8439_2_3_2_block_function() {
        const SERIALIZED_BLOCK: &str =
            "10f1e7e4d13b5915500fdd1fa32071c4c7d1f4c733c068030422aa9ac3d46c4e\
            d2826446079faa0914c2d705d98b02a2b5129cd1de164eb9cbd083e8a2503c4e";
        let mut rng = ChaCha20Rng::new(&K32, &nonce(NONCE_2_3_2), 1);
        let words: [u32; 16] = core::array::from_fn(|_| rng.next_u32());
        assert_eq!([words[0], words[15]], [0xe4e7_f110, 0x4e3c_50a2]);
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        assert_eq!(bytes, hex(SERIALIZED_BLOCK));
    }

    /// RFC 8439 §2.4.2: key 00:01:…:1f, nonce 00:00:00:00:00:00:00:4a:00:00:00:00,
    /// initial counter 1.  The printed 114-byte keystream spans the blocks at
    /// counters 1 and 2, so the 29 `next_u32` words that cover it cross a
    /// refill.
    #[test]
    fn rfc8439_2_4_2_keystream() {
        const KEYSTREAM: &str = "224f51f3401bd9e12fde276fb8631ded8c131f823d2c06e27e4fcaec9ef3cf78\
            8a3b0aa372600a92b57974cded2b9334794cba40c63e34cdea212c4cf07d41b7\
            69a6749f3f630f4122cafe28ec4dc47e26d4346d70b98c73f3e9c53ac40c5945\
            398b6eda1a832c89c167eacd901d7e2bf363";
        let want = hex(KEYSTREAM);
        let mut rng = ChaCha20Rng::new(&K32, &nonce(NONCE_2_4_2), 1);
        let got = word_bytes(&mut rng, want.len().div_ceil(4));
        assert_eq!(got[..want.len()], want[..]);
    }

    /// RFC 8439 Appendix A.1, test vectors #1 to #5: each key, nonce and block
    /// counter with its printed 64-byte keystream, read here through eight
    /// `next_u64` calls of eight little-endian bytes each.  The vectors differ
    /// from the all-zero inputs in one byte at most.
    #[test]
    fn rfc8439_appendix_a1_block_functions() {
        const ZERO_KEY: [u8; 32] = [0; 32];
        const ZERO_NONCE: [u8; 12] = [0; 12];
        let mut key_3 = ZERO_KEY;
        key_3[31] = 1;
        let mut key_4 = ZERO_KEY;
        key_4[1] = 0xff;
        let mut nonce_5 = ZERO_NONCE;
        nonce_5[11] = 2;
        let vectors: [([u8; 32], [u8; 12], u32, &str); 5] = [
            (
                ZERO_KEY,
                ZERO_NONCE,
                0,
                "76b8e0ada0f13d90405d6ae55386bd28bdd219b8a08ded1aa836efcc8b770dc7\
                da41597c5157488d7724e03fb8d84a376a43b8f41518a11cc387b669b2ee6586",
            ),
            (
                ZERO_KEY,
                ZERO_NONCE,
                1,
                "9f07e7be5551387a98ba977c732d080dcb0f29a048e3656912c6533e32ee7aed\
                29b721769ce64e43d57133b074d839d531ed1f28510afb45ace10a1f4b794d6f",
            ),
            (
                key_3,
                ZERO_NONCE,
                1,
                "3aeb5224ecf849929b9d828db1ced4dd832025e8018b8160b82284f3c949aa5a\
                8eca00bbb4a73bdad192b5c42f73f2fd4e273644c8b36125a64addeb006c13a0",
            ),
            (
                key_4,
                ZERO_NONCE,
                2,
                "72d54dfbf12ec44b362692df94137f328fea8da73990265ec1bbbea1ae9af0ca\
                13b25aa26cb4a648cb9b9d1be65b2c0924a66c54d545ec1b7374f4872e99f096",
            ),
            (
                ZERO_KEY,
                nonce_5,
                0,
                "c2c64d378cd536374ae204b9ef933fcd1a8b2288b3dfa49672ab765b54ee27c7\
                8a970e0e955c14f3a88e741b97c286f75f8fc299e8148362fa198a39531bed6d",
            ),
        ];
        for (n, (key, nonce, counter, keystream)) in vectors.iter().enumerate() {
            let mut rng = ChaCha20Rng::new(key, nonce, *counter);
            let got: Vec<u8> = (0..8).flat_map(|_| rng.next_u64().to_le_bytes()).collect();
            assert_eq!(got, hex(keystream), "RFC 8439 A.1 test vector #{}", n + 1);
        }
    }

    /// `cryptography::ChaCha20` wraps the 32-bit block counter, so the block
    /// after counter 2³² − 1 is the block at counter 0.
    #[test]
    fn block_counter_wraps_to_zero() {
        let mut wrapping = ChaCha20Rng::new(&K32, &[0u8; 12], u32::MAX);
        let _ = word_bytes(&mut wrapping, 16);
        let mut from_zero = ChaCha20Rng::new(&K32, &[0u8; 12], 0);
        assert_eq!(
            word_bytes(&mut wrapping, 16),
            word_bytes(&mut from_zero, 16)
        );
    }
}
