//! Power of the NIST SP 800-22 tests against specified defects.
//!
//! `power_curves <streams> <threads>` runs the NIST suite on `streams`
//! independently seeded PCG64 generators through each defect of
//! `entropy::rng::alternatives`, at several strengths and sample sizes, and
//! prints, for each test family, the fraction of streams in which any of its
//! results fell below 0.01.  A family's own false-alarm rate on the
//! undefected generator is printed under the defect `none`.
//! Output lines: `defect strength bits family rejected/streams`.

use entropy::{
    nist,
    result::TestResult,
    rng::{
        alternatives::{Biased, LaggedMsb, RepeatedBlocks, ShortPeriod, StuckLowBits},
        Pcg64,
    },
};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
};

/// Streams rejected, by (defect, size, family).
type Tally = BTreeMap<(usize, usize, &'static str), usize>;

/// A defect and its strength parameter.
#[derive(Clone, Copy)]
enum Defect {
    None,
    Biased(u32),
    StuckLowBits(u32),
    RepeatedBlocks(usize),
    LaggedMsb,
    ShortPeriod(usize),
}

impl Defect {
    fn label(self) -> (&'static str, String) {
        match self {
            Defect::None => ("none", "-".into()),
            Defect::Biased(k) => ("bias", format!("2^-{}", k + 1)),
            Defect::StuckLowBits(b) => ("stuck_low_bits", b.to_string()),
            Defect::RepeatedBlocks(len) => ("repeated_blocks", len.to_string()),
            Defect::LaggedMsb => ("lagged_msb", "-".into()),
            Defect::ShortPeriod(p) => ("short_period", p.to_string()),
        }
    }

    fn run(self, seed: u128, bits: usize) -> Vec<TestResult> {
        let base = Pcg64::new(0x706f_7765_7263_7572, seed);
        match self {
            Defect::None => nist::run_all(&mut { base }, bits),
            Defect::Biased(k) => nist::run_all(&mut Biased::new(base, k), bits),
            Defect::StuckLowBits(b) => nist::run_all(&mut StuckLowBits::new(base, b), bits),
            Defect::RepeatedBlocks(len) => nist::run_all(&mut RepeatedBlocks::new(base, len), bits),
            Defect::LaggedMsb => nist::run_all(&mut LaggedMsb::new(base), bits),
            Defect::ShortPeriod(p) => nist::run_all(&mut ShortPeriod::new(base, p), bits),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let streams: usize = args[1].parse().expect("streams");
    let threads: usize = args[2].parse().expect("threads");
    let defects = [
        Defect::None,
        Defect::Biased(8),
        Defect::Biased(10),
        Defect::Biased(12),
        Defect::StuckLowBits(1),
        Defect::RepeatedBlocks(1 << 12),
        Defect::RepeatedBlocks(1 << 16),
        Defect::LaggedMsb,
        Defect::ShortPeriod(1 << 12),
        Defect::ShortPeriod(1 << 16),
    ];
    let sizes = [1usize << 20, 1 << 22, 1 << 24];
    let jobs: Vec<(usize, usize, usize)> = (0..defects.len())
        .flat_map(|d| (0..sizes.len()).flat_map(move |s| (0..streams).map(move |i| (d, s, i))))
        .collect();
    let jobs = Arc::new(jobs);
    let next = Arc::new(AtomicUsize::new(0));
    let tally: Arc<Mutex<Tally>> = Arc::new(Mutex::new(BTreeMap::new()));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let (jobs, next, tally) = (jobs.clone(), next.clone(), tally.clone());
            thread::spawn(move || loop {
                let j = next.fetch_add(1, Ordering::Relaxed);
                let Some(&(d, s, i)) = jobs.get(j) else { break };
                let seed = ((d as u128) << 80) | ((s as u128) << 64) | i as u128;
                let results = defects[d].run(seed, sizes[s]);
                let mut families: BTreeMap<&'static str, bool> = BTreeMap::new();
                for r in &results {
                    let rejected = families.entry(r.name).or_insert(false);
                    *rejected |= r.failed();
                }
                let mut tally = tally.lock().expect("tally");
                for (family, rejected) in families {
                    *tally.entry((d, s, family)).or_insert(0) += usize::from(rejected);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("worker");
    }
    for ((d, s, family), rejected) in tally.lock().expect("tally").iter() {
        let (name, strength) = defects[*d].label();
        println!(
            "{name} {strength} {} {family} {rejected}/{streams}",
            sizes[*s]
        );
    }
}
