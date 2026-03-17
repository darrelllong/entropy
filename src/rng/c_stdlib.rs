//! Historical Unix libc PRNGs.
//!
//! These are here as negative controls and compatibility probes, not as
//! recommendations. None of them are cryptographically secure, and several are
//! spectacularly weak even by non-cryptographic standards.
//!
//! Implemented from primary-source libc code:
//! - System V / POSIX sample `rand()`: 15-bit LCG output
//! - BSD `random()`: 31-word additive generator with TYPE_3 state
//! - Linux glibc `rand()`: alias of glibc `random()`
//! - FreeBSD 12 compatibility `rand_r()`: Park-Miller style single-word state
//! - Windows MSVCRT/UCRT `rand()`: 15-bit LCG output
//! - Windows VB6/VBA `Rnd`: 24-bit linear congruential state
//! - Windows/.NET Framework `System.Random(seed)`: Knuth-style subtractive PRNG
//! - System V / POSIX `mrand48()`: 48-bit LCG

use super::Rng;

#[derive(Debug, Clone, Default)]
struct PackedBits {
    acc: u64,
    bits: u32,
}

impl PackedBits {
    fn push(&mut self, value: u32, width: u32) {
        debug_assert!((1..=32).contains(&width));
        let masked = if width == 32 {
            u64::from(value)
        } else {
            u64::from(value) & ((1u64 << width) - 1)
        };
        self.acc |= masked << self.bits;
        self.bits += width;
    }

    fn pop_word(&mut self) -> u32 {
        debug_assert!(self.bits >= 32);
        let out = self.acc as u32;
        self.acc >>= 32;
        self.bits -= 32;
        out
    }
}

fn park_miller31(seed: u32) -> u32 {
    let mut x = if seed == 0 { 1i64 } else { i64::from(seed) };
    let hi = x / 127_773;
    let lo = x % 127_773;
    x = 16_807 * lo - 2_836 * hi;
    if x < 0 {
        x += 2_147_483_647;
    }
    x as u32
}

// ── System V rand() ──────────────────────────────────────────────────────────

/// Faithful System V / old-POSIX-style `rand()`:
///
/// `next = next * 1103515245 + 12345; return (next >> 16) & 0x7fff;`
///
/// This is the classic 15-bit libc generator that many systems exposed before
/// higher-quality interfaces were common. It is a bad RNG.
#[derive(Debug, Clone)]
pub struct SystemVRand {
    state: u32,
    bits: PackedBits,
}

impl SystemVRand {
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed,
            bits: PackedBits::default(),
        }
    }

    pub fn next_raw(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (self.state >> 16) & 0x7fff
    }
}

impl Rng for SystemVRand {
    fn next_u32(&mut self) -> u32 {
        while self.bits.bits < 32 {
            let raw = self.next_raw();
            self.bits.push(raw, 15);
        }
        self.bits.pop_word()
    }
}

/// Compatibility alias for existing internal users.
pub type CRand = SystemVRand;

// ── Windows CRT rand() ───────────────────────────────────────────────────────

/// Faithful Windows CRT `rand()` as used by MSVCRT-family runtimes:
///
/// `state = state * 214013 + 2531011; return (state >> 16) & 0x7fff;`
///
/// This is the notoriously weak 15-bit generator associated with many classic
/// Windows/MSVC-era programs. It is included as a bad historical control.
#[derive(Debug, Clone)]
pub struct WindowsMsvcRand {
    state: u32,
    bits: PackedBits,
}

impl WindowsMsvcRand {
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed,
            bits: PackedBits::default(),
        }
    }

    pub fn next_raw(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(214_013).wrapping_add(2_531_011);
        (self.state >> 16) & 0x7fff
    }
}

impl Rng for WindowsMsvcRand {
    fn next_u32(&mut self) -> u32 {
        while self.bits.bits < 32 {
            let raw = self.next_raw();
            self.bits.push(raw, 15);
        }
        self.bits.pop_word()
    }
}

// ── Windows VB6 / VBA Rnd() ──────────────────────────────────────────────────

/// Faithful VB6/VBA `Rnd` core state transition.
///
/// Microsoft still preserves this compatibility algorithm in `VBMath.Rnd`:
/// `seed = (seed * 0x43FD43FD + 0x00C39EC3) & 0x00FF_FFFF`.
///
/// The public API returns a `Single` in `[0, 1)`, so we expose both the raw
/// 24-bit state and a faithful `next_f64()` mapping. This is a tiny-state,
/// trivially predictable historical Windows generator.
#[derive(Debug, Clone)]
pub struct WindowsVb6Rnd {
    state: u32,
    bits: PackedBits,
}

impl WindowsVb6Rnd {
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed & 0x00ff_ffff,
            bits: PackedBits::default(),
        }
    }

    pub fn next_raw(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(0x43fd_43fd)
            .wrapping_add(0x00c3_9ec3)
            & 0x00ff_ffff;
        self.state
    }

    pub fn next_sample(&mut self) -> f64 {
        self.next_raw() as f64 * (1.0 / 16_777_216.0)
    }
}

impl Rng for WindowsVb6Rnd {
    fn next_u32(&mut self) -> u32 {
        while self.bits.bits < 32 {
            let raw = self.next_raw();
            self.bits.push(raw, 24);
        }
        self.bits.pop_word()
    }

    fn next_f64(&mut self) -> f64 {
        self.next_sample()
    }
}

// ── Windows/.NET Random(seed) compatibility ─────────────────────────────────

/// Faithful `.NET Framework` / classic `System.Random(seed)` compatibility PRNG.
///
/// This is the long-lived subtractive generator preserved for seed-compatibility
/// in modern .NET runtimes. It is widely deployed and very much not a CSPRNG.
#[derive(Debug, Clone)]
pub struct WindowsDotNetRandom {
    seed_array: [i32; 56],
    inext: usize,
    inextp: usize,
    bits: PackedBits,
}

impl WindowsDotNetRandom {
    pub fn new(seed: i32) -> Self {
        let mut seed_array = [0i32; 56];
        let subtraction = if seed == i32::MIN {
            i32::MAX
        } else {
            seed.abs()
        };
        let mut mj = 161_803_398 - subtraction;
        seed_array[55] = mj;
        let mut mk = 1i32;
        let mut ii = 0usize;
        for _i in 1..55 {
            ii += 21;
            if ii >= 55 {
                ii -= 55;
            }
            seed_array[ii] = mk;
            mk = mj - mk;
            if mk < 0 {
                mk += i32::MAX;
            }
            mj = seed_array[ii];
        }

        for _ in 1..5 {
            for i in 1..56 {
                let mut n = i + 30;
                if n >= 55 {
                    n -= 55;
                }
                seed_array[i] -= seed_array[1 + n];
                if seed_array[i] < 0 {
                    seed_array[i] += i32::MAX;
                }
            }
        }

        Self {
            seed_array,
            inext: 0,
            inextp: 21,
            bits: PackedBits::default(),
        }
    }

    pub fn next_raw(&mut self) -> u32 {
        self.inext += 1;
        if self.inext >= 56 {
            self.inext = 1;
        }

        self.inextp += 1;
        if self.inextp >= 56 {
            self.inextp = 1;
        }

        let mut ret = self.seed_array[self.inext] - self.seed_array[self.inextp];
        if ret == i32::MAX {
            ret -= 1;
        }
        if ret < 0 {
            ret += i32::MAX;
        }

        self.seed_array[self.inext] = ret;
        ret as u32
    }

    pub fn next_sample(&mut self) -> f64 {
        self.next_raw() as f64 * (1.0 / i32::MAX as f64)
    }
}

impl Rng for WindowsDotNetRandom {
    fn next_u32(&mut self) -> u32 {
        while self.bits.bits < 32 {
            let raw = self.next_raw();
            self.bits.push(raw, 31);
        }
        self.bits.pop_word()
    }

    fn next_f64(&mut self) -> f64 {
        self.next_sample()
    }
}

// ── BSD random() / glibc random() ────────────────────────────────────────────

/// BSD `random()` with the default 128-byte TYPE_3 state (`deg=31`, `sep=3`).
///
/// This is the classic Berkeley additive generator carried into glibc's
/// `random()` and therefore Linux glibc `rand()`. It is much better than the
/// 15-bit System V LCG, but it is still a weak historical userspace PRNG.
#[derive(Debug, Clone)]
pub struct BsdRandom {
    state: [u32; 31],
    fptr: usize,
    rptr: usize,
    bits: PackedBits,
}

impl BsdRandom {
    pub fn new(seed: u32) -> Self {
        let seed = if seed == 0 { 1 } else { seed };
        let mut state = [0u32; 31];
        state[0] = seed;
        for i in 1..31 {
            state[i] = park_miller31(state[i - 1]);
        }

        let mut rng = Self {
            state,
            fptr: 3,
            rptr: 0,
            bits: PackedBits::default(),
        };

        for _ in 0..310 {
            let _ = rng.next_raw();
        }

        rng
    }

    pub fn next_raw(&mut self) -> u32 {
        let val = self.state[self.fptr].wrapping_add(self.state[self.rptr]);
        self.state[self.fptr] = val;
        let out = val >> 1;

        self.fptr += 1;
        if self.fptr == self.state.len() {
            self.fptr = 0;
        }

        self.rptr += 1;
        if self.rptr == self.state.len() {
            self.rptr = 0;
        }

        out
    }
}

impl Rng for BsdRandom {
    fn next_u32(&mut self) -> u32 {
        while self.bits.bits < 32 {
            let raw = self.next_raw();
            self.bits.push(raw, 31);
        }
        self.bits.pop_word()
    }
}

/// Linux glibc `rand()`/`random()` compatibility wrapper.
///
/// glibc's `rand()` is just `random()` under the hood, so the Linux userspace
/// generator many programs used before `/dev/random`, `/dev/urandom`, and
/// `getrandom(2)` became the norm is this same Berkeley-derived TYPE_3 engine.
#[derive(Debug, Clone)]
pub struct LinuxLibcRandom {
    inner: BsdRandom,
}

impl LinuxLibcRandom {
    pub fn new(seed: u32) -> Self {
        Self {
            inner: BsdRandom::new(seed),
        }
    }
}

impl Rng for LinuxLibcRandom {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }
}

// ── FreeBSD compatibility rand_r() ───────────────────────────────────────────

/// FreeBSD 12 compatibility `rand()` / current `rand_r()` core.
///
/// This is the single-word Park-Miller compatibility path kept around by
/// FreeBSD for ABI reasons. FreeBSD's own source calls it garbage.
#[derive(Debug, Clone)]
pub struct BsdRandCompat {
    state: u32,
    bits: PackedBits,
}

impl BsdRandCompat {
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed,
            bits: PackedBits::default(),
        }
    }

    pub fn next_raw(&mut self) -> u32 {
        let x = (u64::from(self.state) % 0x7fff_fffe) + 1;
        let hi = x / 127_773;
        let lo = x % 127_773;
        let mut next = 16_807i64 * lo as i64 - 2_836i64 * hi as i64;
        if next < 0 {
            next += 2_147_483_647;
        }
        let out = (next as u32).wrapping_sub(1);
        self.state = out;
        out
    }
}

impl Rng for BsdRandCompat {
    fn next_u32(&mut self) -> u32 {
        while self.bits.bits < 32 {
            let raw = self.next_raw();
            self.bits.push(raw, 31);
        }
        self.bits.pop_word()
    }
}

// ── System V mrand48() ───────────────────────────────────────────────────────

/// Pure-Rust implementation of POSIX / System V `mrand48()`.
///
/// 48-bit LCG with the mandated parameters:
/// `a = 0x5DEECE66D`, `c = 0xB`, `m = 2^48`.
/// Better than 15-bit `rand()`, but still linear and weak.
#[derive(Debug, Clone)]
pub struct Rand48 {
    state: u64,
}

const RAND48_A: u64 = 0x5DEECE66D;
const RAND48_C: u64 = 0xB;
const RAND48_M: u64 = 1 << 48;

impl Rand48 {
    pub fn new(seed: u64) -> Self {
        Self {
            state: (seed << 16) | 0x330E,
        }
    }
}

impl Rng for Rand48 {
    fn next_u32(&mut self) -> u32 {
        self.state = (RAND48_A.wrapping_mul(self.state).wrapping_add(RAND48_C)) % RAND48_M;
        (self.state >> 16) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BsdRandCompat, BsdRandom, LinuxLibcRandom, Rand48, SystemVRand, WindowsDotNetRandom,
        WindowsMsvcRand, WindowsVb6Rnd,
    };
    use crate::rng::Rng;

    #[test]
    fn system_v_rand_raw_matches_posix_sample() {
        let mut rng = SystemVRand::new(1);
        let expected = [16838, 5758, 10113, 17515, 31051];
        for want in expected {
            assert_eq!(rng.next_raw(), want);
        }
    }

    #[test]
    fn bsd_random_matches_well_known_seed_1_prefix() {
        let mut rng = BsdRandom::new(1);
        let expected = [
            1_804_289_383,
            846_930_886,
            1_681_692_777,
            1_714_636_915,
            1_957_747_793,
        ];
        for want in expected {
            assert_eq!(rng.next_raw(), want);
        }
    }

    #[test]
    fn linux_glibc_rand_alias_matches_random() {
        let mut linux = LinuxLibcRandom::new(1);
        let mut bsd = BsdRandom::new(1);
        for _ in 0..8 {
            assert_eq!(linux.next_u32(), bsd.next_u32());
        }
    }

    #[test]
    fn freebsd_compat_rand_r_prefix_matches_reference_math() {
        let mut rng = BsdRandCompat::new(1);
        let expected = [33_613, 564_950_497, 1_097_816_498, 1_969_887_315];
        for want in expected {
            assert_eq!(rng.next_raw(), want);
        }
    }

    #[test]
    fn rand48_produces_non_constant_output() {
        let mut rng = Rand48::new(1);
        let a = rng.next_u32();
        let b = rng.next_u32();
        assert_ne!(a, b);
    }

    #[test]
    fn windows_msvc_rand_matches_known_seed_1_prefix() {
        let mut rng = WindowsMsvcRand::new(1);
        let expected = [41, 18_467, 6_334, 26_500, 19_169];
        for want in expected {
            assert_eq!(rng.next_raw(), want);
        }
    }

    #[test]
    fn windows_vb6_rnd_matches_known_seed_1_prefix() {
        let mut rng = WindowsVb6Rnd::new(1);
        let expected = [12_640_960, 8_124_035, 4_294_458, 3_961_109, 14_212_996];
        for want in expected {
            assert_eq!(rng.next_raw(), want);
        }
    }

    #[test]
    fn windows_dotnet_random_matches_seed_1_prefix() {
        let mut rng = WindowsDotNetRandom::new(1);
        let expected = [
            534_011_718,
            237_820_880,
            1_002_897_798,
            1_657_007_234,
            1_412_011_072,
        ];
        for want in expected {
            assert_eq!(rng.next_raw(), want);
        }
    }
}
