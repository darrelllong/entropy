//! HMAC_DRBG — NIST SP 800-90A Rev. 1 §10.1.2, instantiated with HMAC-SHA-256.
//!
//! A deterministic random bit generator whose security rests on the
//! pseudorandomness of HMAC-SHA-256.  The state is a key K and a value V
//! (each 32 bytes); every `generate` call advances V via `V = HMAC(K, V)` and
//! then re-keys.
//!
//! This implementation:
//! - Seeds K and V from 384 bits of OS entropy (entropy_input=32 B,
//!   nonce=16 B) with an empty personalization string.
//! - Generates output in 32-byte blocks (one HMAC invocation per block).
//! - The streaming [`Rng`] path uses no additional input and no explicit
//!   reseed (suitable for the test battery); the discrete
//!   [`generate`](HmacDrbg::generate) method implements the standard
//!   `HMAC_DRBG_Update` and `Generate` procedures *with* optional additional
//!   input.
//!
//! The streaming [`Rng`] path re-keys after every 32-byte block, so a long
//! `next_u32` stream is a *sequence* of one-block Generate calls, not a single
//! multi-block Generate.  For a CAVP-conformant single `Generate(N bits)` (no
//! intermediate re-keying, one Update at the end) use
//! [`HmacDrbg::generate`]; that is the path the known-answer test exercises.
//!
//! # Reseed interval
//! SP 800-90A Rev. 1 §10.1 Table 2 sets the maximum `reseed_interval` to 2⁴⁸
//! requests.  `reseed_counter` starts at 1 and counts generate calls, each
//! streaming refill being one.  §10.1.2.5 step 1 asks for a reseed only when
//! `reseed_counter > reseed_interval`, so the 2⁴⁸-th request is served; no
//! reseed is implemented, so the next request panics.  In practice the test
//! battery never approaches 2⁴⁸ calls.
//!
//! # Backtracking and prediction resistance
//! SP 800-90A §8.8 designs every DRBG mechanism for backtracking resistance,
//! and this implementation keeps it: `HMAC_DRBG_Update` runs after every
//! 32-byte block on the streaming path and after each [`generate`] request,
//! and the standard relies on HMAC-SHA-256 to make that update one-way.  A
//! memory compromise therefore reveals K and V, and with them all *future*
//! output, but earlier output only as far as the current block still held
//! in the 32-byte output buffer.  There is **no prediction resistance**:
//! nothing reseeds, so a compromised state predicts every later output.
//! Correct for a test harness; do not copy into applications that need
//! prediction resistance.
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

use cryptography::{Hmac, Sha256};

use super::{ByteBuffered, OsRng, Rng};

const OUT: usize = 32; // HMAC-SHA-256 output length (bytes)

/// Maximum `reseed_interval` (SP 800-90A Rev. 1 §10.1 Table 2).
const RESEED_INTERVAL: u64 = 1 << 48;

/// HMAC_DRBG instantiated with HMAC-SHA-256 per NIST SP 800-90A §10.1.2.
pub struct HmacDrbg {
    k: [u8; OUT],
    v: [u8; OUT],
    buf: [u8; OUT],
    offset: usize,
    /// Generate-call counter, starting at 1; a request that finds it above
    /// `RESEED_INTERVAL` panics (SP 800-90A Rev. 1 §10.1.2.5 step 1).
    reseed_counter: u64,
}

impl HmacDrbg {
    /// Instantiate from OS entropy (entropy_input=32 B, nonce=16 B).
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        let mut seed = [0u8; 48]; // 32-byte entropy_input + 16-byte nonce
        for chunk in seed.chunks_exact_mut(4) {
            chunk.copy_from_slice(&os.next_u32().to_le_bytes());
        }
        // Initial K=0x00…, V=0x01…, then Update(seed_material).
        let mut drbg = Self {
            k: [0x00u8; OUT],
            v: [0x01u8; OUT],
            buf: [0u8; OUT],
            offset: OUT, // force refill on first use
            reseed_counter: 1,
        };
        drbg_update(&mut drbg.k, &mut drbg.v, Some(&seed));
        drbg
    }

    /// Instantiate deterministically from explicit entropy input, nonce, and
    /// personalization string (SP 800-90A §10.1.2.3, seed_material =
    /// entropy_input ‖ nonce ‖ personalization_string).
    ///
    /// Exposed for known-answer testing and reproducible discrete use; the
    /// battery uses [`from_os_rng`](Self::from_os_rng).
    #[must_use]
    pub fn from_entropy(entropy_input: &[u8], nonce: &[u8], personalization: &[u8]) -> Self {
        let mut drbg = Self {
            k: [0x00u8; OUT],
            v: [0x01u8; OUT],
            buf: [0u8; OUT],
            offset: OUT,
            reseed_counter: 1,
        };
        let mut seed =
            Vec::with_capacity(entropy_input.len() + nonce.len() + personalization.len());
        seed.extend_from_slice(entropy_input);
        seed.extend_from_slice(nonce);
        seed.extend_from_slice(personalization);
        drbg_update(&mut drbg.k, &mut drbg.v, Some(&seed));
        drbg
    }

    /// SP 800-90A §10.1.2.5 Generate: produce `nbytes` as a single discrete
    /// Generate call — output blocks are produced WITHOUT intermediate
    /// re-keying, then one `Update` runs at the end.  This is the
    /// CAVP-conformant path.
    ///
    /// It deliberately differs from the streaming [`Rng`] path (which re-keys
    /// after every 32-byte block for incremental forward secrecy, so a long
    /// `next_u32` stream is a sequence of one-block Generates, not one big
    /// Generate — see the module docs).
    pub fn generate(&mut self, nbytes: usize, additional_input: &[u8]) -> Vec<u8> {
        self.check_reseed_interval();
        let add = (!additional_input.is_empty()).then_some(additional_input);
        if let Some(a) = add {
            drbg_update(&mut self.k, &mut self.v, Some(a));
        }
        let mut out = Vec::with_capacity(nbytes);
        while out.len() < nbytes {
            let mac = hmac_sha256(&self.k, &self.v);
            self.v.copy_from_slice(&mac);
            out.extend_from_slice(&self.v);
        }
        out.truncate(nbytes);
        drbg_update(&mut self.k, &mut self.v, add);
        self.reseed_counter += 1;
        out
    }

    /// SP 800-90A Rev. 1 §10.1.2.5 step 1: "If reseed_counter >
    /// reseed_interval, then return an indication that a reseed is
    /// required."  No reseed is implemented, so the indication is a panic.
    fn check_reseed_interval(&self) {
        assert!(
            self.reseed_counter <= RESEED_INTERVAL,
            "HMAC_DRBG: reseed_counter exceeds reseed_interval (2⁴⁸); \
             SP 800-90A Rev. 1 §10.1.2.5 step 1 requires a reseed"
        );
    }
}

impl ByteBuffered<OUT> for HmacDrbg {
    fn buffer(&self) -> &[u8; OUT] {
        &self.buf
    }

    fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }

    /// One streaming Generate: advance V, buffer it, then re-key per
    /// §10.1.2.5.
    fn refill(&mut self) {
        self.check_reseed_interval();
        let mac = hmac_sha256(&self.k, &self.v);
        self.v.copy_from_slice(&mac);
        self.buf = self.v;
        drbg_update(&mut self.k, &mut self.v, None);
        self.reseed_counter += 1;
    }
}

// ── SP 800-90A §10.1.2.2 HMAC_DRBG_Update ─────────────────────────────────

/// Stack scratch for the common short messages: `V ‖ sep ‖ provided_data`
/// with up to 48 bytes of seed material (the `from_os_rng` instantiation)
/// fits here, so `drbg_update` itself adds no heap allocation on the
/// streaming path (`refill`, which passes no `provided_data`).  The HMAC
/// computations still allocate inside cryptography-rs.
const STACK_SCRATCH: usize = 2 * OUT + 1 + 16;

/// SP 800-90A §10.1 Table 2 allows personalization strings and additional
/// input up to 2³⁵ bits, so longer messages spill to a heap buffer sized to
/// the input.  Whichever buffer is used is wiped before it goes out of
/// scope, matching the hygiene applied to `K` and `V` in [`Drop`].
fn drbg_update(k: &mut [u8; OUT], v: &mut [u8; OUT], provided_data: Option<&[u8]>) {
    let pd = provided_data.unwrap_or(&[]);
    let len = OUT + 1 + pd.len();
    let mut stack = [0u8; STACK_SCRATCH];
    let mut heap = Vec::new();
    let msg: &mut [u8] = if len <= STACK_SCRATCH {
        &mut stack[..len]
    } else {
        heap.resize(len, 0);
        heap.as_mut_slice()
    };

    // K = HMAC(K, V || 0x00 [|| provided_data])
    msg[..OUT].copy_from_slice(v);
    msg[OUT] = 0x00;
    msg[OUT + 1..].copy_from_slice(pd);
    let mac = hmac_sha256(k, msg);
    k.copy_from_slice(&mac);

    // V = HMAC(K, V)
    let mac = hmac_sha256(k, v);
    v.copy_from_slice(&mac);

    if provided_data.is_some() {
        // K = HMAC(K, V || 0x01 || provided_data)
        msg[..OUT].copy_from_slice(v);
        msg[OUT] = 0x01;
        // pd slice and length unchanged — reuse msg[OUT+1..] already written
        let mac = hmac_sha256(k, msg);
        k.copy_from_slice(&mac);

        // V = HMAC(K, V)
        let mac = hmac_sha256(k, v);
        v.copy_from_slice(&mac);
    }
    cryptography::zeroize_slice(msg);
}

#[inline]
fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; OUT] {
    let mac = Hmac::<Sha256>::compute(key, data);
    mac.try_into()
        .expect("HMAC-SHA-256 output is always 32 bytes")
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
}

impl Drop for HmacDrbg {
    /// Wipe the key, working value, and buffered output on drop, matching the
    /// hygiene `AesCtr`/`BlockCtrRng` apply to their secret material.
    fn drop(&mut self) {
        cryptography::zeroize_slice(&mut self.k);
        cryptography::zeroize_slice(&mut self.v);
        cryptography::zeroize_slice(&mut self.buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::hex;

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

    /// NIST DRBGVS HMAC_DRBG SHA-256 known-answer test: PredictionResistance =
    /// False, no reseed, empty personalization and additional input,
    /// ReturnedBitsLen = 1024 (two Generate calls, the second returned).  The
    /// expected bits match the published vector (prefix `e528e9ab…`) and were
    /// independently reproduced by a from-spec SP 800-90A replica.  This pins
    /// the HMAC_DRBG_Update / Generate math, not just "output advances".
    #[test]
    fn hmac_drbg_sha256_nist_drbgvs_kat() {
        let entropy = hex("ca851911349384bffe89de1cbdc46e6831e44d34a4fb935ee285dd14b71a7488");
        let nonce = hex("659ba96c601dc69fc902940805ec0ca8");
        let mut drbg = HmacDrbg::from_entropy(&entropy, &nonce, &[]);
        let _ = drbg.generate(128, &[]); // first Generate — discarded per DRBGVS
        let returned = drbg.generate(128, &[]);
        let expected = hex(
            "e528e9abf2dece54d47c7e75e5fe302149f817ea9fb4bee6f4199697d04d5b89\
             d54fbb978a15b5c443c9ec21036d2460b6f73ebad0dc2aba6e624abf07745bc1\
             07694bb7547bb0995f70de25d6b29e2d3011bb19d27676c07162c8b5ccde0668\
             961df86803482cb37ed6d5c0bb8d50cf1f50d476aa0458bdaba806f48be9dcb8",
        );
        assert_eq!(returned, expected);
    }

    /// Regression: `drbg_update` used a fixed 128-byte stack scratch, so any
    /// personalization string longer than 47 bytes or additional input longer
    /// than 95 bytes panicked in release builds.  SP 800-90A permits up to
    /// 2³⁵ bits of each.  The golden outputs come from an independent
    /// from-spec HMAC_DRBG replica (Python `hmac`/`hashlib`), so a fix that
    /// silently truncated the input to the old budget would fail here.
    #[test]
    fn hmac_drbg_long_personalization_and_additional_input_kat() {
        let long_pers = [0xa5u8; 200];
        let mut a = HmacDrbg::from_entropy(&[1u8; 32], &[2u8; 16], &long_pers);
        assert_eq!(
            a.generate(32, &[]),
            hex("b075870331a47cbb0bb09b6bc44181ad8dad91363ba0cb309e7aafc62a96f1aa")
        );

        let long_add = [0x5au8; 1024];
        let mut b = HmacDrbg::from_entropy(&[1u8; 32], &[2u8; 16], &[]);
        assert_eq!(
            b.generate(32, &long_add),
            hex("3d00f0409313ca86990ac50c6376cb3a35589c4eb7c0a209bed5cd8ebc819391")
        );
    }

    /// Exercises the `generate` path *with* non-empty additional input (the
    /// DRBGVS KAT above uses empty input, leaving the two extra Update rounds
    /// untested).  Additional inputs 0x00..0x1f then 0x20..0x3f; golden bits
    /// reproduced by the same from-spec HMAC_DRBG replica.
    #[test]
    fn hmac_drbg_sha256_additional_input_kat() {
        let entropy = hex("ca851911349384bffe89de1cbdc46e6831e44d34a4fb935ee285dd14b71a7488");
        let nonce = hex("659ba96c601dc69fc902940805ec0ca8");
        let a1: Vec<u8> = (0x00u8..0x20).collect();
        let a2: Vec<u8> = (0x20u8..0x40).collect();
        let mut drbg = HmacDrbg::from_entropy(&entropy, &nonce, &[]);
        let _ = drbg.generate(128, &a1);
        let returned = drbg.generate(128, &a2);
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

    /// §10.1.2.5 step 1 refuses a request only when `reseed_counter >
    /// reseed_interval`.  With the counter set to 2⁴⁸, one more request is
    /// served on each path (a discrete Generate, and a streaming refill) and
    /// the request after it panics.
    #[test]
    fn reseed_counter_boundary_is_strictly_greater() {
        let entropy: Vec<u8> = (0x00u8..0x20).collect();
        let nonce: Vec<u8> = (0x20u8..0x30).collect();

        let mut discrete = HmacDrbg::from_entropy(&entropy, &nonce, &[]);
        discrete.reseed_counter = RESEED_INTERVAL;
        assert_eq!(discrete.generate(OUT, &[]).len(), OUT);
        assert_eq!(discrete.reseed_counter, RESEED_INTERVAL + 1);
        let refused =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| discrete.generate(OUT, &[])));
        assert!(
            refused.is_err(),
            "a request past reseed_interval must panic"
        );

        let mut stream = HmacDrbg::from_entropy(&entropy, &nonce, &[]);
        stream.reseed_counter = RESEED_INTERVAL;
        for _ in 0..OUT / 4 {
            let _ = stream.next_u32(); // one refill, made at counter 2⁴⁸
        }
        assert_eq!(stream.reseed_counter, RESEED_INTERVAL + 1);
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| stream.next_u32()));
        assert!(refused.is_err(), "a refill past reseed_interval must panic");
    }
}
