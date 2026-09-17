//! DIEHARD craps test.
//!
//! Plays 200 000 games of craps, each die a uniform integer 1 to 6 from the
//! high bits of one word (see `uniform_bounded`).  Two statistics:
//!
//! 1. Wins: approximately normal with mean 200 000·p and variance
//!    200 000·p(1 − p), p = 244/495, scored two-sided, erfc(|z|/√2).
//! 2. Throws per game: counts for 1 to 21 throws, with 22 or more pooled,
//!    scored with a Pearson χ² against the exact distribution computed by
//!    `expected_throw_probs`, the long-game cells pooled until each expects at
//!    least 5 games.
//!
//! # Calibration
//!
//! 20 000 separately seeded PCG64 streams gave p < 0.01 in 1.025% of the wins
//! results and 0.950% of the throws results (binomial standard deviation
//! 0.070%), with Kolmogorov–Smirnov p-values of 0.20 and 0.16.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{
    math::{chi_square_pooled_tails, erfc, igamc},
    result::TestResult,
    rng::Rng,
};
use std::f64::consts::SQRT_2;

const N_GAMES: usize = 200_000;
const P_WIN: f64 = 244.0 / 495.0;

/// Throw cap per game.  Once a point is set, each throw resolves it with
/// probability at least 9/36 (point 4 or 10); the cap allows 999 point-phase
/// throws after the come-out roll, so an honest generator reaches it with
/// probability below (27/36)^999 ≈ 1.5 × 10⁻¹²⁵ per game.  A degenerate
/// generator that sets a point and then never rolls it or 7 (e.g. constant
/// sum 5 after an opening 4) would otherwise spin forever.  A capped game is
/// recorded as a loss in the ≥22-throw cell.  The cap exists to guarantee termination:
/// each capped game moves the win and throw statistics by one count, so a
/// stream that hits it rarely is judged by its other games, while one that
/// hits it in every game is rejected by those counts alone.
const MAX_THROWS: usize = 1000;

/// Outcome of one 200 000-game simulation, shared by both public entry points.
struct CrapsOutcome {
    wins: usize,
    z_wins: f64,
    p_wins: f64,
    chi_sq: f64,
    df: usize,
    p_throws: f64,
}

fn simulate(rng: &mut impl Rng) -> CrapsOutcome {
    let mut wins = 0usize;
    let mut throw_counts = [0u32; 22]; // index k-1 for k throws (≥22 pooled into index 21)

    for _ in 0..N_GAMES {
        let (won, throws) = play_craps(rng);
        if won {
            wins += 1;
        }
        let idx = (throws - 1).min(21);
        throw_counts[idx] += 1;
    }

    // Test 1: number of wins via normal approximation.
    let mu_w = N_GAMES as f64 * P_WIN;
    let sigma_w = (N_GAMES as f64 * P_WIN * (1.0 - P_WIN)).sqrt();
    let z_wins = (wins as f64 - mu_w) / sigma_w;
    let p_wins = erfc(z_wins.abs() / SQRT_2);

    // Test 2: throws-per-game chi-square against the exact distribution, the
    // long-game tail pooled until each cell expects at least 5 games.
    let expected: Vec<f64> = expected_throw_probs()
        .iter()
        .map(|&p| p * N_GAMES as f64)
        .collect();
    let observed: Vec<f64> = throw_counts.iter().map(|&c| f64::from(c)).collect();
    let (chi_sq, df) = chi_square_pooled_tails(&observed, &expected, 5.0)
        .expect("the throw law has many cells expecting 5 games");
    let p_throws = igamc(df as f64 / 2.0, chi_sq / 2.0);

    CrapsOutcome {
        wins,
        z_wins,
        p_wins,
        chi_sq,
        df,
        p_throws,
    }
}

/// Run the craps test, reporting a single combined p-value.
///
/// The wins and throws statistics come from the *same* games, so independence-
/// based combinations (Fisher, Šidák) do not apply; the two p-values are folded
/// with a Bonferroni bound, which is valid under arbitrary dependence.
/// Prefer [`craps_both`] (used by `run_all`) for the uncombined report.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn craps(rng: &mut impl Rng) -> TestResult {
    let o = simulate(rng);
    let p_value = (2.0 * o.p_wins.min(o.p_throws)).min(1.0);
    TestResult::with_note(
        "diehard::craps",
        p_value,
        format!(
            "games={N_GAMES}, wins={}, p_wins={:.4}, p_throws={:.4} (Bonferroni)",
            o.wins, o.p_wins, o.p_throws
        ),
    )
    .with_statistic(
        "smaller p-value",
        o.p_wins.min(o.p_throws),
        None,
        "Bonferroni bound",
    )
}

/// Run the craps test; returns the two p-values as separate `TestResult`s.
///
/// Test 1: normal-approximation test on win count.
/// Test 2: chi-square test on throws-per-game distribution.
pub fn craps_both(rng: &mut impl Rng) -> Vec<TestResult> {
    let o = simulate(rng);
    vec![
        TestResult::with_note(
            "diehard::craps_wins",
            o.p_wins,
            format!("games={N_GAMES}, wins={}, z={:.4}", o.wins, o.z_wins),
        )
        .normal(o.z_wins),
        TestResult::with_note(
            "diehard::craps_throws",
            o.p_throws,
            format!("games={N_GAMES}, df={}, χ²={:.4}", o.df, o.chi_sq),
        )
        .chi_square(o.chi_sq, o.df as f64),
    ]
}

/// Play one craps game.  Returns (won, number_of_throws).
fn play_craps(rng: &mut impl Rng) -> (bool, usize) {
    let first = roll_dice(rng);
    let mut throws = 1;
    match first {
        7 | 11 => (true, throws),
        2 | 3 | 12 => (false, throws),
        point => {
            while throws < MAX_THROWS {
                let r = roll_dice(rng);
                throws += 1;
                if r == point {
                    return (true, throws);
                }
                if r == 7 {
                    return (false, throws);
                }
            }
            (false, throws)
        }
    }
}

fn roll_dice(rng: &mut impl Rng) -> u32 {
    let d1 = uniform_bounded(rng, 6) + 1;
    let d2 = uniform_bounded(rng, 6) + 1;
    d1 + d2
}

/// Uniform integer in `0..bound` from the high bits of one word.
///
/// With scale = ⌊(2³² − 1)/bound⌋, the value is ⌊x/scale⌋, and a word whose
/// quotient reaches `bound` is redrawn.  Every accepted quotient is hit by
/// exactly `scale` words, so the result is exactly uniform; for dice the top
/// four words are redrawn.
fn uniform_bounded(rng: &mut impl Rng, bound: u32) -> u32 {
    let scale = u32::MAX / bound;
    // Bounded redraws.  An honest generator exhausts 16 retries with
    // probability (4/2³²)¹⁶ ≈ 3 × 10⁻¹⁴⁵; but a degenerate generator stuck in the
    // redraw zone (e.g. `ConstantRng::new(u32::MAX)`) must not hang the
    // battery — fall through to the clamped quotient and let the statistics
    // fail it.
    for _ in 0..16 {
        let k = rng.next_u32() / scale;
        if k < bound {
            return k;
        }
    }
    (rng.next_u32() / scale).min(bound - 1)
}

/// Exact P(game takes exactly k throws) for k = 1..=22 (k=22 means ≥22).
///
/// A game ends on the first throw with probability 12/36.  Otherwise it sets
/// point x with probability pₓ and ends on each later throw with probability
/// pₓ + p₇, so P(k throws) = Σₓ pₓ(1 − pₓ − p₇)^(k−2)(pₓ + p₇) for k ≥ 2.
fn expected_throw_probs() -> [f64; 22] {
    let mut p = [0.0f64; 22];

    // P(win on throw 1) + P(loss on throw 1) = P(game ends on throw 1).
    // P(7 or 11) = 8/36, P(2,3,12) = 4/36.  So P(end on throw 1) = 12/36 = 1/3.
    p[0] = 12.0 / 36.0;

    // For each point value x in {4,5,6,8,9,10}, the probability the game ends
    // on exactly throw k ≥ 2 is:
    //   P(establish x on throw 1) · P(x or 7 resolved on throw 2..k) · P(end on throw k)
    //
    // P(establish x) · [p_x(1−(p_x+p_7))^(k−2)] · (p_x + p_7)  for k ≥ 2
    // where p_x = P(roll = x), p_7 = 6/36.
    //
    // Summing over all point values x:
    let points = [
        (4u32, 3.0 / 36.0),
        (5, 4.0 / 36.0),
        (6, 5.0 / 36.0),
        (8, 5.0 / 36.0),
        (9, 4.0 / 36.0),
        (10, 3.0 / 36.0),
    ];
    let p7 = 6.0 / 36.0;

    // Extend to k=200 so that P(≥22 throws) is accumulated into index 21.
    // The mass left beyond k = 200 is Σₓ pₓ·(1 − pₓ − p₇)¹⁹⁹ ≈ 2.3 × 10⁻²⁶,
    // nearly all from the slowest-resolving points 4 and 10 (each
    // 3/36 · (27/36)¹⁹⁹ ≈ 1.1 × 10⁻²⁶): far below f64 resolution, so the
    // renormalisation below only mops up rounding.
    for k in 2usize..=200 {
        let mut prob = 0.0f64;
        for &(_x, px) in &points {
            // P(establish x) = px; given established, P(resolve on next roll) = px + p7.
            // P(game takes exactly k throws | point = x)
            //   = (1 − px − p7)^(k−2) · (px + p7)
            let resolve = px + p7;
            let stay: f64 = 1.0 - resolve;
            let prob_resolve = stay.powi(k as i32 - 2) * resolve;
            prob += px * prob_resolve;
        }
        // k=1..=21 → individual cells; k≥22 pooled into index 21.
        let idx = if k <= 22 { k - 1 } else { 21 };
        p[idx] += prob;
    }

    // Normalise to sum to 1 (handle floating-point accumulation drift).
    let sum: f64 = p.iter().sum();
    p.iter_mut().for_each(|v| *v /= sum);
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::ConstantRng;

    /// `ConstantRng::new(u32::MAX)` emits only rejection-zone values; the
    /// bounded redraw lets the test terminate and FAIL.
    #[test]
    fn craps_terminates_and_fails_on_stuck_high_generator() {
        let mut rng = ConstantRng::new(u32::MAX);
        for r in craps_both(&mut rng) {
            assert!(!r.skipped(), "{r}");
            assert!(r.p_value < 1e-10, "{r}");
        }
    }

    /// Opens every game with dice (1,3) = point 4, then rolls (1,4) = 5
    /// forever: the point is never made and 7 never appears.
    struct StuckPoint {
        throws: usize,
    }

    /// The divisor for a die: ⌊(2³² − 1) / 6⌋.
    const DIE_SCALE: u32 = u32::MAX / 6;

    impl Rng for StuckPoint {
        fn next_u32(&mut self) -> u32 {
            // Two draws per throw; the word k·scale is face k + 1.
            let die = self.throws % 2;
            let value = if self.throws < 2 {
                [0, 2 * DIE_SCALE][die] // first throw: 1 + 3 = 4
            } else {
                [0, 3 * DIE_SCALE][die] // every later throw: 1 + 4 = 5
            };
            self.throws += 1;
            value
        }
    }

    /// A stream that sets a point and never resolves it is cut off at
    /// `MAX_THROWS` and scored as a loss.
    #[test]
    fn play_craps_is_capped_on_unresolvable_point() {
        let mut rng = StuckPoint { throws: 0 };
        let (won, throws) = play_craps(&mut rng);
        assert!(!won);
        assert_eq!(throws, MAX_THROWS);
    }

    #[test]
    fn craps_terminates_and_fails_on_stuck_point_generator() {
        // Game 1 hits the cap; every later game opens with 5, sets point 5,
        // and makes it on the next throw (a two-throw win).  The whole run
        // must terminate, and both statistics must reject the stream.
        let mut rng = StuckPoint { throws: 0 };
        let results = craps_both(&mut rng);
        assert!(results.iter().all(|r| !r.skipped()));
        assert!(results.iter().all(|r| r.p_value < 1e-10), "{results:?}");
    }

    /// Plays back a fixed list of words and counts the draws.
    struct Script {
        words: &'static [u32],
        draws: usize,
    }

    impl Rng for Script {
        fn next_u32(&mut self) -> u32 {
            let word = self.words[self.draws];
            self.draws += 1;
            word
        }
    }

    /// The face is k = ⌊x / 715 827 882⌋, from the high bits: words 7 and
    /// 715 827 882 would give 1 and 0 from the low bits, `x % 6`.
    #[test]
    fn die_face_is_the_high_bit_quotient() {
        assert_eq!(DIE_SCALE, 715_827_882);
        let cases = [
            (0, 0),
            (7, 0),
            (715_827_881, 0),
            (715_827_882, 1),
            (2_147_483_647, 3),
            (2_147_483_648, 3),
            (4_294_967_291, 5),
        ];
        for (word, face) in cases {
            assert_eq!(
                uniform_bounded(&mut ConstantRng::new(word), 6),
                face,
                "word {word}"
            );
        }
    }

    /// Words 4 294 967 292..=u32::MAX give quotient 6 and are redrawn; a
    /// generator that never leaves that zone gets 16 redraws and then the
    /// clamped quotient.
    #[test]
    fn die_redraws_the_top_words() {
        let mut rng = Script {
            words: &[4_294_967_292, u32::MAX, 2_147_483_648],
            draws: 0,
        };
        assert_eq!(uniform_bounded(&mut rng, 6), 3);
        assert_eq!(rng.draws, 3);

        let mut stuck = Script {
            words: &[u32::MAX; 17],
            draws: 0,
        };
        assert_eq!(uniform_bounded(&mut stuck, 6), 5);
        assert_eq!(stuck.draws, 17);
    }

    #[test]
    fn uniform_bounded_is_in_range_and_unbiased_zone() {
        let mut rng = ConstantRng::new(u32::MAX);
        let v = uniform_bounded(&mut rng, 6);
        assert!(v < 6);
    }

    #[test]
    fn expected_throw_probs_sum_to_one() {
        let p = expected_throw_probs();
        let sum: f64 = p.iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
        // P(end on throw 1) = 12/36.
        assert!((p[0] - 1.0 / 3.0).abs() < 1e-12);
    }
}
