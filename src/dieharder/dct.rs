//! DIEHARDER discrete cosine transform test.
//!
//! Each block of N = 256 words, read as numbers, gets the unnormalised DCT-II
//! X\[k\] = Σⱼ x\[j\]·cos(π(j + ½)k/N).  The DC coefficient is centred by its mean,
//! N·(2³¹ − ½), and divided by √2 so that all coefficients have the same
//! variance; the position of the largest |X\[k\]| should then be uniform over
//! 0 … N − 1.  5 000 blocks give a Pearson χ² on the position counts, df 255.
//! The words are rotated left by 0, 8, 16 and 24 bits in the four quarters of
//! the blocks, so that each byte takes a turn in the most significant place.
//!
//! The coefficients come from one fast Fourier transform per block.  With y
//! the even extension of the block to length 2N, y\[j\] = y\[2N − 1 − j\] = x\[j\],
//! its transform Y\[k\] = Σⱼ y\[j\]·e^(−2πijk/2N) satisfies
//! X\[k\] = Re(e^(−iπk/2N)·Y\[k\])/2 (J. Makhoul, "A fast cosine transform in
//! one and multiple dimensions", *IEEE Transactions on Acoustics, Speech, and
//! Signal Processing* 28(1), pp. 27–34, 1980).  The two computations round
//! differently, which could move the maximum only between coefficients equal
//! to within rounding; the tests check that the position of the maximum
//! agrees with the direct sums on every block they try.
//!
//! # Calibration
//!
//! 100 000 null runs on separately seeded xoshiro256** streams rejected at
//! 0.05 in 5.106%, at 0.01 in 1.041% (binomial standard deviation 0.031%) and
//! at 0.001 in 0.127% (0.010%), with a Kolmogorov–Smirnov p-value of 0.15.
//! Equal variances make the position of the maximum close to uniform, not
//! exactly uniform: the coefficients' higher cumulants differ with k, and the
//! excess at 0.001 is resolved at this size.
//!
//! # Author
//! David Bauer, in Robert G. Brown's *Dieharder: A Random Number Test Suite*
//! (2006).

use crate::{math::igamc, result::TestResult};
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::{f64::consts::PI, sync::Arc};

/// Block length (ntuple), must be a power of 2.
const NTUPLE: usize = 256;
/// Number of blocks: 19.5 expected per position.
const TSAMPLES: usize = 5_000;
/// Bit width of each generator word (rmax_bits = 32 for u32 output).
const RMAX_BITS: u32 = 32;

/// Run the DCT spectral test.
///
/// # Author
/// David Bauer, Dieharder (2006).
pub fn dct(words: &[u32]) -> TestResult {
    let needed = TSAMPLES * NTUPLE;
    if words.len() < needed {
        return TestResult::insufficient("dieharder::dct", "not enough words");
    }

    // v = 2^(rmax_bits−1).  DC mean for a block of N uniform u32 values is
    // N·(v − 0.5) since E[U32] ≈ 2^31 − 0.5.
    let v = 1u64 << (RMAX_BITS - 1);
    let mean_dc = NTUPLE as f64 * (v as f64 - 0.5);

    // position_counts[k]: blocks whose largest |X[k]| is at position k.
    let mut position_counts = vec![0u64; NTUPLE];
    let mut transform = FastDct::new();

    for j in 0..TSAMPLES {
        // The rotation grows by a quarter word every quarter of the blocks.
        let rot_amount = ((j / (TSAMPLES / 4)) as u32 * (RMAX_BITS / 4)) % RMAX_BITS;
        let block = &words[j * NTUPLE..(j + 1) * NTUPLE];
        let coefficients = transform.dct_ii(block, rot_amount);
        position_counts[max_position(coefficients, mean_dc)] += 1;
    }

    // Chi-square for uniformity of position counts.
    // Expected count per position = TSAMPLES / NTUPLE.
    let expected = TSAMPLES as f64 / NTUPLE as f64;
    let chi_sq: f64 = position_counts
        .iter()
        .map(|&c| (c as f64 - expected).powi(2) / expected)
        .sum();
    let df = NTUPLE - 1;

    let p_value = igamc(df as f64 / 2.0, chi_sq / 2.0);

    TestResult::with_note(
        "dieharder::dct",
        p_value,
        format!("ntuple={NTUPLE}, tsamples={TSAMPLES}, χ²={chi_sq:.4}"),
    )
    .chi_square(chi_sq, df as f64)
}

/// The rotated words of a block as numbers.
fn rotated(words: &[u32], rot_amount: u32) -> impl Iterator<Item = f64> + '_ {
    words
        .iter()
        .map(move |&w| f64::from(w.rotate_left(rot_amount)))
}

/// Position of the largest |X\[k\]| after the DC coefficient is centred by
/// `mean_dc` and divided by √2.  Of equal magnitudes the last wins.
fn max_position(coefficients: &mut [f64], mean_dc: f64) -> usize {
    coefficients[0] -= mean_dc;
    coefficients[0] /= 2f64.sqrt();
    coefficients
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.abs().total_cmp(&b.abs()))
        .map_or(0, |(i, _)| i)
}

/// The DCT-II of `NTUPLE` words through one FFT of length 2·`NTUPLE`.
struct FastDct {
    fft: Arc<dyn Fft<f64>>,
    buffer: Vec<Complex<f64>>,
    scratch: Vec<Complex<f64>>,
    /// e^(−iπk/2N) for k = 0 … N − 1.
    phases: Vec<Complex<f64>>,
    coefficients: Vec<f64>,
}

impl FastDct {
    fn new() -> Self {
        let fft = FftPlanner::new().plan_fft_forward(2 * NTUPLE);
        let scratch = vec![Complex::default(); fft.get_inplace_scratch_len()];
        let phases = (0..NTUPLE)
            .map(|k| Complex::from_polar(1.0, -PI * k as f64 / (2 * NTUPLE) as f64))
            .collect();
        Self {
            fft,
            buffer: vec![Complex::default(); 2 * NTUPLE],
            scratch,
            phases,
            coefficients: vec![0.0; NTUPLE],
        }
    }

    /// X\[k\] = Σⱼ x\[j\]·cos(π(j + ½)k/N) of the rotated words, k = 0 … N − 1.
    fn dct_ii(&mut self, words: &[u32], rot_amount: u32) -> &mut [f64] {
        for (j, x) in rotated(words, rot_amount).enumerate() {
            self.buffer[j] = Complex::new(x, 0.0);
            self.buffer[2 * NTUPLE - 1 - j] = Complex::new(x, 0.0);
        }
        self.fft
            .process_with_scratch(&mut self.buffer, &mut self.scratch);
        for ((c, y), phase) in self
            .coefficients
            .iter_mut()
            .zip(&self.buffer)
            .zip(&self.phases)
        {
            *c = (phase * y).re / 2.0;
        }
        &mut self.coefficients
    }
}

#[cfg(test)]
mod tests {
    use super::{dct, max_position, rotated, FastDct, NTUPLE, RMAX_BITS, TSAMPLES};
    use crate::rng::{Mt19937, Pcg32, Rng, Xorshift32};
    use std::f64::consts::PI;

    /// The direct sums X[k] = Σⱼ x[j]·cos(π(j + ½)k/N).
    fn direct_dct(words: &[u32], rot_amount: u32) -> Vec<f64> {
        let x: Vec<f64> = rotated(words, rot_amount).collect();
        (0..NTUPLE)
            .map(|k| {
                x.iter()
                    .enumerate()
                    .map(|(j, &xj)| xj * ((j as f64 + 0.5) * k as f64 * PI / NTUPLE as f64).cos())
                    .sum()
            })
            .collect()
    }

    /// The fast coefficients agree with the direct sums to rounding, and the
    /// position of the maximum is the same, on generator output in every
    /// rotation and on structured blocks.
    #[test]
    fn fast_transform_matches_direct_sums() {
        let mean_dc = NTUPLE as f64 * ((1u64 << (RMAX_BITS - 1)) as f64 - 0.5);
        let mut blocks: Vec<Vec<u32>> = Vec::new();
        let mut mt = Mt19937::new(1);
        let mut pcg = Pcg32::new(42, 54);
        let mut xs = Xorshift32::new(2_463_534_242);
        for _ in 0..700 {
            blocks.push(mt.collect_u32s(NTUPLE));
            blocks.push(pcg.collect_u32s(NTUPLE));
            blocks.push(xs.collect_u32s(NTUPLE));
        }
        blocks.push(vec![0; NTUPLE]);
        blocks.push(vec![u32::MAX; NTUPLE]);
        blocks.push(vec![1 << 31; NTUPLE]);
        blocks.push((0..NTUPLE as u32).collect());
        blocks.push(
            (0..NTUPLE)
                .map(|j| if j % 2 == 0 { u32::MAX } else { 0 })
                .collect(),
        );
        blocks.push(
            (0..NTUPLE)
                .map(|j| (j as u32).wrapping_mul(0x9e37_79b9))
                .collect(),
        );
        let mut fast = FastDct::new();
        for (b, block) in blocks.iter().enumerate() {
            for rot in [0, 8, 16, 24] {
                let mut direct = direct_dct(block, rot);
                let quick = fast.dct_ii(block, rot).to_vec();
                // Both sums round at the scale of N·2³², the largest a
                // coefficient can be.
                let scale = NTUPLE as f64 * 4_294_967_296.0;
                for (k, (d, q)) in direct.iter().zip(&quick).enumerate() {
                    assert!(
                        (d - q).abs() <= 1e-13 * scale,
                        "block {b} rot {rot} k {k}: {d} vs {q}"
                    );
                }
                let mut quick = quick;
                assert_eq!(
                    max_position(&mut direct, mean_dc),
                    max_position(&mut quick, mean_dc),
                    "block {b} rot {rot}"
                );
            }
        }
    }

    #[test]
    fn short_inputs_skip() {
        assert!(dct(&[]).skipped());
        assert!(dct(&vec![0; TSAMPLES * NTUPLE - 1]).skipped());
    }

    /// Only the adjusted DC coefficient is nonzero, so every block's maximum
    /// sits at position 0.
    #[test]
    fn constant_input_fails() {
        let r = dct(&vec![0; TSAMPLES * NTUPLE]);
        assert!(!r.skipped() && !r.passed(), "{r}");
    }
}
