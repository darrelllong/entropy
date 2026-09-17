//! The distribution of the position of the largest adjusted DCT coefficient
//! under the null, for the DCT test's table.
//!
//! `dct_position_table <chunks per thread> <threads> <first stream>` counts
//! positions over chunks of 5 000 blocks, as the test reads them, on PCG64
//! streams `Pcg64::new(0x6463_745f_706f_7369, first + t)` for thread t.
//! Output: `blocks=… counts=c0,c1,…,c255`.

use entropy::{
    dieharder::dct::{position_counts, BLOCK_WORDS},
    rng::{Pcg64, Rng},
};
use std::thread;

/// Blocks per chunk, the test's own sample size.
const CHUNK_BLOCKS: usize = 5_000;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let chunks: u64 = args[1].parse().expect("chunks per thread");
    let threads: u64 = args[2].parse().expect("threads");
    let first: u64 = args[3].parse().expect("first stream");
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            thread::spawn(move || {
                let mut rng = Pcg64::new(0x6463_745f_706f_7369, u128::from(first + t));
                let mut counts = [0u64; BLOCK_WORDS];
                let mut words = vec![0u32; CHUNK_BLOCKS * BLOCK_WORDS];
                for _ in 0..chunks {
                    for w in &mut words {
                        *w = rng.next_u32();
                    }
                    for (total, c) in counts.iter_mut().zip(position_counts(&words)) {
                        *total += c;
                    }
                }
                counts
            })
        })
        .collect();
    let mut counts = [0u64; BLOCK_WORDS];
    for h in handles {
        for (total, c) in counts.iter_mut().zip(h.join().expect("worker")) {
            *total += c;
        }
    }
    let blocks: u64 = counts.iter().sum();
    let list: Vec<String> = counts.iter().map(u64::to_string).collect();
    println!("blocks={blocks} counts={}", list.join(","));
}
