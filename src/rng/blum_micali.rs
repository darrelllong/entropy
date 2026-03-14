//! Blum-Micali pseudorandom bit generator.
//!
//! The discrete-log based one-bit generator of Blum and Micali (1984):
//!
//! ```text
//! xᵢ₊₁ = g^xᵢ mod p
//! bitᵢ  = 1 if xᵢ₊₁ ≤ (p−1)/2, else 0
//! ```
//!
//! Security rests on the hardness of the discrete logarithm problem.
//!
//! For the test battery, 32 generator steps are packed into each 32-bit output
//! word.  The implementation requires p to be a prime < 2^32, g to be an
//! element of large multiplicative order, and x₀ ∈ (0, p).
//!
//! # Reference
//! M. Blum and S. Micali, "How to Generate Cryptographically Strong Sequences
//! of Pseudo-Random Bits", *SIAM Journal on Computing* 13(4), pp. 850–864, 1984.
//!
//! # Author
//! Darrell Long (UC Santa Cruz).

use super::{primes::{is_probable_prime, mod_pow}, Rng};

/// Blum-Micali pseudorandom bit generator.
pub struct BlumMicali {
    p:     u64,   // prime modulus
    g:     u64,   // generator (element of large order in Zp*)
    state: u64,   // current xᵢ
}

impl BlumMicali {
    /// Construct a Blum-Micali generator.
    ///
    /// * `p`    — prime modulus (prime and < 2^32).
    /// * `g`    — generator, 1 < g < p.
    /// * `seed` — initial state x₀, 0 < seed < p.
    ///
    /// # Panics
    /// Panics if the parameters violate the stated preconditions.
    #[must_use]
    pub fn new(p: u64, g: u64, seed: u64) -> Self {
        assert!(p > 2 && p < (1u64 << 32), "p must be in (2, 2³²)");
        assert!(is_probable_prime(p), "p must be prime");
        assert!(g > 1 && g < p, "g must be in (1, p)");
        assert!(seed > 0 && seed < p, "seed must be in (0, p)");
        Self { p, g, state: seed }
    }

    /// Advance one step and return the output bit.
    pub fn next_bit(&mut self) -> u8 {
        self.state = mod_pow(self.g, self.state, self.p);
        u8::from(self.state <= (self.p - 1) / 2)
    }
}

impl Rng for BlumMicali {
    /// Pack 32 generator bits (MSB first) into a u32.
    fn next_u32(&mut self) -> u32 {
        let mut word = 0u32;
        for _ in 0..32 {
            word = (word << 1) | self.next_bit() as u32;
        }
        word
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Small reference: p = 23, g = 5, seed = 3.
    // x₁ = 5³ mod 23 = 125 mod 23 = 10;  10 ≤ 11 → bit = 1
    // x₂ = 5¹⁰ mod 23 = 9765625 mod 23 = 9;  9 ≤ 11 → bit = 1
    // x₃ = 5⁹ mod 23 = 1953125 mod 23 = 1;  1 ≤ 11 → bit = 1
    // (Matches the cryptography crate's own tests.)
    #[test]
    fn small_reference_first_3_bits() {
        let mut bm = BlumMicali::new(23, 5, 3);
        assert_eq!(bm.next_bit(), 1);
        assert_eq!(bm.next_bit(), 1);
        assert_eq!(bm.next_bit(), 1);
    }

    #[test]
    fn larger_prime_runs() {
        // p = 2147483647 (2³¹−1), g = 7, seed = 42
        let mut bm = BlumMicali::new(2_147_483_647, 7, 42);
        let first  = bm.next_u32();
        let second = bm.next_u32();
        assert_ne!(first, second);
    }
}
