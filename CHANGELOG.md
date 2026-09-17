# Changelog

Releases are git tags. Entries record what a consumer must change, not
everything that moved.

## 0.6.0

### Breaking

- **`Sample::normal` is Marsaglia and Tsang's ziggurat.** It samples the same
  standard normal law as before at 270 million draws per second against 0.1
  million, measured beside 311 million raw `next_u64` and rand 0.10.2's 286
  million. A seed gives a different sequence of normals than it did.
  `Sample::normal_inverse` is the inversion of Φ from a dense uniform, whose
  values are the ones `normal` used to give and whose tails resolve to the
  smallest subnormal. The ziggurat's table is derived at run time from the
  equal-area recurrence, not tabulated from the paper.

- **`math::lgamma` is `math::ln_gamma`.** With it, `math::ln_gamma`,
  `math::regularized_incomplete_beta`, `math::student_t_quantile` and
  `math::NumericalError` moved here from rump, with the same signatures rump
  gave them. A consumer of rump's copies changes the path.

- **The test batteries are behind the default `batteries` feature.**
  `entropy::{nist, diehard, dieharder, research}` and `math::fft_magnitudes`
  need it, and it carries the only dependency the library had left,
  `rustfft`. With `--no-default-features` the crate is the generators,
  `Sample`, `Seedable`, `CryptoRng`, `math` and `result`, and links nothing
  at all. Every binary and the battery examples declare the feature.

- **Hash_DRBG, HMAC_DRBG and fast key erasure are cryptography's
  mechanisms.** `HashDrbg`, `HmacDrbg` and `FastKeyErasureRng` keep their
  names, constructors and streams; `from_entropy` now panics on an entropy
  input under 32 bytes or a nonce under 16, which the mechanism refuses.

### Added

- `Rng::fill_native`: whole `next_u64` words, little-endian, with a shorter
  final chunk taking one more word's low bytes. Generators backed by a buffer
  or a keystream serve it from there; Xoshiro256 and JSF64 fill at about
  10 GiB/s, two to four times `Sample::fill_bytes`, which stays four-byte
  `next_u32` words because the batteries read that projection.
- `ThreadRng::fill`, `try_fill` and `try_next_u64`: one thread-local,
  process-id and reseed check per request rather than per word, split at the
  2³⁰-byte reseed limit, with the operating system's error reported instead
  of a panic.
- `Xoshiro256::jump_pow2`, `stream` and the same on `Xoroshiro128`: a jump of
  2ᵏ steps in a few hundred operations, so workers take disjoint segments of
  one stream reproducibly. The jump polynomial is derived from the generator
  by Berlekamp–Massey, not tabulated.
- `math::normal_quantile_ln`, the standard normal quantile from a log
  probability, which reaches probabilities no double can hold.
- `POWER.md`: how often each NIST family detects each specified defect, and
  its false-alarm rate, from 100 streams per cell.
- `examples/battery_null.rs`, `examples/fill_throughput.rs` and
  `examples/variate_throughput.rs`: the null campaign and the two benchmark
  drivers.

### Fixed

- `math::igamc` lost the middle of its range for large shapes: `Q(10¹⁴, 10¹⁴)`
  was 0.59 instead of 0.5, because the prefactor subtracted terms of size
  a·ln a. It now takes a Stirling prefactor from a = 20 and Temme's uniform
  expansion from a = 10⁵, and agrees with R's pgamma to 2·10⁻¹⁴ from a = 20
  to 10¹⁴.
- `Sample::normal` returned ±∞ when the dense uniform gave 2⁻¹⁰⁷⁴, whose half
  rounds to zero.
- The nearest-pair sweep compared every pair of points sharing its sort axis,
  which an input of one tied slab and two outliers forces; such a run is now
  swept on the widest coordinate it has not used, 162 times faster on that
  family.
- `OsRng`'s one-time Linux pool check kept a failure for the life of the
  process; only success is remembered now, so a transient failure is retried.
