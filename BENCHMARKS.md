# Benchmarks

Pilot throughput run for the current `entropy` worktree on `darby.local` (`Linux aarch64`), using `pilot-bench` `run_program --preset normal`.

This pass intentionally excludes `OsRng`, `BBS`, and `Blum-Micali`, per the pilot brief. It does include the historical Unix and Windows generators, including:

- System V `rand()` and `mrand48()`
- BSD `random()`
- Linux glibc `rand()/random()`
- FreeBSD `rand_r()` compatibility
- Windows CRT `rand()`
- VB6/VBA `Rnd()`
- classic `.NET Random(seed)` compatibility

Source log: `darby-pilot.log` (kept locally)

After `SpongeBob` was added, it was measured as a targeted Pilot addendum on
the same host and preset. The row below comes from that later targeted run.

## Results

Throughput is reported in millions of 32-bit words per second (`MW/s`). `Runs` is the number of Pilot samples used to hit the normal-preset confidence target.

| Generator | MW/s | 95% CI | Runs |
|---|---:|---:|---:|
| `MT19937 (seed=19650218)` | 208.8 | ±6.117 | 50 |
| `Xorshift64 (seed=1)` | 643.9 | ±31.97 | 320 |
| `Xorshift32 (seed=1)` | 678.4 | ±36.32 | 201 |
| `BAD Unix System V rand() (seed=1)` | 277.2 | ±17.89 | 111 |
| `BAD Unix System V mrand48() (seed=1)` | 522.9 | ±36.15 | 54 |
| `BAD Unix BSD random() TYPE_3 (seed=1)` | 171.9 | ±8.179 | 58 |
| `BAD Unix Linux glibc rand()/random() (seed=1)` | 172.0 | ±8.611 | 88 |
| `BAD Unix FreeBSD12 rand_r() compat (seed=1)` | 115.1 | ±6.15 | 110 |
| `BAD Windows CRT rand() (seed=1)` | 285.7 | ±16.64 | 290 |
| `BAD Windows VB6/VBA Rnd() (seed=1)` | 251.9 | ±14.94 | 50 |
| `BAD Windows .NET Random(seed=1) compat` | 136.1 | ±8.692 | 50 |
| `ANSI C sample LCG (seed=1)` | 125.6 | ±6.222 | 57 |
| `LCG MINSTD (seed=1)` | 110.9 | ±4.837 | 57 |
| `AES-128-CTR (NIST key)` | 52.16 | ±1.268 | 384 |
| `SpongeBob (SHA3-512 chain, seed=00..3f)` | 10.97 | ±0.1063 | 50 |
| `Squidward (SHA-256 chain, test seed)` | 85.0 | — | — |
| `PCG32 (seed=42, seq=54)` | 356 | — | — |
| `PCG64 (state=1, seq=1)` | 130 | — | — |
| `Xoshiro256** (seeds=1,2,3,4)` | 583 | — | — |
| `Xoroshiro128** (seeds=1,2)` | 537 | — | — |
| `WyRand (seed=42)` | 342 | — | — |
| `SFC64 (seeds=1,2,3)` | 597 | — | — |
| `JSF64 (seed=0xdeadbeef)` | 615 | — | — |
| `ChaCha20 CSPRNG (OsRng key)` | 61.0 | — | — |
| `HMAC_DRBG SHA-256 (OsRng seed)` | 1.41 | — | — |
| `Hash_DRBG SHA-256 (OsRng seed)` | 5.21 | — | — |
| `cryptography::CtrDrbgAes256 (seed=00..2f)` | 0.7516 | ±0.007658 | 50 |
| `Constant (0xDEAD_DEAD)` | 6310 | ±161.8 | 50 |
| `Counter (0,1,2,...)` | 6297 | ±151.2 | 50 |

The synthetic ceiling generators dominate raw throughput, so the visual uses normalized `log10(MW/s)` rather than a linear scale.

![Radar chart: original generator sweep](assets/benchmarks-radar.svg)

![Radar chart: new generators (PCG, xoshiro, WyRand, SFC, JSF, ChaCha20, DRBG)](assets/benchmarks-radar-new.svg)

The new-generator radar uses the same log-normalization scale as the original.
`Squidward`, `ChaCha20`, `HMAC_DRBG`, and `Hash_DRBG` are measured with
`from_os_rng()` rather than a fixed test seed; their throughput is key-agnostic
so the numbers are seed-independent.  The original radar does not yet include
the `SpongeBob` (OsRng-seeded) point.

## Generator Notes

These throughput numbers say how fast each generator emits 32-bit words on
Darby. They do **not** certify quality or safety. For that, see the current
full-battery results in [TESTS.md](TESTS.md).

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
$x_{n+1} = 1103515245\,x_n + 12345 \pmod{2^{32}}$,
with output
$y_n = (x_n \gg 16)\ \&\ 0x7fff$.
It is historically important precisely because it is bad: tiny effective
output width, linear structure, and easy predictability. It remains useful here
as a negative control and compatibility target. The benchmark shows it is fast;
[TESTS.md](TESTS.md) shows why that speed is worthless for serious use.

### `BAD Unix System V mrand48() (seed=1)`

`mrand48()` is the POSIX 48-bit LCG
$x_{n+1} = (0x5DEECE66D\,x_n + 0xB) \bmod 2^{48}$,
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
$x_{n+1} = 16807\,x_n \pmod{2^{31}-1}$,
with a small ABI-shaped wrapper around the returned value. It exists here
because real systems shipped it, not because it is good. Like other one-word
LCGs, it is predictable and structurally weak. The benchmark page shows it is
slower than some other historical junk; [TESTS.md](TESTS.md) shows it also
fails more often than the BSD/glibc additive family.

### `BAD Windows CRT rand() (seed=1)`

This is the old MSVCRT/UCRT generator
$x_{n+1} = 214013\,x_n + 2531011 \pmod{2^{32}}$,
with output
$y_n = (x_n \gg 16)\ \&\ 0x7fff$.
It is one of the notorious bad Windows RNGs: tiny 15-bit outputs, obvious
linearity, and trivial predictability. It is in the benchmark because it was
widely deployed in real code, not because it deserves respect. See
[TESTS.md](TESTS.md) for how the battery treats it.

### `BAD Windows VB6/VBA Rnd() (seed=1)`

VB6/VBA `Rnd()` uses a 24-bit linear state with update
$x_{n+1} = (0x43FD43FD\,x_n + 0x00C39EC3) \bmod 2^{24}$,
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
$x_{n+1} = 1103515245\,x_n + 12345 \pmod{2^{31}}$.
It is not meant to represent some hidden good libc implementation; it is here
as the famous printed-manual recurrence that showed up in endless example code.
It is fast because it is trivial arithmetic, and it is awful because that same
arithmetic creates glaring lattice and serial structure. The battery results in
[TESTS.md](TESTS.md) are exactly the reason this entry exists.

### `LCG MINSTD (seed=1)`

MINSTD is the Park-Miller multiplicative congruential generator
$x_{n+1} = 16807\,x_n \pmod{2^{31}-1}$.
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
for reproducibility, not secrecy. Its slower throughput is the cost of doing
real block-cipher work; its test behavior in [TESTS.md](TESTS.md) is the right
place to judge whether this fixture looks statistically healthy.

### `SpongeBob (SHA3-512 chain, seed=00..3f)`

`SpongeBob` hashes a variable-length seed into a 512-bit state
$x_0 = \text{SHA3-512}(\text{seed})$,
then advances by repeated hashing
$x_{i+1} = \text{SHA3-512}(x_i)$.
The adapter exposes that state as a sequential stream of 32-bit words, with a
fresh 64-byte digest every time the previous one is exhausted. This is a very
simple hash-chain CSPRNG design: no linear recurrence, no tiny hidden state,
and no claim that raw speed is the point. On Darby it lands well below
`AES-128-CTR` in throughput but far above the heavyweight
`CtrDrbgAes256` adapter. The first full battery in [TESTS.md](TESTS.md) looks
promising but not spotless, so the right read is “plausible modern generator,
worth more runs,” not “already proved perfect.”

### `cryptography::CtrDrbgAes256 (seed=00..2f)`

This is the sibling `cryptography` crate's `CtrDrbgAes256`, i.e. an
AES-256-based CTR_DRBG in the NIST SP 800-90A style, seeded here with a fixed
48-byte test vector for repeatability. Unlike the historical libc generators,
this is meant to represent a real cryptographic design. It is by far the
slowest entry in the table because it is doing full DRBG machinery rather than
just a toy recurrence, but that is the price of a serious generator. The
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

### `Squidward (SHA-256 chain, test seed)`

Squidward is a SHA-256 hash chain, the same design as SpongeBob but with
SHA-256 replacing SHA3-512.  The state is a single 32-byte digest; each step
advances by
$x_{i+1} = \mathrm{SHA\text{-}256}(x_i)$,
and output is consumed as a sequential byte stream.  On ARM targets that expose
FEAT_SHA2 hardware acceleration, the implementation detects and uses the
`vsha256*` NEON intrinsics via the `aarch64-alt` crate, falling back to the
portable `cryptography::Sha256` path otherwise.  The Darby ARM board provides
FEAT_SHA2 and reaches 85 MW/s, compared to 11 MW/s for SpongeBob's SHA3-512
chain — a 7.8× speedup from hardware SHA-256.  On Apple Silicon (M-series) the
same hardware path reaches ~242 MW/s.

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
as slower (130 MW/s) despite producing 64 bits per step.  Period: $2^{128}$.

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
cryptographic; the state is trivially invertible from the output.

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
615 MW/s throughput on Darby ARM makes it the fastest generator in the suite
after the trivial ceiling fixtures.

### `JSF64 (seed=0xdeadbeef)`

JSF64 is Bob Jenkins' Small Fast generator (Jenkins 2007) with four 64-bit words:
$e = a - \mathrm{rotl}(b, 7)$,
$a' = b \oplus \mathrm{rotl}(c, 13)$,
$b' = c + \mathrm{rotl}(d, 37)$,
$c' = d + e$,
$d' = e + a'$.
The initial word is fixed at $a = \mathtt{f1ea5eed}_{16}$ and twenty warm-up
steps scatter the seed through the full four-word state.  JSF64 reaches 615 MW/s
on Darby, matching SFC64; their battery counts (12 vs 10 FAILs) are within
one run's statistical noise.  Not cryptographic.

### `ChaCha20 CSPRNG (OsRng key)`

This generator wraps the ChaCha20 stream cipher (Bernstein 2008) as a
pseudorandom byte source.  A 256-bit key and 96-bit nonce are drawn from
`OsRng` at construction; thereafter `keystream_block()` produces 64-byte
blocks, each costing one ChaCha20 core invocation (20 rounds over a
4×4 32-bit word state).  This is structurally identical to how Linux
`/dev/urandom` and macOS `arc4random` work internally.  Output is
computationally indistinguishable from uniform under the PRF assumption; no
reseed is implemented here because the scope is the test battery, not a
long-running daemon.  At 61 MW/s it is the fastest crypto-grade generator in the
suite, about 10× faster than HMAC_DRBG.

### `HMAC_DRBG SHA-256 (OsRng seed)`

HMAC_DRBG (NIST SP 800-90A §10.1.2) is a deterministic RBG whose state is a
pair $(K, V)$ of 32-byte values updated after every generate call by
$K \leftarrow \mathrm{HMAC}(K,\ V \mathbin\| 0\mathrm{x00})$,
$V \leftarrow \mathrm{HMAC}(K,\ V)$,
followed by a second round with byte $0\mathrm{x01}$ to mix in the old output
$V$.  Initial seeding uses 48 bytes of `OsRng` entropy (32 bytes entropy_input
+ 16 bytes nonce).  Security rests on the pseudorandomness of HMAC-SHA-256.
The 1.4 MW/s throughput reflects the cost of two HMAC invocations per 32-byte
output block.

### `Hash_DRBG SHA-256 (OsRng seed)`

Hash_DRBG (NIST SP 800-90A §10.1.1) uses no keying material.  The state is a
single 440-bit value $V$ (the NIST seedlen for SHA-256, Table 2) plus a
constant $C$ derived from $V$ at instantiation time.  Hashgen produces output
by hashing an incrementing counter concatenated with $V$, and after each
generate call the state is updated as
$V \leftarrow (V + \mathrm{SHA\text{-}256}(0\mathrm{x03} \mathbin\| V) + C + \mathtt{reseed\_counter}) \bmod 2^{440}$
using big-endian carry arithmetic.  At 5.2 MW/s it is about 4× faster than
HMAC_DRBG because it replaces the two keyed-MAC steps with a single hash per
output block, and it achieved the best FAIL count of any non-trivial generator
in the battery (2 FAILs / 734 tests).
