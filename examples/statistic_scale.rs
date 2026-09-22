//! The null distribution of each standardised statistic the battery
//! campaign found off its nominal law: measured, not assumed.
//!
//! `statistic_scale <test> <streams> <threads> [bits] [first]` draws `streams`
//! independently seeded null streams (PCG64, xoshiro256** and SFC64 in turn,
//! numbered from `first`, 0 unless given, so a validation run can take streams
//! a calibration never saw),
//! computes the test's standardised statistic on each, and prints its mean,
//! standard deviation and the fraction beyond the two-sided 0.05, 0.01 and
//! 0.001 points of the law the test assumes.  A statistic whose law is right
//! has mean 0, standard deviation 1 and those fractions.
//!
//! Tests and what is measured:
//!
//! * `universal`: the z of each Maurer setting L = 5 … that scores at `bits`
//!   (default 16 000 000): (f_n − μ)/σ with the standard's σ.
//! * `spectral`: the d of SP 800-22 §2.6 at `bits`: (N₁ − N₀)/√(n·0.95·0.05/4).
//! * `count_ones_bytes`: (Q5 − Q4 − 2 500)/√5 000 for every one of the 25
//!   byte windows, so each stream gives 25 values; χ²(2 500) standardised.
//! * `parking_lot`: the number of cars parked in one lot, standardised by
//!   Marsaglia's mean 3 523 and standard deviation 21.9; each stream is one
//!   lot, and the raw mean and standard deviation of the count are printed too.
//!
//! The rows are what a correction has to reproduce; `stats/` keeps the runs.

use entropy::{
    diehard::{
        historical::count_ones_bytes::{count_ones_bytes, WORDS as COUNT_ONES_WORDS},
        parking_lot::parked,
    },
    math::erfc,
    nist::{spectral::spectral, universal::universal_parametric_all},
    result::Status,
    rng::{Pcg64, Rng, Sfc64, Xoshiro256},
};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
};

/// Bits per stream for the NIST tests unless the command line says otherwise:
/// what the NIST suite reads in `run_tests`.
const DEFAULT_BITS: usize = 16_000_000;

/// Two-sided levels whose exceedance is counted.
const LEVELS: [f64; 3] = [0.05, 0.01, 0.001];

/// Marsaglia's mean and standard deviation of the parking-lot count.
const PARKING_MEAN: f64 = 3_523.0;
const PARKING_SIGMA: f64 = 21.9;

/// Degrees of freedom of Q5 − Q4, and so its mean and twice its variance.
const COUNT_ONES_DF: f64 = 2_500.0;

/// What one statistic accumulates: values, Σx, Σx², and exceedances by level.
#[derive(Clone, Default)]
struct Moments {
    count: u64,
    sum: f64,
    sum_squares: f64,
    beyond: [u64; LEVELS.len()],
}

impl Moments {
    /// Add a standardised value.  Its two-sided normal p-value decides the
    /// exceedances, which is the decision the tests make from it.
    fn add(&mut self, z: f64) {
        self.count += 1;
        self.sum += z;
        self.sum_squares += z * z;
        let p = erfc(z.abs() / std::f64::consts::SQRT_2);
        for (count, level) in self.beyond.iter_mut().zip(LEVELS) {
            *count += u64::from(p < level);
        }
    }

    fn merge(&mut self, other: &Self) {
        self.count += other.count;
        self.sum += other.sum;
        self.sum_squares += other.sum_squares;
        for (a, b) in self.beyond.iter_mut().zip(other.beyond) {
            *a += b;
        }
    }

    fn mean(&self) -> f64 {
        self.sum / self.count as f64
    }

    fn sd(&self) -> f64 {
        let n = self.count as f64;
        (self.sum_squares / n - self.mean() * self.mean()).sqrt()
    }
}

/// A null generator for stream `index`, one of three families in turn.
enum Null {
    Pcg(Pcg64),
    Xoshiro(Xoshiro256),
    Sfc(Sfc64),
}

impl Null {
    fn new(index: usize) -> Self {
        let seed = index as u128 + 1;
        match index % 3 {
            0 => Self::Pcg(Pcg64::new(seed, 0x7363_616c_655f_6e75)),
            1 => Self::Xoshiro(Xoshiro256::new(seed as u64 | 1, 2, 3, 4)),
            _ => Self::Sfc(Sfc64::new(seed as u64 | 1, 5, 6)),
        }
    }
}

impl Rng for Null {
    fn next_u32(&mut self) -> u32 {
        match self {
            Self::Pcg(r) => r.next_u32(),
            Self::Xoshiro(r) => r.next_u32(),
            Self::Sfc(r) => r.next_u32(),
        }
    }
    fn next_u64(&mut self) -> u64 {
        match self {
            Self::Pcg(r) => r.next_u64(),
            Self::Xoshiro(r) => r.next_u64(),
            Self::Sfc(r) => r.next_u64(),
        }
    }
}

/// The standardised values one stream yields for `test`, by row name, with
/// any raw value worth reporting beside them.
fn measure(test: &str, index: usize, bits: usize) -> Vec<(String, f64, Option<f64>)> {
    let mut rng = Null::new(index);
    match test {
        "universal" => universal_parametric_all(&rng.collect_bits(bits))
            .into_iter()
            .filter(|r| r.status == Status::Scored)
            .filter_map(|r| r.statistic.map(|s| (r.name.to_string(), s.value, None)))
            .collect(),
        "spectral" => {
            let r = spectral(&rng.collect_bits(bits));
            r.statistic
                .filter(|_| r.status == Status::Scored)
                .map(|s| vec![("nist::spectral".to_string(), s.value, None)])
                .unwrap_or_default()
        }
        "count_ones_bytes" => count_ones_bytes(&rng.collect_u32s(COUNT_ONES_WORDS))
            .into_iter()
            .zip(1..)
            .filter_map(|(r, window)| {
                r.statistic.map(|s| {
                    let z = (s.value - COUNT_ONES_DF) / (2.0 * COUNT_ONES_DF).sqrt();
                    (
                        format!("count_ones_bytes window {window:02}"),
                        z,
                        Some(s.value),
                    )
                })
            })
            .collect(),
        "parking_lot" => {
            let cars = parked(&mut rng) as f64;
            vec![(
                "diehard::parking_lot cars".to_string(),
                (cars - PARKING_MEAN) / PARKING_SIGMA,
                Some(cars),
            )]
        }
        other => panic!(
            "unknown test {other}; one of universal, spectral, count_ones_bytes, parking_lot"
        ),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let test = args.get(1).expect("test name").clone();
    let streams: usize = args.get(2).expect("streams").parse().expect("streams");
    let threads: usize = args.get(3).expect("threads").parse().expect("threads");
    let bits: usize = args
        .get(4)
        .map_or(DEFAULT_BITS, |v| v.parse().expect("bits"));
    let first: usize = args.get(5).map_or(0, |v| v.parse().expect("first index"));
    let next = Arc::new(AtomicUsize::new(0));
    // Row name -> (standardised moments, raw moments).
    let total: Arc<Mutex<BTreeMap<String, (Moments, Moments)>>> =
        Arc::new(Mutex::new(BTreeMap::new()));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let (next, total, test) = (next.clone(), total.clone(), test.clone());
            thread::spawn(move || {
                let mut mine: BTreeMap<String, (Moments, Moments)> = BTreeMap::new();
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    if index >= streams {
                        break;
                    }
                    for (name, z, raw) in measure(&test, first + index, bits) {
                        let entry = mine.entry(name).or_default();
                        entry.0.add(z);
                        if let Some(raw) = raw {
                            entry.1.add(raw);
                        }
                    }
                }
                let mut all = total.lock().expect("total");
                for (name, (z, raw)) in mine {
                    let entry = all.entry(name).or_default();
                    entry.0.merge(&z);
                    entry.1.merge(&raw);
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("worker");
    }

    println!("# {test}: {streams} streams from {first}, {bits} bits where the test reads bits");
    println!("row values mean_z sd_z beyond_0.05 beyond_0.01 beyond_0.001 raw_mean raw_sd");
    for (name, (z, raw)) in total.lock().expect("total").iter() {
        let n = z.count as f64;
        let rates: Vec<String> = z
            .beyond
            .iter()
            .map(|b| format!("{:.5}", *b as f64 / n))
            .collect();
        let raw_text = if raw.count > 0 {
            format!("{:.4} {:.4}", raw.mean(), raw.sd())
        } else {
            "- -".to_string()
        };
        println!(
            "{name} {} {:+.5} {:.5} {} {raw_text}",
            z.count,
            z.mean(),
            z.sd(),
            rates.join(" ")
        );
    }
}
