//! Marsaglia's Xorshift generators.
//!
//! Both are example procedures from §3 of the paper (p. 4): `xor()`, a 32-bit
//! xorshift with [a, b, c] = [13, 17, 5], and `xor64()`, a 64-bit one with
//! [13, 7, 17].  The paper prints the middle statement of `xor()` as
//! `y=(y>>17)`; the [13, 17, 5] xorshift it describes, and [`Xorshift32`],
//! use `y^=(y>>17)`.  The paper's seeds are 2463534242 and
//! 88172645463325252; the constructors here take any non-zero seed.
//!
//! # Author
//! George Marsaglia, "Xorshift RNGs", *Journal of Statistical Software* 8(14),
//! 2003.  <https://doi.org/10.18637/jss.v008.i14>
//! [pubs/marsaglia-2003-xorshift-rngs.pdf]

use super::{
    jump::{advance_linear, annihilating_polynomial, Poly},
    streams::{Advance, Streams},
    Rng,
};
use std::sync::OnceLock;

/// 32-bit Xorshift (Marsaglia, 2003, §3: `xor()`).
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

/// 64-bit Xorshift (Marsaglia, 2003, §3: `xor64()`).
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
    /// Construct from a 64-bit seed.  Seed must be non-zero.
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

/// Stream segments of xorshift32: 2¹⁶ steps each, half the period's bits.
const SEGMENT_LOG2_32: u32 = 16;

/// Stream segments of xorshift64: 2³² steps each.
const SEGMENT_LOG2_64: u32 = 32;

/// Xorshift32's update on a word whose high half stays zero.
fn advance32(s: &mut [u64; 1]) {
    let mut x = s[0] as u32;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    s[0] = u64::from(x);
}

/// Xorshift64's update.
fn advance64(s: &mut [u64; 1]) {
    let mut x = s[0];
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    s[0] = x;
}

/// The polynomial annihilating xorshift32's update, derived once: the
/// recurrence has degree 32 and the factor x³² covers the zero high half.
fn annihilator32() -> &'static Poly {
    static POLY: OnceLock<Poly> = OnceLock::new();
    POLY.get_or_init(|| annihilating_polynomial(advance32))
}

/// The polynomial annihilating xorshift64's update, derived once.
fn annihilator64() -> &'static Poly {
    static POLY: OnceLock<Poly> = OnceLock::new();
    POLY.get_or_init(|| annihilating_polynomial(advance64))
}

impl Advance for Xorshift32 {
    fn advance(&mut self, steps: u128) {
        let mut state = [u64::from(self.state)];
        advance_linear(&mut state, annihilator32(), steps, 0, advance32);
        self.state = state[0] as u32;
    }
}

impl Streams for Xorshift32 {
    /// Segment `index` of 2¹⁶ steps from this generator's position; the
    /// period is 2³² − 1, so segments repeat after 2¹⁶ of them.
    fn stream(&self, index: u64) -> Self {
        let mut state = [u64::from(self.state)];
        advance_linear(
            &mut state,
            annihilator32(),
            u128::from(index),
            SEGMENT_LOG2_32,
            advance32,
        );
        Self {
            state: state[0] as u32,
        }
    }
}

impl Advance for Xorshift64 {
    fn advance(&mut self, steps: u128) {
        let mut state = [self.state];
        advance_linear(&mut state, annihilator64(), steps, 0, advance64);
        self.state = state[0];
    }
}

impl Streams for Xorshift64 {
    /// Segment `index` of 2³² steps from this generator's position; the
    /// period is 2⁶⁴ − 1, so segments repeat after 2³² of them.
    fn stream(&self, index: u64) -> Self {
        let mut state = [self.state];
        advance_linear(
            &mut state,
            annihilator64(),
            u128::from(index),
            SEGMENT_LOG2_64,
            advance64,
        );
        Self { state: state[0] }
    }
}

#[cfg(test)]
mod tests {
    use super::super::streams::{Advance, Streams};

    /// An advance is the steps it stands for, for both widths, and a stream
    /// is an advance of index times the segment.
    #[test]
    fn advances_match_stepping() {
        for steps in [0u128, 1, 7, 1_000_003] {
            let mut stepped = Xorshift32::new(2_463_534_242);
            for _ in 0..steps {
                let _ = stepped.next_u32();
            }
            let mut jumped = Xorshift32::new(2_463_534_242);
            jumped.advance(steps);
            assert_eq!(jumped.state, stepped.state, "xorshift32, {steps}");

            let mut stepped = Xorshift64::new(88_172_645_463_325_252);
            for _ in 0..steps {
                let _ = stepped.next_u32();
            }
            let mut jumped = Xorshift64::new(88_172_645_463_325_252);
            jumped.advance(steps);
            assert_eq!(jumped.state, stepped.state, "xorshift64, {steps}");
        }
        let base = Xorshift64::new(1);
        let mut walked = Xorshift64::new(1);
        walked.advance(5u128 << SEGMENT_LOG2_64);
        assert_eq!(base.stream(5).state, walked.state);
        let base = Xorshift32::new(1);
        let mut walked = Xorshift32::new(1);
        walked.advance(5u128 << SEGMENT_LOG2_32);
        assert_eq!(base.stream(5).state, walked.state);
    }

    use super::*;

    /// Seed 1 through Marsaglia's 32-bit xorshift with the (13, 17, 5) triple
    /// used here.  Values from an independent replica of the generator; the
    /// first four also match the fixed-seed `dump_rng xorshift32` output.
    #[test]
    fn xorshift32_seed_1_kat() {
        let expected: [u32; 8] = [
            0x0004_2021,
            0x0408_0601,
            0x9dcc_a8c5,
            0x1255_994f,
            0x8ef9_17d1,
            0x2c6f_5bd0,
            0x25b2_331a,
            0x19f9_1cb2,
        ];
        let mut rng = Xorshift32::new(1);
        assert_eq!(expected.map(|_| rng.next_u32()), expected);
    }

    /// Seed 1 through Marsaglia's 64-bit xorshift with the (13, 7, 17) triple
    /// used here: `next_u64` returns the state and `next_u32` its high half.
    /// Values from an independent replica of the generator; Marsaglia's
    /// `xor64()` agrees for 5000 outputs at seeds 1 and 88172645463325252.
    #[test]
    fn xorshift64_seed_1_kat() {
        let expected: [u64; 8] = [
            0x0000_0000_4082_2041,
            0x1000_4106_0c01_1441,
            0x9b1e_842f_6e86_2629,
            0xf554_f503_555d_8025,
            0x860c_1fb0_9059_9265,
            0xf6b0_5302_e553_1801,
            0xa246_0108_ebbd_9e71,
            0xc62c_9fc1_14d9_590d,
        ];
        let mut wide = Xorshift64::new(1);
        assert_eq!(expected.map(|_| wide.next_u64()), expected);
        let mut narrow = Xorshift64::new(1);
        assert_eq!(
            expected.map(|_| narrow.next_u32()),
            expected.map(|w| (w >> 32) as u32)
        );
    }
}
