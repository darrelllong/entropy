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

use super::{os::os_random, Rng};
use cryptography::{zeroize_slice, ChaCha20};
use std::{cell::RefCell, io, marker::PhantomData};

/// Keystream bytes per refill: eight ChaCha20 blocks.
const REFILL: usize = 512;
/// Bytes of each refill that become the next key.
const KEY: usize = 32;
/// Output bytes after which a thread's generator takes a fresh key.
const RESEED_BYTES: u64 = 1 << 30;

/// ChaCha20 with fast key erasure.
pub struct FastKeyErasureRng {
    key: [u8; KEY],
    buffer: [u8; REFILL],
    /// Next unserved byte of `buffer`; `REFILL` when empty.
    position: usize,
}

impl FastKeyErasureRng {
    /// A generator whose first key is `key`.  The caller's copy is not wiped.
    #[must_use]
    pub fn new(key: [u8; KEY]) -> Self {
        Self {
            key,
            buffer: [0; REFILL],
            position: REFILL,
        }
    }

    /// A generator keyed from the operating system.
    ///
    /// # Errors
    /// Any error from [`os_random`].
    pub fn from_os() -> io::Result<Self> {
        let mut key = [0u8; KEY];
        os_random(&mut key)?;
        let rng = Self::new(key);
        zeroize_slice(&mut key);
        Ok(rng)
    }

    /// Replace the key with the next 32 bytes of keystream and keep the rest.
    fn refill(&mut self) {
        let mut stream = [0u8; REFILL];
        ChaCha20::new(&self.key, &[0u8; 12]).apply_keystream(&mut stream);
        self.key.copy_from_slice(&stream[..KEY]);
        self.buffer[KEY..].copy_from_slice(&stream[KEY..]);
        zeroize_slice(&mut stream);
        self.position = KEY;
    }

    /// Serve `bytes`, erasing each from the buffer as it goes.
    pub fn fill(&mut self, bytes: &mut [u8]) {
        let mut done = 0;
        while done < bytes.len() {
            if self.position == REFILL {
                self.refill();
            }
            let take = (REFILL - self.position).min(bytes.len() - done);
            let served = &mut self.buffer[self.position..self.position + take];
            bytes[done..done + take].copy_from_slice(served);
            zeroize_slice(served);
            self.position += take;
            done += take;
        }
    }

    /// XOR `fresh` into the key and discard the unserved buffer.
    fn reseed(&mut self, fresh: &[u8; KEY]) {
        for (k, f) in self.key.iter_mut().zip(fresh) {
            *k ^= f;
        }
        zeroize_slice(&mut self.buffer);
        self.position = REFILL;
    }
}

impl Rng for FastKeyErasureRng {
    fn next_u32(&mut self) -> u32 {
        let mut word = [0u8; 4];
        self.fill(&mut word);
        u32::from_le_bytes(word)
    }

    fn next_u64(&mut self) -> u64 {
        let mut word = [0u8; 8];
        self.fill(&mut word);
        u64::from_le_bytes(word)
    }
}

impl Drop for FastKeyErasureRng {
    fn drop(&mut self) {
        zeroize_slice(&mut self.key);
        zeroize_slice(&mut self.buffer);
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
/// if it has served [`RESEED_BYTES`].
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
                rng: FastKeyErasureRng::from_os()?,
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
    try_thread_rng().expect("thread_rng: the operating system's entropy source failed")
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

impl Rng for ThreadRng {
    fn next_u32(&mut self) -> u32 {
        with_generator(std::process::id(), 4, FastKeyErasureRng::next_u32)
            .expect("thread_rng: the operating system's entropy source failed")
    }

    fn next_u64(&mut self) -> u64 {
        with_generator(std::process::id(), 8, FastKeyErasureRng::next_u64)
            .expect("thread_rng: the operating system's entropy source failed")
    }
}

#[cfg(test)]
mod tests {
    use super::{with_generator, FastKeyErasureRng, KEY, REFILL, RESEED_BYTES, STATE};
    use crate::rng::{thread_rng, Rng};
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

    /// Served bytes are erased from the buffer, and the state holds no copy.
    #[test]
    fn served_bytes_are_erased() {
        let mut rng = FastKeyErasureRng::new([7; KEY]);
        let word = rng.next_u64().to_le_bytes();
        assert_eq!(rng.buffer[KEY..KEY + 8], [0; 8]);
        assert!(!rng.buffer.windows(8).any(|w| w == word));
        assert!(!rng.key.windows(8).any(|w| w == word));
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
