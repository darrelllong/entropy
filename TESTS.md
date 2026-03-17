# Full Battery Results

Full `run_tests` battery harvested from `darby.local` on 2026-03-16.

Sample size: **16 Mbit** per generator.

Command:

```sh
./target/release/run_tests
```

Scope:

This top section covers the standard `run_tests` battery only.  The Knuth,
TestU01, PractRand, Webster-Tavares, and Gorilla probes are reported
separately in `## Auxiliary Probes`; use `tests/run_all.sh` for the combined
audit path or `tests/run_aux.sh` for the auxiliary suite alone.

Notes:

**Why result counts vary across generators.**

The battery total differs from run to run because several test families are
conditionally skipped based on properties of the sample, not the generator.

The battery has **738 test slots** at this sample size:

- **738 results** — the "full active battery" outcome: the signed-random-walk
  tests (`random_excursions` and `random_excursions_variant`) completed
  successfully (J ≥ 500 zero-crossing cycles).  At 16 Mbit the expected
  cycle count is J ≈ 3191 (= √(2n/π)),
  which is comfortably above the threshold for well-behaved generators.

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

Full log: [logs/run_all-darby-20260316-192756.log](logs/run_all-darby-20260316-192756.log) (darby.local, 2026-03-16, 738 tests/RNG at α = 0.01)

| RNG | Total | PASS | FAIL | SKIP |
|---|---:|---:|---:|---:|
| OsRng (/dev/urandom) | 738 | 734 | 4 | 0 |
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
| AES-128-CTR (NIST key) | 738 | 733 | 5 | 0 |
| Camellia-128-CTR (key=00..0f) | 738 | 732 | 6 | 0 |
| Twofish-128-CTR (key=00..0f) | 738 | 728 | 10 | 0 |
| Serpent-128-CTR (key=00..0f) | 738 | 727 | 11 | 0 |
| SM4-CTR (key=00..0f) | 738 | 732 | 6 | 0 |
| Grasshopper-CTR (key=00..1f) | 738 | 734 | 4 | 0 |
| CAST-128-CTR (key=00..0f) | 738 | 731 | 7 | 0 |
| SEED-CTR (key=00..0f) | 738 | 732 | 6 | 0 |
| Rabbit (key=00..0f, iv=00..07) | 738 | 732 | 6 | 0 |
| Salsa20 (key=00..1f, nonce=00..07) | 738 | 732 | 6 | 0 |
| Snow3G (key=00..0f, iv=00..0f) | 738 | 729 | 9 | 0 |
| ZUC-128 (key=00..0f, iv=00..0f) | 738 | 736 | 2 | 0 |
| SpongeBob (SHA3-512 chain, OsRng seed) | 738 | 726 | 12 | 0 |
| Squidward (SHA-256 chain, OsRng seed) | 738 | 731 | 7 | 0 |
| PCG32 (OsRng seed) | 738 | 729 | 9 | 0 |
| PCG64 (OsRng seed) | 738 | 726 | 12 | 0 |
| Xoshiro256** (OsRng seed) | 738 | 731 | 7 | 0 |
| Xoroshiro128** (OsRng seed) | 738 | 732 | 6 | 0 |
| WyRand (OsRng seed) | 714 | 706 | 6 | 2 |
| SFC64 (OsRng seed) | 738 | 728 | 10 | 0 |
| JSF64 (OsRng seed) | 738 | 733 | 5 | 0 |
| ChaCha20 CSPRNG (OsRng key) | 738 | 728 | 10 | 0 |
| HMAC_DRBG SHA-256 (OsRng seed) | 738 | 735 | 3 | 0 |
| Hash_DRBG SHA-256 (OsRng seed) | 738 | 730 | 8 | 0 |
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

### Research Probes

These seven tests live in `src/research/` and are run via `cargo test -- --include-ignored`.
They probe structural properties that the standard batteries underweight.

- **Knuth permutation test** (TAOCP §3.3.2).  Draw non-overlapping windows of
  $t$ successive output words ($t = 3$ and $t = 4$ in the crate), rank each
  window to obtain its permutation ordinal in $S_t$, and accumulate counts over
  the $t!$ orderings.  The null distribution is uniform, so the test statistic
  is $\chi^2 = \sum_{i=0}^{t!-1} (O_i - N/t!)^2/(N/t!)$ on $t!-1$ degrees of
  freedom.  Sensitive to short-range ordering biases that survive frequency tests.

- **Knuth gap test** (TAOCP §3.3.2).  Fix an interval $[\alpha,\beta)$ and
  record the lengths $L$ of the gaps (runs of words outside the interval)
  between successive hits.  Under the null, $L$ follows a geometric distribution
  with success probability $p = \beta - \alpha$.  The observed gap-length
  histogram (pooled at an upper cutoff) is compared to this geometric law by
  chi-square.  Tests uniformity of the real-valued projection and independence
  of successive words.

- **Knuth runs test** (Wald–Wolfowitz).  Scan the output sequence and count runs
  of consecutive ascending values.  For a truly uniform i.i.d. sequence of $n$
  words, the number of runs $R$ has mean $\mu = (2n-1)/3$ and variance
  $\sigma^2 = (16n-29)/90$.  The crate reports $z = (R - \mu)/\sigma$ and the
  two-tailed p-value $p = \mathrm{erfc}(|z|/\sqrt{2})$.  Detects serial
  monotonicity biases (too many or too few direction reversals).

- **PractRand FPF** (float-point-frequency).  Convert 32-bit words to IEEE 754
  single-precision floats and partition them by their 8-bit biased exponent into
  256 buckets.  Within each exponent bucket the significand bits form an
  independent bit stream; the test applies a G-test (log-likelihood ratio
  chi-square) to the per-bucket bit-frequency histogram and separately to the
  cross-exponent marginal.  Detects carry and alignment artifacts that are
  invisible to integer-domain frequency tests.

- **TestU01 Lempel–Ziv** (`smultin_Lempel_Ziv`).  Parse the bit stream as an
  LZ78 dictionary: each new phrase extends the longest previously seen prefix by
  one bit.  Let $C_n$ be the number of distinct phrases after reading $n$ bits.
  Asymptotically, $C_n / (n / \log_2 n) \to 1$ for a fair coin.  The crate uses
  empirical tables of $(\mu, \sigma)$ for $n = 2^k$ taken from the TestU01
  source to compute a z-score and derives the p-value from the normal
  approximation.  An outer KS over multiple replications converts per-replication
  z-scores to a single battery p-value.  Low complexity (few phrases) flags
  repetitive structure; high complexity flags over-dispersion.

- **TestU01 Hamming** (`sstring_HammingCorr` / `sstring_HammingIndep`).
  `HammingCorr`: Extract $L$-bit blocks from the bit stream, compute the
  Hamming weight $W = \sum b_i$ of each block, and compare the weight histogram
  to $\mathrm{Bin}(L, \tfrac12)$ by chi-square.  $L = 32$ and $L = 64$ are both
  tested.
  `HammingIndep`: For successive pairs of $L$-bit blocks $(X, Y)$, compute the
  joint weight histogram $(W_X, W_Y)$ and compare to the product distribution
  $\mathrm{Bin}(L,\tfrac12)\times\mathrm{Bin}(L,\tfrac12)$.  The chi-square
  statistic measures whether Hamming weights of consecutive blocks are
  correlated.  Detects linear dependencies across block boundaries.

- **Webster–Tavares strict avalanche criterion (SAC)**.  For each bit position
  $i$ in a $k$-word window, flip bit $i$ in the input seed and measure the
  fraction of output bits that change.  Under the SAC, every output bit should
  flip with probability exactly $\tfrac12$ when any single input bit is flipped.
  The crate computes the pairwise correlation coefficient $\rho_{ij}$ between the
  flip indicators for input bit $i$ and output bit $j$, and reports a chi-square
  on the full $k \times k$ correlation matrix.  Cryptographic generators should
  pass; LCGs and short-state generators fail badly because their internal
  diffusion is limited.

- **Gorilla** (Marsaglia–Tsang).  For each of the 32 bit positions, extract that
  bit from $2^{26}+25$ successive words to form a stream of $2^{26}+25$ bits.
  Count the number of distinct 26-bit patterns that never appear (missing words).
  Under the null, the number of missing words follows approximately
  $N(24{,}687{,}971,\ 4170^2)$ (Marsaglia's analytic result).  The crate
  collects one p-value per bit position and applies a KS test over the 32
  p-values, detecting positional asymmetries and bit-plane correlations invisible
  to the standard birthday-problem tests.

- **Multi-scale approximate entropy (ApEn)**.  For template lengths
  $m = 1, 2, \dots, M$ (default $M = 8$), compute Pincus's
  $\mathrm{ApEn}(m, r, N)$ on the output sequence, where $r$ is set to
  $0.2\,\sigma$ of the data.  Each ApEn value measures the log-likelihood that
  runs of $m$ consecutive samples that are close together remain close for $m+1$
  samples.  The crate reports the ApEn profile across scales; a random sequence
  should sustain near-maximal entropy at every scale, while structured generators
  show a characteristic drop-off at the scale where their internal period or
  dependency length becomes apparent.

## Failure Highlights

One line per generator.  Test-family repetition counts in parentheses.

- **OsRng (/dev/urandom)**: 4/738 — `dieharder::bit_distribution` (×4)
- **MT19937 (seed=19650218)**: 10/738 — `dieharder::bit_distribution` (×8), `nist::non_overlapping_template` (×2)
- **Xorshift64 (seed=1)**: 5/738 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template`
- **Xorshift32 (seed=1)**: 12/738 — `diehard::binary_rank_31x31`, `diehard::binary_rank_32x32`, `dieharder::bit_distribution` (×9), `nist::matrix_rank`
- **BAD Unix System V rand() (15-bit LCG, seed=1)**: 12/738 — `diehard::opso`, `dieharder::bit_distribution` (×8), `nist::non_overlapping_template` (×2), `nist::spectral`
- **BAD Unix System V mrand48() (seed=1)**: 7/738 — `diehard::dna`, `diehard::opso`, `diehard::oqso`, `dieharder::bit_distribution` (×2), `nist::non_overlapping_template` (×2)
- **BAD Unix BSD random() TYPE_3 (seed=1)**: 10/738 — `dieharder::bit_distribution` (×5), `nist::non_overlapping_template` (×5)
- **BAD Unix Linux glibc rand()/random() (seed=1)**: 10/738 — `dieharder::bit_distribution` (×5), `nist::non_overlapping_template` (×5)
- **BAD Unix FreeBSD12 rand_r() compat (seed=1)**: 10/738 — `dieharder::bit_distribution` (×7), `nist::non_overlapping_template` (×2), `nist::spectral`
- **BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1)**: 10/738 — `dieharder::bit_distribution` (×7), `dieharder::dct`, `nist::random_excursions`, `nist::spectral`
- **BAD Windows VB6/VBA Rnd() (project seed=1)**: 529/738 — `diehard::binary_rank_6x8`, `diehard::bitstream`, `diehard::count_ones_stream`, `diehard::craps_throws`, `diehard::dna`, `diehard::minimum_distance_2d`, `diehard::opso`, `diehard::oqso`, `diehard::squeeze`, `dieharder::bit_distribution` (×497), `dieharder::byte_distribution`, `dieharder::dct`, `dieharder::fill_tree_count`, `dieharder::fill_tree_position`, `dieharder::gcd_distribution`, `dieharder::gcd_step_counts`, `dieharder::ks_uniform`, `dieharder::lagged_sums`, `dieharder::minimum_distance_nd`, `maurer::universal_l06`, `maurer::universal_l07`, `maurer::universal_l08`, `maurer::universal_l09`, `maurer::universal_l10`, `maurer::universal_l11`, `maurer::universal_l12`, `maurer::universal_l13`, `maurer::universal_l14`, `maurer::universal_l15`, `maurer::universal_l16`, `nist::overlapping_template`, `nist::spectral`, `nist::universal`
- **BAD Windows .NET Random(seed=1) compat**: 7/738 — `dieharder::bit_distribution` (×3), `dieharder::dct`, `nist::non_overlapping_template` (×3)
- **ANSI C sample LCG (1103515245,12345; seed=1)**: 697/714 — `diehard::binary_rank_32x32`, `diehard::binary_rank_6x8`, `diehard::bitstream`, `diehard::count_ones_stream`, `diehard::craps_throws`, `diehard::craps_wins`, `diehard::dna`, `diehard::minimum_distance_2d`, `diehard::opso`, `diehard::oqso`, `diehard::parking_lot`, `diehard::spheres_3d`, `diehard::squeeze`, `dieharder::bit_distribution` (×510), `dieharder::byte_distribution`, `dieharder::dct`, `dieharder::gcd_distribution`, `dieharder::gcd_step_counts`, `dieharder::ks_uniform`, `dieharder::lagged_sums` (×2), `dieharder::minimum_distance_nd`, `maurer::universal_l06`, `maurer::universal_l08`, `maurer::universal_l09`, `maurer::universal_l10`, `maurer::universal_l12`, `maurer::universal_l16`, `nist::approximate_entropy`, `nist::block_frequency`, `nist::cumulative_sums_backward`, `nist::cumulative_sums_forward`, `nist::frequency`, `nist::longest_run`, `nist::matrix_rank`, `nist::non_overlapping_template` (×148), `nist::overlapping_template`, `nist::runs`, `nist::serial_delta2`, `nist::spectral`, `nist::universal`
- **LCG MINSTD (seed=1)**: 692/714 — `diehard::binary_rank_32x32`, `diehard::bitstream`, `diehard::count_ones_stream`, `diehard::dna`, `diehard::minimum_distance_2d`, `diehard::parking_lot`, `diehard::spheres_3d`, `diehard::squeeze`, `dieharder::bit_distribution` (×510), `dieharder::byte_distribution`, `dieharder::dct`, `dieharder::gcd_step_counts`, `dieharder::ks_uniform`, `dieharder::lagged_sums` (×2), `dieharder::minimum_distance_nd`, `maurer::universal_l06`, `maurer::universal_l07`, `maurer::universal_l08`, `maurer::universal_l09`, `maurer::universal_l10`, `maurer::universal_l11`, `maurer::universal_l12`, `maurer::universal_l13`, `maurer::universal_l14`, `maurer::universal_l15`, `maurer::universal_l16`, `nist::approximate_entropy`, `nist::block_frequency`, `nist::cumulative_sums_backward`, `nist::cumulative_sums_forward`, `nist::frequency`, `nist::longest_run`, `nist::matrix_rank`, `nist::non_overlapping_template` (×144), `nist::overlapping_template`, `nist::runs`, `nist::serial_delta2`, `nist::spectral`, `nist::universal`
- **AES-128-CTR (NIST key)**: 5/738 — `dieharder::bit_distribution` (×3), `nist::non_overlapping_template`, `nist::overlapping_template`
- **Camellia-128-CTR (key=00..0f)**: 6/738 — `dieharder::bit_distribution` (×6)
- **Twofish-128-CTR (key=00..0f)**: 10/738 — `dieharder::bit_distribution` (×6), `nist::non_overlapping_template` (×4)
- **Serpent-128-CTR (key=00..0f)**: 11/738 — `dieharder::bit_distribution` (×5), `dieharder::minimum_distance_nd`, `nist::block_frequency`, `nist::non_overlapping_template` (×4)
- **SM4-CTR (key=00..0f)**: 6/738 — `dieharder::bit_distribution` (×6)
- **Grasshopper-CTR (key=00..1f)**: 4/738 — `dieharder::bit_distribution` (×3), `nist::non_overlapping_template`
- **CAST-128-CTR (key=00..0f)**: 7/738 — `dieharder::bit_distribution` (×6), `nist::non_overlapping_template`
- **SEED-CTR (key=00..0f)**: 6/738 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template` (×2)
- **Rabbit (key=00..0f, iv=00..07)**: 6/738 — `dieharder::bit_distribution` (×3), `maurer::universal_l09`, `nist::non_overlapping_template` (×2)
- **Salsa20 (key=00..1f, nonce=00..07)**: 6/738 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template`, `nist::serial_delta1`
- **Snow3G (key=00..0f, iv=00..0f)**: 9/738 — `dieharder::bit_distribution` (×6), `nist::non_overlapping_template` (×3)
- **ZUC-128 (key=00..0f, iv=00..0f)**: 2/738 — `dieharder::bit_distribution` (×2)
- **SpongeBob (SHA3-512 chain, OsRng seed)**: 12/738 — `dieharder::bit_distribution` (×12)
- **Squidward (SHA-256 chain, OsRng seed)**: 7/738 — `dieharder::bit_distribution` (×5), `nist::non_overlapping_template` (×2)
- **PCG32 (OsRng seed)**: 9/738 — `dieharder::bit_distribution` (×7), `dieharder::ks_uniform`, `nist::random_excursions_variant`
- **PCG64 (OsRng seed)**: 12/738 — `dieharder::bit_distribution` (×7), `dieharder::byte_distribution`, `nist::non_overlapping_template` (×4)
- **Xoshiro256** (OsRng seed)**: 7/738 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template` (×3)
- **Xoroshiro128** (OsRng seed)**: 6/738 — `dieharder::bit_distribution` (×5), `dieharder::fill_tree_count`
- **WyRand (OsRng seed)**: 6/714 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template` (×2)
- **SFC64 (OsRng seed)**: 10/738 — `dieharder::bit_distribution` (×8), `nist::non_overlapping_template` (×2)
- **JSF64 (OsRng seed)**: 5/738 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template`
- **ChaCha20 CSPRNG (OsRng key)**: 10/738 — `dieharder::bit_distribution` (×4), `dieharder::gcd_step_counts`, `nist::non_overlapping_template` (×5)
- **HMAC_DRBG SHA-256 (OsRng seed)**: 3/738 — `dieharder::bit_distribution` (×2), `maurer::universal_l07`
- **Hash_DRBG SHA-256 (OsRng seed)**: 8/738 — `diehard::craps_wins`, `dieharder::bit_distribution` (×2), `nist::non_overlapping_template` (×5)
- **cryptography::CtrDrbgAes256 (seed=00..2f)**: 8/714 — `dieharder::bit_distribution` (×3), `nist::non_overlapping_template` (×5)
- **Constant (0xDEAD_DEAD)**: 711/714 — expected for degenerate generator.
- **Counter (0,1,2,…)**: 710/714 — expected for degenerate generator.
- **Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01)**: 3/199 — `maurer::universal_l15`, `nist::non_overlapping_template` (×2)

## Bottom Line

- Degenerate generators (Constant, Counter) and legacy PRNGs (ANSI C LCG, MINSTD, VB6 Rnd) remain annihilated — the battery continues to distinguish garbage from structure.
- Among non-trivial generators, the lowest FAIL count is **2** (`ZUC-128 (key=00..0f, iv=00..0f)`) and the highest is **12** (`Xorshift32 (seed=1)`).
- Isolated failures in `non_overlapping_template` and `bit_distribution` are expected at α = 0.01; they are noise unless they form a family cluster.

## Auxiliary Probes

Run on a representative subset of generators via `tests/run_all.sh` on `darby.local`
(2026-03-16).  Full output: [logs/run_all-darby-20260316-192756.log](logs/run_all-darby-20260316-192756.log)

Columns show PASS / total scored tests for each probe suite.
**Knuth** = permutation + gap + runs (3 tests).
**Hamming+FPF** = TestU01 HammingCorr + HammingIndep×2 + PractRand FPF cross + platter×8 (12 tests).
**LZ** = TestU01 lzw\_sum + lzw\_ks (2 tests).
**SAC** = Webster–Tavares strict avalanche criterion mean deviation from 0.5 (informational; 0 = ideal, 0.5 = worst).
Gorilla was still running when the log was captured.

| Generator | Knuth (3) | Hamming+FPF (12) | LZ (2) | SAC mean |
|---|---:|---:|---:|---:|
| MT19937 | 3/3 | 12/12 | 2/2 | 0.007 |
| Xorshift32 | 3/3 | 12/12 | 2/2 | 0.500 |
| Xorshift64 | 3/3 | 12/12 | 2/2 | 0.500 |
| BAD Unix System V rand() | 3/3 | 12/12 | 2/2 | 0.333 |
| BAD Unix System V mrand48() | 3/3 | 12/12 | 2/2 | 0.378 |
| BAD Unix BSD random() | 3/3 | 12/12 | 2/2 | 0.043 |
| BAD Unix Linux glibc rand()/random() | 3/3 | 12/12 | 2/2 | 0.043 |
| BAD Windows CRT rand() | 3/3 | 11/12 | 2/2 | 0.331 |
| BAD Windows VB6/VBA Rnd() | 3/3 | 0/12 | 0/2 | 0.417 |
| BAD Windows .NET Random(seed) | 3/3 | 12/12 | 2/2 | 0.235 |
| ANSI C sample LCG | 1/3 | 6/12 | 0/2 | 0.362 |
| LCG MINSTD | 2/3 | 9/12 | 0/2 | 0.246 |
| AES-128-CTR | 3/3 | 12/12 | 2/2 | 0.006 |
| cryptography::CtrDrbgAes256 | 3/3 | 12/12 | 2/2 | 0.006 |
