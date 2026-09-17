//! Throughput of the sampling methods, beside raw generation.
//!
//! `variate_throughput [millions] [rounds]` draws `millions`·10⁶ values by
//! each method from one PCG64, after an unmeasured warm-up round, and prints
//! the median rate of `rounds` measured rounds in millions of draws per
//! second.  Wall-clock time on a busy machine understates every rate; the
//! comparison between methods is the point, so they run in one process,
//! alternating, on the same generator family.

use entropy::rng::{Pcg64, Rng, Sample, Seedable};
use std::{hint::black_box, time::Instant};

/// Draws per round, in millions, unless the command line says otherwise.
const DEFAULT_MILLIONS: usize = 5;

/// Measured rounds unless the command line says otherwise.
const DEFAULT_ROUNDS: usize = 7;

/// Seed for every method, so each sees the same stream.
const SEED: u64 = 0x7661_7269_6174_6573; // "variates"

/// Elements of the slice the shuffle and choice methods work on.
const SLICE: usize = 1_000;

/// A million.
const MILLION: f64 = 1e6;

/// The median of `rates`, which it sorts.
fn median(rates: &mut [f64]) -> f64 {
    rates.sort_by(f64::total_cmp);
    rates[rates.len() / 2]
}

/// Millions of draws per second of `draw`, over `rounds` measured rounds.
fn report(name: &str, draws: usize, rounds: usize, mut draw: impl FnMut(&mut Pcg64) -> f64) {
    let mut rng = Pcg64::seed_from_u64(SEED);
    for _ in 0..draws {
        black_box(draw(&mut rng));
    }
    let mut rates: Vec<f64> = (0..rounds)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..draws {
                black_box(draw(&mut rng));
            }
            draws as f64 / MILLION / start.elapsed().as_secs_f64()
        })
        .collect();
    println!("{name} {:.1}", median(&mut rates));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let parse = |i: usize, default: usize| {
        args.get(i)
            .map_or(default, |v| v.parse().expect("a whole number"))
    };
    let draws = parse(1, DEFAULT_MILLIONS) * MILLION as usize;
    let rounds = parse(2, DEFAULT_ROUNDS);
    println!("# {draws} draws per round, median of {rounds} rounds, millions per second");
    println!("method rate");
    report("next_u64", draws, rounds, |r| r.next_u64() as f64);
    report("unit_f64", draws, rounds, Sample::unit_f64);
    report("unit_f64_dense", draws, rounds, Sample::unit_f64_dense);
    report("range_1_to_7", draws, rounds, |r| r.range(1, 7) as f64);
    report("below_1000", draws, rounds, |r| {
        r.below(SLICE as u64) as f64
    });
    report("bernoulli_0.3", draws, rounds, |r| {
        f64::from(u8::from(r.bernoulli(0.3)))
    });
    report("exponential", draws, rounds, Sample::exponential);
    report("normal_inverse", draws, rounds, Sample::normal_inverse);
    report("normal_ziggurat", draws, rounds, Sample::normal);

    // The sequence methods, per operation rather than per draw.
    let mut items: Vec<u32> = (0..SLICE as u32).collect();
    report("choose_of_1000", draws, rounds, |r| {
        f64::from(*r.choose(&items).expect("nonempty"))
    });
    let shuffles = draws / SLICE;
    let mut rng = Pcg64::seed_from_u64(SEED);
    rng.shuffle(&mut items);
    let mut rates: Vec<f64> = (0..rounds)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..shuffles {
                rng.shuffle(&mut items);
            }
            (shuffles * SLICE) as f64 / MILLION / start.elapsed().as_secs_f64()
        })
        .collect();
    println!("shuffle_1000 {:.1}", median(&mut rates));
}
