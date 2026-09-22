//! Positioning a generator within its output: one interface for every
//! deterministic generator.
//!
//! A *step* is one call of [`Rng::next_u32`](super::Rng::next_u32).  A
//! `next_u64` is one step on the 64-bit generators, which give `next_u32`
//! the high half of a word, and two steps on the 32-bit and byte-backed
//! ones; each generator's [`Rng`](super::Rng) documentation says which.
//!
//! [`Advance`] moves a generator forward by any number of steps in time
//! logarithmic in that number, where the generator's structure allows it:
//! the linear generators (xoshiro, xoroshiro, xorshift, the Mersenne
//! Twister) through a polynomial derived from the generator, the PCG family
//! through the composition of its affine state map, and ChaCha20 by setting
//! its block counter.  SFC64 and JSF64 are chaotic maps with no such
//! structure and do not implement it.
//!
//! [`Streams`] gives a numbered family of generators for parallel work: the
//! same seed and index always produce the same generator, whatever order the
//! workers run in.  For the generators that implement [`Advance`], stream k
//! is segment k of one sequence, the segments consecutive and disjoint with
//! the length the implementation states; stream 0 starts where the seed does.
//! For SFC64 and JSF64 the streams are separate sequences whose seeds are
//! derived from the generator's state and the index, distinct but with no
//! proof that they do not overlap, and the implementation says so.

/// Move forward by a number of steps without taking them one at a time.
pub trait Advance {
    /// Advance by `steps` calls of `next_u32`, in time logarithmic in
    /// `steps`.
    ///
    /// # Panics
    /// Panics where the generator's output would run out: ChaCha20 past its
    /// 2³² blocks per nonce.
    fn advance(&mut self, steps: u128);
}

/// A numbered family of generators from one seed.
pub trait Streams: Sized {
    /// The generator for stream `index`.
    fn stream(&self, index: u64) -> Self;
}
