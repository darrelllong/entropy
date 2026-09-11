//! NIST SP 800-22 §2.9 — Maurer's "Universal Statistical" Test.
//!
//! Tests compressibility: if the sequence is compressible it is not random.
//! Builds a table of the most recent position of each L-bit pattern, then
//! sums the log₂ of gaps between recurrences.
//!
//! Input size (SP 800-22 §2.9.7): 6 ≤ L ≤ 16, Q = 10·2^L and
//! K = n/L − Q ≈ 1000·2^L.  The §2.9.7 table starts L = 6 (Q = 640) at
//! n ≥ 387 840 and L = 7 (Q = 1280) at n ≥ 904 960, and ends with L = 15 at
//! n ≥ 496 435 200 and L = 16 at n ≥ 1 059 061 760.  [`universal`] picks L
//! from that table.
//!
//! # References
//! * A. Rukhin et al., *NIST SP 800-22 Rev. 1a*, 2010, §2.9.
//!   [pubs/NIST-SP-800-22r1a.pdf]
//! * U. M. Maurer, "A universal statistical test for random bit generators,"
//!   *Journal of Cryptology* 5(2), pp. 89–105, 1992.
//!   DOI: 10.1007/BF00193563.
//!   [pubs/maurer-1992-universal-test.pdf]
//!   [Table I: expected value of f_TU and variance of log₂ Aₙ for L = 1..16;
//!   eq. (13): c(L, K)]
//! * NIST, *Statistical Test Suite* 2.1.2, `src/universal.c`.
//!   [pubs/NIST-STS-2.1.2-src-and-constants.zip]  [Same L table, Q, K and
//!   c(L, K); μ and σ² to the printed digits]

use crate::{math::erfc, result::TestResult};
use std::f64::consts::SQRT_2;

/// Expected value μ of fₙ and variance σ² of log₂ Aₙ for each L.  Index is
/// L; entry 0 is unused.
///
/// Maurer (1992) Table I prints both for L = 1..16, compiled from his
/// eqs. (16) and (17), and SP 800-22 §2.9.4 step (5) reprints L = 6..16 with
/// the same digits: μ to 7 decimals (6 from L = 11) and σ² to 3.  Entries
/// L = 1..5 are those printed values.  The extra digits in the L = 6..16
/// entries come from an uncited source; neither table prints them.  They
/// agree with the printed digits to within one unit in the last place (σ²
/// for L = 8 is 3.23866…, which both tables print as 3.238).
///
/// They are the values themselves to about 10⁻¹²: with p = 2^−L, μ is
/// Σ_{i≥1} p(1 − p)^{i−1} log₂ i and σ² is Σ_{i≥1} p(1 − p)^{i−1} (log₂ i)² − μ²,
/// and a direct evaluation of both agrees with every L = 6..16 entry to
/// 10⁻¹¹ (`constants_match_maurer_series`).  STS 2.1.2's `universal.c` uses
/// the printed digits instead (6.1962507 and 3.125 for L = 7), which is why
/// STS and SP 800-22 Appendix B report P-value = 0.282568 for 10⁶ bits of e
/// where this module gives 0.282591.
const EXPECTED_LOG_GAP_STATS: [(f64, f64); 17] = [
    (0.0, 0.0), // L=0 unused
    (0.7326495, 0.690),
    (1.5374383, 1.338),
    (2.4016068, 1.901),
    (3.3112247, 2.358),
    (4.2534266, 2.705),
    (5.217705249861, 2.954032399382),
    (6.196250654102, 3.125391868609),
    (7.183665553492, 3.238662160971),
    (8.176424757913, 3.311200879481),
    (9.172324308195, 3.356456906974),
    (10.170032291923, 3.384087030672),
    (11.168764874402, 3.400654145108),
    (12.168070314219, 3.410438009177),
    (13.167692567118, 3.416141821798),
    (14.167488448576, 3.419430397755),
    (15.167378763638, 3.421308343033),
];

// L = 5 is included for research use: Maurer (1992) tabulates it, but NIST
// SP 800-22 Rev 1a defines the test only for L ∈ [6, 16], so the `nist::`
// wrapper below never selects it.
const PARAMETRIC_LS: [usize; 12] = [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
const PARAMETRIC_NAMES: [&str; 12] = [
    "maurer::universal_l05",
    "maurer::universal_l06",
    "maurer::universal_l07",
    "maurer::universal_l08",
    "maurer::universal_l09",
    "maurer::universal_l10",
    "maurer::universal_l11",
    "maurer::universal_l12",
    "maurer::universal_l13",
    "maurer::universal_l14",
    "maurer::universal_l15",
    "maurer::universal_l16",
];

/// Choose L automatically based on n.
fn choose_l(n: usize) -> usize {
    // The table in SP 800-22 §2.9.7.  Each threshold is n_min = L·(Q + K)
    // with Q = 10·2^L and K = 1000·2^L.
    // Rev 1a defines the test only for L ∈ [6, 16]; smaller L is available
    // through the `maurer::` parametric family, not the NIST-named wrapper.
    match n {
        n if n >= 1_059_061_760 => 16,
        n if n >= 496_435_200 => 15,
        n if n >= 231_669_760 => 14,
        n if n >= 107_560_960 => 13,
        n if n >= 49_643_520 => 12,
        n if n >= 22_753_280 => 11,
        n if n >= 10_342_400 => 10,
        n if n >= 4_654_080 => 9,
        n if n >= 2_068_480 => 8,
        n if n >= 904_960 => 7,
        n if n >= 387_840 => 6,
        _ => 0, // below the NIST-defined domain
    }
}

/// Run Maurer's universal statistical test.
///
/// # Reference
/// Rukhin et al., NIST SP 800-22 Rev 1a (2010), §2.9.
pub fn universal(bits: &[u8]) -> TestResult {
    let n = bits.len();
    let l = choose_l(n);
    if l == 0 {
        return TestResult::insufficient(
            "nist::universal",
            "n below the NIST-defined domain (need ≥ 387840 bits for L = 6)",
        );
    }

    let q = 10 * (1usize << l); // initialisation blocks
    universal_with_l(bits, l, q, "nist::universal")
}

/// Run Maurer's original parametric family for L = 5..=16, using
/// Q = 10·2^L and K = ⌊n/L⌋ − Q.
///
/// Always returns 12 results.  A setting runs only when K ≥ 1000·2^L, that
/// is n ≥ L·(Q + 1000·2^L), and is skipped otherwise.  Every row of the
/// SP 800-22 §2.9.7 table is exactly that bound (387 840 bits for L = 6 up to
/// 1 059 061 760 for L = 16), and Maurer (1992) gives K = 1000·2^L as his
/// example.  L = 5 follows the same rule and needs 161 600 bits.
///
/// This preserves the legacy NIST-shaped single result above while exposing the
/// more sensitive higher-L settings discussed in Maurer (1992).
pub fn universal_parametric_all(bits: &[u8]) -> Vec<TestResult> {
    PARAMETRIC_LS
        .into_iter()
        .zip(PARAMETRIC_NAMES)
        .map(|(l, name)| {
            let q = 10 * (1usize << l);
            universal_with_l(bits, l, q, name)
        })
        .collect()
}

fn universal_with_l(bits: &[u8], l: usize, q: usize, name: &'static str) -> TestResult {
    let n = bits.len();
    let n_blocks = n / l;
    // §2.9.7: K = n/L − Q ≈ 1000·2^L, and each row of its table is
    // n ≥ L·(Q + 1000·2^L).  Below that bound the setting is skipped.
    let k_min = 1000 * (1usize << l);
    if n_blocks < q + k_min {
        return TestResult::insufficient(
            name,
            &format!(
                "n too small for L={l}, Q={q}: K ≥ 1000·2^L needs n ≥ {} bits (§2.9.7)",
                l * (q + k_min)
            ),
        );
    }
    let k = n_blocks - q;

    let f_n = universal_statistic(bits, l, q, k);
    let (mu, sigma2) = EXPECTED_LOG_GAP_STATS[l];
    let sigma = universal_sigma(l, k, sigma2);
    let p_value = erfc((f_n - mu).abs() / (sigma * SQRT_2));

    TestResult::with_note(
        name,
        p_value,
        format!("n={n}, L={l}, Q={q}, K={k}, f_n={f_n:.4}, μ={mu:.4}, σ={sigma:.6}"),
    )
}

fn universal_statistic(bits: &[u8], l: usize, q: usize, k: usize) -> f64 {
    // Build the initialisation table: last occurrence of each L-bit pattern.
    let mut table = vec![0usize; 1usize << l];

    for i in 0..q {
        let pattern = bits_to_index(&bits[i * l..(i + 1) * l]);
        table[pattern] = i + 1;
    }

    let mut sum = 0.0f64;
    for i in q..q + k {
        let pattern = bits_to_index(&bits[i * l..(i + 1) * l]);
        let gap = i + 1 - table[pattern];
        sum += (gap as f64).log2();
        table[pattern] = i + 1;
    }

    sum / k as f64
}

/// Standard deviation of fₙ: σ = c(L, K)·√(σ²/K).
///
/// c(L, K) = 0.7 − 0.8/L + (4 + 32/L)·K^(−3/L)/15 is the form of SP 800-22
/// Rev. 1a §2.9.4 step (5), which is Maurer's (1992) eq. (13).  §3.9 also
/// prints the later Coron–Naccache approximation
/// c(L, K) = 0.7 − 0.8/L + (1.6 + 12.8/L)·K^(−4/L) (its reference [2], SAC '98)
/// but says it is not embedded in the test suite code, so it is not used here.
/// STS 2.1.2's `universal.c` computes this c(L, K).
fn universal_sigma(l: usize, k: usize, sigma2: f64) -> f64 {
    let l = l as f64;
    let k = k as f64;
    let c = 0.7 - 0.8 / l + (4.0 + 32.0 / l) * k.powf(-3.0 / l) / 15.0;
    c * (sigma2 / k).sqrt()
}

/// Interpret an L-bit slice (values 0/1) as a big-endian index.
fn bits_to_index(bits: &[u8]) -> usize {
    bits.iter().fold(0usize, |acc, &b| (acc << 1) | b as usize)
}

#[cfg(test)]
mod tests {
    use super::{
        choose_l, universal, universal_parametric_all, universal_sigma, universal_statistic,
        EXPECTED_LOG_GAP_STATS,
    };
    use crate::math::erfc;
    use crate::nist::test_vectors::e_bits;
    use std::f64::consts::SQRT_2;

    /// μ and σ² as printed in Maurer (1992) Table I for L = 1..16; SP 800-22
    /// §2.9.4 step (5) reprints L = 6..16 with the same digits.
    const PRINTED_TABLE: [(f64, f64); 16] = [
        (0.7326495, 0.690),
        (1.5374383, 1.338),
        (2.4016068, 1.901),
        (3.3112247, 2.358),
        (4.2534266, 2.705),
        (5.2177052, 2.954),
        (6.1962507, 3.125),
        (7.1836656, 3.238),
        (8.1764248, 3.311),
        (9.1723243, 3.356),
        (10.170032, 3.384),
        (11.168765, 3.401),
        (12.168070, 3.410),
        (13.167693, 3.416),
        (14.167488, 3.419),
        (15.167379, 3.421),
    ];

    /// Each constant is within one unit of the last printed place.
    #[test]
    fn constants_agree_with_printed_tables() {
        for (l, (mu, var)) in (1..=16).zip(PRINTED_TABLE) {
            let (m, v) = EXPECTED_LOG_GAP_STATS[l];
            let last_mu_place = if l < 11 { 1e-7 } else { 1e-6 };
            assert!((m - mu).abs() <= last_mu_place, "μ for L = {l}: {m}");
            assert!((v - var).abs() <= 1e-3, "σ² for L = {l}: {v}");
        }
    }

    /// `universal` takes L from the §2.9.7 row at its n and the previous
    /// row's L one bit below it.
    #[test]
    fn nist_wrapper_follows_section_2_9_7_table() {
        for (l, n_min) in SECTION_2_9_7_TABLE {
            assert_eq!(choose_l(n_min), l, "n = {n_min}");
            let below = if l == 6 { 0 } else { l - 1 };
            assert_eq!(choose_l(n_min - 1), below, "n = {}", n_min - 1);
        }
    }

    /// The L = 6..16 entries against the series for μ and σ² in the table's
    /// doc, summed over i ≤ 45·2^L; the tail left out weighs
    /// (1 − 2^−L)^{45·2^L} < e^−45.
    #[test]
    fn constants_match_maurer_series() {
        for (l, &(mu, var)) in EXPECTED_LOG_GAP_STATS.iter().enumerate().skip(6) {
            let p = 2f64.powi(-(l as i32));
            let ln_q = (-p).ln_1p();
            let (mut mean, mut second) = (0.0, 0.0);
            for i in 1..=(45usize << l) {
                let w = p * ((i - 1) as f64 * ln_q).exp();
                let lg = (i as f64).log2();
                mean += w * lg;
                second += w * lg * lg;
            }
            assert!((mean - mu).abs() < 1e-11, "μ for L = {l}: {mean}");
            let variance = second - mean * mean;
            assert!((variance - var).abs() < 1e-11, "σ² for L = {l}: {variance}");
        }
    }

    #[test]
    fn published_table_covers_l16() {
        let (mu, var) = EXPECTED_LOG_GAP_STATS[16];
        assert!((mu - 15.167378763638).abs() < 1e-12);
        assert!((var - 3.421308343033).abs() < 1e-12);
    }

    /// σ for L = 7, K = 1000 with §2.9.4's c(L, K), from an independent
    /// Python evaluation of the formula.  The Coron–Naccache form in §3.9
    /// gives 0.036445141413707395 here, which the old code returned.
    #[test]
    fn uses_nist_correction_factor() {
        let sigma = universal_sigma(7, 1_000, EXPECTED_LOG_GAP_STATS[7].1);
        assert!((sigma - 0.034399103037796475).abs() < 1e-15, "σ = {sigma}");
    }

    /// SP 800-22 §2.9.8: n = 1 048 576, L = 7, Q = 1280, so K = 148 516;
    /// sum = 919 924.038020, and with expectedValue(7) = 6.1962507 and
    /// variance(7) = 3.125 from the §2.9.4 table the publication prints
    /// σ = 0.002703 and P-value = 0.427733.  The Coron–Naccache c(L, K)
    /// gives σ = 0.002704 and P-value = 0.427991.  (The printed
    /// c = 0.591311 matches neither form; the printed σ matches §2.9.4's.)
    #[test]
    fn sigma_reproduces_section_2_9_8_example() {
        let (l, q) = (7, 1280);
        let k = 1_048_576 / l - q;
        assert_eq!(k, 148_516);
        let sigma = universal_sigma(l, k, 3.125);
        assert!((sigma - 0.002703).abs() < 5e-7, "σ = {sigma}");
        let f_n = 919_924.038020 / k as f64;
        let p = erfc((f_n - 6.1962507).abs() / (sigma * SQRT_2));
        assert!((p - 0.427733).abs() < 1e-6, "p = {p}");
        // The shipped 12-digit table entry for L = 7 is more precise than the
        // printed 6.1962507 and 3.125; an independent Python replica gives
        // σ = 0.002702824 and P = 0.427772059 for the same printed sum.
        let (mu7, var7) = EXPECTED_LOG_GAP_STATS[l];
        let sigma = universal_sigma(l, k, var7);
        let p = erfc((f_n - mu7).abs() / (sigma * SQRT_2));
        assert!((sigma - 0.002702824).abs() < 1e-9, "shipped σ = {sigma}");
        assert!((p - 0.427772059).abs() < 1e-8, "shipped p = {p}");
    }

    /// SP 800-22 Appendix B prints P-value = 0.282568 for 10⁶ bits of e, where
    /// L = 7, Q = 1280 and K = 141 577.  STS 2.1.2 reports sum = 877 667.758407
    /// and reaches that P-value with the printed μ = 6.1962507 and σ² = 3.125;
    /// the 12-digit table entries this module uses give 0.282591 for the same
    /// sum.
    #[test]
    fn matches_appendix_b_e_row() {
        let e = e_bits(1_000_000);
        let r = universal(&e);
        assert!((r.p_value - 0.282591).abs() < 1e-6, "{r}");
        let (l, q) = (7, 1280);
        let k = e.len() / l - q;
        assert_eq!(k, 141_577);
        let f_n = universal_statistic(&e, l, q, k);
        let sum = f_n * k as f64;
        assert!((sum - 877_667.758407).abs() < 1e-5, "sum = {sum}");
        let sigma = universal_sigma(l, k, 3.125);
        let p = erfc((f_n - 6.1962507).abs() / (sigma * SQRT_2));
        assert!((p - 0.282568).abs() < 1e-6, "p = {p}");
    }

    /// The SP 800-22 §2.9.7 table, as (L, minimum n).
    const SECTION_2_9_7_TABLE: [(usize, usize); 11] = [
        (6, 387_840),
        (7, 904_960),
        (8, 2_068_480),
        (9, 4_654_080),
        (10, 10_342_400),
        (11, 22_753_280),
        (12, 49_643_520),
        (13, 107_560_960),
        (14, 231_669_760),
        (15, 496_435_200),
        (16, 1_059_061_760),
    ];

    #[test]
    fn parametric_family_marks_unavailable_l_values_as_skipped() {
        // 10^6 bits reach K ≥ 1000·2^L for L = 5, 6 and 7 only.
        let bits = vec![0u8; 1_000_000];
        let results = universal_parametric_all(&bits);
        assert_eq!(results.len(), 12);
        for (r, l) in results.iter().zip(5..=16) {
            assert_eq!(r.skipped(), l > 7, "L = {l}: {r}");
        }
    }

    /// Every §2.9.7 row is n = L·(Q + K) with Q = 10·2^L and K = 1000·2^L,
    /// so a Maurer setting runs from that n and is skipped one bit below it.
    #[test]
    fn parametric_family_requires_k_of_1000_times_2_to_the_l() {
        for (l, n_min) in SECTION_2_9_7_TABLE {
            assert_eq!(l * (10 + 1000) * (1 << l), n_min, "L = {l}");
        }
        let (l, n_min) = SECTION_2_9_7_TABLE[0];
        let slot = l - 5;
        let at = universal_parametric_all(&vec![0u8; n_min]);
        let below = universal_parametric_all(&vec![0u8; n_min - 1]);
        assert!(!at[slot].skipped(), "{}", at[slot]);
        assert!(below[slot].skipped(), "{}", below[slot]);
    }

    #[test]
    fn nist_wrapper_still_runs_on_minimum_supported_size() {
        let bits = vec![0u8; 387_840];
        let result = universal(&bits);
        assert_eq!(result.name, "nist::universal");
        assert!(!result.skipped());
    }
}
