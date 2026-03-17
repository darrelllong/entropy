//! OS entropy source via `/dev/urandom`.

use std::fs::File;
use std::io::Read;

use super::Rng;

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
/// the `getrandom(2)` syscall with the `GRND_RANDOM` flag blocks until the
/// pool is ready and is preferable for cryptographic seeding.  macOS's
/// `/dev/urandom` blocks at boot until the CSPRNG is seeded, so this concern
/// is macOS-specific only at very early boot.
///
/// For this test harness running on a fully-booted system, `/dev/urandom` is
/// fine.  In production, use `getrandom(2)` or a platform API that guarantees
/// the entropy pool is initialized before returning.
pub struct OsRng {
    file: File,
}

impl OsRng {
    /// Open `/dev/urandom`.
    ///
    /// # Panics
    /// Panics if `/dev/urandom` cannot be opened (non-Unix platform).
    pub fn new() -> Self {
        Self {
            file: File::open("/dev/urandom").expect("cannot open /dev/urandom"),
        }
    }
}

impl Default for OsRng {
    fn default() -> Self {
        Self::new()
    }
}

impl Rng for OsRng {
    fn next_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.file
            .read_exact(&mut buf)
            .expect("read from /dev/urandom failed");
        u32::from_le_bytes(buf)
    }
}
