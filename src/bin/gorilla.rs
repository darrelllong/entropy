//! Marsaglia–Tsang Gorilla test (JSS 7(3), 2002) over all 32 bit positions of
//! each seeded generator, with a per-bit summary and the paper's
//! Anderson–Darling ("ADKS") aggregate: the statistic A₃₂ (`agg_ad_A`) and
//! the p-value 1 − Pr(A₃₂ < A) (`agg_ad_p`), which is small when the 32
//! per-bit p-values are far from uniform.  The paper prints Pr(A₃₂ < A)
//! itself.  See `entropy::research::marsaglia_tsang`.

use entropy::research::marsaglia_tsang::{gorilla_aggregate_ad, gorilla_all, GorillaBitResult};
use entropy::rng::Rng;

#[path = "common/cli.rs"]
mod cli;
#[path = "common/family.rs"]
mod family;

use family::Visit;

const GORILLA_STREAM_WORDS: usize = (1 << 26) + 25;

struct Args {
    rng: cli::RngFilter,
}

impl Args {
    fn parse_from(mut argv: cli::Argv) -> Result<Self, cli::Stop> {
        let mut rng = cli::RngFilter::default();
        while let Some(option) = argv.next_option()? {
            match option.as_str() {
                flag @ "--rng" => rng.push(argv.value(flag)?),
                other => return Err(cli::unknown_option(other)),
            }
        }
        Ok(Self { rng })
    }
}

fn print_usage() {
    eprintln!(
        "Usage: gorilla [--rng <label>]\n\
         \n\
         Runs the Marsaglia–Tsang Gorilla test over all 32 bit positions of a\n\
         seeded deterministic RNG family.\n\
         \n\
         Example:\n\
           cargo run --release --bin gorilla -- --rng AES"
    );
}

fn with_rng(mut rng: impl Rng) -> Vec<GorillaBitResult> {
    let words = rng.collect_u32s(GORILLA_STREAM_WORDS);
    gorilla_all(&words)
}

fn summarize(results: &[GorillaBitResult]) -> (f64, f64, usize, f64) {
    let mut min_p = 1.0f64;
    let mut max_p = 0.0f64;
    let mut worst_bit = 0usize;
    let mut worst_abs_z = 0.0f64;
    for result in results {
        if result.p_value < min_p {
            min_p = result.p_value;
        }
        if result.p_value > max_p {
            max_p = result.p_value;
        }
        let abs_z = result.z_score.abs();
        if abs_z > worst_abs_z {
            worst_abs_z = abs_z;
            worst_bit = result.bit_position;
        }
    }
    (min_p, max_p, worst_bit, worst_abs_z)
}

/// Counts the generators `--rng` selects without constructing them.
struct Skip;

impl Visit for Skip {
    fn case<R: Rng>(&mut self, _label: &'static str, _make: impl FnOnce() -> R) {}
}

/// Runs the Gorilla test on a generator and prints its row.
struct Row;

impl Visit for Row {
    fn case<R: Rng>(&mut self, label: &'static str, make: impl FnOnce() -> R) {
        let results = with_rng(make());
        let (min_p, max_p, worst_bit, worst_abs_z) = summarize(&results);
        let aggregate = gorilla_aggregate_ad(&results);
        println!(
            "{:<40} {:>9.6} {:>9.6} {:>9} {:>10.3} {:>10.4} {:>10.6}",
            label, min_p, max_p, worst_bit, worst_abs_z, aggregate.statistic, aggregate.p_value
        );
    }
}

fn main() {
    let args = cli::parse_or_exit(Args::parse_from, print_usage);
    // Each case generates ~268 MB of stream and a full 32-bit-position
    // Gorilla pass, so the --rng filter is checked before any work is done
    // or the header printed.
    if family::visit_matching(&args.rng, &mut Skip) == 0 {
        cli::die_no_rng_matched();
    }

    println!(
        "{:<40} {:>9} {:>9} {:>9} {:>10} {:>10} {:>10}",
        "RNG", "min_p", "max_p", "worst_bit", "worst_|z|", "agg_ad_A", "agg_ad_p"
    );
    println!("{}", "-".repeat(106));

    family::visit_matching(&args.rng, &mut Row);
}
