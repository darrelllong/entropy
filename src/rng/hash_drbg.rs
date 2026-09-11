//! Hash_DRBG — NIST SP 800-90A Rev. 1 §10.1.1, instantiated with SHA-256.
//!
//! A deterministic random bit generator whose security rests on the
//! one-wayness of SHA-256 (modelled as a random oracle).  Unlike HMAC_DRBG,
//! no keying material is used; the state is a single 440-bit value V (the
//! NIST-specified `seedlen` for SHA-256) and a constant C derived from V.
//!
//! Hashgen produces output by hashing V, V + 1, V + 2, … (mod 2^seedlen),
//! delivering 32 bytes per SHA-256 call.  After each generate
//! request V and the reseed counter are updated per §10.1.1.4; C changes
//! only at instantiation and reseeding.
//!
//! Every mod-2^seedlen addition (the Hashgen counter, `V + w`, and
//! `V + H + C + reseed_counter`) goes through rump's `BigUint`, reached as
//! `cryptography::vt::BigUint`; V and C are kept as big-endian byte strings
//! only because SHA-256 consumes bytes.
//!
//! The streaming [`Rng`] path batches `GENERATE_BLOCKS` blocks per refill;
//! for a CAVP-conformant single `Generate(N bits)` (exactly ⌈N/32⌉ blocks then
//! one update) use [`HashDrbg::generate`], the path the known-answer test uses.
//!
//! For uniform-width access (all `next_u32` or all `next_u64`) all 256 bits
//! per block are used; mixing widths at a refill boundary silently discards
//! up to 7 trailing bytes before refilling.
//!
//! # Reseed interval
//! SP 800-90A Rev. 1 §10.1 Table 2 sets the maximum `reseed_interval` to 2⁴⁸
//! requests.  `reseed_counter` starts at 1, is incremented once per generate
//! call (in `finalise_generate`), and is checked at the top of
//! [`HashDrbg::generate`] and of each `refill`.  §10.1.1.4 step 1 asks for a
//! reseed only when `reseed_counter > reseed_interval`, so the 2⁴⁸-th request
//! is served; no reseed is implemented, so the next request panics.  The test
//! battery never approaches this bound.
//!
//! # Backtracking and prediction resistance
//! SP 800-90A §8.8 designs every DRBG mechanism for backtracking resistance,
//! and this implementation keeps it: after each generate request V becomes
//! `V + Hash(0x03 ‖ V) + C + reseed_counter`, an update the standard relies
//! on SHA-256 to make one-way.  A memory compromise therefore reveals V and
//! C, and with them all *future* output, but earlier output only as far as
//! the bytes of the current refill still held in the 256-byte output
//! buffer.  There is **no prediction resistance**: nothing reseeds, so a
//! compromised state predicts every later output.  Correct for a test
//! harness; do not copy into applications that need prediction resistance.
//!
//! # Seedlen rationale (SP 800-90A Table 2)
//! For SHA-256 (outlen=256 bits, security_strength=256 bits):
//!   seedlen = 440 bits = 55 bytes.
//!
//! # References
//! NIST SP 800-90A Rev. 1, "Recommendation for Random Number Generation
//! Using Deterministic Random Bit Generators", §10.1.1, 2015.
//! [pubs/NIST-SP-800-90Ar1.pdf]
//!
//! # Author
//! NIST (specification); Darrell Long (Rust implementation).

use cryptography::{vt::BigUint, Sha256};

use super::{OsRng, Rng};

const SEEDLEN: usize = 55; // 440 bits — Table 2, SHA-256 row
const SEEDLEN_BITS: usize = SEEDLEN * 8;
const OUTLEN: usize = 32; // SHA-256 output = 256 bits = 32 bytes

/// Maximum `reseed_interval` (SP 800-90A Rev. 1 §10.1 Table 2).
const RESEED_INTERVAL: u64 = 1 << 48;

// Number of Hashgen blocks per generate call.  SP 800-90A §10.1.1.4 says
// all blocks are produced from a local data variable before the §10.1.1.4
// update is applied; GENERATE_BLOCKS controls the batch size.
const GENERATE_BLOCKS: usize = 8;
const GENERATE_SIZE: usize = OUTLEN * GENERATE_BLOCKS; // 256 bytes

/// Hash_DRBG instantiated with SHA-256 per NIST SP 800-90A §10.1.1.
pub struct HashDrbg {
    v: [u8; SEEDLEN],
    c: [u8; SEEDLEN],
    reseed_counter: u64,
    buf: [u8; GENERATE_SIZE], // buffered Hashgen output (one full generate call)
    offset: usize,
}

impl HashDrbg {
    /// Instantiate from OS entropy (entropy_input=55 B, nonce=16 B).
    #[must_use]
    pub fn from_os_rng() -> Self {
        let mut os = OsRng::new();
        let mut seed = [0u8; SEEDLEN + 16];
        {
            // 71 bytes = 17 whole u32 words + 3 remainder bytes; fill the
            // remainder from one more next_u32 so every seed byte is entropy.
            let mut chunks = seed.chunks_exact_mut(4);
            for chunk in &mut chunks {
                chunk.copy_from_slice(&os.next_u32().to_le_bytes());
            }
            let rem = chunks.into_remainder();
            let n = rem.len();
            rem.copy_from_slice(&os.next_u32().to_le_bytes()[..n]);
        }
        // V = Hash_df(entropy_input || nonce, 440 bits)
        let v = hash_df(&seed);
        // C = Hash_df(0x00 || V, 440 bits)
        let c = {
            let mut input = [0u8; 1 + SEEDLEN];
            input[0] = 0x00;
            input[1..].copy_from_slice(&v);
            hash_df(&input)
        };
        Self {
            v,
            c,
            reseed_counter: 1,
            buf: [0u8; GENERATE_SIZE],
            offset: GENERATE_SIZE, // force refill on first use
        }
    }

    /// Instantiate deterministically from explicit entropy input, nonce, and
    /// personalization string (SP 800-90A §10.1.1.2, seed_material =
    /// entropy_input ‖ nonce ‖ personalization_string).
    ///
    /// Exposed for known-answer testing and reproducible discrete use; the
    /// battery uses [`from_os_rng`](Self::from_os_rng).
    #[must_use]
    pub fn from_entropy(entropy_input: &[u8], nonce: &[u8], personalization: &[u8]) -> Self {
        let mut seed =
            Vec::with_capacity(entropy_input.len() + nonce.len() + personalization.len());
        seed.extend_from_slice(entropy_input);
        seed.extend_from_slice(nonce);
        seed.extend_from_slice(personalization);
        let v = hash_df(&seed);
        let c = {
            let mut input = [0u8; 1 + SEEDLEN];
            input[0] = 0x00;
            input[1..].copy_from_slice(&v);
            hash_df(&input)
        };
        Self {
            v,
            c,
            reseed_counter: 1,
            buf: [0u8; GENERATE_SIZE],
            offset: GENERATE_SIZE,
        }
    }

    /// SP 800-90A §10.1.1.4 Generate: produce exactly `nbytes` as a single
    /// discrete Generate call — Hashgen emits ⌈nbytes/32⌉ blocks, then V is
    /// updated once.  This is the CAVP-conformant path.
    ///
    /// It deliberately differs from the streaming [`Rng`] path (which batches
    /// `GENERATE_BLOCKS` blocks per refill); see the module docs.
    pub fn generate(&mut self, nbytes: usize, additional_input: &[u8]) -> Vec<u8> {
        self.check_reseed_interval();
        if !additional_input.is_empty() {
            // §10.1.1.4 step 2: w = Hash(0x02 ‖ V ‖ additional_input);
            //                   V = (V + w) mod 2^seedlen.
            let mut input = Vec::with_capacity(1 + SEEDLEN + additional_input.len());
            input.push(0x02);
            input.extend_from_slice(&self.v);
            input.extend_from_slice(additional_input);
            let w = Sha256::digest(&input);
            add_mod_seedlen(&mut self.v, &[&w[..]]);
        }
        // Hashgen(nbytes) with a local counter starting at V.
        let mut out = Vec::with_capacity(nbytes);
        hashgen(&self.v, nbytes.div_ceil(OUTLEN), |_, block| {
            out.extend_from_slice(block);
        });
        out.truncate(nbytes);
        self.finalise_generate();
        out
    }

    /// Produce GENERATE_BLOCKS Hashgen blocks (§10.1.1.4) into buf, then
    /// update V once per §10.1.1.4.  The local data counter is snapshotted
    /// from V and incremented only within this call, not stored in the struct.
    fn refill(&mut self) {
        self.check_reseed_interval();
        let buf = &mut self.buf;
        hashgen(&self.v, GENERATE_BLOCKS, |i, block| {
            buf[i * OUTLEN..(i + 1) * OUTLEN].copy_from_slice(block);
        });
        self.offset = 0;
        self.finalise_generate();
    }

    /// SP 800-90A Rev. 1 §10.1.1.4 step 1: "If reseed_counter >
    /// reseed_interval, then return an indication that a reseed is
    /// required."  No reseed is implemented, so the indication is a panic.
    fn check_reseed_interval(&self) {
        assert!(
            self.reseed_counter <= RESEED_INTERVAL,
            "Hash_DRBG: reseed_counter exceeds reseed_interval (2⁴⁸); \
             SP 800-90A Rev. 1 §10.1.1.4 step 1 requires a reseed"
        );
    }

    /// §10.1.1.4: update V after a generate call.
    fn finalise_generate(&mut self) {
        // H = Hash(0x03 || V)
        let h = {
            let mut input = [0u8; 1 + SEEDLEN];
            input[0] = 0x03;
            input[1..].copy_from_slice(&self.v);
            Sha256::digest(&input)
        };

        // V = (V + H + C + reseed_counter) mod 2^seedlen
        add_mod_seedlen(
            &mut self.v,
            &[&h[..], &self.c[..], &self.reseed_counter.to_be_bytes()[..]],
        );
        self.reseed_counter = self.reseed_counter.wrapping_add(1);
    }

    fn take_bytes<const N: usize>(&mut self) -> [u8; N] {
        const { assert!(N <= OUTLEN, "chunk larger than SHA-256 output") }
        if self.offset + N > GENERATE_SIZE {
            self.refill();
        }
        let out = self.buf[self.offset..self.offset + N].try_into().unwrap();
        self.offset += N;
        out
    }
}

// ── Helper: Hash_df (SP 800-90A §10.3.1) ─────────────────────────────────────

/// Derive exactly SEEDLEN (55) bytes from `input` using SHA-256.
///
/// The general Hash_df takes a requested bit count; this DRBG only ever asks
/// for seedlen = 440 bits, so the length is fixed by the return type.
fn hash_df(input: &[u8]) -> [u8; SEEDLEN] {
    let bits = (SEEDLEN * 8) as u32;
    let num_blocks = SEEDLEN.div_ceil(OUTLEN);
    let mut temp = [0u8; OUTLEN * 2]; // enough for 2 SHA-256 blocks (covers 55 B)
    for i in 0..num_blocks {
        let counter = (i + 1) as u8;
        // Hash(counter || bits_as_4_be_bytes || input)
        let mut msg = Vec::with_capacity(5 + input.len());
        msg.push(counter);
        msg.extend_from_slice(&bits.to_be_bytes());
        msg.extend_from_slice(input);
        let block = Sha256::digest(&msg);
        let start = i * OUTLEN;
        let end = start + OUTLEN;
        temp[start..end].copy_from_slice(&block);
    }
    let mut out = [0u8; SEEDLEN];
    out.copy_from_slice(&temp[..SEEDLEN]);
    out
}

// ── Arithmetic mod 2^seedlen, via rump ───────────────────────────────────────
//
// SP 800-90A reads V, C, the Hashgen counter and every addend as unsigned
// big-endian integers.  The additions are rump `BigUint` arithmetic and the
// reduction is `low_bits(seedlen)`; nothing here carries by hand.

/// Write `value mod 2^seedlen` into `dst` as exactly `SEEDLEN` big-endian
/// bytes, then wipe the padded encoding.  rump's `to_be_bytes_padded` first
/// builds an unpadded copy and frees it unwiped; the reduced `BigUint` is
/// scrubbed on drop by rump's `wipe` feature, which cryptography-rs enables.
fn store_mod_seedlen(dst: &mut [u8; SEEDLEN], value: &BigUint) {
    let mut bytes = value.low_bits(SEEDLEN_BITS).to_be_bytes_padded(SEEDLEN);
    dst.copy_from_slice(&bytes);
    cryptography::zeroize_slice(&mut bytes);
}

/// `acc ← (acc + Σ addends) mod 2^seedlen`, each addend a big-endian
/// unsigned integer (so shorter addends are implicitly right-aligned).
fn add_mod_seedlen(acc: &mut [u8; SEEDLEN], addends: &[&[u8]]) {
    let mut sum = BigUint::from_be_bytes(acc);
    for addend in addends {
        sum += &BigUint::from_be_bytes(addend);
    }
    store_mod_seedlen(acc, &sum);
}

/// Hashgen (SP 800-90A §10.1.1.4): hash `data = V`, `V + 1`, …
/// (mod 2^seedlen), handing each 32-byte block and its index to `sink`.
fn hashgen(v: &[u8; SEEDLEN], blocks: usize, mut sink: impl FnMut(usize, &[u8])) {
    let one = BigUint::one();
    let mut data = BigUint::from_be_bytes(v);
    for i in 0..blocks {
        let mut bytes = data.to_be_bytes_padded(SEEDLEN);
        sink(i, &Sha256::digest(&bytes)[..]);
        cryptography::zeroize_slice(&mut bytes);
        data += &one;
        // `data` stays below 2^seedlen, so the sum reaches 2^seedlen only
        // when it wraps, and 2^seedlen ≡ 0.
        if data.bits() > SEEDLEN_BITS {
            data = BigUint::zero();
        }
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
}

impl Drop for HashDrbg {
    /// Wipe V, C, and the buffered Hashgen output on drop.
    fn drop(&mut self) {
        cryptography::zeroize_slice(&mut self.v);
        cryptography::zeroize_slice(&mut self.c);
        cryptography::zeroize_slice(&mut self.buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn hash_df_length() {
        // Hash_df output must be exactly SEEDLEN bytes.
        let out = hash_df(b"test input");
        assert_eq!(out.len(), SEEDLEN);
    }

    /// (2^440 − 1) + 1 ≡ 0 (mod 2^seedlen).
    #[test]
    fn add_mod_seedlen_wraps_at_2_pow_440() {
        let mut v = [0xffu8; SEEDLEN];
        add_mod_seedlen(&mut v, &[&[1]]);
        assert_eq!(v, [0u8; SEEDLEN]);
    }

    /// Carries cross rump's 64-bit limb boundaries, and seedlen's top limb
    /// holds only 56 of its 440 bits, so a carry out of bit 439 must vanish
    /// rather than survive as bit 440.
    #[test]
    fn add_mod_seedlen_carries_across_limbs_and_truncates_top() {
        let mut v = [0u8; SEEDLEN];
        v[SEEDLEN - 8..].fill(0xff); // 2^64 − 1
        add_mod_seedlen(&mut v, &[&[1]]);
        let mut want = [0u8; SEEDLEN];
        want[SEEDLEN - 9] = 1; // 2^64
        assert_eq!(v, want);

        // (2^440 − 1) + (2^440 − 1) ≡ 2^440 − 2.
        let mut v = [0xffu8; SEEDLEN];
        add_mod_seedlen(&mut v, &[&[0xffu8; SEEDLEN]]);
        let mut want = [0xffu8; SEEDLEN];
        want[SEEDLEN - 1] = 0xfe;
        assert_eq!(v, want);
    }

    /// The §10.1.1.4 update adds three addends of different widths (32-byte
    /// H, 55-byte C, 8-byte counter) in one call; each is right-aligned.
    #[test]
    fn add_mod_seedlen_right_aligns_mixed_width_addends() {
        let mut v = [0u8; SEEDLEN];
        let h = [0x01u8; OUTLEN];
        let c = [0x02u8; SEEDLEN];
        let counter = 3u64.to_be_bytes();
        add_mod_seedlen(&mut v, &[&h, &c, &counter]);
        let mut want = [0x02u8; SEEDLEN];
        want[SEEDLEN - OUTLEN..].fill(0x03);
        want[SEEDLEN - 1] = 0x06;
        assert_eq!(v, want);
    }

    /// Hashgen's counter wraps mod 2^seedlen: starting from 2^440 − 1, the
    /// second block hashes the all-zero string.
    #[test]
    fn hashgen_counter_wraps() {
        let mut blocks = Vec::new();
        hashgen(&[0xffu8; SEEDLEN], 2, |i, b| blocks.push((i, b.to_vec())));
        assert_eq!(blocks[0], (0, Sha256::digest(&[0xffu8; SEEDLEN]).to_vec()));
        assert_eq!(blocks[1], (1, Sha256::digest(&[0u8; SEEDLEN]).to_vec()));
    }

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
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
        let _ = drbg.generate(128, &[]); // first Generate — discarded
        let returned = drbg.generate(128, &[]);
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
        let _ = drbg.generate(128, &a1);
        let returned = drbg.generate(128, &a2);
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

    /// §10.1.1.4 step 1 refuses a request only when `reseed_counter >
    /// reseed_interval`.  With the counter set to 2⁴⁸, one more request is
    /// served on each path (a discrete Generate, and a streaming refill) and
    /// the request after it panics.
    #[test]
    fn reseed_counter_boundary_is_strictly_greater() {
        let entropy: Vec<u8> = (0x00u8..0x37).collect();
        let nonce: Vec<u8> = (0x40u8..0x50).collect();

        let mut discrete = HashDrbg::from_entropy(&entropy, &nonce, &[]);
        discrete.reseed_counter = RESEED_INTERVAL;
        assert_eq!(discrete.generate(OUTLEN, &[]).len(), OUTLEN);
        assert_eq!(discrete.reseed_counter, RESEED_INTERVAL + 1);
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            discrete.generate(OUTLEN, &[])
        }));
        assert!(
            refused.is_err(),
            "a request past reseed_interval must panic"
        );

        let mut stream = HashDrbg::from_entropy(&entropy, &nonce, &[]);
        stream.reseed_counter = RESEED_INTERVAL;
        for _ in 0..GENERATE_SIZE / 4 {
            let _ = stream.next_u32(); // one refill, made at counter 2⁴⁸
        }
        assert_eq!(stream.reseed_counter, RESEED_INTERVAL + 1);
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| stream.next_u32()));
        assert!(refused.is_err(), "a refill past reseed_interval must panic");
    }
}
