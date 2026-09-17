//! Golden p-values for every DIEHARD and DIEHARDER test at a fixed seed.
//!
//! These are regression values produced by this crate's own code and pinned
//! so that a change to any statistic, sample layout or p-value routine fails
//! a test.  Passing says nothing about whether a statistic is right, only
//! that it has not moved.  An intended change to a statistic must update this
//! table and say so in its commit.
//!
//! Every test reads MT19937 seeded with 5489.  Tests over a word slice take
//! the first N words of one stream, N being the smallest length the test's
//! gate accepts, and check that N − 1 words skip.  `bit_distribution`
//! accepts 404 words but scores every pattern of every width through 8 only
//! from 1 456, the length used.  Tests that draw from a generator get a fresh
//! one and run at their `quick` size where they have one; `permutations`
//! (t = 5) and `lagged_sums` (lags 1 and 100) use the battery's parameters.
//!
//! Every golden runs in a few seconds of debug-build test time.

use entropy::{
    diehard, dieharder,
    result::TestResult,
    rng::{Mt19937, Rng},
};
use std::sync::OnceLock;

const SEED: u32 = 5489;

/// P-value tolerance.  Counts are exact on every platform, but p-values and
/// some expected cell counts pass through `exp`, `ln`, `cos`, `sin` and
/// `powf`, which Rust takes from the platform libm, and CI runs on both glibc
/// (Linux x86-64) and Apple's libm (macOS arm64), which can differ by an ulp
/// or so per call.  Shifting every such result one ulp up, then one ulp down,
/// then by ±1 ulp at random on 1%, 3% and 10% of calls over 60 seeds each
/// left every golden but `binary_rank_31x31` within 1e-12, and moved that one
/// by up to 3.7e-10 (see `RANK_31X31_TOL`).  1e-12 therefore holds for the
/// others under those perturbations and still catches any change a real edit
/// makes.  Notes are
/// compared exactly: they print statistics to four decimals, and the same
/// perturbations left every other golden's note unchanged.
const TOL: f64 = 1e-12;

/// Tolerance for `binary_rank_31x31` only.  Its cell probabilities come from
/// `gf2_rank_probability`, which sums about 90 `ln` terms, and its tail cell
/// is 1 − P(31) − P(30) − P(29), so its p-value is far more sensitive to libm
/// rounding than the others.  Shifting every `exp`, `ln`, `cos`, `sin` and
/// `powf` result by one ulp moved this p-value 8.2e-11 upward and 3.7e-10
/// downward, while every other golden stayed within 1e-12.  1e-8 is about 27
/// times the larger move.
const RANK_31X31_TOL: f64 = 1e-8;

/// 9 bit offsets × 500 trials × 512 birthdays, the hungriest slice test.
const BIRTHDAY_WORDS: usize = 9 * 500 * 512;
/// 40 000 matrices of 32 rows.
const RANK_32X32_WORDS: usize = 32 * 40_000;
/// 40 000 matrices of 31 rows.
const RANK_31X31_WORDS: usize = 31 * 40_000;
/// 100 000 matrices of 6 rows.
const RANK_6X8_WORDS: usize = 6 * 100_000;
/// 20 repeats of 2²¹ overlapping 20-bit windows, ⌈(2²¹ + 19) / 32⌉ words each.
const BITSTREAM_WORDS: usize = 20 * ((1usize << 21) + 19).div_ceil(32);
/// 2²¹ words.
const OPSO_WORDS: usize = 1 << 21;
/// Four words per six letters for 2²¹ letters, plus a last partial group.
const OQSO_WORDS: usize = (1 << 21) / 6 * 4 + 4;
/// Ten words per sixteen letters for 2²¹ letters.
const DNA_WORDS: usize = (1 << 21) / 16 * 10;
/// 256 000 overlapping five-byte words, ⌈256 004 / 4⌉ words.
const COUNT_ONES_WORDS: usize = (256_000usize + 4).div_ceil(4);
/// 10 sequences of 10 000 words.
const RUNS_WORDS: usize = 10 * 10_000;
/// 1 000 samples.
const KS_UNIFORM_WORDS: usize = 1_000;
/// 1 280 trials of three words, five expected counts per byte value.
const BYTE_DISTRIBUTION_WORDS: usize = 3 * 5 * 256;
/// 5 000 blocks of 256 words.
const DCT_WORDS: usize = 5_000 * 256;
/// The fewest words that give monobit2 one block size (pairs of words).
const MONOBIT2_WORDS: usize = 404;
/// The fewest words that give monobit2 two block sizes (2 and 4 words).
const MONOBIT2_TWO_LEVEL_WORDS: usize = 1_140;
/// Eight words per trial for 100 000 trials.
const FILL_TREE_WORDS: usize = 8 * 100_000;
/// The fewest words for which every pattern of widths 1 to 8 is scored.
const BIT_DISTRIBUTION_WORDS: usize = 1_456;

/// One pinned result.
struct Golden {
    name: &'static str,
    p: f64,
    note: &'static str,
}

/// The first `BIRTHDAY_WORDS` words of the seeded stream, generated once.
fn words() -> &'static [u32] {
    static WORDS: OnceLock<Vec<u32>> = OnceLock::new();
    WORDS.get_or_init(|| Mt19937::new(SEED).collect_u32s(BIRTHDAY_WORDS))
}

fn fresh() -> Mt19937 {
    Mt19937::new(SEED)
}

/// Runs a slice test on the first `len` words after checking that one word
/// fewer skips.
fn at_gate(test: fn(&[u32]) -> TestResult, len: usize) -> TestResult {
    assert!(
        test(&words()[..len - 1]).skipped(),
        "{len} words is not the smallest length accepted"
    );
    test(&words()[..len])
}

fn check(results: &[TestResult], want: &[Golden]) {
    check_within(results, want, TOL);
}

fn check_within(results: &[TestResult], want: &[Golden], tol: f64) {
    assert_eq!(results.len(), want.len(), "{results:?}");
    for (r, w) in results.iter().zip(want) {
        assert_eq!(r.name, w.name);
        assert!(
            (r.p_value - w.p).abs() <= tol,
            "{}: p = {:?}, pinned {:?}",
            r.name,
            r.p_value,
            w.p
        );
        assert_eq!(r.note.as_deref(), Some(w.note), "{}", r.name);
    }
}

// ── DIEHARD ───────────────────────────────────────────────────────────────────

#[test]
fn birthday_spacings() {
    check(
        &[at_gate(
            diehard::birthday_spacings::birthday_spacings,
            BIRTHDAY_WORDS,
        )],
        &[Golden {
            name: "diehard::birthday_spacings",
            p: 0.16803004996661564,
            note: "m=512, year=2^24, samples=500",
        }],
    );
}

#[test]
fn binary_rank_32x32() {
    check(
        &[at_gate(
            diehard::binary_rank::binary_rank_32x32,
            RANK_32X32_WORDS,
        )],
        &[Golden {
            name: "diehard::binary_rank_32x32",
            p: 0.49409984519437,
            note: "32×32, N=40000, χ²=2.3975",
        }],
    );
}

#[test]
fn binary_rank_31x31() {
    check_within(
        &[at_gate(
            diehard::binary_rank::binary_rank_31x31,
            RANK_31X31_WORDS,
        )],
        &[Golden {
            name: "diehard::binary_rank_31x31",
            p: 0.5968570813626386,
            note: "31×31, N=40000, χ²=1.8839",
        }],
        RANK_31X31_TOL,
    );
}

#[test]
fn binary_rank_6x8() {
    check(
        &[at_gate(
            diehard::binary_rank::binary_rank_6x8,
            RANK_6X8_WORDS,
        )],
        &[Golden {
            name: "diehard::binary_rank_6x8",
            p: 0.6443334492990846,
            note: "N=100000, χ²=0.8791",
        }],
    );
}

#[test]
fn bitstream() {
    check(
        &[at_gate(diehard::bitstream::bitstream, BITSTREAM_WORDS)],
        &[Golden {
            name: "diehard::bitstream",
            p: 0.8116127766486776,
            note: "window=20-bit, stream=2^21, repeats=20",
        }],
    );
}

#[test]
fn opso() {
    check(
        &[at_gate(diehard::monkey::opso, OPSO_WORDS)],
        &[Golden {
            name: "diehard::opso",
            p: 0.21780109549439705,
            note: "missing=142267, z=1.2324",
        }],
    );
}

#[test]
fn oqso() {
    check(
        &[at_gate(diehard::monkey::oqso, OQSO_WORDS)],
        &[Golden {
            name: "diehard::oqso",
            p: 0.3617295034929668,
            note: "missing=142174, z=0.9121",
        }],
    );
}

#[test]
fn dna() {
    check(
        &[at_gate(diehard::monkey::dna, DNA_WORDS)],
        &[Golden {
            name: "diehard::dna",
            p: 0.10097051039306663,
            note: "missing=141433, z=-1.6402",
        }],
    );
}

#[test]
fn count_ones_stream() {
    check(
        &[at_gate(
            diehard::count_ones::count_ones_stream,
            COUNT_ONES_WORDS,
        )],
        &[Golden {
            name: "diehard::count_ones_stream",
            p: 0.20596684098343668,
            note: "n=256000, Q5=3248.43, Q4=658.62, Q5-Q4=2589.82",
        }],
    );
}

#[test]
fn runs_float() {
    check(
        &[at_gate(diehard::runs_float::runs_float, RUNS_WORDS)],
        &[Golden {
            name: "diehard::runs_up_down",
            p: 0.2905887267470262,
            note: "seq_len=10000, repeats=10, covariance-form (Bonferroni)",
        }],
    );
}

#[test]
fn parking_lot() {
    check(
        &[diehard::parking_lot::parking_lot(&mut fresh(), true)],
        &[Golden {
            name: "diehard::parking_lot",
            p: 0.4076012214433825,
            note: "attempts=12000, mean=3523, σ=21.9, repeats=5",
        }],
    );
}

#[test]
fn minimum_distance_2d() {
    check(
        &[diehard::minimum_distance::minimum_distance_2d(
            &mut fresh(),
            true,
        )],
        &[Golden {
            name: "diehard::minimum_distance_2d",
            p: 0.9347249730007391,
            note: "n=500, side=10000, repeats=20",
        }],
    );
}

#[test]
fn spheres_3d() {
    check(
        &[diehard::spheres_3d::spheres_3d(&mut fresh(), true)],
        &[Golden {
            name: "diehard::spheres_3d",
            p: 0.4877886534091478,
            note: "n=500, cube=1000, repeats=10",
        }],
    );
}

#[test]
fn squeeze() {
    check(
        &[diehard::squeeze::squeeze(&mut fresh())],
        &[Golden {
            name: "diehard::squeeze",
            p: 0.7620955667523143,
            note: "trials=100000, cells=43, df=38, χ²=31.5167",
        }],
    );
}

#[test]
fn runs_float_both() {
    check(
        &diehard::runs_float::runs_float_both(&mut fresh()),
        &[
            Golden {
                name: "diehard::runs_up",
                p: 0.886685504594146,
                note: "seq_len=10000, repeats=10, covariance-form",
            },
            Golden {
                name: "diehard::runs_down",
                p: 0.1452943633735131,
                note: "seq_len=10000, repeats=10, covariance-form",
            },
        ],
    );
}

#[test]
fn craps_both() {
    check(
        &diehard::craps::craps_both(&mut fresh()),
        &[
            Golden {
                name: "diehard::craps_wins",
                p: 0.6200184157746006,
                note: "games=200000, wins=98475, z=-0.4958",
            },
            Golden {
                name: "diehard::craps_throws",
                p: 0.04288633711188088,
                note: "games=200000, df=21, χ²=33.3104",
            },
        ],
    );
}

#[test]
fn craps() {
    check(
        &[diehard::craps::craps(&mut fresh())],
        &[Golden {
            name: "diehard::craps",
            p: 0.08577267422376177,
            note: "games=200000, wins=98475, p_wins=0.6200, p_throws=0.0429 (Bonferroni)",
        }],
    );
}

// ── DIEHARDER ─────────────────────────────────────────────────────────────────

#[test]
fn ks_uniform() {
    check(
        &[at_gate(dieharder::ks_uniform::ks_uniform, KS_UNIFORM_WORDS)],
        &[Golden {
            name: "dieharder::ks_uniform",
            p: 0.611698784770077,
            note: "tsamples=1000",
        }],
    );
}

#[test]
fn byte_distribution() {
    check(
        &[at_gate(
            dieharder::byte_distribution::byte_distribution,
            BYTE_DISTRIBUTION_WORDS,
        )],
        &[Golden {
            name: "dieharder::byte_distribution",
            p: 0.7253991774073503,
            note: "tsamples=1280, streams=9, expected/cell=5.0, χ²=2254.0000",
        }],
    );
}

#[test]
fn dct() {
    check(
        &[at_gate(dieharder::dct::dct, DCT_WORDS)],
        &[Golden {
            name: "dieharder::dct",
            p: 0.6771271531620839,
            note: "ntuple=256, tsamples=5000, χ²=244.1088",
        }],
    );
}

#[test]
fn lagged_sums() {
    let lag_1 = |w: &[u32]| dieharder::lagged_sums::lagged_sums(w, 1);
    let lag_100 = |w: &[u32]| dieharder::lagged_sums::lagged_sums(w, 100);
    check(
        &[at_gate(lag_1, 2 * 1_000), at_gate(lag_100, 101 * 1_000)],
        &[
            Golden {
                name: "dieharder::lagged_sums",
                p: 0.22120659094843653,
                note: "lag=1, tsamples=1000, sum=488.8326, z=-1.2233",
            },
            Golden {
                name: "dieharder::lagged_sums",
                p: 0.48115080327704657,
                note: "lag=100, tsamples=1000, sum=493.5693, z=-0.7045",
            },
        ],
    );
}

#[test]
fn monobit2() {
    check(
        &[at_gate(dieharder::monobit2::monobit2, MONOBIT2_WORDS)],
        &[Golden {
            name: "dieharder::monobit2",
            p: 0.5309875859302622,
            note: "tsamples=404, levels=1, block_sizes=2..2",
        }],
    );
}

#[test]
fn monobit2_two_block_sizes() {
    let words = words();
    let fewer = dieharder::monobit2::monobit2(&words[..MONOBIT2_TWO_LEVEL_WORDS - 1]);
    assert!(
        fewer
            .note
            .as_deref()
            .unwrap_or_default()
            .contains("levels=1"),
        "{fewer}"
    );
    check(
        &[dieharder::monobit2::monobit2(
            &words[..MONOBIT2_TWO_LEVEL_WORDS],
        )],
        &[Golden {
            name: "dieharder::monobit2",
            p: 0.07204798568266976,
            note: "tsamples=1140, levels=2, block_sizes=2..4",
        }],
    );
}

#[test]
fn fill_tree_both() {
    let words = words();
    let short = dieharder::fill_tree::fill_tree_both(&words[..FILL_TREE_WORDS - 1]);
    assert!(short.iter().all(TestResult::skipped), "{short:?}");
    check(
        &dieharder::fill_tree::fill_tree_both(&words[..FILL_TREE_WORDS]),
        &[
            Golden {
                name: "dieharder::fill_tree_count",
                p: 0.9183130678415875,
                note: "trials=100000, cells=11, χ²=4.5645",
            },
            Golden {
                name: "dieharder::fill_tree_position",
                p: 0.16416894356765557,
                note: "trials=100000, χ²=20.2061",
            },
        ],
    );
}

#[test]
fn fill_tree() {
    check(
        &[at_gate(dieharder::fill_tree::fill_tree, FILL_TREE_WORDS)],
        &[Golden {
            name: "dieharder::fill_tree",
            p: 0.32833788713531115,
            note: "p_fill=0.9183, p_pos=0.1642 (Bonferroni)",
        }],
    );
}

#[test]
fn bit_distribution_all() {
    let words = words();
    let fewer =
        dieharder::bit_distribution::bit_distribution_all(&words[..BIT_DISTRIBUTION_WORDS - 1], 8);
    assert!(fewer.len() < 510, "{} patterns scored", fewer.len());
    assert_eq!(
        dieharder::bit_distribution::bit_distribution_all(&words[..BIT_DISTRIBUTION_WORDS], 8)
            .len(),
        510
    );
    check(
        &dieharder::bit_distribution::bit_distribution_all(&words[..BIT_DISTRIBUTION_WORDS], 2),
        &[
            Golden {
                name: "dieharder::bit_distribution",
                p: 0.036755773526763784,
                note: "width=1, pattern=0, tsamples=728, bsamples=64, df=13, χ²=23.4338",
            },
            Golden {
                name: "dieharder::bit_distribution",
                p: 0.036755773526765415,
                note: "width=1, pattern=1, tsamples=728, bsamples=64, df=13, χ²=23.4338",
            },
            Golden {
                name: "dieharder::bit_distribution",
                p: 0.18220814856939846,
                note: "width=2, pattern=0, tsamples=364, bsamples=64, df=9, χ²=12.5869",
            },
            Golden {
                name: "dieharder::bit_distribution",
                p: 0.7911180556424718,
                note: "width=2, pattern=1, tsamples=364, bsamples=64, df=9, χ²=5.4747",
            },
            Golden {
                name: "dieharder::bit_distribution",
                p: 0.7606496630664719,
                note: "width=2, pattern=2, tsamples=364, bsamples=64, df=9, χ²=5.7909",
            },
            Golden {
                name: "dieharder::bit_distribution",
                p: 0.22095889779646985,
                note: "width=2, pattern=3, tsamples=364, bsamples=64, df=9, χ²=11.8662",
            },
        ],
    );
}

#[test]
fn bit_distribution() {
    check(
        &[dieharder::bit_distribution::bit_distribution(
            &words()[..BIT_DISTRIBUTION_WORDS],
            8,
        )],
        &[Golden {
            name: "dieharder::bit_distribution",
            p: 0.45470099177940826,
            note: "Bonferroni over 510 patterns; worst: width=8, pattern=229, tsamples=91, bsamples=64, df=1, χ²=11.0402",
        }],
    );
}

#[test]
fn minimum_distance_nd() {
    let results: Vec<TestResult> = (2..=5)
        .map(|d| dieharder::minimum_distance_nd::minimum_distance_nd(&mut fresh(), d, true))
        .collect();
    check(
        &results,
        &[
            Golden {
                name: "dieharder::minimum_distance_nd",
                p: 0.9347249730007391,
                note: "d=2, n=500, repeats=20",
            },
            Golden {
                name: "dieharder::minimum_distance_nd",
                p: 0.7143715432958859,
                note: "d=3, n=500, repeats=20",
            },
            Golden {
                name: "dieharder::minimum_distance_nd",
                p: 0.7540034288529496,
                note: "d=4, n=500, repeats=20",
            },
            Golden {
                name: "dieharder::minimum_distance_nd",
                p: 0.8702829953020814,
                note: "d=5, n=500, repeats=20",
            },
        ],
    );
}

#[test]
fn permutations() {
    check(
        &[dieharder::permutations::permutations(&mut fresh(), 5)],
        &[Golden {
            name: "dieharder::permutations",
            p: 0.8349574841841122,
            note: "t=5, n=100000, χ²=103.9832",
        }],
    );
}

#[test]
fn gcd_both() {
    let both = dieharder::gcd::gcd_both(&mut fresh());
    // `gcd` returns `gcd_both`'s first result, so it is checked against that
    // rather than pinned twice.
    let first = dieharder::gcd::gcd(&mut fresh());
    assert_eq!(first.p_value.to_bits(), both[0].p_value.to_bits());
    assert_eq!(first.note, both[0].note);
    check(
        &both,
        &[
            Golden {
                name: "dieharder::gcd_distribution",
                p: 0.6953495975870201,
                note: "pairs=100000, gtblsize=24, χ²=18.1786",
            },
            Golden {
                name: "dieharder::gcd_step_counts",
                p: 0.49140780973975207,
                note: "pairs=100000, χ²=25.4898",
            },
        ],
    );
}
