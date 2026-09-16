//! The distribution of the LZ78 phrase count W of 2^k fair random bits, the
//! tables in `src/research/lz78_counts.rs`.
//!
//! `exact <k>` enumerates all 2^(2^k) strings (k ≤ 5) and prints the number of
//! strings with each W.  `simulate <k> <replications per thread> <threads>
//! <first stream>` counts W over replications on PCG64 streams.  Output is one
//! line: `k=… total=… w_min=… counts=c,c,…`.

use entropy::{
    research::testu01_lz::phrase_count,
    rng::{Pcg64, Rng},
};
use std::{collections::BTreeMap, thread};

/// Emits one fixed word.
struct Word(u32);

impl Rng for Word {
    fn next_u32(&mut self) -> u32 {
        self.0
    }
}

fn print(k: usize, hist: &BTreeMap<usize, u64>) {
    let total: u64 = hist.values().sum();
    let (&lo, _) = hist.first_key_value().expect("counts");
    let (&hi, _) = hist.last_key_value().expect("counts");
    let counts: Vec<String> = (lo..=hi)
        .map(|w| hist.get(&w).copied().unwrap_or(0).to_string())
        .collect();
    println!("k={k} total={total} w_min={lo} counts={}", counts.join(","));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k: usize = args[2].parse().expect("k");
    match args[1].as_str() {
        "exact" => {
            assert!((3..=5).contains(&k), "exact enumeration covers k = 3 … 5");
            let bits = 1usize << k;
            let threads: u64 = 128;
            let span = 1u64 << bits;
            let handles: Vec<_> = (0..threads)
                .map(|t| {
                    thread::spawn(move || {
                        let mut hist = BTreeMap::new();
                        let lo = span * t / threads;
                        let hi = span * (t + 1) / threads;
                        for x in lo..hi {
                            let word = (x << (32 - bits)) as u32;
                            let (w, _) = phrase_count(&mut Word(word), k, 0, bits);
                            *hist.entry(w).or_insert(0u64) += 1;
                        }
                        hist
                    })
                })
                .collect();
            let mut hist = BTreeMap::new();
            for h in handles {
                for (w, c) in h.join().expect("worker") {
                    *hist.entry(w).or_insert(0) += c;
                }
            }
            print(k, &hist);
        }
        "simulate" => {
            let reps: u64 = args[3].parse().expect("replications");
            let threads: u64 = args[4].parse().expect("threads");
            let first: u64 = args[5].parse().expect("first stream");
            let handles: Vec<_> = (0..threads)
                .map(|t| {
                    thread::spawn(move || {
                        let mut rng = Pcg64::new(0x6c7a_3738_7461_626c, u128::from(first + t));
                        let mut hist = BTreeMap::new();
                        for _ in 0..reps {
                            let (w, _) = phrase_count(&mut rng, k, 0, 32);
                            *hist.entry(w).or_insert(0u64) += 1;
                        }
                        hist
                    })
                })
                .collect();
            let mut hist = BTreeMap::new();
            for h in handles {
                for (w, c) in h.join().expect("worker") {
                    *hist.entry(w).or_insert(0) += c;
                }
            }
            print(k, &hist);
        }
        other => panic!("unknown mode {other}"),
    }
}
