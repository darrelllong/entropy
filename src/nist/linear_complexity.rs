//! NIST SP 800-22 §2.10 — Linear Complexity Test.
//!
//! Divides the sequence into M-bit blocks, computes the linear complexity
//! (shortest LFSR length) of each block via the Berlekamp-Massey algorithm,
//! and tests whether the distribution of complexities is consistent with an
//! i.i.d. Bernoulli(½) source.
//!
//! Recommended defaults: M = 500, n ≥ 10^6.
//!
//! # References
//! * A. Rukhin et al., *NIST SP 800-22 Rev. 1a*, 2010, §2.10.
//!   [pubs/NIST-SP-800-22r1a.pdf]
//! * J. L. Massey, "Shift-register synthesis and BCH decoding,"
//!   *IEEE Transactions on Information Theory* 15(1), pp. 122–127, January 1969.
//!   DOI: 10.1109/TIT.1969.1054260.
//!   [Berlekamp-Massey algorithm used to compute LFSR length of each block]

use crate::{math::chi2_pvalue, result::TestResult};

/// Run the linear complexity test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.10.
pub fn linear_complexity(bits: &[u8], m: usize) -> TestResult {
    if !(500..=5000).contains(&m) {
        return TestResult::unsupported("nist::linear_complexity", "M must be in [500, 5000]");
    }

    let n = bits.len();
    let num_blocks = n / m;

    if num_blocks < 200 {
        return TestResult::insufficient(
            "nist::linear_complexity",
            "n too small — need ≥ 200 blocks",
        );
    }

    let nu = class_counts(bits, m);

    let chi_sq: f64 = nu
        .iter()
        .zip(PI.iter())
        .map(|(&count, &p)| {
            let exp = num_blocks as f64 * p;
            (count as f64 - exp).powi(2) / exp
        })
        .sum();

    let p_value = chi2_pvalue(chi_sq, 6);

    TestResult::with_note(
        "nist::linear_complexity",
        p_value,
        format!("n={n}, M={m}, N={num_blocks}, χ²={chi_sq:.4}"),
    )
    .chi_square(chi_sq, 6.0)
}

/// π₀, …, π₆ of SP 800-22 §2.10.4 step (6), the probabilities of the seven
/// classes of Tᵢ derived in §3.10.  SP 800-22's §2.10.8 and Appendix B
/// figures follow from π₀ = 0.01047 instead (see the tests).
const PI: [f64; 7] = [
    0.010417, 0.031250, 0.125000, 0.500000, 0.250000, 0.062500, 0.020833,
];

/// The theoretical mean μ of SP 800-22 §2.10.4 step (3), which the
/// publication numbers (1):
///
/// μ = M/2 + (9 + (−1)^{M+1})/36 − (M/3 + 2/9)/2^M.
///
/// The middle term is 8/36 for even M and 10/36 for odd M, the (4 + r)/18
/// with r = M mod 2 in §3.10's ξ.  With this μ every Tᵢ lies within
/// (M/3 + 2/9)/2^M of an integer, which is why the class boundaries sit at
/// half-integers.
fn mean(m: usize) -> f64 {
    let pow_neg1_m_plus_1 = if m.is_multiple_of(2) {
        -1.0_f64
    } else {
        1.0_f64
    };
    m as f64 / 2.0 + (9.0 + pow_neg1_m_plus_1) / 36.0
        - (m as f64 / 3.0 + 2.0 / 9.0) / 2f64.powi(m as i32)
}

/// ν₀, …, ν₆ of §2.10.4 step (5): how many M-bit blocks of `bits` put Tᵢ in
/// each class.
fn class_counts(bits: &[u8], m: usize) -> [usize; 7] {
    let mu = mean(m);

    // Category boundaries for T = (−1)^M (L − μ) + 2/9.
    // Six categories: T ≤ −2.5, (−2.5,−1.5], (−1.5,−0.5], (−0.5,0.5],
    //                 (0.5,1.5], (1.5,2.5], T > 2.5  (7 categories total).
    let mut nu = [0usize; 7];
    let sign = if m.is_multiple_of(2) { 1.0 } else { -1.0 };

    for block in bits.chunks_exact(m) {
        let l = berlekamp_massey(block) as f64;
        let t = sign * (l - mu) + 2.0 / 9.0;
        let idx = if t <= -2.5 {
            0
        } else if t <= -1.5 {
            1
        } else if t <= -0.5 {
            2
        } else if t <= 0.5 {
            3
        } else if t <= 1.5 {
            4
        } else if t <= 2.5 {
            5
        } else {
            6
        };
        nu[idx] += 1;
    }
    nu
}

/// Berlekamp-Massey algorithm: returns the linear complexity (shortest LFSR
/// length) of the binary sequence `s`.
///
/// SP 800-22 §2.10.4 step (2) calls for this algorithm and, in its footnote 5,
/// takes the definition from A. Menezes, P. Van Oorschot and S. Vanstone,
/// *Handbook of Applied Cryptography*, CRC Press, 1997; Massey (1969) is in
/// the module references.
pub fn berlekamp_massey(s: &[u8]) -> usize {
    let big_n = s.len();
    let mut c = vec![0u8; big_n + 1];
    let mut b = vec![0u8; big_n + 1];
    // Holds C(D) across a length change and then becomes B(D) by swap, so
    // the loop allocates once instead of cloning C(D) on every discrepancy.
    let mut t = vec![0u8; big_n + 1];
    c[0] = 1;
    b[0] = 1;
    let mut l = 0usize;
    let mut m: i64 = -1;

    for n in 0..big_n {
        // Discrepancy d = s[n] XOR (Σ c[i]·s[n−i] for i = 1..=L)
        let mut d = s[n];
        for i in 1..=l {
            d ^= c[i] & s[n - i];
        }
        if d == 0 {
            continue;
        }
        let lengthens = 2 * l <= n;
        if lengthens {
            t.copy_from_slice(&c);
        }
        let shift = (n as i64 - m) as usize;
        for i in shift..=big_n {
            c[i] ^= b[i - shift];
        }
        if lengthens {
            l = n + 1 - l;
            std::mem::swap(&mut b, &mut t);
            m = n as i64;
        }
    }
    l
}

#[cfg(test)]
mod tests {
    /// A value this test computes exactly, up to floating rounding.
    const CLOSED_FORM: f64 = 1e-12;

    /// SP 800-22 quotes its example values to six decimals.
    const PUBLISHED: f64 = 1e-6;

    use super::*;
    use crate::nist::test_vectors::{bits, e_bits};
    use crate::rng::{Mt19937, Rng};

    /// The π₀ with which the publication's worked figures were computed,
    /// 0.01047, where §2.10.4 step (6) and §3.10 print 0.010417 (the class
    /// probability is 1/96).
    const WORKED_PI0: f64 = 0.01047;

    /// Pearson χ² of class counts against class probabilities.
    fn chi_square(nu: [usize; 7], pi: [f64; 7]) -> f64 {
        let total = nu.iter().sum::<usize>() as f64;
        nu.iter()
            .zip(pi)
            .map(|(&v, p)| (v as f64 - total * p).powi(2) / (total * p))
            .sum()
    }

    /// The battery's M = 500 is even, so μ = 250 + 8/36 − (500/3 + 2/9)/2^500,
    /// whose last term (about 5 × 10⁻¹⁴⁹) is far below f64 resolution.  Every
    /// Tᵢ is then an integer, Lᵢ − 250.
    #[test]
    fn mean_for_even_block_length() {
        let mu = mean(500);
        assert!((mu - (250.0 + 8.0 / 36.0)).abs() < CLOSED_FORM, "μ = {mu}");
        for l in 247..=253 {
            let t = (l as f64 - mu) + 2.0 / 9.0;
            assert!(
                (t - (l as f64 - 250.0)).abs() < CLOSED_FORM,
                "L = {l}: T = {t}"
            );
        }
    }

    /// The first 100 000 bits of e with the battery's M = 500, at the fewest
    /// blocks the gate accepts (N = 200), so debug builds and CI score linear
    /// complexity end to end: class counts ν = (4, 5, 25, 106, 44, 13, 3),
    /// χ² = 3.439789 and P-value = 0.751963, pinned.
    #[test]
    fn pinned_on_100_000_bits_of_e() {
        const NU: [usize; 7] = [4, 5, 25, 106, 44, 13, 3];
        let e = e_bits(100_000);
        assert_eq!(class_counts(&e, 500), NU);
        let r = linear_complexity(&e, 500);
        assert!((r.p_value - 0.751963).abs() < PUBLISHED, "{r}");
        assert!(
            r.note.as_deref().unwrap().contains("N=200, χ²=3.4398"),
            "{r}"
        );
    }

    /// SP 800-22 §2.10.8 on 10⁶ bits of e with M = 1000: the printed counts
    /// ν = (11, 31, 116, 501, 258, 57, 26) and, from them, χ² = 2.700348 and
    /// P-value = 0.845406.  Those two figures use π₀ = 0.01047; with
    /// the π₀ = 0.010417 of §2.10.4 the same counts give χ² = 2.706147 and
    /// P-value = 0.844721, which this module returns.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "about 13 s unoptimised; run with cargo test --release"
    )]
    fn matches_section_2_10_8_counts() {
        const PRINTED_NU: [usize; 7] = [11, 31, 116, 501, 258, 57, 26];
        let e = e_bits(1_000_000);
        assert_eq!(class_counts(&e, 1000), PRINTED_NU);
        let r = linear_complexity(&e, 1000);
        assert!((r.p_value - 0.844721).abs() < PUBLISHED, "{r}");
        assert!((chi_square(PRINTED_NU, PI) - 2.706147).abs() < PUBLISHED);
        let mut worked_pi = PI;
        worked_pi[0] = WORKED_PI0;
        let printed = chi_square(PRINTED_NU, worked_pi);
        assert!((printed - 2.700348).abs() < PUBLISHED, "χ² = {printed}");
        assert!((chi2_pvalue(printed, 6) - 0.845406).abs() < PUBLISHED);
    }

    /// SP 800-22 Appendix B prints P-value = 0.826335 for 10⁶ bits of e with
    /// M = 500.  The class counts on these bits are
    /// ν = (21, 52, 250, 1006, 492, 135, 44), which give that figure
    /// (χ² = 2.858915) with π₀ = 0.01047; the printed π₀ gives χ² = 2.860066
    /// and P-value = 0.826194.
    #[test]
    #[cfg_attr(
        debug_assertions,
        ignore = "about 7 s unoptimised; run with cargo test --release"
    )]
    fn matches_appendix_b_e_row() {
        const NU: [usize; 7] = [21, 52, 250, 1006, 492, 135, 44];
        let e = e_bits(1_000_000);
        assert_eq!(class_counts(&e, 500), NU);
        let r = linear_complexity(&e, 500);
        assert!((r.p_value - 0.826194).abs() < PUBLISHED, "{r}");
        let mut worked_pi = PI;
        worked_pi[0] = WORKED_PI0;
        let worked = chi_square(NU, worked_pi);
        assert!((worked - 2.858915).abs() < PUBLISHED, "χ² = {worked}");
        assert!((chi2_pvalue(worked, 6) - 0.826335).abs() < PUBLISHED);
    }

    /// Berlekamp–Massey written plainly, cloning C(D) on every discrepancy.
    fn berlekamp_massey_cloning(s: &[u8]) -> usize {
        let big_n = s.len();
        let mut c = vec![0u8; big_n + 1];
        let mut b = vec![0u8; big_n + 1];
        c[0] = 1;
        b[0] = 1;
        let mut l = 0usize;
        let mut m: i64 = -1;
        for n in 0..big_n {
            let mut d = s[n];
            for i in 1..=l {
                d ^= c[i] & s[n - i];
            }
            if d == 0 {
                continue;
            }
            let t = c.clone();
            let shift = (n as i64 - m) as usize;
            for i in shift..=big_n {
                c[i] ^= b[i - shift];
            }
            if 2 * l <= n {
                l = n + 1 - l;
                b = t;
                m = n as i64;
            }
        }
        l
    }

    /// SP 800-22 §2.10.4: the block 1101011110001 (M = 13) has Lᵢ = 4, and
    /// M = 13 gives μ = 6.777222 and Tᵢ = 2.999444.
    #[test]
    fn matches_section_2_10_4_example() {
        assert_eq!(berlekamp_massey(&bits("1101011110001")), 4);
        let mu = mean(13);
        assert!((mu - 6.777222).abs() < PUBLISHED, "μ = {mu}");
        let t = -(4.0 - mu) + 2.0 / 9.0;
        assert!((t - 2.999444).abs() < PUBLISHED, "T = {t}");
    }

    /// The scratch-buffer loop returns what the cloning loop returned, on
    /// fixed Mt19937 blocks around the battery's M = 500 and on all-zero and
    /// single-one blocks.
    #[test]
    fn scratch_buffer_matches_cloning_loop() {
        let mut rng = Mt19937::new(5489);
        for len in (0..64).chain([499, 500, 501, 1000]) {
            for _ in 0..8 {
                let block = rng.collect_bits(len);
                let want = berlekamp_massey_cloning(&block);
                assert_eq!(berlekamp_massey(&block), want, "len = {len}");
            }
        }
        for len in [1usize, 13, 500] {
            let zeros = vec![0u8; len];
            let mut impulse = zeros.clone();
            impulse[len - 1] = 1;
            assert_eq!(berlekamp_massey(&zeros), 0);
            assert_eq!(berlekamp_massey(&impulse), len);
            assert_eq!(berlekamp_massey_cloning(&impulse), len);
        }
    }
}
