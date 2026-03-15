//! DIEHARD Test 16 — Craps Test.
//!
//! Plays 200 000 games of craps using pairs of random integers as dice rolls.
//! Two statistics are tested:
//! 1. Number of wins: should be approximately normal with mean = 200 000 · p_win
//!    and σ² = 200 000 · p_win · (1 − p_win), where p_win = 244/495.
//! 2. Distribution of throws per game: throws can range from 1 to ∞; counts
//!    for 1..=21 (with ≥22 pooled) are tested with chi-square.
//!
//! # Author
//! George Marsaglia, *DIEHARD: A Battery of Tests of Randomness* (1995).

use crate::{math::{erfc, igamc}, rng::Rng, result::TestResult};
use std::f64::consts::SQRT_2;

const N_GAMES: usize = 200_000;
const P_WIN: f64 = 244.0 / 495.0;

// Theoretical probabilities for number of throws in a craps game.
// P(throws = k) for k = 1..=21, P(throws ≥ 22) pooled into index 21.
// Derived from standard craps probability theory.

/// Run the craps test.
///
/// # Author
/// George Marsaglia, DIEHARD (1995).
pub fn craps(rng: &mut impl Rng) -> TestResult {
    let mut wins = 0usize;
    let mut throw_counts = [0u32; 22]; // index k-1 for k throws (≥22 pooled into index 21)

    for _ in 0..N_GAMES {
        let (won, throws) = play_craps(rng);
        if won { wins += 1; }
        let idx = (throws - 1).min(21);
        throw_counts[idx] += 1;
    }

    // Test 1: number of wins via normal approximation.
    let mu_w = N_GAMES as f64 * P_WIN;
    let sigma_w = (N_GAMES as f64 * P_WIN * (1.0 - P_WIN)).sqrt();
    let z_wins = (wins as f64 - mu_w) / sigma_w;
    let p_wins = erfc(z_wins.abs() / SQRT_2);

    // Test 2: throws-per-game chi-square.
    // Build expected probabilities from analytical craps theory.
    let expected = expected_throw_probs();
    let chi_sq: f64 = throw_counts
        .iter()
        .zip(expected.iter())
        .filter(|(_, &e)| e * N_GAMES as f64 >= 5.0)
        .map(|(&c, &e)| {
            let exp = e * N_GAMES as f64;
            (c as f64 - exp).powi(2) / exp
        })
        .sum();
    let df = throw_counts.iter().zip(expected.iter()).filter(|(_, &e)| e * N_GAMES as f64 >= 5.0).count() - 1;
    let p_throws = igamc(df as f64 / 2.0, chi_sq / 2.0);

    // Fisher's method: -2·(ln p₁ + ln p₂) ~ χ²(4); p = igamc(2, T/2).
    // min(p_wins, p_throws) was wrong — it inflates FPR to ~2% at α=0.01.
    let fisher = -2.0 * (p_wins.ln() + p_throws.ln());
    let p_value = igamc(2.0, fisher / 2.0);

    TestResult::with_note(
        "diehard::craps",
        p_value,
        format!("games={N_GAMES}, wins={wins}, p_wins={p_wins:.4}, p_throws={p_throws:.4}"),
    )
}

/// Run the craps test; returns the two p-values as separate `TestResult`s.
///
/// Test 1: normal-approximation test on win count.
/// Test 2: chi-square test on throws-per-game distribution.
pub fn craps_both(rng: &mut impl Rng) -> Vec<TestResult> {
    let mut wins = 0usize;
    let mut throw_counts = [0u32; 22];

    for _ in 0..N_GAMES {
        let (won, throws) = play_craps(rng);
        if won { wins += 1; }
        let idx = (throws - 1).min(21);
        throw_counts[idx] += 1;
    }

    let mu_w = N_GAMES as f64 * P_WIN;
    let sigma_w = (N_GAMES as f64 * P_WIN * (1.0 - P_WIN)).sqrt();
    let z_wins = (wins as f64 - mu_w) / sigma_w;
    let p_wins = erfc(z_wins.abs() / SQRT_2);

    let expected = expected_throw_probs();
    let chi_sq: f64 = throw_counts
        .iter()
        .zip(expected.iter())
        .filter(|(_, &e)| e * N_GAMES as f64 >= 5.0)
        .map(|(&c, &e)| {
            let exp = e * N_GAMES as f64;
            (c as f64 - exp).powi(2) / exp
        })
        .sum();
    let df = throw_counts.iter().zip(expected.iter())
        .filter(|(_, &e)| e * N_GAMES as f64 >= 5.0).count() - 1;
    let p_throws = igamc(df as f64 / 2.0, chi_sq / 2.0);

    vec![
        TestResult::with_note(
            "diehard::craps_wins",
            p_wins,
            format!("games={N_GAMES}, wins={wins}, z={z_wins:.4}"),
        ),
        TestResult::with_note(
            "diehard::craps_throws",
            p_throws,
            format!("games={N_GAMES}, df={df}, χ²={chi_sq:.4}"),
        ),
    ]
}

/// Play one craps game.  Returns (won, number_of_throws).
fn play_craps(rng: &mut impl Rng) -> (bool, usize) {
    let first = roll_dice(rng);
    let mut throws = 1;
    match first {
        7 | 11 => return (true, throws),
        2 | 3 | 12 => return (false, throws),
        point => loop {
            let r = roll_dice(rng);
            throws += 1;
            if r == point { return (true, throws); }
            if r == 7     { return (false, throws); }
        },
    }
}

fn roll_dice(rng: &mut impl Rng) -> u32 {
    let d1 = uniform_bounded(rng, 6) + 1;
    let d2 = uniform_bounded(rng, 6) + 1;
    d1 + d2
}

fn uniform_bounded(rng: &mut impl Rng, bound: u32) -> u32 {
    let zone = u32::MAX - (u32::MAX % bound);
    loop {
        let v = rng.next_u32();
        if v < zone {
            return v % bound;
        }
    }
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
    let points = [(4u32, 3.0/36.0), (5, 4.0/36.0), (6, 5.0/36.0),
                  (8, 5.0/36.0), (9, 4.0/36.0), (10, 3.0/36.0)];
    let p7 = 6.0 / 36.0;

    // Extend to k=200 so that P(≥22 throws) is fully accumulated into index 21.
    // The geometric tail decays exponentially; by k=200 the remaining probability
    // is negligible (< 10⁻⁴⁰ for the slowest-resolving point).
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
