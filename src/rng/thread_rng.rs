//! A fast-key-erasure generator, and a per-thread instance seeded and
//! reseeded from the operating system.
//!
//! [`FastKeyErasureRng`] follows D. J. Bernstein, "Fast-key-erasure
//! random-number generators," 23 July 2017,
//! <https://blog.cr.yp.to/20170723-random.html>.  Its state is a 256-bit
//! ChaCha20 key.  Each refill computes 512 bytes of ChaCha20 keystream under
//! that key with an all-zero nonce, replaces the key with the first 32 bytes,
//! and serves the other 480; each byte is overwritten with zero as it is
//! served.  Someone who later reads the whole state learns the key for future
//! blocks and the unserved bytes, but nothing about output already served:
//! the key that produced it is gone, and recovering it from its successor
//! would break ChaCha20.
//!
//! [`thread_rng`] gives each thread one such generator, keyed from
//! [`os_random`].  It mixes in a fresh key from the operating system after
//! every 2³⁰ bytes of output, and whenever the process id differs from the
//! one it was seeded under, so a forked child never repeats its parent's
//! stream.

use super::{os::os_random, Rng, Seedable};
use cryptography::{cprng::fast_key_erasure::KEY, zeroize_slice, FastKeyErasure};
use std::{cell::RefCell, io, marker::PhantomData};

/// Output bytes after which a thread's generator takes a fresh key.
const RESEED_BYTES: u64 = 1 << 30;

/// ChaCha20 with fast key erasure: cryptography's `FastKeyErasure`, whose
/// refill is `REFILL` (512) keystream bytes of which the first `KEY` (32)
/// become the next key.
pub struct FastKeyErasureRng(FastKeyErasure);

impl FastKeyErasureRng {
    /// A generator whose first key is `key`.  The caller's copy is not wiped.
    #[must_use]
    pub fn new(key: [u8; KEY]) -> Self {
        Self(FastKeyErasure::new(key))
    }

    /// Serve `bytes`, erasing each from the generator's buffer as it goes.
    pub fn fill(&mut self, bytes: &mut [u8]) {
        self.0.fill(bytes);
    }

    /// XOR `fresh` into the key and discard the unserved buffer.
    fn reseed(&mut self, fresh: &[u8; KEY]) {
        self.0.reseed(fresh);
    }
}

impl Rng for FastKeyErasureRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    /// Straight from the keystream: whole words as the buffer serves them,
    /// and a shorter final chunk taking one more word's low bytes.
    fn fill_native(&mut self, bytes: &mut [u8]) {
        let whole = bytes.len() - bytes.len() % size_of::<u64>();
        let (words, rest) = bytes.split_at_mut(whole);
        self.0.fill(words);
        if !rest.is_empty() {
            let mut word = [0u8; size_of::<u64>()];
            self.0.fill(&mut word);
            rest.copy_from_slice(&word[..rest.len()]);
        }
    }
}

/// A thread's generator and what decides when it reseeds.
struct ThreadState {
    rng: FastKeyErasureRng,
    served: u64,
    pid: u32,
    seedings: u64,
}

thread_local! {
    static STATE: RefCell<Option<ThreadState>> = const { RefCell::new(None) };
}

/// Run `f` on this thread's generator, charging `bytes` of output, after
/// seeding it if it has none, if `pid` is not the one it was seeded under, or
/// if it has served [`RESEED_BYTES`].  A scalar draw is charged after the
/// check, so it can carry the count up to 7 bytes past the limit before the
/// next call reseeds; [`ThreadRng::try_fill`] splits a longer request at the
/// limit, so no bulk request crosses it.
fn with_generator<T>(
    pid: u32,
    bytes: u64,
    f: impl FnOnce(&mut FastKeyErasureRng) -> T,
) -> io::Result<T> {
    STATE.with(|cell| {
        let mut slot = cell.borrow_mut();
        let fresh_process = slot.as_ref().is_none_or(|s| s.pid != pid);
        if fresh_process {
            let seedings = slot.as_ref().map_or(0, |s| s.seedings);
            *slot = Some(ThreadState {
                rng: FastKeyErasureRng::try_from_os_rng()?,
                served: 0,
                pid,
                seedings: seedings + 1,
            });
        }
        let state = slot.as_mut().expect("seeded above");
        if state.served >= RESEED_BYTES {
            let mut key = [0u8; KEY];
            os_random(&mut key)?;
            state.rng.reseed(&key);
            zeroize_slice(&mut key);
            state.served = 0;
            state.seedings += 1;
        }
        state.served += bytes;
        Ok(f(&mut state.rng))
    })
}

/// What is left of this thread's reseed interval, in bytes, or
/// [`RESEED_BYTES`] if it has no generator yet.
fn until_reseed() -> u64 {
    STATE.with(|cell| {
        cell.borrow()
            .as_ref()
            .map_or(RESEED_BYTES, |s| RESEED_BYTES - s.served.min(RESEED_BYTES))
    })
}

/// A handle to this thread's generator.  It cannot leave the thread.
#[derive(Clone)]
pub struct ThreadRng {
    _not_send: PhantomData<*const ()>,
}

/// This thread's generator, seeded from the operating system on first use.
///
/// # Panics
/// Panics if the operating system's entropy source fails; see
/// [`try_thread_rng`].
#[must_use]
pub fn thread_rng() -> ThreadRng {
    try_thread_rng().expect(OS_FAILED)
}

/// This thread's generator, with any failure to seed it reported.
///
/// # Errors
/// Any error from [`os_random`].
pub fn try_thread_rng() -> io::Result<ThreadRng> {
    with_generator(std::process::id(), 0, |_| ())?;
    Ok(ThreadRng {
        _not_send: PhantomData,
    })
}

impl ThreadRng {
    /// Fill `bytes` from this thread's generator: the thread-local lookup,
    /// process-id check and reseed check happen once per request rather than
    /// once per word, and the whole request is charged against the reseed
    /// interval.
    ///
    /// The bytes are the generator's keystream in order, the same sequence
    /// `next_u32` would deliver little-endian, and a request that would cross
    /// the reseed limit is split there, so no byte is served past it.
    ///
    /// # Errors
    /// Any error from [`os_random`], from seeding or from a reseed.  On a
    /// failure the bytes already written stay written and the rest are
    /// untouched; the generator is left usable and the next call retries.
    pub fn try_fill(&mut self, bytes: &mut [u8]) -> io::Result<()> {
        let pid = std::process::id();
        let mut rest = bytes;
        while !rest.is_empty() {
            let take = usize::try_from(until_reseed().max(1))
                .unwrap_or(usize::MAX)
                .min(rest.len());
            let (now, later) = rest.split_at_mut(take);
            with_generator(pid, take as u64, |rng| rng.fill(now))?;
            rest = later;
        }
        Ok(())
    }

    /// Fill `bytes`, as [`try_fill`](Self::try_fill).
    ///
    /// # Panics
    /// Panics if the operating system's entropy source fails.
    pub fn fill(&mut self, bytes: &mut [u8]) {
        self.try_fill(bytes).expect(OS_FAILED);
    }

    /// One word, with a seeding or reseeding failure reported.
    ///
    /// # Errors
    /// Any error from [`os_random`].
    pub fn try_next_u64(&mut self) -> io::Result<u64> {
        let mut word = [0u8; size_of::<u64>()];
        self.try_fill(&mut word)?;
        Ok(u64::from_le_bytes(word))
    }
}

/// What a panicking `ThreadRng` call reports.
const OS_FAILED: &str = "thread_rng: the operating system's entropy source failed";

impl Rng for ThreadRng {
    fn next_u32(&mut self) -> u32 {
        let charge = size_of::<u32>() as u64;
        with_generator(std::process::id(), charge, FastKeyErasureRng::next_u32).expect(OS_FAILED)
    }

    fn next_u64(&mut self) -> u64 {
        let charge = size_of::<u64>() as u64;
        with_generator(std::process::id(), charge, FastKeyErasureRng::next_u64).expect(OS_FAILED)
    }

    /// [`ThreadRng::fill`] for the whole words, then one more word for a
    /// shorter final chunk, whose remaining bytes are discarded as the
    /// [`Rng::fill_native`] contract requires.  `fill` itself is continuous
    /// and discards nothing.
    fn fill_native(&mut self, bytes: &mut [u8]) {
        let whole = bytes.len() - bytes.len() % size_of::<u64>();
        let (words, rest) = bytes.split_at_mut(whole);
        self.fill(words);
        if !rest.is_empty() {
            let mut word = [0u8; size_of::<u64>()];
            self.fill(&mut word);
            rest.copy_from_slice(&word[..rest.len()]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{with_generator, FastKeyErasureRng, KEY, RESEED_BYTES, STATE};
    use crate::rng::{hex, thread_rng, Rng};
    use cryptography::cprng::fast_key_erasure::REFILL;
    use cryptography::ChaCha20;

    /// Output is the keystream after its first 32 bytes, and the next refill
    /// is keyed by those 32 bytes.
    #[test]
    fn output_is_keystream_under_rotating_keys() {
        let key: [u8; KEY] = std::array::from_fn(|i| i as u8);
        let mut rng = FastKeyErasureRng::new(key);
        let mut out = [0u8; 2 * (REFILL - KEY)];
        rng.fill(&mut out);

        let mut first = [0u8; REFILL];
        ChaCha20::new(&key, &[0; 12]).apply_keystream(&mut first);
        assert_eq!(out[..REFILL - KEY], first[KEY..]);
        let next_key: [u8; KEY] = first[..KEY].try_into().unwrap();
        let mut second = [0u8; REFILL];
        ChaCha20::new(&next_key, &[0; 12]).apply_keystream(&mut second);
        assert_eq!(out[REFILL - KEY..], second[KEY..]);
    }

    /// The first and last bytes of a 960-byte fill under key 00 … 1f, as
    /// OpenSSL's ChaCha20 computes them independently (cryptography pins the
    /// same bytes on its mechanism).
    #[test]
    fn fill_matches_independent_chacha20() {
        const FIRST: &str = "2b23cce7a26023ab3f0eef693ac87f64258235eab1f7a32dc22762a0485b410c\
                             18b84231ade6a6d113615c61af434e27f8b1f3f5e1ad5b5cecf8fc122a35755c";
        const LAST: &str = "d0649d0f9a4306e3aa7c5bcf77cc8d04a1f80e367a24ee97a867b2295c945177";
        let key: [u8; KEY] = std::array::from_fn(|i| i as u8);
        let mut out = [0u8; 2 * (REFILL - KEY)];
        FastKeyErasureRng::new(key).fill(&mut out);
        let first = hex(FIRST);
        let last = hex(LAST);
        assert_eq!(out[..first.len()], first[..]);
        assert_eq!(out[out.len() - last.len()..], last[..]);
    }

    /// A bulk fill is one thread-local visit per request, and a request that
    /// would cross the reseed limit is split at it: the bytes before the limit
    /// come from the old key, the rest from the new one.
    #[test]
    fn bulk_fills_split_at_the_reseed_limit() {
        const BEFORE: usize = 8;
        const AFTER: usize = 24;
        let pid = std::process::id();
        let seedings = || STATE.with(|c| c.borrow().as_ref().map_or(0, |s| s.seedings));
        let served = || STATE.with(|c| c.borrow().as_ref().map_or(0, |s| s.served));
        let mut handle = thread_rng();
        // Leave BEFORE bytes of the interval.
        STATE.with(|c| c.borrow_mut().as_mut().unwrap().served = RESEED_BYTES - BEFORE as u64);
        let before = seedings();
        let mut bytes = [0u8; BEFORE + AFTER];
        handle.fill(&mut bytes);
        assert_eq!(seedings(), before + 1, "the limit forces one reseed");
        assert_eq!(
            served(),
            AFTER as u64,
            "only the tail is charged to the new key"
        );
        assert!(bytes.iter().any(|&b| b != 0));

        // A fill inside the interval reseeds nothing and charges its length.
        let charged = served();
        handle.fill(&mut bytes);
        assert_eq!(seedings(), before + 1);
        assert_eq!(served(), charged + bytes.len() as u64);
        let mut again = [0u8; BEFORE + AFTER];
        handle.fill(&mut again);
        assert_ne!(bytes, again);
        assert!(with_generator(pid, 0, |_| ()).is_ok());
    }

    /// A new process id or a full reseed interval takes a fresh key.
    #[test]
    fn thread_generator_reseeds_on_fork_and_volume() {
        let pid = std::process::id();
        let seedings = || STATE.with(|c| c.borrow().as_ref().map_or(0, |s| s.seedings));
        let a = with_generator(pid, 8, FastKeyErasureRng::next_u64).unwrap();
        let before = seedings();
        // A different process id, as after fork, seeds again.
        let b = with_generator(pid ^ 1, 8, FastKeyErasureRng::next_u64).unwrap();
        assert_eq!(seedings(), before + 1);
        assert_ne!(a, b);
        STATE.with(|c| c.borrow_mut().as_mut().unwrap().served = RESEED_BYTES);
        with_generator(pid ^ 1, 8, FastKeyErasureRng::next_u64).unwrap();
        assert_eq!(seedings(), before + 2);
        let mut handle = thread_rng();
        assert_ne!(handle.next_u64(), handle.next_u64());
    }
}
