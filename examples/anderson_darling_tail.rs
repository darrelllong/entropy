//! Monte Carlo of the Anderson–Darling statistic Aₙ for `n` iid U(0, 1)
//! samples: the upper tails Pr(Aₙ ≥ z) that `entropy::math`'s tests pin.
//!
//! Each sample draws n + 1 exponential variates Eᵢ = −ln Uᵢ and takes the
//! ordered uniforms xᵢ = (E₁ + … + Eᵢ)/S, with S = E₁ + … + Eₙ₊₁, the
//! construction G. Marsaglia and J. C. W. Marsaglia describe for their own
//! simulations ("Evaluating the Anderson-Darling Distribution," *Journal of
//! Statistical Software* 9(2), 2004, §4, p. 5).
//! [pubs/marsaglia-marsaglia-2004-anderson-darling.pdf]  1 − xₙ₊₁₋ᵢ comes from
//! suffix sums, so neither end loses precision to cancellation.  Uniforms are
//! ((w >> 11) + ½)·2⁻⁵³ for successive outputs w of xoshiro256**
//! (`entropy::rng::Xoshiro256`).
//!
//! The samples are split into 64 chunks, each with its own generator seeded
//! by four successive `entropy::seed::splitmix64` outputs from `--seed`, so
//! the counts depend on `--n`, `--samples` and `--seed` but not on
//! `--threads`.  The program prints the tail at each `--z` with its binomial
//! standard error and the crate's `1 − anderson_darling_cdf(n, z)`, the
//! worst relative errors of that p-value over z in 0.01 steps, and the rows
//! pinned in `src/math.rs`.
//!
//! The table pinned in `src/math.rs`, and the accuracy figures its
//! `anderson_darling_cdf` docs quote, come from these runs with the default
//! seed, 20260911 (about four minutes on twelve cores):
//!
//! ```text
//! cargo run --release --example anderson_darling_tail -- --n 8 --samples 4000000000
//! cargo run --release --example anderson_darling_tail -- --n 16 --samples 1000000000
//! cargo run --release --example anderson_darling_tail -- --n 32 --samples 1000000000
//! cargo run --release --example anderson_darling_tail -- --n 64 --samples 200000000
//! cargo run --release --example anderson_darling_tail -- --n 128 --samples 200000000
//! ```
//!
//! For n below the crate's minimum of 8 the program still prints the
//! simulated tails; `anderson_darling_cdf` returns NaN there.

#![forbid(unsafe_code)]

use entropy::math::anderson_darling_cdf;
use entropy::rng::{Rng, Xoshiro256};
use entropy::seed::splitmix64;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;

/// Independent generator streams the samples are split across.
const CHUNKS: usize = 64;
/// Histogram resolution: sample A lands in bin ⌊100·A⌋.
const BINS_PER_UNIT: f64 = 100.0;
/// Last histogram bin; it also holds every A ≥ 40.
const LAST_BIN: usize = 4000;
/// Seed the pinned table was generated with.
const DEFAULT_SEED: u64 = 20_260_911;
/// 2⁻⁵³, the spacing of the uniforms.
const UNIT: f64 = 1.0 / (1u64 << 53) as f64;
/// A tail count below this has a relative standard error above 5% and is
/// left out of the worst-case summary.
const MIN_COUNT: u64 = 400;
/// Last z below the switch in `math::anderson_darling_cdf`, ADinf(z) = 0.9995.
const LAST_Z_BELOW_SWITCH: f64 = 6.61;
/// Default tail points, in units of A.
const DEFAULT_Z: [f64; 14] = [
    2.0, 4.0, 6.0, 6.5, 6.61, 6.62, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0,
];

struct Options {
    n: usize,
    samples: u64,
    seed: u64,
    threads: usize,
    z: Vec<f64>,
}

fn usage(message: &str) -> ! {
    eprintln!(
        "error: {message}\n\
         usage: anderson_darling_tail [--n N] [--samples S] [--seed SEED] [--threads T] [--z Z1,Z2,...]"
    );
    process::exit(1);
}

fn parse_value<T: std::str::FromStr>(flag: &str, value: &str) -> T {
    value
        .parse()
        .unwrap_or_else(|_| usage(&format!("invalid value for {flag}: {value:?}")))
}

fn parse_options() -> Options {
    let mut options = Options {
        n: 32,
        samples: 100_000_000,
        seed: DEFAULT_SEED,
        threads: thread::available_parallelism().map_or(1, usize::from),
        z: DEFAULT_Z.to_vec(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .unwrap_or_else(|| usage(&format!("{flag} requires a value")));
        match flag.as_str() {
            "--n" => options.n = parse_value(&flag, &value),
            "--samples" => options.samples = parse_value(&flag, &value),
            "--seed" => options.seed = parse_value(&flag, &value),
            "--threads" => options.threads = parse_value(&flag, &value),
            "--z" => options.z = value.split(',').map(|z| parse_value(&flag, z)).collect(),
            _ => usage(&format!("unknown option {flag}")),
        }
    }
    if options.n == 0 || options.samples == 0 || options.threads == 0 {
        usage("--n, --samples and --threads must be positive");
    }
    options
}

/// Histogram of ⌊100·Aₙ⌋ over `samples` draws from one generator.
fn chunk_histogram(n: usize, samples: u64, seed: [u64; 4]) -> Vec<u64> {
    let [s0, s1, s2, s3] = seed;
    let mut rng = Xoshiro256::new(s0, s1, s2, s3);
    let mut histogram = vec![0u64; LAST_BIN + 1];
    let mut exponentials = vec![0.0f64; n + 1];
    let mut prefix = vec![0.0f64; n + 1];
    for _ in 0..samples {
        let mut sum = 0.0;
        for (e, p) in exponentials.iter_mut().zip(prefix.iter_mut()) {
            let u = ((rng.next_u64() >> 11) as f64 + 0.5) * UNIT;
            *e = -u.ln();
            sum += *e;
            *p = sum;
        }
        let total = prefix[n];
        // For i = 1..=n: xᵢ = prefix[i − 1]/S and 1 − xₙ₊₁₋ᵢ = (Eₙ₊₂₋ᵢ + … + Eₙ₊₁)/S.
        let mut suffix = 0.0;
        let mut weighted = 0.0;
        for (i, (&p, &e)) in prefix[..n]
            .iter()
            .zip(exponentials[1..].iter().rev())
            .enumerate()
        {
            suffix += e;
            weighted += (2 * i + 1) as f64 * ((p / total).ln() + (suffix / total).ln());
        }
        let a = -(n as f64) - weighted / n as f64;
        let bin = ((a * BINS_PER_UNIT) as usize).min(LAST_BIN);
        histogram[bin] += 1;
    }
    histogram
}

fn main() {
    let options = parse_options();
    let mut state = options.seed;
    let seeds: Vec<[u64; 4]> = (0..CHUNKS)
        .map(|_| std::array::from_fn(|_| splitmix64(&mut state)))
        .collect();
    let next_chunk = AtomicUsize::new(0);
    let merged = Mutex::new(vec![0u64; LAST_BIN + 1]);
    thread::scope(|scope| {
        for _ in 0..options.threads {
            scope.spawn(|| loop {
                let k = next_chunk.fetch_add(1, Ordering::Relaxed);
                if k >= CHUNKS {
                    break;
                }
                let chunks = CHUNKS as u64;
                let samples =
                    options.samples / chunks + u64::from((k as u64) < options.samples % chunks);
                let histogram = chunk_histogram(options.n, samples, seeds[k]);
                let mut merged = merged.lock().expect("a worker panicked");
                for (total, count) in merged.iter_mut().zip(&histogram) {
                    *total += count;
                }
            });
        }
    });
    let histogram = merged.into_inner().expect("a worker panicked");

    // tail[b] counts samples with A ≥ b/100.
    let mut tail = vec![0u64; LAST_BIN + 2];
    for b in (0..=LAST_BIN).rev() {
        tail[b] = tail[b + 1] + histogram[b];
    }
    let total = options.samples as f64;
    let estimate = |bin: usize| {
        let count = tail[bin.min(LAST_BIN)];
        let t = count as f64 / total;
        (count, t, (t * (1.0 - t) / total).sqrt())
    };
    let crate_p = |z: f64| 1.0 - anderson_darling_cdf(options.n, z);

    println!(
        "# Anderson-Darling A_n Monte Carlo: n = {}, samples = {}, seed = {}, chunks = {CHUNKS}",
        options.n, options.samples, options.seed
    );
    println!("#       z           count          tail          se     crate p  crate/tail - 1");
    for &z in &options.z {
        let (count, t, se) = estimate((z * BINS_PER_UNIT).round() as usize);
        let p = crate_p(z);
        println!(
            "{z:9.2} {count:15} {t:13.6e} {se:11.3e} {p:11.4e} {:+14.2}%",
            100.0 * (p / t - 1.0)
        );
    }

    // Worst errors of the crate's p-value over z in 0.01 steps: absolute in
    // the body of the distribution, relative in the tail.
    if crate_p(1.0).is_nan() {
        println!(
            "# anderson_darling_cdf returns NaN for n = {}; only the simulated tails above are defined",
            options.n
        );
    } else {
        let worst_body = (1..=4 * BINS_PER_UNIT as usize)
            .map(|bin| {
                let (_, t, se) = estimate(bin);
                let z = bin as f64 / BINS_PER_UNIT;
                (crate_p(z) - t, z, se)
            })
            .fold((0.0f64, 0.0f64, 0.0f64), |a, b| {
                if b.0.abs() > a.0.abs() {
                    b
                } else {
                    a
                }
            });
        println!(
            "# 0 < z <= 4: crate p absolute error at most {:.2e} (z = {:.2}, {:+.1} standard errors)",
            worst_body.0.abs(),
            worst_body.1,
            worst_body.0 / worst_body.2
        );
        let regions = [
            ("4 < z <= 6.61", 4.01, LAST_Z_BELOW_SWITCH),
            ("6.61 < z <= 12", LAST_Z_BELOW_SWITCH + 0.01, 12.0),
        ];
        for (name, from, to) in regions {
            let first = (from * BINS_PER_UNIT).round() as usize;
            let last = (to * BINS_PER_UNIT).round() as usize;
            let errors: Vec<(f64, f64)> = (first..=last)
                .filter_map(|bin| {
                    let (count, t, _) = estimate(bin);
                    let z = bin as f64 / BINS_PER_UNIT;
                    (count >= MIN_COUNT).then(|| (crate_p(z) / t - 1.0, z))
                })
                .collect();
            let Some(&first_error) = errors.first() else {
                println!("# {name}: no point with at least {MIN_COUNT} counts");
                continue;
            };
            let low = errors
                .iter()
                .copied()
                .fold(first_error, |a, b| if b.0 < a.0 { b } else { a });
            let high = errors
                .iter()
                .copied()
                .fold(first_error, |a, b| if b.0 > a.0 { b } else { a });
            println!(
                "# {name}: crate p relative error {:+.2}% (z = {:.2}) .. {:+.2}% (z = {:.2}) over {} points with at least {MIN_COUNT} counts",
                100.0 * low.0,
                low.1,
                100.0 * high.0,
                high.1,
                errors.len()
            );
        }
    }

    println!("# rows for src/math.rs: (n, z, simulated tail, standard error)");
    for &z in &options.z {
        let (_, t, se) = estimate((z * BINS_PER_UNIT).round() as usize);
        println!("({}, {z:?}, {t:e}, {se:.2e}),", options.n);
    }
}
