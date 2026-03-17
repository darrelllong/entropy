# USAGE — entropy RNG test suite

## Overview

`entropy` is a statistical test harness, not a production RNG library. It
implements the NIST SP 800-22, DIEHARD (Marsaglia), and DIEHARDER (Brown)
batteries against a collection of generators that spans the spectrum from
deliberately broken to cryptographically strong. The purpose is auditing and
comparison, not deployment. No generator in this crate should be copied into
production code on the basis of passing these tests alone: passing is necessary
but nowhere near sufficient for cryptographic suitability.

---

## The `Rng` Trait

Every generator implements the minimal interface in `entropy::rng`:

```rust
pub trait Rng {
    fn next_u32(&mut self) -> u32;
    fn next_u64(&mut self) -> u64;   // default: (next_u32() << 32) | next_u32()
    fn next_f64(&mut self) -> f64;   // uniform [0, 1) from 32 bits
    fn collect_bits(&mut self, n: usize) -> Vec<u8>;   // LSB-first
    fn collect_u32s(&mut self, n: usize) -> Vec<u32>;
    fn collect_f64s(&mut self, n: usize) -> Vec<f64>;
}
```

**Byte-ordering contract.** The default `next_u64` places the first `next_u32`
call in the high 32 bits. Byte-backed generators (HMAC_DRBG, Hash_DRBG,
ChaCha20Rng, Squidward) override this to read 8 little-endian bytes directly;
mixing `next_u32` and `next_u64` calls at a buffer-refill boundary in those
generators silently discards up to 7 trailing bytes.

---

## Seeding Utilities (`entropy::seed`)

### `seed_material(seed: u64) -> [u8; N]`

Expands a single 64-bit seed into `N` bytes using the Vigna splitmix64 mixer,
XOR'd first with the wyhash wyp0 prime `0xa076_1d64_78bd_642f` so that
`seed = 0` does not collapse to the all-zeros splitmix64 state. Output is
deterministic and suitable for reproducible test runs.

### `sequential_bytes<const N>() -> [u8; N]` — FOR TESTS ONLY

Produces `[0x00, 0x01, 0x02, ..., N-1 mod 256]`. This function exists solely
to initialize the fixed test-vector keys `K16`, `K32`, `IV8`, and `IV16` in
one place. **Never call this for production key material.**

### Pre-built constants

| Constant | Value | Use in this crate |
|----------|-------|-------------------|
| `K16`    | `[0x00..0x0f]` 128-bit key | test-vector only |
| `K32`    | `[0x00..0x1f]` 256-bit key | test-vector only |
| `IV8`    | `[0x00..0x07]` 64-bit IV   | test-vector only |
| `IV16`   | `[0x00..0x0f]` 128-bit IV  | test-vector only |

All four are present in every published test-vector corpus. Any cipher
initialized with them in a real deployment is immediately broken.

---

## Generator Categories

### Degenerate — zero entropy, sanity checks only

| Type | Construction |
|------|-------------|
| `ConstantRng` | `ConstantRng::new(value)` — emits the same word forever |
| `CounterRng`  | `CounterRng::new(start)` — emits 0, 1, 2, … |

These exist to verify that the statistical batteries correctly reject
non-random sequences. They must fail every test. Use them as negative controls.

---

### Legacy and broken PRNGs

| Type | Notes |
|------|-------|
| `Lcg32::ansi_c()` | ANSI C sample LCG (multiplier 1103515245, addend 12345); 15 usable bits per word |
| `Lcg32::minstd()` | MINSTD (Park–Miller); full 31-bit period but trivially invertible |
| `SystemVRand`, `Rand48`, `BsdRandom`, `LinuxLibcRandom`, `BsdRandCompat` | Historical Unix libc variants; various short periods and low-bit weaknesses |
| `WindowsMsvcRand`, `WindowsVb6Rnd`, `WindowsDotNetRandom` | Historical Windows-family generators; included as negative controls |
| `Mt19937` | Matsumoto–Nishimura 1998; 624-word state, period 2¹⁹⁹³⁷−1; passes DIEHARD but fails linear-complexity tests. **The full state is recoverable from 624 consecutive 32-bit outputs.** |

**Seeding note.** The internal state of every LCG variant and MT19937 is
trivially recoverable from a short output window. Never use any of these for
key derivation, nonce generation, or any purpose where an adversary may observe
output.

---

### Quality simulation PRNGs

These generators have no cryptographic claims but perform well on all three
batteries and are appropriate for Monte Carlo simulation and statistical testing.

| Type | Construction | State |
|------|-------------|-------|
| `WyRand`              | `WyRand::from_os_rng()`           | 64 bits |
| `Sfc64`               | `Sfc64::from_os_rng()`            | 256 bits |
| `Jsf64`               | `Jsf64::from_os_rng()`            | 256 bits |
| `Pcg32`               | `Pcg32::from_os_rng()`            | 128 bits |
| `Pcg64`               | `Pcg64::from_os_rng()`            | 128 bits |
| `Xoshiro256StarStar`  | `Xoshiro256StarStar::from_os_rng()` | 256 bits |
| `Xoroshiro128StarStar`| `Xoroshiro128StarStar::from_os_rng()` | 128 bits |
| `Xorshift32/64`       | `Xorshift32::new(seed)` — seed must be nonzero | 32/64 bits |

The `from_os_rng()` constructors draw seed bytes from `/dev/urandom` via
`OsRng`. All small-state generators (≤128 bits) are vulnerable to
birthday-bound state collisions for very long outputs; prefer Xoshiro256** or
SFC64 when output lengths exceed a few billion words.

---

### Cryptographic stream-cipher RNGs

These use the `StreamRng` adapter over a cipher primitive from the
`cryptography` crate.

| Generator | Key size | IV size | Construction |
|-----------|----------|---------|-------------|
| `StreamRng<Rabbit>`  | 128 bit | 64 bit  | `StreamRng::new(Rabbit::new(&key, &iv))` |
| `StreamRng<Salsa20>` | 256 bit | 64 bit  | `StreamRng::new(Salsa20::new(&key, &iv))` |
| `StreamRng<Snow3g>`  | 128 bit | 128 bit | `StreamRng::new(Snow3g::new(&key, &iv))` |
| `StreamRng<Zuc128>`  | 128 bit | 128 bit | `StreamRng::new(Zuc128::new(&key, &iv))` |

**Key and IV reuse is catastrophic.** Reusing a (key, IV) pair across sessions
reduces the cipher to a fixed pad; two ciphertexts XOR'd together expose the
plaintext directly. In the test harness these ciphers are all initialized with
`K16`/`K32` and `IV8`/`IV16` — sequential test vectors that must never appear
in any real deployment.

For any non-test use: generate keys with `OsRng` and generate a fresh random IV
per session. Never derive keys with `seed_material` or `sequential_bytes`.

---

### Block-cipher CTR-mode RNGs

`BlockCtrRng<C>` wraps any block cipher implementing the NIST SP 800-38A CTR
mode, starting from a given counter value.

Available block cipher variants and key sizes:

| Cipher | Key | Construction |
|--------|-----|-------------|
| `AesCtr` (NIST key) | 128 bit | `AesCtr::with_nist_key()` |
| `BlockCtrRng<Camellia128>` | 128 bit | `BlockCtrRng::new(Camellia128::new(&key), counter)` |
| `BlockCtrRng<Twofish128>`  | 128 bit | same pattern |
| `BlockCtrRng<Serpent128>`  | 128 bit | same pattern |
| `BlockCtrRng<Sm4>`         | 128 bit | same pattern |
| `BlockCtrRng<Grasshopper>` | 256 bit | `BlockCtrRng::new(Grasshopper::new(&K32), 0)` |
| `BlockCtrRng<Cast128>`     | 128 bit | same pattern |
| `BlockCtrRng<SeedCipher>`  | 128 bit | same pattern |

**Counter reuse warning.** Starting two instances with the same key and counter
value produces identical output streams. In the test harness the counter always
starts at zero; a production system must either use a unique key per session or
maintain a persistent counter that is never rewound.

---

### NIST DRBGs

| Type | Construction | Notes |
|------|-------------|-------|
| `HashDrbg`      | `HashDrbg::from_os_rng()`       | SHA-256 hash chain; NIST SP 800-90A |
| `HmacDrbg`      | `HmacDrbg::from_os_rng()`       | HMAC-SHA-256; NIST SP 800-90A |
| `CryptoCtrDrbg` | `CryptoCtrDrbg::with_test_seed()` | AES-256-CTR DRBG; test seed in harness |
| `ChaCha20Rng`   | `ChaCha20Rng::from_os_rng()`    | ChaCha20-based stream DRBG |
| `SpongeBob`     | `SpongeBob::from_os_rng()`      | SHA3-512 hash chain |
| `Squidward`     | `Squidward::from_os_rng()`      | SHA-256 hash chain |

`from_os_rng()` constructors seed from `/dev/urandom` and are safe to use in
test contexts. For long-running applications, the NIST DRBGs require periodic
reseeding (NIST SP 800-90A §8.6 specifies reseed intervals by security
strength). `CryptoCtrDrbg::with_test_seed()` uses a fixed sequential seed and
must not be used outside the test harness.

---

### Theoretical weaklings

| Type | Construction | Hazard |
|------|-------------|--------|
| `DualEcDrbg`    | `DualEcDrbg::p256(&seed)`       | **BACKDOORED** — see below |

**Dual_EC_DRBG.** The NIST P-256 Q point embedded in this standard encodes a
discrete-log trapdoor (Bernstein et al. 2014; Checkoway et al. 2014). An
adversary who knows the trapdoor scalar can recover the full internal state from
32 bytes of output and predict all future and past output. This generator is
included solely to document that it fails statistical tests and to provide a
reference implementation of a known-bad design. It must never be used to
produce any material in any context. The harness limits it to the NIST battery
because two P-256 scalar multiplications per 30-byte block make DIEHARD and
DIEHARDER runs prohibitively slow.

---

## Critical Seeding Warnings

1. **`sequential_bytes()`, `K16`, `K32`, `IV8`, `IV16` are not secret.** They
   are present verbatim in every published test-vector corpus. Any cipher or
   DRBG initialized with them is trivially broken.

2. **Never reuse a (key, IV) pair** for stream ciphers, or a (key, counter)
   starting state for block CTR generators, across independent sessions. Reuse
   completely destroys confidentiality.

3. **Cryptographic generators must be seeded from `OsRng`** or an equivalent
   OS entropy source. Seeding from `seed_material(42)` or any fixed constant
   produces a deterministic, reproducible stream — suitable for a test harness,
   fatal for a cryptographic application.

4. **MT19937 and all LCG variants are state-recoverable from output.** The
   MT19937 state is fully determined by any 624 consecutive 32-bit outputs.
   LCG states are recoverable in O(1) steps by modular arithmetic. Neither
   family provides any security against an observer.

5. **Passing statistical tests is not a security proof.** NIST SP 800-22,
   DIEHARD, and DIEHARDER test distributional uniformity. They do not test
   unpredictability, backtracking resistance, or resistance to side-channel
   analysis.

---

## Which Generator to Choose

| Goal | Generator | Notes |
|------|-----------|-------|
| Fast simulation, no reproducibility requirement | `WyRand` or `Sfc64` | Fastest generators in the suite |
| Reproducible statistical testing | `Pcg64` or `Xoshiro256**` | Seed with `seed_material(n)`; deterministic across runs |
| Cryptographic-quality output | `ChaCha20Rng` or `CryptoCtrDrbg` (AES-256) | Seed from `OsRng`; reseed periodically |
| OS entropy directly | `OsRng` | Wraps `/dev/urandom`; not buffered |
| Negative control — must fail all tests | `ConstantRng` or `CounterRng` | Sanity check that batteries are working |
| Never use for anything | `DualEcDrbg` | Known backdoor; included for reference only |

For the test harness itself, `seed_material` provides the reproducibility
needed to rerun a specific seed against a suite. For any other purpose, treat
every fixed or sequential seed as a known plaintext attack waiting to happen.
