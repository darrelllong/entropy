//! Estimates the mean and standard deviation of the LZ78 phrase count of 2^k
//! fair random bits, for the Lempel–Ziv test's `LZ_MEAN_SD` table.
//!
//! Each thread runs replications of `lempel_ziv_replication` with r = 0 and
//! s = 32 on its own PCG64 stream and accumulates the phrase counts' first two
//! moments exactly as integers.  One line per k gives the replications, the
//! sums Σ W and Σ W², the mean, the standard deviation and the standard error
//! of the mean, so runs on several machines can be pooled.
//!
//! Usage: `lz78_table <k> <replications per thread> <threads> <first stream>`

use entropy::{research::testu01_lz::lempel_ziv_replication, rng::Pcg64};
use std::thread;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: lz78_table <k> <replications per thread> <threads> <first stream>";
    let [_, k, reps, threads, first] = args.as_slice() else {
        panic!("{usage}");
    };
    let k: usize = k.parse().expect(usage);
    let reps: u64 = reps.parse().expect(usage);
    let threads: u64 = threads.parse().expect(usage);
    let first: u64 = first.parse().expect(usage);
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            thread::spawn(move || {
                let mut rng = Pcg64::new(0x6c7a_3738_7461_626c, u128::from(first + t));
                let (mut sum, mut sum_sq) = (0u128, 0u128);
                for _ in 0..reps {
                    let w = lempel_ziv_replication(&mut rng, k, 0, 32).phrase_count as u128;
                    sum += w;
                    sum_sq += w * w;
                }
                (sum, sum_sq)
            })
        })
        .collect();
    let (mut sum, mut sum_sq) = (0u128, 0u128);
    for handle in handles {
        let (s, q) = handle.join().expect("worker panicked");
        sum += s;
        sum_sq += q;
    }
    let count = u128::from(reps * threads);
    let n = count as f64;
    let mean = sum as f64 / n;
    // n·ΣW² − (ΣW)² is exact in u128 at these sizes.
    let variance = (count * sum_sq - sum * sum) as f64 / (n * (n - 1.0));
    println!(
        "k={k} replications={} sum={sum} sum_sq={sum_sq} mean={mean:.6} sd={:.6} se={:.6}",
        reps * threads,
        variance.sqrt(),
        (variance / n).sqrt()
    );
}
