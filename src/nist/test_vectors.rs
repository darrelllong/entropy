//! Worked-example inputs from SP 800-22 Rev. 1a, shared by the unit tests.

/// ε of the §2.1.8, §2.3.8, §2.6.8, §2.12.8 and §2.13.8 examples (n = 100).
pub(crate) const EPSILON_100: &str = concat!(
    "11001001000011111101101010100010001000010110100011",
    "00001000110100110001001100011001100010100010111000",
);

/// Parse a string of `0` and `1` characters into bits.
pub(crate) fn bits(s: &str) -> Vec<u8> {
    s.bytes()
        .map(|c| match c {
            b'0' => 0,
            b'1' => 1,
            _ => panic!("not a bit: {}", c as char),
        })
        .collect()
}

/// The first 10⁶ bits of the binary expansion of e: the input of the §2.5.8,
/// §2.8.8, §2.10.8, §2.11.8, §2.14.8 and §2.15.8 examples and of the second
/// table of Appendix B.
///
/// Source: member `sts-2.1.2/data/data.e` of
/// [pubs/NIST-STS-2.1.2-src-and-constants.zip], whose sha256 is
/// `b4b49e2987dacdbd5fe75a8ad767f4139e7be7d8f0ff9ddfb23e6081a5feed2d`.  That
/// file holds 1 004 882 ASCII `0` and `1` digits on space-indented lines,
/// starting with the `10` of e's integer part, as written by the Mathematica
/// program of SP 800-22 Appendix F.3 (`RealDigits[N[E, d], 2]`).  STS reads it
/// one digit at a time with `fscanf(fp, "%1d", &bit)`, which skips the
/// whitespace (`readBinaryDigitsInASCIIFormat` in `src/utilities.c`).
///
/// Packing: the first 10⁶ digits, in order, most significant bit first.
/// Digit j is bit 7 − (j mod 8) of byte ⌊j/8⌋, so the file is 125 000 bytes;
/// its sha256 is
/// `7ae61691f949a9a92d5ed8b65722bfcf0179964064d5f2c7e2a971b32ac97d49`.
const E_FIXTURE: &[u8] = include_bytes!("../../tests/data/e_1e6_bits.bin");

/// Number of bits in the e fixture.
pub(crate) const E_BITS: usize = 1_000_000;

/// The first `n` bits of the binary expansion of e, one bit per element.
pub(crate) fn e_bits(n: usize) -> Vec<u8> {
    assert!(n <= E_BITS, "the e fixture holds {E_BITS} bits");
    (0..n)
        .map(|j| (E_FIXTURE[j / 8] >> (7 - j % 8)) & 1)
        .collect()
}

#[cfg(all(test, feature = "cryptography"))]
mod tests {
    use super::{e_bits, E_BITS, E_FIXTURE};
    use cryptography::vt::BigUint;

    /// Leading fixture bits compared with e computed from its series.
    const CROSS_CHECKED_BITS: usize = 1 << 14;

    /// ⌊e·2^(k−2)⌋, whose k binary digits are the first k digits of e's
    /// expansion 10.1011011111100001… (e lies in [2, 4)).
    ///
    /// Sums ⌊2^(k−2+G)/i!⌋ over i ≥ 0 with G guard bits, each term the floor
    /// of the previous one over i.  Every floor loses less than one unit and
    /// the terms left out after the first zero total less than two, so the
    /// sum falls short of e·2^(k−2+G) by less than i_max + 2, about 2 000
    /// units here; dropping the G = 64 guard bits then gives ⌊e·2^(k−2)⌋
    /// unless e·2^(k−2) is within 2^−53 of an integer.
    fn e_prefix(k: usize) -> BigUint {
        const GUARD_BITS: usize = 64;
        let scale = k - 2 + GUARD_BITS;
        let mut term = BigUint::one();
        term.shl_bits(scale);
        let mut sum = term.clone();
        let mut i = 1u64;
        while !term.is_zero() {
            term = term.div_rem_u64(i).0;
            sum += &term;
            i += 1;
        }
        sum.shr_bits(GUARD_BITS);
        sum
    }

    /// SP 800-22 prints no digits of e, but §2.11.8 prints the symbol counts
    /// of these 10⁶ bits: 499 971 zeros and 500 029 ones and, reading the
    /// sequence cyclically, 250 116 of 00, 249 855 each of 01 and 10, and
    /// 250 174 of 11.
    #[test]
    fn e_fixture_has_the_counts_printed_in_section_2_11_8() {
        assert_eq!(E_FIXTURE.len() * 8, E_BITS);
        let e = e_bits(E_BITS);
        let ones = e.iter().filter(|&&b| b == 1).count();
        assert_eq!((E_BITS - ones, ones), (499_971, 500_029));
        let mut pairs = [0usize; 4];
        for (&a, &b) in e.iter().zip(e.iter().cycle().skip(1)) {
            pairs[usize::from((a << 1) | b)] += 1;
        }
        assert_eq!(pairs, [250_116, 249_855, 249_855, 250_174]);
    }

    /// The fixture's first 2¹⁴ bits are those of e computed with rump's
    /// `BigUint`.
    #[test]
    fn e_fixture_prefix_matches_e_computed_with_biguint() {
        let prefix = e_prefix(CROSS_CHECKED_BITS);
        assert_eq!(prefix.bits(), CROSS_CHECKED_BITS);
        for (j, &b) in e_bits(CROSS_CHECKED_BITS).iter().enumerate() {
            let want = prefix.bit(CROSS_CHECKED_BITS - 1 - j);
            assert_eq!(b == 1, want, "bit {j} of e");
        }
    }
}
