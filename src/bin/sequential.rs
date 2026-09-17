//! The anytime-valid Markov-mixture test of fair bits run across the seeded
//! RNG family; see `entropy::research::sequential`.

use entropy::research::sequential::markov_mixture;
use entropy::rng::Rng;

#[path = "common/cli.rs"]
mod cli;
#[path = "common/family.rs"]
mod family;

struct Args {
    words: usize,
    max_order: usize,
    rng: cli::RngFilter,
}

impl Args {
    fn parse_from(mut argv: cli::Argv) -> Result<Self, cli::Stop> {
        let mut words = 1_000_000usize;
        let mut max_order = 16usize;
        let mut rng = cli::RngFilter::default();
        while let Some(option) = argv.next_option()? {
            match option.as_str() {
                flag @ "--words" => words = argv.usize_value(flag)?,
                flag @ "--max-order" => max_order = argv.usize_value(flag)?,
                flag @ "--rng" => rng.push(argv.value(flag)?),
                other => return Err(cli::unknown_option(other)),
            }
        }
        if words == 0 {
            return Err(cli::usage("--words must be positive"));
        }
        if max_order > 24 {
            return Err(cli::usage("--max-order must be at most 24"));
        }
        Ok(Self {
            words,
            max_order,
            rng,
        })
    }
}

fn print_usage() {
    eprintln!(
        "Usage: sequential [--rng <label>] [--words N] [--max-order K]\n\
         \n\
         Bets on each bit with Markov models of orders 0..K and reports\n\
         p = min(1, 1/sup E), valid however long the bits are watched\n\
         (default 1 000 000 words, K = 16).\n\
         \n\
         Example:\n\
           cargo run --release --bin sequential -- --rng MT19937"
    );
}

struct Runner<'a>(&'a Args);

impl family::Visit for Runner<'_> {
    fn case<R: Rng>(&mut self, label: &'static str, make: impl FnOnce() -> R) {
        let result = markov_mixture(&mut make(), self.0.words, self.0.max_order);
        println!("{label}\n  {result}\n");
    }
}

fn main() {
    let args = cli::parse_or_exit(Args::parse_from, print_usage);
    if family::visit_matching(&args.rng, &mut Runner(&args)) == 0 {
        cli::die_no_rng_matched();
    }
}
