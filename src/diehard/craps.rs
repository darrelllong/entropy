//! DIEHARD Test 16 — Craps Test.
//!
//! Plays 200 000 games of craps using pairs of random integers as dice rolls.
//! Two statistics are tested:
//! 1. Number of wins: should be approximately normal with mean = 200 000 · p_win
//!    and σ² = 200 000 · p_win · (1 − p_win), where p_win = 244/495.
//! 2. Distribution of throws per game: throws can range from 1 to ∞; counts
//!    for 1..=21 (with ≥22 pooled) are tested with chi-square.
//!
//! Deliberate deviation from canonical DIEHARD: throws are binned into 22 cells
//! (1..=21 individually, ≥22 pooled) where Marsaglia pools everything above 21
//! into cell 21 (21 cells, df = 20).  The expected probabilities here are
//! derived analytically for this exact 22-cell layout, so the statistic is
//! self-consistent; it is simply one cell finer than the original.
//!
//! Each die is `1 + gsl_rng_uniform_int(rng, 6)`, as in Dieharder's
//! `diehard_craps.c`, so the high bits of each word pick the face (see
//! [`uniform_bounded`]).
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{
    math::{erfc, igamc},
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
/// sum 5 after an opening 4) would otherwise spin forever — the original
/// DIEHARD and Dieharder share that hang.  A capped game is recorded as a
/// loss in the ≥22-throw cell.  The cap exists to guarantee termination:
/// each capped game moves the win and throw statistics by one count, so a
/// stream that hits it rarely is judged by its other games, while one that
/// hits it in every game is rejected by those counts alone.
const MAX_THROWS: usize = 1000;

// Theoretical probabilities for number of throws in a craps game.
// P(throws = k) for k = 1..=21, P(throws ≥ 22) pooled into index 21.
// Derived from standard craps probability theory.

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

    // Test 2: throws-per-game chi-square against the analytical distribution.
    let expected = expected_throw_probs();
    let included = || {
        throw_counts
            .iter()
            .zip(expected.iter())
            .filter(|(_, &e)| e * N_GAMES as f64 >= 5.0)
    };
    let chi_sq: f64 = included()
        .map(|(&c, &e)| {
            let exp = e * N_GAMES as f64;
            (c as f64 - exp).powi(2) / exp
        })
        .sum();
    let df = included().count() - 1;
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
        ),
        TestResult::with_note(
            "diehard::craps_throws",
            o.p_throws,
            format!("games={N_GAMES}, df={}, χ²={:.4}", o.df, o.chi_sq),
        ),
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
/// Dieharder's `diehard_craps.c` rolls each die as
/// `1 + gsl_rng_uniform_int(rng, 6)`.  GSL's routine (GSL itself is not in
/// `pubs/`) divides the word by scale = ⌊range / bound⌋, with range = 2³² − 1
/// for a 32-bit generator, and redraws while the quotient reaches `bound`;
/// for dice, scale = 715 827 882 and the top four words are redrawn.
/// Marsaglia's `tests.txt` instead floats the word to [0, 1) and takes the
/// integer part of 6u, which is also a high-bit map and agrees with GSL's on
/// every word but a dozen next to the face boundaries and the four redrawn
/// ones.
fn uniform_bounded(rng: &mut impl Rng, bound: u32) -> u32 {
    let scale = u32::MAX / bound;
    // Bounded redraws.  An honest generator exhausts 16 retries with
    // probability (4/2³²)¹⁶ ≈ 10⁻¹⁴⁷; but a degenerate generator stuck in the
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
/// Derived from standard craps theory.  See e.g. Feller, *An Introduction
/// to Probability Theory and Its Applications*, Vol 1.
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

    /// Regression: `ConstantRng::new(u32::MAX)` emits only rejection-zone
    /// values; unbounded rejection sampling hung the battery forever here.
    /// The test must terminate and FAIL, not hang and not pass.
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

    /// GSL's divisor for a die: ⌊(2³² − 1) / 6⌋.
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

    /// Regression: `play_craps` looped forever on a stream that sets a point
    /// and never resolves it.  The game must be cut off at `MAX_THROWS` and
    /// scored as a loss.
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

    /// Faces from a Python replica of GSL's `gsl_rng_uniform_int(r, 6)` for a
    /// 32-bit generator: k = ⌊x / 715 827 882⌋.  Words 7 and 715 827 882 give
    /// k = 1 and k = 0 under the old low-bit `x % 6`.
    #[test]
    fn die_face_is_the_gsl_quotient() {
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

    /// Words 4 294 967 292..=u32::MAX give quotient 6 and are redrawn, as in
    /// GSL; a generator that never leaves that zone gets 16 redraws and then
    /// the clamped quotient.
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
