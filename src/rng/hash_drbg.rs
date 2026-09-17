//! Hash_DRBG — NIST SP 800-90A Rev. 1 §10.1.1, instantiated with SHA-256,
//! as an entropy generator over cryptography's mechanism.
//!
//! The mechanism (Hash_df, Hashgen, the V and C updates, the reseed interval
//! and wiping of V and C) is `cryptography::HashDrbg`; this module seeds it,
//! defines the stream the battery reads, and adapts it to [`Rng`].
//!
//! The streaming [`Rng`] path makes one Generate request of `GENERATE_SIZE`
//! (256) bytes per refill, which is Hashgen of eight blocks followed by one
//! §10.1.1.4 update, and serves those bytes in order.  For a CAVP-conformant
//! single `Generate(N bits)` use [`HashDrbg::generate`], the path the
//! known-answer tests use.
//!
//! For uniform-width access (all `next_u32` or all `next_u64`) all 256 bits
//! per block are used; mixing widths at a refill boundary silently discards
//! up to 7 trailing bytes before refilling.
//!
//! # Reseed interval
//! SP 800-90A Rev. 1 §10.1 Table 2 sets the maximum `reseed_interval` to 2⁴⁸
//! requests, and §10.1.1.4 step 1 refuses a request only once
//! `reseed_counter > reseed_interval`.  This adapter never reseeds, so a
//! request after the 2⁴⁸-th panics; the mechanism's own tests pin that
//! boundary.  The test battery never approaches it.
//!
//! # Backtracking and prediction resistance
//! SP 800-90A §8.8 designs every DRBG mechanism for backtracking resistance:
//! after each request V becomes `V + Hash(0x03 ‖ V) + C + reseed_counter`.
//! A memory compromise reveals V and C, and with them all *future* output,
//! but earlier output only as far as the bytes of the current refill still
//! held in the 256-byte output buffer.  There is **no prediction
//! resistance**: nothing reseeds, so a compromised state predicts every later
//! output.  Correct for a test harness; applications that need prediction
//! resistance should reseed the mechanism in cryptography directly.
//!
//! # References
//! NIST SP 800-90A Rev. 1, "Recommendation for Random Number Generation
//! Using Deterministic Random Bit Generators", §10.1.1, 2015.
//! [pubs/NIST-SP-800-90Ar1.pdf]
//!
//! # Author
//! NIST (specification); Darrell Long (Rust implementation).

use cryptography::{cprng::hash_drbg::SEEDLEN, DrbgError};

use super::{os::os_random, ByteBuffered, Rng};

/// SHA-256 output, in bytes: one Hashgen block.
const OUTLEN: usize = 32;

/// Hashgen blocks per streaming refill.
const GENERATE_BLOCKS: usize = 8;

/// Bytes per streaming refill: one Generate request.
const GENERATE_SIZE: usize = OUTLEN * GENERATE_BLOCKS;

/// Nonce bytes drawn by [`HashDrbg::from_os_rng`], half the 256-bit security
/// strength (SP 800-90A §8.6.7).
const NONCE_BYTES: usize = 16;

/// Hash_DRBG instantiated with SHA-256 per NIST SP 800-90A §10.1.1.
pub struct HashDrbg {
    core: cryptography::HashDrbg,
    /// Buffered output of one streaming Generate request.
    buf: [u8; GENERATE_SIZE],
    offset: usize,
}

/// The mechanism's refusal, as a panic: this adapter neither reseeds nor
/// splits a request.
fn refused(error: DrbgError) -> ! {
    panic!("Hash_DRBG: {error}")
}

impl HashDrbg {
    fn from_core(core: cryptography::HashDrbg) -> Self {
        Self {
            core,
            buf: [0u8; GENERATE_SIZE],
            offset: GENERATE_SIZE, // force refill on first use
        }
    }

    /// Instantiate from OS entropy: a `SEEDLEN`-byte (55) entropy input and a
    /// 16-byte nonce, no personalization.
    ///
    /// # Panics
    /// Panics if the operating system's entropy source fails.
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut seed = [0u8; SEEDLEN + NONCE_BYTES];
        os_random(&mut seed).expect("Hash_DRBG: the operating system's entropy source failed");
        let (entropy_input, nonce) = seed.split_at(SEEDLEN);
        let core = cryptography::HashDrbg::instantiate(entropy_input, nonce, &[])
            .unwrap_or_else(|e| refused(e));
        cryptography::zeroize_slice(&mut seed);
        Self::from_core(core)
    }

    /// Instantiate deterministically from explicit entropy input, nonce, and
    /// personalization string (SP 800-90A §10.1.1.2, seed_material =
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
        let core = cryptography::HashDrbg::instantiate(entropy_input, nonce, personalization)
            .unwrap_or_else(|e| refused(e));
        Self::from_core(core)
    }

    /// SP 800-90A §10.1.1.4 Generate: exactly `nbytes` as one discrete
    /// request, the CAVP-conformant path.  It deliberately differs from the
    /// streaming [`Rng`] path, which requests 256 bytes per refill.
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

impl ByteBuffered<GENERATE_SIZE> for HashDrbg {
    fn buffer(&self) -> &[u8; GENERATE_SIZE] {
        &self.buf
    }

    fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }

    /// One Generate request of `GENERATE_SIZE` bytes, no additional input.
    fn refill(&mut self) {
        self.core
            .generate(&mut self.buf, &[])
            .unwrap_or_else(|e| refused(e));
    }
}

impl Default for HashDrbg {
    fn default() -> Self {
        Self::from_os_rng()
    }
}

impl Rng for HashDrbg {
    fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take_bytes::<4>())
    }
    fn next_u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take_bytes::<8>())
    }

    /// From the buffer, eight bytes per word.
    fn fill_native(&mut self, bytes: &mut [u8]) {
        self.fill_words(bytes);
    }
}

impl Drop for HashDrbg {
    /// Wipe the buffered output; the mechanism wipes V and C itself.
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
    fn hash_drbg_nonzero() {
        let mut rng = HashDrbg::from_os_rng();
        let v: u64 = (0..8).map(|_| rng.next_u64()).fold(0, |a, b| a | b);
        assert_ne!(v, 0);
    }

    #[test]
    fn hash_drbg_advances() {
        let mut rng = HashDrbg::from_os_rng();
        let v0 = rng.next_u64();
        let v1 = rng.next_u64();
        assert_ne!(v0, v1);
    }

    /// Hash_DRBG SHA-256 known-answer test (no reseed, empty personalization
    /// and additional input, two Generate calls of 1024 bits, the second
    /// returned).  Entropy input = bytes 0x00..0x36, nonce = 0x40..0x4f.  The
    /// expected bits were independently reproduced from the SP 800-90A §10.1.1
    /// pseudocode by a separate replica, cross-implementation-validating the
    /// Hashgen counter, Hash_df, and V/C update arithmetic.
    #[test]
    fn hash_drbg_sha256_spec_replica_kat() {
        let entropy: Vec<u8> = (0x00u8..0x37).collect();
        let nonce: Vec<u8> = (0x40u8..0x50).collect();
        let mut drbg = HashDrbg::from_entropy(&entropy, &nonce, &[]);
        let _ = drbg.generate(KAT_BYTES, &[]); // first Generate — discarded
        let returned = drbg.generate(KAT_BYTES, &[]);
        let expected = hex(
            "55338e1e62a2de3b061dd4c932ee89d2d1b9db8192cf88db37b50106080c10e1\
             56a147c3a0d0ba4045e2d21e39fad4e5aa155c4a19effdda2426531733b1ba59\
             e45e3e2aef109fe85482169f3ce7182131763c05395074d127c8ee8603ff0713\
             ae9f99215a344fb2dfea4c30e34f078d601c103300c077c5945cfdd1a1991001",
        );
        assert_eq!(returned, expected);
    }

    /// Exercises the `generate` path *with* non-empty additional input (the
    /// KAT above uses empty input, leaving the §10.1.1.4 step-2
    /// `w = Hash(0x02‖V‖add)` update untested).  Golden bits reproduced by the
    /// same from-spec Hash_DRBG replica.
    #[test]
    fn hash_drbg_sha256_additional_input_kat() {
        let entropy: Vec<u8> = (0x00u8..0x37).collect();
        let nonce: Vec<u8> = (0x40u8..0x50).collect();
        let a1: Vec<u8> = (0x00u8..0x20).collect();
        let a2: Vec<u8> = (0x20u8..0x40).collect();
        let mut drbg = HashDrbg::from_entropy(&entropy, &nonce, &[]);
        let _ = drbg.generate(KAT_BYTES, &a1);
        let returned = drbg.generate(KAT_BYTES, &a2);
        let expected = hex(
            "7c23e42fbae910f79d028ad1a146c8f2fd20f13b0fe4e4f36a343aec343c1922\
             a0e4b759736a94fa132ef5a5f0c2e0bb48915028d064c87f925462dbd2d84018\
             6666941fc85130bea189d5afdea3be87c0c8800990b99a5966dc3ee4f1dbcac7\
             8733e896af59437d15623c165f64011b1399e0d7c9222977fc2ef9aeacdfa23e",
        );
        assert_eq!(returned, expected);
    }

    /// Streaming [`Rng`] path golden: 4096 `next_u32` words (64 refills of
    /// eight Hashgen blocks, each followed by the §10.1.1.4 update) from the
    /// same instantiation as the KATs above.  Values come from an independent
    /// Python replica of this streaming layout, so the Hashgen counter and
    /// the V update are pinned on the path the battery actually runs.
    #[test]
    fn hash_drbg_streaming_path_golden() {
        let entropy: Vec<u8> = (0x00u8..0x37).collect();
        let nonce: Vec<u8> = (0x40u8..0x50).collect();
        let mut drbg = HashDrbg::from_entropy(&entropy, &nonce, &[]);
        let words: Vec<u32> = (0..4096).map(|_| drbg.next_u32()).collect();
        assert_eq!(
            [words[0], words[63], words[64], words[255]],
            [0x83f2_33a1, 0xf254_631f, 0x1e8e_3355, 0xca0d_ba53]
        );
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        assert_eq!(
            Sha256::digest(&bytes).to_vec(),
            hex("258cfe0eaacdb03eac051981f14a3bec4612925213adc1de394a4ab55129c9aa")
        );
    }
}
