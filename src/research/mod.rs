//! Research-grade tests and analyses drawn from the bibliography in `BIB.md`.

pub mod approx_entropy;
pub mod knuth;
mod lz78_counts;
pub mod marsaglia_tsang;
pub mod practrand_fpf;
pub mod testu01_hamming;
pub mod testu01_lz;
pub mod webster_tavares;

/// The `s`-bit field of one 32-bit output word after its `r` most significant
/// bits: drop those bits and return the next `s` as an integer in `0..2^s`,
/// the bit extraction L'Ecuyer and Simard's tests use.
///
/// P. L'Ecuyer and R. Simard, "TestU01: A C Library for Empirical Testing of
/// Random Number Generators," *ACM Transactions on Mathematical Software*
/// 33(4), Article 22, 2007, p. 22.  [pubs/lecuyer-simard-2007-testu01.pdf]
///
/// Shared by `testu01_hamming` and `testu01_lz`; callers guarantee `s >= 1`
/// and `r + s <= 32`.
pub(crate) fn strip_b(word: u32, r: usize, s: usize) -> u32 {
    if r == 0 {
        word >> (32 - s)
    } else {
        (word << r) >> (32 - s)
    }
}

#[cfg(test)]
mod tests {
    use super::strip_b;

    #[test]
    fn strip_b_uses_most_significant_bits() {
        let word = 0xDEAD_BEEF;
        assert_eq!(0xD, strip_b(word, 0, 4));
        assert_eq!(0xE, strip_b(word, 4, 4));
    }
}
