//! Runs the classical research probes (the TAOCP §3.3.2 permutation and gap
//! tests, the Wald–Wolfowitz runs test above/below the median, and the
//! NIST-style ApEn profile) across the seeded RNG family.
//! See `tests/run_aux.sh` for the batch harness.

use entropy::research::{
    approx_entropy::approx_entropy_profile,
    knuth::{gap_test, permutation_test, runs_above_below_median_test},
};
use entropy::rng::Rng;

#[path = "common/cli.rs"]
mod cli;
#[path = "common/family.rs"]
mod family;
#[path = "common/outcome.rs"]
mod outcome;

struct Args {
    float_samples: usize,
    bit_samples: usize,
    rng: cli::RngFilter,
}

impl Args {
    fn parse_from(mut argv: cli::Argv) -> Result<Self, cli::Stop> {
        let mut float_samples = 200_000usize;
        let mut bit_samples = 1_000_000usize;
        let mut rng = cli::RngFilter::default();
        while let Some(option) = argv.next_option()? {
            match option.as_str() {
                flag @ "--float-samples" => float_samples = argv.usize_value(flag)?,
                flag @ "--bit-samples" => bit_samples = argv.usize_value(flag)?,
                flag @ "--rng" => rng.push(argv.value(flag)?),
                other => return Err(cli::unknown_option(other)),
            }
        }
        Ok(Self {
            float_samples,
            bit_samples,
            rng,
        })
    }
}

fn print_usage() {
    eprintln!(
        "Usage: bib_tests [--rng <label>] [--float-samples N] [--bit-samples N]\n\
         \n\
         Runs BIB-backed research tests: Knuth permutation/gap, the\n\
         Wald-Wolfowitz runs test above/below the median, and the\n\
         NIST SP 800-22 §2.12 ApEn statistic swept over m=2..6.\n\
         \n\
         Example:\n\
           cargo run --release --bin bib_tests -- --rng AES"
    );
}

fn collect_case(
    mut rng: impl Rng,
    float_samples: usize,
    bit_samples: usize,
) -> (Vec<f64>, Vec<u8>) {
    let floats = rng.collect_f64s(float_samples);
    let bits = rng.collect_bits(bit_samples);
    (floats, bits)
}

fn print_case(label: &str, floats: &[f64], bits: &[u8], outcome: &mut outcome::Outcome) {
    let permutation = permutation_test(floats, 5);
    let gap = gap_test(floats, 0.25, 0.5, 15);
    let runs = runs_above_below_median_test(floats);
    let apen = approx_entropy_profile(bits, &[2, 3, 4, 5, 6]);

    println!("{label}");
    for result in [&permutation, &gap, &runs] {
        println!("  {result}");
        outcome.record(label, result);
    }
    for point in apen {
        println!(
            "  [INFO] approx_entropy_m{:02}   ApEn={:.6} (phi_m={:.6}, phi_m1={:.6})",
            point.m, point.ap_en, point.phi_m, point.phi_m1
        );
    }
    println!();
}

/// Collects each selected generator's samples and prints its probes.
struct Runner<'a>(&'a Args, outcome::Outcome);

impl family::Visit for Runner<'_> {
    fn case<R: Rng>(&mut self, label: &'static str, make: impl FnOnce() -> R) {
        let (floats, bits) = collect_case(make(), self.0.float_samples, self.0.bit_samples);
        print_case(label, &floats, &bits, &mut self.1);
    }
}

fn main() {
    let args = cli::parse_or_exit(Args::parse_from, print_usage);
    let mut runner = Runner(&args, outcome::Outcome::default());
    if family::visit_matching(&args.rng, &mut runner) == 0 {
        cli::die_no_rng_matched();
    }
    runner.1.exit_if_incomplete();
}
