//! Upstream-suite research probes: PractRand FPF (platter + cross) and
//! TestU01 `sstring_HammingCorr` / `sstring_HammingIndep`, run across the
//! seeded RNG family.
//!
//! # References
//! * P. L'Ecuyer and R. Simard, "TestU01: A C Library for Empirical Testing
//!   of Random Number Generators," *ACM Transactions on Mathematical
//!   Software* 33(4), Article 22, 2007, §5.2.1,
//!   pp. 19–20 (`lecuyer2007testu01` in BIB.md).
//!   [pubs/lecuyer-simard-2007-testu01.pdf]
//! * C. Doty-Humphrey, "PractRand: Practically Random — A C++ Library of
//!   Statistical Tests for RNGs," 2018 (`practrand` in BIB.md; not in
//!   `pubs/`).

type Case<'a> = (&'a str, Box<dyn Fn() + 'a>);

use entropy::research::{
    practrand_fpf::{fpf_cross_result, fpf_platter_result, fpf_test, FpfConfig},
    testu01_hamming::{
        hamming_corr, hamming_corr_result, hamming_indep, hamming_indep_block_result,
        hamming_indep_main_result, HAMMING_INDEP_MAX_L,
    },
};
use entropy::rng::{
    AesCtr, BsdRandom, CryptoCtrDrbg, Lcg32, LcgVariant, LinuxLibcRandom, Mt19937, Rand48, Rng,
    SystemVRand, WindowsDotNetRandom, WindowsMsvcRand, WindowsVb6Rnd, Xorshift32, Xorshift64,
};
use entropy::seed::seed_material;

struct Args {
    rng_filters: Vec<String>,
    hc_n: usize,
    hc_r: usize,
    hc_s: usize,
    hc_l: usize,
    hi_n: usize,
    hi_r: usize,
    hi_s: usize,
    hi_l: usize,
    hi_d: usize,
    fpf_bits: usize,
}

impl Args {
    fn parse() -> Self {
        let mut out = Self {
            rng_filters: Vec::new(),
            hc_n: 500_000,
            hc_r: 20,
            hc_s: 10,
            hc_l: 300,
            hi_n: 500_000,
            hi_r: 20,
            hi_s: 10,
            hi_l: 300,
            hi_d: 1,
            fpf_bits: 1 << 27,
        };
        let argv: Vec<String> = std::env::args().skip(1).collect();
        let mut i = 0usize;
        while i < argv.len() {
            match argv[i].as_str() {
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                "--rng" => {
                    i += 1;
                    out.rng_filters.push(
                        argv.get(i)
                            .unwrap_or_else(|| die("--rng requires an argument"))
                            .clone(),
                    );
                }
                "--hc-n" => {
                    i += 1;
                    out.hc_n = parse_usize(argv.get(i), "--hc-n");
                }
                "--hc-r" => {
                    i += 1;
                    out.hc_r = parse_usize(argv.get(i), "--hc-r");
                }
                "--hc-s" => {
                    i += 1;
                    out.hc_s = parse_usize(argv.get(i), "--hc-s");
                }
                "--hc-l" => {
                    i += 1;
                    out.hc_l = parse_usize(argv.get(i), "--hc-l");
                }
                "--hi-n" => {
                    i += 1;
                    out.hi_n = parse_usize(argv.get(i), "--hi-n");
                }
                "--hi-r" => {
                    i += 1;
                    out.hi_r = parse_usize(argv.get(i), "--hi-r");
                }
                "--hi-s" => {
                    i += 1;
                    out.hi_s = parse_usize(argv.get(i), "--hi-s");
                }
                "--hi-l" => {
                    i += 1;
                    out.hi_l = parse_usize(argv.get(i), "--hi-l");
                }
                "--hi-d" => {
                    i += 1;
                    out.hi_d = parse_usize(argv.get(i), "--hi-d");
                }
                "--fpf-bits" => {
                    i += 1;
                    out.fpf_bits = parse_usize(argv.get(i), "--fpf-bits");
                }
                other => die(&format!("unknown option '{other}'")),
            }
            i += 1;
        }

        // Range checks mirror the asserts in research::testu01_hamming and
        // research::practrand_fpf so a bad flag dies with the flag's name
        // (exit 1) instead of a library panic (exit 101).
        if out.hc_n < 2 {
            die("--hc-n must be at least 2");
        }
        if !(1..=32).contains(&out.hc_s) {
            die("--hc-s must be in 1..=32");
        }
        if out.hc_r > 32 || out.hc_r + out.hc_s > 32 {
            die("--hc-r plus --hc-s must be <= 32");
        }
        if out.hc_l == 0 {
            die("--hc-l must be positive");
        }
        if out.hi_n < 20 {
            die("--hi-n must be at least 20");
        }
        if !(1..=32).contains(&out.hi_s) {
            die("--hi-s must be in 1..=32");
        }
        if out.hi_r > 32 || out.hi_r + out.hi_s > 32 {
            die("--hi-r plus --hi-s must be <= 32");
        }
        if !(1..=HAMMING_INDEP_MAX_L).contains(&out.hi_l) {
            die(&format!("--hi-l must be in 1..={HAMMING_INDEP_MAX_L}"));
        }
        if !(1..=8).contains(&out.hi_d) {
            die("--hi-d must be in 1..=8");
        }
        if out.hi_d > out.hi_l.div_ceil(2) {
            die("--hi-d must be <= (--hi-l + 1) / 2");
        }
        let fpf = FpfConfig::default();
        let worst_codeword = (1usize << fpf.exp_bits) - 1 + fpf.sig_bits;
        if out.fpf_bits < worst_codeword {
            die(&format!(
                "--fpf-bits must be at least {worst_codeword} (one worst-case codeword)"
            ));
        }
        out
    }

    fn matches_rng(&self, label: &str) -> bool {
        let label = label.to_lowercase();
        self.rng_filters.is_empty()
            || self
                .rng_filters
                .iter()
                .any(|pat| label.contains(&pat.to_lowercase()))
    }
}

fn parse_usize(v: Option<&String>, flag: &str) -> usize {
    v.unwrap_or_else(|| die(&format!("{flag} requires an argument")))
        .parse()
        .unwrap_or_else(|_| die(&format!("invalid value for {flag}")))
}

fn die(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1);
}

fn print_usage() {
    eprintln!(
        "Usage: upstream_tests [--rng <label>]\n\
                      [--hc-n N] [--hc-r N] [--hc-s N] [--hc-l N]\n\
                      [--hi-n N] [--hi-r N] [--hi-s N] [--hi-l N] [--hi-d N]\n\
                      [--fpf-bits N]\n\
         \n\
         Runs one honest TestU01 bit-string slice and one honest PractRand slice:\n\
         - TestU01 sstring_HammingCorr  (--hc-n blocks, r/s bit window, L bits)\n\
         - TestU01 sstring_HammingIndep (--hi-n pairs,  r/s bit window, L bits, d)\n\
         - PractRand FPF(4,14,6) core   (--fpf-bits total bits), parsing disjoint\n\
           codewords rather than upstream's stride-overlapped windows\n\
         \n\
         Defaults (moderate-size runs suitable for development checks):\n\
           hc-n=500000 hc-r=20 hc-s=10 hc-l=300\n\
           hi-n=500000 hi-r=20 hi-s=10 hi-l=300 hi-d=1  fpf-bits=2^27\n\
         Constraints: hc-n >= 2, hi-n >= 20, s in 1..=32, r+s <= 32,\n\
           hc-l >= 1, hi-l in 1..=4096, d in 1..=8, d <= (hi-l+1)/2,\n\
           fpf-bits >= 77 (one worst-case codeword).\n\
         Example:\n\
           cargo run --release --bin upstream_tests -- --rng AES"
    );
}

fn run_case(label: &str, mut rng: impl Rng, args: &Args) {
    println!("{label}");

    let hc = hamming_corr(&mut rng, args.hc_n, args.hc_r, args.hc_s, args.hc_l);
    println!("  {}", hamming_corr_result(&hc));

    let hi = hamming_indep(
        &mut rng, args.hi_n, args.hi_r, args.hi_s, args.hi_l, args.hi_d,
    );
    println!("  {}", hamming_indep_main_result(&hi));
    for k in 1..=args.hi_d {
        println!("  {}", hamming_indep_block_result(&hi, k));
    }

    let fpf = fpf_test(&mut rng, args.fpf_bits, &FpfConfig::default());
    println!("  {}", fpf_cross_result(&fpf));
    for platter in fpf.platter_results.iter().take(8) {
        println!("  {}", fpf_platter_result(platter, &fpf));
    }
    if fpf.platter_results.len() > 8 {
        println!(
            "  [INFO] practrand::fpf_more                   {} additional platter results omitted",
            fpf.platter_results.len() - 8
        );
    }
    println!();
}

fn main() {
    let args = Args::parse();
    let cases: Vec<Case<'_>> = vec![
        (
            "MT19937",
            Box::new(|| run_case("MT19937", Mt19937::new(19650218), &args)),
        ),
        (
            "Xorshift32",
            Box::new(|| run_case("Xorshift32", Xorshift32::new(1), &args)),
        ),
        (
            "Xorshift64",
            Box::new(|| run_case("Xorshift64", Xorshift64::new(1), &args)),
        ),
        (
            "BAD Unix System V rand()",
            Box::new(|| run_case("BAD Unix System V rand()", SystemVRand::new(1), &args)),
        ),
        (
            "BAD Unix System V mrand48()",
            Box::new(|| run_case("BAD Unix System V mrand48()", Rand48::new(1), &args)),
        ),
        (
            "BAD Unix BSD random()",
            Box::new(|| run_case("BAD Unix BSD random()", BsdRandom::new(1), &args)),
        ),
        (
            "BAD Unix Linux glibc rand()/random()",
            Box::new(|| {
                run_case(
                    "BAD Unix Linux glibc rand()/random()",
                    LinuxLibcRandom::new(1),
                    &args,
                )
            }),
        ),
        (
            "BAD Windows CRT rand()",
            Box::new(|| run_case("BAD Windows CRT rand()", WindowsMsvcRand::new(1), &args)),
        ),
        (
            "BAD Windows VB6/VBA Rnd()",
            Box::new(|| run_case("BAD Windows VB6/VBA Rnd()", WindowsVb6Rnd::new(1), &args)),
        ),
        (
            "BAD Windows .NET Random(seed)",
            Box::new(|| {
                run_case(
                    "BAD Windows .NET Random(seed)",
                    WindowsDotNetRandom::new(1),
                    &args,
                )
            }),
        ),
        (
            "ANSI C sample LCG",
            Box::new(|| run_case("ANSI C sample LCG", Lcg32::new(LcgVariant::AnsiC, 1), &args)),
        ),
        (
            "LCG MINSTD",
            Box::new(|| run_case("LCG MINSTD", Lcg32::new(LcgVariant::Minstd, 1), &args)),
        ),
        (
            "AES-128-CTR",
            Box::new(|| {
                run_case(
                    "AES-128-CTR",
                    AesCtr::new(&seed_material::<16>(1), 0),
                    &args,
                )
            }),
        ),
        (
            "cryptography::CtrDrbgAes256",
            Box::new(|| {
                run_case(
                    "cryptography::CtrDrbgAes256",
                    CryptoCtrDrbg::new(&seed_material::<48>(1)),
                    &args,
                )
            }),
        ),
    ];

    let mut matched = 0usize;
    for (label, case) in cases {
        if !args.matches_rng(label) {
            continue;
        }
        matched += 1;
        case();
    }
    if matched == 0 {
        die("no RNG labels matched --rng filter");
    }
}
