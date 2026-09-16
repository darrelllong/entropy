//! The Lempel–Ziv compressibility test run across the seeded RNG family, with
//! configurable `k`, bit-extraction window, and replications.
//!
//! # References
//! * P. L'Ecuyer and R. Simard, "TestU01: A C Library for Empirical Testing
//!   of Random Number Generators," *ACM Transactions on Mathematical
//!   Software* 33(4), Article 22, 2007, §5.1,
//!   p. 17 (`lecuyer2007testu01` in BIB.md).
//!   [pubs/lecuyer-simard-2007-testu01.pdf]
//! * J. Ziv and A. Lempel, "Compression of individual sequences via
//!   variable-rate coding," *IEEE Transactions on Information Theory* 24(5),
//!   pp. 530–536, 1978.  [Cited from the 2007 paper's reference list.]

use entropy::research::testu01_lz::{
    lempel_ziv_ks_result, lempel_ziv_sum_result, lempel_ziv_summary,
};
use entropy::rng::Rng;

#[path = "common/cli.rs"]
mod cli;
#[path = "common/family.rs"]
mod family;

struct Args {
    replications: usize,
    k: usize,
    r: usize,
    s: usize,
    pit_seed: u64,
    rng: cli::RngFilter,
}

impl Args {
    fn parse_from(mut argv: cli::Argv) -> Result<Self, cli::Stop> {
        let mut replications = 10usize;
        let mut k = 25usize;
        let mut r = 0usize;
        let mut s = 30usize;
        let mut pit_seed = 1u64;
        let mut rng = cli::RngFilter::default();
        while let Some(option) = argv.next_option()? {
            match option.as_str() {
                flag @ "--replications" => replications = argv.usize_value(flag)?,
                flag @ "--k" => k = argv.usize_value(flag)?,
                flag @ "--r" => r = argv.usize_value(flag)?,
                flag @ "--s" => s = argv.usize_value(flag)?,
                flag @ "--pit-seed" => pit_seed = argv.usize_value(flag)? as u64,
                flag @ "--rng" => rng.push(argv.value(flag)?),
                other => return Err(cli::unknown_option(other)),
            }
        }

        // Range checks mirror the asserts in research::testu01_lz so a bad
        // flag dies with the flag's name instead of a library panic.
        if !(3..=28).contains(&k) {
            return Err(cli::usage("--k must be in 3..=28"));
        }
        if !(1..=32).contains(&s) {
            return Err(cli::usage("--s must be in 1..=32"));
        }
        if r > 32 || r + s > 32 {
            return Err(cli::usage("--r plus --s must be <= 32"));
        }
        if replications == 0 {
            return Err(cli::usage("--replications must be positive"));
        }

        Ok(Self {
            replications,
            k,
            r,
            s,
            pit_seed,
            rng,
        })
    }
}

fn print_usage() {
    eprintln!(
        "Usage: testu01_lz [--rng <label>] [--replications N] [--k K] [--r R] [--s S]\n\
                  [--pit-seed SEED]\n\
         \n\
         Runs the Lempel-Ziv phrase-count test against the phrase-count\n\
         tables in entropy::research.  SEED seeds the separate generator of\n\
         the randomized probability-integral transform.  Simulated tables\n\
         support at most a hundredth of their replications.\n\
         \n\
         Example:\n\
           cargo run --release --bin testu01_lz -- --rng AES\n\
           cargo run --release --bin testu01_lz -- --rng MT19937 --k 27"
    );
}

fn run_case(label: &str, mut rng: impl Rng, args: &Args) {
    let (reps, summary) = lempel_ziv_summary(
        &mut rng,
        args.replications,
        args.k,
        args.r,
        args.s,
        args.pit_seed,
    );
    println!("{label}");
    println!("  {}", lempel_ziv_sum_result(&summary));
    println!("  {}", lempel_ziv_ks_result(&summary));
    for (i, rep) in reps.iter().enumerate() {
        println!(
            "  [INFO] testu01::lzw_rep{:02}                    W={} U={:.6} z={:.4}",
            i + 1,
            rep.phrase_count,
            rep.uniform,
            rep.z_score
        );
    }
    println!();
}

/// Runs the Lempel-Ziv replications on each selected generator.
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
