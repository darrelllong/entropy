//! What the golden tests compare against: the input of the DIEHARD fidelity
//! review's gfortran build of `diehard.f`, regenerated in memory, and the
//! formula that build's `KSTEST` prints.
//!
//! The review built Marsaglia's `fortran/diehard.f` with gfortran 16.2 and
//! `-fno-automatic`, patched only so that `jkreset` also resets `jtbl`'s
//! buffer index (the "aligned" build: every test and every bit window starts
//! at word 1), and ran it on 4 000 000 little-endian words that NumPy 2.0.2
//! wrote as
//! `np.random.default_rng(20260911).integers(0, 2**32, size=4_000_000, dtype=np.uint32)`.
//!
//! NumPy's generator is O'Neill's PCG64 (XSL-RR 128/64).  Its `SeedSequence`
//! leaves `default_rng(20260911)` at state
//! 69363733736698288865544730332026520909 and increment
//! 160378821636518163781755759234662531153; [`Pcg64::new`] reaches exactly
//! that state from the two arguments below.  NumPy then hands out each 64-bit
//! output as its low 32 bits followed by its high 32 bits.

use crate::rng::{Pcg64, Rng};

/// `Pcg64::new` state argument that lands on NumPy's seeded state.
const SEED_STATE: u128 = 262_632_947_022_000_207_432_099_020_054_021_885_563;
/// `Pcg64::new` stream argument: NumPy's increment shifted right by one.
const SEED_STREAM: u128 = 80_189_410_818_259_081_890_877_879_617_331_265_576;

/// The first `n` words of `in.bin`.
pub(super) fn words(n: usize) -> Vec<u32> {
    let mut rng = Pcg64::new(SEED_STATE, SEED_STREAM);
    let mut out = Vec::with_capacity(n + 1);
    while out.len() < n {
        let x = rng.next_u64();
        out.push(x as u32);
        out.push((x >> 32) as u32);
    }
    out.truncate(n);
    out
}

/// Marsaglia's `KSTEST` p (`fortran/diehard.f` lines 1668–1709): his
/// asymptotic approximation to the Anderson–Darling CDF at A² = `z`, which
/// the routine returns without the small-sample correction it computes, so
/// `n` does not enter.  The golden tests use it to compare with the values
/// the gfortran build printed.
pub(super) fn diehard_kstest_cdf(_n: usize, z: f64) -> f64 {
    if z < 0.01 {
        0.0
    } else if z <= 2.0 {
        2.0 * (-1.2337 / z).exp() * (1.0 + z / 8.0 - 0.04958 * z * z / (1.325 + z)) / z.sqrt()
    } else if z <= 4.0 {
        1.0 - 0.6621361 * (-1.091638 * z).exp() - 0.95059 * (-2.005138 * z).exp()
    } else {
        1.0 - 0.4938691 * (-1.050321 * z).exp() - 0.5946335 * (-1.527198 * z).exp()
    }
}

/// Sums (mod 2⁶⁴) and last words of `in.bin`'s prefixes, read from the file
/// with NumPy, at the lengths the golden tests use.
#[test]
fn regenerates_the_review_input() {
    let all = words(1_000_005);
    assert_eq!(&all[..2], &[130_883_010, 2_848_563_236]);
    for (n, sum, last) in [
        (199_000, 427_656_745_829_642_u64, 483_034_516_u32),
        (256_005, 550_220_057_975_218, 3_199_886_379),
        (600_000, 1_288_747_560_480_718, 3_255_754_326),
        (1_000_005, 2_147_984_049_878_678, 1_138_279_843),
    ] {
        let prefix = &all[..n];
        let got = prefix
            .iter()
            .fold(0u64, |s, &w| s.wrapping_add(u64::from(w)));
        assert_eq!((got, prefix[n - 1]), (sum, last), "prefix of {n} words");
    }
}
