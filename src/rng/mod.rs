//! [`Rng`] trait and all generator implementations used by the test suite.

pub mod aes_ctr;
pub mod bad;
pub mod blum_blum_shub;
pub mod blum_micali;
pub mod c_stdlib;
pub mod chacha20_rng;
pub mod crypto_cprng;
pub mod dual_ec;
pub mod hash_drbg;
pub mod hmac_drbg;
pub mod lcg;
pub mod mt19937;
pub mod os;
pub mod pcg;
pub mod primes;
pub mod sfc;
pub mod spongebob;
pub mod squidward;
pub mod wyrand;
pub mod xorshift;
pub mod xoshiro;

pub use aes_ctr::AesCtr;
pub use bad::{ConstantRng, CounterRng};
pub use blum_blum_shub::BlumBlumShub;
pub use blum_micali::BlumMicali;
pub use c_stdlib::{
    BsdRandCompat, BsdRandom, CRand, LinuxLibcRandom, Rand48, SystemVRand,
    WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd,
};
pub use chacha20_rng::ChaCha20Rng;
pub use crypto_cprng::CryptoCtrDrbg;
pub use dual_ec::DualEcDrbg;
pub use hash_drbg::HashDrbg;
pub use hmac_drbg::HmacDrbg;
pub use lcg::{Lcg32, LcgVariant};
pub use mt19937::Mt19937;
pub use os::OsRng;
pub use pcg::{Pcg32, Pcg64};
pub use sfc::{Jsf64, Sfc64};
pub use spongebob::SpongeBob;
pub use squidward::Squidward;
pub use wyrand::WyRand;
pub use xorshift::{Xorshift32, Xorshift64};
pub use xoshiro::{Xoroshiro128StarStar, Xoshiro256StarStar};

// ── Rng trait ─────────────────────────────────────────────────────────────────

/// Minimal interface required by every test.
///
/// All tests consume bits or 32-bit words; the trait methods below are the
/// only ones needed.  Blanket impls fill in the derived methods.
pub trait Rng {
    /// Return the next 32-bit pseudo-random word.
    fn next_u32(&mut self) -> u32;

    /// Return the next 64-bit pseudo-random word (default: two `next_u32` calls).
    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | (self.next_u32() as u64)
    }

    /// Uniform float in \[0, 1) built from 32 bits.
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
