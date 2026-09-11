//! Upstream-suite research probes: PractRand FPF (platter + cross) and
//! TestU01 `sstring_HammingCorr` / `sstring_HammingIndep`, run across the
//! seeded RNG family.
//!
//! # References
//! * P. L'Ecuyer and R. Simard, "TestU01: A C Library for Empirical Testing
//!   of Random Number Generators," *ACM Transactions on Mathematical
//!   Software* 33(4), Article 22, 2007, §5.2.1,
//!   pp. 19–20 (`lecuyer2007testu01` in BIB.md).
//!   [pubs/lecuyer-simard-2007-testu01.pdf]
//! * C. Doty-Humphrey, "PractRand: Practically Random — A C++ Library of
//!   Statistical Tests for RNGs," 2018 (`practrand` in BIB.md; not in
//!   `pubs/`).

use entropy::research::{
    practrand_fpf::{fpf_cross_result, fpf_platter_result, fpf_test, FpfConfig},
    testu01_hamming::{
        hamming_corr, hamming_corr_result, hamming_indep, hamming_indep_block_result,
        hamming_indep_main_result, HAMMING_INDEP_MAX_L,
    },
};
use entropy::rng::Rng;

#[path = "common/cli.rs"]
mod cli;
#[path = "common/family.rs"]
mod family;

struct Args {
    rng: cli::RngFilter,
    hc_n: usize,
    hc_r: usize,
    hc_s: usize,
    hc_l: usize,
    hi_n: usize,
    hi_r: usize,
    hi_s: usize,
    hi_l: usize,
    hi_d: usize,
    fpf_bits: usize,
}

impl Args {
    fn parse_from(mut argv: cli::Argv) -> Result<Self, cli::Stop> {
        let mut out = Self {
            rng: cli::RngFilter::default(),
            hc_n: 500_000,
            hc_r: 20,
            hc_s: 10,
            hc_l: 300,
            hi_n: 500_000,
            hi_r: 20,
            hi_s: 10,
            hi_l: 300,
            hi_d: 1,
            fpf_bits: 1 << 27,
        };
        while let Some(option) = argv.next_option()? {
            match option.as_str() {
                flag @ "--rng" => out.rng.push(argv.value(flag)?),
                flag @ "--hc-n" => out.hc_n = parse_usize(&mut argv, flag)?,
                flag @ "--hc-r" => out.hc_r = parse_usize(&mut argv, flag)?,
                flag @ "--hc-s" => out.hc_s = parse_usize(&mut argv, flag)?,
                flag @ "--hc-l" => out.hc_l = parse_usize(&mut argv, flag)?,
                flag @ "--hi-n" => out.hi_n = parse_usize(&mut argv, flag)?,
                flag @ "--hi-r" => out.hi_r = parse_usize(&mut argv, flag)?,
                flag @ "--hi-s" => out.hi_s = parse_usize(&mut argv, flag)?,
                flag @ "--hi-l" => out.hi_l = parse_usize(&mut argv, flag)?,
                flag @ "--hi-d" => out.hi_d = parse_usize(&mut argv, flag)?,
                flag @ "--fpf-bits" => out.fpf_bits = parse_usize(&mut argv, flag)?,
                other => return Err(cli::unknown_option(other)),
            }
        }

        // Range checks mirror the asserts in research::testu01_hamming and
        // research::practrand_fpf so a bad flag dies with the flag's name
        // (exit 1) instead of a library panic (exit 101).
        if out.hc_n < 2 {
            return Err(cli::usage("--hc-n must be at least 2"));
        }
        if !(1..=32).contains(&out.hc_s) {
            return Err(cli::usage("--hc-s must be in 1..=32"));
        }
        if out.hc_r > 32 || out.hc_r + out.hc_s > 32 {
            return Err(cli::usage("--hc-r plus --hc-s must be <= 32"));
        }
        if out.hc_l == 0 {
            return Err(cli::usage("--hc-l must be positive"));
        }
        if out.hi_n < 20 {
            return Err(cli::usage("--hi-n must be at least 20"));
        }
        if !(1..=32).contains(&out.hi_s) {
            return Err(cli::usage("--hi-s must be in 1..=32"));
        }
        if out.hi_r > 32 || out.hi_r + out.hi_s > 32 {
            return Err(cli::usage("--hi-r plus --hi-s must be <= 32"));
        }
        if !(1..=HAMMING_INDEP_MAX_L).contains(&out.hi_l) {
            return Err(cli::usage(format!(
                "--hi-l must be in 1..={HAMMING_INDEP_MAX_L}"
            )));
        }
        if !(1..=8).contains(&out.hi_d) {
            return Err(cli::usage("--hi-d must be in 1..=8"));
        }
        if out.hi_d > out.hi_l.div_ceil(2) {
            return Err(cli::usage("--hi-d must be <= (--hi-l + 1) / 2"));
        }
        let fpf = FpfConfig::default();
        let worst_codeword = (1usize << fpf.exp_bits) - 1 + fpf.sig_bits;
        if out.fpf_bits < worst_codeword {
            return Err(cli::usage(format!(
                "--fpf-bits must be at least {worst_codeword} (one worst-case codeword)"
            )));
        }
        Ok(out)
    }
}

/// The argument after `flag` as a `usize`; this binary reports a value that
/// does not parse as `invalid value for <flag>`.
fn parse_usize(argv: &mut cli::Argv, flag: &str) -> Result<usize, cli::Stop> {
    argv.value(flag)?
        .parse()
        .map_err(|_| cli::usage(format!("invalid value for {flag}")))
}

fn print_usage() {
    eprintln!(
        "Usage: upstream_tests [--rng <label>]\n\
                      [--hc-n N] [--hc-r N] [--hc-s N] [--hc-l N]\n\
                      [--hi-n N] [--hi-r N] [--hi-s N] [--hi-l N] [--hi-d N]\n\
                      [--fpf-bits N]\n\
         \n\
         Runs one honest TestU01 bit-string slice and one honest PractRand slice:\n\
         - TestU01 sstring_HammingCorr  (--hc-n blocks, r/s bit window, L bits)\n\
         - TestU01 sstring_HammingIndep (--hi-n pairs,  r/s bit window, L bits, d)\n\
         - PractRand FPF(4,14,6) core   (--fpf-bits total bits), parsing disjoint\n\
           codewords rather than upstream's stride-overlapped windows\n\
         \n\
         Defaults (moderate-size runs suitable for development checks):\n\
           hc-n=500000 hc-r=20 hc-s=10 hc-l=300\n\
           hi-n=500000 hi-r=20 hi-s=10 hi-l=300 hi-d=1  fpf-bits=2^27\n\
         Constraints: hc-n >= 2, hi-n >= 20, s in 1..=32, r+s <= 32,\n\
           hc-l >= 1, hi-l in 1..=4096, d in 1..=8, d <= (hi-l+1)/2,\n\
           fpf-bits >= 77 (one worst-case codeword).\n\
         Example:\n\
           cargo run --release --bin upstream_tests -- --rng AES"
    );
}

fn run_case(label: &str, mut rng: impl Rng, args: &Args) {
    println!("{label}");

    let hc = hamming_corr(&mut rng, args.hc_n, args.hc_r, args.hc_s, args.hc_l);
    println!("  {}", hamming_corr_result(&hc));

    let hi = hamming_indep(
        &mut rng, args.hi_n, args.hi_r, args.hi_s, args.hi_l, args.hi_d,
    );
    println!("  {}", hamming_indep_main_result(&hi));
    for k in 1..=args.hi_d {
        println!("  {}", hamming_indep_block_result(&hi, k));
    }

    let fpf = fpf_test(&mut rng, args.fpf_bits, &FpfConfig::default());
    println!("  {}", fpf_cross_result(&fpf));
    for platter in fpf.platter_results.iter().take(8) {
        println!("  {}", fpf_platter_result(platter, &fpf));
    }
    if fpf.platter_results.len() > 8 {
        println!(
            "  [INFO] practrand::fpf_more                   {} additional platter results omitted",
            fpf.platter_results.len() - 8
        );
    }
    println!();
}

/// Runs the upstream probes on each selected generator.
struct Runner<'a>(&'a Args);

impl family::Visit for Runner<'_> {
    fn case<R: Rng>(&mut self, label: &'static str, make: impl FnOnce() -> R) {
        run_case(label, make(), self.0);
    }
}

fn main() {
    let args = cli::parse_or_exit(Args::parse_from, print_usage);
    if family::visit_matching(&args.rng, &mut Runner(&args)) == 0 {
        cli::die_no_rng_matched();
    }
}
