//! [`Rng`] trait and all generator implementations used by the test suite.
//!
//! The generators built on the sibling cryptography crate's ciphers,
//! hashes and DRBGs are behind the `cryptography` feature, on by default;
//! without it the statistical generators, [`OsRng`] and the suites remain,
//! and the build links no cryptographic multiprecision.

#[cfg(feature = "cryptography")]
pub mod aes_ctr;
pub mod alternatives;
pub mod bad;
#[cfg(feature = "cryptography")]
pub mod block_ctr;
pub mod c_stdlib;
#[cfg(feature = "cryptography")]
pub mod chacha20_rng;
pub mod corpus;
#[cfg(feature = "cryptography")]
pub mod crypto_cprng;
#[cfg(feature = "cryptography")]
pub mod dual_ec;
#[cfg(feature = "cryptography")]
pub mod hash_drbg;
#[cfg(feature = "cryptography")]
pub mod hmac_drbg;
pub mod lcg;
pub mod mt19937;
pub mod os;
pub mod pcg;
pub mod sfc;
#[cfg(feature = "cryptography")]
pub mod spongebob;
#[cfg(feature = "cryptography")]
pub mod squidward;
#[cfg(feature = "cryptography")]
pub mod stream_rng;
pub mod views;
pub mod xorshift;
pub mod xoshiro;

#[cfg(feature = "cryptography")]
pub use aes_ctr::AesCtr;
pub use bad::{ConstantRng, CounterRng};
#[cfg(feature = "cryptography")]
pub use block_ctr::BlockCtrRng;
pub use c_stdlib::{
    BsdRandCompat, BsdRandom, LinuxLibcRandom, Rand48, SystemVRand, WindowsDotNetRandom,
    WindowsMsvcRand, WindowsVb6Rnd,
};
#[cfg(feature = "cryptography")]
pub use chacha20_rng::ChaCha20Rng;
pub use corpus::Corpus;
#[cfg(feature = "cryptography")]
pub use crypto_cprng::CryptoCtrDrbg;
#[cfg(feature = "cryptography")]
pub use dual_ec::DualEcDrbg;
#[cfg(feature = "cryptography")]
pub use hash_drbg::HashDrbg;
#[cfg(feature = "cryptography")]
pub use hmac_drbg::HmacDrbg;
pub use lcg::{Lcg32, LcgVariant};
pub use mt19937::Mt19937;
pub use os::OsRng;
pub use pcg::{Pcg32, Pcg64};
pub use sfc::{Jsf64, Sfc64};
#[cfg(feature = "cryptography")]
pub use spongebob::SpongeBob;
#[cfg(feature = "cryptography")]
pub use squidward::Squidward;
#[cfg(feature = "cryptography")]
pub use stream_rng::StreamRng;
pub use views::{BitReversed, FullWord, HighHalf, LowHalf};
pub use xorshift::{Xorshift32, Xorshift64};
pub use xoshiro::{Xoroshiro128, Xoshiro256};

// ── Rng trait ─────────────────────────────────────────────────────────────────

/// Minimal interface required by every test.
///
/// All tests consume bits or 32-bit words; the trait methods below are the
/// only ones needed.  Blanket impls fill in the derived methods.
///
/// ## Design note — no CSPRNG marker type
///
/// This trait is intentionally flat: every generator from `ConstantRng` to
/// `ChaCha20Rng` implements the same `Rng`.  This is correct for a test
/// harness whose job is to compare generators uniformly, but it means the
/// **type system provides no barrier** against substituting a weak generator
/// where a strong one is required.  Any function that accepts `impl Rng` will
/// silently compile with `ConstantRng` or `SystemVRand`.
///
/// **Do not copy this design into production code.**  In an application,
/// define a separate `CsprngRng: Rng` marker subtrait (or a newtype) and
/// restrict security-sensitive functions to `impl CsprngRng` so that weak
/// generators are rejected at compile time.
///
/// ## Byte and word ordering contract
///
/// * **64-bit generators** — `next_u32` returns the high half of one 64-bit
///   output and discards the low half, so the batteries test that projection
///   only.  [`views`] provides the low-half, full-word and bit-reversed views.
///
/// * **`next_u64` default** — assembles two `next_u32` calls with the *first*
///   call becoming the **high** 32 bits: `(hi << 32) | lo`.  Generators that
///   override this (e.g. byte-backed DRBG adapters) may emit a different
///   interleaving; those overrides are documented on the concrete type.
///
/// * **`collect_bits`** — extracts bits **LSB-first** from each 32-bit word:
///   bit 0 of word 0 is the first element of the returned slice.
///
/// * **Byte-backed generators** decode words from a byte stream:
///   - `HmacDrbg`, `HashDrbg`, `ChaCha20Rng`, `SpongeBob`, `Squidward` and
///     `StreamRng` read 4 bytes little-endian for `next_u32` and override
///     `next_u64` to read 8 bytes little-endian, independently of the default
///     `next_u64` above.  Mixing `next_u32` and `next_u64` at a buffer refill
///     boundary silently discards up to 7 trailing bytes; see the individual
///     adapter doc comments for the full caveat.
///   - `BlockCtrRng` and `OsRng` read 4 bytes little-endian for `next_u32`
///     and keep the default `next_u64`.
///   - `AesCtr`, `CryptoCtrDrbg` and `DualEcDrbg` decode `next_u32`
///     **big-endian** and keep the default `next_u64`; see the docs on those
///     types.
pub trait Rng {
    /// Return the next 32-bit pseudo-random word.
    fn next_u32(&mut self) -> u32;

    /// Return the next 64-bit pseudo-random word.
    ///
    /// Default: two `next_u32` calls, first call → high 32 bits.
    /// Some byte-backed generators override this to read 8 LE bytes
    /// directly; see the byte-ordering contract above.
    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | (self.next_u32() as u64)
    }

    /// Uniform float in \[0, 1) built from **32 bits** of the generator's output.
    ///
    /// This default always calls `next_u32`, so it delivers at most 2³²
    /// distinct values, and for a 64-bit generator it reads the same high-half
    /// projection as `next_u32`.  The geometric tests (parking lot, minimum
    /// distance, spheres, permutations) draw their coordinates this way.
    fn next_f64(&mut self) -> f64 {
        self.next_u32() as f64 * (1.0 / 4_294_967_296.0)
    }

    /// Collect `n` bits as `Vec<u8>` values 0 or 1, LSB-first from each word.
    fn collect_bits(&mut self, n: usize) -> Vec<u8> {
        let mut bits = Vec::with_capacity(n);
        let mut remaining = n;
        while remaining > 0 {
            let word = self.next_u32();
            let take = remaining.min(32);
            for i in 0..take {
                bits.push(((word >> i) & 1) as u8);
            }
            remaining -= take;
        }
        bits
    }

    /// Collect `n` 32-bit words.
    fn collect_u32s(&mut self, n: usize) -> Vec<u32> {
        (0..n).map(|_| self.next_u32()).collect()
    }

    /// Collect `n` floats in \[0, 1).
    fn collect_f64s(&mut self, n: usize) -> Vec<f64> {
        (0..n).map(|_| self.next_f64()).collect()
    }
}

// ── Byte-buffered generators ─────────────────────────────────────────────────

#[cfg(feature = "cryptography")]
/// Read path shared by the generators that serve words from a buffer of
/// output bytes: `ChaCha20Rng`, `HashDrbg`, `HmacDrbg`, `SpongeBob`,
/// `Squidward` and `StreamRng`.
///
/// An implementor owns a `LEN`-byte buffer and a read offset into it and says
/// how to refill the buffer; [`take_bytes`](Self::take_bytes) does the rest.
/// An offset of `LEN` marks the buffer exhausted, so a constructor that sets
/// it there defers the first refill to the first read.
trait ByteBuffered<const LEN: usize> {
    /// The buffered output bytes.
    fn buffer(&self) -> &[u8; LEN];

    /// The read offset into [`buffer`](Self::buffer).
    fn offset_mut(&mut self) -> &mut usize;

    /// Overwrite the whole buffer with the generator's next `LEN` bytes.
    fn refill(&mut self);

    /// The next `N` bytes.  When fewer than `N` remain, they are discarded
    /// and the buffer is refilled first; that is how mixing `next_u32` and
    /// `next_u64` at a refill boundary drops up to 7 bytes.
    #[inline]
    fn take_bytes<const N: usize>(&mut self) -> [u8; N] {
        const { assert!(N <= LEN, "read wider than the byte buffer") }
        if *self.offset_mut() + N > LEN {
            self.refill();
            *self.offset_mut() = 0;
        }
        let at = *self.offset_mut();
        *self.offset_mut() = at + N;
        let mut out = [0u8; N];
        out.copy_from_slice(&self.buffer()[at..at + N]);
        out
    }
}

/// Decode a hex string, two digits per byte, as the known-answer tests
/// print their vectors.
#[cfg(all(test, feature = "cryptography"))]
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[cfg(test)]
mod tests {
    /// Every `Drop` impl in this module clears state through
    /// `cryptography::zeroize_slice`.  Committed cryptography-rs 0.7
    /// (342989a) makes it an unconditional volatile write; a sibling change
    /// that turned it into a feature-gated no-op would silently disable all
    /// of those wipes, so pin the behaviour here.  Nothing guards the other
    /// half, rump's drop-time limb scrubbing: it cannot be observed from safe
    /// code, and no test inspects the resolved dependency features.
    #[cfg(feature = "cryptography")]
    #[test]
    fn zeroize_slice_clears_memory() {
        let mut buf = [0xa5u8; 64];
        cryptography::zeroize_slice(&mut buf);
        assert_eq!(buf, [0u8; 64]);
    }
}
