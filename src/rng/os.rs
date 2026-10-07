//! OS entropy source: `/dev/urandom` on Unix, `ProcessPrng` on Windows.

use std::io;

use super::Rng;
#[cfg(feature = "cryptography")]
use cryptography::zeroize_slice;

/// Size of the internal read buffer.  Refilled in one call when exhausted, so
/// the syscall cost is amortised over 64 `next_u32` calls instead of one
/// syscall per word.
const BUF_LEN: usize = 256;

/// The operating system's CSPRNG.
///
/// On macOS and Linux it is `/dev/urandom`; on Windows it is `ProcessPrng`,
/// the user-mode per-processor generator in `bcryptprimitives.dll` that the
/// standard library itself seeds `HashMap` from.
///
/// This should **pass** every test in the suite with high probability.
/// On macOS, `/dev/urandom` and `/dev/random` are both backed by the same
/// Fortuna-based CSPRNG since macOS 10.12.
///
/// # Platform support
/// On every target but Windows, `OsRng` reads the `/dev/urandom` character
/// device directly (no `unsafe`, no dependency); on a target without that
/// device the fallible constructors return `NotFound` and the infallible ones
/// panic with a clear message.  On Windows there is no such device and no
/// system call reachable without a foreign function, which this crate's
/// `#![forbid(unsafe_code)]` rules out, so `OsRng` calls `ProcessPrng` through
/// the [`getrandom`](https://docs.rs/getrandom) crate, this crate's only
/// Windows-specific dependency.  Generators that do not seed from the OS
/// (`Mt19937`, the LCG family, the fixed-seed ciphers, …) depend on neither.
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
/// at very early boot.  Windows' `ProcessPrng` is seeded from the kernel
/// generator before any user process can call it.
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
    source: source::Source,
    buf: [u8; BUF_LEN],
    pos: usize, // index of next unread byte; BUF_LEN = exhausted
}

/// The `/dev/urandom` source, on every target but Windows.
#[cfg(not(windows))]
mod source {
    use std::fs::File;
    use std::io::{self, Read};
    use std::sync::OnceLock;

    /// What the panic messages call the source.
    pub(super) const NAME: &str = "/dev/urandom";

    /// An open descriptor on `/dev/urandom`.
    pub(super) struct Source(File);

    /// Wait until the kernel's pool is initialized: on Linux by reading one
    /// byte from `/dev/random`; elsewhere `/dev/urandom` itself blocks until
    /// seeded.
    ///
    /// Only success is remembered, because the pool is initialized once and
    /// stays so.  A failure is not: it can be a descriptor limit, an
    /// interrupted read or a device not yet visible in a starting container,
    /// all of which the next call may find gone.  So a failed check costs one
    /// open and one read per attempt, and a caller that retries gets a fresh
    /// answer.
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

    impl Source {
        /// Open `/dev/urandom` once the kernel's pool is initialized.
        pub(super) fn open() -> io::Result<Self> {
            pool_ready()?;
            File::open(NAME).map(Self)
        }

        /// Fill `bytes` from the device.
        pub(super) fn fill(&mut self, bytes: &mut [u8]) -> io::Result<()> {
            self.0.read_exact(bytes)
        }
    }
}

/// The `ProcessPrng` source, on Windows, through the `getrandom` crate.
#[cfg(windows)]
mod source {
    use std::io;

    /// What the panic messages call the source.
    pub(super) const NAME: &str = "ProcessPrng";

    /// `ProcessPrng` keeps no per-caller state, so there is nothing to open.
    pub(super) struct Source;

    impl Source {
        /// Nothing to open; `ProcessPrng` is always available on Windows 10
        /// and later.
        pub(super) fn open() -> io::Result<Self> {
            Ok(Self)
        }

        /// Fill `bytes` from `ProcessPrng`.
        pub(super) fn fill(&mut self, bytes: &mut [u8]) -> io::Result<()> {
            getrandom::fill(bytes).map_err(io::Error::from)
        }
    }
}

/// Fill `bytes` from the operating system's entropy source, once its pool is
/// initialized, without keeping a copy.
///
/// # Errors
/// Any error opening or reading `/dev/random` or `/dev/urandom`, including
/// `NotFound` on a system without them; on Windows, any error `ProcessPrng`
/// reports.
pub fn os_random(bytes: &mut [u8]) -> io::Result<()> {
    source::Source::open()?.fill(bytes)
}

/// The panic of every infallible constructor whose `try_` form failed.
pub(crate) const OS_FAILED: &str = "the operating system's entropy source failed";

impl OsRng {
    /// Open the operating system's entropy source.
    ///
    /// # Panics
    /// Panics if [`OsRng::try_new`] fails — normal on a target that is
    /// neither Windows nor has `/dev/urandom` (see the type-level Platform
    /// support note).
    pub fn new() -> Self {
        Self::try_new().unwrap_or_else(|e| {
            panic!(
                "OsRng: cannot use {} ({e}); on a platform without it use a \
                 non-OS generator.",
                source::NAME
            )
        })
    }

    /// Open the operating system's entropy source, on Unix once the kernel's
    /// pool is initialized.
    ///
    /// # Errors
    /// Any error opening `/dev/urandom` or waiting on `/dev/random`.  On
    /// Windows this does not fail.
    pub fn try_new() -> io::Result<Self> {
        Ok(Self {
            source: source::Source::open()?,
            buf: [0u8; BUF_LEN],
            pos: BUF_LEN, // force a refill on first use
        })
    }

    /// Fill `bytes` straight from the source, bypassing the word buffer.
    ///
    /// # Errors
    /// Any read error.
    pub fn try_fill(&mut self, bytes: &mut [u8]) -> io::Result<()> {
        self.source.fill(bytes)
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
            self.source
                .fill(&mut self.buf)
                .unwrap_or_else(|e| panic!("read from {} failed: {e}", source::NAME));
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
