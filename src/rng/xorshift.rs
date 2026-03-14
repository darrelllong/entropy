//! Marsaglia's Xorshift generators.
//!
//! # Author
//! George Marsaglia, "Xorshift RNGs", *Journal of Statistical Software* 8(14),
//! 2003.  <https://doi.org/10.18637/jss.v008.i14>

use super::Rng;

/// 32-bit Xorshift (Marsaglia, 2003, listing 1).
///
/// Passes most NIST tests but has known weaknesses in linear-complexity and
/// some spectral measures — a good "medium-quality" comparison target.
///
/// # Author
/// George Marsaglia, "Xorshift RNGs", *Journal of Statistical Software* 8(14), 2003.
#[derive(Debug, Clone)]
pub struct Xorshift32 {
    state: u32,
}

impl Xorshift32 {
    /// Seed must be non-zero.
    pub fn new(seed: u32) -> Self {
        assert!(seed != 0, "Xorshift32 seed must be non-zero");
        Self { state: seed }
    }
}

impl Rng for Xorshift32 {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }
}

/// 64-bit Xorshift (Marsaglia, 2003, listing 2).
///
/// Better statistical quality than the 32-bit variant.
///
/// # Author
/// George Marsaglia, "Xorshift RNGs", *Journal of Statistical Software* 8(14), 2003.
#[derive(Debug, Clone)]
pub struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    pub fn new(seed: u64) -> Self {
        assert!(seed != 0, "Xorshift64 seed must be non-zero");
        Self { state: seed }
    }
}

impl Rng for Xorshift64 {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x >> 32) as u32
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}
