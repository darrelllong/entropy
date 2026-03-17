# Benchmarks

Throughput measured with `pilot-bench` `run_program --preset normal`.

All results are in millions of 32-bit words per second (`MW/s`); 95% CI shown.
The `Dyson` column is an Apple Silicon M4 (`macOS aarch64`) with FEAT_SHA2 and
FEAT_SHA3 hardware acceleration.  The `dmz.lan` column is an Intel Core i5
(`Linux x86_64`).

To benchmark on a new machine:

```
scripts/bench_rngs.sh --preset normal --machine <name>
```

Results land in `stats/<name>/`.  On the next regeneration, this table uses
any measured files it finds there, and the radar chart annotates fallback
points if some benchmark files are still missing.

## Results

| Generator | Dyson MW/s | ±CI | dmz MW/s | ±CI |
|---|---:|---:|---:|---:|
| `OsRng (/dev/urandom)` | 1.191 | ±0.008 | 1.220 | ±0.004 |
| `MT19937 (seed=19650218)` | 641.2 | ±16.28 | 315.4 | ±0.624 |
| `Xorshift64 (seed=1)` | 646.1 | ±2.962 | 566.8 | ±1.900 |
| `Xorshift32 (seed=1)` | 647.7 | ±12.44 | 606.2 | ±2.084 |
| `BAD Unix System V rand() (seed=1)` | 441.4 | ±1.419 | 357.3 | ±1.626 |
| `BAD Unix System V mrand48() (seed=1)` | 973 | ±2.591 | 903.8 | ±5.002 |
| `BAD Unix BSD random() TYPE_3 (seed=1)` | 405.8 | ±1.471 | 306.0 | ±1.558 |
| `BAD Unix Linux glibc rand()/random() (seed=1)` | 405.8 | ±1.736 | 234.0 | ±0.737 |
| `BAD Unix FreeBSD12 rand_r() compat (seed=1)` | 189.1 | ±0.519 | 172.6 | ±0.361 |
| `BAD Windows CRT rand() (seed=1)` | 438.5 | ±12.51 | 358.8 | ±1.625 |
| `BAD Windows VB6/VBA Rnd() (seed=1)` | 379.5 | ±7.133 | 512.8 | ±2.109 |
| `BAD Windows .NET Random(seed=1) compat` | 428.4 | ±14.1 | 279.3 | ±1.142 |
| `ANSI C sample LCG (seed=1)` | 186.7 | ±2.666 | 93.70 | ±0.124 |
| `LCG MINSTD (seed=1)` | 171.6 | ±0.408 | 93.73 | ±0.169 |
| `BBS (p=2^31-1, q=4294967291)` | 61.29 | ±0.312 | 38.53 | ±0.476 |
| `Blum-Micali (p=2^31-1, g=7)` | 0.462 | ±0.002 | 0.197 | ±0.002 |
| `AES-128-CTR (NIST key)` | 137.8 | ±1.404 | 63.55 | ±0.595 |
| `Camellia-128-CTR (key=00..0f)` | 36.21 | ±0.077 | 23.58 | ±0.194 |
| `Twofish-128-CTR (key=00..0f)` | 3.512 | ±0.030 | 1.328 | ±0.004 |
| `Serpent-128-CTR (key=00..0f)` | 2.859 | ±0.018 | 1.110 | ±0.009 |
| `SM4-CTR (key=00..0f)` | 47.20 | ±0.204 | 30.71 | ±0.475 |
| `Grasshopper-CTR (key=00..1f)` | 6.723 | ±0.008 | 3.850 | ±0.016 |
| `CAST-128-CTR (key=00..0f)` | 59.50 | ±2.424 | 28.38 | ±0.564 |
| `SEED-CTR (key=00..0f)` | 18.51 | ±0.163 | 12.98 | ±0.071 |
| `Rabbit (key=00..0f, iv=00..07)` | 352.6 | ±4.358 | 127.0 | ±0.560 |
| `Salsa20 (key=00..1f, nonce=00..07)` | 201.0 | ±1.014 | 117.4 | ±0.598 |
| `Snow3G (key=00..0f, iv=00..0f)` | 136.4 | ±0.270 | 74.16 | ±2.042 |
| `ZUC-128 (key=00..0f, iv=00..0f)` | 142.5 | ±0.595 | 71.55 | ±0.626 |
| `SpongeBob (SHA3-512 chain, seed=00..3f)` | 32.22 | ±0.099 | 24.17 | ±0.201 |
| `Squidward (SHA-256 chain, seed=00..1f)` | 239.6 | ±0.574 | 25.68 | ±0.419 |
| `PCG32 (seed=42, seq=54)` | 934.1 | ±6.738 | 817.1 | ±2.848 |
| `PCG64 (state=1, seq=1)` | 843.8 | ±2.893 | 580.3 | ±2.532 |
| `Xoshiro256** (seeds=1,2,3,4)` | 1287 | ±4.56 | 927.5 | ±2.207 |
| `Xoroshiro128** (seeds=1,2)` | 902.8 | ±9.123 | 730.4 | ±1.964 |
| `WyRand (seed=42)` | 3120 | ±14.32 | 940.7 | ±3.277 |
| `SFC64 (seeds=1,2,3)` | 1262 | ±15.58 | 1001 | ±2.452 |
| `JSF64 (seed=0xdeadbeef)` | 1314 | ±23.22 | 870.6 | ±2.447 |
| `ChaCha20 CSPRNG (OsRng key)` | 170.7 | ±1.076 | 87.78 | ±1.530 |
| `HMAC_DRBG SHA-256 (OsRng seed)` | 3.218 | ±0.026 | 1.969 | ±0.026 |
| `Hash_DRBG SHA-256 (OsRng seed)` | 12.37 | ±0.425 | 7.376 | ±0.029 |
| `cryptography::CtrDrbgAes256 (seed=00..2f)` | 1.893 | ±0.010 | 1.123 | ±0.005 |
| `Constant (0xDEAD_DEAD)` | 31560 | ±108.7 | 23470 | ±394.4 |
| `Counter (0,1,2,...)` | 26300 | ±77.3 | 17630 | ±64.38 |

The synthetic ceiling generators dominate raw throughput, so the visuals use
normalized $\log_{10}(\text{MW/s})$ rather than a linear scale.  Each radar
chart shows one polygon per machine (blue for Dyson, red for dmz.lan).  The
scales are calibrated independently for each chart's throughput range.

**Fast / simulation generators** — scale anchored at sysv\_rand (441 MW/s) → $r=70$
and WyRand (3120 MW/s) → $r=270$.  `mrand48` and `sysv_rand` are included as the
fastest of the "BAD" generators; being fast does not make them good.

![Radar chart: fast/simulation generators](assets/benchmarks-radar-fast.svg)

**Slow generators** — scale anchored at Blum-Micali (0.462 MW/s) → $r=70$
and Squidward (240 MW/s) → $r=270$.  `FreeBSD rand_r` and `ANSI C LCG` land near
ChaCha20 in throughput: nearly identical speed, opposite security posture.

![Radar chart: slow generators](assets/benchmarks-radar-slow.svg)

## Generator Notes

Neither throughput column **certifies quality or safety**. For that, see the
current full-battery results in [TESTS.md](TESTS.md).

### `MT19937 (seed=19650218)`

MT19937 is the 32-bit Mersenne Twister with period $2^{19937}-1$, using the
standard twisted recurrence on a 624-word state plus tempering on output. It is
an excellent historical simulation PRNG and a useful statistical baseline, but
it is not a cryptographic generator: its state can be reconstructed from enough
output, and once the state is known the stream is completely predictable. The
speed here is respectable, and the battery results in [TESTS.md](TESTS.md)
still look broadly healthy, but that should be read as "good for classic
simulation use," not "safe for secrets."

### `Xorshift64 (seed=1)`

This is Marsaglia's 64-bit xorshift core
$x \leftarrow x \oplus (x \ll 13)$,
$x \leftarrow x \oplus (x \gg 7)$,
$x \leftarrow x \oplus (x \ll 17)$,
with the harness emitting the high 32 bits of each updated state. It is very
fast because it is just a few shifts and xors, and it often looks cleaner than
it deserves in medium-sized batteries. That does **not** make it safe: it is a
small-state linear generator over $\mathbb{F}_2$, hence predictable and
unsuitable for cryptography. The current tests do catch some structure; see
[TESTS.md](TESTS.md), but the right interpretation is still "fast historical
toy / simulation-grade only."

### `Xorshift32 (seed=1)`

This is the classic 32-bit xorshift
$x \leftarrow x \oplus (x \ll 13)$,
$x \leftarrow x \oplus (x \gg 17)$,
$x \leftarrow x \oplus (x \ll 5)$.
It is even smaller-state and more fragile than the 64-bit version, which is
why it is both extremely fast and much easier to embarrass statistically. It
is not appropriate for any security use, and the full battery in
[TESTS.md](TESTS.md) already treats it much more harshly than the stronger
generators.

### `BAD Unix System V rand() (seed=1)`

This is the classic 15-bit libc LCG
$x_{n+1} = 1103515245\times x_n + 12345 \pmod{2^{32}}$,
with output
$y_n = (x_n \gg 16) \mathbin{\&} \mathtt{0x7fff}$.
It is historically important precisely because it is bad: tiny effective
output width, linear structure, and easy predictability. It remains useful here
as a negative control and compatibility target. The benchmark shows it is fast;
[TESTS.md](TESTS.md) shows why that speed is worthless for serious use.

### `BAD Unix System V mrand48() (seed=1)`

`mrand48()` is the POSIX 48-bit LCG
$x_{n+1} = (0x5DEECE66D\times x_n + 0xB) \bmod 2^{48}$,
with the high bits returned as output. It is materially better than old
15-bit `rand()`, which is why it survives more tests and runs faster than
heavier modern CSPRNGs, but it is still just a linear congruential generator.
That means it is predictable, non-cryptographic, and inappropriate for key
material, nonces, or anything adversarial. The full battery in
[TESTS.md](TESTS.md) is the right place to see where it still bends.

### `BAD Unix BSD random() TYPE_3 (seed=1)`

BSD `random()` is the additive lagged generator with TYPE_3 state, roughly
$x_n = x_{n-3} + x_{n-31} \pmod{2^{32}}$,
followed by a right shift on output. Historically it was a real improvement
over tiny libc LCGs, and that shows up in the test counts: it often looks much
cleaner than the old 15-bit families. But it is still a weak user-space PRNG,
not a cryptographic one, and its state is recoverable. Treat the decent
throughput and relatively mild fail count as "less embarrassing bad Unix RNG,"
not as a recommendation.

### `BAD Unix Linux glibc rand()/random() (seed=1)`

On glibc, `rand()` is effectively `random()`, so this benchmark entry is the
same Berkeley-derived additive generator as BSD `random()`. Its behavior is
therefore close to the BSD line in both speed and statistical profile. It is
still a historical libc generator, still predictable, and still not safe for
security. Readers should not let the modest fail count in [TESTS.md](TESTS.md)
trick them into thinking it belongs anywhere near cryptographic use.

### `BAD Unix FreeBSD12 rand_r() compat (seed=1)`

This compatibility path is the single-word Park-Miller family:
$x_{n+1} = 16807\times x_n \pmod{2^{31}-1}$,
with a small ABI-shaped wrapper around the returned value. It exists here
because real systems shipped it, not because it is good. Like other one-word
LCGs, it is predictable and structurally weak. The benchmark page shows it is
slower than some other historical junk; [TESTS.md](TESTS.md) shows it also
fails more often than the BSD/glibc additive family.

### `BAD Windows CRT rand() (seed=1)`

This is the old MSVCRT/UCRT generator
$x_{n+1} = 214013\times x_n + 2531011 \pmod{2^{32}}$,
with output
$y_n = (x_n \gg 16) \mathbin{\&} \mathtt{0x7fff}$.
It is one of the notorious bad Windows RNGs: tiny 15-bit outputs, obvious
linearity, and trivial predictability. It is in the benchmark because it was
widely deployed in real code, not because it deserves respect. See
[TESTS.md](TESTS.md) for how the battery treats it.

### `BAD Windows VB6/VBA Rnd() (seed=1)`

VB6/VBA `Rnd()` uses a 24-bit linear state with update
$x_{n+1} = (0x43FD43FD\times x_n + 0x00C39EC3) \bmod 2^{24}$,
then scales that tiny state into a floating-point sample in $[0,1)$. This is
catastrophically small and easy to predict, which is exactly why it is one of
the most heavily destroyed entries in [TESTS.md](TESTS.md). It is a wonderful
museum piece and a terrible random number generator.

### `BAD Windows .NET Random(seed=1) compat`

Classic `.NET` `System.Random(seed)` is a subtractive generator with a
55-element table, descended from Knuth-style lagged subtraction methods rather
than a one-word LCG. That gives it more apparent statistical grace than the
CRT and VB6 generators, but it is still not a CSPRNG and was never meant to
protect secrets. The benchmark shows middling speed; the test report shows that
it can still look deceptively clean in one run. The right conclusion is
"legacy application PRNG," not "safe modern randomness."

### `ANSI C sample LCG (seed=1)`

This is the textbook sample LCG
$x_{n+1} = 1103515245\times x_n + 12345 \pmod{2^{31}}$.
It is not meant to represent some hidden good libc implementation; it is here
as the famous printed-manual recurrence that showed up in endless example code.
It is fast because it is trivial arithmetic, and it is awful because that same
arithmetic creates glaring lattice and serial structure. The battery results in
[TESTS.md](TESTS.md) are exactly the reason this entry exists.

### `LCG MINSTD (seed=1)`

MINSTD is the Park-Miller multiplicative congruential generator
$x_{n+1} = 16807\times x_n \pmod{2^{31}-1}$.
Historically it was a serious improvement over many older LCGs, and it is a
nice clean mathematical benchmark, but it is still a small-state linear
generator and still not remotely cryptographic. The throughput is unremarkable
and the full battery in [TESTS.md](TESTS.md) is very hard on it, which is the
correct modern attitude.

### `AES-128-CTR (NIST key)`

This generator emits
$Y_i = \mathrm{AES}_K(\mathrm{ctr}+i)$
in counter mode under a fixed 128-bit AES key, then slices each 128-bit block
into four 32-bit words. As a construction, AES-CTR is cryptographically strong
when the key is secret and the counter/nonce discipline is correct. In this
repository it is used as a deterministic benchmark fixture, so the key is fixed
for reproducibility, not secrecy. On Dyson (Apple M4) the 137.8 MW/s reflects
hardware AES acceleration (ARMv8 `FEAT_AES`); the throughput will differ on
x86 depending on AES-NI availability.

### `SpongeBob (SHA3-512 chain, seed=00..3f)`

`SpongeBob` hashes a variable-length seed into a 512-bit state
$x_0 = \text{SHA3-512}(\text{seed})$,
then advances by repeated hashing
$x_{i+1} = \text{SHA3-512}(x_i)$.
The adapter exposes that state as a sequential stream of 32-bit words, with a
fresh 64-byte digest every time the previous one is exhausted. This is a very
simple hash-chain CSPRNG design: no linear recurrence, no tiny hidden state,
and no claim that raw speed is the point. On Dyson, FEAT_SHA3 hardware
Keccak-f[1600] (EOR3, RAX1, BCAX intrinsics) is used automatically through
`cryptography::Sha3_512`. The first full battery in [TESTS.md](TESTS.md) looks
promising but not spotless, so the right read is "plausible modern generator,
worth more runs," not "already proved perfect."

### `Squidward (SHA-256 chain, seed=00..1f)`

Squidward is a SHA-256 hash chain, the same design as SpongeBob but with
SHA-256 replacing SHA3-512.  The state is a single 32-byte digest; each step
advances by
$x_{i+1} = \mathrm{SHA\text{-}256}(x_i)$,
and output is consumed as a sequential byte stream.  On ARM targets that expose
FEAT_SHA2 hardware acceleration, the implementation detects and uses the
`vsha256*` NEON intrinsics via the `aarch64-alt` crate, falling back to the
portable `cryptography::Sha256` path otherwise.  On Dyson (Apple M4) the
hardware path reaches 239.6 MW/s.

### `cryptography::CtrDrbgAes256 (seed=00..2f)`

This is the sibling `cryptography` crate's `CtrDrbgAes256`, i.e. an
AES-256-based CTR_DRBG in the NIST SP 800-90A style, seeded here with a fixed
48-byte test vector for repeatability. Unlike the historical libc generators,
this is meant to represent a real cryptographic design. It is among the
slowest entries in the table because it is doing full DRBG machinery rather
than just a toy recurrence, but that is the price of a serious generator. The
full-battery behavior in [TESTS.md](TESTS.md) is the relevant safety evidence.

### `Constant (0xDEAD_DEAD)`

This is not a generator in any meaningful sense: it returns the same 32-bit
word forever,
$x_n = c$ for all $n$.
It appears in the benchmark as a synthetic ceiling and as a sanity check that
the test suite really does annihilate obvious garbage. Its enormous throughput
is meaningless except as a reminder that speed alone says nothing about
randomness.

### `Counter (0,1,2,...)`

This fixture returns the deterministic arithmetic progression
$x_n = x_0 + n \pmod{2^{32}}$.
Like the constant stream, it is present to make sure the statistical battery
and the benchmark report keep a clear distinction between "fast" and "good."
It is almost as fast as the constant generator and almost maximally unsuitable
for any use that actually requires randomness; [TESTS.md](TESTS.md) shows that
plainly.

### `PCG32 (seed=42, seq=54)`

PCG32 is Melissa O'Neill's 32-bit Permuted Congruential Generator (O'Neill
2014).  The inner state is a 64-bit LCG
$s_{n+1} = s_n \cdot \mathtt{6364136223846793005} + \mathtt{inc} \pmod{2^{64}}$,
where `inc` encodes the stream selector.  The output permutation is XSH-RR:
right-shift by a rotation amount extracted from the top 5 bits, xor-shift the
result, then rotate right.  This destroys the visible linearity of the raw LCG
while adding no more state.  The reference sequence (seed=42, seq=54) matches
the C reference implementation exactly, which confirms the initialization order.
Period: $2^{64}$.

### `PCG64 (state=1, seq=1)`

PCG64 uses a 128-bit LCG multiplier
$\mathtt{47026247687942121848144207491837523525}$
with an XSL-RR output permutation: xor the two 64-bit halves, then rotate right
by the top 6 bits of the old state.  The 128-bit arithmetic is more expensive
on a 64-bit machine than the 64-bit LCG used by PCG32, which is why PCG64 reads
as slower (843.8 MW/s vs PCG32's 934.1 MW/s) despite producing 64 bits per step.
Period: $2^{128}$.

### `Xoshiro256** (seeds=1,2,3,4)`

Xoshiro256\*\* (Blackman and Vigna 2021) is a 256-bit linear generator over
$\mathbb{F}_2$ with a starstar scrambler on output.  The linear engine is
a shift-register recurrence over four 64-bit words; the output at each step is
$\mathrm{rotl}(s_1 \cdot 5,\ 7) \cdot 9$.
The starstar multiplications break the linearity that would be visible to
linear-complexity tests.  The generator passes BigCrush and PractRand beyond
32 TiB; it is not cryptographic.  Period: $2^{256}-1$; the all-zero seed is
forbidden and rejected at construction.

### `Xoroshiro128** (seeds=1,2)`

Xoroshiro128\*\* is the 128-bit sibling of Xoshiro256\*\* using the same starstar
scrambler but a two-word xoroshiro recurrence
$(s_0', s_1') = (s_0 \oplus s_1,\ s_1')$
with specific rotation constants $a=24$, $b=16$, $c=37$.  It is slightly faster
than the 256-bit version and uses half the state, at the cost of a shorter period
($2^{128}-1$) and marginally more failures in our battery (13 vs 5).  Not
cryptographic; all-zero seed forbidden.

### `WyRand (seed=42)`

WyRand (Wang Yi, wyhash v4.2, 2022) advances a 64-bit Weyl counter by a fixed
odd increment
$s_{n+1} = s_n + \mathtt{a0761d6478bd642f}_{16}$
then passes the result through the wyhash 128-bit multiply-xorfold mixer:
$\mathrm{wymix}(a,b) = \bigl((a\cdot b \bmod 2^{128}) \gg 64\bigr) \oplus (a\cdot b \bmod 2^{64})$.
The multiplication provides strong avalanche in a single instruction on
architectures with 64×64→128-bit multiply support.  Period: $2^{64}$.  Not
cryptographic; the state is trivially invertible from the output.  WyRand's
3120 MW/s on Dyson reflects Apple Silicon's high-throughput 64×64→128-bit
multiply pipeline; the `wyhash` mixer reduces to two multiply-accumulate
operations per word, which the M4 handles in one or two cycles.

### `SFC64 (seeds=1,2,3)`

SFC64 (Small Fast Counting, Chris Doty-Humphrey, PractRand) is a counter-assisted
chaotic generator with four 64-bit state words.  The recurrence is
$t = a + b + \mathtt{ctr}$,
$a' = b \oplus (b \gg 11)$,
$b' = c + (c \ll 3)$,
$c' = \mathrm{rotl}(c,24) + t$,
with the counter incremented by one each step to guarantee a period of at least
$2^{64}$.  Eighteen warm-up steps are applied after seeding per Doty-Humphrey's
recommendation.  The chaotic recurrence passes BigCrush and PractRand, and its
1262 MW/s throughput on Dyson makes it one of the fastest generators in the
suite after WyRand and the trivial ceiling fixtures.

### `JSF64 (seed=0xdeadbeef)`

JSF64 is Bob Jenkins' Small Fast generator (Jenkins 2007) with four 64-bit words:
$e = a - \mathrm{rotl}(b, 7)$,
$a' = b \oplus \mathrm{rotl}(c, 13)$,
$b' = c + \mathrm{rotl}(d, 37)$,
$c' = d + e$,
$d' = e + a'$.
The initial word is fixed at $a = \mathtt{f1ea5eed}_{16}$ and twenty warm-up
steps scatter the seed through the full four-word state.  JSF64 reaches 1314 MW/s
on Dyson, just above SFC64 (1262 MW/s); their battery counts are within one
run's statistical noise.  Not cryptographic.

### `ChaCha20 CSPRNG (OsRng key)`

This generator wraps the ChaCha20 stream cipher (Bernstein 2008) as a
pseudorandom byte source.  A 256-bit key and 96-bit nonce are drawn from
`OsRng` at construction; thereafter `keystream_block()` produces 64-byte
blocks, each costing one ChaCha20 core invocation (20 rounds over a
4×4 32-bit word state).  This is structurally identical to how Linux
`/dev/urandom` and macOS `arc4random` work internally.  Output is
computationally indistinguishable from uniform under the PRF assumption; no
reseed is implemented here because the scope is the test battery, not a
long-running daemon.  At 170.7 MW/s on Dyson it is the fastest crypto-grade
generator in the suite, about 53× faster than HMAC_DRBG.

### `HMAC_DRBG SHA-256 (OsRng seed)`

HMAC_DRBG (NIST SP 800-90A §10.1.2) is a deterministic RBG whose state is a
pair $(K, V)$ of 32-byte values updated after every generate call by
$K \leftarrow \mathrm{HMAC}(K,\ V \mathbin{\|} 0\mathrm{x00})$,
$V \leftarrow \mathrm{HMAC}(K,\ V)$,
followed by a second round with byte $0\mathrm{x01}$ to mix in the old output
$V$.  Initial seeding uses 48 bytes of `OsRng` entropy (32 bytes entropy_input
+ 16 bytes nonce).  Security rests on the pseudorandomness of HMAC-SHA-256.
The 3.218 MW/s throughput on Dyson reflects the cost of two HMAC invocations
per 32-byte output block.

### `Hash_DRBG SHA-256 (OsRng seed)`

Hash_DRBG (NIST SP 800-90A §10.1.1) uses no keying material.  The state is a
single 440-bit value $V$ (the NIST seedlen for SHA-256, Table 2) plus a
constant $C$ derived from $V$ at instantiation time.  Hashgen produces output
by hashing an incrementing counter concatenated with $V$, and after each
generate call the state is updated as
$V \leftarrow (V + \mathrm{SHA\text{-}256}(0\mathrm{x03} \mathbin{\|} V) + C + \mathtt{reseed\_counter}) \bmod 2^{440}$
using big-endian carry arithmetic.  At 12.37 MW/s on Dyson it is about 3.8×
faster than HMAC_DRBG because it replaces the two keyed-MAC steps with a
single hash per output block, and it achieved the best FAIL count of any
non-trivial generator in the battery (2 FAILs / 734 tests).
