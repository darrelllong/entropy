//! OS entropy source via `/dev/urandom`.

use std::fs::File;
use std::io::Read;

use super::Rng;
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
/// For this test harness running on a fully-booted system, `/dev/urandom` is
/// fine.  In production, use `getrandom(2)` or a platform API that guarantees
/// the entropy pool is initialized before returning.
pub struct OsRng {
    file: File,
    buf: [u8; BUF_LEN],
    pos: usize, // index of next unread byte; BUF_LEN = exhausted
}

impl OsRng {
    /// Open `/dev/urandom`.
    ///
    /// # Panics
    /// Panics if `/dev/urandom` cannot be opened (non-Unix platform).
    pub fn new() -> Self {
        Self {
            file: File::open("/dev/urandom").expect("cannot open /dev/urandom"),
            buf: [0u8; BUF_LEN],
            pos: BUF_LEN, // force a refill on first use
        }
    }
}

impl Default for OsRng {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for OsRng {
    /// Wipe the buffered entropy: `from_os_rng`-style constructors draw seed
    /// material through this buffer, and those bytes must not outlive the
    /// generator (the same hygiene `AesCtr`/`BlockCtrRng` apply to key
    /// material and keystream).
    fn drop(&mut self) {
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
