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

use crate::{math::igamc, result::TestResult};

/// Run the linear complexity test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.10.
pub fn linear_complexity(bits: &[u8], m: usize) -> TestResult {
    if !(500..=5000).contains(&m) {
        return TestResult::insufficient("nist::linear_complexity", "M must be in [500, 5000]");
    }

    let n = bits.len();
    let num_blocks = n / m;

    if num_blocks < 200 {
        return TestResult::insufficient(
            "nist::linear_complexity",
            "n too small — need ≥ 200 blocks",
        );
    }

    // Theoretical mean μ = M/2 + (9 + (−1)^M)/36 − (M/3 + 2/9)/2^M.
    // SP 800-22 §2.10.4 and NIST STS linear.c: the numerator is (9 + (−1)^M),
    // which is 10 for even M and 8 for odd M — NOT (9 + M%2).
    let pow_neg1_m = if m.is_multiple_of(2) {
        1.0_f64
    } else {
        -1.0_f64
    };
    let mu = m as f64 / 2.0 + (9.0 + pow_neg1_m) / 36.0
        - (m as f64 / 3.0 + 2.0 / 9.0) / 2f64.powi(m as i32);

    // Category boundaries for T = (−1)^M (L − μ) + 2/9.
    // Six categories: T ≤ −2.5, (−2.5,−1.5], (−1.5,−0.5], (−0.5,0.5],
    //                 (0.5,1.5], (1.5,2.5], T > 2.5  (7 categories total).
    let pi = [
        0.010417, 0.031250, 0.125000, 0.500000, 0.250000, 0.062500, 0.020833,
    ];

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

    let chi_sq: f64 = nu
        .iter()
        .zip(pi.iter())
        .map(|(&count, &p)| {
            let exp = num_blocks as f64 * p;
            (count as f64 - exp).powi(2) / exp
        })
        .sum();

    let p_value = igamc(3.0, chi_sq / 2.0); // df = 6

    TestResult::with_note(
        "nist::linear_complexity",
        p_value,
        format!("n={n}, M={m}, N={num_blocks}, χ²={chi_sq:.4}"),
    )
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
    use super::*;
    use crate::nist::test_vectors::bits;
    use crate::rng::{Mt19937, Rng};

    /// The loop as it stood before the scratch buffer, cloning C(D) on every
    /// discrepancy.
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

    /// SP 800-22 §2.10.4 step (2): the block 1101011110001 (M = 13) has
    /// Lᵢ = 4.  (The §2.10.8 example runs on 10⁶ bits of e, which the crate
    /// does not ship.)
    #[test]
    fn matches_section_2_10_4_example() {
        assert_eq!(berlekamp_massey(&bits("1101011110001")), 4);
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
