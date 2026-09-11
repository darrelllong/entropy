//! Generic stream-cipher pseudorandom number generator.
//!
//! Any type implementing [`cryptography::StreamCipher`] can be wrapped by
//! [`StreamRng`] to provide the [`super::Rng`] interface.  Stream ciphers
//! are inherently keystream generators; this wrapper simply XORs the keystream
//! into a zero-filled scratch buffer in 64-byte chunks and dispenses words
//! from that buffer.
//!
//! The 64-byte chunk size is a pragmatic choice: it matches ChaCha20's natural
//! block size and is a common multiple of the internal word widths of Rabbit
//! (16-byte state output), Salsa20 (64-byte block), Snow3G (4-byte word), and
//! ZUC-128 (4-byte word).
//!
//! Keystream bytes are consumed little-endian, consistent with the other
//! byte-backed generators in this crate, so what a word holds depends on how
//! each cipher lays out its keystream bytes:
//!
//! * Rabbit: `cryptography::Rabbit` writes each 128-bit block as 16 octets,
//!   most significant first, the octet order of RFC 4503's test vectors, so
//!   the first `next_u32` reads the RFC's first four printed octets
//!   little-endian.
//! * Salsa20: each 64-byte chunk is one `Salsa20_k(v, i)` block of the
//!   specification (§9, §10), whose words are little-endian, so `next_u32`
//!   returns the specification's output words in order.
//! * SNOW 3G and ZUC-128: both produce 32-bit keystream words (SNOW 3G §4.2
//!   `z_t`, ZUC §3.6.2 `Z`) and `cryptography` writes each word big-endian,
//!   so `next_u32` returns each keystream word with its bytes reversed.
//!
//! # Invariants
//!
//! `0 ≤ pos ≤ CHUNK`.  `pos == CHUNK` is the sentinel meaning "buffer
//! exhausted; refill required."  Construction sets `pos = CHUNK` so the first
//! call to `next_u32` always triggers a refill.
//!
//! `CHUNK` must be a multiple of 8 so that a run of `next_u64` calls, like a
//! run of `next_u32` calls, uses every keystream byte.
//!
//! # References
//! The stream ciphers the harness wraps, implemented in the sibling
//! `cryptography` crate.
//! * M. Boesgaard, M. Vesterager and E. Zenner, "A Description of the Rabbit
//!   Stream Cipher Algorithm," RFC 4503, May 2006.  [pubs/rfc4503-rabbit.txt]
//!   [Rabbit; the known-answer tests below use its Appendix A.2 vectors]
//! * M. Boesgaard, M. Vesterager, T. Christensen and E. Zenner, "The Stream
//!   Cipher Rabbit," eSTREAM description.
//!   [pubs/rabbit-estream-description.pdf]  [§2.6, the extraction scheme]
//! * D. J. Bernstein, "Salsa20 specification," 2005.
//!   <https://cr.yp.to/snuffle/spec.pdf>  [pubs/bernstein-2005-salsa20-spec.pdf]
//!   [the Salsa20 cipher; §9's two expansion examples are a known-answer test
//!   below]
//! * ETSI/SAGE, "Specification of the 3GPP Confidentiality and Integrity
//!   Algorithms UEA2 & UIA2, Document 2: SNOW 3G Specification," version 1.1.
//!   [pubs/etsi-sage-snow3g-spec-v1.1.pdf]  [the SNOW 3G cipher; its test data
//!   are in Document 3, which is not in `pubs/`]
//! * ETSI/SAGE, "Specification of the 3GPP Confidentiality and Integrity
//!   Algorithms 128-EEA3 & 128-EIA3, Document 2: ZUC Specification,"
//!   version 1.6.  [pubs/etsi-sage-zuc-spec-v1.6.pdf]  [the ZUC-128 cipher;
//!   its test data are in Document 3, which is not in `pubs/`]
//!
//! # Author
//! M. Boesgaard, M. Vesterager, T. Pedersen, J. Christiansen and O. Scavenius
//! (Rabbit, FSE 2003, as reference [6] of the eSTREAM description lists them;
//! that description is by Boesgaard, Vesterager, T. Christensen and E. Zenner,
//! and RFC 4503 by Boesgaard, Vesterager and Zenner); Daniel J. Bernstein
//! (Salsa20); ETSI SAGE (SNOW 3G and ZUC-128 specifications); Darrell Long
//! (Rust adapter).

use cryptography::StreamCipher;

use super::{ByteBuffered, Rng};

const CHUNK: usize = 64;
// Required invariant: CHUNK % 8 == 0.
const _: () = assert!(CHUNK.is_multiple_of(8), "CHUNK must be a multiple of 8");

/// Stream-cipher RNG wrapping any [`StreamCipher`].
///
/// The cipher must already be initialised (key and IV set) before wrapping.
/// After wrapping, callers only call [`Rng::next_u32`]; the underlying cipher
/// advances automatically as chunks are consumed.
///
/// # Note on `next_u64` and chunk boundaries
///
/// If `next_u32` and `next_u64` are interleaved, a `next_u64` call that finds
/// fewer than 8 bytes remaining will discard those trailing bytes and refill.
/// The output stream is therefore not guaranteed to be a contiguous prefix of
/// the underlying cipher's keystream when the two methods are mixed.
pub struct StreamRng<C: StreamCipher> {
    cipher: C,
    buf: [u8; CHUNK],
    pos: usize,
}

impl<C: StreamCipher> StreamRng<C> {
    /// Wrap an already-initialised stream cipher.
    pub fn new(cipher: C) -> Self {
        // pos == CHUNK forces a refill on the first next_u32() call.
        Self {
            cipher,
            buf: [0u8; CHUNK],
            pos: CHUNK,
        }
    }
}

impl<C: StreamCipher> ByteBuffered<CHUNK> for StreamRng<C> {
    fn buffer(&self) -> &[u8; CHUNK] {
        &self.buf
    }

    fn offset_mut(&mut self) -> &mut usize {
        &mut self.pos
    }

    fn refill(&mut self) {
        self.buf = [0u8; CHUNK];
        // fill() XORs keystream into buf; starting from zeros gives raw keystream.
        self.cipher.fill(&mut self.buf);
    }
}

impl<C: StreamCipher> Rng for StreamRng<C> {
    fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take_bytes::<4>())
    }

    fn next_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take_bytes::<8>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::hex;
    use crate::seed::{IV16, K16};
    use cryptography::{Rabbit, Salsa20, Snow3g, Zuc128};

    /// Known-answer test using RFC 4503 Appendix A.2, Test Vector 1.
    ///
    /// Rabbit with key = 0x00*16, IV = 0x00*8.
    /// Stream[0..7] = C6 A7 27 5E F8 54 95 D8  (RFC 4503 §A.2)
    /// As little-endian u64: u64::from_le_bytes([C6,A7,27,5E,F8,54,95,D8])
    ///                      = 0xD895_54F8_5E27_A7C6
    #[test]
    fn stream_rng_kat_rfc4503() {
        let key = [0u8; 16];
        let iv = [0u8; 8];
        let mut rng = StreamRng::new(Rabbit::new(&key, &iv));
        assert_eq!(
            rng.next_u64(),
            0xd895_54f8_5e27_a7c6,
            "First u64 must match RFC 4503 §A.2 Test Vector 1"
        );
    }

    #[test]
    fn stream_rng_advances() {
        let key = [0u8; 16];
        let iv = [0u8; 8];
        let mut rng = StreamRng::new(Rabbit::new(&key, &iv));
        let a = rng.next_u64();
        let b = rng.next_u64();
        assert_ne!(a, b, "consecutive Rabbit words should differ");
    }

    #[test]
    fn stream_rng_crosses_chunk_boundary() {
        let key = [0u8; 16];
        let iv = [0u8; 8];
        let mut rng = StreamRng::new(Rabbit::new(&key, &iv));
        // Sixteen next_u32() calls read bytes 0..64.  The 17th finds
        // pos + 4 > CHUNK, refills, and reads keystream bytes 64..68.
        for _ in 0..16 {
            let _ = rng.next_u32();
        }
        let mut keystream = [0u8; CHUNK + 4];
        Rabbit::new(&key, &iv).fill(&mut keystream);
        assert_eq!(
            rng.next_u32(),
            u32::from_le_bytes(keystream[CHUNK..].try_into().unwrap()),
            "the word after a refill must continue the keystream"
        );
    }

    /// RFC 4503 Appendix A.2: the all-zero master key with each of its three
    /// IVs, 48 octets apiece, read as twelve little-endian `next_u32` words.
    /// With `stream_rng_kat_rfc4503` above this pins the IV setup, not only
    /// the all-zero case.
    #[test]
    fn rabbit_rfc4503_appendix_a2_through_words() {
        let vectors: [(&str, &str); 3] = [
            (
                "0000000000000000",
                "c6a7275ef85495d87ccd5d376705b7ed5f29a6ac04f5efd47b8f293270dc4a8d\
                2ade822b29de6c1ee52bdb8a47bf8f66",
            ),
            (
                "c373f575c1267e59",
                "1fcd4eb9580012e2e0dccc9222017d6da75f4e10d12125017b2499ffed936f2e\
                ebc112c393e738392356bdd012029ba7",
            ),
            (
                "a6eb561ad2f41727",
                "445ad8c805858dbf70b6af23a151104d96c8f27947f42c5baeae67c6acc35b03\
                9fcbfc895fa71c17313df034f01551cb",
            ),
        ];
        for (iv, stream) in vectors {
            let iv: [u8; 8] = hex(iv).try_into().unwrap();
            let mut rng = StreamRng::new(Rabbit::new(&[0u8; 16], &iv));
            let want: Vec<u32> = hex(stream)
                .chunks_exact(4)
                .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
                .collect();
            let got: Vec<u32> = (0..want.len()).map(|_| rng.next_u32()).collect();
            assert_eq!(got, want, "RFC 4503 A.2, iv {iv:02x?}");
        }
    }

    /// Salsa20 specification §9, both examples: k₀ = (1, …, 16) and
    /// k₁ = (201, …, 216) make the 32-byte key, k₀ alone the 16-byte key, and
    /// n = (101, …, 116) is the 8-byte nonce followed by the 8-byte
    /// little-endian block counter (§10).  Each output, printed in decimal in
    /// the specification, comes back as sixteen little-endian `next_u32`
    /// words.
    #[test]
    fn salsa20_spec_section_9_expansion_examples() {
        const OUT_32_BYTE_KEY: [u8; 64] = [
            69, 37, 68, 39, 41, 15, 107, 193, 255, 139, 122, 6, 170, 233, 217, 98, 89, 144, 182,
            106, 21, 51, 200, 65, 239, 49, 222, 34, 215, 114, 40, 126, 104, 197, 7, 225, 197, 153,
            31, 2, 102, 78, 76, 176, 84, 245, 246, 184, 177, 160, 133, 130, 6, 72, 149, 119, 192,
            195, 132, 236, 234, 103, 246, 74,
        ];
        const OUT_16_BYTE_KEY: [u8; 64] = [
            39, 173, 46, 248, 30, 200, 82, 17, 48, 67, 254, 239, 37, 18, 13, 247, 241, 200, 61,
            144, 10, 55, 50, 185, 6, 47, 246, 253, 143, 86, 187, 225, 134, 85, 110, 246, 161, 163,
            43, 235, 231, 94, 171, 51, 145, 214, 112, 29, 14, 232, 5, 16, 151, 140, 183, 141, 171,
            9, 122, 181, 104, 182, 177, 193,
        ];
        let k0: Vec<u8> = (1..=16).collect();
        let k1: Vec<u8> = (201..=216).collect();
        let n: Vec<u8> = (101..=116).collect();
        let nonce: [u8; 8] = n[..8].try_into().unwrap();
        let counter = u64::from_le_bytes(n[8..].try_into().unwrap());
        let key_32 = [k0.as_slice(), k1.as_slice()].concat();
        for (key, want) in [(key_32, OUT_32_BYTE_KEY), (k0, OUT_16_BYTE_KEY)] {
            let mut rng = StreamRng::new(Salsa20::with_counter(&key, &nonce, counter));
            let got: Vec<u8> = (0..16).flat_map(|_| rng.next_u32().to_le_bytes()).collect();
            assert_eq!(got, want, "{}-byte key", key.len());
        }
    }

    /// SNOW 3G and ZUC-128 write each 32-bit keystream word big-endian, so
    /// `next_u32` returns the cipher's `next_word()` with its bytes reversed,
    /// here for the battery's key and IV across sixteen refills.
    #[test]
    fn snow3g_and_zuc_words_are_byte_reversed_keystream_words() {
        let mut snow = StreamRng::new(Snow3g::new(&K16, &IV16));
        let mut snow_words = Snow3g::new(&K16, &IV16);
        let mut zuc = StreamRng::new(Zuc128::new(&K16, &IV16));
        let mut zuc_words = Zuc128::new(&K16, &IV16);
        for _ in 0..4 * CHUNK {
            assert_eq!(snow.next_u32(), snow_words.next_word().swap_bytes());
            assert_eq!(zuc.next_u32(), zuc_words.next_word().swap_bytes());
        }
    }
}
