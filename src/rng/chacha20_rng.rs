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
//! **Output limit.** ChaCha20 uses a 32-bit block counter, so RFC 8439 §2.3
//! limits one key and nonce to 2³² blocks, 2³⁸ bytes or **256 GiB**, and a
//! stream started at counter `c` has 2³² − `c` of them.  `ChaCha20Rng` counts
//! the blocks it takes and panics ("ChaCha20Rng: block counter exhausted")
//! before it asks `cryptography::ChaCha20` for one more, rather than wrap to
//! block 0 and repeat keystream.  Because the check comes first, the panic
//! and its message are the same whether the cipher wraps its counter or
//! refuses to.  The test battery takes 16 million words from a generator plus
//! what its live-drawing tests draw (see TESTS.md), nowhere near 2³² blocks.
//! No reseed is implemented; for applications that may produce more than a
//! few GiB from a single key, construct a fresh `ChaCha20Rng::from_os_rng()`
//! or use the OS CSPRNG directly.
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

use super::{
    streams::{Advance, Streams},
    ByteBuffered, Rng, Seedable,
};

const BLOCK_BYTES: usize = 64;

/// Blocks one key and nonce address under RFC 8439 §2.3's 32-bit counter.
const BLOCKS_PER_NONCE: u64 = 1 << 32;

/// Key and nonce sizes of RFC 8439 §2.3: 256 and 96 bits.
const KEY_BYTES: usize = 32;
const NONCE_BYTES: usize = 12;

/// ChaCha20 stream cipher used as a CSPRNG.
///
/// Generates 64 bytes per ChaCha20 core invocation.  A read that needs a block
/// past counter 2³² − 1 panics rather than repeat keystream; see the module
/// docs.
pub struct ChaCha20Rng {
    cipher: ChaCha20,
    /// The key and nonce, kept so that a stream can be positioned and so that
    /// numbered streams can take their own nonces; wiped on drop.
    key: [u8; KEY_BYTES],
    nonce: [u8; NONCE_BYTES],
    buf: [u8; BLOCK_BYTES],
    offset: usize,
    /// Blocks the cipher may still produce: 2³² less the initial counter,
    /// less one per refill.
    blocks_left: u64,
}

impl ChaCha20Rng {
    /// Construct from an explicit 256-bit key, 96-bit nonce and initial block
    /// counter, the inputs of RFC 8439 §2.3 and §2.4.
    ///
    /// The first `next_u32` returns word 0 of the block at `counter`; the
    /// module docs give the full byte-to-word order.  The same inputs always
    /// give the same stream, so this is the constructor for reproducible runs
    /// and known-answer tests.  The stream holds 2³² − `counter` blocks, and
    /// the read that would need one more panics.
    #[must_use]
    pub fn new(key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> Self {
        Self {
            cipher: ChaCha20::with_counter(key, nonce, counter),
            key: *key,
            nonce: *nonce,
            buf: [0u8; BLOCK_BYTES],
            offset: BLOCK_BYTES, // force a refill on first use
            blocks_left: BLOCKS_PER_NONCE - u64::from(counter),
        }
    }
}

impl ByteBuffered<BLOCK_BYTES> for ChaCha20Rng {
    fn buffer(&self) -> &[u8; BLOCK_BYTES] {
        &self.buf
    }

    fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }

    /// Take the next block, refusing before the cipher is asked for a block
    /// past counter 2³² − 1.
    fn refill(&mut self) {
        assert!(
            self.blocks_left > 0,
            "ChaCha20Rng: block counter exhausted; RFC 8439 allows 2^32 blocks \
             per key and nonce, and another block would repeat keystream"
        );
        self.blocks_left -= 1;
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

    /// The buffered words first, then whole blocks written straight into
    /// `bytes` by the cipher, then the tail through the buffer again.  The
    /// bytes are the same as one `next_u64` at a time: the buffer holds a
    /// whole number of words, so nothing is discarded at its boundaries
    /// unless a `next_u32` left a half word, in which case the whole request
    /// goes through the buffer and that half word is discarded as usual.
    fn fill_native(&mut self, bytes: &mut [u8]) {
        const WORD: usize = size_of::<u64>();
        let buffered = BLOCK_BYTES - self.offset;
        if !buffered.is_multiple_of(WORD) {
            self.fill_words(bytes);
            return;
        }
        let (head, rest) = bytes.split_at_mut(buffered.min(bytes.len()));
        self.fill_words(head);
        let blocks = rest.len() / BLOCK_BYTES;
        let direct = blocks * BLOCK_BYTES;
        if blocks > 0 {
            assert!(
                self.blocks_left >= blocks as u64,
                "ChaCha20Rng: block counter exhausted; RFC 8439 allows 2^32 blocks \
                 per key and nonce, and another block would repeat keystream"
            );
            self.blocks_left -= blocks as u64;
            self.cipher.keystream(&mut rest[..direct]);
        }
        self.fill_words(&mut rest[direct..]);
    }
}

impl Drop for ChaCha20Rng {
    /// Wipe the buffered keystream, the key and the nonce on drop; the cipher
    /// wipes its own copy.
    fn drop(&mut self) {
        cryptography::zeroize_slice(&mut self.buf);
        cryptography::zeroize_slice(&mut self.key);
        cryptography::zeroize_slice(&mut self.nonce);
    }
}

/// Words of `next_u32` in one block.
const WORDS_PER_BLOCK: u64 = (BLOCK_BYTES / size_of::<u32>()) as u64;

impl Advance for ChaCha20Rng {
    /// Set the block counter and the offset within the block: the position
    /// in words is the blocks the cipher has produced times 16, less the
    /// words still buffered, plus `steps`.
    ///
    /// # Panics
    /// Panics if the position would pass the 2³² blocks one key and nonce
    /// address, the same limit a read hits.
    fn advance(&mut self, steps: u128) {
        let produced = BLOCKS_PER_NONCE - self.blocks_left;
        let buffered = ((BLOCK_BYTES - self.offset) / size_of::<u32>()) as u64;
        let position = u128::from(produced) * u128::from(WORDS_PER_BLOCK) - u128::from(buffered);
        let target = position + steps;
        let block = target / u128::from(WORDS_PER_BLOCK);
        assert!(
            block < u128::from(BLOCKS_PER_NONCE),
            "ChaCha20Rng: block counter exhausted; RFC 8439 allows 2^32 blocks \
             per key and nonce, and another block would repeat keystream"
        );
        let block = block as u64;
        let within = (target % u128::from(WORDS_PER_BLOCK)) as usize * size_of::<u32>();
        self.cipher.set_counter(block as u32);
        self.blocks_left = BLOCKS_PER_NONCE - block;
        if within == 0 {
            self.offset = BLOCK_BYTES;
        } else {
            self.buf = self.cipher.keystream_block();
            self.blocks_left -= 1;
            self.offset = within;
        }
    }
}

impl Streams for ChaCha20Rng {
    /// The keystream of the same key under the nonce with `index` XORed into
    /// its low eight bytes, from block 0: a separate 2³²-block sequence for
    /// each index, disjoint by construction, with stream 0 the nonce's own.
    fn stream(&self, index: u64) -> Self {
        let mut nonce = self.nonce;
        for (byte, mix) in nonce.iter_mut().zip(index.to_le_bytes()) {
            *byte ^= mix;
        }
        Self::new(&self.key, &nonce, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::streams::{Advance, Streams};

    /// An advance is the words it stands for, from every position within a
    /// block and across blocks, mixing widths, and a stream is the nonce's
    /// keystream from block 0.
    #[test]
    fn advances_match_stepping() {
        let key = [3u8; KEY_BYTES];
        let nonce = [5u8; NONCE_BYTES];
        for skip in [0u32, 1, 15, 16, 17] {
            for steps in [0u128, 1, 15, 16, 17, 1_000_003] {
                let mut stepped = ChaCha20Rng::new(&key, &nonce, 7);
                let mut jumped = ChaCha20Rng::new(&key, &nonce, 7);
                for _ in 0..skip {
                    let _ = stepped.next_u32();
                    let _ = jumped.next_u32();
                }
                for _ in 0..steps {
                    let _ = stepped.next_u32();
                }
                jumped.advance(steps);
                let next: Vec<u64> = (0..3).map(|_| stepped.next_u64()).collect();
                let jumped_next: Vec<u64> = (0..3).map(|_| jumped.next_u64()).collect();
                assert_eq!(jumped_next, next, "skip {skip}, {steps} steps");
            }
        }
        let base = ChaCha20Rng::new(&key, &nonce, 9);
        let mut own = base.stream(0);
        let mut fresh = ChaCha20Rng::new(&key, &nonce, 0);
        assert_eq!(own.next_u64(), fresh.next_u64());
        let mut other_nonce = nonce;
        other_nonce[0] ^= 6;
        let mut other = ChaCha20Rng::new(&key, &other_nonce, 0);
        assert_eq!(base.stream(6).next_u64(), other.next_u64());
    }
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
        // A block is sixteen words; the seventeenth comes from the next block.
        for _ in 0..16 {
            let _ = rng.next_u32();
        }
        let v = rng.next_u32();
        assert_ne!(v, 0xffff_ffff);
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

    /// RFC 8439 §2.3 allows one key and nonce 2³² blocks.  Started at the last
    /// counter value, `new` serves exactly one block: sixteen words, equal to
    /// the cipher's own block at counter 2³² − 1.
    #[test]
    fn last_counter_value_serves_one_block() {
        let mut rng = ChaCha20Rng::new(&K32, &[0u8; 12], u32::MAX);
        let block = ChaCha20::with_counter(&K32, &[0u8; 12], u32::MAX).keystream_block();
        assert_eq!(word_bytes(&mut rng, 16), block);
    }

    /// The seventeenth read from that stream would need the block after
    /// counter 2³² − 1, so it panics with this module's message instead of
    /// wrapping to block 0 and repeating keystream.  The check runs before the
    /// cipher is asked for that block, so the panic is the same whether
    /// `cryptography::ChaCha20` wraps its counter or panics with its own
    /// message.
    #[test]
    #[should_panic(expected = "ChaCha20Rng: block counter exhausted")]
    fn read_past_the_last_counter_value_panics() {
        let mut rng = ChaCha20Rng::new(&K32, &[0u8; 12], u32::MAX);
        let _ = word_bytes(&mut rng, 16);
        let _ = rng.next_u32();
    }
}
