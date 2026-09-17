//! Every recorded statistic reproduces its result's p-value under the null
//! distribution it names.
//!
//! The suites run on MT19937; each chi-square statistic must give its p-value
//! through `chi2_pvalue`, each two-sided normal z through erfc(|z|/√2), each
//! Kolmogorov–Smirnov D and Anderson–Darling A² through their distributions
//! at the recorded sample size.  A
//! statistic recorded with the wrong degrees of freedom or scale fails here,
//! and so does a scored result that records none.

use entropy::{
    diehard, dieharder,
    math::{anderson_darling_cdf, chi2_pvalue, erfc, igamc, ks_pvalue},
    nist,
    research::{knuth, practrand_fpf, testu01_hamming, testu01_lz},
    result::{Status, TestResult},
    rng::{Mt19937, Rng},
};

fn check(results: &[TestResult]) -> usize {
    let mut checked = 0;
    for r in results {
        if r.status == Status::Scored {
            assert!(r.statistic.is_some(), "{} records no statistic", r.name);
        }
        let Some(stat) = &r.statistic else { continue };
        if !r.p_value.is_finite() || r.p_value == 0.0 {
            continue;
        }
        let expected = match stat.null {
            "chi-square" => chi2_pvalue(stat.value, stat.df.expect("chi-square df") as usize),
            "normal, two-sided" => erfc(stat.value.abs() / std::f64::consts::SQRT_2),
            "chi-square, two-sided" => {
                let upper = igamc(stat.df.expect("df") / 2.0, stat.value.max(0.0) / 2.0);
                (2.0 * upper.min(1.0 - upper)).min(1.0)
            }
            "Kolmogorov-Smirnov" => ks_pvalue(stat.value, stat.n.expect("KS n")),
            "Anderson-Darling" => 1.0 - anderson_darling_cdf(stat.n.expect("AD n"), stat.value),
            _ => continue,
        };
        assert!(
            (expected - r.p_value).abs() <= 1e-9 * r.p_value.max(1e-300) + 1e-12,
            "{}: p = {}, but {} {} gives {}",
            r.name,
            r.p_value,
            stat.name,
            stat.value,
            expected
        );
        checked += 1;
    }
    checked
}

#[test]
fn nist_statistics_reproduce_their_p_values() {
    let results = nist::run_all(&mut Mt19937::new(5489), 1_000_000);
    assert!(check(&results) > 150);
}

#[test]
fn diehard_statistics_reproduce_their_p_values() {
    let results = diehard::run_all(&mut Mt19937::new(5489), 16_000_000, true);
    assert!(check(&results) >= 15, "{}", check(&results));
}

#[test]
fn historical_statistics_reproduce_their_p_values() {
    let results = diehard::historical::run_all(&mut Mt19937::new(5489), 16_000_000);
    assert!(check(&results) >= 50, "{}", check(&results));
}

#[test]
fn dieharder_statistics_reproduce_their_p_values() {
    let results = dieharder::run_all(&mut Mt19937::new(5489), 16_000_000, true);
    assert!(check(&results) >= 20, "{}", check(&results));
}

#[test]
fn research_statistics_reproduce_their_p_values() {
    let mut rng = Mt19937::new(5489);
    let floats = rng.collect_f64s(200_000);
    let mut results = vec![
        knuth::permutation_test(&floats, 5),
        knuth::gap_test(&floats, 0.25, 0.5, 15),
        knuth::runs_above_below_median_test(&floats),
    ];
    let hc = testu01_hamming::hamming_corr(&mut rng, 20_000, 20, 10, 300);
    results.push(testu01_hamming::hamming_corr_result(&hc));
    let hi = testu01_hamming::hamming_indep(&mut rng, 20_000, 20, 10, 300, 1);
    results.push(testu01_hamming::hamming_indep_main_result(&hi));
    results.push(testu01_hamming::hamming_indep_block_result(&hi, 1));
    let fpf = practrand_fpf::fpf_test(&mut rng, 1 << 22, &practrand_fpf::FpfConfig::default());
    results.push(practrand_fpf::fpf_cross_result(&fpf));
    for platter in &fpf.platter_results {
        results.push(practrand_fpf::fpf_platter_result(platter, &fpf));
    }
    let (_, lz) = testu01_lz::lempel_ziv_summary(&mut rng, 50, 10, 0, 32, 1);
    results.push(testu01_lz::lempel_ziv_sum_result(&lz));
    results.push(testu01_lz::lempel_ziv_ks_result(&lz));
    assert!(check(&results) >= 9, "{}", check(&results));
}
