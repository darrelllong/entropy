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
//!   are in Document 3, Implementors' Test Data, version 1.1,
//!   [pubs/etsi-sage-snow3g-testdata-v1.1.doc]]
//! * ETSI/SAGE, "Specification of the 3GPP Confidentiality and Integrity
//!   Algorithms 128-EEA3 & 128-EIA3, Document 2: ZUC Specification,"
//!   version 1.6.  [pubs/etsi-sage-zuc-spec-v1.6.pdf]  [the ZUC-128 cipher;
//!   its test data are in Document 3, Implementor's Test Data, version 1.1,
//!   [pubs/etsi-sage-zuc-testdata-v1.1.pdf]]
//!
//! # Author
//! M. Boesgaard, M. Vesterager, T. Pedersen, J. Christiansen and O. Scavenius
//! (Rabbit, FSE 2003, as reference \[6\] of the eSTREAM description lists them;
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

    /// One keystream-generator test set from an ETSI/SAGE "Document 3:
    /// Implementors' Test Data": the key and IV octets the generator takes, as
    /// the document prints them, and the printed keystream words.
    struct KeystreamSet {
        section: &'static str,
        set: u32,
        key: &'static str,
        iv: &'static str,
        /// Runs of printed words: each entry holds the octets of `z_t`,
        /// `z_(t+1)`, … concatenated, from its 1-based index `t`.
        words: &'static [(usize, &'static str)],
        /// A UEA2 set's LENGTH in bits, where its run from `z1` stops;
        /// `None` when every printed word is whole.
        length: Option<usize>,
    }

    /// ZUC Document 3 §3.3, Test Set 1, prints its Key and its IV each as
    /// sixteen `00` octets.
    const OCTETS_00: &str = "00000000000000000000000000000000";

    /// ZUC Document 3 §3.4, Test Set 2, prints its Key and its IV each as
    /// sixteen `ff` octets.
    const OCTETS_FF: &str = "ffffffffffffffffffffffffffffffff";

    /// K0 ‖ K1 ‖ K2 ‖ K3 of SNOW 3G Document 3 §4.3, UEA2 Test Set 1, and
    /// §5.3, UIA2 Test Set 1, whose CK and IK are the same key.
    const UEA2_UIA2_SET_1_KEY: &str = "4881ff48952c491082c5b3002bd6459f";

    /// IV0 ‖ IV1 ‖ IV2 ‖ IV3 of SNOW 3G Document 3 §5.7 and §5.8, UIA2 Test
    /// Sets 5 and 6, which share COUNT-I, FRESH and DIRECTION.
    const UIA2_SETS_5_AND_6_IV: &str = "6b22f737a96f393c6b227737296f393c";

    /// SNOW 3G: UEA2 & UIA2 Document 3, §3.3–§3.6, Test Sets 1–4.
    /// [pubs/etsi-sage-snow3g-testdata-v1.1.doc]
    const SNOW3G_DOCUMENT_3: [KeystreamSet; 4] = [
        KeystreamSet {
            section: "3.3",
            set: 1,
            key: "2bd6459f82c5b300952c49104881ff48",
            iv: "ea024714ad5c4d84df1f9b251c0bf45f",
            words: &[(1, "abee9704"), (2, "7ac31373")],
            length: None,
        },
        KeystreamSet {
            section: "3.4",
            set: 2,
            key: "8ce33e2cc3c0b5fc1f3de8a6dc66b1f3",
            iv: "d3c5d592327fb11cde551988ceb2f9b7",
            words: &[(1, "eff8a342"), (2, "f751480f")],
            length: None,
        },
        KeystreamSet {
            section: "3.5",
            set: 3,
            key: "4035c6680af8c6d1a8ff8667b1714013",
            iv: "62a540981ba6f9b74592b0e78690f71b",
            words: &[(1, "a8c874a9"), (2, "7ae7c4f8")],
            length: None,
        },
        KeystreamSet {
            section: "3.6",
            set: 4,
            key: "0ded7263109cf92e3352255a140e0f76",
            iv: "6b68079a41a7c4c91befd79f7fdcc233",
            words: &[
                (1, "d712c05c"),
                (2, "a937c2a6"),
                (3, "eb7eaae3"),
                (2500, "9c0db3aa"),
            ],
            length: None,
        },
    ];

    /// SNOW 3G: UEA2 & UIA2 Document 3, §4.3–§4.7, UEA2 Test Sets 1–5: each
    /// set's K0–K3 and IV0–IV3 tables and its Keystream column, whose word
    /// number 0 is `z1`, with the LENGTH it stops at.
    /// [pubs/etsi-sage-snow3g-testdata-v1.1.doc]
    const SNOW3G_DOCUMENT_3_UEA2: [KeystreamSet; 5] = [
        KeystreamSet {
            section: "4.3",
            set: 1,
            key: UEA2_UIA2_SET_1_KEY,
            iv: "6400000072a4f20f6400000072a4f20f",
            words: &[(
                1,
                "f22db45b37e71c5b4eb6f404cd886c159dca27b1f062af46f8e2f5878976e8b8\
                 33e2b848e798969d85e5961a057983f110f5507671185285d53ced16fd580500\
                 7bee12be1c5c52ec78c12e8ac5b1b9d53bf90900df06df633c3c15d5c270de52\
                 fb4d09c0",
            )],
            length: Some(798),
        },
        KeystreamSet {
            section: "4.4",
            set: 2,
            key: "e96056957c36ea559e720c2aefa8b222",
            iv: "c0000000e28bcf7bc0000000e28bcf7b",
            words: &[(
                1,
                "f0cb07fb6e4571cfa691ab3f3f1a7bb9b4713f3cb592ac3a79af82a83627baab\
                 927d630849000249d0619e91196b16a7114992d826f421e60b1b119800fecb24",
            )],
            length: Some(510),
        },
        KeystreamSet {
            section: "4.5",
            set: 3,
            key: "1010d8524ea5f1454c0d51205acb1d64",
            iv: "1c000000fa556b261c000000fa556b26",
            words: &[(1, "1793752f8a3ffdaf0503d43eaec7ae")],
            length: Some(120),
        },
        KeystreamSet {
            section: "4.6",
            set: 4,
            key: "0af8c6d14035c668327fb11cd3c5d592",
            iv: "2c000000398a59b42c000000398a59b4",
            words: &[(
                1,
                "0080d71e902835ad7ba22d72abcbf214298f7eec685d340bbd945260d2777540",
            )],
            length: Some(253),
        },
        KeystreamSet {
            section: "4.7",
            set: 5,
            key: "e8e36566ecbf652b4c83706e6090eae0",
            iv: "4800000072a4f20f4800000072a4f20f",
            words: &[(
                1,
                "180aa00e09f7d155ecf028391355927e3bc59bd9d97d9bcbcd18f5fa25709b41\
                 612a0c4a6d75d36dae38ceb774daaaadb056fb8e5a935f8293d4ba2857c0fe05\
                 7372b4f24031d316312c8a0bae56e26e907834e73bb4b4ffc8ed7110fb0970eb\
                 db52c0c8e8b2ae04a8",
            )],
            length: Some(837),
        },
    ];

    /// SNOW 3G: UEA2 & UIA2 Document 3, §5.3–§5.8, UIA2 Test Sets 1–6: each
    /// set's K0–K3 and IV0–IV3 tables and its `z1`–`z5`.
    /// [pubs/etsi-sage-snow3g-testdata-v1.1.doc]
    const SNOW3G_DOCUMENT_3_UIA2: [KeystreamSet; 6] = [
        KeystreamSet {
            section: "5.3",
            set: 1,
            key: UEA2_UIA2_SET_1_KEY,
            iv: "05d2ec4938a6f05605d2ec4938a6f056",
            words: &[(1, "dc0d53252a5d31907e1b8e2825ec4caa63d9c77c")],
            length: None,
        },
        KeystreamSet {
            section: "5.4",
            set: 2,
            key: "5e6de7b7cd9f979428201cafd42f6824",
            iv: "a4f258e2bedc87e2a4f2d8e23edc87e2",
            words: &[(1, "670e29de2ad6de7ea42ad048407a24ac20f86070")],
            length: None,
        },
        KeystreamSet {
            section: "5.5",
            set: 3,
            key: "d81b8fab83a3186928936cc4fdb9cfdf",
            iv: "9838703ab6af61449838f03a36af6144",
            words: &[(1, "b39afb5d53aa27d456a1c4aecb68f91abf27347b")],
            length: None,
        },
        KeystreamSet {
            section: "5.6",
            set: 4,
            key: "e22ad57e1e2698d2b22bfff9c736c6aa",
            iv: "039768fd94793e410397e8fd14793e41",
            words: &[(1, "45898e828f27eb98e3230709a00cb70a8f75ac4b")],
            length: None,
        },
        KeystreamSet {
            section: "5.7",
            set: 5,
            key: "b3120ffdb2cf6af4e73eaf2ef4ebec69",
            iv: UIA2_SETS_5_AND_6_IV,
            words: &[(1, "991488471c790308662d90aafac592d2058bea75")],
            length: None,
        },
        KeystreamSet {
            section: "5.8",
            set: 6,
            key: "f4ebec69e73eaf2eb2cf6af4b3120ffd",
            iv: UIA2_SETS_5_AND_6_IV,
            words: &[(1, "ec81b3c23ccf818761f763ff4ba3d37a12c6f4ac")],
            length: None,
        },
    ];

    /// ZUC: 128-EEA3 & 128-EIA3 Document 3, §3.3–§3.6, Test Sets 1–4.
    /// [pubs/etsi-sage-zuc-testdata-v1.1.pdf]
    const ZUC128_DOCUMENT_3: [KeystreamSet; 4] = [
        KeystreamSet {
            section: "3.3",
            set: 1,
            key: OCTETS_00,
            iv: OCTETS_00,
            words: &[(1, "27bede74"), (2, "018082da")],
            length: None,
        },
        KeystreamSet {
            section: "3.4",
            set: 2,
            key: OCTETS_FF,
            iv: OCTETS_FF,
            words: &[(1, "0657cfa0"), (2, "7096398b")],
            length: None,
        },
        KeystreamSet {
            section: "3.5",
            set: 3,
            key: "3d4c4be96a82fdaeb58f641db17b455b",
            iv: "84319aa8de6915ca1f6bda6bfbd8c766",
            words: &[(1, "14f1c272"), (2, "3279c419")],
            length: None,
        },
        KeystreamSet {
            section: "3.6",
            set: 4,
            key: "4d320bfad4c285bfd6b8bd00f39d8b41",
            iv: "52959daba0bf176ece2dc315049eb574",
            words: &[(1, "ed4400e7"), (2, "0633e5c5"), (2000, "7a574cdb")],
            length: None,
        },
    ];

    /// Check `sets` through `StreamRng` over the cipher that `new` builds.
    ///
    /// Both documents print every value most significant octet first
    /// (Document 3 §2.3 of each), and both ciphers take key and IV octets in
    /// printed order: SNOW 3G's `k0` and `IV0` are the first four octets,
    /// loaded big-endian (SNOW 3G Document 2 §4.1; its Document 3 §2.3 splits
    /// `K = 0123456789ABCDEF…` into `K0 = 01234567`, …), and ZUC's `k_i` and
    /// `iv_i` are the `i`-th octets (ZUC Document 2 §3.5, `s_i = k_i ‖ d_i ‖
    /// iv_i`).  So the key and IV go in as printed.
    ///
    /// The words do not.  `cryptography` writes each 32-bit keystream word
    /// (SNOW 3G Document 2 §4.2 `z_t`, ZUC Document 2 §3.6.2 `Z`) big-endian,
    /// so the byte stream is the printed octets in printed order; SNOW 3G
    /// Test Set 1 begins `AB EE 97 04 7A C3 13 73`.  `StreamRng` reads each
    /// `next_u32` as the next four bytes little-endian, and its 64-byte chunk
    /// holds sixteen whole words, so the `t`-th `next_u32` is `z_t` with its
    /// octets reversed: `z1 = AB EE 97 04` comes back as `0x0497_EEAB`.  A
    /// first `next_u64` reads the eight bytes of `z1 ‖ z2` little-endian, the
    /// 64-bit big-endian `z1 ‖ z2` reversed.  The expected values below are
    /// computed that way from the printed words, not read from either crate.
    ///
    /// A UEA2 Keystream column covers only LENGTH bits.  SNOW 3G Document 3
    /// §2.4 lets the last octet of a stream carry up to seven low bits that
    /// LENGTH ignores, and §4 prints just the octets LENGTH reaches, ignored
    /// bits as zeros: Test Set 1's 798 bits end `FB4D09C0`, Test Set 3's 120
    /// end `AEC7AE`.  So a printed word counts for its first `bits` bits, 32
    /// except in such a last word, and only those bits are compared.
    fn assert_document_3_keystream<C: StreamCipher>(
        document: &str,
        sets: &[KeystreamSet],
        new: fn(&[u8; 16], &[u8; 16]) -> C,
    ) {
        for set in sets {
            let label = format!("{document} §{} Test Set {}", set.section, set.set);
            let key: [u8; 16] = hex(set.key).try_into().unwrap();
            let iv: [u8; 16] = hex(set.iv).try_into().unwrap();
            // (t, printed z_t left-aligned in 32 bits, its significant bits)
            let mut printed: Vec<(usize, u32, usize)> = Vec::new();
            for &(first, run) in set.words {
                for (i, octets) in hex(run).chunks(4).enumerate() {
                    let t = first + i;
                    let bits = set
                        .length
                        .map_or(32, |length| length.saturating_sub(32 * (t - 1)).min(32));
                    assert_eq!(octets.len(), bits.div_ceil(8), "{label}: width of z{t}");
                    let mut word = [0u8; 4];
                    word[..octets.len()].copy_from_slice(octets);
                    printed.push((t, u32::from_be_bytes(word), bits));
                }
            }
            if let Some(length) = set.length {
                let bits: usize = printed.iter().map(|&(_, _, bits)| bits).sum();
                assert_eq!(bits, length, "{label}: printed bits against LENGTH");
            }
            let last = printed.iter().map(|&(t, _, _)| t).max().unwrap();
            let mut rng = StreamRng::new(new(&key, &iv));
            let got: Vec<u32> = (0..last).map(|_| rng.next_u32()).collect();
            for &(t, z, bits) in &printed {
                // `keep` selects z_t's first `bits` bits; reversed, it selects
                // the same bits of the t-th next_u32.
                let keep = u32::MAX << (32 - bits);
                assert_eq!(
                    got[t - 1] & keep.swap_bytes(),
                    (z & keep).swap_bytes(),
                    "{label}, z{t}"
                );
            }
            let whole = |t: usize| {
                let &(_, z, bits) = printed.iter().find(|&&(at, _, _)| at == t).unwrap();
                assert_eq!(bits, 32, "{label}: z{t} must be whole");
                z
            };
            let z1_z2 = (u64::from(whole(1)) << 32) | u64::from(whole(2));
            assert_eq!(
                StreamRng::new(new(&key, &iv)).next_u64(),
                z1_z2.swap_bytes(),
                "{label}, z1 ‖ z2 as next_u64"
            );
        }
    }

    /// SNOW 3G against ETSI/SAGE, "Specification of the 3GPP Confidentiality
    /// and Integrity Algorithms UEA2 & UIA2, Document 3: Implementors' Test
    /// Data," version 1.1, 25 October 2012.
    /// [pubs/etsi-sage-snow3g-testdata-v1.1.doc]  [§3.3–§3.6, Test Sets 1–4]
    /// Section 3 is the bare SNOW 3G generator: `z1` and `z2` for each set,
    /// and for Test Set 4 also `z3` and `z2500`, which §3.6 clocks far enough
    /// to use every entry of the specification's tables.  Sections 4 and 5,
    /// the UEA2 and UIA2 test data, print the generator's keystream words as
    /// well, and the two tests below pin them.  The key, IV and word strings
    /// in `SNOW3G_DOCUMENT_3` were transcribed by script from the document's
    /// `textutil` plain-text and HTML conversions, which agree, not typed by
    /// hand.
    #[test]
    fn snow3g_etsi_sage_document_3_keystream_test_sets() {
        assert_document_3_keystream("SNOW 3G Document 3", &SNOW3G_DOCUMENT_3, Snow3g::new);
    }

    /// SNOW 3G against the UEA2 test data of the same Document 3.
    /// [pubs/etsi-sage-snow3g-testdata-v1.1.doc]  [§4.3–§4.7, Test Sets 1–5]
    /// Each set prints the generator's key words K0–K3 and IV words IV0–IV3,
    /// which UEA2 derives from CK, COUNT-C, BEARER and DIRECTION, and a
    /// Keystream column that §4.2 calls "the 32-bit output from SNOW 3G",
    /// numbered from 0, so word number 0 is `z1`.  The column stops at LENGTH
    /// bits: 798, 510, 120, 253 and 837.  The strings in
    /// `SNOW3G_DOCUMENT_3_UEA2` were transcribed by script from the document's
    /// `textutil` plain text and, separately, its HTML table cells, which
    /// agree, not typed by hand.  The script also checked every keystream
    /// octet against the printed plaintext and enc/dec data, and the K and IV
    /// words against CK, COUNT-C, BEARER and DIRECTION.
    #[test]
    fn snow3g_etsi_sage_document_3_uea2_keystream_columns() {
        assert_document_3_keystream("SNOW 3G Document 3", &SNOW3G_DOCUMENT_3_UEA2, Snow3g::new);
    }

    /// SNOW 3G against the UIA2 test data of the same Document 3.
    /// [pubs/etsi-sage-snow3g-testdata-v1.1.doc]  [§5.3–§5.8, Test Sets 1–6]
    /// Each set prints the generator's K0–K3 and IV0–IV3, which UIA2 derives
    /// from IK, COUNT-I, FRESH and DIRECTION, and its keystream words
    /// `z1`–`z5` (§5.2).  The strings in `SNOW3G_DOCUMENT_3_UIA2` were
    /// transcribed by script from the document's `textutil` plain text and,
    /// separately, its HTML table cells, which agree, not typed by hand.  The
    /// script also checked `z1 ‖ z2` and `z3 ‖ z4` against each set's printed
    /// P and Q, `z5` against its MAC-I, which is `z5` XOR the first word of
    /// the final EVAL, and the K and IV words against IK, COUNT-I, FRESH and
    /// DIRECTION.
    #[test]
    fn snow3g_etsi_sage_document_3_uia2_keystream_words() {
        assert_document_3_keystream("SNOW 3G Document 3", &SNOW3G_DOCUMENT_3_UIA2, Snow3g::new);
    }

    /// ZUC-128 against ETSI/SAGE, "Specification of the 3GPP Confidentiality
    /// and Integrity Algorithms 128-EEA3 & 128-EIA3, Document 3: Implementor's
    /// Test Data," version 1.1, 4 January 2011.
    /// [pubs/etsi-sage-zuc-testdata-v1.1.pdf]  [§3.3–§3.6, Test Sets 1–4]
    /// Section 3 is the bare ZUC generator: `z1` and `z2` for each set, and
    /// for Test Set 4 also `z2000`, the last word of the 125th 64-byte chunk.
    /// Sections 4 and 5 print 128-EEA3 plaintexts and ciphertexts and
    /// 128-EIA3 MACs but no keystream words, and are not used.  The key, IV
    /// and word strings in `ZUC128_DOCUMENT_3` were transcribed by script from
    /// the document's `pdftotext -layout` output, checked against renderings
    /// of the pages that print them, not typed by hand.
    #[test]
    fn zuc128_etsi_sage_document_3_keystream_test_sets() {
        assert_document_3_keystream("ZUC Document 3", &ZUC128_DOCUMENT_3, Zuc128::new);
    }
}
