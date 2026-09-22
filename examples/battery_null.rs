//! The whole battery under its null, repeatedly.
//!
//! `battery_null <streams> <threads> <output> [first]` runs the four suites at the
//! sizes `run_tests` uses on independently seeded generators, and accumulates,
//! for every result slot: how often it is scored, insufficient, unsupported or
//! an error, how often its p-value falls below 0.05, 0.01 and 0.001, and a
//! twenty-bin histogram of it.  It also records the decisions a reader makes
//! from a whole run: each family's Bonferroni decision at 0.01 and 0.001, and
//! the same over the whole battery.
//!
//! A snapshot is written to `<output>` every [`SNAPSHOT_EVERY`] streams, so a
//! long campaign can be read while it runs.  Streams are numbered from
//! `first`, 0 unless given, so a run that validates a correction can take
//! streams the calibration never saw.
//!
//! The streams differ from a `run_tests` run in one way: each suite reads the
//! generator where the last one stopped, without the fixed input segments that
//! binary enforces, so a suite that reads fewer words does not skip ahead.
//! Every stream is still a fresh generator seeded from its index.

use entropy::{
    diehard, dieharder, nist,
    result::{Status, TestResult},
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

/// Bits the NIST suite reads, as in `run_tests`.
const NIST_N: usize = 16_000_000;

/// Words the DIEHARD and DIEHARDER suites read.
const DIEHARD_N: usize = 16_000_000;

/// Words the historical DIEHARD tests capture, as in `run_tests`: the same
/// budget as the other suites, and above the 15 000 000 its hungriest test
/// needs.  Fewer leaves every one of them insufficient.
const DIEHARD_HISTORICAL_N: usize = DIEHARD_N;

/// Significance levels counted for every result.
const LEVELS: [f64; 3] = [0.05, 0.01, 0.001];

/// Bins of the p-value histogram.
const BINS: usize = 20;

/// Streams a worker runs between merging its counts into the total and
/// writing a snapshot.  With many workers the file is rewritten often, but a
/// snapshot costs a few milliseconds against tens of seconds of testing.
const SNAPSHOT_EVERY: usize = 25;

/// Counts for one result slot.
#[derive(Clone, Default)]
struct Slot {
    scored: u64,
    insufficient: u64,
    unsupported: u64,
    error: u64,
    below: [u64; LEVELS.len()],
    histogram: [u64; BINS],
}

impl Slot {
    fn add(&mut self, result: &TestResult) {
        match result.status {
            Status::Scored => {
                self.scored += 1;
                let p = result.p_value;
                for (count, level) in self.below.iter_mut().zip(LEVELS) {
                    *count += u64::from(p < level);
                }
                let bin = ((p * BINS as f64) as usize).min(BINS - 1);
                self.histogram[bin] += 1;
            }
            Status::Insufficient => self.insufficient += 1,
            Status::Unsupported => self.unsupported += 1,
            Status::Error => self.error += 1,
        }
    }

    fn merge(&mut self, other: &Self) {
        self.scored += other.scored;
        self.insufficient += other.insufficient;
        self.unsupported += other.unsupported;
        self.error += other.error;
        for (a, b) in self.below.iter_mut().zip(other.below) {
            *a += b;
        }
        for (a, b) in self.histogram.iter_mut().zip(other.histogram) {
            *a += b;
        }
    }
}

/// Everything one worker accumulates.
#[derive(Default)]
struct Tally {
    streams: u64,
    slots: BTreeMap<String, Slot>,
    /// Streams in which a family rejected, by family and level.
    family_rejects: BTreeMap<String, [u64; LEVELS.len()]>,
    /// Streams in which any family rejected, by level.
    battery_rejects: [u64; LEVELS.len()],
}

impl Tally {
    fn add_stream(&mut self, results: &[TestResult]) {
        self.streams += 1;
        // (results, smallest p) per family, over the scored results.
        let mut families: BTreeMap<&str, (usize, f64)> = BTreeMap::new();
        for result in results {
            self.slots
                .entry(result.name.to_string())
                .or_default()
                .add(result);
            if result.status == Status::Scored {
                let entry = families.entry(result.name).or_insert((0, 1.0));
                entry.0 += 1;
                entry.1 = entry.1.min(result.p_value);
            }
        }
        let mut any = [false; LEVELS.len()];
        for (family, (m, smallest)) in families {
            let counts = self.family_rejects.entry(family.to_string()).or_default();
            for ((count, level), flag) in counts.iter_mut().zip(LEVELS).zip(&mut any) {
                let rejected = m as f64 * smallest < level;
                *count += u64::from(rejected);
                *flag |= rejected;
            }
        }
        for (count, flag) in self.battery_rejects.iter_mut().zip(any) {
            *count += u64::from(flag);
        }
    }

    fn merge(&mut self, other: &Self) {
        self.streams += other.streams;
        for (name, slot) in &other.slots {
            self.slots.entry(name.clone()).or_default().merge(slot);
        }
        for (name, counts) in &other.family_rejects {
            let entry = self.family_rejects.entry(name.clone()).or_default();
            for (a, b) in entry.iter_mut().zip(counts) {
                *a += b;
            }
        }
        for (a, b) in self.battery_rejects.iter_mut().zip(other.battery_rejects) {
            *a += b;
        }
    }

    fn report(&self) -> String {
        let mut out = format!("# streams {}\n", self.streams);
        out.push_str("# name scored insufficient unsupported error below0.05 below0.01 below0.001 histogram\n");
        for (name, slot) in &self.slots {
            let bins: Vec<String> = slot.histogram.iter().map(u64::to_string).collect();
            out.push_str(&format!(
                "slot {name} {} {} {} {} {} {} {} {}\n",
                slot.scored,
                slot.insufficient,
                slot.unsupported,
                slot.error,
                slot.below[0],
                slot.below[1],
                slot.below[2],
                bins.join(",")
            ));
        }
        for (name, counts) in &self.family_rejects {
            out.push_str(&format!(
                "family {name} {} {} {}\n",
                counts[0], counts[1], counts[2]
            ));
        }
        out.push_str(&format!(
            "battery {} {} {}\n",
            self.battery_rejects[0], self.battery_rejects[1], self.battery_rejects[2]
        ));
        out
    }
}

/// The three generator families the campaign draws from, by stream index.
fn stream_results(index: usize) -> Vec<TestResult> {
    let seed = index as u128 + 1;
    match index % 3 {
        0 => whole_battery(&mut Pcg64::new(seed, 0x6e75_6c6c_5f63_616c)),
        1 => whole_battery(&mut Xoshiro256::new(
            seed as u64 | 1,
            0x9e37_79b9_7f4a_7c15,
            0xbf58_476d_1ce4_e5b9,
            0x94d0_49bb_1331_11eb,
        )),
        _ => whole_battery(&mut Sfc64::new(seed as u64 | 1, seed as u64 ^ 0xdead, 3)),
    }
}

/// The four suites in the order `run_tests` runs them.
fn whole_battery(rng: &mut impl Rng) -> Vec<TestResult> {
    let mut results = nist::run_all(rng, NIST_N);
    results.extend(diehard::run_all(rng, DIEHARD_N, false));
    results.extend(dieharder::run_all(rng, DIEHARD_N, false));
    results.extend(diehard::historical::run_all(rng, DIEHARD_HISTORICAL_N));
    results
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let streams: usize = args[1].parse().expect("streams");
    let threads: usize = args[2].parse().expect("threads");
    let output = args[3].clone();
    let first: usize = args.get(4).map_or(0, |v| v.parse().expect("first index"));
    let next = Arc::new(AtomicUsize::new(0));
    let total: Arc<Mutex<Tally>> = Arc::new(Mutex::new(Tally::default()));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let (next, total, output) = (next.clone(), total.clone(), output.clone());
            thread::spawn(move || {
                let mut mine = Tally::default();
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    if index >= streams {
                        break;
                    }
                    mine.add_stream(&stream_results(first + index));
                    if mine.streams % SNAPSHOT_EVERY as u64 == 0 {
                        let mut all = total.lock().expect("tally");
                        all.merge(&mine);
                        mine = Tally::default();
                        std::fs::write(&output, all.report()).expect("snapshot");
                    }
                }
                let mut all = total.lock().expect("tally");
                all.merge(&mine);
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("worker");
    }
    let all = total.lock().expect("tally");
    std::fs::write(&output, all.report()).expect("report");
    print!("{}", all.report());
}
