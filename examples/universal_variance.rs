//! Is Maurer's universal test calibrated at the battery's sample size?
//!
//! `universal_variance <streams> <threads> [bits]` runs the parametric family
//! on independently seeded null streams and reports, for each L that scores,
//! the mean and standard deviation of the z the test computes, with the
//! rejection rates that follow.
//!
//! The test divides f_n − μ by σ = c(L, K)·√(σ²/K).  μ and σ² are exact
//! properties of the geometric gap law, so if z has standard deviation 1 the
//! factor c is right for this K, and if it does not, c is what is wrong.  Two
//! published forms of c are compared: SP 800-22 Rev. 1a §2.9.4 step (5), which
//! is Maurer (1992) eq. (13) and what the suite computes, and the
//! Coron–Naccache form SP 800-22 §3.9 prints but does not use.  A standard
//! deviation of s under the first means the second would give s·c/c′.

use entropy::{
    nist::universal::universal_parametric_all,
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

/// Bits per stream unless the command line says otherwise: what the NIST
/// suite reads in `run_tests`.
const DEFAULT_BITS: usize = 16_000_000;

/// The two-sided levels whose rejection rates are counted.
const LEVELS: [f64; 3] = [0.05, 0.01, 0.001];

/// Initialisation blocks per setting, as the test uses: Q = 10·2^L.
const INIT_BLOCKS_PER_PATTERN: usize = 10;

/// c(L, K) as the suite computes it (SP 800-22 §2.9.4 step (5)).
fn c_suite(l: f64, k: f64) -> f64 {
    0.7 - 0.8 / l + (4.0 + 32.0 / l) * k.powf(-3.0 / l) / 15.0
}

/// c(L, K) as SP 800-22 §3.9 prints it from Coron and Naccache (SAC '98).
fn c_coron(l: f64, k: f64) -> f64 {
    0.7 - 0.8 / l + (1.6 + 12.8 / l) * k.powf(-4.0 / l)
}

/// The z values one stream produces, by test name.
fn stream_z(index: usize, bits: usize) -> Vec<(&'static str, f64)> {
    let seed = index as u128 + 1;
    let words: Vec<u8> = match index % 3 {
        0 => Pcg64::new(seed, 0x756e_6976_6572_7361).collect_bits(bits),
        1 => Xoshiro256::new(seed as u64 | 1, 2, 3, 4).collect_bits(bits),
        _ => Sfc64::new(seed as u64 | 1, 5, 6).collect_bits(bits),
    };
    universal_parametric_all(&words)
        .into_iter()
        .filter(|r| r.status == Status::Scored)
        .filter_map(|r| r.statistic.map(|s| (r.name, s.value)))
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let streams: usize = args[1].parse().expect("streams");
    let threads: usize = args[2].parse().expect("threads");
    let bits: usize = args.get(3).map_or(DEFAULT_BITS, |v| v.parse().expect("bits"));
    let next = Arc::new(AtomicUsize::new(0));
    // name -> (count, Σz, Σz², rejections per level)
    let tally: Arc<Mutex<BTreeMap<&'static str, (u64, f64, f64, [u64; LEVELS.len()])>>> =
        Arc::new(Mutex::new(BTreeMap::new()));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let (next, tally) = (next.clone(), tally.clone());
            thread::spawn(move || loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                if index >= streams {
                    break;
                }
                let values = stream_z(index, bits);
                let mut tally = tally.lock().expect("tally");
                for (name, z) in values {
                    let entry = tally.entry(name).or_insert((0, 0.0, 0.0, [0; 3]));
                    entry.0 += 1;
                    entry.1 += z;
                    entry.2 += z * z;
                    // The test's p-value is erfc(|z|/√2), so |z| decides.
                    for (count, level) in entry.3.iter_mut().zip(LEVELS) {
                        *count += u64::from(entropy::math::erfc(z.abs() / 2f64.sqrt()) < level);
                    }
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("worker");
    }

    println!("# {streams} streams of {bits} bits");
    println!("setting streams mean_z sd_z sd_z_coron reject_0.05 reject_0.01 reject_0.001");
    for (name, (count, sum, sum_squares, rejects)) in tally.lock().expect("tally").iter() {
        let n = *count as f64;
        let mean = sum / n;
        let variance = sum_squares / n - mean * mean;
        let sd = variance.sqrt();
        // K and L follow from the name's L and the stream length.
        let l: f64 = name
            .rsplit_once("_l")
            .and_then(|(_, digits)| digits.parse().ok())
            .expect("a parametric name");
        let q = INIT_BLOCKS_PER_PATTERN as f64 * 2f64.powf(l);
        let k = (bits as f64 / l).floor() - q;
        let scaled = sd * c_suite(l, k) / c_coron(l, k);
        let rates: Vec<String> = rejects
            .iter()
            .map(|r| format!("{:.5}", *r as f64 / n))
            .collect();
        println!(
            "{name} {count} {mean:+.4} {sd:.4} {scaled:.4} {}",
            rates.join(" ")
        );
    }
}
