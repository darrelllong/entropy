type Case<'a> = (&'a str, Box<dyn Fn() -> [usize; 64] + 'a>);

use entropy::nist::linear_complexity::berlekamp_massey;
use entropy::rng::{
    AesCtr, CryptoCtrDrbg, Lcg32, LcgVariant, Mt19937, Rng, Xorshift32, Xorshift64,
};
use entropy::seed::seed_material;

struct Args {
    words: usize,
    rng_filters: Vec<String>,
}

impl Args {
    fn parse() -> Self {
        let mut words = 4096usize;
        let mut rng_filters = Vec::new();
        let argv: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0;
        while i < argv.len() {
            match argv[i].as_str() {
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--words" => {
                    i += 1;
                    words = argv
                        .get(i)
                        .unwrap_or_else(|| die("--words requires an argument"))
                        .parse()
                        .unwrap_or_else(|_| die("invalid --words value"));
                }
                "--rng" => {
                    i += 1;
                    rng_filters.push(
                        argv.get(i)
                            .unwrap_or_else(|| die("--rng requires an argument"))
                            .clone(),
                    );
                }
                other => die(&format!("unknown option '{other}'")),
            }
            i += 1;
        }
        Self { words, rng_filters }
    }

    fn matches_rng(&self, label: &str) -> bool {
        self.rng_filters.is_empty() || self.rng_filters.iter().any(|pat| label.contains(pat))
    }
}

fn die(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1);
}

fn print_usage() {
    eprintln!(
        "Usage: bitplane_complexity [--rng <label>] [--words N]\n\
         \n\
         Measures Berlekamp-Massey linear complexity on each individual output\n\
         bit plane across successive 64-bit outputs.\n\
         \n\
         Example:\n\
           cargo run --release --bin bitplane_complexity -- --rng Xorshift64"
    );
}

fn bitplane_complexities(mut rng: impl Rng, words: usize) -> [usize; 64] {
    let outputs: Vec<u64> = (0..words).map(|_| rng.next_u64()).collect();
    let mut out = [0usize; 64];
    for (bit, slot) in out.iter_mut().enumerate() {
        let seq: Vec<u8> = outputs.iter().map(|&w| ((w >> bit) & 1) as u8).collect();
        *slot = berlekamp_massey(&seq);
    }
    out
}

fn summarize(label: &str, complexities: &[usize; 64], words: usize) {
    let min = complexities.iter().min().copied().unwrap_or(0);
    let max = complexities.iter().max().copied().unwrap_or(0);
    let mean = complexities.iter().map(|&x| x as f64).sum::<f64>() / 64.0;
    let low_bits = complexities[..8]
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let high_bits = complexities[56..]
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{label}\n  words={words} bitplane_linear_complexity min={min} mean={mean:.1} max={max}\n  low8=[{low_bits}] high8=[{high_bits}]\n"
    );
}

fn main() {
    let args = Args::parse();
    let cases: Vec<Case<'_>> = vec![
        (
            "Xorshift64",
            Box::new(|| bitplane_complexities(Xorshift64::new(1), args.words)),
        ),
        (
            "Xorshift32",
            Box::new(|| bitplane_complexities(Xorshift32::new(1), args.words)),
        ),
        (
            "MT19937",
            Box::new(|| bitplane_complexities(Mt19937::new(19650218), args.words)),
        ),
        (
            "ANSI C sample LCG",
            Box::new(|| bitplane_complexities(Lcg32::new(LcgVariant::AnsiC, 1), args.words)),
        ),
        (
            "AES-128-CTR",
            Box::new(|| bitplane_complexities(AesCtr::new(&seed_material::<16>(1), 0), args.words)),
        ),
        (
            "cryptography::CtrDrbgAes256",
            Box::new(|| {
                bitplane_complexities(CryptoCtrDrbg::new(&seed_material::<48>(1)), args.words)
            }),
        ),
    ];

    let mut matched = 0usize;
    for (label, case) in cases {
        if !args.matches_rng(label) {
            continue;
        }
        matched += 1;
        summarize(label, &case(), args.words);
    }
    if matched == 0 {
        die("no RNG labels matched --rng filter");
    }
}
