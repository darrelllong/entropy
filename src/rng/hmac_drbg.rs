//! HMAC_DRBG — NIST SP 800-90A Rev. 1 §10.1.2, instantiated with
//! HMAC-SHA-256, as an entropy generator over cryptography's mechanism.
//!
//! The mechanism (`HMAC_DRBG_Update`, Generate, the reseed interval and
//! wiping of K and V) is `cryptography::HmacDrbg`; this module seeds it,
//! defines the stream the battery reads, and adapts it to [`Rng`].
//!
//! - [`from_os_rng`](HmacDrbg::from_os_rng) seeds from 384 bits of OS entropy
//!   (entropy_input 32 bytes, nonce 16 bytes) with an empty personalization.
//! - The streaming [`Rng`] path makes one Generate request of 32 bytes per
//!   refill, with no additional input, so a long `next_u32` stream is a
//!   *sequence* of one-block Generate calls, each ending in an Update, not a
//!   single multi-block Generate.  For a CAVP-conformant single
//!   `Generate(N bits)` use [`HmacDrbg::generate`], the path the known-answer
//!   tests use.
//!
//! # Reseed interval
//! SP 800-90A Rev. 1 §10.1 Table 2 sets the maximum `reseed_interval` to 2⁴⁸
//! requests, each streaming refill being one, and §10.1.2.5 step 1 refuses a
//! request only once `reseed_counter > reseed_interval`.  This adapter never
//! reseeds, so a request after the 2⁴⁸-th panics; the mechanism's own tests
//! pin that boundary.  The test battery never approaches it.
//!
//! # Backtracking and prediction resistance
//! SP 800-90A §8.8 designs every DRBG mechanism for backtracking resistance:
//! `HMAC_DRBG_Update` runs after every streaming block and after each
//! [`generate`] request.  A memory compromise reveals K and V, and with them
//! all *future* output, but earlier output only as far as the current block
//! still held in the 32-byte output buffer.  There is **no prediction
//! resistance**: nothing reseeds, so a compromised state predicts every later
//! output.  Correct for a test harness; applications that need prediction
//! resistance should reseed the mechanism in cryptography directly.
//!
//! [`generate`]: HmacDrbg::generate
//!
//! For uniform-width access (all `next_u32` or all `next_u64`) all 256 bits
//! per block are used; mixing widths at a refill boundary silently discards
//! up to 7 trailing bytes before refilling.
//!
//! # References
//! NIST SP 800-90A Rev. 1, "Recommendation for Random Number Generation
//! Using Deterministic Random Bit Generators", §10.1.2, 2015.
//! [pubs/NIST-SP-800-90Ar1.pdf]
//!
//! # Author
//! NIST (specification); Darrell Long (Rust implementation).

use cryptography::DrbgError;

use super::{os::os_random, ByteBuffered, Rng};

/// HMAC-SHA-256 output, in bytes: one block and one streaming refill.
const OUT: usize = 32;

/// Entropy input drawn by [`HmacDrbg::from_os_rng`]: the 256-bit security
/// strength.
const ENTROPY_BYTES: usize = 32;

/// Nonce drawn by [`HmacDrbg::from_os_rng`]: half the security strength
/// (SP 800-90A §8.6.7).
const NONCE_BYTES: usize = 16;

/// HMAC_DRBG instantiated with HMAC-SHA-256 per NIST SP 800-90A §10.1.2.
pub struct HmacDrbg {
    core: cryptography::HmacDrbg,
    /// Output of the current streaming Generate request.
    buf: [u8; OUT],
    offset: usize,
}

/// The mechanism's refusal, as a panic: this adapter neither reseeds nor
/// splits a request.
fn refused(error: DrbgError) -> ! {
    panic!("HMAC_DRBG: {error}")
}

impl HmacDrbg {
    fn from_core(core: cryptography::HmacDrbg) -> Self {
        Self {
            core,
            buf: [0u8; OUT],
            offset: OUT, // force refill on first use
        }
    }

    /// Instantiate from OS entropy (entropy_input 32 bytes, nonce 16 bytes).
    ///
    /// # Panics
    /// Panics if the operating system's entropy source fails.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut seed = [0u8; ENTROPY_BYTES + NONCE_BYTES];
        os_random(&mut seed).expect("HMAC_DRBG: the operating system's entropy source failed");
        let (entropy_input, nonce) = seed.split_at(ENTROPY_BYTES);
        let core = cryptography::HmacDrbg::instantiate(entropy_input, nonce, &[])
            .unwrap_or_else(|e| refused(e));
        cryptography::zeroize_slice(&mut seed);
        Self::from_core(core)
    }

    /// Instantiate deterministically from explicit entropy input, nonce, and
    /// personalization string (SP 800-90A §10.1.2.3, seed_material =
    /// entropy_input ‖ nonce ‖ personalization_string).
    ///
    /// Exposed for known-answer testing and reproducible discrete use; the
    /// battery uses [`from_os_rng`](Self::from_os_rng).
    ///
    /// # Panics
    /// Panics if `entropy_input` is shorter than 32 bytes or `nonce` shorter
    /// than 16, below the 256-bit security strength.
    #[must_use]
    pub fn from_entropy(entropy_input: &[u8], nonce: &[u8], personalization: &[u8]) -> Self {
        let core = cryptography::HmacDrbg::instantiate(entropy_input, nonce, personalization)
            .unwrap_or_else(|e| refused(e));
        Self::from_core(core)
    }

    /// SP 800-90A §10.1.2.5 Generate: exactly `nbytes` as one discrete
    /// request, with no re-keying between its blocks and one Update at the
    /// end.  This is the CAVP-conformant path; the streaming [`Rng`] path
    /// re-keys after every 32-byte block.
    ///
    /// # Panics
    /// Panics past the reseed interval, or if `nbytes` exceeds 2¹⁹ bits.
    pub fn generate(&mut self, nbytes: usize, additional_input: &[u8]) -> Vec<u8> {
        let mut out = vec![0u8; nbytes];
        self.core
            .generate(&mut out, additional_input)
            .unwrap_or_else(|e| refused(e));
        out
    }
}

impl ByteBuffered<OUT> for HmacDrbg {
    fn buffer(&self) -> &[u8; OUT] {
        &self.buf
    }

    fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }

    /// One Generate request of one block, no additional input.
    fn refill(&mut self) {
        self.core
            .generate(&mut self.buf, &[])
            .unwrap_or_else(|e| refused(e));
    }
}

impl Default for HmacDrbg {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for HmacDrbg {
    fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take_bytes::<4>())
    }
    fn next_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take_bytes::<8>())
    }

    /// From the buffer, eight bytes per word; see [`ByteBuffered::fill_words`].
    fn fill_native(&mut self, bytes: &mut [u8]) {
        self.fill_words(bytes);
    }
}

impl Drop for HmacDrbg {
    /// Wipe the buffered output; the mechanism wipes K and V itself.
    fn drop(&mut self) {
        cryptography::zeroize_slice(&mut self.buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::hex;
    use cryptography::Sha256;

    /// ReturnedBitsLen of the known-answer tests, 1 024 bits.
    const KAT_BYTES: usize = 128;

    #[test]
    fn hmac_drbg_nonzero() {
        let mut rng = HmacDrbg::from_os_rng();
        let v: u64 = (0..8).map(|_| rng.next_u64()).fold(0, |a, b| a | b);
        assert_ne!(v, 0);
    }

    #[test]
    fn hmac_drbg_advances() {
        let mut rng = HmacDrbg::from_os_rng();
        let v0 = rng.next_u64();
        let v1 = rng.next_u64();
        assert_ne!(v0, v1);
    }

    #[test]
    fn hmac_drbg_update_changes_state() {
        // Two fresh instances from OS RNG should produce different streams.
        let mut a = HmacDrbg::from_os_rng();
        let mut b = HmacDrbg::from_os_rng();
        // With 256-bit entropy it's astronomically unlikely these collide.
        assert_ne!(a.next_u64(), b.next_u64());
    }

    /// EntropyInput of the CAVP HMAC_DRBG vector cited on
    /// `hmac_drbg_sha256_nist_drbgvs_kat`, which
    /// `hmac_drbg_sha256_additional_input_kat` also instantiates from.
    const DRBGVS_ENTROPY_INPUT: &str =
        "ca851911349384bffe89de1cbdc46e6831e44d34a4fb935ee285dd14b71a7488";

    /// Nonce of the same CAVP vector.
    const DRBGVS_NONCE: &str = "659ba96c601dc69fc902940805ec0ca8";

    /// NIST CAVP DRBGVS HMAC_DRBG known-answer test: CAVS 14.3 `HMAC_DRBG.rsp`
    /// from `drbgvectors_no_reseed.zip` in the CAVP DRBG test vectors.
    /// [pubs/NIST-CAVP-drbgtestvectors-no_reseed-HMAC_DRBG.rsp]
    ///
    /// Four sections of that file open with the same header,
    /// `[SHA-256] [PredictionResistance = False] [EntropyInputLen = 256]
    /// [NonceLen = 128] [PersonalizationStringLen = 0] [AdditionalInputLen = 0]
    /// [ReturnedBitsLen = 1024]`, and each has its own `COUNT = 0`.  The vector
    /// is in the first of them, whose header is at line 4104: its `COUNT = 0`
    /// record at line 4112, the one whose EntropyInput is
    /// `DRBGVS_ENTROPY_INPUT`.  Its Nonce is `DRBGVS_NONCE`, its
    /// PersonalizationString and both AdditionalInput fields are empty, and its
    /// ReturnedBits (prefix `e528e9ab…`) are the expected bits: two Generate
    /// calls, the second returned.  A from-spec SP 800-90A replica reproduced
    /// them.  This pins the HMAC_DRBG_Update / Generate math, not just "output
    /// advances".
    #[test]
    fn hmac_drbg_sha256_nist_drbgvs_kat() {
        let entropy = hex(DRBGVS_ENTROPY_INPUT);
        let nonce = hex(DRBGVS_NONCE);
        let mut drbg = HmacDrbg::from_entropy(&entropy, &nonce, &[]);
        let _ = drbg.generate(KAT_BYTES, &[]); // first Generate — discarded per DRBGVS
        let returned = drbg.generate(KAT_BYTES, &[]);
        let expected = hex(
            "e528e9abf2dece54d47c7e75e5fe302149f817ea9fb4bee6f4199697d04d5b89\
             d54fbb978a15b5c443c9ec21036d2460b6f73ebad0dc2aba6e624abf07745bc1\
             07694bb7547bb0995f70de25d6b29e2d3011bb19d27676c07162c8b5ccde0668\
             961df86803482cb37ed6d5c0bb8d50cf1f50d476aa0458bdaba806f48be9dcb8",
        );
        assert_eq!(returned, expected);
    }

    /// Personalization strings and additional input longer than the stack
    /// buffer for short messages (SP 800-90A permits up to 2³⁵ bits of each).
    /// The golden outputs come from an independent from-spec HMAC_DRBG replica
    /// (Python `hmac`/`hashlib`), so silently truncating either input would
    /// fail here.
    #[test]
    fn hmac_drbg_long_personalization_and_additional_input_kat() {
        let long_pers = [0xa5u8; 200];
        let mut a = HmacDrbg::from_entropy(&[1u8; 32], &[2u8; 16], &long_pers);
        assert_eq!(
            a.generate(OUT, &[]),
            hex("b075870331a47cbb0bb09b6bc44181ad8dad91363ba0cb309e7aafc62a96f1aa")
        );

        let long_add = [0x5au8; 1024];
        let mut b = HmacDrbg::from_entropy(&[1u8; 32], &[2u8; 16], &[]);
        assert_eq!(
            b.generate(OUT, &long_add),
            hex("3d00f0409313ca86990ac50c6376cb3a35589c4eb7c0a209bed5cd8ebc819391")
        );
    }

    /// Exercises the `generate` path *with* non-empty additional input (the
    /// DRBGVS KAT above uses empty input, leaving the two extra Update rounds
    /// untested).  Additional inputs 0x00..0x1f then 0x20..0x3f; golden bits
    /// reproduced by the same from-spec HMAC_DRBG replica.
    #[test]
    fn hmac_drbg_sha256_additional_input_kat() {
        let entropy = hex(DRBGVS_ENTROPY_INPUT);
        let nonce = hex(DRBGVS_NONCE);
        let a1: Vec<u8> = (0x00u8..0x20).collect();
        let a2: Vec<u8> = (0x20u8..0x40).collect();
        let mut drbg = HmacDrbg::from_entropy(&entropy, &nonce, &[]);
        let _ = drbg.generate(KAT_BYTES, &a1);
        let returned = drbg.generate(KAT_BYTES, &a2);
        let expected = hex(
            "f3acf1a72ab1036b7bd95ffd8c2d8e87944ecaef836e6911b17400fca3d69bc4\
             87f4db662fd6578e103230450a29e6941d0aec3e1db90451c18f6d659870420c\
             b445f361ba2f63e872d89c0a5b835493fec0d0e7e2d9ab4859afb652bbcc350a\
             27589fac10944dee9b34870798d5bb9ee024218642d74fa3c5833666a9b745ec",
        );
        assert_eq!(returned, expected);
    }

    /// Streaming [`Rng`] path golden: 4096 `next_u32` words (512 refills,
    /// each one HMAC step followed by `HMAC_DRBG_Update` with no provided
    /// data).  Values come from an independent Python `hmac`/`hashlib`
    /// replica of this streaming layout; the KATs above only reach
    /// [`HmacDrbg::generate`].
    #[test]
    fn hmac_drbg_streaming_path_golden() {
        let entropy: Vec<u8> = (0x00u8..0x20).collect();
        let nonce: Vec<u8> = (0x20u8..0x30).collect();
        let mut drbg = HmacDrbg::from_entropy(&entropy, &nonce, &[]);
        let words: Vec<u32> = (0..4096).map(|_| drbg.next_u32()).collect();
        assert_eq!(
            [words[0], words[7], words[8], words[255]],
            [0x8780_fb0f, 0x8768_d53a, 0x5676_7608, 0xe6f5_8492]
        );
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        assert_eq!(
            Sha256::digest(&bytes).to_vec(),
            hex("1a6cb87172228bbfb066badc2be0ca8a1a4c1b3efb572ee1e1a70be2e34e55a7")
        );
    }
}
