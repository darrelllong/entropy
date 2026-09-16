//! Estimates the distribution of the number of steps of Euclid's algorithm on
//! pairs of independent uniform nonzero 32-bit integers, for the DIEHARDER
//! GCD test.
//!
//! The exact law over all 2⁶⁴ pairs is out of reach, so it is estimated by
//! simulation.  Each thread draws pairs (u, v) from its own generator stream,
//! discards pairs with a zero, and counts k, the steps of
//! `while v != 0 { (u, v) = (v, u mod v) }`.  The output is one line per k of
//! raw counts, so runs on several machines and generators can be added.
//!
//! Usage: `gcd_step_table <pcg64|xoshiro256> <pairs per thread> <threads> <first stream>`
//!
//! The generator names select [`Pcg64`] with stream = thread index plus
//! `first stream`, or [`Xoshiro256`] seeded from that index through
//! SplitMix64.  The two are unrelated designs, so agreement between their
//! tables checks the estimate against generator artefacts.

use entropy::{
    rng::{Pcg64, Rng, Xoshiro256},
    seed::splitmix64,
};
use std::thread;

/// Step counts above this are counted in the last cell; 2³² < F₄₉ bounds
/// every k by 47.
const MAX_STEPS: usize = 64;

fn steps(mut u: u32, mut v: u32) -> usize {
    let mut k = 0;
    while v != 0 {
        (u, v) = (v, u % v);
        k += 1;
    }
    k
}

fn count(mut rng: impl Rng, pairs: u64) -> ([u64; MAX_STEPS + 1], u64) {
    let mut counts = [0u64; MAX_STEPS + 1];
    let mut discarded = 0;
    for _ in 0..pairs {
        let (u, v) = (rng.next_u32(), rng.next_u32());
        if u == 0 || v == 0 {
            discarded += 1;
            continue;
        }
        counts[steps(u, v).min(MAX_STEPS)] += 1;
    }
    (counts, discarded)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage =
        "usage: gcd_step_table <pcg64|xoshiro256> <pairs per thread> <threads> <first stream>";
    let [_, generator, pairs, threads, first] = args.as_slice() else {
        panic!("{usage}");
    };
    let pairs: u64 = pairs.parse().expect(usage);
    let threads: u64 = threads.parse().expect(usage);
    let first: u64 = first.parse().expect(usage);
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let generator = generator.clone();
            let stream = first + t;
            thread::spawn(move || match generator.as_str() {
                "pcg64" => count(Pcg64::new(0x6763_645f_7374_6570, u128::from(stream)), pairs),
                "xoshiro256" => {
                    let mut state = stream ^ 0x7873_6868_3235_3621;
                    let seed: [u64; 4] = std::array::from_fn(|_| splitmix64(&mut state));
                    count(Xoshiro256::new(seed[0], seed[1], seed[2], seed[3]), pairs)
                }
                other => panic!("unknown generator {other}"),
            })
        })
        .collect();
    let mut total = [0u64; MAX_STEPS + 1];
    let mut discarded = 0;
    for handle in handles {
        let (counts, d) = handle.join().expect("worker panicked");
        for (t, c) in total.iter_mut().zip(counts) {
            *t += c;
        }
        discarded += d;
    }
    println!("# generator={generator} pairs_per_thread={pairs} threads={threads} first_stream={first} discarded={discarded}");
    for (k, c) in total.iter().enumerate() {
        println!("{k} {c}");
    }
}
