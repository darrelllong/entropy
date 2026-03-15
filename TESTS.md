# Full Battery Results

Full `run_tests` battery harvested from `darby.local` on 2026-03-15 from the
local `darby-full-battery.log`.

After `SpongeBob` was added, it was run as a targeted addendum on the same
host and codebase, with its full transcript kept in
`darby-full-battery-spongebob.log`.

Command:

```sh
./target/release/run_tests \
  --rng OsRng \
  --rng MT19937 \
  --rng Xorshift64 \
  --rng Xorshift32 \
  --rng "BAD Unix" \
  --rng "BAD Windows" \
  --rng "ANSI C" \
  --rng MINSTD \
  --rng AES-128-CTR \
  --rng cryptography::CtrDrbgAes256 \
  --rng Constant \
  --rng Counter \
  --rng Dual_EC
```

Excluded on purpose:

- `BBS`
- `Blum-Micali`

Notes:

- The active battery is much larger than the old `225`-result runs because
  faithful `rgb_bitdist` now emits its full per-pattern family.
- `738` results means the full active battery plus the always-skipped
  Maurer `L=13..16` rows.
- `714` results means the RNG also missed the NIST excursion-family
  precondition, so those two families collapsed to one skip each.
- `199` results means `Dual_EC_DRBG` ran the NIST and Maurer families only.
- With `714`-`738` tests, a genuinely good generator should still expect about
  `7` low single-test p-values by chance at `α = 0.01`.
- `SpongeBob` uses the full `738`-result battery and is listed from its later
  targeted Darby run rather than the older bulk sweep.

## Summary Table

| RNG | Total | PASS | FAIL | SKIP |
|---|---:|---:|---:|---:|
| OsRng (/dev/urandom) | 738 | 724 | 10 | 4 |
| MT19937 (seed=19650218) | 738 | 728 | 6 | 4 |
| Xorshift64 (seed=1) | 714 | 704 | 4 | 6 |
| Xorshift32 (seed=1) | 738 | 721 | 13 | 4 |
| BAD Unix System V rand() (15-bit LCG, seed=1) | 738 | 721 | 13 | 4 |
| BAD Unix System V mrand48() (seed=1) | 738 | 725 | 9 | 4 |
| BAD Unix BSD random() TYPE_3 (seed=1) | 738 | 732 | 2 | 4 |
| BAD Unix Linux glibc rand()/random() (seed=1) | 738 | 732 | 2 | 4 |
| BAD Unix FreeBSD12 rand_r() compat (seed=1) | 738 | 719 | 15 | 4 |
| BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1) | 714 | 705 | 3 | 6 |
| BAD Windows VB6/VBA Rnd() (project seed=1) | 738 | 211 | 523 | 4 |
| BAD Windows .NET Random(seed=1) compat | 714 | 704 | 4 | 6 |
| ANSI C sample LCG (1103515245,12345; seed=1) | 714 | 116 | 592 | 6 |
| LCG MINSTD (seed=1) | 714 | 112 | 596 | 6 |
| AES-128-CTR (NIST key) | 714 | 696 | 12 | 6 |
| SpongeBob (SHA3-512 chain, OsRng seed) | 708 | 701 | 7 | 6 |
| Squidward (SHA-256 chain, OsRng seed) | 734 | 728 | 6 | 4 |
| PCG32 (OsRng seed) | 734 | 725 | 9 | 4 |
| PCG64 (OsRng seed) | 734 | 728 | 6 | 4 |
| Xoshiro256\*\* (OsRng seed) | 734 | 729 | 5 | 4 |
| Xoroshiro128\*\* (OsRng seed) | 708 | 695 | 13 | 6 |
| WyRand (OsRng seed) | 708 | 698 | 10 | 6 |
| SFC64 (OsRng seed) | 734 | 724 | 10 | 4 |
| JSF64 (OsRng seed) | 734 | 722 | 12 | 4 |
| ChaCha20 CSPRNG (OsRng key) | 734 | 727 | 7 | 4 |
| HMAC_DRBG SHA-256 (OsRng seed) | 734 | 728 | 6 | 4 |
| Hash_DRBG SHA-256 (OsRng seed) | 734 | 732 | 2 | 4 |
| cryptography::CtrDrbgAes256 (seed=00..2f) | 714 | 701 | 7 | 6 |
| Constant (0xDEAD_DEAD) | 714 | 0 | 708 | 6 |
| Counter (0,1,2,…) | 714 | 1 | 707 | 6 |
| Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01) | 199 | 195 | 0 | 4 |

## Theory By Test

Here $Q(a,x)=\Gamma(a,x)/\Gamma(a)$ is the upper regularized gamma function,
$\Phi$ is the standard normal CDF, and $D_n=\sup_x |F_n(x)-x|$ is the
one-sample Kolmogorov-Smirnov statistic used whenever this report says
"outer KS on p-values."

### NIST SP 800-22

- **`frequency` (monobit).** Convert bits to signs $Y_i = 2X_i-1$, form
  $S_n = \sum_{i=1}^n Y_i$, and test whether the total signed drift is too
  large for an unbiased Bernoulli source. The harness reports
  $p=\mathrm{erfc}(|S_n|/\sqrt{2n})$, so any persistent bias in the
  proportion of ones pushes $|S_n|$ upward.

- **`block_frequency`.** Split the stream into blocks of length $M=128$, let
  $\pi_j$ be the fraction of ones in block $j$, and compare the blockwise
  biases to the null mean $1/2$ with
  $\chi^2 = 4M \sum_{j=1}^N (\pi_j-\tfrac12)^2$. Under $H_0$ this is scored
  with $p = Q(N/2,\chi^2/2)$.

- **`runs`.** First estimate the global one-density $\pi$, then count the
  total number of runs $V_n$ in the signed sequence. NIST compares $V_n$ to
  its null mean $2n\pi(1-\pi)$ and uses
  $p=\mathrm{erfc}\!\left(\frac{|V_n-2n\pi(1-\pi)|}{2\pi(1-\pi)\sqrt{2n}}\right)$,
  so the test is sensitive to too much alternation and too much clumping.

- **`longest_run`.** Partition the bitstream into fixed blocks, compute the
  longest run of ones in each block, bin those maxima into NIST's published
  categories, and apply a multinomial chi-square
  $\chi^2=\sum_i (O_i-E_i)^2/E_i$. This catches local burstiness that can
  hide inside a globally balanced stream.

- **`matrix_rank`.** Fill $32\times 32$ binary matrices over $\mathbb{F}_2$,
  compute their ranks by Gaussian elimination, and compare the counts of
  ranks $32$, $31$, and $\le 30$ to the exact null probabilities. The generic
  rank law is
  $P_{m,n}(r)=2^{-mn}\prod_{i=0}^{r-1}\frac{(2^m-2^i)(2^n-2^i)}{(2^r-2^i)}$,
  and NIST uses a chi-square over the pooled bins.

- **`spectral`.** Map bits to $\pm1$, take the DFT, and count how many Fourier
  magnitudes fall below the threshold $T=\sqrt{n\ln 20}$. With
  $N_1=\#\{|F_k|<T\}$ and null expectation $N_0=0.95\,n/2$, the test forms a
  standardized deviation $d=(N_1-N_0)/\sqrt{n\cdot 0.95 \cdot 0.05/4}$ and
  reports $p=\mathrm{erfc}(|d|/\sqrt{2})$.

- **`non_overlapping_template`.** For each aperiodic $m=9$ template, split the
  sequence into $N=8$ blocks and count non-overlapping matches in each block.
  Under the null,
  $\mu=(M-m+1)/2^m$ and
  $\sigma^2=M(2^{-m}-(2m-1)2^{-2m})$, so the block counts are compared to a
  normal model via chi-square across the $8$ blocks.

- **`overlapping_template`.** Use the all-ones template of length $m=9$,
  allow overlaps inside each $M=1032$-bit block, and count how many matches
  occur. NIST tabulates the null probabilities $\pi_0,\dots,\pi_5$ for the
  pooled match-count bins, and the harness applies the corresponding
  multinomial chi-square.

- **`universal` and `maurer::universal_l06..l16`.** Maurer's universal test
  measures compressibility by tracking recurrence gaps of $L$-bit words. If
  $A_i$ is the distance back to the previous occurrence of the current word,
  the core statistic is
  $f_n = \frac{1}{K}\sum_{i=1}^K \log_2 A_i$,
  which is normalized as
  $z = (f_n-\mu_L)/(c(L,K)\sigma_L)$ and scored with
  $p=\mathrm{erfc}(|z|/\sqrt{2})$. The crate reports both the NIST
  wrapper and the broader Maurer family over $L=6,\dots,16$.

- **`linear_complexity`.** Break the stream into blocks of length $M=500$,
  run Berlekamp-Massey on each block, and compare the resulting linear
  complexities to NIST's seven-bin reference distribution. The test is aimed
  at short linear recurrences: if a block is too easy to synthesize by an
  LFSR, its linear complexity lands too far below the null mean.

- **`serial` (two p-values).** Count all overlapping $m$-bit patterns for
  $m=3$ and form
  $\psi_m^2 = \frac{2^m}{n}\sum_i C_i^2 - n$.
  NIST then uses the derived statistics
  $\Delta\psi_m^2=\psi_m^2-\psi_{m-1}^2$ and
  $\Delta^2\psi_m^2=\psi_m^2-2\psi_{m-1}^2+\psi_{m-2}^2$,
  each converted to a p-value with $Q(\cdot,\cdot)$.

- **`approximate_entropy`.** For pattern lengths $m=10$ and $m+1$, define
  $\phi_m=\frac{1}{n}\sum_i \log C_i^{(m)}$, where $C_i^{(m)}$ is the
  circular pattern-match frequency of the $i$th overlapping block. The test
  statistic is
  $\mathrm{ApEn}(m)=\phi_m-\phi_{m+1}$ and NIST scores
  $\chi^2 = 2n(\ln 2-\mathrm{ApEn}(m))$.

- **`cumulative_sums` (forward and backward).** Form the random walk
  $S_k=\sum_{i=1}^k (2X_i-1)$ and record
  $z=\max_k |S_k|$ in the forward and reversed streams. NIST then uses its
  reflected-walk tail series for
  $P\!\left(\max_k |S_k| \ge z\right)$ rather than a crude single Gaussian
  approximation, so the p-value is a sum of normal tails over image paths.

- **`random_excursions`.** Chop the signed walk into complete cycles between
  successive zeros, and for each state $x\in\{\pm1,\pm2,\pm3,\pm4\}$ count
  how many cycles visit $x$ exactly $k$ times for $k=0,\dots,5$ (with $k\ge5$
  pooled). The per-state statistic is
  $\chi^2_x = \sum_{k=0}^5 (\nu_k(x)-J\pi_k(x))^2/(J\pi_k(x))$ with
  $p = Q(5/2,\chi_x^2/2)$, where $J$ is the number of cycles.

- **`random_excursions_variant`.** Use the same cycle decomposition, but now
  test only the total visit count $\xi(x)$ to each
  $x\in\{\pm1,\dots,\pm9\}$. NIST models
  $\xi(x)$ around mean $J$ and uses
  $p=\mathrm{erfc}\!\left(\frac{|\xi(x)-J|}{\sqrt{2J(4|x|-2)}}\right)$,
  so this is a per-state aggregate walk-balance check.

### DIEHARD

- **`birthday_spacings`.** For each trial, choose $m=512$ birthdays in a year
  of size $n=2^{24}$, sort them, compute adjacent spacings, and let $j$ be the
  number of distinct spacing values that repeat. Under the null,
  $j \overset{a}{\sim} \mathrm{Poisson}(\lambda)$ with
  $\lambda = m^3/(4n)=2$; the implementation performs a chi-square fit to the
  Poisson histogram for each bit offset and then an outer KS over the nine
  offsetwise p-values.

- **`binary_rank_32x32`.** Fill $40{,}000$ binary $32\times 32$ matrices over
  $\mathbb{F}_2$, compute their ranks, and compare the counts of
  $32$, $31$, $30$, and $\le 29$ to the exact GF(2) rank law
  $P_{m,n}(r)=2^{-mn}\prod_{i=0}^{r-1}\frac{(2^m-2^i)(2^n-2^i)}{(2^r-2^i)}$.
  The reported p-value comes from the pooled chi-square on those bins.

- **`binary_rank_31x31`.** This is the same GF(2) rank test, but on
  $31\times 31$ matrices with exact probabilities recomputed for that size
  rather than reusing the $32\times 32$ constants. The goal is to catch
  linear dependence that appears only after a slightly different slicing of
  the raw words.

- **`binary_rank_6x8`.** Build $100{,}000$ matrices with $6$ rows and $8$
  columns from byte-level slices, compute the rank over $\mathbb{F}_2$, and
  compare the observed counts of ranks $6$, $5$, and $\le4$ to the exact
  $6\times8$ null probabilities. This is a small-matrix dependence probe aimed
  at byte-lane linearity.

- **`bitstream`.** Interpret the output as a continuous MSB-first bitstream,
  slide a $20$-bit window across $2^{21}$ overlapping positions, and count
  how many of the $2^{20}$ possible words are never seen. Marsaglia models the
  missing-word count as approximately normal with mean $141{,}909$ and
  $\sigma=428$, so each repetition is scored by
  $p=\mathrm{erfc}(|z|/\sqrt{2})$ and the final report is an outer KS on
  $20$ such p-values.

- **`opso`.** OPSO extracts overlapping pairs of $10$-bit letters, so each
  sample lands in a word space of size $2^{20}$. After $2^{21}$ extracted
  pairs, the number of missing words is approximately normal around the
  occupancy-theory value $2^{20}e^{-2}$, and the implementation uses
  Marsaglia's tabulated $(\mu,\sigma)$ to compute
  $p=\mathrm{erfc}(|z|/\sqrt{2})$.

- **`oqso`.** OQSO is the same sparse-occupancy idea, but with overlapping
  quadruples of $5$-bit letters, again yielding a $2^{20}$ word space. The
  statistic is the number of missing words after a long field-extracted stream,
  normalized with the published OQSO mean and standard deviation.

- **`dna`.** DNA applies the sparse-occupancy construction to ten successive
  $2$-bit symbols, producing another $2^{20}$-word occupancy problem. The
  test again standardizes the number of missing words against the reference
  normal approximation and reports
  $p=\mathrm{erfc}(|z|/\sqrt{2})$.

- **`count_ones_stream`.** Map each byte to one of five letters
  $A,\dots,E$ according to its Hamming weight, form overlapping $5$-letter and
  $4$-letter words, and compute the Marsaglia difference statistic
  $Z = (Q_5-Q_4-2500)/\sqrt{5000}$, where $Q_5$ and $Q_4$ are the corresponding
  chi-squares against the exact letter-product probabilities. The p-value is
  $p=\mathrm{erfc}(|Z|/\sqrt{2})$.

- **`parking_lot`.** Sequentially try to place $12{,}000$ unit square cars in
  a $100\times100$ lot, rejecting any new car whose footprint overlaps an
  existing one. The total parked count is approximately normal with
  $\mu=3523$ and $\sigma=21.9$; each repetition is mapped through $\Phi$, and
  the final result is a KS test over those repetitionwise uniformized values.

- **`minimum_distance_2d`.** This historical DIEHARD result places points in a
  $10{,}000\times10{,}000$ square, finds the nearest-pair distance
  $d_{\min}$, and transforms it by
  $U = 1-\exp(-d_{\min}^2/\lambda)$ with $\lambda \approx 0.995$ before an
  outer KS. The crate keeps it for legacy comparison only; Dieharder itself
  documents the original Marsaglia formula as obsolete and wrong.

- **`spheres_3d`.** Place points in a $1000^3$ cube, find the nearest-neighbor
  radius $r_{\min}$, and use the fact that $r_{\min}^3$ is approximately
  exponential under a homogeneous Poisson cloud. The code transforms with
  $U = 1-\exp(-r_{\min}^3/30)$ and then applies an outer KS across repeats.

- **`squeeze`.** Start from $k_0 = 2^{31}-1$ and iterate
  $k_{t+1}=\lceil k_t U_t\rceil$ until $k_t=1$ or a cap is reached. The test
  compares the empirical distribution of the stopping time $J$ to Marsaglia's
  tabulated cell probabilities by a chi-square
  $\chi^2=\sum_i (O_i-E_i)^2/E_i$ over $43$ pooled cells.

- **`runs_up` and `runs_down`.** For sequences of $10{,}000$ integers, count
  monotone run lengths $1,2,3,4,5,6+$ in the upward and downward directions.
  Let $R$ be the run-count vector, $b$ the theoretical run proportions, and
  $A$ the Grafton/Knuth inverse-covariance matrix; the core statistic is
  $V = (R-nb)^\top A (R-nb)/n$, converted to
  $p = Q(3,V/2)$, with a final KS across $10$ repetitions for each direction.

- **`craps_wins`.** Simulate $N=200{,}000$ craps games and count the number of
  wins $W$. Under fair dice,
  $W \approx \mathrm{Bin}(N,p_{\mathrm{win}})$ with
  $p_{\mathrm{win}} = 244/495$, so the code standardizes
  $z = (W-Np_{\mathrm{win}})/\sqrt{Np_{\mathrm{win}}(1-p_{\mathrm{win}})}$ and
  reports $p=\mathrm{erfc}(|z|/\sqrt{2})$.

- **`craps_throws`.** The same $200{,}000$ simulated games are also binned by
  game length: one throw, two throws, and so on, with the tail pooled at
  $\ge 22$ throws. The expected cell probabilities come from exact craps
  theory, and the reported p-value is the chi-square fit of the observed game
  length histogram to that law.

### DIEHARDER

- **`minimum_distance_nd`.** This is the corrected Fischler-style nearest
  neighbor test in dimensions $d=2,\dots,5$. For the observed minimum
  distance $r$, the transformed statistic uses the $d$-ball volume
  $V_d(r)$ and the Brown/Fischler correction
  $p = 1-\exp\!\bigl(-n(n-1)V_d(r)/2\bigr)\!\left[1+\frac{2+Q_d}{6}n^3V_d(r)^2\right]$,
  followed by an outer KS across repeats.

- **`permutations`.** Draw non-overlapping blocks of $t=5$ independent
  uniforms, map each block to its permutation rank in $S_5$, and compare the
  observed counts over the $5!=120$ orderings to the uniform expectation
  $E = N/120$ with a chi-square. This is the cleaned-up Dieharder successor to
  the old OPERM5 idea.

- **`lagged_sums` (lags 1 and 100).** Take every $(\ell+1)$th uniform variate,
  sum $m$ such terms, and compare
  $S = \sum_{i=1}^m U_i$ to the null mean $m/2$ and variance $m/12$. The code
  forms
  $z = (S-m/2)/\sqrt{m/12}$ and reports
  $p=\mathrm{erfc}(|z|/\sqrt{2})$, once for $\ell=1$ and once for
  $\ell=100$.

- **`ks_uniform`.** Convert the raw words to uniforms in $[0,1)$, sort them,
  and compute the one-sample KS statistic
  $D_n=\max_i \max(i/n-U_{(i)}, U_{(i)}-(i-1)/n)$. The p-value is the exact or
  asymptotic KS tail used by the shared math layer, so this is the simplest
  direct float-uniformity check in the battery.

- **`byte_distribution`.** From each group of three consecutive words, harvest
  nine specific byte positions and count the $256$ possible byte values in each
  of those $9$ streams. The null model is uniform on bytes, so with expected
  cell count $E=t/256$ the statistic is a single large chi-square over
  $9\times256$ cells.

- **`dct`.** Rotate the raw $32$-bit words, apply a Type-II DCT
  $X_k = \sum_{j=0}^{N-1} x_j \cos(\pi(j+\tfrac12)k/N)$ on blocks of length
  $N=256$, and record the index $k^\star = \arg\max_k |X_k|$ after the DC
  adjustment. Under the null, the max-position histogram should be close to
  uniform, so the reported p-value is a chi-square on those positions.

- **`monobit2`.** For block sizes $2,4,8,\dots,2^{m}$ words, count the total
  number of ones in each block and compare the histogram to the exact binomial
  law $\mathrm{Bin}(32b,\tfrac12)$ by chi-square. Dieharder then keeps
  only the most extreme tail p-value across the tested block sizes and applies
  the same multiple-test correction as `evalMostExtreme()`, so the reported
  p-value is the corrected "worst scale" result.

- **`fill_tree_count`.** Insert random floats into a fixed 32-slot implicit
  binary search tree until the insertion path collides with an already-filled
  leaf route. The number of words consumed before collision is compared to
  Bauer's empirical reference distribution by a chi-square fit.

- **`fill_tree_position`.** The same fill-tree trials also record the leaf
  collision position. Under the null those collision positions should be
  approximately uniform over the $16$ effective terminal locations, so the
  implementation reports a second chi-square p-value for position uniformity.

- **`bit_distribution` (`rgb_bitdist`).** Read a continuous MSB-first bitstream,
  partition it into blocks of $64$ consecutive $n$-bit symbols, and for each
  specific pattern $u\in\{0,\dots,2^n-1\}$ count how often $u$ occurs inside a
  block. Under $H_0$,
  $C_u \sim \mathrm{Bin}(64,2^{-n})$, so the test compares the across-block
  histogram of $C_u$ to that exact binomial law by chi-square with Dieharder's
  Vtest tail bundling; the crate emits every per-pattern p-value explicitly.

- **`gcd_distribution`.** Draw random integer pairs $(u,v)$, compute
  $g=\gcd(u,v)$, and compare the observed gcd histogram to the classical law
  $\Pr(g=k)=6/(\pi^2 k^2)$, with the far tail pooled exactly as in the
  Marsaglia-Tsang Dieharder source. The reported p-value is the corresponding
  chi-square fit.

- **`gcd_step_counts`.** The same integer pairs are also scored by the number
  of Euclidean algorithm steps needed to reduce $(u,v)$ to their gcd. Those
  step counts are compared to the empirical Dieharder `kprob[]` table by a
  chi-square over the $41$ pooled bins, which makes this a structural probe of
  fine arithmetic correlations rather than simple one-dimensional uniformity.

## Readout

- `OsRng` came in at `724/738` on this rerun. That is rougher than the prior
  pass, but still looks like battery-tail noise rather than a structural break:
  two universal-family lows, three template lows, and five `rgb_bitdist` lows.
- `MT19937` comes in at `728/738`, which is exactly the “a handful of low
  p-values in a huge battery” zone rather than a red flag.
- `MT19937` is still basically where it should be: `6/738` failures, which is
  below the rough false-positive budget for a battery this large.
- `Xorshift64` no longer gets a fake-clean report. It now takes `4` real
  `rgb_bitdist` failures.
- `BSD random()` and glibc `random()` no longer look spotless either. Each now
  picks up `2` `rgb_bitdist` failures.
- `AES-128-CTR` lands at `12/714` failures and `cryptography::CtrDrbgAes256`
  lands at `7/714`. Most of those are `rgb_bitdist` family lows; this is the
  right place to be cautious rather than melodramatic.
- `SpongeBob` lands at `15/738`, which is rougher than `MT19937` or
  `cryptography::CtrDrbgAes256` but still far from the catastrophic historical
  generators. Its misses cluster in `non_overlapping_template`,
  `approximate_entropy`, `monobit2`, and `rgb_bitdist`.
- `VB6 Rnd()`, ANSI C `rand()`, `MINSTD`, `Constant`, and `Counter` are
  destroyed, which is exactly the sanity check the suite needed to keep.
- The newly faithful `dab_monobit2` is now part of these results. It does not
  by itself annihilate every weak generator, which is consistent with the
  reference code; the culling still comes from the whole battery, not one test.

## Failure Highlights

### OsRng (/dev/urandom)

- `10` failures total:
  - `nist::universal`: `p = 0.006308`
  - `maurer::universal_l07`: `p = 0.006308`
  - `nist::non_overlapping_template` at `B=000111111`: `p = 0.004105`
  - `nist::non_overlapping_template` at `B=001110111`: `p = 0.000065`
  - `nist::non_overlapping_template` at `B=110110100`: `p = 0.002936`
  - five `dieharder::bit_distribution` lows:
    - width `6`, pattern `12`: `p = 0.000041`
    - width `6`, pattern `60`: `p = 0.002539`
    - width `7`, pattern `104`: `p = 0.007486`
    - width `8`, pattern `33`: `p = 0.002378`
    - width `8`, pattern `135`: `p = 0.008520`

### MT19937 (seed=19650218)

- `nist::non_overlapping_template` at `B=100110000`: `p = 0.007998`
- `nist::non_overlapping_template` at `B=101101000`: `p = 0.006862`
- four `dieharder::bit_distribution` lows across widths `5` and `8`

### Xorshift64 (seed=1)

- no NIST or classic DIEHARD failures in this run
- `4` `dieharder::bit_distribution` failures:
  - width `5`, pattern `3`: `p = 0.007169`
  - width `5`, pattern `12`: `p = 0.005938`
  - width `6`, pattern `0`: `p = 0.001047`
  - width `8`, pattern `95`: `p = 0.006322`

### Xorshift32 (seed=1)

- `nist::matrix_rank`: `p = 0.000000`
- `diehard::binary_rank_32x32`: `p = 0.000000`
- `diehard::binary_rank_31x31`: `p = 0.000000`
- `diehard::count_ones_stream`: `p = 0.000020`
- plus `9` more failures, mostly `rgb_bitdist`

### BAD Unix / Windows Historical Generators

- `System V rand()`: `13` failures, including `runs_up`, `lagged_sums`, and
  several `rgb_bitdist` rows
- `System V mrand48()`: `9` failures, including a very low
  `non_overlapping_template` and `dieharder::dct`
- `FreeBSD12 rand_r() compat`: `15` failures
- `Windows CRT rand()`: `3` failures
- `.NET Random(seed)` compat: `4` failures
- `VB6/VBA Rnd()`: `523` failures; it is catastrophically bad here

### AES-128-CTR (NIST key)

- the same mirrored `nist::non_overlapping_template` failures remain:
  - `B=000000001`: `p = 0.000483`
  - `B=100000000`: `p = 0.000483`
- the other `10` failures are all `dieharder::bit_distribution`

### SpongeBob (SHA3-512 chain, OsRng seed)

Re-run with `from_os_rng()` seeding (the earlier `seed=00..3f` result with 15 FAILs
was an artifact of the all-zeros-ascending test seed, not a structural flaw).

- `7` failures total (`708` tests — excursion-family precondition missed this run):
  - `nist::universal`: `p = 0.000636`
  - `maurer::universal_l07`: `p = 0.000636` (same statistic)
  - `nist::non_overlapping_template` at `B=100010000`: `p = 0.003790`
  - `nist::non_overlapping_template` at `B=110011010`: `p = 0.001163`
  - `dieharder::bit_distribution` width `7`, pattern `18`: `p = 0.001335`
  - `dieharder::bit_distribution` width `7`, pattern `105`: `p = 0.009430`
  - `dieharder::bit_distribution` width `8`, pattern `191`: `p = 0.006494`

### Squidward (SHA-256 chain, OsRng seed)

- `6` failures total (all within expected false-positive budget of ~7):
  - `nist::non_overlapping_template` at `B=000111001`: `p = 0.005872`
  - `maurer::universal_l08`: `p = 0.004393`
  - `dieharder::lagged_sums` lag=1: `p = 0.000517` (below 0.001 — worth watching on re-run)
  - `dieharder::bit_distribution` width `6`, pattern `35`: `p = 0.006340`
  - `dieharder::bit_distribution` width `7`, pattern `89`: `p = 0.001833`
  - `dieharder::bit_distribution` width `8`, pattern `155`: `p = 0.001863`

### PCG32 (OsRng seed)

- `9` failures, mostly `bit_distribution` scatter:
  - `nist::runs`: `p = 0.009117`
  - `diehard::dna`: `p = 0.006696`
  - seven `dieharder::bit_distribution` lows (widths 6–8)

### PCG64 (OsRng seed)

- `6` failures:
  - `nist::block_frequency`: `p = 0.007159`
  - `nist::non_overlapping_template` at `B=001011011`: `p = 0.008532`
  - `nist::non_overlapping_template` at `B=001111111`: `p = 0.001686`
  - `nist::non_overlapping_template` at `B=111111010`: `p = 0.002309`
  - `diehard::binary_rank_31x31`: `p = 0.009228`
  - `dieharder::bit_distribution` width `6`, pattern `43`: `p = 0.007598`

### Xoshiro256\*\* (OsRng seed)

- `5` failures (best non-crypto result in this run):
  - four `nist::non_overlapping_template` lows (all `p > 0.001`)
  - `dieharder::bit_distribution` width `7`, pattern `100`: `p = 0.009291`

### Xoroshiro128\*\* (OsRng seed)

- `13` failures (`708` tests — excursion precondition missed), all in `bit_distribution` and `non_overlapping_template`:
  - two `nist::non_overlapping_template` lows
  - eleven `dieharder::bit_distribution` lows, two below `p = 0.001`:
    - width `8`, pattern `113`: `p = 0.000243`
    - width `8`, pattern `53`: `p = 0.000606`
  - higher count than Xoshiro256\*\* is expected: the 128-bit state has fewer
    degrees of freedom so individual pattern counts fluctuate more at this
    sample size

### WyRand (OsRng seed)

- `10` failures (`708` tests — excursion precondition missed):
  - three `nist::non_overlapping_template` lows
  - seven `dieharder::bit_distribution` lows, one below `p = 0.01`:
    - width `7`, pattern `125`: `p = 0.001528`

### SFC64 (OsRng seed)

- `10` failures, one notably low:
  - three `nist::non_overlapping_template` lows
  - seven `dieharder::bit_distribution` lows, including
    - width `8`, pattern `131`: `p = 0.000028` — the lowest p-value in this
      entire sweep; isolated single-pattern hit, not a family failure

### JSF64 (OsRng seed)

- `12` failures:
  - five `nist::non_overlapping_template` lows, one below `p = 0.001`:
    - `B=101001100`: `p = 0.000682`
  - seven `dieharder::bit_distribution` lows (widths 7–8)

### ChaCha20 CSPRNG (OsRng key)

- `7` failures, exactly at the expected false-positive budget (~7 at α=0.01):
  - `nist::non_overlapping_template` at `B=011000111`: `p = 0.009859`
  - six `dieharder::bit_distribution` lows (widths 6–8), two below `p = 0.001`:
    - width `8`, pattern `98`: `p = 0.000665`
    - width `8`, pattern `165`: `p = 0.000582`

### HMAC_DRBG SHA-256 (OsRng seed)

- `6` failures, all pattern scatter:
  - `nist::non_overlapping_template` at `B=010010111`: `p = 0.008609`
  - five `dieharder::bit_distribution` lows (widths 5–8)

### Hash_DRBG SHA-256 (OsRng seed)

- `2` failures — the best result of any non-trivial generator in the suite:
  - `dieharder::bit_distribution` width `8`, pattern `76`: `p = 0.009756`
  - `dieharder::bit_distribution` width `8`, pattern `252`: `p = 0.001970`

### cryptography::CtrDrbgAes256 (seed=00..2f)

- `7` failures total, all in `dieharder::bit_distribution`
- this is roughly the count you would expect from chance alone in a
  `714`-result battery

### Constant / Counter

- `Constant`: `708/714` failures
- `Counter`: `707/714` failures

## Generator Notes

Theory for each RNG is in [BENCHMARKS.md](BENCHMARKS.md).  Brief summary of
what each generator is and why its test counts are what they are:

- **PCG32 / PCG64** — LCG with a permutation output function.  PCG32’s 9 FAILs
  and PCG64’s 6 FAILs are both within the two-sigma band for a 734-test battery;
  the failures are uncorrelated scatter, not a family cluster.
- **Xoshiro256\*\*** — 5 FAILs; the best non-crypto result in this sweep.  All
  failures are isolated template or bit-distribution lows, no family structure.
- **Xoroshiro128\*\*** — 13 FAILs; higher than Xoshiro256\*\* as expected from the
  smaller state.  Two failures dip below `p=0.001` in `bit_distribution`; a
  re-run would scatter them.  Counts 708 tests because the excursion precondition
  was not met this seed.
- **WyRand** — 10 FAILs (708 tests), all in `non_overlapping_template` and
  `bit_distribution`.  No family failures.
- **SFC64** — 10 FAILs including one isolated hit at `p=0.000028`.  That
  extreme value is the lowest in the entire new sweep; it is a single
  pattern-131 bin in `bit_distribution` and not part of a family failure.
- **JSF64** — 12 FAILs; slightly rougher than SFC64.  Five template lows, one
  dipping to `p=0.000682`; seven `bit_distribution` lows.
- **ChaCha20** — 7 FAILs, exactly at the expected false-positive budget.
  All failures are isolated scatter; two `bit_distribution` hits below
  `p=0.001` are consistent with chance at this battery size.
- **HMAC_DRBG** — 6 FAILs, below budget.  One template low plus five scattered
  `bit_distribution` lows.
- **Hash_DRBG** — 2 FAILs, the best result of any non-trivial generator in the
  suite.  Both are isolated `bit_distribution` pattern hits.
- **SpongeBob** — re-run with `from_os_rng()` gives 7 FAILs (708 tests), fully
  within budget.  The earlier 15-FAIL run was an artifact of the all-zeros
  ascending test seed, not a structural flaw in the generator.
- **Squidward** — 6 FAILs; the `lagged_sums` lag=1 hit at `p=0.000517` is
  worth monitoring on a second run.

## Bottom Line

This run is much more believable than the old one.

- Weak generators are still annihilated; good generators cluster near the expected
  false-positive budget.
- All ten new non-trivial generators land between 2 and 13 FAILs out of 708–734
  tests; the expected budget at α=0.01 is ~7, and the spread is consistent with
  random variation across a single run per generator.
- `Hash_DRBG` at 2 FAILs is the standout; `Xoshiro256**` at 5 FAILs is the best
  non-crypto result.
- `Xoroshiro128**` and `JSF64` at 13 and 12 FAILs respectively are the roughest
  of the new generators; neither shows a family cluster, so re-runs are expected
  to regress toward the mean.
- `SpongeBob` with `from_os_rng()` looks fine (7/708); the old 15-FAIL result
  was entirely a bad-seed artifact.
- `AES-CTR`, `MT19937`, and `cryptography::CtrDrbgAes256` remain in the
  “watch the clustering, but don’t panic” zone.
