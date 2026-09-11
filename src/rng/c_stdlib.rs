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
//!
//! # References
//! * K. Thompson and D. M. Ritchie, *Unix Programmer's Manual*, 7th Edition,
//!   Bell Laboratories, 1979.  [pubs/v7-unix-programmers-manual-vol1.pdf]
//!   [`rand(3)` describes a multiplicative congruential generator with period
//!   2³² returning 0 to 2¹⁵ − 1, the shape of `SystemVRand`; it does not print
//!   the 1103515245/12345 parameters, which are those of the sample `rand()`
//!   in the C standard and POSIX (not in `pubs/`).  `WindowsMsvcRand` uses
//!   Microsoft's distinct 214013/2531011 parameters, not this pair.]
//! * S. K. Park and K. W. Miller, "Random number generators: good ones are
//!   hard to find," *Communications of the ACM* 31(10), pp. 1192–1201, 1988.
//!   DOI: 10.1145/63039.63042.
//!   [MINSTD; basis of the `park_miller31` table fill in `BsdRandom` and of
//!   `BsdRandCompat`]
//! * D. E. Knuth, *The Art of Computer Programming, Volume 2: Seminumerical
//!   Algorithms*, 3rd edition, Addison-Wesley, 1997. §3.2.2.
//!   [Subtractive generator used by Windows/.NET `System.Random`]
//! * J. P. Hughes, "BADRANDOM: The Effect and Mitigations for Low Entropy
//!   Random Numbers in TLS," PhD thesis, UC Santa Cruz, 2021.
//!   [pubs/hughes-2022-badrandom-the-effect-and-mitigations-for-low-entropy-random-numbers-in-tls.pdf]
//!   [§2 surveys historical weak libc generators of this class as prior failure modes]
//! * IEEE Std 1003.1 (POSIX.1), *The Open Group Base Specifications*, Issue 8, 2024.
//!   [Normative specification of `rand()`, `rand_r()`, and `mrand48()`]
//! * The GNU C Library 2.40, `stdlib/random_r.c` and `stdlib/random.c`.
//!   [pubs/glibc-2.40-random_r.c] [pubs/glibc-2.40-random.c]
//!   [`__srandom_r` and `__random_r` on the TYPE_3 state are `BsdRandom`;
//!   `random.c` makes `srand` a weak alias of `__srandom`]
//! * The FreeBSD Project, `lib/libc/stdlib/rand.c` and `random.c` at commit
//!   0d022baa047a.  [pubs/freebsd-0d022baa047a-rand.c]
//!   [pubs/freebsd-0d022baa047a-random.c]  [`do_rand`, behind `rand_r`, is
//!   `BsdRandCompat`; `random.c` seeds differently from `BsdRandom`]

use super::Rng;

/// Bit-packing accumulator used by the historical `rand()` adapters that
/// produce fewer than 32 bits per call.  Shared with [`super::lcg::Lcg32`]
/// so both code paths agree on a single packing rule.
#[derive(Debug, Clone, Default)]
pub(super) struct PackedBits {
    pub(super) acc: u64,
    pub(super) bits: u32,
}

impl PackedBits {
    pub(super) fn push(&mut self, value: u32, width: u32) {
        debug_assert!((1..=32).contains(&width));
        let masked = if width == 32 {
            u64::from(value)
        } else {
            u64::from(value) & ((1u64 << width) - 1)
        };
        self.acc |= masked << self.bits;
        self.bits += width;
    }

    pub(super) fn pop_word(&mut self) -> u32 {
        debug_assert!(self.bits >= 32);
        let out = self.acc as u32;
        self.acc >>= 32;
        self.bits -= 32;
        out
    }
}

/// One step of the Park–Miller table fill in glibc's `__srandom_r`
/// (`stdlib/random_r.c`, [pubs/glibc-2.40-random_r.c]).
///
/// glibc keeps the running value as `int32_t word = seed` and applies
/// Schrage's decomposition to that *signed* word: `hi = word / 127773`,
/// `lo = word % 127773`, `word = 16807·lo − 2836·hi`, plus 2³¹ − 1 if the
/// result is negative.  A seed ≥ 2³¹ therefore enters as a negative number.
/// Rust's `i64` division truncates toward zero exactly as C's does, and the
/// products fit the `long` glibc computes them in.  glibc replaces only the
/// seed 0 by 1 (see [`BsdRandom::new`]); a word that reaches 0 mid-fill
/// (seeds 2³¹ − 1 and 2³¹ + 1) stays 0.
fn park_miller31(word: u32) -> u32 {
    // `as i32` reinterprets the bits, as the conversion to `int32_t` does.
    let word = i64::from(word as i32);
    let hi = word / 127_773;
    let lo = word % 127_773;
    let mut next = 16_807 * lo - 2_836 * hi;
    if next < 0 {
        next += 2_147_483_647;
    }
    next as u32
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
    /// Construct from a 32-bit seed (the `srand()` value).
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed,
            bits: PackedBits::default(),
        }
    }

    /// One raw `rand()` step: returns the 15-bit value `(state >> 16) & 0x7fff`.
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

// ── Windows CRT rand() ───────────────────────────────────────────────────────

/// Faithful Windows CRT `rand()` as used by MSVCRT-family runtimes:
///
/// `state = state * 214013 + 2531011; return (state >> 16) & 0x7fff;`
///
/// This is the notoriously weak 15-bit generator associated with many classic
/// Windows/MSVC-era programs. It is included as a bad historical control.
///
/// The same generator as [`LcgVariant::Msvc`](super::LcgVariant::Msvc), the
/// parameterised-LCG view: for any seed the two give identical `next_raw` and
/// `next_u32` streams, and their tests share one known-answer vector.  The
/// battery runs only this type; `dump_rng` and `pilot_rng` expose both, as
/// `windows_msvc_rand` and `msvc_lcg`.
#[derive(Debug, Clone)]
pub struct WindowsMsvcRand {
    state: u32,
    bits: PackedBits,
}

impl WindowsMsvcRand {
    /// Construct from a 32-bit seed (the `srand()` value).
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed,
            bits: PackedBits::default(),
        }
    }

    /// One raw `rand()` step: returns the 15-bit value `(state >> 16) & 0x7fff`.
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
    /// Construct from a seed; only the low 24 bits are kept as state.
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed & 0x00ff_ffff,
            bits: PackedBits::default(),
        }
    }

    /// One raw `Rnd` state transition: returns the new 24-bit state.
    pub fn next_raw(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(0x43fd_43fd)
            .wrapping_add(0x00c3_9ec3)
            & 0x00ff_ffff;
        self.state
    }

    /// Faithful `Rnd` return value: the 24-bit state mapped into `[0, 1)`.
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
    /// Construct from an `i32` seed exactly as `System.Random(seed)` does,
    /// including the Knuth-style seed-array initialisation rounds.
    ///
    /// C# evaluates `int` arithmetic unchecked, and exactly one step here can
    /// overflow: the correction rounds' subtraction
    /// `seed_array[i] - seed_array[1 + n]`.  It uses `wrapping_sub` so debug
    /// builds match C# instead of panicking.  An exhaustive check over every
    /// seed magnitude 0..=2³¹ − 1 found that the smallest magnitude that
    /// overflows there is 161 844 078, that 1 269 681 342 of the 2³¹
    /// magnitudes do, and that no other step overflows: after construction
    /// the array lies in [0, 2³¹ − 2], so `next_raw` cannot overflow.
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
                seed_array[i] = seed_array[i].wrapping_sub(seed_array[1 + n]);
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

    /// One raw `InternalSample()` step: returns a value in `[0, 2³¹ − 1)`.
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

    /// Faithful `Sample()`: the raw value mapped into `[0, 1)`.
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
///
/// Seeding follows glibc's `__srandom_r` [pubs/glibc-2.40-random_r.c]:
/// `__initstate_r` with a 128-byte state and then `__random_r`, compiled from
/// that file, match `next_raw` for 2000 outputs at each of the seeds 0, 1, 2,
/// 12345, 2³¹ − 1, 2³¹, 2³¹ + 1, 3 000 000 000 and 2³² − 1.  The macOS libc
/// `srandom` agrees at all of those except 0, 2³¹ − 1 and 2³¹ + 1: it does not
/// map seed 0 to 1, and it replaces a zero word during the table fill.
/// Current FreeBSD agrees at none of them: `srandom_r` in
/// [pubs/freebsd-0d022baa047a-random.c] fills the table with `parkmiller32`,
/// which moves each word into [1, 2³¹ − 2] before the Park–Miller step and
/// back down by one after it.
#[derive(Debug, Clone)]
pub struct BsdRandom {
    state: [u32; 31],
    fptr: usize,
    rptr: usize,
    bits: PackedBits,
}

impl BsdRandom {
    /// Construct from a 32-bit seed (`srandom()`): seed 0 is mapped to 1,
    /// the 31-word table is filled Park-Miller style on signed 32-bit words
    /// (see `park_miller31`), and 310 warm-up steps are discarded, as glibc's
    /// `__srandom_r` does.
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

    /// One raw `random()` step: returns the 31-bit value `(sum >> 1)`.
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
///
/// In [pubs/glibc-2.40-random.c], `srand` is a weak alias of `__srandom`;
/// `rand` itself is in `stdlib/rand.c`, which is not in `pubs/`.
pub type LinuxLibcRandom = BsdRandom;

// ── FreeBSD compatibility rand_r() ───────────────────────────────────────────

/// FreeBSD 12 compatibility `rand()` / current `rand_r()` core.
///
/// This is the single-word Park-Miller compatibility path kept around by
/// FreeBSD for ABI reasons: `do_rand` in [pubs/freebsd-0d022baa047a-rand.c],
/// reached through `rand_r` and the compatibility symbol `__rand_fbsd12`.
/// FreeBSD's own source calls it garbage ("Can't fix this garbage; too little
/// state").  Current FreeBSD `rand()` runs `random_r` on a TYPE_3 state
/// instead.  `rand_r` compiled from that file matches `next_raw` for 2000
/// outputs at each of the seeds 0, 1, 12345, 2³¹ − 1 and 2³² − 1.
#[derive(Debug, Clone)]
pub struct BsdRandCompat {
    state: u32,
    bits: PackedBits,
}

impl BsdRandCompat {
    /// Construct from a 32-bit seed (the `rand_r()` state word).
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed,
            bits: PackedBits::default(),
        }
    }

    /// One raw `rand_r()` step: a Park-Miller update returning a 31-bit value.
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
    /// Construct from a seed exactly as `srand48()` does: the seed fills the
    /// high 32 bits of the 48-bit state and the low 16 bits are set to 0x330E.
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

/// First five `rand()` values after `srand(1)` for the MSVC CRT recurrence
/// (`x = 214013·x + 2531011 mod 2³²`, output bits 30..16), as an independent
/// replica of that recurrence produces them.  Shared by the
/// [`WindowsMsvcRand`] and `LcgVariant::Msvc` known-answer tests.
#[cfg(test)]
pub(super) const MSVC_RAND_SEED_1_PREFIX: [u32; 5] = [41, 18_467, 6_334, 26_500, 19_169];

/// Former name of [`SystemVRand`], kept so code written against 0.5.0 still
/// compiles.
#[deprecated(note = "use SystemVRand")]
pub type CRand = SystemVRand;

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

    /// glibc's `__random_r` after `__srandom_r(1)`, compiled from
    /// [pubs/glibc-2.40-random_r.c], returns these values, and so does the
    /// macOS libc `random()` after `srandom(1)`.
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

    /// Seeds ≥ 2³¹ enter glibc's `__srandom_r` table fill as negative
    /// `int32_t` words; a word that reaches 0 (seed 2³¹ − 1) stays 0; and
    /// seed 0 is replaced by 1 before the fill.  The values first came from
    /// independent C and Python replicas of the glibc seeding loop; glibc's
    /// own `__initstate_r` and `__random_r`, compiled from
    /// [pubs/glibc-2.40-random_r.c], produce the same outputs.
    #[test]
    fn glibc_random_seeds_through_signed_int32_words() {
        const SEED_ABOVE_2_POW_31: u32 = 3_000_000_000;
        const SEED_2_POW_31_MINUS_1: u32 = i32::MAX as u32;
        let cases: [(u32, [u32; 6]); 2] = [
            (
                SEED_ABOVE_2_POW_31,
                [
                    2_058_147_116,
                    854_483_408,
                    922_419_988,
                    286_396_165,
                    2_068_523_933,
                    1_172_167_191,
                ],
            ),
            (
                SEED_2_POW_31_MINUS_1,
                [
                    1_065_668_062,
                    2_142_264_300,
                    1_066_566_375,
                    1_064_012_770,
                    2_141_034_222,
                    1_065_509_725,
                ],
            ),
        ];
        for (seed, want) in cases {
            let mut rng = LinuxLibcRandom::new(seed);
            let got = want.map(|_| rng.next_raw());
            assert_eq!(got, want, "seed {seed}");
        }

        let mut zero = LinuxLibcRandom::new(0);
        let mut one = LinuxLibcRandom::new(1);
        for _ in 0..8 {
            assert_eq!(zero.next_raw(), one.next_raw());
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

    /// `rand_r` compiled from [pubs/freebsd-0d022baa047a-rand.c] returns
    /// these values for seed 1.
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

    /// `srand48(1)` then `mrand48()`.  The reference is the macOS libc: the
    /// values come from a scratch C program that calls its `srand48` and
    /// `mrand48` and prints the signed results.
    #[test]
    fn rand48_matches_macos_libc_mrand48_seed_1() {
        let expected: [i32; 8] = [
            178_800_969,
            1_952_030_186,
            -709_454_646,
            1_443_049_011,
            -1_866_208_802,
            7_588_830,
            805_690_840,
            -41_085_314,
        ];
        let mut rng = Rand48::new(1);
        let got = expected.map(|_| rng.next_u32() as i32);
        assert_eq!(got, expected);
    }

    #[test]
    fn windows_msvc_rand_matches_known_seed_1_prefix() {
        let mut rng = WindowsMsvcRand::new(1);
        for want in super::MSVC_RAND_SEED_1_PREFIX {
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

    /// These seeds overflow the correction rounds' subtraction, which C#
    /// evaluates unchecked (the smallest overflowing magnitude is
    /// 161 844 078).  Expected values come from a replica of the .NET
    /// Framework reference source (`random.cs`) with explicit 32-bit wrapping.
    /// No .NET runtime was available, so these four seeds are pinned against
    /// that replica alone; the replica reproduces the seed-1 prefix above and
    /// the widely cited first outputs for seeds 0 and 42, none of which
    /// overflow.  Seeds `i32::MAX` and `i32::MIN` share one stream because the
    /// reference source maps `Int32.MinValue` to `Int32.MaxValue`.
    #[test]
    fn windows_dotnet_random_wraps_like_csharp_for_large_seeds() {
        const SEED_2E9: [u32; 5] = [
            224_431_583,
            2_141_996_799,
            1_553_033_465,
            1_565_626_964,
            1_582_548_916,
        ];
        const SEED_EXTREME: [u32; 5] = [
            1_559_595_546,
            1_755_192_844,
            1_649_316_172,
            1_198_642_031,
            442_452_829,
        ];
        for (seed, expected) in [
            (2_000_000_000, SEED_2E9),
            (-2_000_000_000, SEED_2E9),
            (i32::MAX, SEED_EXTREME),
            (i32::MIN, SEED_EXTREME),
        ] {
            let mut rng = WindowsDotNetRandom::new(seed);
            for want in expected {
                assert_eq!(rng.next_raw(), want, "seed {seed}");
            }
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

    /// `CRand` stays available, deprecated, as the same generator.
    #[test]
    #[allow(deprecated)]
    fn crand_alias_is_system_v_rand() {
        let mut old = super::CRand::new(1);
        let mut new = SystemVRand::new(1);
        assert_eq!(old.next_raw(), new.next_raw());
    }
}
