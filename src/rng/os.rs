//! OS entropy source via `/dev/urandom`.

use std::fs::File;
use std::io::{self, Read};
use std::sync::OnceLock;

use super::Rng;
#[cfg(feature = "cryptography")]
use cryptography::zeroize_slice;

/// Size of the internal read buffer.  Refilled in one `read_exact` call when
/// exhausted, so the syscall cost is amortised over 64 `next_u32` calls
/// instead of one syscall per word.
const BUF_LEN: usize = 256;

/// Reads from `/dev/urandom` — the platform CSPRNG on macOS/Linux.
///
/// This should **pass** every test in the suite with high probability.
/// On macOS, `/dev/urandom` and `/dev/random` are both backed by the same
/// Fortuna-based CSPRNG since macOS 10.12.
///
/// # Platform support
/// `OsRng` is **Unix-only**: it reads the `/dev/urandom` character device
/// directly (no `unsafe`, no `getrandom` dependency).  On a non-Unix target
/// (e.g. Windows) construction panics with a clear message rather than reading
/// entropy — the crate as a whole is developed and run on Unix.  A portable
/// build would depend on the `getrandom` crate; that dependency choice is left
/// to the consumer.  Generators that do not seed from the OS (`Mt19937`, the
/// LCG family, the fixed-seed ciphers, …) work on every platform.
///
/// # Early-boot entropy warning
/// `/dev/urandom` on Linux does **not** block if the kernel entropy pool is
/// not yet fully initialized (e.g., early in the boot sequence or inside a
/// container/VM with limited entropy sources).  Reading before the pool is
/// seeded can return low-quality output; this is the failure mode documented
/// in Hughes (2021) "BADRANDOM" where TLS servers starting before sufficient
/// entropy was available produced predictable key material.  On Linux 3.17+
/// the `getrandom(2)` syscall with `flags = 0` already blocks until the
/// entropy pool is initialized and is the preferred interface for
/// cryptographic seeding.  (The `GRND_RANDOM` flag instead selects the legacy
/// `/dev/random` pool and is not recommended.)  macOS's `/dev/urandom` blocks
/// at boot until the CSPRNG is seeded, so this concern is macOS-specific only
/// at very early boot.
///
/// On Linux, before its first read the process reads one byte from
/// `/dev/random`, which since Linux 5.6 blocks until the kernel pool is
/// initialized and then never again, and older kernels release once the pool
/// holds enough entropy: after it returns, `/dev/urandom` is seeded, as
/// `getrandom(2)` with no flags would guarantee.  The check runs once per
/// process once it succeeds; a failed check is retried, since it can come from
/// a transient condition.  [`OsRng::try_new`], [`OsRng::try_fill`] and
/// [`os_random`] report errors instead of panicking.
pub struct OsRng {
    file: File,
    buf: [u8; BUF_LEN],
    pos: usize, // index of next unread byte; BUF_LEN = exhausted
}

/// Wait until the kernel's pool is initialized: on Linux by reading one byte
/// from `/dev/random`; elsewhere `/dev/urandom` itself blocks until seeded.
///
/// Only success is remembered, because the pool is initialized once and stays
/// so.  A failure is not: it can be a descriptor limit, an interrupted read or
/// a device not yet visible in a starting container, all of which the next
/// call may find gone.  So a failed check costs one open and one read per
/// attempt, and a caller that retries gets a fresh answer.
fn pool_ready() -> io::Result<()> {
    static READY: OnceLock<()> = OnceLock::new();
    if READY.get().is_some() || !cfg!(target_os = "linux") {
        return Ok(());
    }
    let mut byte = [0u8; 1];
    File::open("/dev/random").and_then(|mut f| f.read_exact(&mut byte))?;
    let _ = READY.set(());
    Ok(())
}

/// Fill `bytes` from the operating system's entropy source, once its pool is
/// initialized, without keeping a copy.
///
/// # Errors
/// Any error opening or reading `/dev/random` or `/dev/urandom`, including
/// `NotFound` on a system without them.
pub fn os_random(bytes: &mut [u8]) -> io::Result<()> {
    pool_ready()?;
    File::open("/dev/urandom")?.read_exact(bytes)
}

/// The panic of every infallible constructor whose `try_` form failed.
pub(crate) const OS_FAILED: &str = "the operating system's entropy source failed";

impl OsRng {
    /// Open `/dev/urandom`.
    ///
    /// # Panics
    /// Panics if [`OsRng::try_new`] fails — normal on non-Unix targets, where
    /// `OsRng` is unsupported (see the type-level Platform support note).
    pub fn new() -> Self {
        Self::try_new().unwrap_or_else(|e| {
            panic!(
                "OsRng: cannot use /dev/urandom ({e}). OsRng is Unix-only; on \
                 other platforms use a non-OS generator."
            )
        })
    }

    /// Open `/dev/urandom` once the kernel's pool is initialized.
    ///
    /// # Errors
    /// Any error opening `/dev/urandom` or waiting on `/dev/random`.
    pub fn try_new() -> io::Result<Self> {
        pool_ready()?;
        Ok(Self {
            file: File::open("/dev/urandom")?,
            buf: [0u8; BUF_LEN],
            pos: BUF_LEN, // force a refill on first use
        })
    }

    /// Fill `bytes` straight from the device, bypassing the word buffer.
    ///
    /// # Errors
    /// Any read error.
    pub fn try_fill(&mut self, bytes: &mut [u8]) -> io::Result<()> {
        self.file.read_exact(bytes)
    }
}

impl Default for OsRng {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for OsRng {
    /// With the `cryptography` feature, wipe the buffered entropy with that
    /// crate's volatile write: `from_os_rng`-style constructors draw seed
    /// material through this buffer.  Without the feature nothing is wiped,
    /// here or in rump.
    fn drop(&mut self) {
        #[cfg(feature = "cryptography")]
        zeroize_slice(&mut self.buf);
    }
}

impl Rng for OsRng {
    fn next_u32(&mut self) -> u32 {
        // BUF_LEN is a multiple of 4, so a word never straddles a refill.
        if self.pos + 4 > BUF_LEN {
            self.file
                .read_exact(&mut self.buf)
                .expect("read from /dev/urandom failed");
            self.pos = 0;
        }
        let w = u32::from_le_bytes(self.buf[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        w
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fallible_reads_fill_buffers() {
        let mut a = [0u8; 64];
        let mut b = [0u8; 64];
        super::os_random(&mut a).unwrap();
        super::OsRng::try_new().unwrap().try_fill(&mut b).unwrap();
        assert_ne!(a, [0u8; 64]);
        assert_ne!(a, b);
    }
}
