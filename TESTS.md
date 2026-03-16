# Full Battery Results

Full `run_tests` battery harvested from `darby.local` on 2026-03-15.

Sample size: **16 Mbit** per generator.

Command:

```sh
./target/release/run_tests
```

Notes:

**Why result counts vary across generators.**

The battery total differs from run to run because several test families are
conditionally skipped based on properties of the sample, not the generator.

The battery has **738 test slots** at this sample size:

- **738 results** — the "full active battery" outcome: the signed-random-walk
  tests (`random_excursions` and `random_excursions_variant`) completed
  successfully (J ≥ 500 zero-crossing cycles).  At 16 Mbit the expected
  cycle count is J ≈ 3191 (= √(2n/π)),
  comfortably above the threshold for well-behaved generators.

- **714 results** — 24 fewer slots than the full battery.  The excursion
  families normally emit 8 + 18 = 26 individual per-state results; when the
  signed random walk produces fewer than J = 500 complete zero-crossing cycles
  both families are each collapsed to a single family-level SKIP entry,
  yielding 26 − 2 = 24 fewer slots.  Degenerate generators (Constant,
  Counter, ANSI C LCG, MINSTD) always land here; a handful of non-degenerate
  generators can too, depending on their random seed.

- **199 results** — `Dual_EC_DRBG` only: two P-256 scalar multiplications per
  30-byte output block makes DIEHARD and DIEHARDER prohibitively slow, so only
  the NIST SP 800-22 suite is run.

**Expected false positives.**  At α = 0.01, a perfect generator should fail
roughly 1% of tests by chance.  With 714–738 active tests, the expected
false-fail count is approximately 7.  Isolated failures below that threshold
are noise, not structure.

## Summary Table

| RNG | Total | PASS | FAIL | SKIP |
|---|---:|---:|---:|---:|
| OsRng (/dev/urandom) | 714 | 707 | 5 | 2 |
| MT19937 (seed=19650218) | 738 | 728 | 10 | 0 |
| Xorshift64 (seed=1) | 738 | 733 | 5 | 0 |
| Xorshift32 (seed=1) | 738 | 726 | 12 | 0 |
| BAD Unix System V rand() (15-bit LCG, seed=1) | 738 | 726 | 12 | 0 |
| BAD Unix System V mrand48() (seed=1) | 738 | 731 | 7 | 0 |
| BAD Unix BSD random() TYPE_3 (seed=1) | 738 | 728 | 10 | 0 |
| BAD Unix Linux glibc rand()/random() (seed=1) | 738 | 728 | 10 | 0 |
| BAD Unix FreeBSD12 rand_r() compat (seed=1) | 738 | 728 | 10 | 0 |
| BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1) | 738 | 728 | 10 | 0 |
| BAD Windows VB6/VBA Rnd() (project seed=1) | 738 | 209 | 529 | 0 |
| BAD Windows .NET Random(seed=1) compat | 738 | 731 | 7 | 0 |
| ANSI C sample LCG (1103515245,12345; seed=1) | 714 | 15 | 697 | 2 |
| LCG MINSTD (seed=1) | 714 | 20 | 692 | 2 |
| BBS (p=2³¹−1, q=4294967291) | 738 | 731 | 7 | 0 |
| Blum-Micali (p=2³¹−1, g=7) | 738 | 734 | 4 | 0 |
| AES-128-CTR (NIST key) | 738 | 733 | 5 | 0 |
| SpongeBob (SHA3-512 chain, OsRng seed) | 738 | 727 | 11 | 0 |
| Squidward (SHA-256 chain, OsRng seed) | 738 | 731 | 7 | 0 |
| PCG32 (OsRng seed) | 738 | 728 | 10 | 0 |
| PCG64 (OsRng seed) | 738 | 729 | 9 | 0 |
| Xoshiro256** (OsRng seed) | 738 | 726 | 12 | 0 |
| Xoroshiro128** (OsRng seed) | 738 | 731 | 7 | 0 |
| WyRand (OsRng seed) | 738 | 737 | 1 | 0 |
| SFC64 (OsRng seed) | 714 | 708 | 4 | 2 |
| JSF64 (OsRng seed) | 738 | 727 | 11 | 0 |
| ChaCha20 CSPRNG (OsRng key) | 738 | 730 | 8 | 0 |
| HMAC_DRBG SHA-256 (OsRng seed) | 714 | 702 | 10 | 2 |
| Hash_DRBG SHA-256 (OsRng seed) | 714 | 706 | 6 | 2 |
| cryptography::CtrDrbgAes256 (seed=00..2f) | 714 | 704 | 8 | 2 |
| Constant (0xDEAD_DEAD) | 714 | 1 | 711 | 2 |
| Counter (0,1,2,…) | 714 | 2 | 710 | 2 |
| Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01) | 199 | 196 | 3 | 0 |

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

$$P_{m,n}(r)=2^{-mn}\prod_{i=0}^{r-1}\frac{(2^m-2^i)(2^n-2^i)}{(2^r-2^i)}$$

  and NIST uses a chi-square over the pooled bins.

- **`spectral`.** Map bits to $\pm1$, take the DFT, and count how many Fourier
  magnitudes fall below the threshold $T=\sqrt{n\ln 20}$. With
  $N_1=\lvert\{k:\lvert F_k\rvert<T\}\rvert$ and null expectation $N_0=0.95\,n/2$, the test forms a
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

$$P_{m,n}(r)=2^{-mn}\prod_{i=0}^{r-1}\frac{(2^m-2^i)(2^n-2^i)}{(2^r-2^i)}$$

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

## Failure Highlights

### OsRng (/dev/urandom)

- `5` failures out of `714` tests:
  - `diehard::binary_rank_32x32`: p = 0.004461  (32×32, N=40000, χ²=13.0826)
  - `dieharder::fill_tree_count`: p = 0.003674  (trials=100000, χ²=24.4273, start=4, end=14)
  - `dieharder::bit_distribution`: p = 0.004686  (width=5, pattern=1, tsamples=1600000, bsamples=64, df=10, χ²=25.3703)
  - `dieharder::bit_distribution`: p = 0.009778  (width=7, pattern=126, tsamples=1142857, bsamples=64, df=5, χ²=15.1406)
  - `dieharder::bit_distribution`: p = 0.008999  (width=8, pattern=165, tsamples=1000000, bsamples=64, df=4, χ²=13.5191)

### MT19937 (seed=19650218)

- `10` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.001183  (B=101000000, N=8, M=2000000, χ²=25.6980)
  - `nist::non_overlapping_template`: p = 0.001895  (B=111010110, N=8, M=2000000, χ²=24.4909)
  - `dieharder::bit_distribution`: p = 0.002180  (width=6, pattern=15, tsamples=1333333, bsamples=64, df=7, χ²=22.3846)
  - `dieharder::bit_distribution`: p = 0.003466  (width=6, pattern=62, tsamples=1333333, bsamples=64, df=7, χ²=21.2137)
  - `dieharder::bit_distribution`: p = 0.009476  (width=7, pattern=90, tsamples=1142857, bsamples=64, df=5, χ²=15.2166)
  - `dieharder::bit_distribution`: p = 0.007480  (width=7, pattern=100, tsamples=1142857, bsamples=64, df=5, χ²=15.7866)
  - `dieharder::bit_distribution`: p = 0.009370  (width=8, pattern=41, tsamples=1000000, bsamples=64, df=4, χ²=13.4264)
  - `dieharder::bit_distribution`: p = 0.003424  (width=8, pattern=96, tsamples=1000000, bsamples=64, df=4, χ²=15.7167)
  - `dieharder::bit_distribution`: p = 0.003962  (width=8, pattern=122, tsamples=1000000, bsamples=64, df=4, χ²=15.3870)
  - `dieharder::bit_distribution`: p = 0.007388  (width=8, pattern=250, tsamples=1000000, bsamples=64, df=4, χ²=13.9711)

### Xorshift64 (seed=1)

- `5` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.005897  (B=001000011, N=8, M=2000000, χ²=21.5154)
  - `dieharder::bit_distribution`: p = 0.007156  (width=6, pattern=34, tsamples=1333333, bsamples=64, df=7, χ²=19.3510)
  - `dieharder::bit_distribution`: p = 0.009979  (width=7, pattern=112, tsamples=1142857, bsamples=64, df=5, χ²=15.0912)
  - `dieharder::bit_distribution`: p = 0.004961  (width=8, pattern=88, tsamples=1000000, bsamples=64, df=4, χ²=14.8781)
  - `dieharder::bit_distribution`: p = 0.007538  (width=8, pattern=168, tsamples=1000000, bsamples=64, df=4, χ²=13.9252)

### Xorshift32 (seed=1)

- `12` failures out of `738` tests:
  - `nist::matrix_rank`: p = 0.000000  (N=15625, F32=15625, F31=0, F≤30=0, χ²=38478.1856)
  - `diehard::binary_rank_32x32`: p = 0.000000  (32×32, N=40000, χ²=98509.8647)
  - `diehard::binary_rank_31x31`: p = 0.000000  (31×31, N=40000, χ²=12021.1172)
  - `dieharder::bit_distribution`: p = 0.000000  (width=1, pattern=0, tsamples=8000000, bsamples=64, df=36, χ²=3065.7343)
  - `dieharder::bit_distribution`: p = 0.000000  (width=1, pattern=1, tsamples=8000000, bsamples=64, df=36, χ²=3065.7343)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=0, tsamples=4000000, bsamples=64, df=30, χ²=259.1680)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=3, tsamples=4000000, bsamples=64, df=30, χ²=105.6466)
  - `dieharder::bit_distribution`: p = 0.000358  (width=3, pattern=0, tsamples=2666666, bsamples=64, df=21, χ²=50.0609)
  - `dieharder::bit_distribution`: p = 0.000161  (width=6, pattern=23, tsamples=1333333, bsamples=64, df=7, χ²=28.7516)
  - `dieharder::bit_distribution`: p = 0.000842  (width=8, pattern=10, tsamples=1000000, bsamples=64, df=4, χ²=18.8474)
  - `dieharder::bit_distribution`: p = 0.005681  (width=8, pattern=174, tsamples=1000000, bsamples=64, df=4, χ²=14.5701)
  - `dieharder::bit_distribution`: p = 0.003937  (width=8, pattern=238, tsamples=1000000, bsamples=64, df=4, χ²=15.4015)

### BAD Unix System V rand() (15-bit LCG, seed=1)

- `12` failures out of `738` tests:
  - `nist::spectral`: p = 0.000000  (n=16000000, N₀=7600000.0, N₁=7596196, T=6923.2735, d=-8.7270)
  - `nist::non_overlapping_template`: p = 0.000973  (B=110010100, N=8, M=2000000, χ²=26.1948)
  - `nist::non_overlapping_template`: p = 0.004389  (B=111001010, N=8, M=2000000, χ²=22.3000)
  - `diehard::opso`: p = 0.000000  (missing=146136, z=14.5515)
  - `dieharder::bit_distribution`: p = 0.003642  (width=6, pattern=25, tsamples=1333333, bsamples=64, df=7, χ²=21.0881)
  - `dieharder::bit_distribution`: p = 0.003517  (width=7, pattern=26, tsamples=1142857, bsamples=64, df=5, χ²=17.5834)
  - `dieharder::bit_distribution`: p = 0.003087  (width=7, pattern=44, tsamples=1142857, bsamples=64, df=5, χ²=17.8901)
  - `dieharder::bit_distribution`: p = 0.007548  (width=8, pattern=10, tsamples=1000000, bsamples=64, df=4, χ²=13.9220)
  - `dieharder::bit_distribution`: p = 0.008482  (width=8, pattern=79, tsamples=1000000, bsamples=64, df=4, χ²=13.6550)
  - `dieharder::bit_distribution`: p = 0.006008  (width=8, pattern=82, tsamples=1000000, bsamples=64, df=4, χ²=14.4429)
  - `dieharder::bit_distribution`: p = 0.001352  (width=8, pattern=112, tsamples=1000000, bsamples=64, df=4, χ²=17.7972)
  - `dieharder::bit_distribution`: p = 0.000197  (width=8, pattern=166, tsamples=1000000, bsamples=64, df=4, χ²=22.0377)

### BAD Unix System V mrand48() (seed=1)

- `7` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.006449  (B=000011001, N=8, M=2000000, χ²=21.2763)
  - `nist::non_overlapping_template`: p = 0.004371  (B=111010000, N=8, M=2000000, χ²=22.3112)
  - `diehard::opso`: p = 0.000000  (missing=139772, z=-7.3584)
  - `diehard::oqso`: p = 0.000000  (missing=146497, z=15.5687)
  - `diehard::dna`: p = 0.000082  (missing=140582, z=-3.9385)
  - `dieharder::bit_distribution`: p = 0.007808  (width=8, pattern=164, tsamples=1000000, bsamples=64, df=4, χ²=13.8445)
  - `dieharder::bit_distribution`: p = 0.003404  (width=8, pattern=212, tsamples=1000000, bsamples=64, df=4, χ²=15.7297)

### BAD Unix BSD random() TYPE_3 (seed=1)

- `10` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.008664  (B=001010011, N=8, M=2000000, χ²=20.4804)
  - `nist::non_overlapping_template`: p = 0.008826  (B=001101111, N=8, M=2000000, χ²=20.4301)
  - `nist::non_overlapping_template`: p = 0.001400  (B=101001100, N=8, M=2000000, χ²=25.2676)
  - `nist::non_overlapping_template`: p = 0.002688  (B=110011010, N=8, M=2000000, χ²=23.5863)
  - `nist::non_overlapping_template`: p = 0.000900  (B=110111000, N=8, M=2000000, χ²=26.3903)
  - `dieharder::bit_distribution`: p = 0.008114  (width=5, pattern=6, tsamples=1600000, bsamples=64, df=10, χ²=23.8126)
  - `dieharder::bit_distribution`: p = 0.004759  (width=7, pattern=51, tsamples=1142857, bsamples=64, df=5, χ²=16.8669)
  - `dieharder::bit_distribution`: p = 0.006149  (width=7, pattern=91, tsamples=1142857, bsamples=64, df=5, χ²=16.2563)
  - `dieharder::bit_distribution`: p = 0.000887  (width=8, pattern=7, tsamples=1000000, bsamples=64, df=4, χ²=18.7323)
  - `dieharder::bit_distribution`: p = 0.001708  (width=8, pattern=114, tsamples=1000000, bsamples=64, df=4, χ²=17.2766)

### BAD Unix Linux glibc rand()/random() (seed=1)

- `10` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.008664  (B=001010011, N=8, M=2000000, χ²=20.4804)
  - `nist::non_overlapping_template`: p = 0.008826  (B=001101111, N=8, M=2000000, χ²=20.4301)
  - `nist::non_overlapping_template`: p = 0.001400  (B=101001100, N=8, M=2000000, χ²=25.2676)
  - `nist::non_overlapping_template`: p = 0.002688  (B=110011010, N=8, M=2000000, χ²=23.5863)
  - `nist::non_overlapping_template`: p = 0.000900  (B=110111000, N=8, M=2000000, χ²=26.3903)
  - `dieharder::bit_distribution`: p = 0.008114  (width=5, pattern=6, tsamples=1600000, bsamples=64, df=10, χ²=23.8126)
  - `dieharder::bit_distribution`: p = 0.004759  (width=7, pattern=51, tsamples=1142857, bsamples=64, df=5, χ²=16.8669)
  - `dieharder::bit_distribution`: p = 0.006149  (width=7, pattern=91, tsamples=1142857, bsamples=64, df=5, χ²=16.2563)
  - `dieharder::bit_distribution`: p = 0.000887  (width=8, pattern=7, tsamples=1000000, bsamples=64, df=4, χ²=18.7323)
  - `dieharder::bit_distribution`: p = 0.001708  (width=8, pattern=114, tsamples=1000000, bsamples=64, df=4, χ²=17.2766)

### BAD Unix FreeBSD12 rand_r() compat (seed=1)

- `10` failures out of `738` tests:
  - `nist::spectral`: p = 0.000916  (n=16000000, N₀=7600000.0, N₁=7598555, T=6923.2735, d=-3.3151)
  - `nist::non_overlapping_template`: p = 0.001509  (B=000001101, N=8, M=2000000, χ²=25.0756)
  - `nist::non_overlapping_template`: p = 0.001397  (B=000011011, N=8, M=2000000, χ²=25.2731)
  - `dieharder::bit_distribution`: p = 0.008891  (width=5, pattern=4, tsamples=1600000, bsamples=64, df=10, χ²=23.5492)
  - `dieharder::bit_distribution`: p = 0.008123  (width=7, pattern=0, tsamples=1142857, bsamples=64, df=5, χ²=15.5884)
  - `dieharder::bit_distribution`: p = 0.009874  (width=8, pattern=78, tsamples=1000000, bsamples=64, df=4, χ²=13.3060)
  - `dieharder::bit_distribution`: p = 0.000412  (width=8, pattern=88, tsamples=1000000, bsamples=64, df=4, χ²=20.4234)
  - `dieharder::bit_distribution`: p = 0.003072  (width=8, pattern=95, tsamples=1000000, bsamples=64, df=4, χ²=15.9607)
  - `dieharder::bit_distribution`: p = 0.005988  (width=8, pattern=114, tsamples=1000000, bsamples=64, df=4, χ²=14.4503)
  - `dieharder::bit_distribution`: p = 0.007135  (width=8, pattern=179, tsamples=1000000, bsamples=64, df=4, χ²=14.0507)

### BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1)

- `10` failures out of `738` tests:
  - `nist::spectral`: p = 0.000001  (n=16000000, N₀=7600000.0, N₁=7597822, T=6923.2735, d=-4.9967)
  - `nist::random_excursions`: p = 0.008107  (x=-1, J=3112, χ²=15.5932)
  - `dieharder::dct`: p = 0.005792  (ntuple=256, tsamples=5000, χ²=315.5840)
  - `dieharder::bit_distribution`: p = 0.008848  (width=5, pattern=24, tsamples=1600000, bsamples=64, df=10, χ²=23.5633)
  - `dieharder::bit_distribution`: p = 0.000594  (width=6, pattern=36, tsamples=1333333, bsamples=64, df=7, χ²=25.5989)
  - `dieharder::bit_distribution`: p = 0.007644  (width=7, pattern=54, tsamples=1142857, bsamples=64, df=5, χ²=15.7347)
  - `dieharder::bit_distribution`: p = 0.001798  (width=8, pattern=12, tsamples=1000000, bsamples=64, df=4, χ²=17.1620)
  - `dieharder::bit_distribution`: p = 0.009471  (width=8, pattern=58, tsamples=1000000, bsamples=64, df=4, χ²=13.4018)
  - `dieharder::bit_distribution`: p = 0.001004  (width=8, pattern=159, tsamples=1000000, bsamples=64, df=4, χ²=18.4574)
  - `dieharder::bit_distribution`: p = 0.006187  (width=8, pattern=241, tsamples=1000000, bsamples=64, df=4, χ²=14.3758)

### BAD Windows VB6/VBA Rnd() (project seed=1)

- `529` failures out of `738` tests:
  - `nist::spectral`: p = 0.000000  (n=16000000, N₀=7600000.0, N₁=7807605, T=6923.2735, d=476.2785)
  - `nist::overlapping_template`: p = 0.000000  (n=16000000, m=9, N=15503, ν=[5353, 2966, 2254, 1727, 1127, 2076], χ²=44.8745)
  - `nist::universal`: p = 0.000000  (n=16000000, L=10, Q=10240, K=1589760, f_n=9.1887, μ=9.1723, σ=0.000915)
  - `maurer::universal_l06`: p = 0.000000  (n=16000000, L=6, Q=640, K=2666026, f_n=5.2599, μ=5.2177, σ=0.000597)
  - `maurer::universal_l07`: p = 0.000000  (n=16000000, L=7, Q=1280, K=2284434, f_n=6.2010, μ=6.1963, σ=0.000686)
  - `maurer::universal_l08`: p = 0.000000  (n=16000000, L=8, Q=2560, K=1997440, f_n=7.2752, μ=7.1837, σ=0.000767)
  - `maurer::universal_l09`: p = 0.000000  (n=16000000, L=9, Q=5120, K=1772657, f_n=8.2016, μ=8.1764, σ=0.000842)
  - `maurer::universal_l10`: p = 0.000000  (n=16000000, L=10, Q=10240, K=1589760, f_n=9.1887, μ=9.1723, σ=0.000915)
  - `maurer::universal_l11`: p = 0.000000  (n=16000000, L=11, Q=20480, K=1434065, f_n=10.1794, μ=10.1700, σ=0.000988)
  - `maurer::universal_l12`: p = 0.000000  (n=16000000, L=12, Q=40960, K=1292373, f_n=11.3964, μ=11.1688, σ=0.001067)
  - `maurer::universal_l13`: p = 0.000000  (n=16000000, L=13, Q=81920, K=1148849, f_n=12.1787, μ=12.1681, σ=0.001161)
  - `maurer::universal_l14`: p = 0.000000  (n=16000000, L=14, Q=163840, K=979017, f_n=13.1871, μ=13.1677, σ=0.001292)
  - `maurer::universal_l15`: p = 0.000000  (n=16000000, L=15, Q=327680, K=738986, f_n=14.1973, μ=14.1675, σ=0.001535)
  - `maurer::universal_l16`: p = 0.001982  (n=16000000, L=16, Q=655360, K=344640, f_n=15.1601, μ=15.1674, σ=0.002360)
  - `diehard::binary_rank_6x8`: p = 0.000000  (N=100000, χ²=128.6485)
  - `diehard::bitstream`: p = 0.000000  (window=20-bit, stream=2^21, repeats=20)
  - `diehard::opso`: p = 0.000000  (missing=577156, z=1498.4620)
  - `diehard::oqso`: p = 0.000000  (missing=462472, z=1087.9213)
  - `diehard::dna`: p = 0.000000  (missing=434982, z=868.9005)
  - `diehard::count_ones_stream`: p = 0.000000  (n=256000, Q5=5470.51, Q4=1693.14, Q5-Q4=3777.37, Z=18.0647)
  - `diehard::minimum_distance_2d`: p = 0.000000  (n=8000, side=10000, repeats=100 [BUGGY FORMULA — see diehard_2dsphere.c; use minimum_distance_nd(d=2) instead])
  - `diehard::squeeze`: p = 0.000000  (trials=100000, cells=43, df=37, χ²=1755.6843)
  - `diehard::craps_throws`: p = 0.000001  (games=200000, df=21, χ²=68.4389)
  - `dieharder::minimum_distance_nd`: p = 0.000000  (d=5, n=8000, repeats=100)
  - `dieharder::lagged_sums`: p = 0.000000  (lag=1, tsamples=8000000, sum=4036352.7106, z=44.5228)
  - `dieharder::ks_uniform`: p = 0.000000  (tsamples=16000000)
  - `dieharder::byte_distribution`: p = 0.000000  (tsamples=5333333, streams=9, expected/cell=20833.3, χ²=48000131.1456)
  - `dieharder::dct`: p = 0.000000  (ntuple=256, tsamples=5000, χ²=8750.9888)
  - `dieharder::fill_tree_count`: p = 0.000000  (trials=100000, χ²=75.5551, start=4, end=14)
  - `dieharder::fill_tree_position`: p = 0.000000  (trials=100000, χ²=238.7482)
  - `dieharder::bit_distribution`: p = 0.000000  (width=1, pattern=0, tsamples=8000000, bsamples=64, df=36, χ²=18904.4896)
  - `dieharder::bit_distribution`: p = 0.000000  (width=1, pattern=1, tsamples=8000000, bsamples=64, df=36, χ²=18904.4896)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=0, tsamples=4000000, bsamples=64, df=30, χ²=6748.3328)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=1, tsamples=4000000, bsamples=64, df=30, χ²=4745.2277)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=2, tsamples=4000000, bsamples=64, df=30, χ²=28331.9154)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=3, tsamples=4000000, bsamples=64, df=30, χ²=36254.4862)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=0, tsamples=2666666, bsamples=64, df=21, χ²=25165.2535)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=1, tsamples=2666666, bsamples=64, df=21, χ²=30755.3092)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=2, tsamples=2666666, bsamples=64, df=21, χ²=67485.7314)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=3, tsamples=2666666, bsamples=64, df=21, χ²=661336.5249)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=4, tsamples=2666666, bsamples=64, df=21, χ²=25599.9108)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=5, tsamples=2666666, bsamples=64, df=21, χ²=219126.3722)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=6, tsamples=2666666, bsamples=64, df=21, χ²=1369696.6043)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=7, tsamples=2666666, bsamples=64, df=21, χ²=25086.7519)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=0, tsamples=2000000, bsamples=64, df=14, χ²=29595.2980)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=1, tsamples=2000000, bsamples=64, df=14, χ²=12849.8407)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=2, tsamples=2000000, bsamples=64, df=14, χ²=16370.9640)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=3, tsamples=2000000, bsamples=64, df=14, χ²=7954.9889)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=4, tsamples=2000000, bsamples=64, df=14, χ²=27127.3948)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=5, tsamples=2000000, bsamples=64, df=14, χ²=7408.7538)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=6, tsamples=2000000, bsamples=64, df=14, χ²=22835.5936)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=7, tsamples=2000000, bsamples=64, df=14, χ²=1152.7523)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=8, tsamples=2000000, bsamples=64, df=14, χ²=20289.8328)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=9, tsamples=2000000, bsamples=64, df=14, χ²=11702.8257)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=10, tsamples=2000000, bsamples=64, df=14, χ²=28349.8177)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=11, tsamples=2000000, bsamples=64, df=14, χ²=2305.0413)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=12, tsamples=2000000, bsamples=64, df=14, χ²=30023.8407)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=13, tsamples=2000000, bsamples=64, df=14, χ²=12603.5508)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=14, tsamples=2000000, bsamples=64, df=14, χ²=28142.0452)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=15, tsamples=2000000, bsamples=64, df=14, χ²=8049.1469)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=0, tsamples=1600000, bsamples=64, df=10, χ²=322.5272)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=1, tsamples=1600000, bsamples=64, df=10, χ²=308.8343)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=2, tsamples=1600000, bsamples=64, df=10, χ²=2016.3656)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=3, tsamples=1600000, bsamples=64, df=10, χ²=420.6417)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=4, tsamples=1600000, bsamples=64, df=10, χ²=4391.6174)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=5, tsamples=1600000, bsamples=64, df=10, χ²=4228.1680)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=6, tsamples=1600000, bsamples=64, df=10, χ²=1255.4749)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=7, tsamples=1600000, bsamples=64, df=10, χ²=602.5907)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=8, tsamples=1600000, bsamples=64, df=10, χ²=1178.2731)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=9, tsamples=1600000, bsamples=64, df=10, χ²=2908.8657)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=10, tsamples=1600000, bsamples=64, df=10, χ²=1187.3198)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=11, tsamples=1600000, bsamples=64, df=10, χ²=627.0363)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=12, tsamples=1600000, bsamples=64, df=10, χ²=554.1912)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=13, tsamples=1600000, bsamples=64, df=10, χ²=312.6549)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=14, tsamples=1600000, bsamples=64, df=10, χ²=3516.5476)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=15, tsamples=1600000, bsamples=64, df=10, χ²=532.0191)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=16, tsamples=1600000, bsamples=64, df=10, χ²=4097.4362)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=17, tsamples=1600000, bsamples=64, df=10, χ²=1321.9681)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=18, tsamples=1600000, bsamples=64, df=10, χ²=927.7272)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=19, tsamples=1600000, bsamples=64, df=10, χ²=914.6305)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=20, tsamples=1600000, bsamples=64, df=10, χ²=1214.8949)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=21, tsamples=1600000, bsamples=64, df=10, χ²=1763.0648)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=22, tsamples=1600000, bsamples=64, df=10, χ²=877.2030)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=23, tsamples=1600000, bsamples=64, df=10, χ²=3175.5184)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=24, tsamples=1600000, bsamples=64, df=10, χ²=235.7298)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=25, tsamples=1600000, bsamples=64, df=10, χ²=1278.7342)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=26, tsamples=1600000, bsamples=64, df=10, χ²=3372.3180)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=27, tsamples=1600000, bsamples=64, df=10, χ²=189.0051)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=28, tsamples=1600000, bsamples=64, df=10, χ²=1358.4878)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=29, tsamples=1600000, bsamples=64, df=10, χ²=1013.4986)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=30, tsamples=1600000, bsamples=64, df=10, χ²=1999.1940)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=31, tsamples=1600000, bsamples=64, df=10, χ²=3362.4121)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=0, tsamples=1333333, bsamples=64, df=7, χ²=4346.5204)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=1, tsamples=1333333, bsamples=64, df=7, χ²=1989.8515)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=2, tsamples=1333333, bsamples=64, df=7, χ²=87275.0738)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=3, tsamples=1333333, bsamples=64, df=7, χ²=2352.4890)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=4, tsamples=1333333, bsamples=64, df=7, χ²=84947.7311)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=5, tsamples=1333333, bsamples=64, df=7, χ²=85273.6403)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=6, tsamples=1333333, bsamples=64, df=7, χ²=359154.9762)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=7, tsamples=1333333, bsamples=64, df=7, χ²=84902.8102)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=8, tsamples=1333333, bsamples=64, df=7, χ²=910.4177)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=9, tsamples=1333333, bsamples=64, df=7, χ²=633.9356)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=10, tsamples=1333333, bsamples=64, df=7, χ²=93811.5736)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=11, tsamples=1333333, bsamples=64, df=7, χ²=389.0973)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=12, tsamples=1333333, bsamples=64, df=7, χ²=6506.0818)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=13, tsamples=1333333, bsamples=64, df=7, χ²=85.9152)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=14, tsamples=1333333, bsamples=64, df=7, χ²=86782.7416)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=15, tsamples=1333333, bsamples=64, df=7, χ²=772.6800)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=16, tsamples=1333333, bsamples=64, df=7, χ²=85236.4128)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=17, tsamples=1333333, bsamples=64, df=7, χ²=85286.2880)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=18, tsamples=1333333, bsamples=64, df=7, χ²=8160.4468)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=19, tsamples=1333333, bsamples=64, df=7, χ²=87305.5134)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=20, tsamples=1333333, bsamples=64, df=7, χ²=4062.9275)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=21, tsamples=1333333, bsamples=64, df=7, χ²=7695.2553)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=22, tsamples=1333333, bsamples=64, df=7, χ²=99183.0198)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=23, tsamples=1333333, bsamples=64, df=7, χ²=11094.1436)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=24, tsamples=1333333, bsamples=64, df=7, χ²=85967.9566)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=25, tsamples=1333333, bsamples=64, df=7, χ²=85105.9204)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=26, tsamples=1333333, bsamples=64, df=7, χ²=3030.7489)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=27, tsamples=1333333, bsamples=64, df=7, χ²=85765.7167)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=28, tsamples=1333333, bsamples=64, df=7, χ²=102772.9472)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=29, tsamples=1333333, bsamples=64, df=7, χ²=85058.2265)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=30, tsamples=1333333, bsamples=64, df=7, χ²=10116.2083)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=31, tsamples=1333333, bsamples=64, df=7, χ²=87762.5932)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=32, tsamples=1333333, bsamples=64, df=7, χ²=89735.6162)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=33, tsamples=1333333, bsamples=64, df=7, χ²=85492.1718)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=34, tsamples=1333333, bsamples=64, df=7, χ²=1559.5336)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=35, tsamples=1333333, bsamples=64, df=7, χ²=85321.0539)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=36, tsamples=1333333, bsamples=64, df=7, χ²=6636.0589)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=37, tsamples=1333333, bsamples=64, df=7, χ²=1161.0419)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=38, tsamples=1333333, bsamples=64, df=7, χ²=84845.1355)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=39, tsamples=1333333, bsamples=64, df=7, χ²=2335.6586)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=40, tsamples=1333333, bsamples=64, df=7, χ²=88144.9689)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=41, tsamples=1333333, bsamples=64, df=7, χ²=87718.0320)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=42, tsamples=1333333, bsamples=64, df=7, χ²=10515.4469)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=43, tsamples=1333333, bsamples=64, df=7, χ²=86900.0521)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=44, tsamples=1333333, bsamples=64, df=7, χ²=116376.9162)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=45, tsamples=1333333, bsamples=64, df=7, χ²=84958.6499)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=46, tsamples=1333333, bsamples=64, df=7, χ²=1001.4908)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=47, tsamples=1333333, bsamples=64, df=7, χ²=85126.8601)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=48, tsamples=1333333, bsamples=64, df=7, χ²=194.7322)
  - `dieharder::bit_distribution`: p = 0.000100  (width=6, pattern=49, tsamples=1333333, bsamples=64, df=7, χ²=29.8754)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=50, tsamples=1333333, bsamples=64, df=7, χ²=93085.8198)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=51, tsamples=1333333, bsamples=64, df=7, χ²=118.4357)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=52, tsamples=1333333, bsamples=64, df=7, χ²=84622.6430)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=53, tsamples=1333333, bsamples=64, df=7, χ²=85647.7103)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=54, tsamples=1333333, bsamples=64, df=7, χ²=341045.2623)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=55, tsamples=1333333, bsamples=64, df=7, χ²=84770.5556)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=56, tsamples=1333333, bsamples=64, df=7, χ²=605.1355)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=57, tsamples=1333333, bsamples=64, df=7, χ²=355.8878)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=58, tsamples=1333333, bsamples=64, df=7, χ²=88601.5437)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=59, tsamples=1333333, bsamples=64, df=7, χ²=1077.4870)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=60, tsamples=1333333, bsamples=64, df=7, χ²=4139.6831)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=61, tsamples=1333333, bsamples=64, df=7, χ²=327.6819)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=62, tsamples=1333333, bsamples=64, df=7, χ²=92924.9392)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=63, tsamples=1333333, bsamples=64, df=7, χ²=1781.9142)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=0, tsamples=1142857, bsamples=64, df=5, χ²=122.1814)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=1, tsamples=1142857, bsamples=64, df=5, χ²=162.0591)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=2, tsamples=1142857, bsamples=64, df=5, χ²=926.1240)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=3, tsamples=1142857, bsamples=64, df=5, χ²=949.4597)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=4, tsamples=1142857, bsamples=64, df=5, χ²=49.2090)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=5, tsamples=1142857, bsamples=64, df=5, χ²=407.3079)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=6, tsamples=1142857, bsamples=64, df=5, χ²=104.7337)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=7, tsamples=1142857, bsamples=64, df=5, χ²=216.9378)
  - `dieharder::bit_distribution`: p = 0.000009  (width=7, pattern=8, tsamples=1142857, bsamples=64, df=5, χ²=31.0191)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=9, tsamples=1142857, bsamples=64, df=5, χ²=737.5444)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=10, tsamples=1142857, bsamples=64, df=5, χ²=48.8603)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=11, tsamples=1142857, bsamples=64, df=5, χ²=3864.1341)
  - `dieharder::bit_distribution`: p = 0.000003  (width=7, pattern=12, tsamples=1142857, bsamples=64, df=5, χ²=33.2811)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=13, tsamples=1142857, bsamples=64, df=5, χ²=97.5065)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=14, tsamples=1142857, bsamples=64, df=5, χ²=226.9319)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=15, tsamples=1142857, bsamples=64, df=5, χ²=301.5490)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=16, tsamples=1142857, bsamples=64, df=5, χ²=919.5351)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=17, tsamples=1142857, bsamples=64, df=5, χ²=165.5355)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=18, tsamples=1142857, bsamples=64, df=5, χ²=304.2348)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=19, tsamples=1142857, bsamples=64, df=5, χ²=257.8393)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=20, tsamples=1142857, bsamples=64, df=5, χ²=39.7995)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=21, tsamples=1142857, bsamples=64, df=5, χ²=169.4726)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=22, tsamples=1142857, bsamples=64, df=5, χ²=157.6641)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=23, tsamples=1142857, bsamples=64, df=5, χ²=2833.0287)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=24, tsamples=1142857, bsamples=64, df=5, χ²=912.7315)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=25, tsamples=1142857, bsamples=64, df=5, χ²=640.1212)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=26, tsamples=1142857, bsamples=64, df=5, χ²=797.9949)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=27, tsamples=1142857, bsamples=64, df=5, χ²=1252.1898)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=28, tsamples=1142857, bsamples=64, df=5, χ²=68.1247)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=29, tsamples=1142857, bsamples=64, df=5, χ²=111.8020)
  - `dieharder::bit_distribution`: p = 0.000380  (width=7, pattern=30, tsamples=1142857, bsamples=64, df=5, χ²=22.7287)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=31, tsamples=1142857, bsamples=64, df=5, χ²=198.7508)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=32, tsamples=1142857, bsamples=64, df=5, χ²=450.8283)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=33, tsamples=1142857, bsamples=64, df=5, χ²=55.5757)
  - `dieharder::bit_distribution`: p = 0.009305  (width=7, pattern=34, tsamples=1142857, bsamples=64, df=5, χ²=15.2606)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=35, tsamples=1142857, bsamples=64, df=5, χ²=38.1932)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=36, tsamples=1142857, bsamples=64, df=5, χ²=255.7850)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=37, tsamples=1142857, bsamples=64, df=5, χ²=714.3719)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=38, tsamples=1142857, bsamples=64, df=5, χ²=397.1835)
  - `dieharder::bit_distribution`: p = 0.006419  (width=7, pattern=39, tsamples=1142857, bsamples=64, df=5, χ²=16.1536)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=40, tsamples=1142857, bsamples=64, df=5, χ²=711.8183)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=41, tsamples=1142857, bsamples=64, df=5, χ²=59.3054)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=42, tsamples=1142857, bsamples=64, df=5, χ²=163.4221)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=43, tsamples=1142857, bsamples=64, df=5, χ²=239.4812)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=44, tsamples=1142857, bsamples=64, df=5, χ²=109.0013)
  - `dieharder::bit_distribution`: p = 0.002478  (width=7, pattern=45, tsamples=1142857, bsamples=64, df=5, χ²=18.4061)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=46, tsamples=1142857, bsamples=64, df=5, χ²=2700.6041)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=47, tsamples=1142857, bsamples=64, df=5, χ²=2195.5800)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=48, tsamples=1142857, bsamples=64, df=5, χ²=409.2794)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=49, tsamples=1142857, bsamples=64, df=5, χ²=359.7949)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=50, tsamples=1142857, bsamples=64, df=5, χ²=402.9003)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=52, tsamples=1142857, bsamples=64, df=5, χ²=171.3922)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=53, tsamples=1142857, bsamples=64, df=5, χ²=6824.9882)
  - `dieharder::bit_distribution`: p = 0.000102  (width=7, pattern=54, tsamples=1142857, bsamples=64, df=5, χ²=25.7041)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=55, tsamples=1142857, bsamples=64, df=5, χ²=74.8335)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=56, tsamples=1142857, bsamples=64, df=5, χ²=545.5651)
  - `dieharder::bit_distribution`: p = 0.000001  (width=7, pattern=57, tsamples=1142857, bsamples=64, df=5, χ²=35.8528)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=58, tsamples=1142857, bsamples=64, df=5, χ²=979.5210)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=59, tsamples=1142857, bsamples=64, df=5, χ²=132.2139)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=60, tsamples=1142857, bsamples=64, df=5, χ²=1533.6508)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=61, tsamples=1142857, bsamples=64, df=5, χ²=107.2627)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=62, tsamples=1142857, bsamples=64, df=5, χ²=109.1359)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=63, tsamples=1142857, bsamples=64, df=5, χ²=158.9193)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=64, tsamples=1142857, bsamples=64, df=5, χ²=216.1296)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=66, tsamples=1142857, bsamples=64, df=5, χ²=64.6009)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=67, tsamples=1142857, bsamples=64, df=5, χ²=180.9455)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=68, tsamples=1142857, bsamples=64, df=5, χ²=302.6280)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=70, tsamples=1142857, bsamples=64, df=5, χ²=158.2732)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=71, tsamples=1142857, bsamples=64, df=5, χ²=164.5412)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=72, tsamples=1142857, bsamples=64, df=5, χ²=906.5436)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=73, tsamples=1142857, bsamples=64, df=5, χ²=191.4568)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=75, tsamples=1142857, bsamples=64, df=5, χ²=109.5511)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=76, tsamples=1142857, bsamples=64, df=5, χ²=84.5098)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=77, tsamples=1142857, bsamples=64, df=5, χ²=99.9667)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=78, tsamples=1142857, bsamples=64, df=5, χ²=86.7340)
  - `dieharder::bit_distribution`: p = 0.000359  (width=7, pattern=79, tsamples=1142857, bsamples=64, df=5, χ²=22.8578)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=80, tsamples=1142857, bsamples=64, df=5, χ²=55.0796)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=82, tsamples=1142857, bsamples=64, df=5, χ²=47.5476)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=83, tsamples=1142857, bsamples=64, df=5, χ²=308.5743)
  - `dieharder::bit_distribution`: p = 0.001475  (width=7, pattern=84, tsamples=1142857, bsamples=64, df=5, χ²=19.6157)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=85, tsamples=1142857, bsamples=64, df=5, χ²=323.2610)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=86, tsamples=1142857, bsamples=64, df=5, χ²=235.4935)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=87, tsamples=1142857, bsamples=64, df=5, χ²=189.3184)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=88, tsamples=1142857, bsamples=64, df=5, χ²=755.4224)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=89, tsamples=1142857, bsamples=64, df=5, χ²=175.6997)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=90, tsamples=1142857, bsamples=64, df=5, χ²=46.7006)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=91, tsamples=1142857, bsamples=64, df=5, χ²=82.1653)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=92, tsamples=1142857, bsamples=64, df=5, χ²=132.4041)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=95, tsamples=1142857, bsamples=64, df=5, χ²=1275.7542)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=96, tsamples=1142857, bsamples=64, df=5, χ²=58.3013)
  - `dieharder::bit_distribution`: p = 0.000006  (width=7, pattern=97, tsamples=1142857, bsamples=64, df=5, χ²=32.1305)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=98, tsamples=1142857, bsamples=64, df=5, χ²=202.1119)
  - `dieharder::bit_distribution`: p = 0.000235  (width=7, pattern=99, tsamples=1142857, bsamples=64, df=5, χ²=23.8213)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=100, tsamples=1142857, bsamples=64, df=5, χ²=200.9319)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=102, tsamples=1142857, bsamples=64, df=5, χ²=87.1214)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=103, tsamples=1142857, bsamples=64, df=5, χ²=593.8654)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=104, tsamples=1142857, bsamples=64, df=5, χ²=505.2755)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=105, tsamples=1142857, bsamples=64, df=5, χ²=437.9882)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=107, tsamples=1142857, bsamples=64, df=5, χ²=501.5168)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=108, tsamples=1142857, bsamples=64, df=5, χ²=1121.6804)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=110, tsamples=1142857, bsamples=64, df=5, χ²=145.2410)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=111, tsamples=1142857, bsamples=64, df=5, χ²=663.8479)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=112, tsamples=1142857, bsamples=64, df=5, χ²=187.6830)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=113, tsamples=1142857, bsamples=64, df=5, χ²=68.3322)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=115, tsamples=1142857, bsamples=64, df=5, χ²=67.2104)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=117, tsamples=1142857, bsamples=64, df=5, χ²=154.4368)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=118, tsamples=1142857, bsamples=64, df=5, χ²=1412.4914)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=119, tsamples=1142857, bsamples=64, df=5, χ²=175.5134)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=120, tsamples=1142857, bsamples=64, df=5, χ²=125.9630)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=121, tsamples=1142857, bsamples=64, df=5, χ²=150.2927)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=122, tsamples=1142857, bsamples=64, df=5, χ²=156.6896)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=123, tsamples=1142857, bsamples=64, df=5, χ²=227.3872)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=125, tsamples=1142857, bsamples=64, df=5, χ²=434.3703)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=126, tsamples=1142857, bsamples=64, df=5, χ²=150.7513)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=127, tsamples=1142857, bsamples=64, df=5, χ²=459.3311)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=0, tsamples=1000000, bsamples=64, df=4, χ²=423.4332)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=1, tsamples=1000000, bsamples=64, df=4, χ²=178.5241)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=2, tsamples=1000000, bsamples=64, df=4, χ²=209.2439)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=3, tsamples=1000000, bsamples=64, df=4, χ²=217.4302)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=4, tsamples=1000000, bsamples=64, df=4, χ²=519.9527)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=5, tsamples=1000000, bsamples=64, df=4, χ²=593.2202)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=6, tsamples=1000000, bsamples=64, df=4, χ²=387.6493)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=7, tsamples=1000000, bsamples=64, df=4, χ²=852.6198)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=8, tsamples=1000000, bsamples=64, df=4, χ²=43.3688)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=9, tsamples=1000000, bsamples=64, df=4, χ²=214.8719)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=10, tsamples=1000000, bsamples=64, df=4, χ²=688.6656)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=11, tsamples=1000000, bsamples=64, df=4, χ²=819.6254)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=12, tsamples=1000000, bsamples=64, df=4, χ²=386.8597)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=13, tsamples=1000000, bsamples=64, df=4, χ²=593.5931)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=14, tsamples=1000000, bsamples=64, df=4, χ²=650.3961)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=15, tsamples=1000000, bsamples=64, df=4, χ²=293.2047)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=16, tsamples=1000000, bsamples=64, df=4, χ²=365.5868)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=17, tsamples=1000000, bsamples=64, df=4, χ²=443.4517)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=18, tsamples=1000000, bsamples=64, df=4, χ²=672.6719)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=19, tsamples=1000000, bsamples=64, df=4, χ²=562.9371)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=20, tsamples=1000000, bsamples=64, df=4, χ²=287.2127)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=21, tsamples=1000000, bsamples=64, df=4, χ²=350.5440)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=22, tsamples=1000000, bsamples=64, df=4, χ²=212.7285)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=23, tsamples=1000000, bsamples=64, df=4, χ²=525.3942)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=24, tsamples=1000000, bsamples=64, df=4, χ²=307.8167)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=25, tsamples=1000000, bsamples=64, df=4, χ²=145.7281)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=26, tsamples=1000000, bsamples=64, df=4, χ²=43.4513)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=27, tsamples=1000000, bsamples=64, df=4, χ²=662.1044)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=28, tsamples=1000000, bsamples=64, df=4, χ²=709.7664)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=29, tsamples=1000000, bsamples=64, df=4, χ²=1227.3708)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=30, tsamples=1000000, bsamples=64, df=4, χ²=636.2967)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=31, tsamples=1000000, bsamples=64, df=4, χ²=1212.3892)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=32, tsamples=1000000, bsamples=64, df=4, χ²=462.7363)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=33, tsamples=1000000, bsamples=64, df=4, χ²=338.8900)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=34, tsamples=1000000, bsamples=64, df=4, χ²=568.4099)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=35, tsamples=1000000, bsamples=64, df=4, χ²=743.7909)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=36, tsamples=1000000, bsamples=64, df=4, χ²=395.5698)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=37, tsamples=1000000, bsamples=64, df=4, χ²=220.5300)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=38, tsamples=1000000, bsamples=64, df=4, χ²=377.8561)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=39, tsamples=1000000, bsamples=64, df=4, χ²=501.9351)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=40, tsamples=1000000, bsamples=64, df=4, χ²=209.5992)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=41, tsamples=1000000, bsamples=64, df=4, χ²=230.9994)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=42, tsamples=1000000, bsamples=64, df=4, χ²=679.5072)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=43, tsamples=1000000, bsamples=64, df=4, χ²=526.7728)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=44, tsamples=1000000, bsamples=64, df=4, χ²=442.3457)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=45, tsamples=1000000, bsamples=64, df=4, χ²=424.9998)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=46, tsamples=1000000, bsamples=64, df=4, χ²=352.5052)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=47, tsamples=1000000, bsamples=64, df=4, χ²=617.1468)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=48, tsamples=1000000, bsamples=64, df=4, χ²=908.9865)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=49, tsamples=1000000, bsamples=64, df=4, χ²=548.3090)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=50, tsamples=1000000, bsamples=64, df=4, χ²=525.1352)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=51, tsamples=1000000, bsamples=64, df=4, χ²=727.5799)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=52, tsamples=1000000, bsamples=64, df=4, χ²=426.7365)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=53, tsamples=1000000, bsamples=64, df=4, χ²=752.8263)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=54, tsamples=1000000, bsamples=64, df=4, χ²=336.5395)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=55, tsamples=1000000, bsamples=64, df=4, χ²=224.2312)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=56, tsamples=1000000, bsamples=64, df=4, χ²=240.3230)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=57, tsamples=1000000, bsamples=64, df=4, χ²=260.3318)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=58, tsamples=1000000, bsamples=64, df=4, χ²=216.7026)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=59, tsamples=1000000, bsamples=64, df=4, χ²=644.1759)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=60, tsamples=1000000, bsamples=64, df=4, χ²=518.7175)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=61, tsamples=1000000, bsamples=64, df=4, χ²=284.2278)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=62, tsamples=1000000, bsamples=64, df=4, χ²=526.9505)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=63, tsamples=1000000, bsamples=64, df=4, χ²=678.3387)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=64, tsamples=1000000, bsamples=64, df=4, χ²=443.7166)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=65, tsamples=1000000, bsamples=64, df=4, χ²=174.2625)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=66, tsamples=1000000, bsamples=64, df=4, χ²=210.4400)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=67, tsamples=1000000, bsamples=64, df=4, χ²=164.8663)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=68, tsamples=1000000, bsamples=64, df=4, χ²=499.0321)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=69, tsamples=1000000, bsamples=64, df=4, χ²=1031.2979)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=70, tsamples=1000000, bsamples=64, df=4, χ²=332.7013)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=71, tsamples=1000000, bsamples=64, df=4, χ²=446.7180)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=72, tsamples=1000000, bsamples=64, df=4, χ²=61.4267)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=73, tsamples=1000000, bsamples=64, df=4, χ²=201.0944)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=74, tsamples=1000000, bsamples=64, df=4, χ²=692.0882)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=75, tsamples=1000000, bsamples=64, df=4, χ²=853.2041)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=76, tsamples=1000000, bsamples=64, df=4, χ²=357.7139)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=77, tsamples=1000000, bsamples=64, df=4, χ²=672.3045)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=78, tsamples=1000000, bsamples=64, df=4, χ²=673.2066)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=79, tsamples=1000000, bsamples=64, df=4, χ²=270.7910)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=80, tsamples=1000000, bsamples=64, df=4, χ²=743.2843)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=81, tsamples=1000000, bsamples=64, df=4, χ²=414.5441)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=82, tsamples=1000000, bsamples=64, df=4, χ²=662.5248)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=83, tsamples=1000000, bsamples=64, df=4, χ²=575.1472)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=84, tsamples=1000000, bsamples=64, df=4, χ²=283.5924)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=85, tsamples=1000000, bsamples=64, df=4, χ²=343.1166)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=86, tsamples=1000000, bsamples=64, df=4, χ²=164.4976)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=87, tsamples=1000000, bsamples=64, df=4, χ²=497.1440)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=88, tsamples=1000000, bsamples=64, df=4, χ²=174.0307)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=89, tsamples=1000000, bsamples=64, df=4, χ²=131.4569)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=90, tsamples=1000000, bsamples=64, df=4, χ²=35.6732)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=91, tsamples=1000000, bsamples=64, df=4, χ²=449.1880)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=92, tsamples=1000000, bsamples=64, df=4, χ²=594.8690)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=93, tsamples=1000000, bsamples=64, df=4, χ²=1117.5748)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=94, tsamples=1000000, bsamples=64, df=4, χ²=543.5737)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=95, tsamples=1000000, bsamples=64, df=4, χ²=347.5260)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=96, tsamples=1000000, bsamples=64, df=4, χ²=443.2636)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=97, tsamples=1000000, bsamples=64, df=4, χ²=353.5261)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=98, tsamples=1000000, bsamples=64, df=4, χ²=585.7018)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=99, tsamples=1000000, bsamples=64, df=4, χ²=574.3040)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=100, tsamples=1000000, bsamples=64, df=4, χ²=489.0327)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=101, tsamples=1000000, bsamples=64, df=4, χ²=281.6798)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=102, tsamples=1000000, bsamples=64, df=4, χ²=366.1423)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=103, tsamples=1000000, bsamples=64, df=4, χ²=228.6270)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=104, tsamples=1000000, bsamples=64, df=4, χ²=246.2106)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=105, tsamples=1000000, bsamples=64, df=4, χ²=169.4633)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=106, tsamples=1000000, bsamples=64, df=4, χ²=724.0753)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=107, tsamples=1000000, bsamples=64, df=4, χ²=527.5680)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=108, tsamples=1000000, bsamples=64, df=4, χ²=363.8120)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=109, tsamples=1000000, bsamples=64, df=4, χ²=415.5825)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=110, tsamples=1000000, bsamples=64, df=4, χ²=670.1749)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=111, tsamples=1000000, bsamples=64, df=4, χ²=609.9074)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=112, tsamples=1000000, bsamples=64, df=4, χ²=898.3881)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=113, tsamples=1000000, bsamples=64, df=4, χ²=432.8993)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=114, tsamples=1000000, bsamples=64, df=4, χ²=434.7356)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=115, tsamples=1000000, bsamples=64, df=4, χ²=897.8302)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=116, tsamples=1000000, bsamples=64, df=4, χ²=459.9898)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=117, tsamples=1000000, bsamples=64, df=4, χ²=599.7420)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=118, tsamples=1000000, bsamples=64, df=4, χ²=170.8863)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=119, tsamples=1000000, bsamples=64, df=4, χ²=105.9215)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=120, tsamples=1000000, bsamples=64, df=4, χ²=122.2569)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=121, tsamples=1000000, bsamples=64, df=4, χ²=273.3945)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=122, tsamples=1000000, bsamples=64, df=4, χ²=269.7188)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=123, tsamples=1000000, bsamples=64, df=4, χ²=570.6813)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=124, tsamples=1000000, bsamples=64, df=4, χ²=563.8690)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=125, tsamples=1000000, bsamples=64, df=4, χ²=344.5512)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=126, tsamples=1000000, bsamples=64, df=4, χ²=633.0857)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=127, tsamples=1000000, bsamples=64, df=4, χ²=655.7348)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=128, tsamples=1000000, bsamples=64, df=4, χ²=445.5243)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=129, tsamples=1000000, bsamples=64, df=4, χ²=162.7371)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=130, tsamples=1000000, bsamples=64, df=4, χ²=259.9922)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=131, tsamples=1000000, bsamples=64, df=4, χ²=176.4549)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=132, tsamples=1000000, bsamples=64, df=4, χ²=527.9863)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=133, tsamples=1000000, bsamples=64, df=4, χ²=620.0394)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=134, tsamples=1000000, bsamples=64, df=4, χ²=428.4677)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=135, tsamples=1000000, bsamples=64, df=4, χ²=69.3735)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=136, tsamples=1000000, bsamples=64, df=4, χ²=708.2954)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=137, tsamples=1000000, bsamples=64, df=4, χ²=167.7101)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=138, tsamples=1000000, bsamples=64, df=4, χ²=701.0494)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=139, tsamples=1000000, bsamples=64, df=4, χ²=1010.5548)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=140, tsamples=1000000, bsamples=64, df=4, χ²=493.9612)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=141, tsamples=1000000, bsamples=64, df=4, χ²=628.1050)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=142, tsamples=1000000, bsamples=64, df=4, χ²=672.8728)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=143, tsamples=1000000, bsamples=64, df=4, χ²=277.2564)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=144, tsamples=1000000, bsamples=64, df=4, χ²=367.3896)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=145, tsamples=1000000, bsamples=64, df=4, χ²=378.2675)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=146, tsamples=1000000, bsamples=64, df=4, χ²=887.5322)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=147, tsamples=1000000, bsamples=64, df=4, χ²=680.8533)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=148, tsamples=1000000, bsamples=64, df=4, χ²=277.9619)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=149, tsamples=1000000, bsamples=64, df=4, χ²=361.2531)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=150, tsamples=1000000, bsamples=64, df=4, χ²=361.8189)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=151, tsamples=1000000, bsamples=64, df=4, χ²=514.5082)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=152, tsamples=1000000, bsamples=64, df=4, χ²=152.5896)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=153, tsamples=1000000, bsamples=64, df=4, χ²=168.7535)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=154, tsamples=1000000, bsamples=64, df=4, χ²=127.4119)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=155, tsamples=1000000, bsamples=64, df=4, χ²=1242.7709)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=156, tsamples=1000000, bsamples=64, df=4, χ²=608.6234)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=157, tsamples=1000000, bsamples=64, df=4, χ²=1164.7871)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=158, tsamples=1000000, bsamples=64, df=4, χ²=497.0954)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=159, tsamples=1000000, bsamples=64, df=4, χ²=444.9363)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=160, tsamples=1000000, bsamples=64, df=4, χ²=434.0631)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=161, tsamples=1000000, bsamples=64, df=4, χ²=338.7680)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=162, tsamples=1000000, bsamples=64, df=4, χ²=616.6940)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=163, tsamples=1000000, bsamples=64, df=4, χ²=629.5730)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=164, tsamples=1000000, bsamples=64, df=4, χ²=362.7010)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=165, tsamples=1000000, bsamples=64, df=4, χ²=237.0572)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=166, tsamples=1000000, bsamples=64, df=4, χ²=342.7136)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=167, tsamples=1000000, bsamples=64, df=4, χ²=296.2244)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=168, tsamples=1000000, bsamples=64, df=4, χ²=92.6184)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=169, tsamples=1000000, bsamples=64, df=4, χ²=127.3223)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=170, tsamples=1000000, bsamples=64, df=4, χ²=731.7505)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=171, tsamples=1000000, bsamples=64, df=4, χ²=506.5342)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=172, tsamples=1000000, bsamples=64, df=4, χ²=328.0912)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=173, tsamples=1000000, bsamples=64, df=4, χ²=409.3753)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=174, tsamples=1000000, bsamples=64, df=4, χ²=357.0621)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=175, tsamples=1000000, bsamples=64, df=4, χ²=771.5215)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=176, tsamples=1000000, bsamples=64, df=4, χ²=952.0346)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=177, tsamples=1000000, bsamples=64, df=4, χ²=459.5702)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=178, tsamples=1000000, bsamples=64, df=4, χ²=414.4088)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=179, tsamples=1000000, bsamples=64, df=4, χ²=717.2164)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=180, tsamples=1000000, bsamples=64, df=4, χ²=435.0130)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=181, tsamples=1000000, bsamples=64, df=4, χ²=599.8242)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=182, tsamples=1000000, bsamples=64, df=4, χ²=167.5486)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=183, tsamples=1000000, bsamples=64, df=4, χ²=98.6406)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=184, tsamples=1000000, bsamples=64, df=4, χ²=132.6832)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=185, tsamples=1000000, bsamples=64, df=4, χ²=240.5026)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=186, tsamples=1000000, bsamples=64, df=4, χ²=302.6411)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=187, tsamples=1000000, bsamples=64, df=4, χ²=593.6825)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=188, tsamples=1000000, bsamples=64, df=4, χ²=550.7698)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=189, tsamples=1000000, bsamples=64, df=4, χ²=309.7539)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=190, tsamples=1000000, bsamples=64, df=4, χ²=532.9465)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=191, tsamples=1000000, bsamples=64, df=4, χ²=661.0299)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=192, tsamples=1000000, bsamples=64, df=4, χ²=575.0397)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=193, tsamples=1000000, bsamples=64, df=4, χ²=174.1082)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=194, tsamples=1000000, bsamples=64, df=4, χ²=171.1625)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=195, tsamples=1000000, bsamples=64, df=4, χ²=204.2437)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=196, tsamples=1000000, bsamples=64, df=4, χ²=686.3143)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=197, tsamples=1000000, bsamples=64, df=4, χ²=587.2819)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=198, tsamples=1000000, bsamples=64, df=4, χ²=337.3222)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=199, tsamples=1000000, bsamples=64, df=4, χ²=92.5113)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=200, tsamples=1000000, bsamples=64, df=4, χ²=228.3304)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=201, tsamples=1000000, bsamples=64, df=4, χ²=228.5453)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=202, tsamples=1000000, bsamples=64, df=4, χ²=671.0355)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=203, tsamples=1000000, bsamples=64, df=4, χ²=783.7865)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=204, tsamples=1000000, bsamples=64, df=4, χ²=358.1052)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=205, tsamples=1000000, bsamples=64, df=4, χ²=609.9349)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=206, tsamples=1000000, bsamples=64, df=4, χ²=540.3399)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=207, tsamples=1000000, bsamples=64, df=4, χ²=330.0842)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=208, tsamples=1000000, bsamples=64, df=4, χ²=384.4577)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=209, tsamples=1000000, bsamples=64, df=4, χ²=387.3904)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=210, tsamples=1000000, bsamples=64, df=4, χ²=656.2870)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=211, tsamples=1000000, bsamples=64, df=4, χ²=667.6297)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=212, tsamples=1000000, bsamples=64, df=4, χ²=330.6230)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=213, tsamples=1000000, bsamples=64, df=4, χ²=442.7139)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=214, tsamples=1000000, bsamples=64, df=4, χ²=188.5037)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=215, tsamples=1000000, bsamples=64, df=4, χ²=658.3336)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=216, tsamples=1000000, bsamples=64, df=4, χ²=132.3014)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=217, tsamples=1000000, bsamples=64, df=4, χ²=219.9739)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=218, tsamples=1000000, bsamples=64, df=4, χ²=53.4283)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=219, tsamples=1000000, bsamples=64, df=4, χ²=342.7601)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=220, tsamples=1000000, bsamples=64, df=4, χ²=608.6547)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=221, tsamples=1000000, bsamples=64, df=4, χ²=1133.4509)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=222, tsamples=1000000, bsamples=64, df=4, χ²=516.6688)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=223, tsamples=1000000, bsamples=64, df=4, χ²=357.6016)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=224, tsamples=1000000, bsamples=64, df=4, χ²=464.3515)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=225, tsamples=1000000, bsamples=64, df=4, χ²=461.9498)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=226, tsamples=1000000, bsamples=64, df=4, χ²=567.0963)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=227, tsamples=1000000, bsamples=64, df=4, χ²=618.2420)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=228, tsamples=1000000, bsamples=64, df=4, χ²=342.7914)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=229, tsamples=1000000, bsamples=64, df=4, χ²=222.7882)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=230, tsamples=1000000, bsamples=64, df=4, χ²=362.7367)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=231, tsamples=1000000, bsamples=64, df=4, χ²=499.6912)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=232, tsamples=1000000, bsamples=64, df=4, χ²=89.8341)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=233, tsamples=1000000, bsamples=64, df=4, χ²=123.1207)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=234, tsamples=1000000, bsamples=64, df=4, χ²=723.6543)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=235, tsamples=1000000, bsamples=64, df=4, χ²=533.4262)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=236, tsamples=1000000, bsamples=64, df=4, χ²=344.0102)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=237, tsamples=1000000, bsamples=64, df=4, χ²=441.6757)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=238, tsamples=1000000, bsamples=64, df=4, χ²=380.5676)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=239, tsamples=1000000, bsamples=64, df=4, χ²=567.3540)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=240, tsamples=1000000, bsamples=64, df=4, χ²=1222.0790)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=241, tsamples=1000000, bsamples=64, df=4, χ²=438.3042)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=242, tsamples=1000000, bsamples=64, df=4, χ²=584.6992)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=243, tsamples=1000000, bsamples=64, df=4, χ²=873.1182)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=244, tsamples=1000000, bsamples=64, df=4, χ²=562.2894)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=245, tsamples=1000000, bsamples=64, df=4, χ²=587.4667)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=246, tsamples=1000000, bsamples=64, df=4, χ²=160.0815)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=247, tsamples=1000000, bsamples=64, df=4, χ²=221.5646)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=248, tsamples=1000000, bsamples=64, df=4, χ²=118.0597)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=249, tsamples=1000000, bsamples=64, df=4, χ²=300.6836)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=250, tsamples=1000000, bsamples=64, df=4, χ²=287.9672)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=251, tsamples=1000000, bsamples=64, df=4, χ²=588.2766)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=252, tsamples=1000000, bsamples=64, df=4, χ²=645.3547)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=253, tsamples=1000000, bsamples=64, df=4, χ²=387.2671)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=254, tsamples=1000000, bsamples=64, df=4, χ²=481.2019)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=255, tsamples=1000000, bsamples=64, df=4, χ²=677.9089)
  - `dieharder::gcd_distribution`: p = 0.000000  (pairs=100000, gtblsize=24, χ²=24380.6073)
  - `dieharder::gcd_step_counts`: p = 0.000000  (pairs=100000, χ²=301.1362)

### BAD Windows .NET Random(seed=1) compat

- `7` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.000260  (B=000010011, N=8, M=2000000, χ²=29.4907)
  - `nist::non_overlapping_template`: p = 0.002848  (B=001101011, N=8, M=2000000, χ²=23.4351)
  - `nist::non_overlapping_template`: p = 0.008462  (B=001111011, N=8, M=2000000, χ²=20.5443)
  - `dieharder::dct`: p = 0.002305  (ntuple=256, tsamples=5000, χ²=323.6736)
  - `dieharder::bit_distribution`: p = 0.009510  (width=6, pattern=53, tsamples=1333333, bsamples=64, df=7, χ²=18.6075)
  - `dieharder::bit_distribution`: p = 0.008266  (width=8, pattern=114, tsamples=1000000, bsamples=64, df=4, χ²=13.7141)
  - `dieharder::bit_distribution`: p = 0.006723  (width=8, pattern=171, tsamples=1000000, bsamples=64, df=4, χ²=14.1863)

### ANSI C sample LCG (1103515245,12345; seed=1)

- `697` failures out of `714` tests:
  - `nist::frequency`: p = 0.000000  (n=16000000, S_n=-495934, s_obs=123.9835)
  - `nist::block_frequency`: p = 0.000000  (n=16000000, M=128, N=125000, χ²=138257.9688)
  - `nist::runs`: p = 0.000000  (pre-test failed: π=0.4845)
  - `nist::longest_run`: p = 0.000000  (n=16000000, M=10000, N=1600, χ²=243.9337)
  - `nist::matrix_rank`: p = 0.000000  (N=15625, F32=0, F31=9016, F≤30=6609, χ²=14306.0238)
  - `nist::spectral`: p = 0.000000  (n=16000000, N₀=7600000.0, N₁=7808351, T=6923.2735, d=477.9900)
  - `nist::overlapping_template`: p = 0.000000  (n=16000000, m=9, N=15503, ν=[7326, 2881, 1915, 1263, 810, 1308], χ²=999.2733)
  - `nist::universal`: p = 0.000007  (n=16000000, L=10, Q=10240, K=1589760, f_n=9.1764, μ=9.1723, σ=0.000915)
  - `nist::approximate_entropy`: p = 0.000000  (n=16000000, m=10, ApEn=0.692640, χ²=16237.1531)
  - `nist::cumulative_sums_forward`: p = 0.000000  (n=16000000, z=495976)
  - `nist::cumulative_sums_backward`: p = 0.000000  (n=16000000, z=495980)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000000001, N=8, M=2000000, χ²=1561.3566)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000000011, N=8, M=2000000, χ²=771.9684)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000000101, N=8, M=2000000, χ²=823.4215)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000000111, N=8, M=2000000, χ²=262.6766)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000001001, N=8, M=2000000, χ²=754.1113)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000001011, N=8, M=2000000, χ²=301.3373)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000001101, N=8, M=2000000, χ²=329.5638)
  - `nist::non_overlapping_template`: p = 0.000008  (B=000001111, N=8, M=2000000, χ²=37.9371)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000010001, N=8, M=2000000, χ²=845.8791)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000010011, N=8, M=2000000, χ²=271.1930)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000010101, N=8, M=2000000, χ²=304.4811)
  - `nist::non_overlapping_template`: p = 0.000001  (B=000010111, N=8, M=2000000, χ²=44.1660)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000011001, N=8, M=2000000, χ²=277.2198)
  - `nist::non_overlapping_template`: p = 0.000110  (B=000011011, N=8, M=2000000, χ²=31.6053)
  - `nist::non_overlapping_template`: p = 0.004113  (B=000011101, N=8, M=2000000, χ²=22.4719)
  - `nist::non_overlapping_template`: p = 0.000001  (B=000011111, N=8, M=2000000, χ²=42.6260)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000100011, N=8, M=2000000, χ²=314.0600)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000100101, N=8, M=2000000, χ²=286.7076)
  - `nist::non_overlapping_template`: p = 0.000011  (B=000100111, N=8, M=2000000, χ²=37.0207)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000101001, N=8, M=2000000, χ²=296.8489)
  - `nist::non_overlapping_template`: p = 0.000019  (B=000101011, N=8, M=2000000, χ²=35.7810)
  - `nist::non_overlapping_template`: p = 0.000004  (B=000101101, N=8, M=2000000, χ²=39.4819)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000101111, N=8, M=2000000, χ²=46.8205)
  - `nist::non_overlapping_template`: p = 0.000833  (B=000110011, N=8, M=2000000, χ²=26.5878)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000110101, N=8, M=2000000, χ²=44.3195)
  - `nist::non_overlapping_template`: p = 0.000040  (B=000110111, N=8, M=2000000, χ²=34.0181)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000111001, N=8, M=2000000, χ²=55.0607)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000111011, N=8, M=2000000, χ²=51.5416)
  - `nist::non_overlapping_template`: p = 0.000012  (B=000111101, N=8, M=2000000, χ²=36.8970)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000111111, N=8, M=2000000, χ²=276.9682)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001000011, N=8, M=2000000, χ²=260.3470)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001000101, N=8, M=2000000, χ²=275.1861)
  - `nist::non_overlapping_template`: p = 0.000278  (B=001000111, N=8, M=2000000, χ²=29.3297)
  - `nist::non_overlapping_template`: p = 0.004719  (B=001001011, N=8, M=2000000, χ²=22.1086)
  - `nist::non_overlapping_template`: p = 0.000004  (B=001001101, N=8, M=2000000, χ²=39.3092)
  - `nist::non_overlapping_template`: p = 0.000373  (B=001001111, N=8, M=2000000, χ²=28.5955)
  - `nist::non_overlapping_template`: p = 0.000178  (B=001010011, N=8, M=2000000, χ²=30.4213)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001010101, N=8, M=2000000, χ²=64.2681)
  - `nist::non_overlapping_template`: p = 0.000011  (B=001010111, N=8, M=2000000, χ²=37.1273)
  - `nist::non_overlapping_template`: p = 0.000015  (B=001011011, N=8, M=2000000, χ²=36.4411)
  - `nist::non_overlapping_template`: p = 0.000042  (B=001011101, N=8, M=2000000, χ²=33.9206)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001011111, N=8, M=2000000, χ²=274.7581)
  - `nist::non_overlapping_template`: p = 0.000006  (B=001100101, N=8, M=2000000, χ²=38.7226)
  - `nist::non_overlapping_template`: p = 0.000005  (B=001100111, N=8, M=2000000, χ²=38.9109)
  - `nist::non_overlapping_template`: p = 0.000295  (B=001101011, N=8, M=2000000, χ²=29.1798)
  - `nist::non_overlapping_template`: p = 0.000101  (B=001101101, N=8, M=2000000, χ²=31.8137)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001101111, N=8, M=2000000, χ²=278.7682)
  - `nist::non_overlapping_template`: p = 0.000034  (B=001110101, N=8, M=2000000, χ²=34.3950)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001110111, N=8, M=2000000, χ²=310.4098)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001111011, N=8, M=2000000, χ²=251.9288)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001111101, N=8, M=2000000, χ²=299.6217)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001111111, N=8, M=2000000, χ²=734.5769)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010000011, N=8, M=2000000, χ²=304.8782)
  - `nist::non_overlapping_template`: p = 0.000059  (B=010000111, N=8, M=2000000, χ²=33.1225)
  - `nist::non_overlapping_template`: p = 0.000016  (B=010001011, N=8, M=2000000, χ²=36.2065)
  - `nist::non_overlapping_template`: p = 0.000001  (B=010001111, N=8, M=2000000, χ²=42.1419)
  - `nist::non_overlapping_template`: p = 0.000013  (B=010010011, N=8, M=2000000, χ²=36.6933)
  - `nist::non_overlapping_template`: p = 0.000008  (B=010010111, N=8, M=2000000, χ²=37.8532)
  - `nist::non_overlapping_template`: p = 0.000001  (B=010011011, N=8, M=2000000, χ²=43.6728)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010011111, N=8, M=2000000, χ²=261.7076)
  - `nist::non_overlapping_template`: p = 0.000001  (B=010100011, N=8, M=2000000, χ²=43.2670)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010100111, N=8, M=2000000, χ²=46.8159)
  - `nist::non_overlapping_template`: p = 0.000652  (B=010101011, N=8, M=2000000, χ²=27.2017)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010101111, N=8, M=2000000, χ²=272.2838)
  - `nist::non_overlapping_template`: p = 0.000236  (B=010110011, N=8, M=2000000, χ²=29.7276)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010110111, N=8, M=2000000, χ²=286.6503)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010111011, N=8, M=2000000, χ²=245.1761)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010111111, N=8, M=2000000, χ²=735.6060)
  - `nist::non_overlapping_template`: p = 0.000010  (B=011000111, N=8, M=2000000, χ²=37.4041)
  - `nist::non_overlapping_template`: p = 0.000000  (B=011001111, N=8, M=2000000, χ²=263.8815)
  - `nist::non_overlapping_template`: p = 0.000000  (B=011010111, N=8, M=2000000, χ²=255.0639)
  - `nist::non_overlapping_template`: p = 0.000000  (B=011011111, N=8, M=2000000, χ²=768.9849)
  - `nist::non_overlapping_template`: p = 0.000000  (B=011101111, N=8, M=2000000, χ²=766.2961)
  - `nist::non_overlapping_template`: p = 0.000000  (B=011111111, N=8, M=2000000, χ²=1482.5367)
  - `nist::non_overlapping_template`: p = 0.000000  (B=100000000, N=8, M=2000000, χ²=1561.3566)
  - `nist::non_overlapping_template`: p = 0.000000  (B=100010000, N=8, M=2000000, χ²=888.8757)
  - `nist::non_overlapping_template`: p = 0.000000  (B=100100000, N=8, M=2000000, χ²=813.0804)
  - `nist::non_overlapping_template`: p = 0.000000  (B=100101000, N=8, M=2000000, χ²=262.6744)
  - `nist::non_overlapping_template`: p = 0.000000  (B=100110000, N=8, M=2000000, χ²=258.4156)
  - `nist::non_overlapping_template`: p = 0.000002  (B=100111000, N=8, M=2000000, χ²=40.5878)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101000000, N=8, M=2000000, χ²=735.9677)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101000100, N=8, M=2000000, χ²=338.0678)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101001000, N=8, M=2000000, χ²=261.9028)
  - `nist::non_overlapping_template`: p = 0.000060  (B=101001100, N=8, M=2000000, χ²=33.0660)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101010000, N=8, M=2000000, χ²=286.4036)
  - `nist::non_overlapping_template`: p = 0.000001  (B=101010100, N=8, M=2000000, χ²=43.3952)
  - `nist::non_overlapping_template`: p = 0.000029  (B=101011000, N=8, M=2000000, χ²=34.8323)
  - `nist::non_overlapping_template`: p = 0.004659  (B=101011100, N=8, M=2000000, χ²=22.1421)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101100000, N=8, M=2000000, χ²=289.5684)
  - `nist::non_overlapping_template`: p = 0.000266  (B=101100100, N=8, M=2000000, χ²=29.4353)
  - `nist::non_overlapping_template`: p = 0.000004  (B=101101000, N=8, M=2000000, χ²=39.2963)
  - `nist::non_overlapping_template`: p = 0.000003  (B=101101100, N=8, M=2000000, χ²=39.8851)
  - `nist::non_overlapping_template`: p = 0.000001  (B=101110000, N=8, M=2000000, χ²=42.2341)
  - `nist::non_overlapping_template`: p = 0.002026  (B=101110100, N=8, M=2000000, χ²=24.3184)
  - `nist::non_overlapping_template`: p = 0.000221  (B=101111000, N=8, M=2000000, χ²=29.8951)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101111100, N=8, M=2000000, χ²=291.3543)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110000000, N=8, M=2000000, χ²=789.4454)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110000010, N=8, M=2000000, χ²=294.6265)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110000100, N=8, M=2000000, χ²=263.4440)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110001000, N=8, M=2000000, χ²=320.8368)
  - `nist::non_overlapping_template`: p = 0.000046  (B=110001010, N=8, M=2000000, χ²=33.7050)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110010000, N=8, M=2000000, χ²=292.0245)
  - `nist::non_overlapping_template`: p = 0.000006  (B=110010010, N=8, M=2000000, χ²=38.4048)
  - `nist::non_overlapping_template`: p = 0.006687  (B=110010100, N=8, M=2000000, χ²=21.1790)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110011000, N=8, M=2000000, χ²=54.4254)
  - `nist::non_overlapping_template`: p = 0.000091  (B=110011010, N=8, M=2000000, χ²=32.0477)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110100000, N=8, M=2000000, χ²=295.3998)
  - `nist::non_overlapping_template`: p = 0.000014  (B=110100010, N=8, M=2000000, χ²=36.5580)
  - `nist::non_overlapping_template`: p = 0.000016  (B=110100100, N=8, M=2000000, χ²=36.2361)
  - `nist::non_overlapping_template`: p = 0.000003  (B=110101000, N=8, M=2000000, χ²=40.2570)
  - `nist::non_overlapping_template`: p = 0.000003  (B=110101010, N=8, M=2000000, χ²=40.0463)
  - `nist::non_overlapping_template`: p = 0.000045  (B=110101100, N=8, M=2000000, χ²=33.7516)
  - `nist::non_overlapping_template`: p = 0.000009  (B=110110000, N=8, M=2000000, χ²=37.4620)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110110010, N=8, M=2000000, χ²=44.6931)
  - `nist::non_overlapping_template`: p = 0.000001  (B=110110100, N=8, M=2000000, χ²=42.4555)
  - `nist::non_overlapping_template`: p = 0.000034  (B=110111000, N=8, M=2000000, χ²=34.4155)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110111010, N=8, M=2000000, χ²=261.1702)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110111100, N=8, M=2000000, χ²=256.0534)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111000000, N=8, M=2000000, χ²=284.3438)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111000010, N=8, M=2000000, χ²=51.4421)
  - `nist::non_overlapping_template`: p = 0.000001  (B=111000100, N=8, M=2000000, χ²=41.8948)
  - `nist::non_overlapping_template`: p = 0.000519  (B=111000110, N=8, M=2000000, χ²=27.7746)
  - `nist::non_overlapping_template`: p = 0.000001  (B=111001000, N=8, M=2000000, χ²=42.9340)
  - `nist::non_overlapping_template`: p = 0.000101  (B=111001010, N=8, M=2000000, χ²=31.7989)
  - `nist::non_overlapping_template`: p = 0.000278  (B=111001100, N=8, M=2000000, χ²=29.3261)
  - `nist::non_overlapping_template`: p = 0.000317  (B=111010000, N=8, M=2000000, χ²=29.0040)
  - `nist::non_overlapping_template`: p = 0.000002  (B=111010010, N=8, M=2000000, χ²=41.7331)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111010100, N=8, M=2000000, χ²=52.0395)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111010110, N=8, M=2000000, χ²=243.0482)
  - `nist::non_overlapping_template`: p = 0.000091  (B=111011000, N=8, M=2000000, χ²=32.0666)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111011010, N=8, M=2000000, χ²=307.6459)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111011100, N=8, M=2000000, χ²=272.2709)
  - `nist::non_overlapping_template`: p = 0.000206  (B=111100000, N=8, M=2000000, χ²=30.0647)
  - `nist::non_overlapping_template`: p = 0.000006  (B=111100010, N=8, M=2000000, χ²=38.6022)
  - `nist::non_overlapping_template`: p = 0.000006  (B=111100100, N=8, M=2000000, χ²=38.5856)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111100110, N=8, M=2000000, χ²=252.8765)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111101000, N=8, M=2000000, χ²=52.2868)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111101010, N=8, M=2000000, χ²=296.4499)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111101100, N=8, M=2000000, χ²=253.0491)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111101110, N=8, M=2000000, χ²=765.7184)
  - `nist::non_overlapping_template`: p = 0.000139  (B=111110000, N=8, M=2000000, χ²=31.0256)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111110010, N=8, M=2000000, χ²=259.9584)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111110100, N=8, M=2000000, χ²=335.6153)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111110110, N=8, M=2000000, χ²=747.7105)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111111000, N=8, M=2000000, χ²=247.4288)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111111010, N=8, M=2000000, χ²=815.7401)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111111100, N=8, M=2000000, χ²=705.9195)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111111110, N=8, M=2000000, χ²=1482.5367)
  - `nist::serial_delta2`: p = 0.000000  (n=16000000, m=3, ∇ψ²=15373.5039)
  - `maurer::universal_l06`: p = 0.000000  (n=16000000, L=6, Q=640, K=2666026, f_n=5.2211, μ=5.2177, σ=0.000597)
  - `maurer::universal_l08`: p = 0.000000  (n=16000000, L=8, Q=2560, K=1997440, f_n=7.1951, μ=7.1837, σ=0.000767)
  - `maurer::universal_l09`: p = 0.004475  (n=16000000, L=9, Q=5120, K=1772657, f_n=8.1788, μ=8.1764, σ=0.000842)
  - `maurer::universal_l10`: p = 0.000007  (n=16000000, L=10, Q=10240, K=1589760, f_n=9.1764, μ=9.1723, σ=0.000915)
  - `maurer::universal_l12`: p = 0.000084  (n=16000000, L=12, Q=40960, K=1292373, f_n=11.1646, μ=11.1688, σ=0.001067)
  - `maurer::universal_l16`: p = 0.000000  (n=16000000, L=16, Q=655360, K=344640, f_n=15.2642, μ=15.1674, σ=0.002360)
  - `diehard::binary_rank_32x32`: p = 0.000000  (32×32, N=40000, χ²=39671.6425)
  - `diehard::binary_rank_6x8`: p = 0.000000  (N=100000, χ²=3313.0145)
  - `diehard::bitstream`: p = 0.000000  (window=20-bit, stream=2^21, repeats=20)
  - `diehard::opso`: p = 0.000000  (missing=523776, z=1314.6860)
  - `diehard::oqso`: p = 0.000000  (missing=409563, z=908.3593)
  - `diehard::dna`: p = 0.000000  (missing=423813, z=835.7866)
  - `diehard::count_ones_stream`: p = 0.000000  (n=256000, Q5=13907.59, Q4=8451.59, Q5-Q4=5455.99, Z=41.8041)
  - `diehard::parking_lot`: p = 0.000000  (attempts=12000, mean=3523, σ=21.9, repeats=10)
  - `diehard::minimum_distance_2d`: p = 0.000000  (n=8000, side=10000, repeats=100 [BUGGY FORMULA — see diehard_2dsphere.c; use minimum_distance_nd(d=2) instead])
  - `diehard::spheres_3d`: p = 0.000000  (n=4000, cube=1000, repeats=20)
  - `diehard::squeeze`: p = 0.000000  (trials=100000, cells=43, df=37, χ²=1887424.7810)
  - `diehard::craps_wins`: p = 0.000000  (games=200000, wins=124455, z=115.7019)
  - `diehard::craps_throws`: p = 0.000000  (games=200000, df=21, χ²=74691.7990)
  - `dieharder::minimum_distance_nd`: p = 0.000000  (d=5, n=8000, repeats=100)
  - `dieharder::lagged_sums`: p = 0.000000  (lag=1, tsamples=8000000, sum=2000458.1510, z=-2448.9286)
  - `dieharder::lagged_sums`: p = 0.000000  (lag=100, tsamples=158415, sum=39490.5395, z=-345.6754)
  - `dieharder::ks_uniform`: p = 0.000000  (tsamples=16000000)
  - `dieharder::byte_distribution`: p = 0.000000  (tsamples=5333333, streams=9, expected/cell=20833.3, χ²=16000798.1973)
  - `dieharder::dct`: p = 0.000000  (ntuple=256, tsamples=5000, χ²=160005.3120)
  - `dieharder::bit_distribution`: p = 0.000000  (width=1, pattern=0, tsamples=8000000, bsamples=64, df=36, χ²=513345.0846)
  - `dieharder::bit_distribution`: p = 0.000000  (width=1, pattern=1, tsamples=8000000, bsamples=64, df=36, χ²=513345.0846)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=0, tsamples=4000000, bsamples=64, df=30, χ²=339808.1356)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=1, tsamples=4000000, bsamples=64, df=30, χ²=340420.3064)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=2, tsamples=4000000, bsamples=64, df=30, χ²=339452.8754)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=3, tsamples=4000000, bsamples=64, df=30, χ²=340427.3291)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=0, tsamples=2666666, bsamples=64, df=21, χ²=215544.9054)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=1, tsamples=2666666, bsamples=64, df=21, χ²=29561.3247)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=2, tsamples=2666666, bsamples=64, df=21, χ²=34288.9304)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=3, tsamples=2666666, bsamples=64, df=21, χ²=23951.7928)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=4, tsamples=2666666, bsamples=64, df=21, χ²=34780.6518)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=5, tsamples=2666666, bsamples=64, df=21, χ²=29505.0832)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=6, tsamples=2666666, bsamples=64, df=21, χ²=25388.6673)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=7, tsamples=2666666, bsamples=64, df=21, χ²=218205.0739)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=0, tsamples=2000000, bsamples=64, df=14, χ²=134186.7548)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=1, tsamples=2000000, bsamples=64, df=14, χ²=133606.3507)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=2, tsamples=2000000, bsamples=64, df=14, χ²=133374.9518)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=3, tsamples=2000000, bsamples=64, df=14, χ²=135658.0000)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=4, tsamples=2000000, bsamples=64, df=14, χ²=133482.3482)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=5, tsamples=2000000, bsamples=64, df=14, χ²=134090.1190)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=6, tsamples=2000000, bsamples=64, df=14, χ²=133480.5156)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=7, tsamples=2000000, bsamples=64, df=14, χ²=135192.8466)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=8, tsamples=2000000, bsamples=64, df=14, χ²=135713.6959)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=9, tsamples=2000000, bsamples=64, df=14, χ²=134215.8494)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=10, tsamples=2000000, bsamples=64, df=14, χ²=133749.4288)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=11, tsamples=2000000, bsamples=64, df=14, χ²=135011.6569)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=12, tsamples=2000000, bsamples=64, df=14, χ²=135478.3465)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=13, tsamples=2000000, bsamples=64, df=14, χ²=134307.9261)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=14, tsamples=2000000, bsamples=64, df=14, χ²=133582.3259)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=15, tsamples=2000000, bsamples=64, df=14, χ²=135662.6733)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=0, tsamples=1600000, bsamples=64, df=10, χ²=81585.7316)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=1, tsamples=1600000, bsamples=64, df=10, χ²=33595.2447)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=2, tsamples=1600000, bsamples=64, df=10, χ²=29404.4038)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=3, tsamples=1600000, bsamples=64, df=10, χ²=4477.2708)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=4, tsamples=1600000, bsamples=64, df=10, χ²=28913.5608)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=5, tsamples=1600000, bsamples=64, df=10, χ²=3837.0414)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=6, tsamples=1600000, bsamples=64, df=10, χ²=3628.8152)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=7, tsamples=1600000, bsamples=64, df=10, χ²=4573.4140)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=8, tsamples=1600000, bsamples=64, df=10, χ²=29353.4847)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=9, tsamples=1600000, bsamples=64, df=10, χ²=3220.5870)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=10, tsamples=1600000, bsamples=64, df=10, χ²=4183.3119)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=11, tsamples=1600000, bsamples=64, df=10, χ²=4061.3380)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=12, tsamples=1600000, bsamples=64, df=10, χ²=3424.4367)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=13, tsamples=1600000, bsamples=64, df=10, χ²=3539.6323)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=14, tsamples=1600000, bsamples=64, df=10, χ²=5706.2667)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=15, tsamples=1600000, bsamples=64, df=10, χ²=29365.6770)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=16, tsamples=1600000, bsamples=64, df=10, χ²=29861.0762)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=17, tsamples=1600000, bsamples=64, df=10, χ²=5910.7140)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=18, tsamples=1600000, bsamples=64, df=10, χ²=8124.7016)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=19, tsamples=1600000, bsamples=64, df=10, χ²=4401.9003)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=20, tsamples=1600000, bsamples=64, df=10, χ²=5050.2816)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=21, tsamples=1600000, bsamples=64, df=10, χ²=3369.2376)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=22, tsamples=1600000, bsamples=64, df=10, χ²=15940.0533)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=23, tsamples=1600000, bsamples=64, df=10, χ²=29122.3837)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=24, tsamples=1600000, bsamples=64, df=10, χ²=3420.3233)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=25, tsamples=1600000, bsamples=64, df=10, χ²=4549.0082)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=26, tsamples=1600000, bsamples=64, df=10, χ²=3709.9302)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=27, tsamples=1600000, bsamples=64, df=10, χ²=28990.3456)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=28, tsamples=1600000, bsamples=64, df=10, χ²=4800.0853)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=29, tsamples=1600000, bsamples=64, df=10, χ²=31462.3172)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=30, tsamples=1600000, bsamples=64, df=10, χ²=29496.9526)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=31, tsamples=1600000, bsamples=64, df=10, χ²=81435.9343)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=0, tsamples=1333333, bsamples=64, df=7, χ²=47972.6949)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=1, tsamples=1333333, bsamples=64, df=7, χ²=47775.4428)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=2, tsamples=1333333, bsamples=64, df=7, χ²=5591.4066)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=3, tsamples=1333333, bsamples=64, df=7, χ²=5359.5002)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=4, tsamples=1333333, bsamples=64, df=7, χ²=46989.7199)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=5, tsamples=1333333, bsamples=64, df=7, χ²=48623.0030)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=6, tsamples=1333333, bsamples=64, df=7, χ²=5892.3929)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=7, tsamples=1333333, bsamples=64, df=7, χ²=5377.8886)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=8, tsamples=1333333, bsamples=64, df=7, χ²=5744.6970)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=9, tsamples=1333333, bsamples=64, df=7, χ²=5392.1381)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=10, tsamples=1333333, bsamples=64, df=7, χ²=5058.8828)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=11, tsamples=1333333, bsamples=64, df=7, χ²=6115.0958)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=12, tsamples=1333333, bsamples=64, df=7, χ²=5702.2692)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=13, tsamples=1333333, bsamples=64, df=7, χ²=6063.8614)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=14, tsamples=1333333, bsamples=64, df=7, χ²=5323.6222)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=15, tsamples=1333333, bsamples=64, df=7, χ²=6235.8762)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=16, tsamples=1333333, bsamples=64, df=7, χ²=47378.3787)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=17, tsamples=1333333, bsamples=64, df=7, χ²=47508.0118)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=18, tsamples=1333333, bsamples=64, df=7, χ²=5239.0193)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=19, tsamples=1333333, bsamples=64, df=7, χ²=5382.9920)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=20, tsamples=1333333, bsamples=64, df=7, χ²=47720.2603)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=21, tsamples=1333333, bsamples=64, df=7, χ²=48816.3112)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=22, tsamples=1333333, bsamples=64, df=7, χ²=5460.6306)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=23, tsamples=1333333, bsamples=64, df=7, χ²=5300.6473)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=24, tsamples=1333333, bsamples=64, df=7, χ²=6123.9364)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=25, tsamples=1333333, bsamples=64, df=7, χ²=5432.8702)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=26, tsamples=1333333, bsamples=64, df=7, χ²=5504.0471)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=27, tsamples=1333333, bsamples=64, df=7, χ²=5800.3156)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=28, tsamples=1333333, bsamples=64, df=7, χ²=5396.1934)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=29, tsamples=1333333, bsamples=64, df=7, χ²=7556.4234)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=30, tsamples=1333333, bsamples=64, df=7, χ²=5582.1890)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=31, tsamples=1333333, bsamples=64, df=7, χ²=5584.7839)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=32, tsamples=1333333, bsamples=64, df=7, χ²=5778.2360)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=33, tsamples=1333333, bsamples=64, df=7, χ²=5375.9929)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=34, tsamples=1333333, bsamples=64, df=7, χ²=5586.7223)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=35, tsamples=1333333, bsamples=64, df=7, χ²=6851.4191)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=36, tsamples=1333333, bsamples=64, df=7, χ²=8993.6944)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=37, tsamples=1333333, bsamples=64, df=7, χ²=6713.9739)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=38, tsamples=1333333, bsamples=64, df=7, χ²=5691.6882)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=39, tsamples=1333333, bsamples=64, df=7, χ²=5747.5770)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=40, tsamples=1333333, bsamples=64, df=7, χ²=8404.3575)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=41, tsamples=1333333, bsamples=64, df=7, χ²=6196.0603)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=42, tsamples=1333333, bsamples=64, df=7, χ²=48287.8703)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=43, tsamples=1333333, bsamples=64, df=7, χ²=51229.9008)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=44, tsamples=1333333, bsamples=64, df=7, χ²=5715.2891)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=45, tsamples=1333333, bsamples=64, df=7, χ²=5352.9106)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=46, tsamples=1333333, bsamples=64, df=7, χ²=47753.3979)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=47, tsamples=1333333, bsamples=64, df=7, χ²=48620.0715)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=48, tsamples=1333333, bsamples=64, df=7, χ²=6686.4799)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=49, tsamples=1333333, bsamples=64, df=7, χ²=7323.1399)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=50, tsamples=1333333, bsamples=64, df=7, χ²=5816.6439)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=51, tsamples=1333333, bsamples=64, df=7, χ²=5395.3560)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=52, tsamples=1333333, bsamples=64, df=7, χ²=5777.0649)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=53, tsamples=1333333, bsamples=64, df=7, χ²=7749.2304)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=54, tsamples=1333333, bsamples=64, df=7, χ²=5470.6547)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=55, tsamples=1333333, bsamples=64, df=7, χ²=5799.3226)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=56, tsamples=1333333, bsamples=64, df=7, χ²=7502.6187)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=57, tsamples=1333333, bsamples=64, df=7, χ²=8374.8876)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=58, tsamples=1333333, bsamples=64, df=7, χ²=53847.2762)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=59, tsamples=1333333, bsamples=64, df=7, χ²=47540.0680)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=60, tsamples=1333333, bsamples=64, df=7, χ²=6721.6567)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=61, tsamples=1333333, bsamples=64, df=7, χ²=7386.1002)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=62, tsamples=1333333, bsamples=64, df=7, χ²=48009.0676)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=63, tsamples=1333333, bsamples=64, df=7, χ²=48709.7518)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=0, tsamples=1142857, bsamples=64, df=5, χ²=28462.1718)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=1, tsamples=1142857, bsamples=64, df=5, χ²=14244.1183)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=2, tsamples=1142857, bsamples=64, df=5, χ²=13898.8251)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=3, tsamples=1142857, bsamples=64, df=5, χ²=5447.3681)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=4, tsamples=1142857, bsamples=64, df=5, χ²=14568.7901)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=5, tsamples=1142857, bsamples=64, df=5, χ²=5856.0719)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=6, tsamples=1142857, bsamples=64, df=5, χ²=5243.5974)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=7, tsamples=1142857, bsamples=64, df=5, χ²=956.7757)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=8, tsamples=1142857, bsamples=64, df=5, χ²=14745.7322)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=9, tsamples=1142857, bsamples=64, df=5, χ²=5202.0933)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=10, tsamples=1142857, bsamples=64, df=5, χ²=5355.6687)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=11, tsamples=1142857, bsamples=64, df=5, χ²=667.0579)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=12, tsamples=1142857, bsamples=64, df=5, χ²=5234.0816)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=13, tsamples=1142857, bsamples=64, df=5, χ²=660.5740)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=14, tsamples=1142857, bsamples=64, df=5, χ²=786.8825)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=15, tsamples=1142857, bsamples=64, df=5, χ²=689.5785)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=16, tsamples=1142857, bsamples=64, df=5, χ²=13863.0678)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=17, tsamples=1142857, bsamples=64, df=5, χ²=5694.2807)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=18, tsamples=1142857, bsamples=64, df=5, χ²=5116.4394)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=19, tsamples=1142857, bsamples=64, df=5, χ²=624.7288)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=20, tsamples=1142857, bsamples=64, df=5, χ²=5174.8075)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=21, tsamples=1142857, bsamples=64, df=5, χ²=594.1208)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=22, tsamples=1142857, bsamples=64, df=5, χ²=1554.9789)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=23, tsamples=1142857, bsamples=64, df=5, χ²=590.3767)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=24, tsamples=1142857, bsamples=64, df=5, χ²=5268.1001)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=25, tsamples=1142857, bsamples=64, df=5, χ²=697.5785)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=26, tsamples=1142857, bsamples=64, df=5, χ²=892.8766)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=27, tsamples=1142857, bsamples=64, df=5, χ²=735.5216)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=28, tsamples=1142857, bsamples=64, df=5, χ²=635.2546)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=29, tsamples=1142857, bsamples=64, df=5, χ²=568.4258)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=30, tsamples=1142857, bsamples=64, df=5, χ²=545.4902)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=31, tsamples=1142857, bsamples=64, df=5, χ²=5236.0727)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=32, tsamples=1142857, bsamples=64, df=5, χ²=15512.7771)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=33, tsamples=1142857, bsamples=64, df=5, χ²=5099.6161)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=34, tsamples=1142857, bsamples=64, df=5, χ²=6255.7248)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=35, tsamples=1142857, bsamples=64, df=5, χ²=1166.9786)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=36, tsamples=1142857, bsamples=64, df=5, χ²=4924.9958)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=37, tsamples=1142857, bsamples=64, df=5, χ²=1319.7685)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=38, tsamples=1142857, bsamples=64, df=5, χ²=717.6250)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=39, tsamples=1142857, bsamples=64, df=5, χ²=620.3088)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=40, tsamples=1142857, bsamples=64, df=5, χ²=6134.3444)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=41, tsamples=1142857, bsamples=64, df=5, χ²=656.5829)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=42, tsamples=1142857, bsamples=64, df=5, χ²=936.3722)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=43, tsamples=1142857, bsamples=64, df=5, χ²=625.6809)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=44, tsamples=1142857, bsamples=64, df=5, χ²=942.7601)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=45, tsamples=1142857, bsamples=64, df=5, χ²=601.3388)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=46, tsamples=1142857, bsamples=64, df=5, χ²=1217.8603)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=47, tsamples=1142857, bsamples=64, df=5, χ²=5251.2567)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=48, tsamples=1142857, bsamples=64, df=5, χ²=5340.8134)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=49, tsamples=1142857, bsamples=64, df=5, χ²=618.4478)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=50, tsamples=1142857, bsamples=64, df=5, χ²=803.7324)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=51, tsamples=1142857, bsamples=64, df=5, χ²=1132.6545)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=52, tsamples=1142857, bsamples=64, df=5, χ²=987.0481)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=53, tsamples=1142857, bsamples=64, df=5, χ²=895.1349)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=54, tsamples=1142857, bsamples=64, df=5, χ²=804.6913)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=55, tsamples=1142857, bsamples=64, df=5, χ²=5015.1040)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=56, tsamples=1142857, bsamples=64, df=5, χ²=943.9878)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=57, tsamples=1142857, bsamples=64, df=5, χ²=579.6179)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=58, tsamples=1142857, bsamples=64, df=5, χ²=575.0206)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=59, tsamples=1142857, bsamples=64, df=5, χ²=5013.1175)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=60, tsamples=1142857, bsamples=64, df=5, χ²=874.0740)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=61, tsamples=1142857, bsamples=64, df=5, χ²=5233.4712)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=62, tsamples=1142857, bsamples=64, df=5, χ²=5322.1461)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=63, tsamples=1142857, bsamples=64, df=5, χ²=14046.6102)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=64, tsamples=1142857, bsamples=64, df=5, χ²=15616.0011)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=65, tsamples=1142857, bsamples=64, df=5, χ²=5073.9677)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=66, tsamples=1142857, bsamples=64, df=5, χ²=6250.8619)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=67, tsamples=1142857, bsamples=64, df=5, χ²=607.9845)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=68, tsamples=1142857, bsamples=64, df=5, χ²=5083.0055)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=69, tsamples=1142857, bsamples=64, df=5, χ²=744.5410)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=70, tsamples=1142857, bsamples=64, df=5, χ²=1385.8339)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=71, tsamples=1142857, bsamples=64, df=5, χ²=762.5719)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=72, tsamples=1142857, bsamples=64, df=5, χ²=5324.1989)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=73, tsamples=1142857, bsamples=64, df=5, χ²=1056.7961)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=74, tsamples=1142857, bsamples=64, df=5, χ²=638.6158)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=75, tsamples=1142857, bsamples=64, df=5, χ²=1200.0528)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=76, tsamples=1142857, bsamples=64, df=5, χ²=793.6162)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=77, tsamples=1142857, bsamples=64, df=5, χ²=708.1927)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=78, tsamples=1142857, bsamples=64, df=5, χ²=679.7856)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=79, tsamples=1142857, bsamples=64, df=5, χ²=5136.6059)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=80, tsamples=1142857, bsamples=64, df=5, χ²=5280.9163)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=81, tsamples=1142857, bsamples=64, df=5, χ²=962.9737)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=82, tsamples=1142857, bsamples=64, df=5, χ²=878.6230)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=83, tsamples=1142857, bsamples=64, df=5, χ²=945.6314)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=84, tsamples=1142857, bsamples=64, df=5, χ²=1006.4701)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=85, tsamples=1142857, bsamples=64, df=5, χ²=781.4825)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=86, tsamples=1142857, bsamples=64, df=5, χ²=621.0935)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=87, tsamples=1142857, bsamples=64, df=5, χ²=4896.7107)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=88, tsamples=1142857, bsamples=64, df=5, χ²=799.7665)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=89, tsamples=1142857, bsamples=64, df=5, χ²=828.2943)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=90, tsamples=1142857, bsamples=64, df=5, χ²=597.3101)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=91, tsamples=1142857, bsamples=64, df=5, χ²=5074.6595)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=92, tsamples=1142857, bsamples=64, df=5, χ²=1613.6477)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=93, tsamples=1142857, bsamples=64, df=5, χ²=5079.8896)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=94, tsamples=1142857, bsamples=64, df=5, χ²=5449.2142)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=95, tsamples=1142857, bsamples=64, df=5, χ²=14400.7966)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=96, tsamples=1142857, bsamples=64, df=5, χ²=5253.7856)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=97, tsamples=1142857, bsamples=64, df=5, χ²=614.2559)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=98, tsamples=1142857, bsamples=64, df=5, χ²=693.8365)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=99, tsamples=1142857, bsamples=64, df=5, χ²=998.2185)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=100, tsamples=1142857, bsamples=64, df=5, χ²=847.8992)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=101, tsamples=1142857, bsamples=64, df=5, χ²=584.9120)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=102, tsamples=1142857, bsamples=64, df=5, χ²=568.8634)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=103, tsamples=1142857, bsamples=64, df=5, χ²=5213.5888)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=104, tsamples=1142857, bsamples=64, df=5, χ²=772.3170)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=105, tsamples=1142857, bsamples=64, df=5, χ²=743.2882)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=106, tsamples=1142857, bsamples=64, df=5, χ²=580.0808)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=107, tsamples=1142857, bsamples=64, df=5, χ²=6566.8630)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=108, tsamples=1142857, bsamples=64, df=5, χ²=576.6552)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=109, tsamples=1142857, bsamples=64, df=5, χ²=5492.7668)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=110, tsamples=1142857, bsamples=64, df=5, χ²=4918.4986)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=111, tsamples=1142857, bsamples=64, df=5, χ²=13950.3586)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=112, tsamples=1142857, bsamples=64, df=5, χ²=605.4117)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=113, tsamples=1142857, bsamples=64, df=5, χ²=1159.9935)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=114, tsamples=1142857, bsamples=64, df=5, χ²=1008.6911)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=115, tsamples=1142857, bsamples=64, df=5, χ²=5157.9181)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=116, tsamples=1142857, bsamples=64, df=5, χ²=554.2637)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=117, tsamples=1142857, bsamples=64, df=5, χ²=4996.6177)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=118, tsamples=1142857, bsamples=64, df=5, χ²=5147.4784)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=119, tsamples=1142857, bsamples=64, df=5, χ²=14014.4917)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=120, tsamples=1142857, bsamples=64, df=5, χ²=674.2991)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=121, tsamples=1142857, bsamples=64, df=5, χ²=5406.7677)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=122, tsamples=1142857, bsamples=64, df=5, χ²=5208.5995)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=123, tsamples=1142857, bsamples=64, df=5, χ²=14259.0920)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=124, tsamples=1142857, bsamples=64, df=5, χ²=4955.5568)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=125, tsamples=1142857, bsamples=64, df=5, χ²=14260.4458)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=126, tsamples=1142857, bsamples=64, df=5, χ²=14058.6660)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=127, tsamples=1142857, bsamples=64, df=5, χ²=27899.5205)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=0, tsamples=1000000, bsamples=64, df=4, χ²=15632.2767)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=1, tsamples=1000000, bsamples=64, df=4, χ²=15735.4096)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=2, tsamples=1000000, bsamples=64, df=4, χ²=15927.4776)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=3, tsamples=1000000, bsamples=64, df=4, χ²=15511.8551)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=4, tsamples=1000000, bsamples=64, df=4, χ²=15547.1225)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=5, tsamples=1000000, bsamples=64, df=4, χ²=15987.8010)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=6, tsamples=1000000, bsamples=64, df=4, χ²=15486.2427)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=7, tsamples=1000000, bsamples=64, df=4, χ²=15743.3302)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=8, tsamples=1000000, bsamples=64, df=4, χ²=15756.2686)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=9, tsamples=1000000, bsamples=64, df=4, χ²=15698.0212)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=10, tsamples=1000000, bsamples=64, df=4, χ²=15716.7447)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=11, tsamples=1000000, bsamples=64, df=4, χ²=15648.2511)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=12, tsamples=1000000, bsamples=64, df=4, χ²=15711.9520)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=13, tsamples=1000000, bsamples=64, df=4, χ²=15860.5181)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=14, tsamples=1000000, bsamples=64, df=4, χ²=15939.9616)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=15, tsamples=1000000, bsamples=64, df=4, χ²=15808.1528)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=16, tsamples=1000000, bsamples=64, df=4, χ²=15673.8636)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=17, tsamples=1000000, bsamples=64, df=4, χ²=15488.9432)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=18, tsamples=1000000, bsamples=64, df=4, χ²=15708.2376)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=19, tsamples=1000000, bsamples=64, df=4, χ²=15762.9636)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=20, tsamples=1000000, bsamples=64, df=4, χ²=15705.2976)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=21, tsamples=1000000, bsamples=64, df=4, χ²=15772.1055)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=22, tsamples=1000000, bsamples=64, df=4, χ²=16141.9604)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=23, tsamples=1000000, bsamples=64, df=4, χ²=15779.2461)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=24, tsamples=1000000, bsamples=64, df=4, χ²=15563.8894)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=25, tsamples=1000000, bsamples=64, df=4, χ²=15742.9453)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=26, tsamples=1000000, bsamples=64, df=4, χ²=15924.7633)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=27, tsamples=1000000, bsamples=64, df=4, χ²=15529.8064)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=28, tsamples=1000000, bsamples=64, df=4, χ²=15884.0431)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=29, tsamples=1000000, bsamples=64, df=4, χ²=15676.6266)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=30, tsamples=1000000, bsamples=64, df=4, χ²=15718.1595)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=31, tsamples=1000000, bsamples=64, df=4, χ²=15238.0422)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=32, tsamples=1000000, bsamples=64, df=4, χ²=15762.9082)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=33, tsamples=1000000, bsamples=64, df=4, χ²=15620.3143)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=34, tsamples=1000000, bsamples=64, df=4, χ²=15646.1733)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=35, tsamples=1000000, bsamples=64, df=4, χ²=15808.1021)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=36, tsamples=1000000, bsamples=64, df=4, χ²=15759.6404)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=37, tsamples=1000000, bsamples=64, df=4, χ²=15402.4723)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=38, tsamples=1000000, bsamples=64, df=4, χ²=15528.8930)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=39, tsamples=1000000, bsamples=64, df=4, χ²=15856.5630)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=40, tsamples=1000000, bsamples=64, df=4, χ²=15858.8179)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=41, tsamples=1000000, bsamples=64, df=4, χ²=15821.8927)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=42, tsamples=1000000, bsamples=64, df=4, χ²=15383.1270)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=43, tsamples=1000000, bsamples=64, df=4, χ²=15628.2503)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=44, tsamples=1000000, bsamples=64, df=4, χ²=15830.9037)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=45, tsamples=1000000, bsamples=64, df=4, χ²=15600.5631)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=46, tsamples=1000000, bsamples=64, df=4, χ²=15690.6896)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=47, tsamples=1000000, bsamples=64, df=4, χ²=15907.8263)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=48, tsamples=1000000, bsamples=64, df=4, χ²=15830.2175)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=49, tsamples=1000000, bsamples=64, df=4, χ²=15763.4760)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=50, tsamples=1000000, bsamples=64, df=4, χ²=15874.5074)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=51, tsamples=1000000, bsamples=64, df=4, χ²=15507.1294)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=52, tsamples=1000000, bsamples=64, df=4, χ²=15664.2925)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=53, tsamples=1000000, bsamples=64, df=4, χ²=15883.0092)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=54, tsamples=1000000, bsamples=64, df=4, χ²=16153.3669)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=55, tsamples=1000000, bsamples=64, df=4, χ²=16363.3920)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=56, tsamples=1000000, bsamples=64, df=4, χ²=15455.1058)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=57, tsamples=1000000, bsamples=64, df=4, χ²=15826.4973)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=58, tsamples=1000000, bsamples=64, df=4, χ²=15840.4144)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=59, tsamples=1000000, bsamples=64, df=4, χ²=15726.9501)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=60, tsamples=1000000, bsamples=64, df=4, χ²=15562.9141)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=61, tsamples=1000000, bsamples=64, df=4, χ²=15708.9030)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=62, tsamples=1000000, bsamples=64, df=4, χ²=15965.6555)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=63, tsamples=1000000, bsamples=64, df=4, χ²=15617.6246)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=64, tsamples=1000000, bsamples=64, df=4, χ²=15822.0143)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=65, tsamples=1000000, bsamples=64, df=4, χ²=15752.9512)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=66, tsamples=1000000, bsamples=64, df=4, χ²=15487.7646)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=67, tsamples=1000000, bsamples=64, df=4, χ²=15546.9264)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=68, tsamples=1000000, bsamples=64, df=4, χ²=15777.1837)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=69, tsamples=1000000, bsamples=64, df=4, χ²=15761.1967)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=70, tsamples=1000000, bsamples=64, df=4, χ²=15615.2602)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=71, tsamples=1000000, bsamples=64, df=4, χ²=15617.7656)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=72, tsamples=1000000, bsamples=64, df=4, χ²=15545.9032)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=73, tsamples=1000000, bsamples=64, df=4, χ²=15360.4333)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=74, tsamples=1000000, bsamples=64, df=4, χ²=15793.1830)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=75, tsamples=1000000, bsamples=64, df=4, χ²=15469.7662)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=76, tsamples=1000000, bsamples=64, df=4, χ²=15687.8347)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=77, tsamples=1000000, bsamples=64, df=4, χ²=15490.2525)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=78, tsamples=1000000, bsamples=64, df=4, χ²=15842.3394)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=79, tsamples=1000000, bsamples=64, df=4, χ²=15586.8954)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=80, tsamples=1000000, bsamples=64, df=4, χ²=15671.5642)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=81, tsamples=1000000, bsamples=64, df=4, χ²=15710.6820)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=82, tsamples=1000000, bsamples=64, df=4, χ²=15403.3334)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=83, tsamples=1000000, bsamples=64, df=4, χ²=15625.3138)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=84, tsamples=1000000, bsamples=64, df=4, χ²=15715.3473)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=85, tsamples=1000000, bsamples=64, df=4, χ²=15633.1808)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=86, tsamples=1000000, bsamples=64, df=4, χ²=15507.9408)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=87, tsamples=1000000, bsamples=64, df=4, χ²=15587.8118)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=88, tsamples=1000000, bsamples=64, df=4, χ²=15937.4599)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=89, tsamples=1000000, bsamples=64, df=4, χ²=16150.9504)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=90, tsamples=1000000, bsamples=64, df=4, χ²=16116.4287)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=91, tsamples=1000000, bsamples=64, df=4, χ²=15710.3147)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=92, tsamples=1000000, bsamples=64, df=4, χ²=15741.4551)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=93, tsamples=1000000, bsamples=64, df=4, χ²=15586.3254)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=94, tsamples=1000000, bsamples=64, df=4, χ²=15695.6300)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=95, tsamples=1000000, bsamples=64, df=4, χ²=16099.0611)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=96, tsamples=1000000, bsamples=64, df=4, χ²=15988.6934)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=97, tsamples=1000000, bsamples=64, df=4, χ²=16054.6623)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=98, tsamples=1000000, bsamples=64, df=4, χ²=15783.8489)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=99, tsamples=1000000, bsamples=64, df=4, χ²=15654.8054)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=100, tsamples=1000000, bsamples=64, df=4, χ²=15613.6803)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=101, tsamples=1000000, bsamples=64, df=4, χ²=16023.3020)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=102, tsamples=1000000, bsamples=64, df=4, χ²=15814.6481)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=103, tsamples=1000000, bsamples=64, df=4, χ²=15604.2566)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=104, tsamples=1000000, bsamples=64, df=4, χ²=15282.8937)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=105, tsamples=1000000, bsamples=64, df=4, χ²=15679.0664)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=106, tsamples=1000000, bsamples=64, df=4, χ²=15744.1527)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=107, tsamples=1000000, bsamples=64, df=4, χ²=15922.9021)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=108, tsamples=1000000, bsamples=64, df=4, χ²=15298.0417)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=109, tsamples=1000000, bsamples=64, df=4, χ²=15789.3956)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=110, tsamples=1000000, bsamples=64, df=4, χ²=15500.9975)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=111, tsamples=1000000, bsamples=64, df=4, χ²=15320.5622)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=112, tsamples=1000000, bsamples=64, df=4, χ²=15643.5550)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=113, tsamples=1000000, bsamples=64, df=4, χ²=15504.9108)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=114, tsamples=1000000, bsamples=64, df=4, χ²=16005.5845)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=115, tsamples=1000000, bsamples=64, df=4, χ²=15662.4789)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=116, tsamples=1000000, bsamples=64, df=4, χ²=15944.2927)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=117, tsamples=1000000, bsamples=64, df=4, χ²=15764.5826)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=118, tsamples=1000000, bsamples=64, df=4, χ²=15408.3549)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=119, tsamples=1000000, bsamples=64, df=4, χ²=15811.5982)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=120, tsamples=1000000, bsamples=64, df=4, χ²=16110.7117)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=121, tsamples=1000000, bsamples=64, df=4, χ²=15581.6049)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=122, tsamples=1000000, bsamples=64, df=4, χ²=15710.0233)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=123, tsamples=1000000, bsamples=64, df=4, χ²=15769.9067)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=124, tsamples=1000000, bsamples=64, df=4, χ²=15736.0642)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=125, tsamples=1000000, bsamples=64, df=4, χ²=15602.2415)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=126, tsamples=1000000, bsamples=64, df=4, χ²=15712.5498)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=127, tsamples=1000000, bsamples=64, df=4, χ²=15636.6972)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=128, tsamples=1000000, bsamples=64, df=4, χ²=15696.7118)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=129, tsamples=1000000, bsamples=64, df=4, χ²=15644.1420)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=130, tsamples=1000000, bsamples=64, df=4, χ²=16007.3029)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=131, tsamples=1000000, bsamples=64, df=4, χ²=15684.4277)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=132, tsamples=1000000, bsamples=64, df=4, χ²=15700.9344)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=133, tsamples=1000000, bsamples=64, df=4, χ²=15777.2976)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=134, tsamples=1000000, bsamples=64, df=4, χ²=15679.7708)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=135, tsamples=1000000, bsamples=64, df=4, χ²=15678.0903)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=136, tsamples=1000000, bsamples=64, df=4, χ²=15685.8337)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=137, tsamples=1000000, bsamples=64, df=4, χ²=15697.9161)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=138, tsamples=1000000, bsamples=64, df=4, χ²=15690.2276)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=139, tsamples=1000000, bsamples=64, df=4, χ²=15769.5207)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=140, tsamples=1000000, bsamples=64, df=4, χ²=15726.9187)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=141, tsamples=1000000, bsamples=64, df=4, χ²=15717.6401)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=142, tsamples=1000000, bsamples=64, df=4, χ²=15757.4225)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=143, tsamples=1000000, bsamples=64, df=4, χ²=15760.3692)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=144, tsamples=1000000, bsamples=64, df=4, χ²=15678.0205)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=145, tsamples=1000000, bsamples=64, df=4, χ²=15694.9987)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=146, tsamples=1000000, bsamples=64, df=4, χ²=15706.1908)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=147, tsamples=1000000, bsamples=64, df=4, χ²=15692.2113)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=148, tsamples=1000000, bsamples=64, df=4, χ²=15784.8954)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=149, tsamples=1000000, bsamples=64, df=4, χ²=15739.6930)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=150, tsamples=1000000, bsamples=64, df=4, χ²=15719.8857)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=151, tsamples=1000000, bsamples=64, df=4, χ²=15730.6504)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=152, tsamples=1000000, bsamples=64, df=4, χ²=15703.0717)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=153, tsamples=1000000, bsamples=64, df=4, χ²=15726.5519)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=154, tsamples=1000000, bsamples=64, df=4, χ²=15694.4498)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=155, tsamples=1000000, bsamples=64, df=4, χ²=15752.0742)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=156, tsamples=1000000, bsamples=64, df=4, χ²=15765.8554)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=157, tsamples=1000000, bsamples=64, df=4, χ²=15662.1188)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=158, tsamples=1000000, bsamples=64, df=4, χ²=15686.5888)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=159, tsamples=1000000, bsamples=64, df=4, χ²=15681.7640)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=160, tsamples=1000000, bsamples=64, df=4, χ²=15806.9500)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=161, tsamples=1000000, bsamples=64, df=4, χ²=15712.8668)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=162, tsamples=1000000, bsamples=64, df=4, χ²=15730.3948)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=163, tsamples=1000000, bsamples=64, df=4, χ²=15678.7635)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=164, tsamples=1000000, bsamples=64, df=4, χ²=15671.6892)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=165, tsamples=1000000, bsamples=64, df=4, χ²=15763.0842)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=166, tsamples=1000000, bsamples=64, df=4, χ²=15709.8461)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=167, tsamples=1000000, bsamples=64, df=4, χ²=15667.1712)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=168, tsamples=1000000, bsamples=64, df=4, χ²=15725.3818)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=169, tsamples=1000000, bsamples=64, df=4, χ²=15707.9179)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=170, tsamples=1000000, bsamples=64, df=4, χ²=15750.4056)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=171, tsamples=1000000, bsamples=64, df=4, χ²=15636.1403)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=172, tsamples=1000000, bsamples=64, df=4, χ²=15731.7886)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=173, tsamples=1000000, bsamples=64, df=4, χ²=15771.7910)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=174, tsamples=1000000, bsamples=64, df=4, χ²=15749.6621)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=175, tsamples=1000000, bsamples=64, df=4, χ²=15754.2744)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=176, tsamples=1000000, bsamples=64, df=4, χ²=15645.8963)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=177, tsamples=1000000, bsamples=64, df=4, χ²=15687.5804)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=178, tsamples=1000000, bsamples=64, df=4, χ²=15807.9225)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=179, tsamples=1000000, bsamples=64, df=4, χ²=15756.2000)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=180, tsamples=1000000, bsamples=64, df=4, χ²=15687.3590)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=181, tsamples=1000000, bsamples=64, df=4, χ²=15687.9428)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=182, tsamples=1000000, bsamples=64, df=4, χ²=15696.4988)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=183, tsamples=1000000, bsamples=64, df=4, χ²=15895.6961)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=184, tsamples=1000000, bsamples=64, df=4, χ²=15768.8360)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=185, tsamples=1000000, bsamples=64, df=4, χ²=15736.3658)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=186, tsamples=1000000, bsamples=64, df=4, χ²=15762.9032)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=187, tsamples=1000000, bsamples=64, df=4, χ²=15758.8198)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=188, tsamples=1000000, bsamples=64, df=4, χ²=15695.3936)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=189, tsamples=1000000, bsamples=64, df=4, χ²=15709.3668)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=190, tsamples=1000000, bsamples=64, df=4, χ²=15816.7086)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=191, tsamples=1000000, bsamples=64, df=4, χ²=15717.5416)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=192, tsamples=1000000, bsamples=64, df=4, χ²=15716.8620)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=193, tsamples=1000000, bsamples=64, df=4, χ²=15693.2891)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=194, tsamples=1000000, bsamples=64, df=4, χ²=15787.0470)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=195, tsamples=1000000, bsamples=64, df=4, χ²=15710.7736)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=196, tsamples=1000000, bsamples=64, df=4, χ²=15673.0697)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=197, tsamples=1000000, bsamples=64, df=4, χ²=15745.9611)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=198, tsamples=1000000, bsamples=64, df=4, χ²=15737.9852)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=199, tsamples=1000000, bsamples=64, df=4, χ²=15712.2107)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=200, tsamples=1000000, bsamples=64, df=4, χ²=15833.0087)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=201, tsamples=1000000, bsamples=64, df=4, χ²=15639.3681)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=202, tsamples=1000000, bsamples=64, df=4, χ²=15676.6187)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=203, tsamples=1000000, bsamples=64, df=4, χ²=15649.2141)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=204, tsamples=1000000, bsamples=64, df=4, χ²=15791.6352)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=205, tsamples=1000000, bsamples=64, df=4, χ²=15725.3504)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=206, tsamples=1000000, bsamples=64, df=4, χ²=15908.0346)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=207, tsamples=1000000, bsamples=64, df=4, χ²=15703.1564)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=208, tsamples=1000000, bsamples=64, df=4, χ²=15727.5032)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=209, tsamples=1000000, bsamples=64, df=4, χ²=15662.2914)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=210, tsamples=1000000, bsamples=64, df=4, χ²=15729.9894)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=211, tsamples=1000000, bsamples=64, df=4, χ²=15761.5666)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=212, tsamples=1000000, bsamples=64, df=4, χ²=15703.8599)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=213, tsamples=1000000, bsamples=64, df=4, χ²=15718.8933)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=214, tsamples=1000000, bsamples=64, df=4, χ²=15712.4173)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=215, tsamples=1000000, bsamples=64, df=4, χ²=15678.8604)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=216, tsamples=1000000, bsamples=64, df=4, χ²=16188.9323)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=217, tsamples=1000000, bsamples=64, df=4, χ²=15648.3334)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=218, tsamples=1000000, bsamples=64, df=4, χ²=15647.6967)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=219, tsamples=1000000, bsamples=64, df=4, χ²=15709.7343)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=220, tsamples=1000000, bsamples=64, df=4, χ²=15689.8171)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=221, tsamples=1000000, bsamples=64, df=4, χ²=15635.6376)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=222, tsamples=1000000, bsamples=64, df=4, χ²=15688.2438)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=223, tsamples=1000000, bsamples=64, df=4, χ²=15797.4242)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=224, tsamples=1000000, bsamples=64, df=4, χ²=15723.1004)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=225, tsamples=1000000, bsamples=64, df=4, χ²=15696.2204)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=226, tsamples=1000000, bsamples=64, df=4, χ²=15673.2392)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=227, tsamples=1000000, bsamples=64, df=4, χ²=15862.3097)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=228, tsamples=1000000, bsamples=64, df=4, χ²=15758.4315)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=229, tsamples=1000000, bsamples=64, df=4, χ²=15704.2979)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=230, tsamples=1000000, bsamples=64, df=4, χ²=15715.5566)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=231, tsamples=1000000, bsamples=64, df=4, χ²=15714.7438)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=232, tsamples=1000000, bsamples=64, df=4, χ²=16190.6318)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=233, tsamples=1000000, bsamples=64, df=4, χ²=15743.7095)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=234, tsamples=1000000, bsamples=64, df=4, χ²=15725.7267)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=235, tsamples=1000000, bsamples=64, df=4, χ²=15721.0193)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=236, tsamples=1000000, bsamples=64, df=4, χ²=15651.3032)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=237, tsamples=1000000, bsamples=64, df=4, χ²=15741.0985)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=238, tsamples=1000000, bsamples=64, df=4, χ²=15714.4073)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=239, tsamples=1000000, bsamples=64, df=4, χ²=15686.7130)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=240, tsamples=1000000, bsamples=64, df=4, χ²=15695.3857)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=241, tsamples=1000000, bsamples=64, df=4, χ²=15745.4944)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=242, tsamples=1000000, bsamples=64, df=4, χ²=15931.2481)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=243, tsamples=1000000, bsamples=64, df=4, χ²=15712.1752)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=244, tsamples=1000000, bsamples=64, df=4, χ²=15657.8027)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=245, tsamples=1000000, bsamples=64, df=4, χ²=15707.7681)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=246, tsamples=1000000, bsamples=64, df=4, χ²=15770.3271)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=247, tsamples=1000000, bsamples=64, df=4, χ²=15822.8419)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=248, tsamples=1000000, bsamples=64, df=4, χ²=15636.1799)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=249, tsamples=1000000, bsamples=64, df=4, χ²=15749.5581)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=250, tsamples=1000000, bsamples=64, df=4, χ²=15701.1600)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=251, tsamples=1000000, bsamples=64, df=4, χ²=15728.5995)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=252, tsamples=1000000, bsamples=64, df=4, χ²=15709.5437)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=253, tsamples=1000000, bsamples=64, df=4, χ²=15715.7387)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=254, tsamples=1000000, bsamples=64, df=4, χ²=15699.5190)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=255, tsamples=1000000, bsamples=64, df=4, χ²=15690.1042)
  - `dieharder::gcd_distribution`: p = 0.000000  (pairs=100000, gtblsize=24, χ²=25534.1632)
  - `dieharder::gcd_step_counts`: p = 0.000000  (pairs=100000, χ²=1316.1876)

### LCG MINSTD (seed=1)

- `692` failures out of `714` tests:
  - `nist::frequency`: p = 0.000000  (n=16000000, S_n=-499968, s_obs=124.9920)
  - `nist::block_frequency`: p = 0.000000  (n=16000000, M=128, N=125000, χ²=136953.5000)
  - `nist::runs`: p = 0.000000  (pre-test failed: π=0.4844)
  - `nist::longest_run`: p = 0.000000  (n=16000000, M=10000, N=1600, χ²=260.2927)
  - `nist::matrix_rank`: p = 0.000000  (N=15625, F32=0, F31=9039, F≤30=6586, χ²=14206.6545)
  - `nist::spectral`: p = 0.000000  (n=16000000, N₀=7600000.0, N₁=7633289, T=6923.2735, d=76.3702)
  - `nist::overlapping_template`: p = 0.000000  (n=16000000, m=9, N=15503, ν=[7316, 3010, 1891, 1245, 781, 1260], χ²=1067.0293)
  - `nist::universal`: p = 0.000000  (n=16000000, L=10, Q=10240, K=1589760, f_n=9.1603, μ=9.1723, σ=0.000915)
  - `nist::approximate_entropy`: p = 0.000000  (n=16000000, m=10, ApEn=0.692621, χ²=16848.6205)
  - `nist::cumulative_sums_forward`: p = 0.000000  (n=16000000, z=499999)
  - `nist::cumulative_sums_backward`: p = 0.000000  (n=16000000, z=499971)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000000001, N=8, M=2000000, χ²=1420.0365)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000000011, N=8, M=2000000, χ²=708.8610)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000000101, N=8, M=2000000, χ²=701.1555)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000000111, N=8, M=2000000, χ²=239.9199)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000001001, N=8, M=2000000, χ²=736.5316)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000001011, N=8, M=2000000, χ²=384.9434)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000001101, N=8, M=2000000, χ²=304.0924)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000010001, N=8, M=2000000, χ²=770.6057)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000010011, N=8, M=2000000, χ²=281.9806)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000010101, N=8, M=2000000, χ²=277.6148)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000010111, N=8, M=2000000, χ²=72.2793)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000011001, N=8, M=2000000, χ²=320.6351)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000011011, N=8, M=2000000, χ²=47.2394)
  - `nist::non_overlapping_template`: p = 0.000121  (B=000011101, N=8, M=2000000, χ²=31.3633)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000011111, N=8, M=2000000, χ²=46.6282)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000100011, N=8, M=2000000, χ²=299.6609)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000100101, N=8, M=2000000, χ²=307.9440)
  - `nist::non_overlapping_template`: p = 0.006494  (B=000100111, N=8, M=2000000, χ²=21.2573)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000101001, N=8, M=2000000, χ²=298.0983)
  - `nist::non_overlapping_template`: p = 0.007207  (B=000101011, N=8, M=2000000, χ²=20.9778)
  - `nist::non_overlapping_template`: p = 0.001546  (B=000101101, N=8, M=2000000, χ²=25.0140)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000110011, N=8, M=2000000, χ²=60.4207)
  - `nist::non_overlapping_template`: p = 0.001989  (B=000110101, N=8, M=2000000, χ²=24.3664)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000110111, N=8, M=2000000, χ²=46.8660)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000111001, N=8, M=2000000, χ²=46.3141)
  - `nist::non_overlapping_template`: p = 0.000001  (B=000111011, N=8, M=2000000, χ²=43.0506)
  - `nist::non_overlapping_template`: p = 0.000001  (B=000111101, N=8, M=2000000, χ²=43.3837)
  - `nist::non_overlapping_template`: p = 0.000000  (B=000111111, N=8, M=2000000, χ²=317.0114)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001000011, N=8, M=2000000, χ²=289.7764)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001000101, N=8, M=2000000, χ²=232.8271)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001000111, N=8, M=2000000, χ²=48.9529)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001001011, N=8, M=2000000, χ²=51.6556)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001001101, N=8, M=2000000, χ²=67.0063)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001001111, N=8, M=2000000, χ²=46.0265)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001010011, N=8, M=2000000, χ²=46.9868)
  - `nist::non_overlapping_template`: p = 0.000791  (B=001010101, N=8, M=2000000, χ²=26.7186)
  - `nist::non_overlapping_template`: p = 0.000005  (B=001010111, N=8, M=2000000, χ²=38.9721)
  - `nist::non_overlapping_template`: p = 0.000003  (B=001011011, N=8, M=2000000, χ²=40.5619)
  - `nist::non_overlapping_template`: p = 0.000786  (B=001011101, N=8, M=2000000, χ²=26.7341)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001011111, N=8, M=2000000, χ²=268.9119)
  - `nist::non_overlapping_template`: p = 0.000019  (B=001100101, N=8, M=2000000, χ²=35.7752)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001101011, N=8, M=2000000, χ²=49.1162)
  - `nist::non_overlapping_template`: p = 0.000854  (B=001101101, N=8, M=2000000, χ²=26.5224)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001101111, N=8, M=2000000, χ²=257.9548)
  - `nist::non_overlapping_template`: p = 0.000714  (B=001110101, N=8, M=2000000, χ²=26.9753)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001110111, N=8, M=2000000, χ²=311.8011)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001111011, N=8, M=2000000, χ²=267.6753)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001111101, N=8, M=2000000, χ²=280.7683)
  - `nist::non_overlapping_template`: p = 0.000000  (B=001111111, N=8, M=2000000, χ²=805.6695)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010000011, N=8, M=2000000, χ²=345.4422)
  - `nist::non_overlapping_template`: p = 0.000283  (B=010000111, N=8, M=2000000, χ²=29.2801)
  - `nist::non_overlapping_template`: p = 0.000060  (B=010001011, N=8, M=2000000, χ²=33.0751)
  - `nist::non_overlapping_template`: p = 0.000047  (B=010001111, N=8, M=2000000, χ²=33.6384)
  - `nist::non_overlapping_template`: p = 0.000004  (B=010010011, N=8, M=2000000, χ²=39.7208)
  - `nist::non_overlapping_template`: p = 0.000475  (B=010010111, N=8, M=2000000, χ²=27.9968)
  - `nist::non_overlapping_template`: p = 0.000019  (B=010011011, N=8, M=2000000, χ²=35.8330)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010011111, N=8, M=2000000, χ²=346.2848)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010100011, N=8, M=2000000, χ²=62.8868)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010100111, N=8, M=2000000, χ²=54.4127)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010101011, N=8, M=2000000, χ²=50.0026)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010101111, N=8, M=2000000, χ²=304.1945)
  - `nist::non_overlapping_template`: p = 0.007600  (B=010110011, N=8, M=2000000, χ²=20.8345)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010110111, N=8, M=2000000, χ²=269.5718)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010111011, N=8, M=2000000, χ²=277.0181)
  - `nist::non_overlapping_template`: p = 0.000000  (B=010111111, N=8, M=2000000, χ²=770.1565)
  - `nist::non_overlapping_template`: p = 0.000034  (B=011000111, N=8, M=2000000, χ²=34.4095)
  - `nist::non_overlapping_template`: p = 0.000000  (B=011001111, N=8, M=2000000, χ²=291.7534)
  - `nist::non_overlapping_template`: p = 0.000000  (B=011010111, N=8, M=2000000, χ²=359.2223)
  - `nist::non_overlapping_template`: p = 0.000000  (B=011011111, N=8, M=2000000, χ²=788.7859)
  - `nist::non_overlapping_template`: p = 0.000000  (B=011101111, N=8, M=2000000, χ²=863.7958)
  - `nist::non_overlapping_template`: p = 0.000000  (B=011111111, N=8, M=2000000, χ²=1633.8217)
  - `nist::non_overlapping_template`: p = 0.000000  (B=100000000, N=8, M=2000000, χ²=1420.0365)
  - `nist::non_overlapping_template`: p = 0.000000  (B=100010000, N=8, M=2000000, χ²=729.9435)
  - `nist::non_overlapping_template`: p = 0.000000  (B=100100000, N=8, M=2000000, χ²=764.7667)
  - `nist::non_overlapping_template`: p = 0.000000  (B=100101000, N=8, M=2000000, χ²=296.7387)
  - `nist::non_overlapping_template`: p = 0.000000  (B=100110000, N=8, M=2000000, χ²=298.6117)
  - `nist::non_overlapping_template`: p = 0.000006  (B=100111000, N=8, M=2000000, χ²=38.5438)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101000000, N=8, M=2000000, χ²=780.1663)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101000100, N=8, M=2000000, χ²=302.0744)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101001000, N=8, M=2000000, χ²=270.2521)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101001100, N=8, M=2000000, χ²=51.1824)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101010000, N=8, M=2000000, χ²=316.7301)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101010100, N=8, M=2000000, χ²=45.6844)
  - `nist::non_overlapping_template`: p = 0.002460  (B=101011000, N=8, M=2000000, χ²=23.8159)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101011100, N=8, M=2000000, χ²=49.8470)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101100000, N=8, M=2000000, χ²=273.5614)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101100100, N=8, M=2000000, χ²=49.3458)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101101000, N=8, M=2000000, χ²=44.8899)
  - `nist::non_overlapping_template`: p = 0.002317  (B=101101100, N=8, M=2000000, χ²=23.9719)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101110000, N=8, M=2000000, χ²=46.8533)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101110100, N=8, M=2000000, χ²=47.9187)
  - `nist::non_overlapping_template`: p = 0.000006  (B=101111000, N=8, M=2000000, χ²=38.4361)
  - `nist::non_overlapping_template`: p = 0.000000  (B=101111100, N=8, M=2000000, χ²=248.8447)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110000000, N=8, M=2000000, χ²=657.0292)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110000010, N=8, M=2000000, χ²=352.3625)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110000100, N=8, M=2000000, χ²=298.6881)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110001000, N=8, M=2000000, χ²=238.2081)
  - `nist::non_overlapping_template`: p = 0.002272  (B=110001010, N=8, M=2000000, χ²=24.0225)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110010000, N=8, M=2000000, χ²=300.4322)
  - `nist::non_overlapping_template`: p = 0.000026  (B=110010010, N=8, M=2000000, χ²=35.0488)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110010100, N=8, M=2000000, χ²=61.1532)
  - `nist::non_overlapping_template`: p = 0.000458  (B=110011000, N=8, M=2000000, χ²=28.0864)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110100000, N=8, M=2000000, χ²=292.0122)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110100010, N=8, M=2000000, χ²=59.5429)
  - `nist::non_overlapping_template`: p = 0.000843  (B=110100100, N=8, M=2000000, χ²=26.5557)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110101000, N=8, M=2000000, χ²=59.6059)
  - `nist::non_overlapping_template`: p = 0.003718  (B=110101010, N=8, M=2000000, χ²=22.7373)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110101100, N=8, M=2000000, χ²=61.5376)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110110000, N=8, M=2000000, χ²=47.5929)
  - `nist::non_overlapping_template`: p = 0.000187  (B=110110010, N=8, M=2000000, χ²=30.3034)
  - `nist::non_overlapping_template`: p = 0.000010  (B=110110100, N=8, M=2000000, χ²=37.3543)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110111000, N=8, M=2000000, χ²=47.2348)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110111010, N=8, M=2000000, χ²=222.0473)
  - `nist::non_overlapping_template`: p = 0.000000  (B=110111100, N=8, M=2000000, χ²=266.7188)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111000000, N=8, M=2000000, χ²=243.7238)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111000010, N=8, M=2000000, χ²=55.6886)
  - `nist::non_overlapping_template`: p = 0.000040  (B=111000100, N=8, M=2000000, χ²=34.0558)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111000110, N=8, M=2000000, χ²=46.7018)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111001000, N=8, M=2000000, χ²=61.0239)
  - `nist::non_overlapping_template`: p = 0.001851  (B=111001010, N=8, M=2000000, χ²=24.5523)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111001100, N=8, M=2000000, χ²=55.0316)
  - `nist::non_overlapping_template`: p = 0.000715  (B=111010000, N=8, M=2000000, χ²=26.9710)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111010010, N=8, M=2000000, χ²=54.9982)
  - `nist::non_overlapping_template`: p = 0.000003  (B=111010100, N=8, M=2000000, χ²=39.9090)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111010110, N=8, M=2000000, χ²=293.6455)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111011000, N=8, M=2000000, χ²=47.7857)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111011010, N=8, M=2000000, χ²=283.1607)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111011100, N=8, M=2000000, χ²=283.6445)
  - `nist::non_overlapping_template`: p = 0.000052  (B=111100000, N=8, M=2000000, χ²=33.3899)
  - `nist::non_overlapping_template`: p = 0.000001  (B=111100010, N=8, M=2000000, χ²=43.9280)
  - `nist::non_overlapping_template`: p = 0.000315  (B=111100100, N=8, M=2000000, χ²=29.0180)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111100110, N=8, M=2000000, χ²=256.8252)
  - `nist::non_overlapping_template`: p = 0.000103  (B=111101000, N=8, M=2000000, χ²=31.7574)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111101010, N=8, M=2000000, χ²=279.8199)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111101100, N=8, M=2000000, χ²=284.6658)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111101110, N=8, M=2000000, χ²=796.2538)
  - `nist::non_overlapping_template`: p = 0.000015  (B=111110000, N=8, M=2000000, χ²=36.3548)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111110010, N=8, M=2000000, χ²=271.2297)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111110100, N=8, M=2000000, χ²=313.7289)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111110110, N=8, M=2000000, χ²=877.6949)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111111000, N=8, M=2000000, χ²=309.8090)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111111010, N=8, M=2000000, χ²=776.7680)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111111100, N=8, M=2000000, χ²=817.8248)
  - `nist::non_overlapping_template`: p = 0.000000  (B=111111110, N=8, M=2000000, χ²=1633.8217)
  - `nist::serial_delta2`: p = 0.000000  (n=16000000, m=3, ∇ψ²=15630.6676)
  - `maurer::universal_l06`: p = 0.000000  (n=16000000, L=6, Q=640, K=2666026, f_n=5.2133, μ=5.2177, σ=0.000597)
  - `maurer::universal_l07`: p = 0.000065  (n=16000000, L=7, Q=1280, K=2284434, f_n=6.1935, μ=6.1963, σ=0.000686)
  - `maurer::universal_l08`: p = 0.000000  (n=16000000, L=8, Q=2560, K=1997440, f_n=7.1397, μ=7.1837, σ=0.000767)
  - `maurer::universal_l09`: p = 0.000000  (n=16000000, L=9, Q=5120, K=1772657, f_n=8.1704, μ=8.1764, σ=0.000842)
  - `maurer::universal_l10`: p = 0.000000  (n=16000000, L=10, Q=10240, K=1589760, f_n=9.1603, μ=9.1723, σ=0.000915)
  - `maurer::universal_l11`: p = 0.000000  (n=16000000, L=11, Q=20480, K=1434065, f_n=10.1625, μ=10.1700, σ=0.000988)
  - `maurer::universal_l12`: p = 0.000000  (n=16000000, L=12, Q=40960, K=1292373, f_n=11.1347, μ=11.1688, σ=0.001067)
  - `maurer::universal_l13`: p = 0.000000  (n=16000000, L=13, Q=81920, K=1148849, f_n=12.1600, μ=12.1681, σ=0.001161)
  - `maurer::universal_l14`: p = 0.000000  (n=16000000, L=14, Q=163840, K=979017, f_n=13.1461, μ=13.1677, σ=0.001292)
  - `maurer::universal_l15`: p = 0.000000  (n=16000000, L=15, Q=327680, K=738986, f_n=14.1559, μ=14.1675, σ=0.001535)
  - `maurer::universal_l16`: p = 0.000000  (n=16000000, L=16, Q=655360, K=344640, f_n=14.9796, μ=15.1674, σ=0.002360)
  - `diehard::binary_rank_32x32`: p = 0.000000  (32×32, N=40000, χ²=41190.0638)
  - `diehard::bitstream`: p = 0.000000  (window=20-bit, stream=2^21, repeats=20)
  - `diehard::dna`: p = 0.000000  (missing=160546, z=55.2509)
  - `diehard::count_ones_stream`: p = 0.000000  (n=256000, Q5=12878.35, Q4=8136.79, Q5-Q4=4741.56, Z=31.7004)
  - `diehard::parking_lot`: p = 0.000000  (attempts=12000, mean=3523, σ=21.9, repeats=10)
  - `diehard::minimum_distance_2d`: p = 0.000000  (n=8000, side=10000, repeats=100 [BUGGY FORMULA — see diehard_2dsphere.c; use minimum_distance_nd(d=2) instead])
  - `diehard::spheres_3d`: p = 0.000000  (n=4000, cube=1000, repeats=20)
  - `diehard::squeeze`: p = 0.000000  (trials=100000, cells=43, df=37, χ²=1876804.7032)
  - `dieharder::minimum_distance_nd`: p = 0.000000  (d=5, n=8000, repeats=100)
  - `dieharder::lagged_sums`: p = 0.000000  (lag=1, tsamples=8000000, sum=1999889.9720, z=-2449.6245)
  - `dieharder::lagged_sums`: p = 0.000000  (lag=100, tsamples=158415, sum=39484.7631, z=-345.7257)
  - `dieharder::ks_uniform`: p = 0.000000  (tsamples=16000000)
  - `dieharder::byte_distribution`: p = 0.000000  (tsamples=5333333, streams=9, expected/cell=20833.3, χ²=16002364.6208)
  - `dieharder::dct`: p = 0.000000  (ntuple=256, tsamples=5000, χ²=79171.7760)
  - `dieharder::bit_distribution`: p = 0.000000  (width=1, pattern=0, tsamples=8000000, bsamples=64, df=36, χ²=505094.6460)
  - `dieharder::bit_distribution`: p = 0.000000  (width=1, pattern=1, tsamples=8000000, bsamples=64, df=36, χ²=505094.6460)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=0, tsamples=4000000, bsamples=64, df=30, χ²=341472.0689)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=1, tsamples=4000000, bsamples=64, df=30, χ²=343336.7701)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=2, tsamples=4000000, bsamples=64, df=30, χ²=341847.0795)
  - `dieharder::bit_distribution`: p = 0.000000  (width=2, pattern=3, tsamples=4000000, bsamples=64, df=30, χ²=342595.0446)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=0, tsamples=2666666, bsamples=64, df=21, χ²=220598.2724)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=1, tsamples=2666666, bsamples=64, df=21, χ²=23314.5904)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=2, tsamples=2666666, bsamples=64, df=21, χ²=24003.3564)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=3, tsamples=2666666, bsamples=64, df=21, χ²=24170.5472)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=4, tsamples=2666666, bsamples=64, df=21, χ²=24288.5447)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=5, tsamples=2666666, bsamples=64, df=21, χ²=23740.9165)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=6, tsamples=2666666, bsamples=64, df=21, χ²=23386.8850)
  - `dieharder::bit_distribution`: p = 0.000000  (width=3, pattern=7, tsamples=2666666, bsamples=64, df=21, χ²=221783.0465)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=0, tsamples=2000000, bsamples=64, df=14, χ²=137183.8181)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=1, tsamples=2000000, bsamples=64, df=14, χ²=138156.4068)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=2, tsamples=2000000, bsamples=64, df=14, χ²=136191.0588)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=3, tsamples=2000000, bsamples=64, df=14, χ²=137192.0348)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=4, tsamples=2000000, bsamples=64, df=14, χ²=136151.3777)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=5, tsamples=2000000, bsamples=64, df=14, χ²=137041.2373)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=6, tsamples=2000000, bsamples=64, df=14, χ²=138337.6012)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=7, tsamples=2000000, bsamples=64, df=14, χ²=136301.7777)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=8, tsamples=2000000, bsamples=64, df=14, χ²=137716.5167)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=9, tsamples=2000000, bsamples=64, df=14, χ²=136988.7241)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=10, tsamples=2000000, bsamples=64, df=14, χ²=137033.3513)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=11, tsamples=2000000, bsamples=64, df=14, χ²=137292.1440)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=12, tsamples=2000000, bsamples=64, df=14, χ²=137273.7674)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=13, tsamples=2000000, bsamples=64, df=14, χ²=136380.4739)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=14, tsamples=2000000, bsamples=64, df=14, χ²=137383.9378)
  - `dieharder::bit_distribution`: p = 0.000000  (width=4, pattern=15, tsamples=2000000, bsamples=64, df=14, χ²=137449.4743)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=0, tsamples=1600000, bsamples=64, df=10, χ²=82519.1599)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=1, tsamples=1600000, bsamples=64, df=10, χ²=29965.5717)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=2, tsamples=1600000, bsamples=64, df=10, χ²=29225.6607)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=3, tsamples=1600000, bsamples=64, df=10, χ²=3206.0946)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=4, tsamples=1600000, bsamples=64, df=10, χ²=29703.8290)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=5, tsamples=1600000, bsamples=64, df=10, χ²=3368.0840)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=6, tsamples=1600000, bsamples=64, df=10, χ²=3066.3269)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=7, tsamples=1600000, bsamples=64, df=10, χ²=3311.1597)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=8, tsamples=1600000, bsamples=64, df=10, χ²=29334.4638)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=9, tsamples=1600000, bsamples=64, df=10, χ²=3378.5026)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=10, tsamples=1600000, bsamples=64, df=10, χ²=3191.8073)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=11, tsamples=1600000, bsamples=64, df=10, χ²=3003.9245)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=12, tsamples=1600000, bsamples=64, df=10, χ²=3306.0059)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=13, tsamples=1600000, bsamples=64, df=10, χ²=3297.0566)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=14, tsamples=1600000, bsamples=64, df=10, χ²=3292.1537)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=15, tsamples=1600000, bsamples=64, df=10, χ²=29334.6849)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=16, tsamples=1600000, bsamples=64, df=10, χ²=28594.1092)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=17, tsamples=1600000, bsamples=64, df=10, χ²=2950.7216)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=18, tsamples=1600000, bsamples=64, df=10, χ²=3073.9702)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=19, tsamples=1600000, bsamples=64, df=10, χ²=3347.0818)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=20, tsamples=1600000, bsamples=64, df=10, χ²=3210.1909)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=21, tsamples=1600000, bsamples=64, df=10, χ²=3187.5274)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=22, tsamples=1600000, bsamples=64, df=10, χ²=3091.7389)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=23, tsamples=1600000, bsamples=64, df=10, χ²=28974.7272)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=24, tsamples=1600000, bsamples=64, df=10, χ²=3237.8420)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=25, tsamples=1600000, bsamples=64, df=10, χ²=2991.9074)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=26, tsamples=1600000, bsamples=64, df=10, χ²=3222.7806)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=27, tsamples=1600000, bsamples=64, df=10, χ²=29249.1341)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=28, tsamples=1600000, bsamples=64, df=10, χ²=3406.1101)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=29, tsamples=1600000, bsamples=64, df=10, χ²=29280.6264)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=30, tsamples=1600000, bsamples=64, df=10, χ²=29298.0690)
  - `dieharder::bit_distribution`: p = 0.000000  (width=5, pattern=31, tsamples=1600000, bsamples=64, df=10, χ²=82888.8440)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=0, tsamples=1333333, bsamples=64, df=7, χ²=48353.1119)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=1, tsamples=1333333, bsamples=64, df=7, χ²=48160.3040)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=2, tsamples=1333333, bsamples=64, df=7, χ²=5374.4571)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=3, tsamples=1333333, bsamples=64, df=7, χ²=5036.0604)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=4, tsamples=1333333, bsamples=64, df=7, χ²=49119.5219)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=5, tsamples=1333333, bsamples=64, df=7, χ²=48504.2459)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=6, tsamples=1333333, bsamples=64, df=7, χ²=5286.8738)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=7, tsamples=1333333, bsamples=64, df=7, χ²=5381.6825)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=8, tsamples=1333333, bsamples=64, df=7, χ²=5237.7741)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=9, tsamples=1333333, bsamples=64, df=7, χ²=5018.3035)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=10, tsamples=1333333, bsamples=64, df=7, χ²=5391.5798)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=11, tsamples=1333333, bsamples=64, df=7, χ²=5279.4670)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=12, tsamples=1333333, bsamples=64, df=7, χ²=5393.3758)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=13, tsamples=1333333, bsamples=64, df=7, χ²=5396.5590)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=14, tsamples=1333333, bsamples=64, df=7, χ²=5387.8254)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=15, tsamples=1333333, bsamples=64, df=7, χ²=5365.0845)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=16, tsamples=1333333, bsamples=64, df=7, χ²=47947.5515)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=17, tsamples=1333333, bsamples=64, df=7, χ²=47584.6258)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=18, tsamples=1333333, bsamples=64, df=7, χ²=5340.9873)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=19, tsamples=1333333, bsamples=64, df=7, χ²=5195.3948)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=20, tsamples=1333333, bsamples=64, df=7, χ²=48519.0626)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=21, tsamples=1333333, bsamples=64, df=7, χ²=49150.8254)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=22, tsamples=1333333, bsamples=64, df=7, χ²=5368.9413)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=23, tsamples=1333333, bsamples=64, df=7, χ²=5462.6470)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=24, tsamples=1333333, bsamples=64, df=7, χ²=5394.6056)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=25, tsamples=1333333, bsamples=64, df=7, χ²=5522.2510)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=26, tsamples=1333333, bsamples=64, df=7, χ²=5174.1290)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=27, tsamples=1333333, bsamples=64, df=7, χ²=5206.7398)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=28, tsamples=1333333, bsamples=64, df=7, χ²=5141.8273)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=29, tsamples=1333333, bsamples=64, df=7, χ²=5147.7364)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=30, tsamples=1333333, bsamples=64, df=7, χ²=5306.6979)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=31, tsamples=1333333, bsamples=64, df=7, χ²=5426.1569)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=32, tsamples=1333333, bsamples=64, df=7, χ²=5342.2437)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=33, tsamples=1333333, bsamples=64, df=7, χ²=5100.6517)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=34, tsamples=1333333, bsamples=64, df=7, χ²=5214.3385)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=35, tsamples=1333333, bsamples=64, df=7, χ²=5216.8744)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=36, tsamples=1333333, bsamples=64, df=7, χ²=5467.6934)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=37, tsamples=1333333, bsamples=64, df=7, χ²=5196.3865)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=38, tsamples=1333333, bsamples=64, df=7, χ²=5284.2641)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=39, tsamples=1333333, bsamples=64, df=7, χ²=5193.3074)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=40, tsamples=1333333, bsamples=64, df=7, χ²=5651.2377)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=41, tsamples=1333333, bsamples=64, df=7, χ²=5298.0885)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=42, tsamples=1333333, bsamples=64, df=7, χ²=48503.2941)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=43, tsamples=1333333, bsamples=64, df=7, χ²=48385.6765)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=44, tsamples=1333333, bsamples=64, df=7, χ²=5130.7615)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=45, tsamples=1333333, bsamples=64, df=7, χ²=5195.8181)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=46, tsamples=1333333, bsamples=64, df=7, χ²=48767.6881)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=47, tsamples=1333333, bsamples=64, df=7, χ²=48257.3527)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=48, tsamples=1333333, bsamples=64, df=7, χ²=5434.3264)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=49, tsamples=1333333, bsamples=64, df=7, χ²=5331.1137)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=50, tsamples=1333333, bsamples=64, df=7, χ²=5174.6268)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=51, tsamples=1333333, bsamples=64, df=7, χ²=5366.4957)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=52, tsamples=1333333, bsamples=64, df=7, χ²=5330.4987)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=53, tsamples=1333333, bsamples=64, df=7, χ²=5390.6583)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=54, tsamples=1333333, bsamples=64, df=7, χ²=4924.2556)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=55, tsamples=1333333, bsamples=64, df=7, χ²=5435.4970)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=56, tsamples=1333333, bsamples=64, df=7, χ²=5175.2416)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=57, tsamples=1333333, bsamples=64, df=7, χ²=5423.3145)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=58, tsamples=1333333, bsamples=64, df=7, χ²=48833.4636)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=59, tsamples=1333333, bsamples=64, df=7, χ²=49566.6043)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=60, tsamples=1333333, bsamples=64, df=7, χ²=5361.2724)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=61, tsamples=1333333, bsamples=64, df=7, χ²=5271.4194)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=62, tsamples=1333333, bsamples=64, df=7, χ²=48387.4746)
  - `dieharder::bit_distribution`: p = 0.000000  (width=6, pattern=63, tsamples=1333333, bsamples=64, df=7, χ²=48464.1970)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=0, tsamples=1142857, bsamples=64, df=5, χ²=27709.4842)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=1, tsamples=1142857, bsamples=64, df=5, χ²=14233.1664)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=2, tsamples=1142857, bsamples=64, df=5, χ²=14623.7700)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=3, tsamples=1142857, bsamples=64, df=5, χ²=4911.9939)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=4, tsamples=1142857, bsamples=64, df=5, χ²=14123.7117)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=5, tsamples=1142857, bsamples=64, df=5, χ²=5218.7457)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=6, tsamples=1142857, bsamples=64, df=5, χ²=5262.4404)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=7, tsamples=1142857, bsamples=64, df=5, χ²=547.0149)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=8, tsamples=1142857, bsamples=64, df=5, χ²=14162.2801)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=9, tsamples=1142857, bsamples=64, df=5, χ²=5052.3160)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=10, tsamples=1142857, bsamples=64, df=5, χ²=5260.1159)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=11, tsamples=1142857, bsamples=64, df=5, χ²=653.6002)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=12, tsamples=1142857, bsamples=64, df=5, χ²=5052.0826)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=13, tsamples=1142857, bsamples=64, df=5, χ²=538.7417)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=14, tsamples=1142857, bsamples=64, df=5, χ²=531.7827)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=15, tsamples=1142857, bsamples=64, df=5, χ²=618.7761)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=16, tsamples=1142857, bsamples=64, df=5, χ²=13769.3213)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=17, tsamples=1142857, bsamples=64, df=5, χ²=4959.1597)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=18, tsamples=1142857, bsamples=64, df=5, χ²=4996.6923)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=19, tsamples=1142857, bsamples=64, df=5, χ²=545.4589)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=20, tsamples=1142857, bsamples=64, df=5, χ²=5110.1335)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=21, tsamples=1142857, bsamples=64, df=5, χ²=541.8753)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=22, tsamples=1142857, bsamples=64, df=5, χ²=536.5515)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=23, tsamples=1142857, bsamples=64, df=5, χ²=547.7655)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=24, tsamples=1142857, bsamples=64, df=5, χ²=5025.2557)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=25, tsamples=1142857, bsamples=64, df=5, χ²=585.9685)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=26, tsamples=1142857, bsamples=64, df=5, χ²=566.4037)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=27, tsamples=1142857, bsamples=64, df=5, χ²=474.8374)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=28, tsamples=1142857, bsamples=64, df=5, χ²=519.1337)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=29, tsamples=1142857, bsamples=64, df=5, χ²=562.6749)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=30, tsamples=1142857, bsamples=64, df=5, χ²=550.3105)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=31, tsamples=1142857, bsamples=64, df=5, χ²=5194.6809)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=32, tsamples=1142857, bsamples=64, df=5, χ²=14029.3274)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=33, tsamples=1142857, bsamples=64, df=5, χ²=4893.6930)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=34, tsamples=1142857, bsamples=64, df=5, χ²=4841.3660)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=35, tsamples=1142857, bsamples=64, df=5, χ²=538.9243)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=36, tsamples=1142857, bsamples=64, df=5, χ²=5197.7272)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=37, tsamples=1142857, bsamples=64, df=5, χ²=643.6198)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=38, tsamples=1142857, bsamples=64, df=5, χ²=542.3578)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=39, tsamples=1142857, bsamples=64, df=5, χ²=556.5101)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=40, tsamples=1142857, bsamples=64, df=5, χ²=5141.7097)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=41, tsamples=1142857, bsamples=64, df=5, χ²=548.5327)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=42, tsamples=1142857, bsamples=64, df=5, χ²=628.9856)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=43, tsamples=1142857, bsamples=64, df=5, χ²=490.7695)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=44, tsamples=1142857, bsamples=64, df=5, χ²=662.9036)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=45, tsamples=1142857, bsamples=64, df=5, χ²=558.6298)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=46, tsamples=1142857, bsamples=64, df=5, χ²=560.7577)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=47, tsamples=1142857, bsamples=64, df=5, χ²=5050.0647)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=48, tsamples=1142857, bsamples=64, df=5, χ²=5164.8027)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=49, tsamples=1142857, bsamples=64, df=5, χ²=588.1460)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=50, tsamples=1142857, bsamples=64, df=5, χ²=582.8538)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=51, tsamples=1142857, bsamples=64, df=5, χ²=480.5866)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=52, tsamples=1142857, bsamples=64, df=5, χ²=577.2139)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=53, tsamples=1142857, bsamples=64, df=5, χ²=556.5844)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=54, tsamples=1142857, bsamples=64, df=5, χ²=590.1777)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=55, tsamples=1142857, bsamples=64, df=5, χ²=4970.0466)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=56, tsamples=1142857, bsamples=64, df=5, χ²=635.5205)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=57, tsamples=1142857, bsamples=64, df=5, χ²=491.8524)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=58, tsamples=1142857, bsamples=64, df=5, χ²=573.7380)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=59, tsamples=1142857, bsamples=64, df=5, χ²=5270.6365)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=60, tsamples=1142857, bsamples=64, df=5, χ²=554.8180)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=61, tsamples=1142857, bsamples=64, df=5, χ²=4924.3439)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=62, tsamples=1142857, bsamples=64, df=5, χ²=5063.9941)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=63, tsamples=1142857, bsamples=64, df=5, χ²=14283.2126)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=64, tsamples=1142857, bsamples=64, df=5, χ²=14247.2228)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=65, tsamples=1142857, bsamples=64, df=5, χ²=4910.4692)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=66, tsamples=1142857, bsamples=64, df=5, χ²=5118.5512)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=67, tsamples=1142857, bsamples=64, df=5, χ²=584.9712)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=68, tsamples=1142857, bsamples=64, df=5, χ²=4917.6735)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=69, tsamples=1142857, bsamples=64, df=5, χ²=495.7743)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=70, tsamples=1142857, bsamples=64, df=5, χ²=563.7776)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=71, tsamples=1142857, bsamples=64, df=5, χ²=574.7309)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=72, tsamples=1142857, bsamples=64, df=5, χ²=4880.7410)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=73, tsamples=1142857, bsamples=64, df=5, χ²=531.8943)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=74, tsamples=1142857, bsamples=64, df=5, χ²=649.6270)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=75, tsamples=1142857, bsamples=64, df=5, χ²=542.0526)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=76, tsamples=1142857, bsamples=64, df=5, χ²=566.7957)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=77, tsamples=1142857, bsamples=64, df=5, χ²=642.4319)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=78, tsamples=1142857, bsamples=64, df=5, χ²=662.7087)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=79, tsamples=1142857, bsamples=64, df=5, χ²=5055.1929)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=80, tsamples=1142857, bsamples=64, df=5, χ²=4884.8186)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=81, tsamples=1142857, bsamples=64, df=5, χ²=569.8645)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=82, tsamples=1142857, bsamples=64, df=5, χ²=562.5688)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=83, tsamples=1142857, bsamples=64, df=5, χ²=612.8204)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=84, tsamples=1142857, bsamples=64, df=5, χ²=601.2714)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=85, tsamples=1142857, bsamples=64, df=5, χ²=624.5231)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=86, tsamples=1142857, bsamples=64, df=5, χ²=586.5131)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=87, tsamples=1142857, bsamples=64, df=5, χ²=4997.4742)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=88, tsamples=1142857, bsamples=64, df=5, χ²=597.2314)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=89, tsamples=1142857, bsamples=64, df=5, χ²=628.5453)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=90, tsamples=1142857, bsamples=64, df=5, χ²=628.8857)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=91, tsamples=1142857, bsamples=64, df=5, χ²=4879.7718)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=92, tsamples=1142857, bsamples=64, df=5, χ²=543.9055)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=93, tsamples=1142857, bsamples=64, df=5, χ²=4955.7784)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=94, tsamples=1142857, bsamples=64, df=5, χ²=5055.4880)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=95, tsamples=1142857, bsamples=64, df=5, χ²=13961.3078)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=96, tsamples=1142857, bsamples=64, df=5, χ²=5132.1652)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=97, tsamples=1142857, bsamples=64, df=5, χ²=602.2253)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=98, tsamples=1142857, bsamples=64, df=5, χ²=636.3409)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=99, tsamples=1142857, bsamples=64, df=5, χ²=536.3707)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=100, tsamples=1142857, bsamples=64, df=5, χ²=613.1366)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=101, tsamples=1142857, bsamples=64, df=5, χ²=531.7351)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=102, tsamples=1142857, bsamples=64, df=5, χ²=595.1376)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=103, tsamples=1142857, bsamples=64, df=5, χ²=4952.5611)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=104, tsamples=1142857, bsamples=64, df=5, χ²=557.8234)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=105, tsamples=1142857, bsamples=64, df=5, χ²=577.3923)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=106, tsamples=1142857, bsamples=64, df=5, χ²=658.7466)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=107, tsamples=1142857, bsamples=64, df=5, χ²=5066.0620)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=108, tsamples=1142857, bsamples=64, df=5, χ²=584.8277)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=109, tsamples=1142857, bsamples=64, df=5, χ²=5080.1153)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=110, tsamples=1142857, bsamples=64, df=5, χ²=5204.2186)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=111, tsamples=1142857, bsamples=64, df=5, χ²=14313.6914)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=112, tsamples=1142857, bsamples=64, df=5, χ²=569.0097)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=113, tsamples=1142857, bsamples=64, df=5, χ²=514.8755)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=114, tsamples=1142857, bsamples=64, df=5, χ²=544.4900)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=115, tsamples=1142857, bsamples=64, df=5, χ²=5161.6870)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=116, tsamples=1142857, bsamples=64, df=5, χ²=573.2573)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=117, tsamples=1142857, bsamples=64, df=5, χ²=5061.9720)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=118, tsamples=1142857, bsamples=64, df=5, χ²=5113.9900)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=119, tsamples=1142857, bsamples=64, df=5, χ²=14464.8393)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=120, tsamples=1142857, bsamples=64, df=5, χ²=565.4411)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=121, tsamples=1142857, bsamples=64, df=5, χ²=5314.8710)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=122, tsamples=1142857, bsamples=64, df=5, χ²=5188.1500)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=123, tsamples=1142857, bsamples=64, df=5, χ²=13970.5785)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=124, tsamples=1142857, bsamples=64, df=5, χ²=5123.3091)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=125, tsamples=1142857, bsamples=64, df=5, χ²=14039.3176)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=126, tsamples=1142857, bsamples=64, df=5, χ²=14006.8651)
  - `dieharder::bit_distribution`: p = 0.000000  (width=7, pattern=127, tsamples=1142857, bsamples=64, df=5, χ²=28055.5044)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=0, tsamples=1000000, bsamples=64, df=4, χ²=15913.8209)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=1, tsamples=1000000, bsamples=64, df=4, χ²=15950.1142)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=2, tsamples=1000000, bsamples=64, df=4, χ²=15998.1983)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=3, tsamples=1000000, bsamples=64, df=4, χ²=16135.8619)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=4, tsamples=1000000, bsamples=64, df=4, χ²=15943.7951)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=5, tsamples=1000000, bsamples=64, df=4, χ²=16201.3426)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=6, tsamples=1000000, bsamples=64, df=4, χ²=15967.0705)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=7, tsamples=1000000, bsamples=64, df=4, χ²=15331.5527)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=8, tsamples=1000000, bsamples=64, df=4, χ²=15858.1506)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=9, tsamples=1000000, bsamples=64, df=4, χ²=15626.6626)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=10, tsamples=1000000, bsamples=64, df=4, χ²=15906.4747)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=11, tsamples=1000000, bsamples=64, df=4, χ²=15567.9969)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=12, tsamples=1000000, bsamples=64, df=4, χ²=15892.8936)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=13, tsamples=1000000, bsamples=64, df=4, χ²=15497.9639)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=14, tsamples=1000000, bsamples=64, df=4, χ²=15594.2321)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=15, tsamples=1000000, bsamples=64, df=4, χ²=15782.1542)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=16, tsamples=1000000, bsamples=64, df=4, χ²=15417.8203)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=17, tsamples=1000000, bsamples=64, df=4, χ²=16193.4778)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=18, tsamples=1000000, bsamples=64, df=4, χ²=15390.5283)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=19, tsamples=1000000, bsamples=64, df=4, χ²=15708.3614)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=20, tsamples=1000000, bsamples=64, df=4, χ²=15938.8771)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=21, tsamples=1000000, bsamples=64, df=4, χ²=15799.5298)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=22, tsamples=1000000, bsamples=64, df=4, χ²=15821.1941)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=23, tsamples=1000000, bsamples=64, df=4, χ²=15940.4743)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=24, tsamples=1000000, bsamples=64, df=4, χ²=15820.9573)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=25, tsamples=1000000, bsamples=64, df=4, χ²=15199.2163)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=26, tsamples=1000000, bsamples=64, df=4, χ²=15641.7851)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=27, tsamples=1000000, bsamples=64, df=4, χ²=15690.9790)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=28, tsamples=1000000, bsamples=64, df=4, χ²=15638.3774)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=29, tsamples=1000000, bsamples=64, df=4, χ²=15926.6408)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=30, tsamples=1000000, bsamples=64, df=4, χ²=15601.2144)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=31, tsamples=1000000, bsamples=64, df=4, χ²=15477.5514)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=32, tsamples=1000000, bsamples=64, df=4, χ²=15412.9614)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=33, tsamples=1000000, bsamples=64, df=4, χ²=15696.5099)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=34, tsamples=1000000, bsamples=64, df=4, χ²=15512.9516)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=35, tsamples=1000000, bsamples=64, df=4, χ²=15535.2670)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=36, tsamples=1000000, bsamples=64, df=4, χ²=15473.6252)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=37, tsamples=1000000, bsamples=64, df=4, χ²=15833.2741)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=38, tsamples=1000000, bsamples=64, df=4, χ²=16155.3571)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=39, tsamples=1000000, bsamples=64, df=4, χ²=15545.8316)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=40, tsamples=1000000, bsamples=64, df=4, χ²=15994.0116)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=41, tsamples=1000000, bsamples=64, df=4, χ²=15898.9303)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=42, tsamples=1000000, bsamples=64, df=4, χ²=15849.6535)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=43, tsamples=1000000, bsamples=64, df=4, χ²=15561.5765)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=44, tsamples=1000000, bsamples=64, df=4, χ²=16006.8813)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=45, tsamples=1000000, bsamples=64, df=4, χ²=15735.1513)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=46, tsamples=1000000, bsamples=64, df=4, χ²=15954.8225)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=47, tsamples=1000000, bsamples=64, df=4, χ²=15629.7580)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=48, tsamples=1000000, bsamples=64, df=4, χ²=16007.5917)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=49, tsamples=1000000, bsamples=64, df=4, χ²=16272.3942)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=50, tsamples=1000000, bsamples=64, df=4, χ²=15597.8991)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=51, tsamples=1000000, bsamples=64, df=4, χ²=16194.2700)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=52, tsamples=1000000, bsamples=64, df=4, χ²=15387.8100)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=53, tsamples=1000000, bsamples=64, df=4, χ²=15759.5161)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=54, tsamples=1000000, bsamples=64, df=4, χ²=16287.7055)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=55, tsamples=1000000, bsamples=64, df=4, χ²=15257.3220)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=56, tsamples=1000000, bsamples=64, df=4, χ²=16001.4753)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=57, tsamples=1000000, bsamples=64, df=4, χ²=15915.6116)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=58, tsamples=1000000, bsamples=64, df=4, χ²=15917.3177)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=59, tsamples=1000000, bsamples=64, df=4, χ²=15324.8013)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=60, tsamples=1000000, bsamples=64, df=4, χ²=15743.4685)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=61, tsamples=1000000, bsamples=64, df=4, χ²=16250.7253)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=62, tsamples=1000000, bsamples=64, df=4, χ²=15686.3371)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=63, tsamples=1000000, bsamples=64, df=4, χ²=16160.5816)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=64, tsamples=1000000, bsamples=64, df=4, χ²=15914.1904)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=65, tsamples=1000000, bsamples=64, df=4, χ²=16204.3448)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=66, tsamples=1000000, bsamples=64, df=4, χ²=16037.2305)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=67, tsamples=1000000, bsamples=64, df=4, χ²=15596.5305)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=68, tsamples=1000000, bsamples=64, df=4, χ²=15537.7690)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=69, tsamples=1000000, bsamples=64, df=4, χ²=15760.7862)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=70, tsamples=1000000, bsamples=64, df=4, χ²=16086.6918)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=71, tsamples=1000000, bsamples=64, df=4, χ²=15547.3199)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=72, tsamples=1000000, bsamples=64, df=4, χ²=15856.9401)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=73, tsamples=1000000, bsamples=64, df=4, χ²=15695.7180)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=74, tsamples=1000000, bsamples=64, df=4, χ²=15687.0984)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=75, tsamples=1000000, bsamples=64, df=4, χ²=16126.5747)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=76, tsamples=1000000, bsamples=64, df=4, χ²=15708.0645)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=77, tsamples=1000000, bsamples=64, df=4, χ²=15637.1412)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=78, tsamples=1000000, bsamples=64, df=4, χ²=16148.7718)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=79, tsamples=1000000, bsamples=64, df=4, χ²=15579.7846)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=80, tsamples=1000000, bsamples=64, df=4, χ²=16038.6722)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=81, tsamples=1000000, bsamples=64, df=4, χ²=15632.6515)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=82, tsamples=1000000, bsamples=64, df=4, χ²=16014.6621)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=83, tsamples=1000000, bsamples=64, df=4, χ²=15188.8298)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=84, tsamples=1000000, bsamples=64, df=4, χ²=16199.8775)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=85, tsamples=1000000, bsamples=64, df=4, χ²=15911.9871)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=86, tsamples=1000000, bsamples=64, df=4, χ²=15674.0146)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=87, tsamples=1000000, bsamples=64, df=4, χ²=15559.8093)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=88, tsamples=1000000, bsamples=64, df=4, χ²=15562.2023)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=89, tsamples=1000000, bsamples=64, df=4, χ²=16215.9226)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=90, tsamples=1000000, bsamples=64, df=4, χ²=15422.3173)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=91, tsamples=1000000, bsamples=64, df=4, χ²=15604.1104)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=92, tsamples=1000000, bsamples=64, df=4, χ²=15552.5850)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=93, tsamples=1000000, bsamples=64, df=4, χ²=16176.0269)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=94, tsamples=1000000, bsamples=64, df=4, χ²=15834.6670)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=95, tsamples=1000000, bsamples=64, df=4, χ²=15755.9676)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=96, tsamples=1000000, bsamples=64, df=4, χ²=15982.7883)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=97, tsamples=1000000, bsamples=64, df=4, χ²=16297.0701)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=98, tsamples=1000000, bsamples=64, df=4, χ²=16099.5090)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=99, tsamples=1000000, bsamples=64, df=4, χ²=15926.0434)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=100, tsamples=1000000, bsamples=64, df=4, χ²=15858.8659)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=101, tsamples=1000000, bsamples=64, df=4, χ²=15964.4140)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=102, tsamples=1000000, bsamples=64, df=4, χ²=15307.5012)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=103, tsamples=1000000, bsamples=64, df=4, χ²=16143.0002)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=104, tsamples=1000000, bsamples=64, df=4, χ²=15494.2490)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=105, tsamples=1000000, bsamples=64, df=4, χ²=15753.7616)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=106, tsamples=1000000, bsamples=64, df=4, χ²=15514.0183)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=107, tsamples=1000000, bsamples=64, df=4, χ²=15460.9415)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=108, tsamples=1000000, bsamples=64, df=4, χ²=16263.2680)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=109, tsamples=1000000, bsamples=64, df=4, χ²=16221.0628)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=110, tsamples=1000000, bsamples=64, df=4, χ²=15743.4083)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=111, tsamples=1000000, bsamples=64, df=4, χ²=16266.2955)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=112, tsamples=1000000, bsamples=64, df=4, χ²=16278.8712)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=113, tsamples=1000000, bsamples=64, df=4, χ²=16382.1550)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=114, tsamples=1000000, bsamples=64, df=4, χ²=15331.0377)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=115, tsamples=1000000, bsamples=64, df=4, χ²=15674.9830)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=116, tsamples=1000000, bsamples=64, df=4, χ²=15753.3873)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=117, tsamples=1000000, bsamples=64, df=4, χ²=16070.0702)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=118, tsamples=1000000, bsamples=64, df=4, χ²=16025.6663)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=119, tsamples=1000000, bsamples=64, df=4, χ²=15808.2810)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=120, tsamples=1000000, bsamples=64, df=4, χ²=15743.5511)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=121, tsamples=1000000, bsamples=64, df=4, χ²=16012.2722)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=122, tsamples=1000000, bsamples=64, df=4, χ²=15245.9070)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=123, tsamples=1000000, bsamples=64, df=4, χ²=15453.8244)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=124, tsamples=1000000, bsamples=64, df=4, χ²=15966.3451)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=125, tsamples=1000000, bsamples=64, df=4, χ²=15330.2355)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=126, tsamples=1000000, bsamples=64, df=4, χ²=15805.8782)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=127, tsamples=1000000, bsamples=64, df=4, χ²=15879.4927)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=128, tsamples=1000000, bsamples=64, df=4, χ²=15729.9489)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=129, tsamples=1000000, bsamples=64, df=4, χ²=15755.6174)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=130, tsamples=1000000, bsamples=64, df=4, χ²=15736.3000)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=131, tsamples=1000000, bsamples=64, df=4, χ²=16198.5674)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=132, tsamples=1000000, bsamples=64, df=4, χ²=16259.2685)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=133, tsamples=1000000, bsamples=64, df=4, χ²=16023.6601)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=134, tsamples=1000000, bsamples=64, df=4, χ²=16230.3734)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=135, tsamples=1000000, bsamples=64, df=4, χ²=15778.1252)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=136, tsamples=1000000, bsamples=64, df=4, χ²=15749.1731)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=137, tsamples=1000000, bsamples=64, df=4, χ²=15500.1210)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=138, tsamples=1000000, bsamples=64, df=4, χ²=15517.9322)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=139, tsamples=1000000, bsamples=64, df=4, χ²=15900.3098)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=140, tsamples=1000000, bsamples=64, df=4, χ²=15847.8344)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=141, tsamples=1000000, bsamples=64, df=4, χ²=15734.8495)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=142, tsamples=1000000, bsamples=64, df=4, χ²=15971.5116)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=143, tsamples=1000000, bsamples=64, df=4, χ²=15902.1434)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=144, tsamples=1000000, bsamples=64, df=4, χ²=16124.0836)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=145, tsamples=1000000, bsamples=64, df=4, χ²=15655.2017)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=146, tsamples=1000000, bsamples=64, df=4, χ²=15623.3300)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=147, tsamples=1000000, bsamples=64, df=4, χ²=15359.3448)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=148, tsamples=1000000, bsamples=64, df=4, χ²=16106.9978)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=149, tsamples=1000000, bsamples=64, df=4, χ²=15423.7605)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=150, tsamples=1000000, bsamples=64, df=4, χ²=15477.1934)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=151, tsamples=1000000, bsamples=64, df=4, χ²=15786.5033)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=152, tsamples=1000000, bsamples=64, df=4, χ²=15586.0156)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=153, tsamples=1000000, bsamples=64, df=4, χ²=16059.3099)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=154, tsamples=1000000, bsamples=64, df=4, χ²=16496.3198)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=155, tsamples=1000000, bsamples=64, df=4, χ²=15969.8402)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=156, tsamples=1000000, bsamples=64, df=4, χ²=15545.3410)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=157, tsamples=1000000, bsamples=64, df=4, χ²=15782.8554)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=158, tsamples=1000000, bsamples=64, df=4, χ²=15978.7748)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=159, tsamples=1000000, bsamples=64, df=4, χ²=15683.7354)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=160, tsamples=1000000, bsamples=64, df=4, χ²=15638.8352)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=161, tsamples=1000000, bsamples=64, df=4, χ²=15486.6826)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=162, tsamples=1000000, bsamples=64, df=4, χ²=15720.2515)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=163, tsamples=1000000, bsamples=64, df=4, χ²=15651.7424)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=164, tsamples=1000000, bsamples=64, df=4, χ²=15679.6286)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=165, tsamples=1000000, bsamples=64, df=4, χ²=15803.0522)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=166, tsamples=1000000, bsamples=64, df=4, χ²=15875.9440)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=167, tsamples=1000000, bsamples=64, df=4, χ²=15717.8173)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=168, tsamples=1000000, bsamples=64, df=4, χ²=15976.5778)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=169, tsamples=1000000, bsamples=64, df=4, χ²=15867.3640)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=170, tsamples=1000000, bsamples=64, df=4, χ²=15768.8263)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=171, tsamples=1000000, bsamples=64, df=4, χ²=16154.6163)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=172, tsamples=1000000, bsamples=64, df=4, χ²=15835.3034)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=173, tsamples=1000000, bsamples=64, df=4, χ²=15693.4371)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=174, tsamples=1000000, bsamples=64, df=4, χ²=15539.2139)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=175, tsamples=1000000, bsamples=64, df=4, χ²=15557.7297)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=176, tsamples=1000000, bsamples=64, df=4, χ²=15678.4084)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=177, tsamples=1000000, bsamples=64, df=4, χ²=15723.6772)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=178, tsamples=1000000, bsamples=64, df=4, χ²=15648.2357)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=179, tsamples=1000000, bsamples=64, df=4, χ²=15909.1195)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=180, tsamples=1000000, bsamples=64, df=4, χ²=15663.3780)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=181, tsamples=1000000, bsamples=64, df=4, χ²=15588.5267)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=182, tsamples=1000000, bsamples=64, df=4, χ²=15580.3681)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=183, tsamples=1000000, bsamples=64, df=4, χ²=15789.2029)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=184, tsamples=1000000, bsamples=64, df=4, χ²=16152.0476)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=185, tsamples=1000000, bsamples=64, df=4, χ²=15909.6924)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=186, tsamples=1000000, bsamples=64, df=4, χ²=15555.2638)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=187, tsamples=1000000, bsamples=64, df=4, χ²=15621.6389)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=188, tsamples=1000000, bsamples=64, df=4, χ²=15816.6339)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=189, tsamples=1000000, bsamples=64, df=4, χ²=16079.0856)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=190, tsamples=1000000, bsamples=64, df=4, χ²=15395.0568)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=191, tsamples=1000000, bsamples=64, df=4, χ²=15478.4749)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=192, tsamples=1000000, bsamples=64, df=4, χ²=15812.5126)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=193, tsamples=1000000, bsamples=64, df=4, χ²=15511.0944)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=194, tsamples=1000000, bsamples=64, df=4, χ²=16097.7862)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=195, tsamples=1000000, bsamples=64, df=4, χ²=16138.1161)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=196, tsamples=1000000, bsamples=64, df=4, χ²=15919.6541)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=197, tsamples=1000000, bsamples=64, df=4, χ²=15871.6944)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=198, tsamples=1000000, bsamples=64, df=4, χ²=15647.0797)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=199, tsamples=1000000, bsamples=64, df=4, χ²=15938.0688)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=200, tsamples=1000000, bsamples=64, df=4, χ²=15928.5841)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=201, tsamples=1000000, bsamples=64, df=4, χ²=15977.5854)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=202, tsamples=1000000, bsamples=64, df=4, χ²=15757.6330)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=203, tsamples=1000000, bsamples=64, df=4, χ²=15796.5698)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=204, tsamples=1000000, bsamples=64, df=4, χ²=15899.5642)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=205, tsamples=1000000, bsamples=64, df=4, χ²=15841.1270)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=206, tsamples=1000000, bsamples=64, df=4, χ²=15852.3000)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=207, tsamples=1000000, bsamples=64, df=4, χ²=15788.6439)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=208, tsamples=1000000, bsamples=64, df=4, χ²=16249.9265)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=209, tsamples=1000000, bsamples=64, df=4, χ²=15870.4642)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=210, tsamples=1000000, bsamples=64, df=4, χ²=15786.4383)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=211, tsamples=1000000, bsamples=64, df=4, χ²=15756.9542)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=212, tsamples=1000000, bsamples=64, df=4, χ²=15978.7124)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=213, tsamples=1000000, bsamples=64, df=4, χ²=15829.8909)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=214, tsamples=1000000, bsamples=64, df=4, χ²=15636.1056)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=215, tsamples=1000000, bsamples=64, df=4, χ²=15583.4877)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=216, tsamples=1000000, bsamples=64, df=4, χ²=15267.2316)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=217, tsamples=1000000, bsamples=64, df=4, χ²=15465.6148)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=218, tsamples=1000000, bsamples=64, df=4, χ²=15734.2525)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=219, tsamples=1000000, bsamples=64, df=4, χ²=15648.7610)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=220, tsamples=1000000, bsamples=64, df=4, χ²=15702.3863)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=221, tsamples=1000000, bsamples=64, df=4, χ²=15931.3329)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=222, tsamples=1000000, bsamples=64, df=4, χ²=16075.8712)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=223, tsamples=1000000, bsamples=64, df=4, χ²=15730.1864)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=224, tsamples=1000000, bsamples=64, df=4, χ²=15760.9344)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=225, tsamples=1000000, bsamples=64, df=4, χ²=15518.0270)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=226, tsamples=1000000, bsamples=64, df=4, χ²=16085.9800)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=227, tsamples=1000000, bsamples=64, df=4, χ²=15882.8031)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=228, tsamples=1000000, bsamples=64, df=4, χ²=15908.1073)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=229, tsamples=1000000, bsamples=64, df=4, χ²=15774.1860)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=230, tsamples=1000000, bsamples=64, df=4, χ²=15620.7690)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=231, tsamples=1000000, bsamples=64, df=4, χ²=15735.3308)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=232, tsamples=1000000, bsamples=64, df=4, χ²=16064.7634)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=233, tsamples=1000000, bsamples=64, df=4, χ²=15823.6253)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=234, tsamples=1000000, bsamples=64, df=4, χ²=15876.4049)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=235, tsamples=1000000, bsamples=64, df=4, χ²=15680.4451)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=236, tsamples=1000000, bsamples=64, df=4, χ²=16097.3277)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=237, tsamples=1000000, bsamples=64, df=4, χ²=15672.8836)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=238, tsamples=1000000, bsamples=64, df=4, χ²=16172.4822)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=239, tsamples=1000000, bsamples=64, df=4, χ²=15886.2811)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=240, tsamples=1000000, bsamples=64, df=4, χ²=15821.2310)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=241, tsamples=1000000, bsamples=64, df=4, χ²=15844.8603)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=242, tsamples=1000000, bsamples=64, df=4, χ²=15815.6877)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=243, tsamples=1000000, bsamples=64, df=4, χ²=16114.3497)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=244, tsamples=1000000, bsamples=64, df=4, χ²=16327.5589)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=245, tsamples=1000000, bsamples=64, df=4, χ²=15823.4536)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=246, tsamples=1000000, bsamples=64, df=4, χ²=15787.4604)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=247, tsamples=1000000, bsamples=64, df=4, χ²=15866.2715)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=248, tsamples=1000000, bsamples=64, df=4, χ²=16181.3653)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=249, tsamples=1000000, bsamples=64, df=4, χ²=16026.1258)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=250, tsamples=1000000, bsamples=64, df=4, χ²=15736.6727)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=251, tsamples=1000000, bsamples=64, df=4, χ²=15887.6011)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=252, tsamples=1000000, bsamples=64, df=4, χ²=15878.6320)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=253, tsamples=1000000, bsamples=64, df=4, χ²=15855.8975)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=254, tsamples=1000000, bsamples=64, df=4, χ²=15678.3051)
  - `dieharder::bit_distribution`: p = 0.000000  (width=8, pattern=255, tsamples=1000000, bsamples=64, df=4, χ²=15792.0635)
  - `dieharder::gcd_step_counts`: p = 0.000000  (pairs=100000, χ²=2844.7637)

### BBS (p=2³¹−1, q=4294967291)

- `7` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.000161  (B=000010111, N=8, M=2000000, χ²=30.6653)
  - `dieharder::bit_distribution`: p = 0.007852  (width=7, pattern=10, tsamples=1142857, bsamples=64, df=5, χ²=15.6700)
  - `dieharder::bit_distribution`: p = 0.005451  (width=7, pattern=11, tsamples=1142857, bsamples=64, df=5, χ²=16.5438)
  - `dieharder::bit_distribution`: p = 0.008154  (width=7, pattern=114, tsamples=1142857, bsamples=64, df=5, χ²=15.5793)
  - `dieharder::bit_distribution`: p = 0.001682  (width=8, pattern=32, tsamples=1000000, bsamples=64, df=4, χ²=17.3100)
  - `dieharder::bit_distribution`: p = 0.009844  (width=8, pattern=64, tsamples=1000000, bsamples=64, df=4, χ²=13.3128)
  - `dieharder::bit_distribution`: p = 0.007117  (width=8, pattern=209, tsamples=1000000, bsamples=64, df=4, χ²=14.0566)

### Blum-Micali (p=2³¹−1, g=7)

- `4` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.004964  (B=100110000, N=8, M=2000000, χ²=21.9741)
  - `dieharder::bit_distribution`: p = 0.009187  (width=6, pattern=18, tsamples=1333333, bsamples=64, df=7, χ²=18.6981)
  - `dieharder::bit_distribution`: p = 0.004965  (width=7, pattern=122, tsamples=1142857, bsamples=64, df=5, χ²=16.7661)
  - `dieharder::bit_distribution`: p = 0.001438  (width=8, pattern=69, tsamples=1000000, bsamples=64, df=4, χ²=17.6596)

### AES-128-CTR (NIST key)

- `5` failures out of `738` tests:
  - `nist::overlapping_template`: p = 0.007942  (n=16000000, m=9, N=15503, ν=[5581, 2795, 2201, 1650, 1029, 2247], χ²=15.6427)
  - `nist::non_overlapping_template`: p = 0.007730  (B=001001111, N=8, M=2000000, χ²=20.7888)
  - `dieharder::bit_distribution`: p = 0.004771  (width=8, pattern=123, tsamples=1000000, bsamples=64, df=4, χ²=14.9665)
  - `dieharder::bit_distribution`: p = 0.005510  (width=8, pattern=211, tsamples=1000000, bsamples=64, df=4, χ²=14.6397)
  - `dieharder::bit_distribution`: p = 0.000724  (width=8, pattern=235, tsamples=1000000, bsamples=64, df=4, χ²=19.1816)

### SpongeBob (SHA3-512 chain, OsRng seed)

- `11` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.008069  (B=000010001, N=8, M=2000000, χ²=20.6729)
  - `nist::non_overlapping_template`: p = 0.000105  (B=000100011, N=8, M=2000000, χ²=31.7155)
  - `nist::non_overlapping_template`: p = 0.007154  (B=000101011, N=8, M=2000000, χ²=20.9975)
  - `nist::non_overlapping_template`: p = 0.006093  (B=101010000, N=8, M=2000000, χ²=21.4280)
  - `diehard::binary_rank_31x31`: p = 0.003067  (31×31, N=40000, χ²=13.8845)
  - `dieharder::bit_distribution`: p = 0.000866  (width=3, pattern=7, tsamples=2666666, bsamples=64, df=21, χ²=47.2595)
  - `dieharder::bit_distribution`: p = 0.002888  (width=4, pattern=1, tsamples=2000000, bsamples=64, df=14, χ²=32.9931)
  - `dieharder::bit_distribution`: p = 0.009661  (width=5, pattern=5, tsamples=1600000, bsamples=64, df=10, χ²=23.3092)
  - `dieharder::bit_distribution`: p = 0.008021  (width=7, pattern=114, tsamples=1142857, bsamples=64, df=5, χ²=15.6187)
  - `dieharder::bit_distribution`: p = 0.000287  (width=8, pattern=85, tsamples=1000000, bsamples=64, df=4, χ²=21.2116)
  - `dieharder::bit_distribution`: p = 0.002204  (width=8, pattern=221, tsamples=1000000, bsamples=64, df=4, χ²=16.7063)

### Squidward (SHA-256 chain, OsRng seed)

- `7` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.009023  (B=100100000, N=8, M=2000000, χ²=20.3701)
  - `nist::non_overlapping_template`: p = 0.006036  (B=111111100, N=8, M=2000000, χ²=21.4533)
  - `dieharder::bit_distribution`: p = 0.005415  (width=7, pattern=30, tsamples=1142857, bsamples=64, df=5, χ²=16.5597)
  - `dieharder::bit_distribution`: p = 0.004934  (width=7, pattern=60, tsamples=1142857, bsamples=64, df=5, χ²=16.7814)
  - `dieharder::bit_distribution`: p = 0.007852  (width=8, pattern=32, tsamples=1000000, bsamples=64, df=4, χ²=13.8318)
  - `dieharder::bit_distribution`: p = 0.004028  (width=8, pattern=197, tsamples=1000000, bsamples=64, df=4, χ²=15.3499)
  - `dieharder::bit_distribution`: p = 0.002553  (width=8, pattern=201, tsamples=1000000, bsamples=64, df=4, χ²=16.3767)

### PCG32 (OsRng seed)

- `10` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.005114  (B=000111001, N=8, M=2000000, χ²=21.8951)
  - `nist::non_overlapping_template`: p = 0.001519  (B=010011011, N=8, M=2000000, χ²=25.0594)
  - `nist::non_overlapping_template`: p = 0.000010  (B=011011111, N=8, M=2000000, χ²=37.4318)
  - `dieharder::byte_distribution`: p = 0.005384  (tsamples=5333333, streams=9, expected/cell=20833.3, χ²=2471.4373)
  - `dieharder::bit_distribution`: p = 0.003157  (width=7, pattern=105, tsamples=1142857, bsamples=64, df=5, χ²=17.8372)
  - `dieharder::bit_distribution`: p = 0.004051  (width=8, pattern=95, tsamples=1000000, bsamples=64, df=4, χ²=15.3369)
  - `dieharder::bit_distribution`: p = 0.002548  (width=8, pattern=151, tsamples=1000000, bsamples=64, df=4, χ²=16.3811)
  - `dieharder::bit_distribution`: p = 0.009921  (width=8, pattern=163, tsamples=1000000, bsamples=64, df=4, χ²=13.2950)
  - `dieharder::bit_distribution`: p = 0.005514  (width=8, pattern=197, tsamples=1000000, bsamples=64, df=4, χ²=14.6379)
  - `dieharder::bit_distribution`: p = 0.003610  (width=8, pattern=246, tsamples=1000000, bsamples=64, df=4, χ²=15.5975)

### PCG64 (OsRng seed)

- `9` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.005613  (B=000000101, N=8, M=2000000, χ²=21.6472)
  - `maurer::universal_l16`: p = 0.005797  (n=16000000, L=16, Q=655360, K=344640, f_n=15.1609, μ=15.1674, σ=0.002360)
  - `dieharder::bit_distribution`: p = 0.009041  (width=1, pattern=0, tsamples=8000000, bsamples=64, df=36, χ²=59.0594)
  - `dieharder::bit_distribution`: p = 0.009041  (width=1, pattern=1, tsamples=8000000, bsamples=64, df=36, χ²=59.0594)
  - `dieharder::bit_distribution`: p = 0.007115  (width=5, pattern=10, tsamples=1600000, bsamples=64, df=10, χ²=24.1886)
  - `dieharder::bit_distribution`: p = 0.000638  (width=7, pattern=14, tsamples=1142857, bsamples=64, df=5, χ²=21.5466)
  - `dieharder::bit_distribution`: p = 0.001251  (width=8, pattern=29, tsamples=1000000, bsamples=64, df=4, χ²=17.9691)
  - `dieharder::bit_distribution`: p = 0.002123  (width=8, pattern=31, tsamples=1000000, bsamples=64, df=4, χ²=16.7907)
  - `dieharder::bit_distribution`: p = 0.005890  (width=8, pattern=245, tsamples=1000000, bsamples=64, df=4, χ²=14.4881)

### Xoshiro256** (OsRng seed)

- `12` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.008791  (B=000011011, N=8, M=2000000, χ²=20.4407)
  - `nist::non_overlapping_template`: p = 0.003802  (B=101010100, N=8, M=2000000, χ²=22.6788)
  - `nist::non_overlapping_template`: p = 0.002845  (B=111101110, N=8, M=2000000, χ²=23.4378)
  - `maurer::universal_l13`: p = 0.003833  (n=16000000, L=13, Q=81920, K=1148849, f_n=12.1647, μ=12.1681, σ=0.001161)
  - `maurer::universal_l15`: p = 0.005133  (n=16000000, L=15, Q=327680, K=738986, f_n=14.1718, μ=14.1675, σ=0.001535)
  - `dieharder::byte_distribution`: p = 0.001246  (tsamples=5333333, streams=9, expected/cell=20833.3, χ²=2505.3394)
  - `dieharder::bit_distribution`: p = 0.005948  (width=4, pattern=1, tsamples=2000000, bsamples=64, df=14, χ²=30.7811)
  - `dieharder::bit_distribution`: p = 0.005697  (width=6, pattern=2, tsamples=1333333, bsamples=64, df=7, χ²=19.9419)
  - `dieharder::bit_distribution`: p = 0.007777  (width=6, pattern=29, tsamples=1333333, bsamples=64, df=7, χ²=19.1344)
  - `dieharder::bit_distribution`: p = 0.001711  (width=7, pattern=17, tsamples=1142857, bsamples=64, df=5, χ²=19.2706)
  - `dieharder::bit_distribution`: p = 0.005421  (width=8, pattern=83, tsamples=1000000, bsamples=64, df=4, χ²=14.6765)
  - `dieharder::bit_distribution`: p = 0.004659  (width=8, pattern=110, tsamples=1000000, bsamples=64, df=4, χ²=15.0204)

### Xoroshiro128** (OsRng seed)

- `7` failures out of `738` tests:
  - `diehard::oqso`: p = 0.003278  (missing=142776, z=2.9404)
  - `dieharder::bit_distribution`: p = 0.006685  (width=5, pattern=19, tsamples=1600000, bsamples=64, df=10, χ²=24.3663)
  - `dieharder::bit_distribution`: p = 0.001948  (width=7, pattern=43, tsamples=1142857, bsamples=64, df=5, χ²=18.9688)
  - `dieharder::bit_distribution`: p = 0.008076  (width=7, pattern=71, tsamples=1142857, bsamples=64, df=5, χ²=15.6022)
  - `dieharder::bit_distribution`: p = 0.001167  (width=8, pattern=60, tsamples=1000000, bsamples=64, df=4, χ²=18.1236)
  - `dieharder::bit_distribution`: p = 0.001042  (width=8, pattern=147, tsamples=1000000, bsamples=64, df=4, χ²=18.3762)
  - `dieharder::bit_distribution`: p = 0.000321  (width=8, pattern=168, tsamples=1000000, bsamples=64, df=4, χ²=20.9682)

### WyRand (OsRng seed)

- `1` failure out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.002594  (B=110101010, N=8, M=2000000, χ²=23.6783)

### SFC64 (OsRng seed)

- `4` failures out of `714` tests:
  - `nist::non_overlapping_template`: p = 0.003297  (B=000001001, N=8, M=2000000, χ²=23.0525)
  - `nist::non_overlapping_template`: p = 0.002998  (B=000010011, N=8, M=2000000, χ²=23.3013)
  - `dieharder::bit_distribution`: p = 0.006759  (width=8, pattern=134, tsamples=1000000, bsamples=64, df=4, χ²=14.1743)
  - `dieharder::bit_distribution`: p = 0.009010  (width=8, pattern=145, tsamples=1000000, bsamples=64, df=4, χ²=13.5164)

### JSF64 (OsRng seed)

- `11` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.001467  (B=001111111, N=8, M=2000000, χ²=25.1480)
  - `diehard::opso`: p = 0.009372  (missing=142664, z=2.5982)
  - `dieharder::bit_distribution`: p = 0.001337  (width=6, pattern=17, tsamples=1333333, bsamples=64, df=7, χ²=23.6047)
  - `dieharder::bit_distribution`: p = 0.009284  (width=6, pattern=43, tsamples=1333333, bsamples=64, df=7, χ²=18.6707)
  - `dieharder::bit_distribution`: p = 0.001486  (width=7, pattern=91, tsamples=1142857, bsamples=64, df=5, χ²=19.5984)
  - `dieharder::bit_distribution`: p = 0.004605  (width=8, pattern=11, tsamples=1000000, bsamples=64, df=4, χ²=15.0471)
  - `dieharder::bit_distribution`: p = 0.002149  (width=8, pattern=60, tsamples=1000000, bsamples=64, df=4, χ²=16.7634)
  - `dieharder::bit_distribution`: p = 0.000357  (width=8, pattern=110, tsamples=1000000, bsamples=64, df=4, χ²=20.7395)
  - `dieharder::bit_distribution`: p = 0.000875  (width=8, pattern=144, tsamples=1000000, bsamples=64, df=4, χ²=18.7634)
  - `dieharder::bit_distribution`: p = 0.003020  (width=8, pattern=174, tsamples=1000000, bsamples=64, df=4, χ²=15.9991)
  - `dieharder::bit_distribution`: p = 0.003426  (width=8, pattern=222, tsamples=1000000, bsamples=64, df=4, χ²=15.7152)

### ChaCha20 CSPRNG (OsRng key)

- `8` failures out of `738` tests:
  - `nist::non_overlapping_template`: p = 0.009124  (B=010000011, N=8, M=2000000, χ²=20.3399)
  - `nist::non_overlapping_template`: p = 0.009726  (B=100010000, N=8, M=2000000, χ²=20.1659)
  - `nist::non_overlapping_template`: p = 0.000809  (B=101010100, N=8, M=2000000, χ²=26.6592)
  - `nist::non_overlapping_template`: p = 0.005626  (B=110101010, N=8, M=2000000, χ²=21.6411)
  - `diehard::craps_wins`: p = 0.003487  (games=200000, wins=99239, z=2.9212)
  - `dieharder::bit_distribution`: p = 0.006419  (width=6, pattern=40, tsamples=1333333, bsamples=64, df=7, χ²=19.6330)
  - `dieharder::bit_distribution`: p = 0.008913  (width=7, pattern=119, tsamples=1142857, bsamples=64, df=5, χ²=15.3646)
  - `dieharder::bit_distribution`: p = 0.004372  (width=8, pattern=92, tsamples=1000000, bsamples=64, df=4, χ²=15.1646)

### HMAC_DRBG SHA-256 (OsRng seed)

- `10` failures out of `714` tests:
  - `nist::non_overlapping_template`: p = 0.000364  (B=011000111, N=8, M=2000000, χ²=28.6593)
  - `nist::non_overlapping_template`: p = 0.008579  (B=101001100, N=8, M=2000000, χ²=20.5071)
  - `nist::non_overlapping_template`: p = 0.007108  (B=101011000, N=8, M=2000000, χ²=21.0149)
  - `nist::non_overlapping_template`: p = 0.005716  (B=101011100, N=8, M=2000000, χ²=21.5987)
  - `maurer::universal_l08`: p = 0.009795  (n=16000000, L=8, Q=2560, K=1997440, f_n=7.1817, μ=7.1837, σ=0.000767)
  - `dieharder::bit_distribution`: p = 0.005007  (width=7, pattern=23, tsamples=1142857, bsamples=64, df=5, χ²=16.7461)
  - `dieharder::bit_distribution`: p = 0.007504  (width=8, pattern=47, tsamples=1000000, bsamples=64, df=4, χ²=13.9353)
  - `dieharder::bit_distribution`: p = 0.000822  (width=8, pattern=119, tsamples=1000000, bsamples=64, df=4, χ²=18.9012)
  - `dieharder::bit_distribution`: p = 0.009918  (width=8, pattern=218, tsamples=1000000, bsamples=64, df=4, χ²=13.2957)
  - `dieharder::bit_distribution`: p = 0.007995  (width=8, pattern=238, tsamples=1000000, bsamples=64, df=4, χ²=13.7904)

### Hash_DRBG SHA-256 (OsRng seed)

- `6` failures out of `714` tests:
  - `nist::non_overlapping_template`: p = 0.005472  (B=111101010, N=8, M=2000000, χ²=21.7149)
  - `dieharder::bit_distribution`: p = 0.006506  (width=6, pattern=55, tsamples=1333333, bsamples=64, df=7, χ²=19.5984)
  - `dieharder::bit_distribution`: p = 0.008598  (width=7, pattern=114, tsamples=1142857, bsamples=64, df=5, χ²=15.4514)
  - `dieharder::bit_distribution`: p = 0.006620  (width=8, pattern=11, tsamples=1000000, bsamples=64, df=4, χ²=14.2216)
  - `dieharder::bit_distribution`: p = 0.007003  (width=8, pattern=162, tsamples=1000000, bsamples=64, df=4, χ²=14.0935)
  - `dieharder::bit_distribution`: p = 0.004440  (width=8, pattern=230, tsamples=1000000, bsamples=64, df=4, χ²=15.1295)

### cryptography::CtrDrbgAes256 (seed=00..2f)

- `8` failures out of `714` tests:
  - `nist::non_overlapping_template`: p = 0.004786  (B=101111000, N=8, M=2000000, χ²=22.0709)
  - `nist::non_overlapping_template`: p = 0.003928  (B=110110100, N=8, M=2000000, χ²=22.5930)
  - `nist::non_overlapping_template`: p = 0.009683  (B=110111100, N=8, M=2000000, χ²=20.1779)
  - `nist::non_overlapping_template`: p = 0.007342  (B=111001100, N=8, M=2000000, χ²=20.9276)
  - `nist::non_overlapping_template`: p = 0.008163  (B=111011000, N=8, M=2000000, χ²=20.6417)
  - `dieharder::bit_distribution`: p = 0.001059  (width=6, pattern=5, tsamples=1333333, bsamples=64, df=7, χ²=24.1806)
  - `dieharder::bit_distribution`: p = 0.004132  (width=7, pattern=124, tsamples=1142857, bsamples=64, df=5, χ²=17.2024)
  - `dieharder::bit_distribution`: p = 0.000411  (width=8, pattern=104, tsamples=1000000, bsamples=64, df=4, χ²=20.4259)

### Constant (0xDEAD_DEAD)

- `711/714` failures — expected for a degenerate generator.

### Counter (0,1,2,…)

- `710/714` failures — expected for a degenerate generator.

### Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01)

- `3` failures out of `199` tests:
  - `nist::non_overlapping_template`: p = 0.002209  (B=000010001, N=8, M=2000000, χ²=24.0956)
  - `nist::non_overlapping_template`: p = 0.007094  (B=010011011, N=8, M=2000000, χ²=21.0202)
  - `maurer::universal_l15`: p = 0.009827  (n=16000000, L=15, Q=327680, K=738986, f_n=14.1635, μ=14.1675, σ=0.001535)

## Bottom Line

- Degenerate generators (Constant, Counter) and legacy PRNGs (ANSI C LCG, MINSTD, VB6 Rnd) remain annihilated — the battery continues to distinguish garbage from structure.
- Among non-trivial generators, the lowest FAIL count is **1** (`WyRand (OsRng seed)`) and the highest is **12** (`Xorshift32 (seed=1)`).
- Isolated failures in `non_overlapping_template` and `bit_distribution` are expected at α = 0.01; they are noise unless they form a family cluster.

## Auxiliary Probes

Auxiliary probes run via `tests/run_aux.sh` on `darby.local` (2026-03-15).

These probes exercise statistical properties not covered by the NIST/DIEHARD/DIEHARDER
battery.  They run with their default parameters; use the individual binaries for
filtered or resized runs.  All probes exit 0 (no crashes or panics).

```

========================================================================
bib_tests  (Knuth + NIST ApEn profile m=2..6)
========================================================================

MT19937
  [PASS] knuth::permutation                                p = 0.106832  (t=5, blocks=40000, χ²=138.5000, df=119)
  [PASS] knuth::gap                                        p = 0.966310  ([0.250,0.500) gaps=49922, r=15, χ²=6.6641, df=15)
  [PASS] knuth::runs_median                                p = 0.582269  (median=0.500112, below=100000, above=100000, runs=100124, z=0.5501)
  [INFO] approx_entropy_m02   ApEn=0.693144 (phi_m=-1.386294, phi_m1=-2.079438)
  [INFO] approx_entropy_m03   ApEn=0.693144 (phi_m=-2.079438, phi_m1=-2.772581)
  [INFO] approx_entropy_m04   ApEn=0.693141 (phi_m=-2.772581, phi_m1=-3.465723)
  [INFO] approx_entropy_m05   ApEn=0.693133 (phi_m=-3.465723, phi_m1=-4.158856)
  [INFO] approx_entropy_m06   ApEn=0.693118 (phi_m=-4.158856, phi_m1=-4.851974)

Xorshift32
  [PASS] knuth::permutation                                p = 0.622050  (t=5, blocks=40000, χ²=113.6180, df=119)
  [PASS] knuth::gap                                        p = 0.237153  ([0.250,0.500) gaps=50065, r=15, χ²=18.5028, df=15)
  [PASS] knuth::runs_median                                p = 0.190081  (median=0.499376, below=100000, above=100000, runs=100294, z=1.3103)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693145 (phi_m=-2.079439, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693138 (phi_m=-2.772583, phi_m1=-3.465722)
  [INFO] approx_entropy_m05   ApEn=0.693133 (phi_m=-3.465722, phi_m1=-4.158855)
  [INFO] approx_entropy_m06   ApEn=0.693115 (phi_m=-4.158855, phi_m1=-4.851970)

Xorshift64
  [PASS] knuth::permutation                                p = 0.606131  (t=5, blocks=40000, χ²=114.2420, df=119)
  [PASS] knuth::gap                                        p = 0.750205  ([0.250,0.500) gaps=50116, r=15, χ²=11.0337, df=15)
  [PASS] knuth::runs_median                                p = 0.134089  (median=0.499730, below=100000, above=100000, runs=99666, z=-1.4982)
  [INFO] approx_entropy_m02   ApEn=0.693147 (phi_m=-1.386294, phi_m1=-2.079441)
  [INFO] approx_entropy_m03   ApEn=0.693143 (phi_m=-2.079441, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693136 (phi_m=-2.772583, phi_m1=-3.465720)
  [INFO] approx_entropy_m05   ApEn=0.693130 (phi_m=-3.465720, phi_m1=-4.158850)
  [INFO] approx_entropy_m06   ApEn=0.693121 (phi_m=-4.158850, phi_m1=-4.851971)

BAD Unix System V rand()
  [PASS] knuth::permutation                                p = 0.410880  (t=5, blocks=40000, χ²=121.8320, df=119)
  [PASS] knuth::gap                                        p = 0.397280  ([0.250,0.500) gaps=50005, r=15, χ²=15.7732, df=15)
  [PASS] knuth::runs_median                                p = 0.576149  (median=0.499845, below=100000, above=100000, runs=99876, z=-0.5590)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079440)
  [INFO] approx_entropy_m03   ApEn=0.693145 (phi_m=-2.079440, phi_m1=-2.772584)
  [INFO] approx_entropy_m04   ApEn=0.693137 (phi_m=-2.772584, phi_m1=-3.465721)
  [INFO] approx_entropy_m05   ApEn=0.693129 (phi_m=-3.465721, phi_m1=-4.158850)
  [INFO] approx_entropy_m06   ApEn=0.693108 (phi_m=-4.158850, phi_m1=-4.851957)

BAD Unix System V mrand48()
  [PASS] knuth::permutation                                p = 0.026338  (t=5, blocks=40000, χ²=150.6800, df=119)
  [PASS] knuth::gap                                        p = 0.887223  ([0.250,0.500) gaps=49804, r=15, χ²=8.8104, df=15)
  [PASS] knuth::runs_median                                p = 0.582269  (median=0.501971, below=100000, above=100000, runs=100124, z=0.5501)
  [INFO] approx_entropy_m02   ApEn=0.693145 (phi_m=-1.386294, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693144 (phi_m=-2.079439, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693135 (phi_m=-2.772583, phi_m1=-3.465718)
  [INFO] approx_entropy_m05   ApEn=0.693130 (phi_m=-3.465718, phi_m1=-4.158848)
  [INFO] approx_entropy_m06   ApEn=0.693115 (phi_m=-4.158848, phi_m1=-4.851964)

BAD Unix BSD random()
  [PASS] knuth::permutation                                p = 0.801062  (t=5, blocks=40000, χ²=105.8060, df=119)
  [PASS] knuth::gap                                        p = 0.823636  ([0.250,0.500) gaps=50051, r=15, χ²=9.9378, df=15)
  [PASS] knuth::runs_median                                p = 0.255988  (median=0.499583, below=100000, above=100000, runs=100255, z=1.1359)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386294, phi_m1=-2.079441)
  [INFO] approx_entropy_m03   ApEn=0.693146 (phi_m=-2.079441, phi_m1=-2.772587)
  [INFO] approx_entropy_m04   ApEn=0.693143 (phi_m=-2.772587, phi_m1=-3.465730)
  [INFO] approx_entropy_m05   ApEn=0.693137 (phi_m=-3.465730, phi_m1=-4.158866)
  [INFO] approx_entropy_m06   ApEn=0.693119 (phi_m=-4.158866, phi_m1=-4.851985)

BAD Unix Linux glibc rand()/random()
  [PASS] knuth::permutation                                p = 0.801062  (t=5, blocks=40000, χ²=105.8060, df=119)
  [PASS] knuth::gap                                        p = 0.823636  ([0.250,0.500) gaps=50051, r=15, χ²=9.9378, df=15)
  [PASS] knuth::runs_median                                p = 0.255988  (median=0.499583, below=100000, above=100000, runs=100255, z=1.1359)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386294, phi_m1=-2.079441)
  [INFO] approx_entropy_m03   ApEn=0.693146 (phi_m=-2.079441, phi_m1=-2.772587)
  [INFO] approx_entropy_m04   ApEn=0.693143 (phi_m=-2.772587, phi_m1=-3.465730)
  [INFO] approx_entropy_m05   ApEn=0.693137 (phi_m=-3.465730, phi_m1=-4.158866)
  [INFO] approx_entropy_m06   ApEn=0.693119 (phi_m=-4.158866, phi_m1=-4.851985)

BAD Windows CRT rand()
  [PASS] knuth::permutation                                p = 0.086299  (t=5, blocks=40000, χ²=140.5640, df=119)
  [PASS] knuth::gap                                        p = 0.170906  ([0.250,0.500) gaps=50111, r=15, χ²=20.0268, df=15)
  [PASS] knuth::runs_median                                p = 0.208872  (median=0.500955, below=100000, above=100000, runs=99720, z=-1.2567)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693142 (phi_m=-2.079439, phi_m1=-2.772581)
  [INFO] approx_entropy_m04   ApEn=0.693132 (phi_m=-2.772581, phi_m1=-3.465713)
  [INFO] approx_entropy_m05   ApEn=0.693124 (phi_m=-3.465713, phi_m1=-4.158837)
  [INFO] approx_entropy_m06   ApEn=0.693098 (phi_m=-4.158837, phi_m1=-4.851935)

BAD Windows VB6/VBA Rnd()
  [PASS] knuth::permutation                                p = 0.668822  (t=5, blocks=40000, χ²=111.7460, df=119)
  [PASS] knuth::gap                                        p = 0.114820  ([0.250,0.500) gaps=49910, r=15, χ²=21.7394, df=15)
  [PASS] knuth::runs_median                                p = 0.431225  (median=0.498848, below=100000, above=100000, runs=100177, z=0.7871)
  [INFO] approx_entropy_m02   ApEn=0.693147 (phi_m=-1.386294, phi_m1=-2.079440)
  [INFO] approx_entropy_m03   ApEn=0.693146 (phi_m=-2.079440, phi_m1=-2.772586)
  [INFO] approx_entropy_m04   ApEn=0.693140 (phi_m=-2.772586, phi_m1=-3.465726)
  [INFO] approx_entropy_m05   ApEn=0.693136 (phi_m=-3.465726, phi_m1=-4.158862)
  [INFO] approx_entropy_m06   ApEn=0.693124 (phi_m=-4.158862, phi_m1=-4.851986)

BAD Windows .NET Random(seed)
  [PASS] knuth::permutation                                p = 0.634953  (t=5, blocks=40000, χ²=113.1080, df=119)
  [PASS] knuth::gap                                        p = 0.906751  ([0.250,0.500) gaps=50074, r=15, χ²=8.3999, df=15)
  [PASS] knuth::runs_median                                p = 0.310021  (median=0.498075, below=100000, above=100000, runs=100228, z=1.0152)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079440)
  [INFO] approx_entropy_m03   ApEn=0.693144 (phi_m=-2.079440, phi_m1=-2.772584)
  [INFO] approx_entropy_m04   ApEn=0.693140 (phi_m=-2.772584, phi_m1=-3.465724)
  [INFO] approx_entropy_m05   ApEn=0.693134 (phi_m=-3.465724, phi_m1=-4.158858)
  [INFO] approx_entropy_m06   ApEn=0.693114 (phi_m=-4.158858, phi_m1=-4.851972)

ANSI C sample LCG
  [PASS] knuth::permutation                                p = 0.861091  (t=5, blocks=40000, χ²=102.4220, df=119)
  [FAIL] knuth::gap                                        p = 0.000000  ([0.250,0.500) gaps=100484, r=15, χ²=51401.6062, df=15)
  [FAIL] knuth::runs_median                                p = 0.008891  (median=0.251237, below=100000, above=100000, runs=99416, z=-2.6162)
  [INFO] approx_entropy_m02   ApEn=0.692679 (phi_m=-1.385362, phi_m1=-2.078042)
  [INFO] approx_entropy_m03   ApEn=0.692678 (phi_m=-2.078042, phi_m1=-2.770720)
  [INFO] approx_entropy_m04   ApEn=0.692676 (phi_m=-2.770720, phi_m1=-3.463396)
  [INFO] approx_entropy_m05   ApEn=0.692669 (phi_m=-3.463396, phi_m1=-4.156065)
  [INFO] approx_entropy_m06   ApEn=0.692659 (phi_m=-4.156065, phi_m1=-4.848724)

LCG MINSTD
  [PASS] knuth::permutation                                p = 0.261206  (t=5, blocks=40000, χ²=128.4440, df=119)
  [FAIL] knuth::gap                                        p = 0.000000  ([0.250,0.500) gaps=100358, r=15, χ²=50762.1340, df=15)
  [PASS] knuth::runs_median                                p = 0.889737  (median=0.250876, below=100000, above=100000, runs=99970, z=-0.1386)
  [INFO] approx_entropy_m02   ApEn=0.692728 (phi_m=-1.385463, phi_m1=-2.078191)
  [INFO] approx_entropy_m03   ApEn=0.692727 (phi_m=-2.078191, phi_m1=-2.770918)
  [INFO] approx_entropy_m04   ApEn=0.692722 (phi_m=-2.770918, phi_m1=-3.463640)
  [INFO] approx_entropy_m05   ApEn=0.692714 (phi_m=-3.463640, phi_m1=-4.156354)
  [INFO] approx_entropy_m06   ApEn=0.692696 (phi_m=-4.156354, phi_m1=-4.849050)

AES-128-CTR
  [PASS] knuth::permutation                                p = 0.224876  (t=5, blocks=40000, χ²=130.3400, df=119)
  [PASS] knuth::gap                                        p = 0.111037  ([0.250,0.500) gaps=50076, r=15, χ²=21.8782, df=15)
  [PASS] knuth::runs_median                                p = 0.508050  (median=0.499150, below=100000, above=100000, runs=100149, z=0.6619)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386294, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693143 (phi_m=-2.079439, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693139 (phi_m=-2.772583, phi_m1=-3.465722)
  [INFO] approx_entropy_m05   ApEn=0.693132 (phi_m=-3.465722, phi_m1=-4.158853)
  [INFO] approx_entropy_m06   ApEn=0.693118 (phi_m=-4.158853, phi_m1=-4.851972)

cryptography::CtrDrbgAes256
  [PASS] knuth::permutation                                p = 0.357595  (t=5, blocks=40000, χ²=124.0340, df=119)
  [PASS] knuth::gap                                        p = 0.760668  ([0.250,0.500) gaps=50101, r=15, χ²=10.8855, df=15)
  [PASS] knuth::runs_median                                p = 0.205650  (median=0.500309, below=100000, above=100000, runs=99718, z=-1.2656)
  [INFO] approx_entropy_m02   ApEn=0.693147 (phi_m=-1.386294, phi_m1=-2.079441)
  [INFO] approx_entropy_m03   ApEn=0.693145 (phi_m=-2.079441, phi_m1=-2.772586)
  [INFO] approx_entropy_m04   ApEn=0.693141 (phi_m=-2.772586, phi_m1=-3.465727)
  [INFO] approx_entropy_m05   ApEn=0.693134 (phi_m=-3.465727, phi_m1=-4.158861)
  [INFO] approx_entropy_m06   ApEn=0.693120 (phi_m=-4.158861, phi_m1=-4.851981)


========================================================================
upstream_tests  (TestU01 HammingCorr/HammingIndep · PractRand FPF)
========================================================================

MT19937
  [PASS] testu01::hamming_corr                             p = 0.371781  (n=500000, r=20, s=10, L=300, rho_hat=-0.001263, z=-0.8931)
  [PASS] testu01::hamming_indep_main                       p = 0.331202  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2237.4752)
  [PASS] testu01::hamming_indep_block                      p = 0.933386  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.1379)
  [PASS] practrand::fpf_cross                              p = 0.631100  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=16.3903)
  [PASS] practrand::fpf_platter                            p = 0.022244  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8450.2163)
  [PASS] practrand::fpf_platter                            p = 0.990433  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=3885.9682)
  [PASS] practrand::fpf_platter                            p = 0.604242  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=4070.4596)
  [PASS] practrand::fpf_platter                            p = 0.678800  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=2016.7717)
  [PASS] practrand::fpf_platter                            p = 0.401956  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=1033.6009)
  [PASS] practrand::fpf_platter                            p = 0.610841  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=501.3935)
  [PASS] practrand::fpf_platter                            p = 0.929404  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=464.7634)
  [PASS] practrand::fpf_platter                            p = 0.349386  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=263.1596)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

Xorshift32
  [PASS] testu01::hamming_corr                             p = 0.191457  (n=500000, r=20, s=10, L=300, rho_hat=-0.001847, z=-1.3063)
  [PASS] testu01::hamming_indep_main                       p = 0.527734  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2203.7128)
  [PASS] testu01::hamming_indep_block                      p = 0.348958  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=2.1056)
  [PASS] practrand::fpf_cross                              p = 0.972798  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=9.0407)
  [PASS] practrand::fpf_platter                            p = 0.865927  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8049.4249)
  [PASS] practrand::fpf_platter                            p = 0.522883  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=4089.1424)
  [PASS] practrand::fpf_platter                            p = 0.788734  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=4022.1852)
  [PASS] practrand::fpf_platter                            p = 0.145000  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=2114.7723)
  [PASS] practrand::fpf_platter                            p = 0.165906  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=1066.8437)
  [PASS] practrand::fpf_platter                            p = 0.828665  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=480.6186)
  [PASS] practrand::fpf_platter                            p = 0.197699  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=537.9656)
  [PASS] practrand::fpf_platter                            p = 0.295569  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=266.6396)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

Xorshift64
  [PASS] testu01::hamming_corr                             p = 0.071751  (n=500000, r=20, s=10, L=300, rho_hat=-0.002547, z=-1.8007)
  [PASS] testu01::hamming_indep_main                       p = 0.515812  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2205.6998)
  [PASS] testu01::hamming_indep_block                      p = 0.935703  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.1329)
  [PASS] practrand::fpf_cross                              p = 0.431052  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=19.4065)
  [PASS] practrand::fpf_platter                            p = 0.603575  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8156.7688)
  [PASS] practrand::fpf_platter                            p = 0.371102  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=4124.1710)
  [PASS] practrand::fpf_platter                            p = 0.055432  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=4240.3030)
  [PASS] practrand::fpf_platter                            p = 0.865455  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=1976.4455)
  [PASS] practrand::fpf_platter                            p = 0.272079  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=1050.0041)
  [PASS] practrand::fpf_platter                            p = 0.978919  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=448.1475)
  [PASS] practrand::fpf_platter                            p = 0.896420  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=471.0920)
  [PASS] practrand::fpf_platter                            p = 0.207501  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=273.1591)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Unix System V rand()
  [PASS] testu01::hamming_corr                             p = 0.423715  (n=500000, r=20, s=10, L=300, rho_hat=0.001131, z=0.8000)
  [PASS] testu01::hamming_indep_main                       p = 0.273523  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2248.5950)
  [PASS] testu01::hamming_indep_block                      p = 0.336313  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=2.1794)
  [PASS] practrand::fpf_cross                              p = 0.060520  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=29.3607)
  [PASS] practrand::fpf_platter                            p = 0.883328  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8038.7460)
  [PASS] practrand::fpf_platter                            p = 0.471376  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=4100.8351)
  [PASS] practrand::fpf_platter                            p = 0.910063  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=3974.1693)
  [PASS] practrand::fpf_platter                            p = 0.385990  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=2064.9277)
  [PASS] practrand::fpf_platter                            p = 0.637622  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=1006.4951)
  [PASS] practrand::fpf_platter                            p = 0.462770  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=513.3248)
  [PASS] practrand::fpf_platter                            p = 0.809604  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=482.8459)
  [PASS] practrand::fpf_platter                            p = 0.061361  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=290.7420)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Unix System V mrand48()
  [PASS] testu01::hamming_corr                             p = 0.723207  (n=500000, r=20, s=10, L=300, rho_hat=-0.000501, z=-0.3542)
  [PASS] testu01::hamming_indep_main                       p = 0.153387  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2276.9508)
  [PASS] testu01::hamming_indep_block                      p = 0.078374  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=5.0925)
  [PASS] practrand::fpf_cross                              p = 0.688598  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=15.5265)
  [PASS] practrand::fpf_platter                            p = 0.862557  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8051.3815)
  [PASS] practrand::fpf_platter                            p = 0.487258  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=4097.2248)
  [PASS] practrand::fpf_platter                            p = 0.966075  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=3931.3134)
  [PASS] practrand::fpf_platter                            p = 0.239054  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=2092.0481)
  [PASS] practrand::fpf_platter                            p = 0.506887  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=1021.5530)
  [PASS] practrand::fpf_platter                            p = 0.057796  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=562.2635)
  [PASS] practrand::fpf_platter                            p = 0.426535  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=516.2726)
  [PASS] practrand::fpf_platter                            p = 0.207497  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=273.1595)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Unix BSD random()
  [PASS] testu01::hamming_corr                             p = 0.743907  (n=500000, r=20, s=10, L=300, rho_hat=-0.000462, z=-0.3267)
  [PASS] testu01::hamming_indep_main                       p = 0.795139  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2154.0001)
  [PASS] testu01::hamming_indep_block                      p = 0.698912  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.7165)
  [PASS] practrand::fpf_cross                              p = 0.861253  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=12.5403)
  [PASS] practrand::fpf_platter                            p = 0.129164  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8335.8560)
  [PASS] practrand::fpf_platter                            p = 0.068781  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=4230.1791)
  [PASS] practrand::fpf_platter                            p = 0.557969  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=4081.1521)
  [PASS] practrand::fpf_platter                            p = 0.302264  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=2079.6437)
  [PASS] practrand::fpf_platter                            p = 0.205470  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=1059.9634)
  [PASS] practrand::fpf_platter                            p = 0.947774  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=460.2094)
  [PASS] practrand::fpf_platter                            p = 0.502751  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=510.1132)
  [PASS] practrand::fpf_platter                            p = 0.576083  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=250.0314)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Unix Linux glibc rand()/random()
  [PASS] testu01::hamming_corr                             p = 0.743907  (n=500000, r=20, s=10, L=300, rho_hat=-0.000462, z=-0.3267)
  [PASS] testu01::hamming_indep_main                       p = 0.795139  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2154.0001)
  [PASS] testu01::hamming_indep_block                      p = 0.698912  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.7165)
  [PASS] practrand::fpf_cross                              p = 0.861253  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=12.5403)
  [PASS] practrand::fpf_platter                            p = 0.129164  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8335.8560)
  [PASS] practrand::fpf_platter                            p = 0.068781  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=4230.1791)
  [PASS] practrand::fpf_platter                            p = 0.557969  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=4081.1521)
  [PASS] practrand::fpf_platter                            p = 0.302264  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=2079.6437)
  [PASS] practrand::fpf_platter                            p = 0.205470  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=1059.9634)
  [PASS] practrand::fpf_platter                            p = 0.947774  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=460.2094)
  [PASS] practrand::fpf_platter                            p = 0.502751  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=510.1132)
  [PASS] practrand::fpf_platter                            p = 0.576083  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=250.0314)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Windows CRT rand()
  [PASS] testu01::hamming_corr                             p = 0.240982  (n=500000, r=20, s=10, L=300, rho_hat=-0.001658, z=-1.1725)
  [PASS] testu01::hamming_indep_main                       p = 0.890950  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2127.4942)
  [FAIL] testu01::hamming_indep_block                      p = 0.004296  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=10.9001)
  [PASS] practrand::fpf_cross                              p = 0.720721  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=15.0295)
  [PASS] practrand::fpf_platter                            p = 0.894019  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8031.6192)
  [PASS] practrand::fpf_platter                            p = 0.059439  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=4237.0784)
  [PASS] practrand::fpf_platter                            p = 0.614359  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=4068.0843)
  [PASS] practrand::fpf_platter                            p = 0.802556  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=1992.3882)
  [PASS] practrand::fpf_platter                            p = 0.348652  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=1040.0215)
  [PASS] practrand::fpf_platter                            p = 0.193420  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=538.4783)
  [PASS] practrand::fpf_platter                            p = 0.578127  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=504.0630)
  [PASS] practrand::fpf_platter                            p = 0.815609  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=234.6026)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Windows VB6/VBA Rnd()
  [FAIL] testu01::hamming_corr                             p = 0.000000  (n=500000, r=20, s=10, L=300, rho_hat=-0.087334, z=-61.7546)
  [FAIL] testu01::hamming_indep_main                       p = 0.000000  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=60021.7661)
  [FAIL] testu01::hamming_indep_block                      p = 0.000000  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=3183.8786)
  [FAIL] practrand::fpf_cross                              p = 0.000000  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=952971.3223)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=734194.2780)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=177402.7720)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=89375.7595)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=44629.7724)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=22281.2976)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=11166.3602)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=5713.3618)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=2866.1390)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Windows .NET Random(seed)
  [PASS] testu01::hamming_corr                             p = 0.467237  (n=500000, r=20, s=10, L=300, rho_hat=-0.001028, z=-0.7270)
  [PASS] testu01::hamming_indep_main                       p = 0.118997  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2287.6840)
  [PASS] testu01::hamming_indep_block                      p = 0.813001  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.4140)
  [PASS] practrand::fpf_cross                              p = 0.400004  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=19.9101)
  [PASS] practrand::fpf_platter                            p = 0.758138  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8101.0281)
  [PASS] practrand::fpf_platter                            p = 0.236445  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=4159.6296)
  [PASS] practrand::fpf_platter                            p = 0.138485  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=4193.4990)
  [PASS] practrand::fpf_platter                            p = 0.322854  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=2075.8845)
  [PASS] practrand::fpf_platter                            p = 0.261043  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=1051.5512)
  [PASS] practrand::fpf_platter                            p = 0.732929  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=490.7306)
  [PASS] practrand::fpf_platter                            p = 0.643630  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=498.6624)
  [PASS] practrand::fpf_platter                            p = 0.941413  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=220.6223)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

ANSI C sample LCG
  [FAIL] testu01::hamming_corr                             p = 0.000000  (n=500000, r=20, s=10, L=300, rho_hat=-0.035991, z=-25.4497)
  [FAIL] testu01::hamming_indep_main                       p = 0.000000  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=813696.0622)
  [FAIL] testu01::hamming_indep_block                      p = 0.000000  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=1298.3545)
  [FAIL] practrand::fpf_cross                              p = 0.000000  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=76.1364)
  [PASS] practrand::fpf_platter                            p = 1.000000  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=4067.7166)
  [PASS] practrand::fpf_platter                            p = 1.000000  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=2051.1844)
  [PASS] practrand::fpf_platter                            p = 1.000000  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=1995.9486)
  [PASS] practrand::fpf_platter                            p = 1.000000  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=1040.0821)
  [PASS] practrand::fpf_platter                            p = 1.000000  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=536.6210)
  [PASS] practrand::fpf_platter                            p = 1.000000  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=265.3846)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=17189.9711)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=8837.4647)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

LCG MINSTD
  [PASS] testu01::hamming_corr                             p = 0.923793  (n=500000, r=20, s=10, L=300, rho_hat=-0.000135, z=-0.0957)
  [PASS] testu01::hamming_indep_main                       p = 0.069719  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2308.0100)
  [PASS] testu01::hamming_indep_block                      p = 0.465298  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=1.5302)
  [FAIL] practrand::fpf_cross                              p = 0.000000  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=74.5128)
  [PASS] practrand::fpf_platter                            p = 0.638646  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8145.0023)
  [PASS] practrand::fpf_platter                            p = 0.036596  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=4258.6048)
  [PASS] practrand::fpf_platter                            p = 0.237293  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=4159.3785)
  [PASS] practrand::fpf_platter                            p = 0.771823  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=1999.0520)
  [PASS] practrand::fpf_platter                            p = 0.401277  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=1033.6809)
  [PASS] practrand::fpf_platter                            p = 0.810535  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=482.7402)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=17738.8305)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=8973.2165)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

AES-128-CTR
  [PASS] testu01::hamming_corr                             p = 0.604025  (n=500000, r=20, s=10, L=300, rho_hat=0.000733, z=0.5186)
  [PASS] testu01::hamming_indep_main                       p = 0.346377  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2234.6957)
  [PASS] testu01::hamming_indep_block                      p = 0.936284  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.1317)
  [PASS] practrand::fpf_cross                              p = 0.255439  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=22.5996)
  [PASS] practrand::fpf_platter                            p = 0.558517  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8171.5068)
  [PASS] practrand::fpf_platter                            p = 0.322792  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=4136.0914)
  [PASS] practrand::fpf_platter                            p = 0.596694  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=4072.2213)
  [PASS] practrand::fpf_platter                            p = 0.290077  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=2081.9245)
  [PASS] practrand::fpf_platter                            p = 0.043946  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=1101.4531)
  [PASS] practrand::fpf_platter                            p = 0.813697  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=482.3792)
  [PASS] practrand::fpf_platter                            p = 0.827641  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=480.7419)
  [PASS] practrand::fpf_platter                            p = 0.179830  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=275.5516)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

cryptography::CtrDrbgAes256
  [PASS] testu01::hamming_corr                             p = 0.310632  (n=500000, r=20, s=10, L=300, rho_hat=0.001434, z=1.0139)
  [PASS] testu01::hamming_indep_main                       p = 0.763462  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2160.9946)
  [PASS] testu01::hamming_indep_block                      p = 0.244947  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=2.8134)
  [PASS] practrand::fpf_cross                              p = 0.596878  (samples=8388604, stride_bits=16, sig_bits=14, max_exp=63, dof=19, chi2=16.8965)
  [PASS] practrand::fpf_platter                            p = 0.418568  (samples=8388604, stride_bits=16, e=0, sig_bins=2^13, dof=8191, chi2=8216.6700)
  [PASS] practrand::fpf_platter                            p = 0.518802  (samples=8388604, stride_bits=16, e=1, sig_bins=2^12, dof=4095, chi2=4090.0684)
  [PASS] practrand::fpf_platter                            p = 0.910187  (samples=8388604, stride_bits=16, e=2, sig_bins=2^12, dof=4095, chi2=3974.1018)
  [PASS] practrand::fpf_platter                            p = 0.614900  (samples=8388604, stride_bits=16, e=3, sig_bins=2^11, dof=2047, chi2=2027.7031)
  [PASS] practrand::fpf_platter                            p = 0.785338  (samples=8388604, stride_bits=16, e=4, sig_bins=2^10, dof=1023, chi2=987.0125)
  [PASS] practrand::fpf_platter                            p = 0.669124  (samples=8388604, stride_bits=16, e=5, sig_bins=2^9, dof=511, chi2=496.4853)
  [PASS] practrand::fpf_platter                            p = 0.273148  (samples=8388604, stride_bits=16, e=6, sig_bins=2^9, dof=511, chi2=529.8496)
  [PASS] practrand::fpf_platter                            p = 0.346767  (samples=8388604, stride_bits=16, e=7, sig_bins=2^8, dof=255, chi2=263.3231)
  [INFO] practrand::fpf_more                   9 additional platter results omitted


========================================================================
testu01_lz  (TestU01 Lempel-Ziv  k=25  replications=10)
========================================================================

MT19937
  [PASS] testu01::lzw_sum                                  p = 0.216239  (N=10, k=25, r=0, s=30, z_mean=-0.3910, z_sum=-1.2366)
  [PASS] testu01::lzw_ks                                   p = 0.999637  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1762933 z=-0.9552
  [INFO] testu01::lzw_rep02                    W=1762950 z=-0.4478
  [INFO] testu01::lzw_rep03                    W=1762918 z=-1.4030
  [INFO] testu01::lzw_rep04                    W=1762999 z=1.0149
  [INFO] testu01::lzw_rep05                    W=1762971 z=0.1791
  [INFO] testu01::lzw_rep06                    W=1762935 z=-0.8955
  [INFO] testu01::lzw_rep07                    W=1762939 z=-0.7761
  [INFO] testu01::lzw_rep08                    W=1762981 z=0.4776
  [INFO] testu01::lzw_rep09                    W=1762940 z=-0.7463
  [INFO] testu01::lzw_rep10                    W=1762953 z=-0.3582

Xorshift32
  [PASS] testu01::lzw_sum                                  p = 0.992469  (N=10, k=25, r=0, s=30, z_mean=-0.0030, z_sum=-0.0094)
  [PASS] testu01::lzw_ks                                   p = 0.999637  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1762985 z=0.5970
  [INFO] testu01::lzw_rep02                    W=1762916 z=-1.4627
  [INFO] testu01::lzw_rep03                    W=1762950 z=-0.4478
  [INFO] testu01::lzw_rep04                    W=1762990 z=0.7463
  [INFO] testu01::lzw_rep05                    W=1762959 z=-0.1791
  [INFO] testu01::lzw_rep06                    W=1762943 z=-0.6567
  [INFO] testu01::lzw_rep07                    W=1762962 z=-0.0896
  [INFO] testu01::lzw_rep08                    W=1762978 z=0.3881
  [INFO] testu01::lzw_rep09                    W=1763035 z=2.0896
  [INFO] testu01::lzw_rep10                    W=1762931 z=-1.0149

Xorshift64
  [PASS] testu01::lzw_sum                                  p = 0.151337  (N=10, k=25, r=0, s=30, z_mean=-0.4537, z_sum=-1.4348)
  [PASS] testu01::lzw_ks                                   p = 0.999637  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1762946 z=-0.5672
  [INFO] testu01::lzw_rep02                    W=1762978 z=0.3881
  [INFO] testu01::lzw_rep03                    W=1762961 z=-0.1194
  [INFO] testu01::lzw_rep04                    W=1762935 z=-0.8955
  [INFO] testu01::lzw_rep05                    W=1762918 z=-1.4030
  [INFO] testu01::lzw_rep06                    W=1762952 z=-0.3881
  [INFO] testu01::lzw_rep07                    W=1762940 z=-0.7463
  [INFO] testu01::lzw_rep08                    W=1763007 z=1.2537
  [INFO] testu01::lzw_rep09                    W=1762943 z=-0.6567
  [INFO] testu01::lzw_rep10                    W=1762918 z=-1.4030

BAD Unix System V rand()
  [PASS] testu01::lzw_sum                                  p = 0.590536  (N=10, k=25, r=0, s=30, z_mean=-0.1701, z_sum=-0.5381)
  [PASS] testu01::lzw_ks                                   p = 0.999637  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1762954 z=-0.3284
  [INFO] testu01::lzw_rep02                    W=1762949 z=-0.4776
  [INFO] testu01::lzw_rep03                    W=1762945 z=-0.5970
  [INFO] testu01::lzw_rep04                    W=1762958 z=-0.2090
  [INFO] testu01::lzw_rep05                    W=1762974 z=0.2687
  [INFO] testu01::lzw_rep06                    W=1762936 z=-0.8657
  [INFO] testu01::lzw_rep07                    W=1763024 z=1.7612
  [INFO] testu01::lzw_rep08                    W=1762971 z=0.1791
  [INFO] testu01::lzw_rep09                    W=1762951 z=-0.4179
  [INFO] testu01::lzw_rep10                    W=1762931 z=-1.0149

BAD Unix System V mrand48()
  [PASS] testu01::lzw_sum                                  p = 0.719815  (N=10, k=25, r=0, s=30, z_mean=0.1134, z_sum=0.3587)
  [PASS] testu01::lzw_ks                                   p = 0.999637  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1762884 z=-2.4179
  [INFO] testu01::lzw_rep02                    W=1762984 z=0.5672
  [INFO] testu01::lzw_rep03                    W=1762921 z=-1.3134
  [INFO] testu01::lzw_rep04                    W=1762998 z=0.9851
  [INFO] testu01::lzw_rep05                    W=1762951 z=-0.4179
  [INFO] testu01::lzw_rep06                    W=1763024 z=1.7612
  [INFO] testu01::lzw_rep07                    W=1763014 z=1.4627
  [INFO] testu01::lzw_rep08                    W=1762970 z=0.1493
  [INFO] testu01::lzw_rep09                    W=1762962 z=-0.0896
  [INFO] testu01::lzw_rep10                    W=1762980 z=0.4478

BAD Unix BSD random()
  [PASS] testu01::lzw_sum                                  p = 0.969880  (N=10, k=25, r=0, s=30, z_mean=-0.0119, z_sum=-0.0378)
  [PASS] testu01::lzw_ks                                   p = 0.999637  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1762964 z=-0.0299
  [INFO] testu01::lzw_rep02                    W=1763021 z=1.6716
  [INFO] testu01::lzw_rep03                    W=1762904 z=-1.8209
  [INFO] testu01::lzw_rep04                    W=1762982 z=0.5075
  [INFO] testu01::lzw_rep05                    W=1763007 z=1.2537
  [INFO] testu01::lzw_rep06                    W=1762961 z=-0.1194
  [INFO] testu01::lzw_rep07                    W=1762949 z=-0.4776
  [INFO] testu01::lzw_rep08                    W=1762981 z=0.4776
  [INFO] testu01::lzw_rep09                    W=1762932 z=-0.9851
  [INFO] testu01::lzw_rep10                    W=1762945 z=-0.5970

BAD Unix Linux glibc rand()/random()
  [PASS] testu01::lzw_sum                                  p = 0.969880  (N=10, k=25, r=0, s=30, z_mean=-0.0119, z_sum=-0.0378)
  [PASS] testu01::lzw_ks                                   p = 0.999637  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1762964 z=-0.0299
  [INFO] testu01::lzw_rep02                    W=1763021 z=1.6716
  [INFO] testu01::lzw_rep03                    W=1762904 z=-1.8209
  [INFO] testu01::lzw_rep04                    W=1762982 z=0.5075
  [INFO] testu01::lzw_rep05                    W=1763007 z=1.2537
  [INFO] testu01::lzw_rep06                    W=1762961 z=-0.1194
  [INFO] testu01::lzw_rep07                    W=1762949 z=-0.4776
  [INFO] testu01::lzw_rep08                    W=1762981 z=0.4776
  [INFO] testu01::lzw_rep09                    W=1762932 z=-0.9851
  [INFO] testu01::lzw_rep10                    W=1762945 z=-0.5970

BAD Windows CRT rand()
  [PASS] testu01::lzw_sum                                  p = 0.159574  (N=10, k=25, r=0, s=30, z_mean=0.4448, z_sum=1.4065)
  [PASS] testu01::lzw_ks                                   p = 0.999637  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1762952 z=-0.3881
  [INFO] testu01::lzw_rep02                    W=1763026 z=1.8209
  [INFO] testu01::lzw_rep03                    W=1762990 z=0.7463
  [INFO] testu01::lzw_rep04                    W=1762961 z=-0.1194
  [INFO] testu01::lzw_rep05                    W=1763020 z=1.6418
  [INFO] testu01::lzw_rep06                    W=1762944 z=-0.6269
  [INFO] testu01::lzw_rep07                    W=1762963 z=-0.0597
  [INFO] testu01::lzw_rep08                    W=1762997 z=0.9552
  [INFO] testu01::lzw_rep09                    W=1762973 z=0.2388
  [INFO] testu01::lzw_rep10                    W=1762973 z=0.2388

BAD Windows VB6/VBA Rnd()
  [FAIL] testu01::lzw_sum                                  p = 0.000000  (N=10, k=25, r=0, s=30, z_mean=-221.9851, z_sum=-701.9784)
  [FAIL] testu01::lzw_ks                                   p = 0.000000  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1755485 z=-223.2836
  [INFO] testu01::lzw_rep02                    W=1755545 z=-221.4925
  [INFO] testu01::lzw_rep03                    W=1755563 z=-220.9552
  [INFO] testu01::lzw_rep04                    W=1755535 z=-221.7910
  [INFO] testu01::lzw_rep05                    W=1755551 z=-221.3134
  [INFO] testu01::lzw_rep06                    W=1755521 z=-222.2090
  [INFO] testu01::lzw_rep07                    W=1755536 z=-221.7612
  [INFO] testu01::lzw_rep08                    W=1755473 z=-223.6418
  [INFO] testu01::lzw_rep09                    W=1755512 z=-222.4776
  [INFO] testu01::lzw_rep10                    W=1755564 z=-220.9254

BAD Windows .NET Random(seed)
  [PASS] testu01::lzw_sum                                  p = 0.219765  (N=10, k=25, r=0, s=30, z_mean=0.3881, z_sum=1.2272)
  [PASS] testu01::lzw_ks                                   p = 0.999637  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1762967 z=0.0597
  [INFO] testu01::lzw_rep02                    W=1762886 z=-2.3582
  [INFO] testu01::lzw_rep03                    W=1762991 z=0.7761
  [INFO] testu01::lzw_rep04                    W=1762976 z=0.3284
  [INFO] testu01::lzw_rep05                    W=1763020 z=1.6418
  [INFO] testu01::lzw_rep06                    W=1762976 z=0.3284
  [INFO] testu01::lzw_rep07                    W=1762883 z=-2.4478
  [INFO] testu01::lzw_rep08                    W=1763001 z=1.0746
  [INFO] testu01::lzw_rep09                    W=1763018 z=1.5821
  [INFO] testu01::lzw_rep10                    W=1763062 z=2.8955

ANSI C sample LCG
  [FAIL] testu01::lzw_sum                                  p = 0.000000  (N=10, k=25, r=0, s=30, z_mean=-34.9821, z_sum=-110.6231)
  [FAIL] testu01::lzw_ks                                   p = 0.000000  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1761859 z=-33.0149
  [INFO] testu01::lzw_rep02                    W=1761837 z=-33.6716
  [INFO] testu01::lzw_rep03                    W=1761768 z=-35.7313
  [INFO] testu01::lzw_rep04                    W=1761798 z=-34.8358
  [INFO] testu01::lzw_rep05                    W=1761764 z=-35.8507
  [INFO] testu01::lzw_rep06                    W=1761816 z=-34.2985
  [INFO] testu01::lzw_rep07                    W=1761741 z=-36.5373
  [INFO] testu01::lzw_rep08                    W=1761768 z=-35.7313
  [INFO] testu01::lzw_rep09                    W=1761800 z=-34.7761
  [INFO] testu01::lzw_rep10                    W=1761780 z=-35.3731

LCG MINSTD
  [FAIL] testu01::lzw_sum                                  p = 0.000000  (N=10, k=25, r=0, s=30, z_mean=-37.7104, z_sum=-119.2509)
  [FAIL] testu01::lzw_ks                                   p = 0.000000  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1761635 z=-39.7015
  [INFO] testu01::lzw_rep02                    W=1761843 z=-33.4925
  [INFO] testu01::lzw_rep03                    W=1761693 z=-37.9701
  [INFO] testu01::lzw_rep04                    W=1761676 z=-38.4776
  [INFO] testu01::lzw_rep05                    W=1761717 z=-37.2537
  [INFO] testu01::lzw_rep06                    W=1761680 z=-38.3582
  [INFO] testu01::lzw_rep07                    W=1761686 z=-38.1791
  [INFO] testu01::lzw_rep08                    W=1761681 z=-38.3284
  [INFO] testu01::lzw_rep09                    W=1761689 z=-38.0896
  [INFO] testu01::lzw_rep10                    W=1761717 z=-37.2537

AES-128-CTR
  [PASS] testu01::lzw_sum                                  p = 0.052975  (N=10, k=25, r=0, s=30, z_mean=0.6119, z_sum=1.9351)
  [PASS] testu01::lzw_ks                                   p = 0.999637  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1762990 z=0.7463
  [INFO] testu01::lzw_rep02                    W=1762977 z=0.3582
  [INFO] testu01::lzw_rep03                    W=1762968 z=0.0896
  [INFO] testu01::lzw_rep04                    W=1763003 z=1.1343
  [INFO] testu01::lzw_rep05                    W=1762950 z=-0.4478
  [INFO] testu01::lzw_rep06                    W=1762949 z=-0.4776
  [INFO] testu01::lzw_rep07                    W=1763021 z=1.6716
  [INFO] testu01::lzw_rep08                    W=1763026 z=1.8209
  [INFO] testu01::lzw_rep09                    W=1762964 z=-0.0299
  [INFO] testu01::lzw_rep10                    W=1763007 z=1.2537

cryptography::CtrDrbgAes256
  [PASS] testu01::lzw_sum                                  p = 0.527089  (N=10, k=25, r=0, s=30, z_mean=0.2000, z_sum=0.6325)
  [PASS] testu01::lzw_ks                                   p = 0.999637  (N=10, k=25, r=0, s=30)
  [INFO] testu01::lzw_rep01                    W=1762988 z=0.6866
  [INFO] testu01::lzw_rep02                    W=1763007 z=1.2537
  [INFO] testu01::lzw_rep03                    W=1763003 z=1.1343
  [INFO] testu01::lzw_rep04                    W=1762932 z=-0.9851
  [INFO] testu01::lzw_rep05                    W=1762953 z=-0.3582
  [INFO] testu01::lzw_rep06                    W=1762982 z=0.5075
  [INFO] testu01::lzw_rep07                    W=1762965 z=0.0000
  [INFO] testu01::lzw_rep08                    W=1762954 z=-0.3284
  [INFO] testu01::lzw_rep09                    W=1763001 z=1.0746
  [INFO] testu01::lzw_rep10                    W=1762932 z=-0.9851


========================================================================
webster_tavares  (SAC / BIC avalanche  samples=4096  bits=32)
========================================================================

RNG                                       samples  SACmean   SACmax  BICmean   BICmax
----------------------------------------------------------------------------------------
MT19937                                      4096   0.0065   0.0281   0.0124   0.0611
Xorshift32                                   4096   0.5000   0.5000   0.0000   0.0000
Xorshift64                                   4096   0.5000   0.5000   0.0000   0.0000
BAD Unix System V rand()                     4096   0.3334   0.5000   0.0473   0.7659
BAD Unix System V mrand48()                  4096   0.3782   0.5000   0.0349   0.9011
BAD Unix BSD random()                        4096   0.0432   0.4834   0.0208   0.7244
BAD Unix Linux glibc rand()/random()         4096   0.0432   0.4834   0.0208   0.7244
BAD Windows CRT rand()                       4096   0.3310   0.5000   0.0486   0.8559
BAD Windows VB6/VBA Rnd()                    4096   0.4173   0.5000   0.0257   0.8228
BAD Windows .NET Random(seed)                4096   0.2346   0.4944   0.0697   0.7415
ANSI C sample LCG                            4096   0.3617   0.5000   0.0280   0.7015
LCG MINSTD                                   4096   0.2464   0.5000   0.0728   0.7244
AES-128-CTR                                  4096   0.0061   0.0232   0.0125   0.0663
cryptography::CtrDrbgAes256                  4096   0.0063   0.0308   0.0125   0.0610

========================================================================
gorilla  (Marsaglia-Tsang Gorilla  all 32 bit positions)
========================================================================

RNG                                          min_p     max_p worst_bit  worst_|z|   agg_ks_p
-----------------------------------------------------------------------------------------------
MT19937                                   0.025510  0.963248        28      1.951   1.000000
Xorshift32                                1.000000  1.000000        24     47.913   0.000000
Xorshift64                                0.042486  0.959682        19      1.747   1.000000
BAD Unix System V rand()                  0.103973  0.999980        18      4.111   0.000000
BAD Unix System V mrand48()               0.000000  1.000000        31  10141.521   1.000000
BAD Unix BSD random()                     0.114651  0.975209        13      1.964   1.000000
BAD Unix Linux glibc rand()/random()      0.114651  0.975209        13      1.964   1.000000
BAD Windows CRT rand()                    0.000000  0.999969         3     12.004   1.000000
BAD Windows VB6/VBA Rnd()                 0.000000  0.000000         7  10162.438   0.000000
BAD Windows .NET Random(seed)             0.026731  0.969952        28      1.931   1.000000
ANSI C sample LCG                         0.000000  1.000000         0  10172.876   0.000000
LCG MINSTD                                0.000000  1.000000         0  10172.876   0.000000
AES-128-CTR                               0.050697  0.987885         9      2.253   1.000000
cryptography::CtrDrbgAes256               0.000534  0.993353        31      3.272   1.000000
```
