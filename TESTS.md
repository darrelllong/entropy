# Full Battery Results

Full `run_tests` battery harvested from `tolkien` (Apple M1, 4P+4E cores) on 2026-07-22.

Sample size: **16 Mbit** per generator for NIST; DIEHARD/DIEHARDER
consume **16 M 32-bit words** (plus what the live-drawing tests take
directly).

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

The battery has **739 test slots** at this sample size:

- **739 results** — the "full active battery" outcome: the signed-random-walk
  tests (`random_excursions` and `random_excursions_variant`) completed
  successfully (J ≥ 500 zero-crossing cycles).  At 16 Mbit the expected
  cycle count is J ≈ 3191 (= √(2n/π)),
  which is comfortably above the threshold for well-behaved generators.

- **715 results** — 24 fewer slots than the full battery.  The excursion
  families normally emit 8 + 18 = 26 individual per-state results; when the
  signed random walk produces fewer than J = 500 complete zero-crossing cycles
  both families are each collapsed to a single family-level SKIP entry,
  yielding 26 − 2 = 24 fewer slots.  Degenerate generators (Constant,
  Counter, ANSI C LCG, MINSTD) always land here; a handful of non-degenerate
  generators can too, depending on their random seed.

- **200 results** — `Dual_EC_DRBG` only: two P-256 scalar multiplications per
  30-byte output block makes DIEHARD and DIEHARDER prohibitively slow, so only
  the NIST SP 800-22 suite is run.

**Expected false positives.**  At α = 0.01, a perfect generator should fail
roughly 1% of tests by chance.  With 715–739 active tests, the expected
false-fail count is approximately 7.  Isolated failures below that threshold
are noise, not structure.

## Summary Table

| RNG | Total | PASS | FAIL | SKIP |
|---|---:|---:|---:|---:|
| OsRng (/dev/urandom) | 739 | 733 | 6 | 0 |
| MT19937 (seed=19650218) | 739 | 729 | 10 | 0 |
| Xorshift64 (seed=1) | 739 | 734 | 5 | 0 |
| Xorshift32 (seed=1) | 739 | 726 | 13 | 0 |
| BAD Unix System V rand() (15-bit LCG, seed=1) | 739 | 727 | 12 | 0 |
| BAD Unix System V mrand48() (seed=1) | 739 | 732 | 7 | 0 |
| BAD Unix BSD random() TYPE_3 (seed=1) | 739 | 728 | 11 | 0 |
| BAD Unix Linux glibc rand()/random() (seed=1) | 739 | 728 | 11 | 0 |
| BAD Unix FreeBSD12 rand_r() compat (seed=1) | 739 | 729 | 10 | 0 |
| BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1) | 739 | 729 | 10 | 0 |
| BAD Windows VB6/VBA Rnd() (project seed=1) | 739 | 204 | 535 | 0 |
| BAD Windows .NET Random(seed=1) compat | 739 | 732 | 7 | 0 |
| ANSI C sample LCG (1103515245,12345; seed=1) | 715 | 15 | 698 | 2 |
| LCG MINSTD (seed=1) | 715 | 20 | 693 | 2 |
| BAD Borland C++ rand() LCG (seed=1) | 739 | 726 | 13 | 0 |
| AES-128-CTR (NIST key) | 739 | 734 | 5 | 0 |
| Camellia-128-CTR (key=00..0f) | 739 | 732 | 7 | 0 |
| Twofish-128-CTR (key=00..0f) | 739 | 729 | 10 | 0 |
| Serpent-128-CTR (key=00..0f) | 739 | 728 | 11 | 0 |
| SM4-CTR (key=00..0f) | 739 | 732 | 7 | 0 |
| Grasshopper-CTR (key=00..1f) | 739 | 735 | 4 | 0 |
| CAST-128-CTR (key=00..0f) | 739 | 732 | 7 | 0 |
| SEED-CTR (key=00..0f) | 739 | 733 | 6 | 0 |
| Rabbit (key=00..0f, iv=00..07) | 739 | 733 | 6 | 0 |
| Salsa20 (key=00..1f, nonce=00..07) | 739 | 732 | 7 | 0 |
| Snow3G (key=00..0f, iv=00..0f) | 739 | 730 | 9 | 0 |
| ZUC-128 (key=00..0f, iv=00..0f) | 739 | 737 | 2 | 0 |
| SpongeBob (SHA3-512 chain, OsRng seed) | 739 | 729 | 10 | 0 |
| Squidward (SHA-256 chain, OsRng seed) | 739 | 735 | 4 | 0 |
| PCG32 (OsRng seed) | 739 | 735 | 4 | 0 |
| PCG64 (OsRng seed) | 739 | 732 | 7 | 0 |
| Xoshiro256 (OsRng seed) | 739 | 732 | 7 | 0 |
| Xoroshiro128 (OsRng seed) | 739 | 730 | 9 | 0 |
| WyRand (OsRng seed) | 739 | 734 | 5 | 0 |
| SFC64 (OsRng seed) | 739 | 735 | 4 | 0 |
| JSF64 (OsRng seed) | 715 | 701 | 12 | 2 |
| ChaCha20 CSPRNG (OsRng key) | 739 | 736 | 3 | 0 |
| HMAC_DRBG SHA-256 (OsRng seed) | 715 | 707 | 6 | 2 |
| Hash_DRBG SHA-256 (OsRng seed) | 739 | 730 | 9 | 0 |
| cryptography::CtrDrbgAes256 (seed=00..2f) | 715 | 705 | 8 | 2 |
| Constant (0xDEAD_DEAD) | 715 | 0 | 713 | 2 |
| Counter (0,1,2,…) | 715 | 1 | 712 | 2 |
| Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01) | 200 | 197 | 3 | 0 |

## Theory By Test

Here $Q(a,x)=\Gamma(a,x)/\Gamma(a)$ is the upper regularized gamma function,
$\Phi$ is the standard normal CDF, and $D_n=\sup_x |F_n(x)-x|$ is the
one-sample Kolmogorov-Smirnov statistic used whenever this report says
"outer KS on p-values."  The KS tail probability uses the
Marsaglia-Tsang-Wang exact small-$n$ evaluation ($n \le 4999$) with a
Stephens-corrected asymptotic series beyond.  (July 2026 fixes corrected an
H-matrix bug that had collapsed exact-branch p-values toward 1 and an
asymptotic-series non-convergence misroute that reported p = 1 for
catastrophically large $D$; the tables above were harvested with the
corrected code.)

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
  multinomial chi-square.  The implementation requires at least $N=72$
  blocks (expected cell counts ≥ 5 in the rarest bin).

- **`universal` and `maurer::universal_l05..l16`**. Maurer's universal test
  measures compressibility by tracking recurrence gaps of $L$-bit words. If
  $A_i$ is the distance back to the previous occurrence of the current word,
  the core statistic is
  $f_n = \frac{1}{K}\sum_{i=1}^K \log_2 A_i$,
  which is normalized as
  $z = (f_n-\mu_L)/(c(L,K)\sigma_L)$ and scored with
  $p=\mathrm{erfc}(|z|/\sqrt{2})$. The crate reports both the NIST
  wrapper (which selects $L$ from $n$ over NIST's domain $L\in[6,16]$)
  and the broader Maurer parametric family over $L=5,\dots,16$.

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
  $\Delta^2\psi_m^2=\psi_m^2-2\psi_{m-1}^2+\psi_{m-2}^2$.
  Per §2.11.4, `serial_delta1` carries $\Delta\psi_m^2 \sim \chi^2(2^{m-1})$
  scored as $Q(2^{m-2},\Delta\psi_m^2/2)$ and `serial_delta2` carries
  $\Delta^2\psi_m^2 \sim \chi^2(2^{m-2})$ scored as
  $Q(2^{m-3},\Delta^2\psi_m^2/2)$; the implementation enforces
  $m < \lfloor\log_2 n\rfloor - 2$.  (A July 2026 fix corrected a
  cross-wired df/statistic pairing that had doubled the degrees of freedom;
  the tables above were harvested with the corrected pairing.)

- **`approximate_entropy`.** For pattern lengths $m=10$ and $m+1$, define
  $\phi_m=\frac{1}{n}\sum_i \log C_i^{(m)}$, where $C_i^{(m)}$ is the
  circular pattern-match frequency of the $i$th overlapping block. The test
  statistic is
  $\mathrm{ApEn}(m)=\phi_m-\phi_{m+1}$ and NIST scores
  $\chi^2 = 2n(\ln 2-\mathrm{ApEn}(m))$.  The implementation enforces
  $m < \log_2 n - 5$ per the SP 800-22 validity condition.

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
  $p = Q(5/2,\chi_x^2/2)$, where $J$ is the number of cycles.  Both
  excursion families require $J \ge \max(0.005\sqrt{n}, 500)$ per
  §2.14.4/§2.15.4,
  and a walk ending exactly at zero no longer contributes a spurious empty
  trailing cycle.

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

- **`opso`.** OPSO builds each $20$-bit sample from two $10$-bit letters
  taken from two separate words, so each sample lands in a word space of
  size $2^{20}$.  The crate deliberately deviates from the canonical
  overlapping construction: the $2^{21}$ samples use *disjoint* bit fields,
  so they are mutually independent and the missing-words count is scored
  with the exact iid moments $\mu \approx 141{,}909.19$,
  $\sigma \approx 290.33$ (the canonical overlapping OPSO $\sigma$ is 290),
  giving $p=\mathrm{erfc}(|z|/\sqrt{2})$.

- **`oqso`.** OQSO is the same sparse-occupancy idea with one $5$-bit letter
  from each of four words per sample, again a $2^{20}$ word space.  As with
  OPSO the samples come from disjoint bit fields, so the same exact iid
  moments ($\mu \approx 141{,}909.19$, $\sigma \approx 290.33$) apply — a
  documented deviation from the canonical overlapping OQSO $\sigma = 295$.

- **`dna`.** DNA applies the sparse-occupancy construction with one $2$-bit
  symbol from each of ten words per sample, another $2^{20}$-word occupancy
  problem.  Samples are again disjoint/iid and scored with
  $\mu \approx 141{,}909.19$, $\sigma \approx 290.33$ (canonical overlapping
  DNA uses $\sigma = 339$), reporting $p=\mathrm{erfc}(|z|/\sqrt{2})$.

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

- **`squeeze`.** Start from $k_0 = 2^{31}$ and iterate
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
  law $\mathrm{Bin}(32b,\tfrac12)$ by chi-square.  The crate keeps the most
  extreme per-block-size p-value, but — a documented deviation from
  dieharder's `evalMostExtreme()`, which maps low-side extremes to $p\approx1$
  — each per-block p-value is first folded two-sided
  ($2\min(p,1-p)$) and the extreme is then Šidák-corrected, so both failure
  directions map to small p while null uniformity is preserved.

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

These nine probes live in `src/research/` and run via `tests/run_aux.sh`,
which builds and executes the five auxiliary binaries (`bib_tests`,
`upstream_tests`, `testu01_lz`, `webster_tavares`, `gorilla`) with their
default parameters.  A sixth standalone binary, `bitplane_complexity`,
measures Berlekamp-Massey linear complexity on each individual output bit
plane across successive 64-bit outputs; it is not part of `run_aux.sh`.
The probes target structural properties that the standard batteries
underweight.

- **Knuth permutation test** (TAOCP §3.3.2).  Draw non-overlapping windows of
  $t$ successive output words ($t = 5$ in the auxiliary run), rank each
  window to obtain its permutation ordinal in $S_t$, and accumulate counts over
  the $t!$ orderings.  The null distribution is uniform, so the test statistic
  is $\chi^2 = \sum_{i=0}^{t!-1} (O_i - N/t!)^2/(N/t!)$ on $t!-1$ degrees of
  freedom.  Cochran's rule is enforced: the test requires at least $5\,t!$
  blocks so every cell has expected count ≥ 5.  Sensitive to short-range
  ordering biases that survive frequency tests.

- **Knuth gap test** (TAOCP §3.3.2).  Fix an interval $[\alpha,\beta)$ and
  record the lengths $L$ of the gaps (runs of words outside the interval)
  between successive hits.  Under the null, $L$ follows a geometric distribution
  with success probability $p = \beta - \alpha$.  The observed gap-length
  histogram is compared to this geometric law by chi-square after Cochran-rule
  tail merging: trailing cells whose expected count falls below 5 are pooled
  into a single tail cell, and the test runs on the surviving `cells` with
  $df = \mathrm{cells} - 1$.  Tests uniformity of the real-valued projection
  and independence of successive words.

- **Knuth runs test** (Wald–Wolfowitz runs above/below the median).  Compute
  the sample median, classify each output value as above or below it
  (values equal to the median are dropped), and count the runs $R$ of
  consecutive same-side values.  Conditional on $n_1$ values above and $n_2$
  below ($n = n_1 + n_2$), the null moments are
  $\mu = 1 + 2n_1 n_2/n$ and
  $\sigma^2 = 2n_1 n_2(2n_1 n_2 - n)/(n^2(n-1))$.  The crate reports
  $z = (R - \mu)/\sigma$ and the two-tailed p-value
  $p = \mathrm{erfc}(|z|/\sqrt{2})$.  Detects clustering (too few runs) and
  over-alternation (too many runs) around the sample median.

- **PractRand FPF** (floating-point frequency).  Parse the LSB-first bitstream
  into FPF codewords: a run of zeros terminated by a stop bit gives the
  geometric exponent (capped at $2^{\mathrm{exp\_bits}}-1$), followed by
  `sig_bits` of significand (defaults: `sig_bits` = 14, `exp_bits` = 6).
  Per exponent "platter," the significand histogram is scored with
  PractRand's intra-platter truncation rule and a G-test (log-likelihood
  chi-square); a separate grouped exponent-distribution G-test (`:cross`)
  checks the geometric law across platters.  Documented deviation from
  upstream: PractRand slides its sample window a fixed 16-bit stride, so
  successive samples share bits — a dependence its empirical calibration
  tables absorb.  This port has no calibration tables, so it parses
  *disjoint* codewords instead: samples are iid, the quoted asymptotic
  chi-square/G law is actually valid, and mean consumption (≈ 16 bits per
  sample at `sig_bits` = 14) matches upstream's stride.  Suspicion scores
  are not reproduced.

- **TestU01 Lempel–Ziv** (`scomp_LempelZiv`).  Parse the bit stream as an
  LZ78 dictionary: each new phrase extends the longest previously seen prefix by
  one bit.  Let $C_n$ be the number of distinct phrases after reading $n$ bits.
  Asymptotically, $C_n / (n / \log_2 n) \to 1$ for a fair coin.  The crate uses
  empirical tables of $(\mu, \sigma)$ for $n = 2^k$ taken from the TestU01
  source to compute a z-score and derives the p-value from the normal
  approximation.  An outer KS over multiple replications converts per-replication
  z-scores to a single battery p-value.  Low complexity (few phrases) flags
  repetitive structure; high complexity flags over-dispersion.

- **TestU01 Hamming** (`sstring_HammingCorr` / `sstring_HammingIndep`).
  `HammingCorr`: Extract $n$ successive $L$-bit blocks from the bit stream
  (TestU01 `unif01_StripB`-style extraction; the auxiliary run uses a single
  block length, $L = 300$ by default) and compute each block's Hamming weight
  $W_i$.  The statistic is the lag-1 correlation of successive centered
  weights,
  $\hat\rho = 4\sum_{i}(W_i - L/2)(W_{i+1} - L/2)\,/\,((n-1)L)$,
  standardized as $z = \hat\rho\sqrt{n-1}$ and scored two-sided with
  $p = \mathrm{erfc}(|z|/\sqrt{2})$.
  `HammingIndep`: For successive pairs of $L$-bit blocks $(X, Y)$, compute the
  joint weight histogram $(W_X, W_Y)$ and compare to the product distribution
  $\mathrm{Bin}(L,\tfrac12)\times\mathrm{Bin}(L,\tfrac12)$ by chi-square with
  TestU01's `gofs_MinExpected = 10` cell lumping.  Detects linear
  dependencies across block boundaries.

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

- **Multi-scale approximate entropy (ApEn)**.  A sweep of the NIST SP 800-22
  §2.12 bit-level ApEn statistic over embedding dimensions $m = 2,\dots,6$
  (the single NIST test fixes one $m$).  For each $m$, compute
  $\phi(m) = \frac1n \sum_p C_p \ln(C_p/n)$ over the circular overlapping
  $m$-bit pattern counts $C_p$ and report
  $\mathrm{ApEn}(m) = \phi(m) - \phi(m+1)$.  A value near $\ln 2$ indicates
  randomness at that pattern length; the profile reveals at which scale a
  structured generator's internal period or dependency length becomes
  apparent.  (This is the bit-pattern ApEn of SP 800-22, not Pincus's
  real-valued $r$-tolerance $\mathrm{ApEn}(m,r,N)$.)

## Failure Highlights

One line per generator.  Test-family repetition counts in parentheses.

- **OsRng (/dev/urandom)**: 6/739 — `diehard::runs_down`, `diehard::spheres_3d`, `dieharder::bit_distribution` (×4)
- **MT19937 (seed=19650218)**: 10/739 — `dieharder::bit_distribution` (×8), `nist::non_overlapping_template` (×2)
- **Xorshift64 (seed=1)**: 5/739 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template`
- **Xorshift32 (seed=1)**: 13/739 — `diehard::binary_rank_31x31`, `diehard::binary_rank_32x32`, `dieharder::bit_distribution` (×9), `dieharder::monobit2`, `nist::matrix_rank`
- **BAD Unix System V rand() (15-bit LCG, seed=1)**: 12/739 — `diehard::opso`, `dieharder::bit_distribution` (×8), `nist::non_overlapping_template` (×2), `nist::spectral`
- **BAD Unix System V mrand48() (seed=1)**: 7/739 — `diehard::dna`, `diehard::opso`, `diehard::oqso`, `dieharder::bit_distribution` (×2), `nist::non_overlapping_template` (×2)
- **BAD Unix BSD random() TYPE_3 (seed=1)**: 11/739 — `dieharder::bit_distribution` (×5), `nist::non_overlapping_template` (×5), `nist::serial_delta2`
- **BAD Unix Linux glibc rand()/random() (seed=1)**: 11/739 — `dieharder::bit_distribution` (×5), `nist::non_overlapping_template` (×5), `nist::serial_delta2`
- **BAD Unix FreeBSD12 rand_r() compat (seed=1)**: 10/739 — `dieharder::bit_distribution` (×7), `nist::non_overlapping_template` (×2), `nist::spectral`
- **BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1)**: 10/739 — `dieharder::bit_distribution` (×7), `dieharder::dct`, `nist::random_excursions`, `nist::spectral`
- **BAD Windows VB6/VBA Rnd() (project seed=1)**: 535/739 — `diehard::binary_rank_6x8`, `diehard::birthday_spacings`, `diehard::bitstream`, `diehard::count_ones_stream`, `diehard::craps_throws`, `diehard::dna`, `diehard::minimum_distance_2d`, `diehard::opso`, `diehard::oqso`, `diehard::parking_lot`, `diehard::runs_up`, `diehard::spheres_3d`, `diehard::squeeze`, `dieharder::bit_distribution` (×497), `dieharder::byte_distribution`, `dieharder::dct`, `dieharder::fill_tree_count`, `dieharder::fill_tree_position`, `dieharder::gcd_distribution`, `dieharder::gcd_step_counts`, `dieharder::ks_uniform`, `dieharder::lagged_sums`, `dieharder::minimum_distance_nd`, `dieharder::monobit2`, `maurer::universal_l05`, `maurer::universal_l06`, `maurer::universal_l07`, `maurer::universal_l08`, `maurer::universal_l09`, `maurer::universal_l10`, `maurer::universal_l11`, `maurer::universal_l12`, `maurer::universal_l13`, `maurer::universal_l14`, `maurer::universal_l15`, `maurer::universal_l16`, `nist::overlapping_template`, `nist::spectral`, `nist::universal`
- **BAD Windows .NET Random(seed=1) compat**: 7/739 — `dieharder::bit_distribution` (×3), `dieharder::dct`, `nist::non_overlapping_template` (×3)
- **ANSI C sample LCG (1103515245,12345; seed=1)**: 698/715 — `diehard::binary_rank_32x32`, `diehard::binary_rank_6x8`, `diehard::bitstream`, `diehard::count_ones_stream`, `diehard::craps_throws`, `diehard::craps_wins`, `diehard::dna`, `diehard::minimum_distance_2d`, `diehard::opso`, `diehard::oqso`, `diehard::parking_lot`, `diehard::spheres_3d`, `diehard::squeeze`, `dieharder::bit_distribution` (×510), `dieharder::byte_distribution`, `dieharder::dct`, `dieharder::gcd_distribution`, `dieharder::gcd_step_counts`, `dieharder::ks_uniform`, `dieharder::lagged_sums` (×2), `dieharder::minimum_distance_nd`, `dieharder::monobit2`, `maurer::universal_l06`, `maurer::universal_l08`, `maurer::universal_l09`, `maurer::universal_l10`, `maurer::universal_l12`, `maurer::universal_l16`, `nist::approximate_entropy`, `nist::block_frequency`, `nist::cumulative_sums_backward`, `nist::cumulative_sums_forward`, `nist::frequency`, `nist::longest_run`, `nist::matrix_rank`, `nist::non_overlapping_template` (×148), `nist::overlapping_template`, `nist::runs`, `nist::serial_delta1`, `nist::spectral`, `nist::universal`
- **LCG MINSTD (seed=1)**: 693/715 — `diehard::binary_rank_32x32`, `diehard::bitstream`, `diehard::count_ones_stream`, `diehard::dna`, `diehard::minimum_distance_2d`, `diehard::parking_lot`, `diehard::spheres_3d`, `diehard::squeeze`, `dieharder::bit_distribution` (×510), `dieharder::byte_distribution`, `dieharder::dct`, `dieharder::gcd_step_counts`, `dieharder::ks_uniform`, `dieharder::lagged_sums` (×2), `dieharder::minimum_distance_nd`, `dieharder::monobit2`, `maurer::universal_l06`, `maurer::universal_l07`, `maurer::universal_l08`, `maurer::universal_l09`, `maurer::universal_l10`, `maurer::universal_l11`, `maurer::universal_l12`, `maurer::universal_l13`, `maurer::universal_l14`, `maurer::universal_l15`, `maurer::universal_l16`, `nist::approximate_entropy`, `nist::block_frequency`, `nist::cumulative_sums_backward`, `nist::cumulative_sums_forward`, `nist::frequency`, `nist::longest_run`, `nist::matrix_rank`, `nist::non_overlapping_template` (×144), `nist::overlapping_template`, `nist::runs`, `nist::serial_delta1`, `nist::spectral`, `nist::universal`
- **BAD Borland C++ rand() LCG (seed=1)**: 13/739 — `diehard::opso`, `dieharder::bit_distribution` (×9), `dieharder::minimum_distance_nd`, `nist::non_overlapping_template`, `nist::spectral`
- **AES-128-CTR (NIST key)**: 5/739 — `dieharder::bit_distribution` (×3), `nist::non_overlapping_template`, `nist::overlapping_template`
- **Camellia-128-CTR (key=00..0f)**: 7/739 — `diehard::runs_up`, `dieharder::bit_distribution` (×6)
- **Twofish-128-CTR (key=00..0f)**: 10/739 — `dieharder::bit_distribution` (×6), `nist::non_overlapping_template` (×4)
- **Serpent-128-CTR (key=00..0f)**: 11/739 — `dieharder::bit_distribution` (×5), `dieharder::minimum_distance_nd`, `nist::block_frequency`, `nist::non_overlapping_template` (×4)
- **SM4-CTR (key=00..0f)**: 7/739 — `dieharder::bit_distribution` (×6), `nist::serial_delta2`
- **Grasshopper-CTR (key=00..1f)**: 4/739 — `dieharder::bit_distribution` (×3), `nist::non_overlapping_template`
- **CAST-128-CTR (key=00..0f)**: 7/739 — `dieharder::bit_distribution` (×6), `nist::non_overlapping_template`
- **SEED-CTR (key=00..0f)**: 6/739 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template` (×2)
- **Rabbit (key=00..0f, iv=00..07)**: 6/739 — `dieharder::bit_distribution` (×3), `maurer::universal_l09`, `nist::non_overlapping_template` (×2)
- **Salsa20 (key=00..1f, nonce=00..07)**: 7/739 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template`, `nist::serial_delta1`, `nist::serial_delta2`
- **Snow3G (key=00..0f, iv=00..0f)**: 9/739 — `dieharder::bit_distribution` (×6), `nist::non_overlapping_template` (×3)
- **ZUC-128 (key=00..0f, iv=00..0f)**: 2/739 — `dieharder::bit_distribution` (×2)
- **SpongeBob (SHA3-512 chain, OsRng seed)**: 10/739 — `dieharder::bit_distribution` (×9), `nist::non_overlapping_template`
- **Squidward (SHA-256 chain, OsRng seed)**: 4/739 — `dieharder::bit_distribution` (×2), `nist::non_overlapping_template` (×2)
- **PCG32 (OsRng seed)**: 4/739 — `dieharder::bit_distribution` (×2), `nist::non_overlapping_template` (×2)
- **PCG64 (OsRng seed)**: 7/739 — `dieharder::bit_distribution` (×5), `nist::non_overlapping_template` (×2)
- **Xoshiro256 (OsRng seed)**: 7/739 — `dieharder::bit_distribution` (×6), `nist::non_overlapping_template`
- **Xoroshiro128 (OsRng seed)**: 9/739 — `dieharder::bit_distribution` (×6), `nist::non_overlapping_template` (×3)
- **WyRand (OsRng seed)**: 5/739 — `dieharder::bit_distribution` (×3), `nist::non_overlapping_template` (×2)
- **SFC64 (OsRng seed)**: 4/739 — `dieharder::bit_distribution` (×3), `nist::non_overlapping_template`
- **JSF64 (OsRng seed)**: 12/715 — `diehard::binary_rank_6x8`, `dieharder::bit_distribution` (×11)
- **ChaCha20 CSPRNG (OsRng key)**: 3/739 — `dieharder::bit_distribution` (×3)
- **HMAC_DRBG SHA-256 (OsRng seed)**: 6/715 — `dieharder::bit_distribution` (×3), `maurer::universal_l16`, `nist::non_overlapping_template` (×2)
- **Hash_DRBG SHA-256 (OsRng seed)**: 9/739 — `dieharder::bit_distribution` (×8), `nist::linear_complexity`
- **cryptography::CtrDrbgAes256 (seed=00..2f)**: 8/715 — `dieharder::bit_distribution` (×3), `nist::non_overlapping_template` (×5)
- **Constant (0xDEAD_DEAD)**: 713/715 — expected for degenerate generator.
- **Counter (0,1,2,…)**: 712/715 — expected for degenerate generator.
- **Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01)**: 3/200 — `maurer::universal_l15`, `nist::non_overlapping_template` (×2)

## Bottom Line

- Degenerate generators (Constant, Counter) and legacy PRNGs (ANSI C LCG, MINSTD, VB6 Rnd) remain annihilated — the battery continues to distinguish garbage from structure.
- Among non-trivial generators, the lowest FAIL count is **2** (`ZUC-128 (key=00..0f, iv=00..0f)`) and the highest is **13** (`Xorshift32 (seed=1)`).
- Isolated failures in `non_overlapping_template` and `bit_distribution` are expected at α = 0.01; they are noise unless they form a family cluster.

## Auxiliary Probes

These probes are not part of `run_tests`; they are recorded separately here
from `tests/run_all.sh` on `tolkien` (2026-07-22).

These probes exercise statistical properties not covered by the NIST/DIEHARD/DIEHARDER
battery.  They run with their default parameters; use the individual binaries for
filtered or resized runs.  All probes exit 0 (no crashes or panics).

```
========================================================================
bib_tests  (Knuth + NIST ApEn profile m=2..6)
========================================================================

MT19937
  [PASS] knuth::permutation                                p = 0.106832  (t=5, blocks=40000, χ²=138.5000, df=119)
  [PASS] knuth::gap                                        p = 0.966310  ([0.250,0.500) gaps=49922, r=15, cells=16, χ²=6.6641, df=15)
  [PASS] knuth::runs_median                                p = 0.582269  (median=0.500112, below=100000, above=100000, runs=100124, z=0.5501)
  [INFO] approx_entropy_m02   ApEn=0.693144 (phi_m=-1.386294, phi_m1=-2.079438)
  [INFO] approx_entropy_m03   ApEn=0.693144 (phi_m=-2.079438, phi_m1=-2.772581)
  [INFO] approx_entropy_m04   ApEn=0.693141 (phi_m=-2.772581, phi_m1=-3.465723)
  [INFO] approx_entropy_m05   ApEn=0.693133 (phi_m=-3.465723, phi_m1=-4.158856)
  [INFO] approx_entropy_m06   ApEn=0.693118 (phi_m=-4.158856, phi_m1=-4.851974)

Xorshift32
  [PASS] knuth::permutation                                p = 0.622050  (t=5, blocks=40000, χ²=113.6180, df=119)
  [PASS] knuth::gap                                        p = 0.237153  ([0.250,0.500) gaps=50065, r=15, cells=16, χ²=18.5028, df=15)
  [PASS] knuth::runs_median                                p = 0.190081  (median=0.499376, below=100000, above=100000, runs=100294, z=1.3103)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693145 (phi_m=-2.079439, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693138 (phi_m=-2.772583, phi_m1=-3.465722)
  [INFO] approx_entropy_m05   ApEn=0.693133 (phi_m=-3.465722, phi_m1=-4.158855)
  [INFO] approx_entropy_m06   ApEn=0.693115 (phi_m=-4.158855, phi_m1=-4.851970)

Xorshift64
  [PASS] knuth::permutation                                p = 0.606131  (t=5, blocks=40000, χ²=114.2420, df=119)
  [PASS] knuth::gap                                        p = 0.750205  ([0.250,0.500) gaps=50116, r=15, cells=16, χ²=11.0337, df=15)
  [PASS] knuth::runs_median                                p = 0.134089  (median=0.499730, below=100000, above=100000, runs=99666, z=-1.4982)
  [INFO] approx_entropy_m02   ApEn=0.693147 (phi_m=-1.386294, phi_m1=-2.079441)
  [INFO] approx_entropy_m03   ApEn=0.693143 (phi_m=-2.079441, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693136 (phi_m=-2.772583, phi_m1=-3.465720)
  [INFO] approx_entropy_m05   ApEn=0.693130 (phi_m=-3.465720, phi_m1=-4.158850)
  [INFO] approx_entropy_m06   ApEn=0.693121 (phi_m=-4.158850, phi_m1=-4.851971)

BAD Unix System V rand()
  [PASS] knuth::permutation                                p = 0.410880  (t=5, blocks=40000, χ²=121.8320, df=119)
  [PASS] knuth::gap                                        p = 0.397280  ([0.250,0.500) gaps=50005, r=15, cells=16, χ²=15.7732, df=15)
  [PASS] knuth::runs_median                                p = 0.576149  (median=0.499845, below=100000, above=100000, runs=99876, z=-0.5590)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079440)
  [INFO] approx_entropy_m03   ApEn=0.693145 (phi_m=-2.079440, phi_m1=-2.772584)
  [INFO] approx_entropy_m04   ApEn=0.693137 (phi_m=-2.772584, phi_m1=-3.465721)
  [INFO] approx_entropy_m05   ApEn=0.693129 (phi_m=-3.465721, phi_m1=-4.158850)
  [INFO] approx_entropy_m06   ApEn=0.693108 (phi_m=-4.158850, phi_m1=-4.851957)

BAD Unix System V mrand48()
  [PASS] knuth::permutation                                p = 0.026338  (t=5, blocks=40000, χ²=150.6800, df=119)
  [PASS] knuth::gap                                        p = 0.887223  ([0.250,0.500) gaps=49804, r=15, cells=16, χ²=8.8104, df=15)
  [PASS] knuth::runs_median                                p = 0.582269  (median=0.501971, below=100000, above=100000, runs=100124, z=0.5501)
  [INFO] approx_entropy_m02   ApEn=0.693145 (phi_m=-1.386294, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693144 (phi_m=-2.079439, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693135 (phi_m=-2.772583, phi_m1=-3.465718)
  [INFO] approx_entropy_m05   ApEn=0.693130 (phi_m=-3.465718, phi_m1=-4.158848)
  [INFO] approx_entropy_m06   ApEn=0.693115 (phi_m=-4.158848, phi_m1=-4.851964)

BAD Unix BSD random()
  [PASS] knuth::permutation                                p = 0.801062  (t=5, blocks=40000, χ²=105.8060, df=119)
  [PASS] knuth::gap                                        p = 0.823636  ([0.250,0.500) gaps=50051, r=15, cells=16, χ²=9.9378, df=15)
  [PASS] knuth::runs_median                                p = 0.255988  (median=0.499583, below=100000, above=100000, runs=100255, z=1.1359)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386294, phi_m1=-2.079441)
  [INFO] approx_entropy_m03   ApEn=0.693146 (phi_m=-2.079441, phi_m1=-2.772587)
  [INFO] approx_entropy_m04   ApEn=0.693143 (phi_m=-2.772587, phi_m1=-3.465730)
  [INFO] approx_entropy_m05   ApEn=0.693137 (phi_m=-3.465730, phi_m1=-4.158866)
  [INFO] approx_entropy_m06   ApEn=0.693119 (phi_m=-4.158866, phi_m1=-4.851985)

BAD Unix Linux glibc rand()/random()
  [PASS] knuth::permutation                                p = 0.801062  (t=5, blocks=40000, χ²=105.8060, df=119)
  [PASS] knuth::gap                                        p = 0.823636  ([0.250,0.500) gaps=50051, r=15, cells=16, χ²=9.9378, df=15)
  [PASS] knuth::runs_median                                p = 0.255988  (median=0.499583, below=100000, above=100000, runs=100255, z=1.1359)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386294, phi_m1=-2.079441)
  [INFO] approx_entropy_m03   ApEn=0.693146 (phi_m=-2.079441, phi_m1=-2.772587)
  [INFO] approx_entropy_m04   ApEn=0.693143 (phi_m=-2.772587, phi_m1=-3.465730)
  [INFO] approx_entropy_m05   ApEn=0.693137 (phi_m=-3.465730, phi_m1=-4.158866)
  [INFO] approx_entropy_m06   ApEn=0.693119 (phi_m=-4.158866, phi_m1=-4.851985)

BAD Windows CRT rand()
  [PASS] knuth::permutation                                p = 0.086299  (t=5, blocks=40000, χ²=140.5640, df=119)
  [PASS] knuth::gap                                        p = 0.170906  ([0.250,0.500) gaps=50111, r=15, cells=16, χ²=20.0268, df=15)
  [PASS] knuth::runs_median                                p = 0.208872  (median=0.500955, below=100000, above=100000, runs=99720, z=-1.2567)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693142 (phi_m=-2.079439, phi_m1=-2.772581)
  [INFO] approx_entropy_m04   ApEn=0.693132 (phi_m=-2.772581, phi_m1=-3.465713)
  [INFO] approx_entropy_m05   ApEn=0.693124 (phi_m=-3.465713, phi_m1=-4.158837)
  [INFO] approx_entropy_m06   ApEn=0.693098 (phi_m=-4.158837, phi_m1=-4.851935)

BAD Windows VB6/VBA Rnd()
  [PASS] knuth::permutation                                p = 0.668822  (t=5, blocks=40000, χ²=111.7460, df=119)
  [PASS] knuth::gap                                        p = 0.114820  ([0.250,0.500) gaps=49910, r=15, cells=16, χ²=21.7394, df=15)
  [PASS] knuth::runs_median                                p = 0.431225  (median=0.498848, below=100000, above=100000, runs=100177, z=0.7871)
  [INFO] approx_entropy_m02   ApEn=0.693147 (phi_m=-1.386294, phi_m1=-2.079440)
  [INFO] approx_entropy_m03   ApEn=0.693146 (phi_m=-2.079440, phi_m1=-2.772586)
  [INFO] approx_entropy_m04   ApEn=0.693140 (phi_m=-2.772586, phi_m1=-3.465726)
  [INFO] approx_entropy_m05   ApEn=0.693136 (phi_m=-3.465726, phi_m1=-4.158862)
  [INFO] approx_entropy_m06   ApEn=0.693124 (phi_m=-4.158862, phi_m1=-4.851986)

BAD Windows .NET Random(seed)
  [PASS] knuth::permutation                                p = 0.634953  (t=5, blocks=40000, χ²=113.1080, df=119)
  [PASS] knuth::gap                                        p = 0.906751  ([0.250,0.500) gaps=50074, r=15, cells=16, χ²=8.3999, df=15)
  [PASS] knuth::runs_median                                p = 0.310021  (median=0.498075, below=100000, above=100000, runs=100228, z=1.0152)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079440)
  [INFO] approx_entropy_m03   ApEn=0.693144 (phi_m=-2.079440, phi_m1=-2.772584)
  [INFO] approx_entropy_m04   ApEn=0.693140 (phi_m=-2.772584, phi_m1=-3.465724)
  [INFO] approx_entropy_m05   ApEn=0.693134 (phi_m=-3.465724, phi_m1=-4.158858)
  [INFO] approx_entropy_m06   ApEn=0.693114 (phi_m=-4.158858, phi_m1=-4.851972)

ANSI C sample LCG
  [PASS] knuth::permutation                                p = 0.861091  (t=5, blocks=40000, χ²=102.4220, df=119)
  [FAIL] knuth::gap                                        p = 0.000000  ([0.250,0.500) gaps=100484, r=15, cells=16, χ²=51401.6062, df=15)
  [FAIL] knuth::runs_median                                p = 0.008891  (median=0.251237, below=100000, above=100000, runs=99416, z=-2.6162)
  [INFO] approx_entropy_m02   ApEn=0.692679 (phi_m=-1.385362, phi_m1=-2.078042)
  [INFO] approx_entropy_m03   ApEn=0.692678 (phi_m=-2.078042, phi_m1=-2.770720)
  [INFO] approx_entropy_m04   ApEn=0.692676 (phi_m=-2.770720, phi_m1=-3.463396)
  [INFO] approx_entropy_m05   ApEn=0.692669 (phi_m=-3.463396, phi_m1=-4.156065)
  [INFO] approx_entropy_m06   ApEn=0.692659 (phi_m=-4.156065, phi_m1=-4.848724)

LCG MINSTD
  [PASS] knuth::permutation                                p = 0.261206  (t=5, blocks=40000, χ²=128.4440, df=119)
  [FAIL] knuth::gap                                        p = 0.000000  ([0.250,0.500) gaps=100358, r=15, cells=16, χ²=50762.1340, df=15)
  [PASS] knuth::runs_median                                p = 0.889737  (median=0.250876, below=100000, above=100000, runs=99970, z=-0.1386)
  [INFO] approx_entropy_m02   ApEn=0.692728 (phi_m=-1.385463, phi_m1=-2.078191)
  [INFO] approx_entropy_m03   ApEn=0.692727 (phi_m=-2.078191, phi_m1=-2.770918)
  [INFO] approx_entropy_m04   ApEn=0.692722 (phi_m=-2.770918, phi_m1=-3.463640)
  [INFO] approx_entropy_m05   ApEn=0.692714 (phi_m=-3.463640, phi_m1=-4.156354)
  [INFO] approx_entropy_m06   ApEn=0.692696 (phi_m=-4.156354, phi_m1=-4.849050)

AES-128-CTR
  [PASS] knuth::permutation                                p = 0.224876  (t=5, blocks=40000, χ²=130.3400, df=119)
  [PASS] knuth::gap                                        p = 0.111037  ([0.250,0.500) gaps=50076, r=15, cells=16, χ²=21.8782, df=15)
  [PASS] knuth::runs_median                                p = 0.508050  (median=0.499150, below=100000, above=100000, runs=100149, z=0.6619)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386294, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693143 (phi_m=-2.079439, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693139 (phi_m=-2.772583, phi_m1=-3.465722)
  [INFO] approx_entropy_m05   ApEn=0.693132 (phi_m=-3.465722, phi_m1=-4.158853)
  [INFO] approx_entropy_m06   ApEn=0.693118 (phi_m=-4.158853, phi_m1=-4.851972)

cryptography::CtrDrbgAes256
  [PASS] knuth::permutation                                p = 0.357595  (t=5, blocks=40000, χ²=124.0340, df=119)
  [PASS] knuth::gap                                        p = 0.760668  ([0.250,0.500) gaps=50101, r=15, cells=16, χ²=10.8855, df=15)
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
  [PASS] practrand::fpf_cross                              p = 0.425415  (samples=8388834, sig_bits=14, max_exp=63, dof=19, chi2=19.4967)
  [PASS] practrand::fpf_platter                            p = 0.686096  (samples=8388834, e=0, sig_bins=2^13, dof=8191, chi2=8128.4406)
  [PASS] practrand::fpf_platter                            p = 0.541074  (samples=8388834, e=1, sig_bins=2^12, dof=4095, chi2=4085.0074)
  [PASS] practrand::fpf_platter                            p = 0.023851  (samples=8388834, e=2, sig_bins=2^12, dof=4095, chi2=4276.1283)
  [PASS] practrand::fpf_platter                            p = 0.771494  (samples=8388834, e=3, sig_bins=2^11, dof=2047, chi2=1999.1204)
  [PASS] practrand::fpf_platter                            p = 0.789475  (samples=8388834, e=4, sig_bins=2^10, dof=1023, chi2=986.3833)
  [PASS] practrand::fpf_platter                            p = 0.018369  (samples=8388834, e=5, sig_bins=2^9, dof=511, chi2=579.9940)
  [PASS] practrand::fpf_platter                            p = 0.805642  (samples=8388834, e=6, sig_bins=2^9, dof=511, chi2=483.2921)
  [PASS] practrand::fpf_platter                            p = 0.908556  (samples=8388834, e=7, sig_bins=2^8, dof=255, chi2=225.4713)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

Xorshift32
  [PASS] testu01::hamming_corr                             p = 0.191457  (n=500000, r=20, s=10, L=300, rho_hat=-0.001847, z=-1.3063)
  [PASS] testu01::hamming_indep_main                       p = 0.527734  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2203.7128)
  [PASS] testu01::hamming_indep_block                      p = 0.348958  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=2.1056)
  [PASS] practrand::fpf_cross                              p = 0.832827  (samples=8388382, sig_bits=14, max_exp=63, dof=19, chi2=13.1113)
  [PASS] practrand::fpf_platter                            p = 0.461980  (samples=8388382, e=0, sig_bins=2^13, dof=8191, chi2=8202.5554)
  [PASS] practrand::fpf_platter                            p = 0.834145  (samples=8388382, e=1, sig_bins=2^12, dof=4095, chi2=4007.1238)
  [PASS] practrand::fpf_platter                            p = 0.741304  (samples=8388382, e=2, sig_bins=2^12, dof=4095, chi2=4036.0318)
  [PASS] practrand::fpf_platter                            p = 0.355040  (samples=8388382, e=3, sig_bins=2^11, dof=2047, chi2=2070.2072)
  [PASS] practrand::fpf_platter                            p = 0.364261  (samples=8388382, e=4, sig_bins=2^10, dof=1023, chi2=1038.1078)
  [PASS] practrand::fpf_platter                            p = 0.402118  (samples=8388382, e=5, sig_bins=2^9, dof=511, chi2=518.2925)
  [PASS] practrand::fpf_platter                            p = 0.815797  (samples=8388382, e=6, sig_bins=2^9, dof=511, chi2=482.1375)
  [PASS] practrand::fpf_platter                            p = 0.673029  (samples=8388382, e=7, sig_bins=2^8, dof=255, chi2=244.3587)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

Xorshift64
  [PASS] testu01::hamming_corr                             p = 0.071751  (n=500000, r=20, s=10, L=300, rho_hat=-0.002547, z=-1.8007)
  [PASS] testu01::hamming_indep_main                       p = 0.515812  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2205.6998)
  [PASS] testu01::hamming_indep_block                      p = 0.935703  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.1329)
  [PASS] practrand::fpf_cross                              p = 0.835378  (samples=8388671, sig_bits=14, max_exp=63, dof=19, chi2=13.0621)
  [PASS] practrand::fpf_platter                            p = 0.927751  (samples=8388671, e=0, sig_bins=2^13, dof=8191, chi2=8004.9869)
  [PASS] practrand::fpf_platter                            p = 0.593282  (samples=8388671, e=1, sig_bins=2^12, dof=4095, chi2=4073.0152)
  [PASS] practrand::fpf_platter                            p = 0.779622  (samples=8388671, e=2, sig_bins=2^12, dof=4095, chi2=4024.9687)
  [PASS] practrand::fpf_platter                            p = 0.416396  (samples=8388671, e=3, sig_bins=2^11, dof=2047, chi2=2059.8691)
  [PASS] practrand::fpf_platter                            p = 0.211317  (samples=8388671, e=4, sig_bins=2^10, dof=1023, chi2=1059.0199)
  [PASS] practrand::fpf_platter                            p = 0.155324  (samples=8388671, e=5, sig_bins=2^9, dof=511, chi2=543.4096)
  [PASS] practrand::fpf_platter                            p = 0.178503  (samples=8388671, e=6, sig_bins=2^9, dof=511, chi2=540.3252)
  [PASS] practrand::fpf_platter                            p = 0.622237  (samples=8388671, e=7, sig_bins=2^8, dof=255, chi2=247.3773)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Unix System V rand()
  [PASS] testu01::hamming_corr                             p = 0.423715  (n=500000, r=20, s=10, L=300, rho_hat=0.001131, z=0.8000)
  [PASS] testu01::hamming_indep_main                       p = 0.273523  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2248.5950)
  [PASS] testu01::hamming_indep_block                      p = 0.336313  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=2.1794)
  [PASS] practrand::fpf_cross                              p = 0.253946  (samples=8388697, sig_bits=14, max_exp=63, dof=19, chi2=22.6319)
  [PASS] practrand::fpf_platter                            p = 0.294054  (samples=8388697, e=0, sig_bins=2^13, dof=8191, chi2=8259.8438)
  [PASS] practrand::fpf_platter                            p = 0.431730  (samples=8388697, e=1, sig_bins=2^12, dof=4095, chi2=4109.9148)
  [PASS] practrand::fpf_platter                            p = 0.225414  (samples=8388697, e=2, sig_bins=2^12, dof=4095, chi2=4162.9457)
  [PASS] practrand::fpf_platter                            p = 0.418690  (samples=8388697, e=3, sig_bins=2^11, dof=2047, chi2=2059.4915)
  [PASS] practrand::fpf_platter                            p = 0.950532  (samples=8388697, e=4, sig_bins=2^10, dof=1023, chi2=949.5300)
  [PASS] practrand::fpf_platter                            p = 0.213835  (samples=8388697, e=5, sig_bins=2^9, dof=511, chi2=536.0925)
  [PASS] practrand::fpf_platter                            p = 0.539844  (samples=8388697, e=6, sig_bins=2^9, dof=511, chi2=507.1444)
  [PASS] practrand::fpf_platter                            p = 0.095453  (samples=8388697, e=7, sig_bins=2^8, dof=255, chi2=284.9763)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Unix System V mrand48()
  [PASS] testu01::hamming_corr                             p = 0.723207  (n=500000, r=20, s=10, L=300, rho_hat=-0.000501, z=-0.3542)
  [PASS] testu01::hamming_indep_main                       p = 0.153387  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2276.9508)
  [PASS] testu01::hamming_indep_block                      p = 0.078374  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=5.0925)
  [PASS] practrand::fpf_cross                              p = 0.609658  (samples=8388833, sig_bits=14, max_exp=63, dof=19, chi2=16.7078)
  [PASS] practrand::fpf_platter                            p = 0.419329  (samples=8388833, e=0, sig_bins=2^13, dof=8191, chi2=8216.4201)
  [PASS] practrand::fpf_platter                            p = 0.483616  (samples=8388833, e=1, sig_bins=2^12, dof=4095, chi2=4098.0519)
  [PASS] practrand::fpf_platter                            p = 0.910049  (samples=8388833, e=2, sig_bins=2^12, dof=4095, chi2=3974.1770)
  [PASS] practrand::fpf_platter                            p = 0.253373  (samples=8388833, e=3, sig_bins=2^11, dof=2047, chi2=2089.0998)
  [PASS] practrand::fpf_platter                            p = 0.454368  (samples=8388833, e=4, sig_bins=2^10, dof=1023, chi2=1027.5253)
  [PASS] practrand::fpf_platter                            p = 0.363250  (samples=8388833, e=5, sig_bins=2^9, dof=511, chi2=521.5888)
  [PASS] practrand::fpf_platter                            p = 0.980016  (samples=8388833, e=6, sig_bins=2^9, dof=511, chi2=447.4992)
  [PASS] practrand::fpf_platter                            p = 0.124659  (samples=8388833, e=7, sig_bins=2^8, dof=255, chi2=281.2020)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Unix BSD random()
  [PASS] testu01::hamming_corr                             p = 0.743907  (n=500000, r=20, s=10, L=300, rho_hat=-0.000462, z=-0.3267)
  [PASS] testu01::hamming_indep_main                       p = 0.795139  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2154.0001)
  [PASS] testu01::hamming_indep_block                      p = 0.698912  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.7165)
  [PASS] practrand::fpf_cross                              p = 0.167061  (samples=8388328, sig_bits=14, max_exp=63, dof=19, chi2=24.8050)
  [PASS] practrand::fpf_platter                            p = 0.536830  (samples=8388328, e=0, sig_bins=2^13, dof=8191, chi2=8178.5065)
  [PASS] practrand::fpf_platter                            p = 0.254369  (samples=8388328, e=1, sig_bins=2^12, dof=4095, chi2=4154.4209)
  [PASS] practrand::fpf_platter                            p = 0.028174  (samples=8388328, e=2, sig_bins=2^12, dof=4095, chi2=4269.4552)
  [PASS] practrand::fpf_platter                            p = 0.771547  (samples=8388328, e=3, sig_bins=2^11, dof=2047, chi2=1999.1095)
  [PASS] practrand::fpf_platter                            p = 0.731517  (samples=8388328, e=4, sig_bins=2^10, dof=1023, chi2=994.6705)
  [PASS] practrand::fpf_platter                            p = 0.842265  (samples=8388328, e=5, sig_bins=2^9, dof=511, chi2=478.9356)
  [PASS] practrand::fpf_platter                            p = 0.412325  (samples=8388328, e=6, sig_bins=2^9, dof=511, chi2=517.4441)
  [PASS] practrand::fpf_platter                            p = 0.576952  (samples=8388328, e=7, sig_bins=2^8, dof=255, chi2=249.9820)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Unix Linux glibc rand()/random()
  [PASS] testu01::hamming_corr                             p = 0.743907  (n=500000, r=20, s=10, L=300, rho_hat=-0.000462, z=-0.3267)
  [PASS] testu01::hamming_indep_main                       p = 0.795139  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2154.0001)
  [PASS] testu01::hamming_indep_block                      p = 0.698912  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.7165)
  [PASS] practrand::fpf_cross                              p = 0.167061  (samples=8388328, sig_bits=14, max_exp=63, dof=19, chi2=24.8050)
  [PASS] practrand::fpf_platter                            p = 0.536830  (samples=8388328, e=0, sig_bins=2^13, dof=8191, chi2=8178.5065)
  [PASS] practrand::fpf_platter                            p = 0.254369  (samples=8388328, e=1, sig_bins=2^12, dof=4095, chi2=4154.4209)
  [PASS] practrand::fpf_platter                            p = 0.028174  (samples=8388328, e=2, sig_bins=2^12, dof=4095, chi2=4269.4552)
  [PASS] practrand::fpf_platter                            p = 0.771547  (samples=8388328, e=3, sig_bins=2^11, dof=2047, chi2=1999.1095)
  [PASS] practrand::fpf_platter                            p = 0.731517  (samples=8388328, e=4, sig_bins=2^10, dof=1023, chi2=994.6705)
  [PASS] practrand::fpf_platter                            p = 0.842265  (samples=8388328, e=5, sig_bins=2^9, dof=511, chi2=478.9356)
  [PASS] practrand::fpf_platter                            p = 0.412325  (samples=8388328, e=6, sig_bins=2^9, dof=511, chi2=517.4441)
  [PASS] practrand::fpf_platter                            p = 0.576952  (samples=8388328, e=7, sig_bins=2^8, dof=255, chi2=249.9820)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Windows CRT rand()
  [PASS] testu01::hamming_corr                             p = 0.240982  (n=500000, r=20, s=10, L=300, rho_hat=-0.001658, z=-1.1725)
  [PASS] testu01::hamming_indep_main                       p = 0.890950  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2127.4942)
  [FAIL] testu01::hamming_indep_block                      p = 0.004296  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=10.9001)
  [PASS] practrand::fpf_cross                              p = 0.205634  (samples=8389066, sig_bits=14, max_exp=63, dof=19, chi2=23.7571)
  [PASS] practrand::fpf_platter                            p = 0.377790  (samples=8389066, e=0, sig_bins=2^13, dof=8191, chi2=8230.2387)
  [PASS] practrand::fpf_platter                            p = 0.839022  (samples=8389066, e=1, sig_bins=2^12, dof=4095, chi2=4005.3606)
  [PASS] practrand::fpf_platter                            p = 0.015704  (samples=8389066, e=2, sig_bins=2^12, dof=4095, chi2=4292.1546)
  [PASS] practrand::fpf_platter                            p = 0.859748  (samples=8389066, e=3, sig_bins=2^11, dof=2047, chi2=1978.0696)
  [PASS] practrand::fpf_platter                            p = 0.674242  (samples=8389066, e=4, sig_bins=2^10, dof=1023, chi2=1002.0472)
  [PASS] practrand::fpf_platter                            p = 0.173477  (samples=8389066, e=5, sig_bins=2^9, dof=511, chi2=540.9701)
  [PASS] practrand::fpf_platter                            p = 0.647279  (samples=8389066, e=6, sig_bins=2^9, dof=511, chi2=498.3541)
  [PASS] practrand::fpf_platter                            p = 0.775595  (samples=8389066, e=7, sig_bins=2^8, dof=255, chi2=237.6355)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Windows VB6/VBA Rnd()
  [FAIL] testu01::hamming_corr                             p = 0.000000  (n=500000, r=20, s=10, L=300, rho_hat=-0.087334, z=-61.7546)
  [FAIL] testu01::hamming_indep_main                       p = 0.000000  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=60021.7661)
  [FAIL] testu01::hamming_indep_block                      p = 0.000000  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=3183.8786)
  [FAIL] practrand::fpf_cross                              p = 3.516e-155  (samples=8388981, sig_bits=14, max_exp=63, dof=19, chi2=789.5838)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=0, sig_bins=2^13, dof=8191, chi2=199008.6017)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=1, sig_bins=2^12, dof=4095, chi2=91830.3666)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=2, sig_bins=2^12, dof=4095, chi2=43338.5087)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=3, sig_bins=2^11, dof=2047, chi2=31405.5339)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=4, sig_bins=2^10, dof=1023, chi2=10968.8452)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=5, sig_bins=2^9, dof=511, chi2=4189.8599)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=6, sig_bins=2^9, dof=511, chi2=3257.9030)
  [FAIL] practrand::fpf_platter                            p = 1.759e-161  (samples=8388981, e=7, sig_bins=2^8, dof=255, chi2=1424.1904)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Windows .NET Random(seed)
  [PASS] testu01::hamming_corr                             p = 0.467237  (n=500000, r=20, s=10, L=300, rho_hat=-0.001028, z=-0.7270)
  [PASS] testu01::hamming_indep_main                       p = 0.118997  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2287.6840)
  [PASS] testu01::hamming_indep_block                      p = 0.813001  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.4140)
  [PASS] practrand::fpf_cross                              p = 0.630666  (samples=8388481, sig_bits=14, max_exp=63, dof=19, chi2=16.3967)
  [PASS] practrand::fpf_platter                            p = 0.976319  (samples=8388481, e=0, sig_bins=2^13, dof=8191, chi2=7939.1463)
  [PASS] practrand::fpf_platter                            p = 0.371308  (samples=8388481, e=1, sig_bins=2^12, dof=4095, chi2=4124.1213)
  [PASS] practrand::fpf_platter                            p = 0.791004  (samples=8388481, e=2, sig_bins=2^12, dof=4095, chi2=4021.4813)
  [PASS] practrand::fpf_platter                            p = 0.548263  (samples=8388481, e=3, sig_bins=2^11, dof=2047, chi2=2038.5850)
  [PASS] practrand::fpf_platter                            p = 0.674115  (samples=8388481, e=4, sig_bins=2^10, dof=1023, chi2=1002.0630)
  [PASS] practrand::fpf_platter                            p = 0.294976  (samples=8388481, e=5, sig_bins=2^9, dof=511, chi2=527.7426)
  [PASS] practrand::fpf_platter                            p = 0.333665  (samples=8388481, e=6, sig_bins=2^9, dof=511, chi2=524.1871)
  [PASS] practrand::fpf_platter                            p = 0.462087  (samples=8388481, e=7, sig_bins=2^8, dof=255, chi2=256.4858)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

ANSI C sample LCG
  [FAIL] testu01::hamming_corr                             p = 7.113e-143  (n=500000, r=20, s=10, L=300, rho_hat=-0.035991, z=-25.4497)
  [FAIL] testu01::hamming_indep_main                       p = 0.000000  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=813696.0622)
  [FAIL] testu01::hamming_indep_block                      p = 1.164e-282  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=1298.3545)
  [FAIL] practrand::fpf_cross                              p = 0.000000  (samples=8346698, sig_bits=14, max_exp=63, dof=19, chi2=34790.4838)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=0, sig_bins=2^13, dof=8191, chi2=77502.8900)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=1, sig_bins=2^12, dof=4095, chi2=37813.7381)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=2, sig_bins=2^12, dof=4095, chi2=22037.2531)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=3, sig_bins=2^11, dof=2047, chi2=10621.6915)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=4, sig_bins=2^10, dof=1023, chi2=4928.2652)
  [FAIL] practrand::fpf_platter                            p = 1.477e-180  (samples=8346698, e=5, sig_bins=2^9, dof=511, chi2=2035.9620)
  [FAIL] practrand::fpf_platter                            p = 1.906e-102  (samples=8346698, e=6, sig_bins=2^9, dof=511, chi2=1531.5849)
  [FAIL] practrand::fpf_platter                            p = 1.402e-47  (samples=8346698, e=7, sig_bins=2^8, dof=255, chi2=731.5678)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

LCG MINSTD
  [PASS] testu01::hamming_corr                             p = 0.923793  (n=500000, r=20, s=10, L=300, rho_hat=-0.000135, z=-0.0957)
  [PASS] testu01::hamming_indep_main                       p = 0.069719  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2308.0100)
  [PASS] testu01::hamming_indep_block                      p = 0.465298  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=1.5302)
  [FAIL] practrand::fpf_cross                              p = 0.000000  (samples=8354812, sig_bits=14, max_exp=63, dof=19, chi2=18922.5021)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8354812, e=0, sig_bins=2^13, dof=8191, chi2=59351.9347)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8354812, e=1, sig_bins=2^12, dof=4095, chi2=26947.5197)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8354812, e=2, sig_bins=2^12, dof=4095, chi2=16042.0308)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8354812, e=3, sig_bins=2^11, dof=2047, chi2=7838.1824)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8354812, e=4, sig_bins=2^10, dof=1023, chi2=3871.6841)
  [FAIL] practrand::fpf_platter                            p = 4.543e-130  (samples=8354812, e=5, sig_bins=2^9, dof=511, chi2=1716.7939)
  [FAIL] practrand::fpf_platter                            p = 1.154e-76  (samples=8354812, e=6, sig_bins=2^9, dof=511, chi2=1348.0053)
  [FAIL] practrand::fpf_platter                            p = 1.849e-30  (samples=8354812, e=7, sig_bins=2^8, dof=255, chi2=604.8242)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

AES-128-CTR
  [PASS] testu01::hamming_corr                             p = 0.604025  (n=500000, r=20, s=10, L=300, rho_hat=0.000733, z=0.5186)
  [PASS] testu01::hamming_indep_main                       p = 0.346377  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2234.6957)
  [PASS] testu01::hamming_indep_block                      p = 0.936284  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.1317)
  [PASS] practrand::fpf_cross                              p = 0.972140  (samples=8388629, sig_bits=14, max_exp=63, dof=19, chi2=9.0793)
  [PASS] practrand::fpf_platter                            p = 0.148616  (samples=8388629, e=0, sig_bins=2^13, dof=8191, chi2=8324.4700)
  [PASS] practrand::fpf_platter                            p = 0.592010  (samples=8388629, e=1, sig_bins=2^12, dof=4095, chi2=4073.3106)
  [PASS] practrand::fpf_platter                            p = 0.787117  (samples=8388629, e=2, sig_bins=2^12, dof=4095, chi2=4022.6841)
  [PASS] practrand::fpf_platter                            p = 0.011439  (samples=8388629, e=3, sig_bins=2^11, dof=2047, chi2=2195.3725)
  [PASS] practrand::fpf_platter                            p = 0.263568  (samples=8388629, e=4, sig_bins=2^10, dof=1023, chi2=1051.1942)
  [PASS] practrand::fpf_platter                            p = 0.219498  (samples=8388629, e=5, sig_bins=2^9, dof=511, chi2=535.4557)
  [PASS] practrand::fpf_platter                            p = 0.033161  (samples=8388629, e=6, sig_bins=2^9, dof=511, chi2=571.2598)
  [FAIL] practrand::fpf_platter                            p = 0.004655  (samples=8388629, e=7, sig_bins=2^8, dof=255, chi2=317.5623)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

cryptography::CtrDrbgAes256
  [PASS] testu01::hamming_corr                             p = 0.310632  (n=500000, r=20, s=10, L=300, rho_hat=0.001434, z=1.0139)
  [PASS] testu01::hamming_indep_main                       p = 0.763462  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2160.9946)
  [PASS] testu01::hamming_indep_block                      p = 0.244947  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=2.8134)
  [PASS] practrand::fpf_cross                              p = 0.527737  (samples=8388217, sig_bits=14, max_exp=63, dof=19, chi2=17.9208)
  [PASS] practrand::fpf_platter                            p = 0.395926  (samples=8388217, e=0, sig_bins=2^13, dof=8191, chi2=8224.1561)
  [PASS] practrand::fpf_platter                            p = 0.565567  (samples=8388217, e=1, sig_bins=2^12, dof=4095, chi2=4079.4117)
  [PASS] practrand::fpf_platter                            p = 0.438206  (samples=8388217, e=2, sig_bins=2^12, dof=4095, chi2=4108.4224)
  [PASS] practrand::fpf_platter                            p = 0.530552  (samples=8388217, e=3, sig_bins=2^11, dof=2047, chi2=2041.4333)
  [PASS] practrand::fpf_platter                            p = 0.593116  (samples=8388217, e=4, sig_bins=2^10, dof=1023, chi2=1011.7190)
  [PASS] practrand::fpf_platter                            p = 0.241790  (samples=8388217, e=5, sig_bins=2^9, dof=511, chi2=533.0406)
  [PASS] practrand::fpf_platter                            p = 0.645828  (samples=8388217, e=6, sig_bins=2^9, dof=511, chi2=498.4768)
  [PASS] practrand::fpf_platter                            p = 0.263159  (samples=8388217, e=7, sig_bins=2^8, dof=255, chi2=268.8902)
  [INFO] practrand::fpf_more                   9 additional platter results omitted


========================================================================
testu01_lz  (TestU01 Lempel-Ziv  k=25  replications=10)
========================================================================

MT19937
  [PASS] testu01::lzw_sum                                  p = 0.216239  (N=10, k=25, r=0, s=30, z_mean=-0.3910, z_sum=-1.2366)
  [PASS] testu01::lzw_ks                                   p = 0.155883  (N=10, k=25, r=0, s=30)
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
  [PASS] testu01::lzw_ks                                   p = 0.980845  (N=10, k=25, r=0, s=30)
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
  [PASS] testu01::lzw_ks                                   p = 0.131902  (N=10, k=25, r=0, s=30)
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
  [PASS] testu01::lzw_ks                                   p = 0.291519  (N=10, k=25, r=0, s=30)
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
  [PASS] testu01::lzw_ks                                   p = 0.878463  (N=10, k=25, r=0, s=30)
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
  [PASS] testu01::lzw_ks                                   p = 0.997893  (N=10, k=25, r=0, s=30)
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
  [PASS] testu01::lzw_ks                                   p = 0.997893  (N=10, k=25, r=0, s=30)
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
  [PASS] testu01::lzw_ks                                   p = 0.409984  (N=10, k=25, r=0, s=30)
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
  [PASS] testu01::lzw_ks                                   p = 0.183482  (N=10, k=25, r=0, s=30)
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
  [PASS] testu01::lzw_ks                                   p = 0.217584  (N=10, k=25, r=0, s=30)
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
  [PASS] testu01::lzw_ks                                   p = 0.778942  (N=10, k=25, r=0, s=30)
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
MT19937                                   0.025510  0.963248        28      1.951   0.615351
Xorshift32                                1.000000  1.000000        24     47.913   0.000000
Xorshift64                                0.042486  0.959682        19      1.747   0.586547
BAD Unix System V rand()                  0.103973  0.999980        18      4.111   0.000000
BAD Unix System V mrand48()               0.000000  1.000000        31  10141.521   0.009882
BAD Unix BSD random()                     0.114651  0.975209        13      1.964   0.358899
BAD Unix Linux glibc rand()/random()      0.114651  0.975209        13      1.964   0.358899
BAD Windows CRT rand()                    0.000000  0.999969         3     12.004   0.000927
BAD Windows VB6/VBA Rnd()                 0.000000  0.000000         7  10162.438   0.000000
BAD Windows .NET Random(seed)             0.026731  0.969952        28      1.931   0.978001
ANSI C sample LCG                         0.000000  1.000000         0  10172.876   0.000000
LCG MINSTD                                0.000000  1.000000         0  10172.876   0.000000
AES-128-CTR                               0.050697  0.987885         9      2.253   0.395101
cryptography::CtrDrbgAes256               0.000534  0.993353        31      3.272   0.390955
```
