# Using entropy

`entropy` is two things behind one `Rng` trait: a library of generators with
an exact sampling interface for programs, and the statistical batteries that
test them.  This document is the library's interface; [README.md](README.md)
covers the batteries' command lines and results.

The cryptographic generators here are standard constructions (ChaCha20 with
fast key erasure, the SP 800-90A DRBGs, AES in counter mode) and are marked
`CryptoRng`.  Passing the batteries is not what makes them fit for use; their
construction is, and every fixed or test seed in this document is public.

## The `Rng` trait

```rust
pub trait Rng {
    fn next_u32(&mut self) -> u32;
    fn next_u64(&mut self) -> u64;                     // default: (next_u32() << 32) | next_u32()
    fn fill_native(&mut self, bytes: &mut [u8]);      // little-endian next_u64 words
    fn next_f64(&mut self) -> f64;                     // [0, 1) from one next_u32
    fn collect_bits(&mut self, n: usize) -> Vec<u8>;  // bit 0 of word 0 first
    fn collect_u32s(&mut self, n: usize) -> Vec<u32>;
    fn collect_f64s(&mut self, n: usize) -> Vec<f64>;
}
```

**Words.** A 64-bit generator gives `next_u32` the high half of one output
and discards the low half, so the batteries test that projection; `--views`
in the runner tests the others.  The default `next_u64` puts the first
`next_u32` in the high half.  Byte-backed generators (the DRBGs, the hash
chains, ChaCha20Rng, `StreamRng`) read four or eight little-endian bytes
instead, and mixing `next_u32` with `next_u64` at one of their buffer
boundaries discards up to seven bytes; `AesCtr`, `CryptoCtrDrbg` and
`DualEcDrbg` read `next_u32` big-endian.  Each type's documentation states
its rule.

**Bytes.** `fill_native` is the byte interface for programs: whole
`next_u64` words, least significant byte first, a shorter final chunk
taking the low bytes of one more word.  A buffered or keystream generator
serves it from its buffer rather than a call per word, and the bytes are
the same either way.  `Sample::fill_bytes` is a different stream,
little-endian `next_u32` words, kept so the battery's projection of a
64-bit generator does not move.  On this Mac Xoshiro256 and JSF64 fill at
about 10 GiB/s through `fill_native`, and the fast-key-erasure generator
at about 850 MiB/s, the rate of cryptography's own core
(`examples/fill_throughput.rs`).

**`next_f64` uses 32 bits** and is what the batteries need.  Programs take
`Sample::unit_f64` (53 bits) or `unit_f64_dense`.

**`CryptoRng`** marks `OsRng`, `ChaCha20Rng`, `FastKeyErasureRng`,
`ThreadRng`, `HmacDrbg`, `HashDrbg` and `CryptoCtrDrbg`.  Accept
`impl CryptoRng` where a weak generator must not compile.  The marker
describes the construction, not the key.

## Constructing a generator

Every generator follows one vocabulary:

| Constructor | Meaning |
|---|---|
| `new(…)` | The generator from its state: the integers of the recurrence, the key and nonce of a cipher, the seed bytes of a hash chain |
| `from_os_rng()` | Seeded from the operating system; panics if the source fails |
| `try_from_os_rng()` | The same, returning `io::Result` |
| `with_test_seed()`, `AesCtr::with_nist_key()` | A fixed published seed or key, for reproducible runs of the battery and benchmarks; never for anything else |
| `HashDrbg::from_entropy(entropy, nonce, personalization)`, `HmacDrbg::from_entropy(…)` | SP 800-90A instantiation from explicit inputs, for known-answer tests |
| `Default` | `from_os_rng()` for the generators that have it |

`Seedable` is implemented by PCG32, PCG64, Xoshiro256, Xoroshiro128, SFC64,
JSF64, MT19937, ChaCha20Rng and FastKeyErasureRng, and adds two portable
seeds to the pair above:

```rust
pub trait Seedable: Sized {
    const SEED_BYTES: usize;
    fn from_seed_bytes(seed: &[u8]) -> Self;   // the constructor's integers, little-endian
    fn seed_from_u64(seed: u64) -> Self;       // SplitMix64 expansion of one word
    fn from_os_rng() -> Self;
    fn try_from_os_rng() -> io::Result<Self>;
}
```

An all-zero byte string, which xoshiro and xoroshiro cannot use, is replaced
by the SplitMix64 expansion of 0, so every call returns a working generator.

`OsRng::new()` and `OsRng::try_new()` open `/dev/urandom`.  On Linux the
first use reads one byte from `/dev/random`, which blocks until the kernel
pool is initialized, so `/dev/urandom` is never read unseeded; only a
successful check is remembered.  `os_random(&mut bytes)` fills a buffer the
same way.  `OsRng` is Unix-only: without FFI or a dependency there is no
portable system call for Windows.

## Random values in programs

```rust
use entropy::rng::{thread_rng, Pcg64, Sample, Seedable};

let mut rng = thread_rng();              // per-thread ChaCha20 with fast key erasure
let die = rng.range(1, 7);               // exactly uniform in 1..=6
let coin = rng.bernoulli(0.3);           // exactly probability 0.3
let x = rng.unit_f64();                  // 53-bit uniform in [0, 1)
let z = rng.normal();                    // standard normal, full tails
let mut deck: Vec<u8> = (0..52).collect();
rng.shuffle(&mut deck);                  // uniform over all orderings

let mut sim = Pcg64::seed_from_u64(42);  // a reproducible stream
```

| `Sample` method, on every generator | What it guarantees |
|---|---|
| `below(n)`, `range(lo, hi)`, `below_u128(n)` | Exactly uniform integers by Lemire's multiply-and-reject method, checked exhaustively at 12-bit words for every bound |
| `ratio(a, b)`, `bernoulli(p)` | Probability exactly a/b, or exactly the double p, ties included |
| `unit_f64()` | Uniform on the 2⁵³ multiples of 2⁻⁵³ in [0, 1) |
| `unit_f64_dense()` | A uniform real rounded down to a double: every double in [0, 1), subnormals included, with its gap's probability |
| `normal()` | Marsaglia and Tsang's ziggurat with its table derived at run time; 271 million draws a second here beside 311 million raw `next_u64` |
| `normal_inverse()` | The same law by inverting Φ from a dense uniform, 2 700 times slower, resolved to the smallest subnormal |
| `exponential()` | The ziggurat over e^{−x}, the tail drawn by the law's lack of memory |
| `exponential_inverse()` | −ln U from a dense uniform, so the tail reaches about 744 |
| `shuffle`, `partial_shuffle`, `choose`, `choose_mut` | Durstenfeld's Fisher–Yates with exact indices |
| `sample_indices`, `sample`, `sample_array` | Distinct elements, every subset equally likely, in random order (Floyd's algorithm or a partial shuffle) |
| `choose_weighted`, `sample_weighted` | Probability exactly proportional to integer weights, summed in 128 bits |
| `choose_from_iter`, `sample_from_iter` | Uniform choice from an iterator of unknown length (Vitter's Algorithm R) |
| `fill_bytes` | Little-endian `next_u32` words, the battery's projection |

The ziggurat tables are not tabulated: the equal-area recurrence is closed by
bisecting for r, and the tests check the areas, the closure, the
distribution, the moments and the tail's own law.
`examples/variate_throughput.rs` measures every method.

**`thread_rng()`** gives each thread a `FastKeyErasureRng` keyed from the
operating system: ChaCha20 whose key is replaced by the first 32 bytes of
each refill and whose served bytes are erased, so a captured state reveals
no earlier output (Bernstein, "Fast-key-erasure random-number generators",
2017).  It takes a fresh key after every 2³⁰ bytes and whenever the process
id changes, so a forked child never repeats its parent.  `ThreadRng::fill`
serves a whole request with one thread-local lookup and one reseed check,
splitting a request that would cross the 2³⁰-byte limit so the bytes after
it come from the new key.  `try_thread_rng()`, `ThreadRng::try_fill` and
`ThreadRng::try_next_u64` return the operating system's error instead of
panicking.

### Positioning and parallel streams

`Advance::advance(steps)` moves a generator forward by any number of
`next_u32` calls in time logarithmic in that number; `Streams::stream(k)`
gives the generator for stream *k*, the same for the same seed and index
whatever order the workers run in.

| Generator | `advance` | `stream(k)` |
|---|---|---|
| `Xoshiro256`, `Xoroshiro128`, `Xorshift64`, `Xorshift32`, `Mt19937` | A polynomial in the update matrix, derived from the generator: Berlekamp–Massey recovers the recurrence from a bit of the state and x^steps mod it is applied, a few hundred ordinary steps for xoshiro, one block for the Twister | Segment *k* of one sequence: 2¹²⁸ steps for xoshiro256, 2⁶⁴ for xoroshiro128 and MT19937, 2³² and 2¹⁶ for the xorshifts |
| `Pcg64`, `Pcg32` | The LCG's affine state map composed `steps` times by square-and-multiply | Segment *k* of 2⁶⁴ or 2³² steps on the same PCG stream |
| `ChaCha20Rng` | The block counter and offset are set | The same key under the nonce with *k* XORed into its low bytes, from block 0: a separate 2³²-block keystream per index |
| `Sfc64`, `Jsf64` | Not implemented: chaotic maps have no shortcut | A separate sequence seeded by SplitMix64 from the state and *k*: distinct and reproducible, but not provably disjoint |

The jump polynomials are recovered from the generators themselves and
cached, and each `advance` is tested against a million real steps.

### Value stability

Every generator, `Sample` method and `Seedable` derivation produces the same
values from the same seed in every release; known-answer tests pin them, and
a change to any of them is a breaking change.  `thread_rng` is seeded from
the operating system and has no stable values.

## Seeding utilities (`entropy::seed`)

These exist for the battery and the tests, and nothing here is secret.

- `seed_material::<N>(seed: u64) -> [u8; N]`: SplitMix64 expansion of one
  word, XORed first with a fixed constant so that seed 0 is not the all-zero
  state.
- `sequential_bytes::<N>() -> [u8; N]`: `00 01 02 …`, the keys and IVs of
  the published test vectors: `K16`, `K32`, `IV8`, `IV16`.
- `splitmix64(&mut state) -> u64`: the mixer itself.

A cipher or DRBG keyed with any of these in a program is broken by
construction; key it from `OsRng`.

## The generators

### Degenerate controls

| Type | Construction |
|---|---|
| `ConstantRng` | `ConstantRng::new(value)`: the same word forever |
| `CounterRng` | `CounterRng::new(start)`: 0, 1, 2, … |

They must fail every test.

### Historical generators

| Type | Notes |
|---|---|
| `Lcg32::ansi_c(seed)` | The ANSI C sample LCG (1103515245, 12345); its 31-bit output is zero-extended, so bit 31 is always 0 |
| `Lcg32::minstd(seed)` | MINSTD, a = 48 271 (C++ `minstd_rand`); `LcgVariant::Minstd0` is the original a = 16 807; a seed ≡ 0 (mod 2³¹ − 1) is remapped to 1 |
| `Lcg32::new(LcgVariant::Borland, seed)` | Borland C++ `rand()`: the 15-bit raws are packed into 32-bit words by `next_u32`; `next_raw()` gives the C value |
| `SystemVRand`, `Rand48`, `BsdRandom`, `LinuxLibcRandom`, `BsdRandCompat` | Unix libc generators, each `new(seed)` |
| `WindowsMsvcRand`, `WindowsVb6Rnd`, `WindowsDotNetRandom` | Windows-family generators, each `new(seed)` |

Every one is recoverable from a short output window.

### Simulation generators

| Type | Construction | State |
|---|---|---|
| `Mt19937` | `new(seed)` | 19 968 bits |
| `Xorshift32`, `Xorshift64` | `new(seed)`, nonzero | 32, 64 bits |
| `Pcg32` | `new(state, seq)` | 64 + 64 bits |
| `Pcg64` | `new(state, seq)` | 128 + 128 bits |
| `Xoshiro256` | `new(s0, s1, s2, s3)`, not all zero | 256 bits |
| `Xoroshiro128` | `new(s0, s1)`, not all zero | 128 bits |
| `Sfc64` | `new(a, b, c)` | 256 bits |
| `Jsf64` | `new(seed)` | 256 bits |

All but the xorshifts implement `Seedable`.  They pass the batteries and
suit simulation; every one is invertible from its output, so none belongs
where an observer benefits from predicting the next value: session
identifiers, nonces, tokens, experiment assignment under an adversary.  The
MT19937 state is recoverable from 624 consecutive outputs.

### Cipher-based generators

`StreamRng<C>` reads a stream cipher's keystream and `BlockCtrRng<C>` runs a
block cipher in SP 800-38A counter mode from a given counter; the ciphers
come from `cryptography`.

| Generator | Key | IV | Construction |
|---|---|---|---|
| `StreamRng<Rabbit>` | 128 | 64 | `StreamRng::new(Rabbit::new(&key, &iv))` |
| `StreamRng<Salsa20>` | 256 | 64 | same |
| `StreamRng<Snow3g>` | 128 | 128 | same |
| `StreamRng<Zuc128>` | 128 | 128 | same |
| `AesCtr` | 128 | counter | `AesCtr::new(&key, counter)`; `with_nist_key()` is the SP 800-38A F.5 key at counter 0 |
| `BlockCtrRng<Camellia128>`, `<Twofish128>`, `<Serpent128>`, `<Sm4>`, `<Cast128>`, `<SeedCipher>` | 128 | counter | `BlockCtrRng::new(Camellia128::new(&key), counter)` |
| `BlockCtrRng<Grasshopper>` | 256 | counter | same |

The same key with the same IV or counter gives the same stream.

### DRBGs and hash chains

| Type | Construction | Notes |
|---|---|---|
| `ChaCha20Rng` | `new(&key, &nonce, counter)`, `from_os_rng()` | RFC 8439 keystream; 2³² blocks (256 GiB) per key and nonce, after which `advance` and the stream panic rather than wrap |
| `FastKeyErasureRng` | `new(key)`, `from_os_rng()` | What `thread_rng` uses |
| `HmacDrbg` | `from_os_rng()`, `from_entropy(…)` | HMAC-SHA-256, SP 800-90A; `generate(nbytes, additional_input)` is the discrete Generate |
| `HashDrbg` | `from_os_rng()`, `from_entropy(…)` | SHA-256, SP 800-90A |
| `CryptoCtrDrbg` | `new(&seed_material)`, `with_test_seed()` | AES-256 CTR_DRBG from `cryptography` |
| `SpongeBob` | `new(seed)`, `from_os_rng()`, `with_test_seed()` | SHA3-512 hash chain |
| `Squidward` | `new(seed)`, `from_os_rng()`, `with_test_seed()` | SHA-256 hash chain |

`HmacDrbg` and `HashDrbg` panic at the SP 800-90A reseed interval of 2⁴⁸
requests rather than continue; a long-running program reseeds by
constructing anew.

### Backdoored control

`DualEcDrbg::p256(&seed)` is Dual_EC_DRBG on P-256 with the Q point of
SP 800-90 Appendix A.1.1.  Whoever holds the discrete logarithm of Q to P
recovers the state from one 30-byte output block.
It passes the batteries, which is the point of including it; the runner
limits it to the NIST suite because three scalar multiplications per block
make the others prohibitively slow.  It must never produce anything.

## Choosing

| Goal | Generator |
|---|---|
| Random values in a program | `thread_rng()` |
| Cryptographic output under a key you hold | `FastKeyErasureRng`, `ChaCha20Rng` or `CryptoCtrDrbg`, seeded with `from_os_rng()` |
| Reproducible simulation | `Pcg64` or `Xoshiro256` from `seed_from_u64`, with `Streams` for parallel work |
| Fastest simulation, no reproducibility | `Sfc64` or `Jsf64` |
| Operating-system entropy directly | `OsRng` |
| A control that must fail | `ConstantRng`, `CounterRng` |

## Running the batteries from the library

```rust
use entropy::{diehard, dieharder, nist, rng::{Pcg64, Seedable}};

let mut rng = Pcg64::seed_from_u64(1);
let quick = false;
let results = nist::run_all(&mut rng, 16_000_000);          // bits
let results = diehard::run_all(&mut rng, 16_000_000, quick); // 32-bit words
let results = dieharder::run_all(&mut rng, 16_000_000, quick);
```

Each result carries its name, status, p-value, statistic and a note, and
prints as one line or as JSON.  Every test is also a function on a bit or
word slice in its own module, for example `nist::spectral::spectral(&bits)`.

### The historical DIEHARD suite

`diehard::historical::run_all(&mut rng, n_words)` runs four DIEHARD tests the
default battery does not, and the runner takes them only by name:

```sh
cargo run --release -- --suite diehard-historical --rng MT19937
cargo run --release -- --test diehard_historical::operm5 --rng PCG64
```

| Result name | Test | Words |
|---|---|---|
| `diehard_historical::operm5` | Overlapping 5-permutations, χ² through the pseudoinverse of their exact covariance, df 96 | 1 000 005 |
| `diehard_historical::overlapping_sums` | Decorrelated overlapping sums of uniforms with their computed correction table, three Anderson–Darling layers | 199 000 |
| `diehard_historical::count_ones_bytes` | Count-the-1s on each of the 25 byte offsets of a word, each on its own words; 25 results | 6 400 100 |
| `diehard_historical::rank_6x8_windows` | 6×8 rank on each of the 25 byte offsets, each on its own words; 25 results | 15 000 000 |
| `diehard_historical::rank_6x8_windows_summary` | Anderson–Darling summary of those 25 p-values | (same words) |

The suite reads one capture of 16 000 000 words and reports 53 results per
generator; `--quick` does not change them.  A `--test` pattern that matches
nothing the selection runs is a usage error and exits 1.
