//! Derives the cell probabilities of the DIEHARD squeeze test.
//!
//! Evaluates [`step_count_distribution`] at k = 2³¹ − 1 (about six minutes in
//! a release build) and prints the 43 cells the test scores, j ≤ 6, 7 … 47
//! and 48, as the `CELL_PROBABILITIES` array of `src/diehard/squeeze.rs`.
//!
//! Run with `cargo run --release --example squeeze_table`.

use entropy::diehard::squeeze::{cells_from_steps, step_count_distribution};

fn main() {
    let cells = cells_from_steps(&step_count_distribution((1 << 31) - 1, 48));
    println!("pub const CELL_PROBABILITIES: [f64; 43] = [");
    for p in cells {
        println!("    {p:.17e},");
    }
    println!("];");
}
