//! OS entropy source via `/dev/urandom`.

use std::fs::File;
use std::io::Read;

use super::Rng;

/// Reads from `/dev/urandom` — the platform CSPRNG on macOS/Linux.
///
/// This should **pass** every test in the suite with high probability.
/// On macOS, `/dev/urandom` and `/dev/random` are both backed by the same
/// Fortuna-based CSPRNG since macOS 10.12.
pub struct OsRng {
    file: File,
}

impl OsRng {
    /// Open `/dev/urandom`.
    ///
    /// # Panics
    /// Panics if `/dev/urandom` cannot be opened (non-Unix platform).
    pub fn new() -> Self {
        Self { file: File::open("/dev/urandom").expect("cannot open /dev/urandom") }
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
        self.file.read_exact(&mut buf).expect("read from /dev/urandom failed");
        u32::from_le_bytes(buf)
    }
}
