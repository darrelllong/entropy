//! Byte-filling throughput: `Rng::fill_native` against `Sample::fill_bytes`
//! and a scalar `next_u64` loop.
//!
//! `fill_throughput [mib] [rounds]` fills a buffer of `mib` mebibytes
//! `rounds` times by each path, after one unmeasured warm-up round, and
//! prints the median rate of each.  Wall-clock time on an otherwise busy
//! machine understates every rate; the comparison between paths is the point,
//! so all three run in the same process, on the same buffer, in alternating
//! order.

use entropy::rng::{Jsf64, Pcg64, Sample, Seedable, Xoshiro256};
#[cfg(feature = "cryptography")]
use entropy::rng::{ChaCha20Rng, FastKeyErasureRng};
use std::time::Instant;

/// Mebibytes filled per round unless the command line says otherwise.
const DEFAULT_MIB: usize = 64;

/// Measured rounds unless the command line says otherwise.
const DEFAULT_ROUNDS: usize = 7;

/// Seed for the deterministic generators.
const SEED: u64 = 0x6669_6c6c_5f62_656e; // "fill_ben"

/// Bytes per mebibyte.
const MIB: usize = 1 << 20;

/// The median of `rates`, which it sorts.
fn median(rates: &mut [f64]) -> f64 {
    rates.sort_by(f64::total_cmp);
    rates[rates.len() / 2]
}

/// Mebibytes per second of `fill` over `rounds` measured rounds.
fn rate(buffer: &mut [u8], rounds: usize, mut fill: impl FnMut(&mut [u8])) -> f64 {
    fill(buffer); // warm-up, unmeasured
    let mut rates: Vec<f64> = (0..rounds)
        .map(|_| {
            let start = Instant::now();
            fill(buffer);
            buffer.len() as f64 / MIB as f64 / start.elapsed().as_secs_f64()
        })
        .collect();
    median(&mut rates)
}

/// One generator's three paths, as `name native fill_bytes scalar`.
fn report<R: Sample>(name: &str, buffer: &mut [u8], rounds: usize, new: impl Fn() -> R) {
    let mut native = new();
    let native_rate = rate(buffer, rounds, |b| native.fill_native(b));
    let mut projected = new();
    let projected_rate = rate(buffer, rounds, |b| projected.fill_bytes(b));
    let mut scalar = new();
    let scalar_rate = rate(buffer, rounds, |b| {
        for chunk in b.chunks_mut(size_of::<u64>()) {
            let word = scalar.next_u64().to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }
    });
    println!("{name} {native_rate:.1} {projected_rate:.1} {scalar_rate:.1}");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let parse = |i: usize, default: usize| {
        args.get(i)
            .map_or(default, |v| v.parse().expect("a whole number"))
    };
    let mib = parse(1, DEFAULT_MIB);
    let rounds = parse(2, DEFAULT_ROUNDS);
    let mut buffer = vec![0u8; mib * MIB];
    println!("# {mib} MiB per round, median of {rounds} rounds, MiB/s");
    println!("generator fill_native fill_bytes next_u64_loop");
    report("Xoshiro256", &mut buffer, rounds, || {
        Xoshiro256::seed_from_u64(SEED)
    });
    report("Jsf64", &mut buffer, rounds, || {
        Jsf64::seed_from_u64(SEED)
    });
    report("Pcg64", &mut buffer, rounds, || {
        Pcg64::seed_from_u64(SEED)
    });
    #[cfg(feature = "cryptography")]
    {
        report("ChaCha20Rng", &mut buffer, rounds, || {
            ChaCha20Rng::new(&[7u8; 32], &[0u8; 12], 0)
        });
        report("FastKeyErasureRng", &mut buffer, rounds, || {
            FastKeyErasureRng::new([7u8; 32])
        });
    }
}
