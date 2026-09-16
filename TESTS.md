# Full Battery Results

Full `run_tests` battery harvested from `darby.local` (Apple M4 Pro, 8P+4E cores) on 2026-09-16.

Sample size: **16 Mbit** per generator for NIST; DIEHARD/DIEHARDER
consume **16 M 32-bit words** (plus what the live-drawing tests take
directly).

Command:

```sh
./target/release/run_tests
```

Scope:

This top section covers the standard `run_tests` battery only.  The Knuth,
Hamming-weight, FPF, Lempel-Ziv, Webster-Tavares, and Gorilla probes are
reported separately in `## Auxiliary Probes`; use `tests/run_all.sh` for the
combined run or `tests/run_aux.sh` for the auxiliary suite alone.

Notes:

**Reproducibility.**  Generators with a fixed seed (the ciphers, the `BAD…`
family, `Constant`/`Counter`) reproduce these results exactly on any machine.
The rows labeled **`(OsRng seed)`** / **`(OsRng key)`** are deliberately seeded
from operating-system entropy — testing each algorithm as it would be deployed
— so their specific per-slot PASS/FAIL pattern varies from run to run; read
those rows as indicative of the algorithm, not as a reproducible fixture.

**Why result counts vary across generators.**

The battery total differs from run to run because several test families are
conditionally skipped based on properties of the sample, not the generator.

The battery has **739 test slots** at this sample size:

- **739 results** — every generator except `Dual_EC_DRBG`.  Some slots report
  SKIP rather than PASS or FAIL, depending on the sample:
  - the 8 + 18 = 26 per-state `random_excursions` and
    `random_excursions_variant` slots skip when the signed random walk completes
    fewer than J = 500 zero-crossing cycles.  At 16 Mbit the expected
    cycle count is J ≈ 3191 (= √(2n/π)), comfortably above
    the threshold for well-behaved generators.  Degenerate generators (Constant,
    Counter, ANSI C LCG, MINSTD) always skip them; a handful of others can,
    depending on their random seed.
  - the parametric Maurer slots skip for any `L` whose sample holds fewer than
    K = 1000·2^L test blocks (at 16 Mbit, L = 11..16).

- **200 results** — `Dual_EC_DRBG` only: three P-256 scalar multiplications per
  30-byte output block make DIEHARD and DIEHARDER prohibitively slow, so only
  the NIST SP 800-22 suite is run.

**Expected false positives.**  At α = 0.01, a perfect generator should fail
roughly 1% of tests by chance.  With 707–733 scored tests per generator
(the rest report SKIP), the expected false-fail count is approximately 7.  Isolated failures below that threshold
are noise, not structure.

## Summary Table

| RNG | Total | PASS | FAIL | SKIP |
|---|---:|---:|---:|---:|
| OsRng (/dev/urandom) | 739 | 728 | 5 | 6 |
| MT19937 (seed=19650218) | 739 | 728 | 5 | 6 |
| Xorshift64 (seed=1) | 739 | 726 | 7 | 6 |
| Xorshift32 (seed=1) | 739 | 720 | 13 | 6 |
| BAD Unix System V rand() (15-bit LCG, seed=1) | 739 | 728 | 5 | 6 |
| BAD Unix System V mrand48() (seed=1) | 739 | 726 | 7 | 6 |
| BAD Unix BSD random() TYPE_3 (seed=1) | 739 | 721 | 12 | 6 |
| BAD Unix Linux glibc rand()/random() (seed=1) | 739 | 721 | 12 | 6 |
| BAD Unix FreeBSD12 rand_r() compat (seed=1) | 739 | 720 | 13 | 6 |
| BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1) | 739 | 727 | 6 | 6 |
| BAD Windows VB6/VBA Rnd() (project seed=1) | 739 | 202 | 531 | 6 |
| BAD Windows .NET Random(seed=1) compat | 739 | 726 | 7 | 6 |
| ANSI C sample LCG (1103515245,12345; seed=1) | 739 | 10 | 697 | 32 |
| LCG MINSTD (seed=1) | 739 | 16 | 691 | 32 |
| BAD Borland C++ rand() LCG (seed=1) | 739 | 722 | 11 | 6 |
| AES-128-CTR (NIST key) | 739 | 728 | 5 | 6 |
| Camellia-128-CTR (key=00..0f) | 739 | 724 | 9 | 6 |
| Twofish-128-CTR (key=00..0f) | 739 | 722 | 11 | 6 |
| Serpent-128-CTR (key=00..0f) | 739 | 701 | 6 | 32 |
| SM4-CTR (key=00..0f) | 739 | 725 | 8 | 6 |
| Grasshopper-CTR (key=00..1f) | 739 | 727 | 6 | 6 |
| CAST-128-CTR (key=00..0f) | 739 | 725 | 8 | 6 |
| SEED-CTR (key=00..0f) | 739 | 721 | 12 | 6 |
| Rabbit (key=00..0f, iv=00..07) | 739 | 724 | 9 | 6 |
| Salsa20 (key=00..1f, nonce=00..07) | 739 | 725 | 8 | 6 |
| Snow3G (key=00..0f, iv=00..0f) | 739 | 726 | 7 | 6 |
| ZUC-128 (key=00..0f, iv=00..0f) | 739 | 729 | 4 | 6 |
| SpongeBob (SHA3-512 chain, OsRng seed) | 739 | 731 | 2 | 6 |
| Squidward (SHA-256 chain, OsRng seed) | 739 | 726 | 7 | 6 |
| PCG32 (OsRng seed) | 739 | 725 | 8 | 6 |
| PCG64 (OsRng seed) | 739 | 701 | 6 | 32 |
| Xoshiro256 (OsRng seed) | 739 | 720 | 13 | 6 |
| Xoroshiro128 (OsRng seed) | 739 | 728 | 5 | 6 |
| SFC64 (OsRng seed) | 739 | 727 | 6 | 6 |
| JSF64 (OsRng seed) | 739 | 722 | 11 | 6 |
| ChaCha20 CSPRNG (OsRng key) | 739 | 727 | 6 | 6 |
| HMAC_DRBG SHA-256 (OsRng seed) | 739 | 725 | 8 | 6 |
| Hash_DRBG SHA-256 (OsRng seed) | 739 | 727 | 6 | 6 |
| cryptography::CtrDrbgAes256 (seed=00..2f) | 739 | 699 | 8 | 32 |
| Constant (0xDEAD_DEAD) | 739 | 0 | 707 | 32 |
| Counter (0,1,2,…) | 739 | 1 | 706 | 32 |
| Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01) | 200 | 192 | 2 | 6 |

## Theory By Test

Here $Q(a,x)=\Gamma(a,x)/\Gamma(a)$ is the upper regularized gamma function,
$\Phi$ is the standard normal CDF, and $D_n=\sup_x |F_n(x)-x|$ is the
one-sample Kolmogorov-Smirnov statistic used whenever this report says
"outer KS on p-values."  The KS tail probability uses the
exact distribution by Durbin's matrix formula ($n \le 4999$), as Marsaglia,
Tsang and Wang present it, with a Stephens-corrected asymptotic series
beyond.

Every DIEHARD p-value below is small when the test fails.  Seven results
summarize several p-values with the KS test above and report its p-value:
`birthday_spacings`, `runs_up` and `runs_down` feed it upper-tail chi-square
p-values, `bitstream` two-sided normal p-values $\mathrm{erfc}(|z|/\sqrt{2})$,
and `parking_lot`, `minimum_distance_2d` and `spheres_3d` CDF values that are
approximately uniform under the null: $\Phi(z)$ for parking lots, and
$1-\exp(\cdot)$ from the Poisson approximation for the two distance tests.
The other ten report a single p-value: an upper-tail chi-square for the three
rank tests, `squeeze` and `craps_throws`, and a two-sided normal p-value for
`opso`, `oqso`, `dna`, `count_ones_stream` and `craps_wins`.

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
  $f_n = \frac{1}{K}\sum_{i=1}^K \log_2 A_i$ over the $K$ blocks that follow
  $Q = 10\cdot2^L$ initialization blocks. It is normalized as
  $z = (f_n-\mu_L)/\sigma$ with $\sigma = c(L,K)\sqrt{v_L/K}$, where $\mu_L$
  and $v_L$ are the tabulated mean and variance of $\log_2 A_i$ and $c(L,K)$
  is the factor of SP 800-22 §2.9.4, and scored with
  $p=\mathrm{erfc}(|z|/\sqrt{2})$. The crate reports both the NIST wrapper
  (which selects $L$ from $n$ over NIST's domain $L\in[6,16]$) and the broader
  Maurer parametric
  family over $L=5,\dots,16$.  A parametric setting runs only when the
  sample holds $K \ge 1000\cdot 2^L$ test blocks, the $K$ behind every row of
  §2.9.7's table, so at 16 Mbit $L=5,\dots,10$ run and $L=11,\dots,16$
  report SKIP.

- **`linear_complexity`.** Break the stream into blocks of length $M=500$,
  run Berlekamp-Massey on each block to get its linear complexity $L_i$, and
  form $T_i=(-1)^M(L_i-\mu)+2/9$ with
  $\mu = M/2 + (9+(-1)^{M+1})/36 - (M/3+2/9)/2^M$, as §2.10.4 prints it.
  The $T_i$ fall into NIST's seven classes, split at $\pm0.5$, $\pm1.5$ and
  $\pm2.5$, and a chi-square with $df=6$ compares the class counts to the
  reference probabilities. The test is aimed at short linear recurrences: if
  a block is too easy to synthesize by an LFSR, its linear complexity lands
  too far below the null mean.

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
  $m < \lfloor\log_2 n\rfloor - 2$.

- **`approximate_entropy`.** For pattern lengths $m=10$ and $m+1$, define
  $\phi_m=\frac{1}{n}\sum_i \log C_i^{(m)}$, where $C_i^{(m)}$ is the
  circular pattern-match frequency of the $i$th overlapping block. The test
  statistic is
  $\mathrm{ApEn}(m)=\phi_m-\phi_{m+1}$ and NIST scores
  $\chi^2 = 2n(\ln 2-\mathrm{ApEn}(m))$.  The implementation enforces
  $m < \lfloor\log_2 n\rfloor - 5$ per the SP 800-22 validity condition.

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
  §2.14.4/§2.15.4; below that they still report every per-state slot, as
  SKIP.  A walk ending exactly at zero does not contribute an empty trailing
  cycle.

- **`random_excursions_variant`.** Use the same cycle decomposition, but now
  test only the total visit count $\xi(x)$ to each
  $x\in\{\pm1,\dots,\pm9\}$. NIST models
  $\xi(x)$ around mean $J$ and uses
  $p=\mathrm{erfc}\!\left(\frac{|\xi(x)-J|}{\sqrt{2J(4|x|-2)}}\right)$,
  so this is a per-state aggregate walk-balance check.

### DIEHARD

- **`birthday_spacings`.** For each trial, choose $m=512$ birthdays in a year
  of size $n=2^{24}$, sort them, sort their spacings, and let $j$ be the
  number of adjacent equal sorted spacings, so a value seen $r$ times adds
  $r-1$. Under the null, $j \overset{a}{\sim} \mathrm{Poisson}(\lambda)$ with
  $\lambda = m^3/(4n)=2$. Each of nine bit offsets (bits $o$ to $o+23$,
  $o=0,\dots,8$) draws fresh words for its $500$ trials, and its histogram is
  scored by chi-square in the cells $j=0,\dots,5$ and $j\ge6$ ($df=6$); an
  outer KS combines the nine p-values.

- **`binary_rank_32x32`.** Fill $40{,}000$ binary $32\times 32$ matrices over
  $\mathbb{F}_2$, compute their ranks, and compare the counts of
  $32$, $31$, $30$, and $\le 29$ to the exact GF(2) rank law

$$P_{m,n}(r)=2^{-mn}\prod_{i=0}^{r-1}\frac{(2^m-2^i)(2^n-2^i)}{(2^r-2^i)}$$

  The reported p-value comes from the pooled chi-square on those bins.

- **`binary_rank_31x31`.** The same GF(2) rank test on $31\times 31$
  matrices whose rows are the leftmost 31 bits of each word, with exact
  probabilities for that size. A generator that returns zero-extended 31-bit
  words leaves a column of zeros here and fails.

- **`binary_rank_6x8`.** Build $100{,}000$ matrices with $6$ rows and $8$
  columns, each row the low byte of one word, compute the rank over
  $\mathbb{F}_2$, and compare the observed counts of ranks $6$, $5$, and
  $\le4$ to the exact $6\times8$ probabilities. This is a small-matrix
  dependence probe aimed at byte-lane linearity.

- **`bitstream`.** Read each word high bit first and, in each of $20$
  disjoint chunks, slide a $20$-bit window across $2^{21}$ overlapping
  positions and count how many of the $2^{20}$ possible words are never seen.
  Marsaglia models the missing-word count as approximately normal with mean
  $141{,}909$ and $\sigma=428$, so each chunk is scored by
  $p=\mathrm{erfc}(|z|/\sqrt{2})$ and the final report is an outer KS on
  the $20$ p-values.

- **`opso`.** OPSO counts the missing words among $2^{21}$ samples from a
  $2^{20}$-word space, each sample two $10$-bit letters from two words: bits
  0–9 of a pair of words, then bits 10–19 of the same pair.  Every letter is
  a disjoint bit field, so the samples are iid and the count is scored with
  their exact moments $\mu \approx 141{,}909.19$, $\sigma \approx 290.33$,
  giving $p=\mathrm{erfc}(|z|/\sqrt{2})$.

- **`oqso`.** OQSO is the same count with four $5$-bit letters from four
  words per sample; each group of four words yields six samples from the
  disjoint fields at bits 0–29, scored with the same exact moments.

- **`dna`.** DNA uses ten $2$-bit letters from ten words per sample; each
  group of ten words yields sixteen samples from its disjoint $2$-bit fields,
  scored with the same exact moments.

- **`count_ones_stream`.** Map each byte, taking each word's low byte first,
  to one of five letters $A,\dots,E$ according to its Hamming weight, form
  $256{,}000$ overlapping $5$-letter words and their leading $4$-letter
  words, and compute $Z = (Q_5-Q_4-2500)/\sqrt{5000}$, where $Q_5$ and $Q_4$
  are the corresponding Pearson sums against the exact letter-product
  probabilities and $Q_5-Q_4$ is asymptotically $\chi^2(2500)$. The p-value
  is $p=\mathrm{erfc}(|Z|/\sqrt{2})$.

- **`parking_lot`.** Sequentially try to place $12{,}000$ unit square cars in
  a $100\times100$ lot, rejecting any new car whose footprint overlaps an
  existing one. The total parked count is approximately normal with
  $\mu=3523$ and $\sigma=21.9$; each of $10$ repetitions is mapped through
  $\Phi$, and the final result is a KS test over those uniformized values.

- **`minimum_distance_2d`.** Place $8{,}000$ points in a
  $10{,}000\times10{,}000$ square and find the nearest-pair distance
  $d_{\min}$, with $r = d_{\min}/10{,}000$.  Two uniform points in the unit
  square lie within $r$ with probability
  $H_2(r) = \pi r^2 - \tfrac{8}{3}r^3 + \tfrac{1}{2}r^4$, the pairs that do
  are approximately Poisson, and $U = 1-\exp(-\binom{n}{2}H_2(r))$ is
  approximately uniform; an outer KS runs over $100$ repeats.

- **`spheres_3d`.** Place $4{,}000$ points in a $1000^3$ cube, find the
  nearest-pair distance and score $U = 1-\exp(-\binom{n}{2}H_3(r))$ in the
  same way over $20$ repeats.

- **`squeeze`.** Start from $k_0 = 2^{31}-1$ and iterate
  $k_{t+1}=\lceil k_t U_t\rceil$ until $k_t=1$ or $48$ steps are taken. The
  test compares the distribution of the stopping time $J$, in $43$ cells
  ($J\le6$, $7,\dots,47$, $48$), to its exact law by chi-square.  With $f_k(j)$
  the probability of $j$ steps from $k$, $f_k(j) = \frac1k\sum_{m\le k}
  f_m(j-1)$, which `examples/squeeze_table.rs` evaluates at $k = 2^{31}-1$.
  Cells expecting fewer than $5$ counts are pooled into one cell, which at
  $N=100{,}000$ leaves $39$ scored cells ($df=38$).

- **`runs_up` and `runs_down`.** For sequences of $10{,}000$ integers, count
  monotone run lengths $1,2,3,4,5,6+$ in the upward and downward directions,
  including both runs still open when the sequence ends.  Let $R$ be the
  run-count vector, $b$ the theoretical run proportions, and $A$ the
  Grafton/Knuth inverse-covariance matrix; the core statistic is
  $V = (R-nb)^\top A (R-nb)/n$, converted to $p = Q(3,V/2)$, with a final KS
  across $10$ repetitions for each direction.

- **`craps_wins`.** Simulate $N=200{,}000$ craps games, each die the quotient
  of a word by $\lfloor (2^{32}-1)/6 \rfloor$ with rejection of the top four
  words, and count the number of wins $W$. Under fair dice,
  $W \approx \mathrm{Bin}(N,p_{\mathrm{win}})$ with
  $p_{\mathrm{win}} = 244/495$, so the code standardizes
  $z = (W-Np_{\mathrm{win}})/\sqrt{Np_{\mathrm{win}}(1-p_{\mathrm{win}})}$ and
  reports the two-sided $p=\mathrm{erfc}(|z|/\sqrt{2})$.

- **`craps_throws`.** The same $200{,}000$ simulated games are also binned by
  game length, with the tail pooled at $\ge 22$ throws ($df=21$). The expected
  cell probabilities are exact, and the reported p-value is the chi-square
  fit of the observed game length histogram to that law.  A game still
  unresolved after 1000 throws is stopped and scored as a loss in the tail
  cell, so a degenerate stream cannot hang the battery.

### DIEHARDER

- **`minimum_distance_nd`.** A nearest-neighbour test in $d=2,\dots,5$
  dimensions; the battery runs $d=5$, with $n=8{,}000$ points in the unit
  cube per repeat.  Two uniform points in the unit $d$-cube lie within $r$
  with probability
  $H_d(r)=\sum_{k=0}^{d}(-1)^k\binom{d}{k}\pi^{(d-k)/2}r^{d+k}/\Gamma(1+\tfrac{d+k}{2})$,
  the integral over the ball of the difference density
  $\prod_i(1-|z_i|)$.  $U = 1-\exp(-\binom{n}{2}H_d(r))$ is approximately
  uniform, followed by an outer KS across $100$ repeats.

- **`permutations`.** Draw non-overlapping blocks of $t=5$ independent
  uniforms, map each block to its permutation rank in $S_5$, and compare the
  observed counts over the $5!=120$ orderings to the uniform expectation
  $E = N/120$ with a chi-square.

- **`lagged_sums` (lags 1 and 100).** Take every $(\ell+1)$th uniform variate,
  sum $m$ such terms, and compare $S = \sum_{i=1}^m U_i$ to the null mean
  $m/2$ and variance $m/12$: $z = (S-m/2)/\sqrt{m/12}$ and
  $p=\mathrm{erfc}(|z|/\sqrt{2})$, once for $\ell=1$ and once for $\ell=100$.

- **`ks_uniform`.** Convert the raw words to uniforms in $[0,1)$, sort them,
  and compute the one-sample KS statistic
  $D_n=\max_i \max(i/n-U_{(i)}, U_{(i)}-(i-1)/n)$ with its exact or
  asymptotic tail.

- **`byte_distribution`.** From each of three consecutive words take the bytes
  at bit offsets 0, 12 and 24 and count the $256$ possible byte values in each
  of those $9$ streams. With expected cell count $E=t/256$ the statistic is a
  single chi-square over $9\times256$ cells ($df = 9 \times 255$).

- **`dct`.** Rotate the raw $32$-bit words, apply a Type-II DCT
  $X_k = \sum_{j=0}^{N-1} x_j \cos(\pi(j+\tfrac12)k/N)$ on blocks of length
  $N=256$, and record the index $k^\star = \arg\max_k |X_k|$ after centring
  and rescaling the DC coefficient.  The max-position histogram should be
  close to uniform, so the reported p-value is a chi-square on those
  positions.

- **`monobit2`.** For block lengths of $2,4,8,\dots$ words, count the ones in
  each complete, non-overlapping block and compare the histogram to
  $\mathrm{Bin}(32b,\tfrac12)$ by chi-square, with tail cells pooled until
  each expects at least $5$ blocks.  A block length is used while its most
  likely cell expects at least $20$ blocks.  Each level's p-value is folded
  two-sided, $2\min(p,1-p)$, and the levels, which share their words, are
  combined by Bonferroni's bound, $\min(1, L\cdot\min_\ell \mathrm{fold}_\ell)$.

- **`fill_tree_count`.** Insert uniform samples into a fixed 32-slot binary
  search tree of four levels until a sample's path is fully occupied, and
  count the samples placed, $4 \le t \le 15$.  Its exact law follows from a
  recurrence over random binary search trees: $P(4)=2/15$, $P(5)=1/5$,
  $P(6)=13/63$, …, $P(15)=1/59{,}535$.  The counts are scored by chi-square
  with tail cells pooled to at least $5$ expected trials.

- **`fill_tree_position`.** The same trials also record which of the $16$
  gaps below the tree the colliding sample falls into; by the same recurrence
  every gap has probability exactly $1/16$, and the position counts get a
  chi-square with $df=15$.

- **`bit_distribution`.** Read a continuous MSB-first bitstream,
  partition it into blocks of $64$ consecutive $n$-bit symbols, and for each
  specific pattern $u\in\{0,\dots,2^n-1\}$ count how often $u$ occurs inside a
  block. Under $H_0$, $C_u \sim \mathrm{Bin}(64,2^{-n})$, so the test compares
  the across-block histogram of $C_u$ to that binomial law by chi-square,
  pooling the cells expecting fewer than $20$ counts so that every block is
  counted once; the crate emits every per-pattern p-value explicitly.

- **`gcd_distribution`.** Draw $100{,}000$ random integer pairs $(u,v)$,
  compute $g=\gcd(u,v)$, and compare the observed gcd histogram to the
  classical law $\Pr(g=k)=6/(\pi^2 k^2)$ over
  $\lfloor\sqrt{6N/(100\pi^2)}\rfloor$ cells, the last pooling every larger
  gcd, with $g=1$ not scored; at $N=100{,}000$ that scores $k=2,\dots,22$ and
  pools $g\ge23$.  A stream of zero words, which leaves nothing to score,
  fails.

- **`gcd_step_counts`.** The same integer pairs are also scored by the number
  of Euclidean algorithm steps.  No closed form is known for 32-bit words, so
  the reference law is estimated from $10^{12}$ simulated pairs
  (`examples/gcd_step_table.rs`, PCG64 and xoshiro256**); the chi-square runs
  over the bins that expect at least $5$ counts.

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
  into a single tail cell (folded into the last kept cell if the pool still
  expects fewer than 5), and the test runs on the surviving `cells` with
  $df = \mathrm{cells} - 1$.  Tests uniformity of the real-valued projection
  and independence of successive words.

- **Wald–Wolfowitz runs test** (runs above/below the median; not TAOCP's run test).  Compute
  the sample median, classify each output value as above or below it
  (values equal to the median are dropped), and count the runs $R$ of
  consecutive same-side values.  Conditional on $n_1$ values above and $n_2$
  below ($n = n_1 + n_2$), the null moments are
  $\mu = 1 + 2n_1 n_2/n$ and
  $\sigma^2 = 2n_1 n_2(2n_1 n_2 - n)/(n^2(n-1))$.  The crate reports
  $z = (R - \mu)/\sigma$ and the two-tailed p-value
  $p = \mathrm{erfc}(|z|/\sqrt{2})$.  Detects clustering (too few runs) and
  over-alternation (too many runs) around the sample median.

- **FPF** (floating-point-format frequency, after Doty-Humphrey).  Parse the
  LSB-first bitstream into disjoint codewords: a run of zeros terminated by a
  stop bit gives the geometric exponent (capped at $2^{\mathrm{exp\_bits}}-1$),
  followed by `sig_bits` of significand (defaults: `sig_bits` = 14,
  `exp_bits` = 6).  Per exponent, the low $b$ significand bits are scored by a
  G-test, with $b$ the widest width at which every bin expects at least $16$
  samples; a separate exponent-distribution G-test (`:cross`) checks the
  geometric law.  Disjoint codewords make the samples independent, so the
  asymptotic $\chi^2$ law of each G statistic applies.

- **Lempel–Ziv** (L'Ecuyer–Simard).  Parse an $n = 2^k$-bit stream as an LZ78
  dictionary: each new phrase extends the longest previously seen prefix by
  one bit.  The stream concatenates the $s$ bits kept from each word after
  dropping its $r$ leading bits ($k = 25$, $r = 0$, $s = 30$ in the auxiliary
  run).  The phrase count $W$ is discrete, with distribution $F$ enumerated
  exactly for $k \le 5$ and simulated above (`examples/lz78_table.rs`).
  Each replication gives $U = F(W-1) + V\,P(W)$, with $V$ uniform from a
  separately seeded generator, which is exactly uniform under $F$.  Over $N$
  replications the crate reports a KS test of the $U$ (`lzw_ks`) and the
  two-sided normal p-value of $\sum \Phi^{-1}(U)/\sqrt{N}$ (`lzw_sum`).  A
  simulated table of $M$ replications supports $N \le M/100$.  Low complexity
  (few phrases) flags repetitive structure; high complexity flags
  over-dispersion.

- **Hamming weights** (L'Ecuyer–Simard).  Each word contributes the $s$-bit
  field left after dropping its $r$ leading bits, and $L$-bit blocks are
  packed from those fields: for $L \ge s$, $\lfloor L/s \rfloor$ whole fields
  plus the leading $L \bmod s$ bits of one more word's field; for $L < s$,
  $\lfloor s/L \rfloor$ blocks from the low end of each field.
  Correlation: compute the Hamming weights $W_i$ of $n = 500{,}000$
  successive blocks; the lag-1 correlation of centered weights,
  $\hat\rho = 4\sum_{i}(W_i - L/2)(W_{i+1} - L/2)\,/\,((n-1)L)$,
  is standardized as $z = \hat\rho\sqrt{n-1}$ and scored two-sided.
  Independence: for $n = 500{,}000$ successive pairs of $L$-bit blocks
  $(X, Y)$, compare the joint weight histogram to
  $\mathrm{Bin}(L,\tfrac12)\times\mathrm{Bin}(L,\tfrac12)$ by chi-square, with
  cells expecting fewer than $10$ pairs pooled, beside a corner-block
  chi-square that counts pairs whose weights both sit on the same side of
  the middle against pairs on opposite sides.  Detects linear dependencies
  across block boundaries.

- **Webster–Tavares strict avalanche and bit independence (SAC/BIC)**.  Treat
  a generator's seed as the input and the low bits of its first `next_u64`
  value as the output (32 input and 32 output bits and 4 096 sampled seeds by
  default; for a generator of 32-bit words those output bits are its second
  word).  For each input bit $j$, complement it in every sampled seed and
  record which output bits change.
  The dependence matrix entry $A_{ij}$ is the fraction of samples in which
  output bit $i$ flips; the SAC wants $A_{ij} = \tfrac12$, and the probe
  reports the mean and maximum of $|A_{ij} - \tfrac12|$ (`SACmean`,
  `SACmax`).  For bit independence it correlates the change indicators of
  every output-bit pair under each flipped input bit and reports the mean
  and maximum $|\rho|$ (`BICmean`, `BICmax`).  A pair whose indicator never
  or always fires has no defined correlation; such pairs are counted in
  `BICdegen` and left out of the BIC figures, so a GF(2)-linear generator
  such as Xorshift32 shows up as entirely degenerate rather than perfectly
  independent.  Cryptographic generators should land near the ideal values;
  LCGs and short-state generators do not, because their seeding diffuses
  little.

- **Gorilla** (Marsaglia–Tsang).  For each of the 32 bit positions, extract that
  bit from $2^{26}+25$ successive words to form a stream of $2^{26}+25$ bits;
  every position reads the same words.  Count the number of distinct 26-bit
  patterns that never appear (missing words).  Under the null the count is
  approximately $N(24{,}687{,}971,\ 4170^2)$, the values the paper gives, and
  each position's p-value is the upper tail $u = 1-\Phi(z)$.  The aggregate is
  the Anderson–Darling statistic $A$ of the 32 sorted p-values, with each
  product $u_i(1-u_{33-i})$ floored at $10^{-30}$, converted to
  $\Pr(A_{32} < A)$ with Marsaglia and Marsaglia's (2004) distribution,
  $x + \mathrm{errfix}(32, x)$ for $x = \mathrm{ADinf}(A)$, up to
  $x^* = 0.9995$ ($A \approx 6.61$).  Above that the upper tail is the
  limiting tail scaled to meet it, $(1-x)\,(1-\mathrm{errfix}(32, x^*)/(1-x^*))$,
  a rule chosen from simulation.  The probe prints $A$ and $1-\Pr(A_{32} < A)$,
  so small values fail.  Against simulation the $n = 32$ tail errors are
  mostly positive, so p-values are mostly conservative; the largest reliably
  resolved error is $+2.46 \pm 0.14\%$ at $A = 6.62$.  The ADKS values the
  paper prints match this distribution to within the four-decimal rounding of
  their per-bit inputs.  The aggregate detects positional asymmetries and
  bit-plane correlations invisible to the standard birthday-problem tests.

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

- **OsRng (/dev/urandom)**: 5/739 — `dieharder::bit_distribution` (×5)
- **MT19937 (seed=19650218)**: 5/739 — `dieharder::bit_distribution` (×3), `nist::non_overlapping_template` (×2)
- **Xorshift64 (seed=1)**: 7/739 — `dieharder::bit_distribution` (×6), `nist::non_overlapping_template`
- **Xorshift32 (seed=1)**: 13/739 — `diehard::binary_rank_31x31`, `diehard::binary_rank_32x32`, `dieharder::bit_distribution` (×9), `dieharder::monobit2`, `nist::matrix_rank`
- **BAD Unix System V rand() (15-bit LCG, seed=1)**: 5/739 — `diehard::opso`, `dieharder::bit_distribution`, `nist::non_overlapping_template` (×2), `nist::spectral`
- **BAD Unix System V mrand48() (seed=1)**: 7/739 — `diehard::dna`, `diehard::opso`, `diehard::oqso`, `dieharder::bit_distribution` (×2), `nist::non_overlapping_template` (×2)
- **BAD Unix BSD random() TYPE_3 (seed=1)**: 12/739 — `dieharder::bit_distribution` (×6), `nist::non_overlapping_template` (×5), `nist::serial_delta2`
- **BAD Unix Linux glibc rand()/random() (seed=1)**: 12/739 — `dieharder::bit_distribution` (×6), `nist::non_overlapping_template` (×5), `nist::serial_delta2`
- **BAD Unix FreeBSD12 rand_r() compat (seed=1)**: 13/739 — `dieharder::bit_distribution` (×10), `nist::non_overlapping_template` (×2), `nist::spectral`
- **BAD Windows CRT rand() (MSVC/UCRT lineage, seed=1)**: 6/739 — `dieharder::bit_distribution` (×4), `nist::random_excursions`, `nist::spectral`
- **BAD Windows VB6/VBA Rnd() (project seed=1)**: 531/739 — `diehard::binary_rank_6x8`, `diehard::birthday_spacings`, `diehard::bitstream`, `diehard::count_ones_stream`, `diehard::craps_throws`, `diehard::dna`, `diehard::minimum_distance_2d`, `diehard::opso`, `diehard::oqso`, `diehard::parking_lot`, `diehard::spheres_3d`, `diehard::squeeze`, `dieharder::bit_distribution` (×499), `dieharder::byte_distribution`, `dieharder::dct`, `dieharder::fill_tree_count`, `dieharder::fill_tree_position`, `dieharder::gcd_distribution`, `dieharder::gcd_step_counts`, `dieharder::ks_uniform`, `dieharder::lagged_sums` (×2), `dieharder::minimum_distance_nd`, `dieharder::monobit2`, `maurer::universal_l05`, `maurer::universal_l06`, `maurer::universal_l07`, `maurer::universal_l08`, `maurer::universal_l09`, `maurer::universal_l10`, `nist::overlapping_template`, `nist::spectral`, `nist::universal`
- **BAD Windows .NET Random(seed=1) compat**: 7/739 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template` (×3)
- **ANSI C sample LCG (1103515245,12345; seed=1)**: 697/739 — `diehard::binary_rank_31x31`, `diehard::binary_rank_32x32`, `diehard::binary_rank_6x8`, `diehard::bitstream`, `diehard::count_ones_stream`, `diehard::craps_throws`, `diehard::craps_wins`, `diehard::dna`, `diehard::minimum_distance_2d`, `diehard::opso`, `diehard::oqso`, `diehard::parking_lot`, `diehard::spheres_3d`, `diehard::squeeze`, `dieharder::bit_distribution` (×510), `dieharder::byte_distribution`, `dieharder::dct`, `dieharder::gcd_distribution`, `dieharder::gcd_step_counts`, `dieharder::ks_uniform`, `dieharder::lagged_sums` (×2), `dieharder::minimum_distance_nd`, `dieharder::monobit2`, `maurer::universal_l06`, `maurer::universal_l08`, `maurer::universal_l09`, `maurer::universal_l10`, `nist::approximate_entropy`, `nist::block_frequency`, `nist::cumulative_sums_backward`, `nist::cumulative_sums_forward`, `nist::frequency`, `nist::longest_run`, `nist::matrix_rank`, `nist::non_overlapping_template` (×148), `nist::overlapping_template`, `nist::runs`, `nist::serial_delta1`, `nist::spectral`, `nist::universal`
- **LCG MINSTD (seed=1)**: 691/739 — `diehard::binary_rank_31x31`, `diehard::binary_rank_32x32`, `diehard::bitstream`, `diehard::count_ones_stream`, `diehard::craps_throws`, `diehard::craps_wins`, `diehard::dna`, `diehard::minimum_distance_2d`, `diehard::parking_lot`, `diehard::spheres_3d`, `diehard::squeeze`, `dieharder::bit_distribution` (×510), `dieharder::byte_distribution`, `dieharder::dct`, `dieharder::gcd_step_counts`, `dieharder::ks_uniform`, `dieharder::lagged_sums` (×2), `dieharder::minimum_distance_nd`, `dieharder::monobit2`, `maurer::universal_l06`, `maurer::universal_l08`, `maurer::universal_l09`, `maurer::universal_l10`, `nist::approximate_entropy`, `nist::block_frequency`, `nist::cumulative_sums_backward`, `nist::cumulative_sums_forward`, `nist::frequency`, `nist::longest_run`, `nist::matrix_rank`, `nist::non_overlapping_template` (×146), `nist::overlapping_template`, `nist::runs`, `nist::serial_delta1`, `nist::spectral`, `nist::universal`
- **BAD Borland C++ rand() LCG (seed=1)**: 11/739 — `diehard::opso`, `dieharder::bit_distribution` (×8), `nist::non_overlapping_template`, `nist::spectral`
- **AES-128-CTR (NIST key)**: 5/739 — `dieharder::bit_distribution` (×3), `nist::non_overlapping_template`, `nist::overlapping_template`
- **Camellia-128-CTR (key=00..0f)**: 9/739 — `dieharder::bit_distribution` (×9)
- **Twofish-128-CTR (key=00..0f)**: 11/739 — `diehard::runs_down`, `dieharder::bit_distribution` (×6), `nist::non_overlapping_template` (×4)
- **Serpent-128-CTR (key=00..0f)**: 6/739 — `dieharder::bit_distribution` (×5), `nist::non_overlapping_template`
- **SM4-CTR (key=00..0f)**: 8/739 — `dieharder::bit_distribution` (×7), `nist::serial_delta2`
- **Grasshopper-CTR (key=00..1f)**: 6/739 — `dieharder::bit_distribution` (×5), `nist::non_overlapping_template`
- **CAST-128-CTR (key=00..0f)**: 8/739 — `diehard::runs_down`, `dieharder::bit_distribution` (×6), `nist::non_overlapping_template`
- **SEED-CTR (key=00..0f)**: 12/739 — `dieharder::bit_distribution` (×10), `nist::non_overlapping_template` (×2)
- **Rabbit (key=00..0f, iv=00..07)**: 9/739 — `dieharder::bit_distribution` (×6), `maurer::universal_l09`, `nist::non_overlapping_template` (×2)
- **Salsa20 (key=00..1f, nonce=00..07)**: 8/739 — `dieharder::bit_distribution` (×5), `nist::non_overlapping_template`, `nist::serial_delta1`, `nist::serial_delta2`
- **Snow3G (key=00..0f, iv=00..0f)**: 7/739 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template` (×3)
- **ZUC-128 (key=00..0f, iv=00..0f)**: 4/739 — `dieharder::bit_distribution` (×4)
- **SpongeBob (SHA3-512 chain, OsRng seed)**: 2/739 — `dieharder::bit_distribution`, `nist::non_overlapping_template`
- **Squidward (SHA-256 chain, OsRng seed)**: 7/739 — `dieharder::bit_distribution` (×7)
- **PCG32 (OsRng seed)**: 8/739 — `dieharder::bit_distribution` (×8)
- **PCG64 (OsRng seed)**: 6/739 — `dieharder::bit_distribution` (×5), `nist::non_overlapping_template`
- **Xoshiro256 (OsRng seed)**: 13/739 — `diehard::binary_rank_32x32`, `dieharder::bit_distribution` (×9), `nist::non_overlapping_template` (×3)
- **Xoroshiro128 (OsRng seed)**: 5/739 — `dieharder::bit_distribution` (×4), `nist::random_excursions`
- **SFC64 (OsRng seed)**: 6/739 — `dieharder::bit_distribution` (×5), `nist::non_overlapping_template`
- **JSF64 (OsRng seed)**: 11/739 — `dieharder::bit_distribution` (×7), `nist::non_overlapping_template` (×4)
- **ChaCha20 CSPRNG (OsRng key)**: 6/739 — `dieharder::bit_distribution` (×4), `nist::non_overlapping_template` (×2)
- **HMAC_DRBG SHA-256 (OsRng seed)**: 8/739 — `dieharder::bit_distribution` (×7), `nist::non_overlapping_template`
- **Hash_DRBG SHA-256 (OsRng seed)**: 6/739 — `dieharder::bit_distribution` (×4), `dieharder::fill_tree_count`, `nist::longest_run`
- **cryptography::CtrDrbgAes256 (seed=00..2f)**: 8/739 — `dieharder::bit_distribution` (×3), `nist::non_overlapping_template` (×5)
- **Constant (0xDEAD_DEAD)**: 707/739 — expected for degenerate generator.
- **Counter (0,1,2,…)**: 706/739 — expected for degenerate generator.
- **Dual_EC_DRBG P-256 (NIST Q, seed=0x00..01)**: 2/200 — `nist::non_overlapping_template` (×2)

## Bottom Line

- Degenerate generators (Constant, Counter) and legacy PRNGs (ANSI C LCG, MINSTD, VB6 Rnd) remain annihilated — the battery continues to distinguish garbage from structure.
- Among non-trivial generators, the lowest FAIL count is **2** (`SpongeBob (SHA3-512 chain, OsRng seed)`) and the highest is **13** (`Xorshift32 (seed=1)`).
- Isolated failures in `non_overlapping_template` and `bit_distribution` are expected at α = 0.01; they are noise unless they form a family cluster.

## Auxiliary Probes

These probes are not part of `run_tests`; they are recorded separately here
from `tests/run_all.sh` on `darby.local` (2026-09-16).

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
  [PASS] wald_wolfowitz::runs_median                       p = 0.582269  (median=0.500112, below=100000, above=100000, runs=100124, z=0.5501)
  [INFO] approx_entropy_m02   ApEn=0.693144 (phi_m=-1.386294, phi_m1=-2.079438)
  [INFO] approx_entropy_m03   ApEn=0.693144 (phi_m=-2.079438, phi_m1=-2.772581)
  [INFO] approx_entropy_m04   ApEn=0.693141 (phi_m=-2.772581, phi_m1=-3.465723)
  [INFO] approx_entropy_m05   ApEn=0.693133 (phi_m=-3.465723, phi_m1=-4.158856)
  [INFO] approx_entropy_m06   ApEn=0.693118 (phi_m=-4.158856, phi_m1=-4.851974)

Xorshift32
  [PASS] knuth::permutation                                p = 0.622050  (t=5, blocks=40000, χ²=113.6180, df=119)
  [PASS] knuth::gap                                        p = 0.237153  ([0.250,0.500) gaps=50065, r=15, cells=16, χ²=18.5028, df=15)
  [PASS] wald_wolfowitz::runs_median                       p = 0.190081  (median=0.499376, below=100000, above=100000, runs=100294, z=1.3103)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693145 (phi_m=-2.079439, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693138 (phi_m=-2.772583, phi_m1=-3.465722)
  [INFO] approx_entropy_m05   ApEn=0.693133 (phi_m=-3.465722, phi_m1=-4.158855)
  [INFO] approx_entropy_m06   ApEn=0.693115 (phi_m=-4.158855, phi_m1=-4.851970)

Xorshift64
  [PASS] knuth::permutation                                p = 0.606131  (t=5, blocks=40000, χ²=114.2420, df=119)
  [PASS] knuth::gap                                        p = 0.750205  ([0.250,0.500) gaps=50116, r=15, cells=16, χ²=11.0337, df=15)
  [PASS] wald_wolfowitz::runs_median                       p = 0.134089  (median=0.499730, below=100000, above=100000, runs=99666, z=-1.4982)
  [INFO] approx_entropy_m02   ApEn=0.693147 (phi_m=-1.386294, phi_m1=-2.079441)
  [INFO] approx_entropy_m03   ApEn=0.693143 (phi_m=-2.079441, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693136 (phi_m=-2.772583, phi_m1=-3.465720)
  [INFO] approx_entropy_m05   ApEn=0.693130 (phi_m=-3.465720, phi_m1=-4.158850)
  [INFO] approx_entropy_m06   ApEn=0.693121 (phi_m=-4.158850, phi_m1=-4.851971)

BAD Unix System V rand()
  [PASS] knuth::permutation                                p = 0.410880  (t=5, blocks=40000, χ²=121.8320, df=119)
  [PASS] knuth::gap                                        p = 0.397280  ([0.250,0.500) gaps=50005, r=15, cells=16, χ²=15.7732, df=15)
  [PASS] wald_wolfowitz::runs_median                       p = 0.576149  (median=0.499845, below=100000, above=100000, runs=99876, z=-0.5590)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079440)
  [INFO] approx_entropy_m03   ApEn=0.693145 (phi_m=-2.079440, phi_m1=-2.772584)
  [INFO] approx_entropy_m04   ApEn=0.693137 (phi_m=-2.772584, phi_m1=-3.465721)
  [INFO] approx_entropy_m05   ApEn=0.693129 (phi_m=-3.465721, phi_m1=-4.158850)
  [INFO] approx_entropy_m06   ApEn=0.693108 (phi_m=-4.158850, phi_m1=-4.851957)

BAD Unix System V mrand48()
  [PASS] knuth::permutation                                p = 0.026338  (t=5, blocks=40000, χ²=150.6800, df=119)
  [PASS] knuth::gap                                        p = 0.887223  ([0.250,0.500) gaps=49804, r=15, cells=16, χ²=8.8104, df=15)
  [PASS] wald_wolfowitz::runs_median                       p = 0.582269  (median=0.501971, below=100000, above=100000, runs=100124, z=0.5501)
  [INFO] approx_entropy_m02   ApEn=0.693145 (phi_m=-1.386294, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693144 (phi_m=-2.079439, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693135 (phi_m=-2.772583, phi_m1=-3.465718)
  [INFO] approx_entropy_m05   ApEn=0.693130 (phi_m=-3.465718, phi_m1=-4.158848)
  [INFO] approx_entropy_m06   ApEn=0.693115 (phi_m=-4.158848, phi_m1=-4.851964)

BAD Unix BSD random()
  [PASS] knuth::permutation                                p = 0.801062  (t=5, blocks=40000, χ²=105.8060, df=119)
  [PASS] knuth::gap                                        p = 0.823636  ([0.250,0.500) gaps=50051, r=15, cells=16, χ²=9.9378, df=15)
  [PASS] wald_wolfowitz::runs_median                       p = 0.255988  (median=0.499583, below=100000, above=100000, runs=100255, z=1.1359)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386294, phi_m1=-2.079441)
  [INFO] approx_entropy_m03   ApEn=0.693146 (phi_m=-2.079441, phi_m1=-2.772587)
  [INFO] approx_entropy_m04   ApEn=0.693143 (phi_m=-2.772587, phi_m1=-3.465730)
  [INFO] approx_entropy_m05   ApEn=0.693137 (phi_m=-3.465730, phi_m1=-4.158866)
  [INFO] approx_entropy_m06   ApEn=0.693119 (phi_m=-4.158866, phi_m1=-4.851985)

BAD Unix Linux glibc rand()/random()
  [PASS] knuth::permutation                                p = 0.801062  (t=5, blocks=40000, χ²=105.8060, df=119)
  [PASS] knuth::gap                                        p = 0.823636  ([0.250,0.500) gaps=50051, r=15, cells=16, χ²=9.9378, df=15)
  [PASS] wald_wolfowitz::runs_median                       p = 0.255988  (median=0.499583, below=100000, above=100000, runs=100255, z=1.1359)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386294, phi_m1=-2.079441)
  [INFO] approx_entropy_m03   ApEn=0.693146 (phi_m=-2.079441, phi_m1=-2.772587)
  [INFO] approx_entropy_m04   ApEn=0.693143 (phi_m=-2.772587, phi_m1=-3.465730)
  [INFO] approx_entropy_m05   ApEn=0.693137 (phi_m=-3.465730, phi_m1=-4.158866)
  [INFO] approx_entropy_m06   ApEn=0.693119 (phi_m=-4.158866, phi_m1=-4.851985)

BAD Windows CRT rand()
  [PASS] knuth::permutation                                p = 0.086299  (t=5, blocks=40000, χ²=140.5640, df=119)
  [PASS] knuth::gap                                        p = 0.170906  ([0.250,0.500) gaps=50111, r=15, cells=16, χ²=20.0268, df=15)
  [PASS] wald_wolfowitz::runs_median                       p = 0.208872  (median=0.500955, below=100000, above=100000, runs=99720, z=-1.2567)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693142 (phi_m=-2.079439, phi_m1=-2.772581)
  [INFO] approx_entropy_m04   ApEn=0.693132 (phi_m=-2.772581, phi_m1=-3.465713)
  [INFO] approx_entropy_m05   ApEn=0.693124 (phi_m=-3.465713, phi_m1=-4.158837)
  [INFO] approx_entropy_m06   ApEn=0.693098 (phi_m=-4.158837, phi_m1=-4.851935)

BAD Windows VB6/VBA Rnd()
  [PASS] knuth::permutation                                p = 0.668822  (t=5, blocks=40000, χ²=111.7460, df=119)
  [PASS] knuth::gap                                        p = 0.114820  ([0.250,0.500) gaps=49910, r=15, cells=16, χ²=21.7394, df=15)
  [PASS] wald_wolfowitz::runs_median                       p = 0.431225  (median=0.498848, below=100000, above=100000, runs=100177, z=0.7871)
  [INFO] approx_entropy_m02   ApEn=0.693147 (phi_m=-1.386294, phi_m1=-2.079440)
  [INFO] approx_entropy_m03   ApEn=0.693146 (phi_m=-2.079440, phi_m1=-2.772586)
  [INFO] approx_entropy_m04   ApEn=0.693140 (phi_m=-2.772586, phi_m1=-3.465726)
  [INFO] approx_entropy_m05   ApEn=0.693136 (phi_m=-3.465726, phi_m1=-4.158862)
  [INFO] approx_entropy_m06   ApEn=0.693124 (phi_m=-4.158862, phi_m1=-4.851986)

BAD Windows .NET Random(seed)
  [PASS] knuth::permutation                                p = 0.634953  (t=5, blocks=40000, χ²=113.1080, df=119)
  [PASS] knuth::gap                                        p = 0.906751  ([0.250,0.500) gaps=50074, r=15, cells=16, χ²=8.3999, df=15)
  [PASS] wald_wolfowitz::runs_median                       p = 0.310021  (median=0.498075, below=100000, above=100000, runs=100228, z=1.0152)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386293, phi_m1=-2.079440)
  [INFO] approx_entropy_m03   ApEn=0.693144 (phi_m=-2.079440, phi_m1=-2.772584)
  [INFO] approx_entropy_m04   ApEn=0.693140 (phi_m=-2.772584, phi_m1=-3.465724)
  [INFO] approx_entropy_m05   ApEn=0.693134 (phi_m=-3.465724, phi_m1=-4.158858)
  [INFO] approx_entropy_m06   ApEn=0.693114 (phi_m=-4.158858, phi_m1=-4.851972)

ANSI C sample LCG
  [PASS] knuth::permutation                                p = 0.861091  (t=5, blocks=40000, χ²=102.4220, df=119)
  [FAIL] knuth::gap                                        p = 0.000000  ([0.250,0.500) gaps=100484, r=15, cells=16, χ²=51401.6062, df=15)
  [FAIL] wald_wolfowitz::runs_median                       p = 0.008891  (median=0.251237, below=100000, above=100000, runs=99416, z=-2.6162)
  [INFO] approx_entropy_m02   ApEn=0.692679 (phi_m=-1.385362, phi_m1=-2.078042)
  [INFO] approx_entropy_m03   ApEn=0.692678 (phi_m=-2.078042, phi_m1=-2.770720)
  [INFO] approx_entropy_m04   ApEn=0.692676 (phi_m=-2.770720, phi_m1=-3.463396)
  [INFO] approx_entropy_m05   ApEn=0.692669 (phi_m=-3.463396, phi_m1=-4.156065)
  [INFO] approx_entropy_m06   ApEn=0.692659 (phi_m=-4.156065, phi_m1=-4.848724)

LCG MINSTD
  [PASS] knuth::permutation                                p = 0.151153  (t=5, blocks=40000, χ²=134.9120, df=119)
  [FAIL] knuth::gap                                        p = 0.000000  ([0.250,0.500) gaps=100130, r=15, cells=16, χ²=49909.9252, df=15)
  [FAIL] wald_wolfowitz::runs_median                       p = 0.007897  (median=0.250342, below=100000, above=100000, runs=100595, z=2.6565)
  [INFO] approx_entropy_m02   ApEn=0.692637 (phi_m=-1.385279, phi_m1=-2.077916)
  [INFO] approx_entropy_m03   ApEn=0.692632 (phi_m=-2.077916, phi_m1=-2.770549)
  [INFO] approx_entropy_m04   ApEn=0.692628 (phi_m=-2.770549, phi_m1=-3.463177)
  [INFO] approx_entropy_m05   ApEn=0.692612 (phi_m=-3.463177, phi_m1=-4.155788)
  [INFO] approx_entropy_m06   ApEn=0.692596 (phi_m=-4.155788, phi_m1=-4.848384)

AES-128-CTR
  [PASS] knuth::permutation                                p = 0.224876  (t=5, blocks=40000, χ²=130.3400, df=119)
  [PASS] knuth::gap                                        p = 0.111037  ([0.250,0.500) gaps=50076, r=15, cells=16, χ²=21.8782, df=15)
  [PASS] wald_wolfowitz::runs_median                       p = 0.508050  (median=0.499150, below=100000, above=100000, runs=100149, z=0.6619)
  [INFO] approx_entropy_m02   ApEn=0.693146 (phi_m=-1.386294, phi_m1=-2.079439)
  [INFO] approx_entropy_m03   ApEn=0.693143 (phi_m=-2.079439, phi_m1=-2.772583)
  [INFO] approx_entropy_m04   ApEn=0.693139 (phi_m=-2.772583, phi_m1=-3.465722)
  [INFO] approx_entropy_m05   ApEn=0.693132 (phi_m=-3.465722, phi_m1=-4.158853)
  [INFO] approx_entropy_m06   ApEn=0.693118 (phi_m=-4.158853, phi_m1=-4.851972)

cryptography::CtrDrbgAes256
  [PASS] knuth::permutation                                p = 0.357595  (t=5, blocks=40000, χ²=124.0340, df=119)
  [PASS] knuth::gap                                        p = 0.760668  ([0.250,0.500) gaps=50101, r=15, cells=16, χ²=10.8855, df=15)
  [PASS] wald_wolfowitz::runs_median                       p = 0.205650  (median=0.500309, below=100000, above=100000, runs=99718, z=-1.2656)
  [INFO] approx_entropy_m02   ApEn=0.693147 (phi_m=-1.386294, phi_m1=-2.079441)
  [INFO] approx_entropy_m03   ApEn=0.693145 (phi_m=-2.079441, phi_m1=-2.772586)
  [INFO] approx_entropy_m04   ApEn=0.693141 (phi_m=-2.772586, phi_m1=-3.465727)
  [INFO] approx_entropy_m05   ApEn=0.693134 (phi_m=-3.465727, phi_m1=-4.158861)
  [INFO] approx_entropy_m06   ApEn=0.693120 (phi_m=-4.158861, phi_m1=-4.851981)


========================================================================
upstream_tests  (Hamming correlation/independence · FPF)
========================================================================

MT19937
  [PASS] testu01::hamming_corr                             p = 0.371781  (n=500000, r=20, s=10, L=300, rho_hat=-0.001263, z=-0.8931)
  [PASS] testu01::hamming_indep_main                       p = 0.331202  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2237.4752)
  [PASS] testu01::hamming_indep_block                      p = 0.933386  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.1379)
  [PASS] practrand::fpf_cross                              p = 0.425415  (samples=8388834, sig_bits=14, max_exp=63, dof=19, chi2=19.4967)
  [PASS] practrand::fpf_platter                            p = 0.899734  (samples=8388834, e=0, sig_bins=2^14, dof=16383, chi2=16151.7250)
  [PASS] practrand::fpf_platter                            p = 0.162113  (samples=8388834, e=1, sig_bins=2^14, dof=16383, chi2=16561.4228)
  [PASS] practrand::fpf_platter                            p = 0.498961  (samples=8388834, e=2, sig_bins=2^14, dof=16383, chi2=16382.8046)
  [PASS] practrand::fpf_platter                            p = 0.327181  (samples=8388834, e=3, sig_bins=2^14, dof=16383, chi2=16463.5070)
  [PASS] practrand::fpf_platter                            p = 0.497863  (samples=8388834, e=4, sig_bins=2^14, dof=16383, chi2=16383.3032)
  [PASS] practrand::fpf_platter                            p = 0.114337  (samples=8388834, e=5, sig_bins=2^13, dof=8191, chi2=8345.3682)
  [PASS] practrand::fpf_platter                            p = 0.695702  (samples=8388834, e=6, sig_bins=2^12, dof=4095, chi2=4048.1699)
  [PASS] practrand::fpf_platter                            p = 0.972453  (samples=8388834, e=7, sig_bins=2^11, dof=2047, chi2=1926.0662)
  [INFO] practrand::fpf_more                   10 additional platter results omitted

Xorshift32
  [PASS] testu01::hamming_corr                             p = 0.191457  (n=500000, r=20, s=10, L=300, rho_hat=-0.001847, z=-1.3063)
  [PASS] testu01::hamming_indep_main                       p = 0.527734  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2203.7128)
  [PASS] testu01::hamming_indep_block                      p = 0.348958  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=2.1056)
  [PASS] practrand::fpf_cross                              p = 0.832827  (samples=8388382, sig_bits=14, max_exp=63, dof=19, chi2=13.1113)
  [PASS] practrand::fpf_platter                            p = 0.558563  (samples=8388382, e=0, sig_bins=2^14, dof=16383, chi2=16355.6803)
  [PASS] practrand::fpf_platter                            p = 0.279727  (samples=8388382, e=1, sig_bins=2^14, dof=16383, chi2=16488.2073)
  [PASS] practrand::fpf_platter                            p = 0.210572  (samples=8388382, e=2, sig_bins=2^14, dof=16383, chi2=16528.3759)
  [FAIL] practrand::fpf_platter                            p = 0.002997  (samples=8388382, e=3, sig_bins=2^14, dof=16383, chi2=16884.8128)
  [PASS] practrand::fpf_platter                            p = 0.437089  (samples=8388382, e=4, sig_bins=2^13, dof=8191, chi2=8210.6171)
  [PASS] practrand::fpf_platter                            p = 0.402201  (samples=8388382, e=5, sig_bins=2^12, dof=4095, chi2=4116.7846)
  [PASS] practrand::fpf_platter                            p = 0.916158  (samples=8388382, e=6, sig_bins=2^11, dof=2047, chi2=1959.3363)
  [PASS] practrand::fpf_platter                            p = 0.663282  (samples=8388382, e=7, sig_bins=2^10, dof=1023, chi2=1003.3961)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

Xorshift64
  [PASS] testu01::hamming_corr                             p = 0.071751  (n=500000, r=20, s=10, L=300, rho_hat=-0.002547, z=-1.8007)
  [PASS] testu01::hamming_indep_main                       p = 0.515812  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2205.6998)
  [PASS] testu01::hamming_indep_block                      p = 0.935703  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.1329)
  [PASS] practrand::fpf_cross                              p = 0.835378  (samples=8388671, sig_bits=14, max_exp=63, dof=19, chi2=13.0621)
  [PASS] practrand::fpf_platter                            p = 0.843963  (samples=8388671, e=0, sig_bins=2^14, dof=16383, chi2=16200.0349)
  [PASS] practrand::fpf_platter                            p = 0.825083  (samples=8388671, e=1, sig_bins=2^14, dof=16383, chi2=16213.6875)
  [PASS] practrand::fpf_platter                            p = 0.899475  (samples=8388671, e=2, sig_bins=2^14, dof=16383, chi2=16151.9891)
  [PASS] practrand::fpf_platter                            p = 0.084495  (samples=8388671, e=3, sig_bins=2^14, dof=16383, chi2=16632.5669)
  [PASS] practrand::fpf_platter                            p = 0.174825  (samples=8388671, e=4, sig_bins=2^14, dof=16383, chi2=16552.2092)
  [PASS] practrand::fpf_platter                            p = 0.013549  (samples=8388671, e=5, sig_bins=2^13, dof=8191, chi2=8476.4598)
  [PASS] practrand::fpf_platter                            p = 0.367985  (samples=8388671, e=6, sig_bins=2^12, dof=4095, chi2=4124.9219)
  [PASS] practrand::fpf_platter                            p = 0.371440  (samples=8388671, e=7, sig_bins=2^11, dof=2047, chi2=2067.3907)
  [INFO] practrand::fpf_more                   10 additional platter results omitted

BAD Unix System V rand()
  [PASS] testu01::hamming_corr                             p = 0.423715  (n=500000, r=20, s=10, L=300, rho_hat=0.001131, z=0.8000)
  [PASS] testu01::hamming_indep_main                       p = 0.273523  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2248.5950)
  [PASS] testu01::hamming_indep_block                      p = 0.336313  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=2.1794)
  [PASS] practrand::fpf_cross                              p = 0.253946  (samples=8388697, sig_bits=14, max_exp=63, dof=19, chi2=22.6319)
  [PASS] practrand::fpf_platter                            p = 0.914281  (samples=8388697, e=0, sig_bins=2^14, dof=16383, chi2=16136.0304)
  [PASS] practrand::fpf_platter                            p = 0.476354  (samples=8388697, e=1, sig_bins=2^14, dof=16383, chi2=16393.0709)
  [PASS] practrand::fpf_platter                            p = 0.601921  (samples=8388697, e=2, sig_bins=2^14, dof=16383, chi2=16335.6191)
  [PASS] practrand::fpf_platter                            p = 0.299939  (samples=8388697, e=3, sig_bins=2^14, dof=16383, chi2=16477.4702)
  [PASS] practrand::fpf_platter                            p = 0.015711  (samples=8388697, e=4, sig_bins=2^14, dof=16383, chi2=16774.8995)
  [PASS] practrand::fpf_platter                            p = 0.664469  (samples=8388697, e=5, sig_bins=2^13, dof=8191, chi2=8136.0988)
  [PASS] practrand::fpf_platter                            p = 0.251539  (samples=8388697, e=6, sig_bins=2^12, dof=4095, chi2=4155.2299)
  [PASS] practrand::fpf_platter                            p = 0.547063  (samples=8388697, e=7, sig_bins=2^11, dof=2047, chi2=2038.7783)
  [INFO] practrand::fpf_more                   10 additional platter results omitted

BAD Unix System V mrand48()
  [PASS] testu01::hamming_corr                             p = 0.723207  (n=500000, r=20, s=10, L=300, rho_hat=-0.000501, z=-0.3542)
  [PASS] testu01::hamming_indep_main                       p = 0.153387  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2276.9508)
  [PASS] testu01::hamming_indep_block                      p = 0.078374  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=5.0925)
  [PASS] practrand::fpf_cross                              p = 0.609658  (samples=8388833, sig_bits=14, max_exp=63, dof=19, chi2=16.7078)
  [PASS] practrand::fpf_platter                            p = 0.723039  (samples=8388833, e=0, sig_bins=2^14, dof=16383, chi2=16275.4285)
  [PASS] practrand::fpf_platter                            p = 0.965097  (samples=8388833, e=1, sig_bins=2^14, dof=16383, chi2=16056.3205)
  [PASS] practrand::fpf_platter                            p = 0.523587  (samples=8388833, e=2, sig_bins=2^14, dof=16383, chi2=16371.6273)
  [PASS] practrand::fpf_platter                            p = 0.563120  (samples=8388833, e=3, sig_bins=2^14, dof=16383, chi2=16353.5905)
  [PASS] practrand::fpf_platter                            p = 0.259842  (samples=8388833, e=4, sig_bins=2^14, dof=16383, chi2=16499.1497)
  [PASS] practrand::fpf_platter                            p = 0.036824  (samples=8388833, e=5, sig_bins=2^13, dof=8191, chi2=8421.4123)
  [PASS] practrand::fpf_platter                            p = 0.734321  (samples=8388833, e=6, sig_bins=2^12, dof=4095, chi2=4037.9535)
  [PASS] practrand::fpf_platter                            p = 0.127270  (samples=8388833, e=7, sig_bins=2^11, dof=2047, chi2=2120.0908)
  [INFO] practrand::fpf_more                   10 additional platter results omitted

BAD Unix BSD random()
  [PASS] testu01::hamming_corr                             p = 0.743907  (n=500000, r=20, s=10, L=300, rho_hat=-0.000462, z=-0.3267)
  [PASS] testu01::hamming_indep_main                       p = 0.795139  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2154.0001)
  [PASS] testu01::hamming_indep_block                      p = 0.698912  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.7165)
  [PASS] practrand::fpf_cross                              p = 0.167061  (samples=8388328, sig_bits=14, max_exp=63, dof=19, chi2=24.8050)
  [PASS] practrand::fpf_platter                            p = 0.509799  (samples=8388328, e=0, sig_bins=2^14, dof=16383, chi2=16377.8873)
  [PASS] practrand::fpf_platter                            p = 0.020835  (samples=8388328, e=1, sig_bins=2^14, dof=16383, chi2=16753.7829)
  [PASS] practrand::fpf_platter                            p = 0.467219  (samples=8388328, e=2, sig_bins=2^14, dof=16383, chi2=16397.2280)
  [PASS] practrand::fpf_platter                            p = 0.679459  (samples=8388328, e=3, sig_bins=2^14, dof=16383, chi2=16298.0942)
  [PASS] practrand::fpf_platter                            p = 0.548268  (samples=8388328, e=4, sig_bins=2^13, dof=8191, chi2=8174.8203)
  [PASS] practrand::fpf_platter                            p = 0.517017  (samples=8388328, e=5, sig_bins=2^12, dof=4095, chi2=4090.4735)
  [PASS] practrand::fpf_platter                            p = 0.714845  (samples=8388328, e=6, sig_bins=2^11, dof=2047, chi2=2010.2375)
  [PASS] practrand::fpf_platter                            p = 0.140398  (samples=8388328, e=7, sig_bins=2^10, dof=1023, chi2=1071.8784)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Unix Linux glibc rand()/random()
  [PASS] testu01::hamming_corr                             p = 0.743907  (n=500000, r=20, s=10, L=300, rho_hat=-0.000462, z=-0.3267)
  [PASS] testu01::hamming_indep_main                       p = 0.795139  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2154.0001)
  [PASS] testu01::hamming_indep_block                      p = 0.698912  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.7165)
  [PASS] practrand::fpf_cross                              p = 0.167061  (samples=8388328, sig_bits=14, max_exp=63, dof=19, chi2=24.8050)
  [PASS] practrand::fpf_platter                            p = 0.509799  (samples=8388328, e=0, sig_bins=2^14, dof=16383, chi2=16377.8873)
  [PASS] practrand::fpf_platter                            p = 0.020835  (samples=8388328, e=1, sig_bins=2^14, dof=16383, chi2=16753.7829)
  [PASS] practrand::fpf_platter                            p = 0.467219  (samples=8388328, e=2, sig_bins=2^14, dof=16383, chi2=16397.2280)
  [PASS] practrand::fpf_platter                            p = 0.679459  (samples=8388328, e=3, sig_bins=2^14, dof=16383, chi2=16298.0942)
  [PASS] practrand::fpf_platter                            p = 0.548268  (samples=8388328, e=4, sig_bins=2^13, dof=8191, chi2=8174.8203)
  [PASS] practrand::fpf_platter                            p = 0.517017  (samples=8388328, e=5, sig_bins=2^12, dof=4095, chi2=4090.4735)
  [PASS] practrand::fpf_platter                            p = 0.714845  (samples=8388328, e=6, sig_bins=2^11, dof=2047, chi2=2010.2375)
  [PASS] practrand::fpf_platter                            p = 0.140398  (samples=8388328, e=7, sig_bins=2^10, dof=1023, chi2=1071.8784)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

BAD Windows CRT rand()
  [PASS] testu01::hamming_corr                             p = 0.240982  (n=500000, r=20, s=10, L=300, rho_hat=-0.001658, z=-1.1725)
  [PASS] testu01::hamming_indep_main                       p = 0.890950  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2127.4942)
  [FAIL] testu01::hamming_indep_block                      p = 0.004296  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=10.9001)
  [PASS] practrand::fpf_cross                              p = 0.205634  (samples=8389066, sig_bits=14, max_exp=63, dof=19, chi2=23.7571)
  [PASS] practrand::fpf_platter                            p = 0.275991  (samples=8389066, e=0, sig_bins=2^14, dof=16383, chi2=16490.2323)
  [PASS] practrand::fpf_platter                            p = 0.343549  (samples=8389066, e=1, sig_bins=2^14, dof=16383, chi2=16455.3513)
  [PASS] practrand::fpf_platter                            p = 0.083778  (samples=8389066, e=2, sig_bins=2^14, dof=16383, chi2=16633.4160)
  [PASS] practrand::fpf_platter                            p = 0.105132  (samples=8389066, e=3, sig_bins=2^14, dof=16383, chi2=16610.1565)
  [PASS] practrand::fpf_platter                            p = 0.112067  (samples=8389066, e=4, sig_bins=2^14, dof=16383, chi2=16603.3561)
  [PASS] practrand::fpf_platter                            p = 0.028018  (samples=8389066, e=5, sig_bins=2^13, dof=8191, chi2=8437.3228)
  [PASS] practrand::fpf_platter                            p = 0.800391  (samples=8389066, e=6, sig_bins=2^12, dof=4095, chi2=4018.5216)
  [PASS] practrand::fpf_platter                            p = 0.191730  (samples=8389066, e=7, sig_bins=2^11, dof=2047, chi2=2102.5952)
  [INFO] practrand::fpf_more                   10 additional platter results omitted

BAD Windows VB6/VBA Rnd()
  [FAIL] testu01::hamming_corr                             p = 0.000000  (n=500000, r=20, s=10, L=300, rho_hat=-0.087334, z=-61.7546)
  [FAIL] testu01::hamming_indep_main                       p = 0.000000  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=60021.7661)
  [FAIL] testu01::hamming_indep_block                      p = 0.000000  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=3183.8786)
  [FAIL] practrand::fpf_cross                              p = 3.516e-155  (samples=8388981, sig_bits=14, max_exp=63, dof=19, chi2=789.5838)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=0, sig_bins=2^14, dof=16383, chi2=216877.1472)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=1, sig_bins=2^14, dof=16383, chi2=115556.5146)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=2, sig_bins=2^14, dof=16383, chi2=62565.4290)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=3, sig_bins=2^14, dof=16383, chi2=50472.9396)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=4, sig_bins=2^14, dof=16383, chi2=34721.8201)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8388981, e=5, sig_bins=2^13, dof=8191, chi2=16365.3480)
  [FAIL] practrand::fpf_platter                            p = 9.421e-319  (samples=8388981, e=6, sig_bins=2^12, dof=4095, chi2=8577.8104)
  [FAIL] practrand::fpf_platter                            p = 2.824e-160  (samples=8388981, e=7, sig_bins=2^11, dof=2047, chi2=4285.0227)
  [INFO] practrand::fpf_more                   10 additional platter results omitted

BAD Windows .NET Random(seed)
  [PASS] testu01::hamming_corr                             p = 0.467237  (n=500000, r=20, s=10, L=300, rho_hat=-0.001028, z=-0.7270)
  [PASS] testu01::hamming_indep_main                       p = 0.118997  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2287.6840)
  [PASS] testu01::hamming_indep_block                      p = 0.813001  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.4140)
  [PASS] practrand::fpf_cross                              p = 0.630666  (samples=8388481, sig_bits=14, max_exp=63, dof=19, chi2=16.3967)
  [PASS] practrand::fpf_platter                            p = 0.944301  (samples=8388481, e=0, sig_bins=2^14, dof=16383, chi2=16095.8641)
  [PASS] practrand::fpf_platter                            p = 0.087850  (samples=8388481, e=1, sig_bins=2^14, dof=16383, chi2=16628.6653)
  [PASS] practrand::fpf_platter                            p = 0.832866  (samples=8388481, e=2, sig_bins=2^14, dof=16383, chi2=16208.1801)
  [PASS] practrand::fpf_platter                            p = 0.808227  (samples=8388481, e=3, sig_bins=2^14, dof=16383, chi2=16225.1107)
  [PASS] practrand::fpf_platter                            p = 0.566746  (samples=8388481, e=4, sig_bins=2^13, dof=8191, chi2=8168.8383)
  [PASS] practrand::fpf_platter                            p = 0.417988  (samples=8388481, e=5, sig_bins=2^12, dof=4095, chi2=4113.0972)
  [PASS] practrand::fpf_platter                            p = 0.260935  (samples=8388481, e=6, sig_bins=2^11, dof=2047, chi2=2087.5793)
  [PASS] practrand::fpf_platter                            p = 0.995436  (samples=8388481, e=7, sig_bins=2^10, dof=1023, chi2=908.9340)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

ANSI C sample LCG
  [FAIL] testu01::hamming_corr                             p = 7.113e-143  (n=500000, r=20, s=10, L=300, rho_hat=-0.035991, z=-25.4497)
  [FAIL] testu01::hamming_indep_main                       p = 0.000000  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=813696.0622)
  [FAIL] testu01::hamming_indep_block                      p = 1.164e-282  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=1298.3545)
  [FAIL] practrand::fpf_cross                              p = 0.000000  (samples=8346698, sig_bits=14, max_exp=63, dof=19, chi2=34790.4838)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=0, sig_bins=2^14, dof=16383, chi2=95532.1155)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=1, sig_bins=2^14, dof=16383, chi2=65453.8367)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=2, sig_bins=2^14, dof=16383, chi2=59128.0603)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=3, sig_bins=2^14, dof=16383, chi2=67918.6734)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=4, sig_bins=2^13, dof=8191, chi2=34027.2844)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=5, sig_bins=2^12, dof=4095, chi2=15944.1870)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=6, sig_bins=2^11, dof=2047, chi2=7497.2604)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8346698, e=7, sig_bins=2^10, dof=1023, chi2=4164.0049)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

LCG MINSTD
  [PASS] testu01::hamming_corr                             p = 0.574341  (n=500000, r=20, s=10, L=300, rho_hat=-0.000794, z=-0.5617)
  [PASS] testu01::hamming_indep_main                       p = 0.318058  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2239.9257)
  [PASS] testu01::hamming_indep_block                      p = 0.088892  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=4.8407)
  [FAIL] practrand::fpf_cross                              p = 0.000000  (samples=8354530, sig_bits=14, max_exp=63, dof=19, chi2=18914.9527)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8354530, e=0, sig_bins=2^14, dof=16383, chi2=77218.4397)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8354530, e=1, sig_bins=2^14, dof=16383, chi2=46824.6122)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8354530, e=2, sig_bins=2^14, dof=16383, chi2=34686.9410)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8354530, e=3, sig_bins=2^14, dof=16383, chi2=35472.5001)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8354530, e=4, sig_bins=2^13, dof=8191, chi2=17452.3266)
  [FAIL] practrand::fpf_platter                            p = 0.000000  (samples=8354530, e=5, sig_bins=2^12, dof=4095, chi2=8690.6996)
  [FAIL] practrand::fpf_platter                            p = 1.093e-160  (samples=8354530, e=6, sig_bins=2^11, dof=2047, chi2=4288.6499)
  [FAIL] practrand::fpf_platter                            p = 1.634e-63  (samples=8354530, e=7, sig_bins=2^10, dof=1023, chi2=1979.4614)
  [INFO] practrand::fpf_more                   9 additional platter results omitted

AES-128-CTR
  [PASS] testu01::hamming_corr                             p = 0.604025  (n=500000, r=20, s=10, L=300, rho_hat=0.000733, z=0.5186)
  [PASS] testu01::hamming_indep_main                       p = 0.346377  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2234.6957)
  [PASS] testu01::hamming_indep_block                      p = 0.936284  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=0.1317)
  [PASS] practrand::fpf_cross                              p = 0.972140  (samples=8388629, sig_bits=14, max_exp=63, dof=19, chi2=9.0793)
  [PASS] practrand::fpf_platter                            p = 0.856875  (samples=8388629, e=0, sig_bins=2^14, dof=16383, chi2=16190.0652)
  [PASS] practrand::fpf_platter                            p = 0.566485  (samples=8388629, e=1, sig_bins=2^14, dof=16383, chi2=16352.0452)
  [PASS] practrand::fpf_platter                            p = 0.862661  (samples=8388629, e=2, sig_bins=2^14, dof=16383, chi2=16185.4016)
  [PASS] practrand::fpf_platter                            p = 0.435937  (samples=8388629, e=3, sig_bins=2^14, dof=16383, chi2=16411.5437)
  [PASS] practrand::fpf_platter                            p = 0.071000  (samples=8388629, e=4, sig_bins=2^14, dof=16383, chi2=16649.5641)
  [PASS] practrand::fpf_platter                            p = 0.565550  (samples=8388629, e=5, sig_bins=2^13, dof=8191, chi2=8169.2266)
  [PASS] practrand::fpf_platter                            p = 0.035436  (samples=8388629, e=6, sig_bins=2^12, dof=4095, chi2=4259.9676)
  [PASS] practrand::fpf_platter                            p = 0.148911  (samples=8388629, e=7, sig_bins=2^11, dof=2047, chi2=2113.6604)
  [INFO] practrand::fpf_more                   10 additional platter results omitted

cryptography::CtrDrbgAes256
  [PASS] testu01::hamming_corr                             p = 0.310631  (n=500000, r=20, s=10, L=300, rho_hat=0.001434, z=1.0139)
  [PASS] testu01::hamming_indep_main                       p = 0.763462  (n=500000, r=20, s=10, L=300, dof=2209, lumped_cells=88392, chi2=2160.9946)
  [PASS] testu01::hamming_indep_block                      p = 0.244947  (n=500000, r=20, s=10, L=300, d=1, dof=2, chi2=2.8134)
  [PASS] practrand::fpf_cross                              p = 0.527737  (samples=8388217, sig_bits=14, max_exp=63, dof=19, chi2=17.9208)
  [PASS] practrand::fpf_platter                            p = 0.337168  (samples=8388217, e=0, sig_bins=2^14, dof=16383, chi2=16458.5120)
  [PASS] practrand::fpf_platter                            p = 0.353430  (samples=8388217, e=1, sig_bins=2^14, dof=16383, chi2=16450.5013)
  [PASS] practrand::fpf_platter                            p = 0.699547  (samples=8388217, e=2, sig_bins=2^14, dof=16383, chi2=16287.8301)
  [PASS] practrand::fpf_platter                            p = 0.385410  (samples=8388217, e=3, sig_bins=2^14, dof=16383, chi2=16435.1184)
  [FAIL] practrand::fpf_platter                            p = 0.006559  (samples=8388217, e=4, sig_bins=2^13, dof=8191, chi2=8511.9231)
  [PASS] practrand::fpf_platter                            p = 0.042286  (samples=8388217, e=5, sig_bins=2^12, dof=4095, chi2=4252.3959)
  [PASS] practrand::fpf_platter                            p = 0.492483  (samples=8388217, e=6, sig_bins=2^11, dof=2047, chi2=2047.5391)
  [PASS] practrand::fpf_platter                            p = 0.421195  (samples=8388217, e=7, sig_bins=2^10, dof=1023, chi2=1031.3503)
  [INFO] practrand::fpf_more                   9 additional platter results omitted


========================================================================
testu01_lz  (Lempel-Ziv  k=25  replications=10)
========================================================================

MT19937
  [PASS] testu01::lzw_sum                                  p = 0.190009  (N=10, k=25, r=0, s=30, pit_seed=1, Z=-1.3106)
  [PASS] testu01::lzw_ks                                   p = 0.149165  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1762933 U=0.156179 z=-1.0103
  [INFO] testu01::lzw_rep02                    W=1762950 U=0.317314 z=-0.4752
  [INFO] testu01::lzw_rep03                    W=1762918 U=0.075535 z=-1.4358
  [INFO] testu01::lzw_rep04                    W=1762999 U=0.843154 z=1.0075
  [INFO] testu01::lzw_rep05                    W=1762971 U=0.567327 z=0.1696
  [INFO] testu01::lzw_rep06                    W=1762935 U=0.173859 z=-0.9390
  [INFO] testu01::lzw_rep07                    W=1762939 U=0.210948 z=-0.8031
  [INFO] testu01::lzw_rep08                    W=1762981 U=0.677739 z=0.4614
  [INFO] testu01::lzw_rep09                    W=1762940 U=0.225662 z=-0.7532
  [INFO] testu01::lzw_rep10                    W=1762953 U=0.357127 z=-0.3661

Xorshift32
  [PASS] testu01::lzw_sum                                  p = 0.926662  (N=10, k=25, r=0, s=30, pit_seed=1, Z=-0.0920)
  [PASS] testu01::lzw_ks                                   p = 0.972799  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1762985 U=0.715592 z=0.5698
  [INFO] testu01::lzw_rep02                    W=1762916 U=0.064821 z=-1.5155
  [INFO] testu01::lzw_rep03                    W=1762950 U=0.322283 z=-0.4613
  [INFO] testu01::lzw_rep04                    W=1762990 U=0.770940 z=0.7419
  [INFO] testu01::lzw_rep05                    W=1762959 U=0.423821 z=-0.1921
  [INFO] testu01::lzw_rep06                    W=1762943 U=0.247026 z=-0.6839
  [INFO] testu01::lzw_rep07                    W=1762962 U=0.459082 z=-0.1027
  [INFO] testu01::lzw_rep08                    W=1762978 U=0.647156 z=0.3777
  [INFO] testu01::lzw_rep09                    W=1763035 U=0.978662 z=2.0269
  [INFO] testu01::lzw_rep10                    W=1762931 U=0.146459 z=-1.0517

Xorshift64
  [PASS] testu01::lzw_sum                                  p = 0.131754  (N=10, k=25, r=0, s=30, pit_seed=1, Z=-1.5072)
  [PASS] testu01::lzw_ks                                   p = 0.114121  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1762946 U=0.274592 z=-0.5990
  [INFO] testu01::lzw_rep02                    W=1762978 U=0.644883 z=0.3715
  [INFO] testu01::lzw_rep03                    W=1762961 U=0.447918 z=-0.1309
  [INFO] testu01::lzw_rep04                    W=1762935 U=0.178910 z=-0.9195
  [INFO] testu01::lzw_rep05                    W=1762918 U=0.075044 z=-1.4392
  [INFO] testu01::lzw_rep06                    W=1762952 U=0.339646 z=-0.4134
  [INFO] testu01::lzw_rep07                    W=1762940 U=0.221477 z=-0.7672
  [INFO] testu01::lzw_rep08                    W=1763007 U=0.889641 z=1.2246
  [INFO] testu01::lzw_rep09                    W=1762943 U=0.253772 z=-0.6627
  [INFO] testu01::lzw_rep10                    W=1762918 U=0.076295 z=-1.4304

BAD Unix System V rand()
  [PASS] testu01::lzw_sum                                  p = 0.545233  (N=10, k=25, r=0, s=30, pit_seed=1, Z=-0.6049)
  [PASS] testu01::lzw_ks                                   p = 0.278731  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1762954 U=0.361037 z=-0.3557
  [INFO] testu01::lzw_rep02                    W=1762949 U=0.307228 z=-0.5037
  [INFO] testu01::lzw_rep03                    W=1762945 U=0.269408 z=-0.6146
  [INFO] testu01::lzw_rep04                    W=1762958 U=0.415098 z=-0.2144
  [INFO] testu01::lzw_rep05                    W=1762974 U=0.602342 z=0.2594
  [INFO] testu01::lzw_rep06                    W=1762936 U=0.183155 z=-0.9034
  [INFO] testu01::lzw_rep07                    W=1763024 U=0.957985 z=1.7278
  [INFO] testu01::lzw_rep08                    W=1762971 U=0.565723 z=0.1655
  [INFO] testu01::lzw_rep09                    W=1762951 U=0.336519 z=-0.4220
  [INFO] testu01::lzw_rep10                    W=1762931 U=0.146459 z=-1.0517

BAD Unix System V mrand48()
  [PASS] testu01::lzw_sum                                  p = 0.748611  (N=10, k=25, r=0, s=30, pit_seed=1, Z=0.3205)
  [PASS] testu01::lzw_ks                                   p = 0.883224  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1762884 U=0.008498 z=-2.3868
  [INFO] testu01::lzw_rep02                    W=1762984 U=0.707475 z=0.5460
  [INFO] testu01::lzw_rep03                    W=1762921 U=0.089821 z=-1.3419
  [INFO] testu01::lzw_rep04                    W=1762998 U=0.836558 z=0.9804
  [INFO] testu01::lzw_rep05                    W=1762951 U=0.331865 z=-0.4348
  [INFO] testu01::lzw_rep06                    W=1763024 U=0.957381 z=1.7211
  [INFO] testu01::lzw_rep07                    W=1763014 U=0.924491 z=1.4359
  [INFO] testu01::lzw_rep08                    W=1762970 U=0.554925 z=0.1381
  [INFO] testu01::lzw_rep09                    W=1762962 U=0.464433 z=-0.0893
  [INFO] testu01::lzw_rep10                    W=1762980 U=0.671675 z=0.4445

BAD Unix BSD random()
  [PASS] testu01::lzw_sum                                  p = 0.913444  (N=10, k=25, r=0, s=30, pit_seed=1, Z=-0.1087)
  [PASS] testu01::lzw_ks                                   p = 0.993820  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1762964 U=0.478113 z=-0.0549
  [INFO] testu01::lzw_rep02                    W=1763021 U=0.948391 z=1.6294
  [INFO] testu01::lzw_rep03                    W=1762904 U=0.032027 z=-1.8518
  [INFO] testu01::lzw_rep04                    W=1762982 U=0.693300 z=0.5052
  [INFO] testu01::lzw_rep05                    W=1763007 U=0.890324 z=1.2283
  [INFO] testu01::lzw_rep06                    W=1762961 U=0.443325 z=-0.1425
  [INFO] testu01::lzw_rep07                    W=1762949 U=0.310867 z=-0.4934
  [INFO] testu01::lzw_rep08                    W=1762981 U=0.677739 z=0.4614
  [INFO] testu01::lzw_rep09                    W=1762932 U=0.154946 z=-1.0154
  [INFO] testu01::lzw_rep10                    W=1762945 U=0.270944 z=-0.6100

BAD Unix Linux glibc rand()/random()
  [PASS] testu01::lzw_sum                                  p = 0.913444  (N=10, k=25, r=0, s=30, pit_seed=1, Z=-0.1087)
  [PASS] testu01::lzw_ks                                   p = 0.993820  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1762964 U=0.478113 z=-0.0549
  [INFO] testu01::lzw_rep02                    W=1763021 U=0.948391 z=1.6294
  [INFO] testu01::lzw_rep03                    W=1762904 U=0.032027 z=-1.8518
  [INFO] testu01::lzw_rep04                    W=1762982 U=0.693300 z=0.5052
  [INFO] testu01::lzw_rep05                    W=1763007 U=0.890324 z=1.2283
  [INFO] testu01::lzw_rep06                    W=1762961 U=0.443325 z=-0.1425
  [INFO] testu01::lzw_rep07                    W=1762949 U=0.310867 z=-0.4934
  [INFO] testu01::lzw_rep08                    W=1762981 U=0.677739 z=0.4614
  [INFO] testu01::lzw_rep09                    W=1762932 U=0.154946 z=-1.0154
  [INFO] testu01::lzw_rep10                    W=1762945 U=0.270944 z=-0.6100

BAD Windows CRT rand()
  [PASS] testu01::lzw_sum                                  p = 0.177062  (N=10, k=25, r=0, s=30, pit_seed=1, Z=1.3499)
  [PASS] testu01::lzw_ks                                   p = 0.453306  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1762952 U=0.338317 z=-0.4171
  [INFO] testu01::lzw_rep02                    W=1763026 U=0.961849 z=1.7726
  [INFO] testu01::lzw_rep03                    W=1762990 U=0.769659 z=0.7377
  [INFO] testu01::lzw_rep04                    W=1762961 U=0.449522 z=-0.1269
  [INFO] testu01::lzw_rep05                    W=1763020 U=0.946377 z=1.6107
  [INFO] testu01::lzw_rep06                    W=1762944 U=0.256224 z=-0.6550
  [INFO] testu01::lzw_rep07                    W=1762963 U=0.471103 z=-0.0725
  [INFO] testu01::lzw_rep08                    W=1762997 U=0.826199 z=0.9393
  [INFO] testu01::lzw_rep09                    W=1762973 U=0.595852 z=0.2426
  [INFO] testu01::lzw_rep10                    W=1762973 U=0.593758 z=0.2372

BAD Windows VB6/VBA Rnd()
  [FAIL] testu01::lzw_sum                                  p = 2.029e-35  (N=10, k=25, r=0, s=30, pit_seed=1, Z=-12.4203)
  [FAIL] testu01::lzw_ks                                   p = 3.550e-10  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1755485 U=0.000012 z=-4.2153
  [INFO] testu01::lzw_rep02                    W=1755545 U=0.000017 z=-4.1440
  [INFO] testu01::lzw_rep03                    W=1755563 U=0.000062 z=-3.8372
  [INFO] testu01::lzw_rep04                    W=1755535 U=0.000076 z=-3.7887
  [INFO] testu01::lzw_rep05                    W=1755551 U=0.000052 z=-3.8801
  [INFO] testu01::lzw_rep06                    W=1755521 U=0.000024 z=-4.0687
  [INFO] testu01::lzw_rep07                    W=1755536 U=0.000054 z=-3.8727
  [INFO] testu01::lzw_rep08                    W=1755473 U=0.000037 z=-3.9607
  [INFO] testu01::lzw_rep09                    W=1755512 U=0.000097 z=-3.7268
  [INFO] testu01::lzw_rep10                    W=1755564 U=0.000078 z=-3.7821

BAD Windows .NET Random(seed)
  [PASS] testu01::lzw_sum                                  p = 0.240978  (N=10, k=25, r=0, s=30, pit_seed=1, Z=1.1725)
  [PASS] testu01::lzw_ks                                   p = 0.199677  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1762967 U=0.515394 z=0.0386
  [INFO] testu01::lzw_rep02                    W=1762886 U=0.009449 z=-2.3475
  [INFO] testu01::lzw_rep03                    W=1762991 U=0.779531 z=0.7706
  [INFO] testu01::lzw_rep04                    W=1762976 U=0.629089 z=0.3294
  [INFO] testu01::lzw_rep05                    W=1763020 U=0.946377 z=1.6107
  [INFO] testu01::lzw_rep06                    W=1762976 U=0.622684 z=0.3125
  [INFO] testu01::lzw_rep07                    W=1762883 U=0.008167 z=-2.4013
  [INFO] testu01::lzw_rep08                    W=1763001 U=0.853421 z=1.0512
  [INFO] testu01::lzw_rep09                    W=1763018 U=0.941106 z=1.5641
  [INFO] testu01::lzw_rep10                    W=1763062 U=0.997278 z=2.7796

ANSI C sample LCG
  [FAIL] testu01::lzw_sum                                  p = 2.029e-35  (N=10, k=25, r=0, s=30, pit_seed=1, Z=-12.4203)
  [FAIL] testu01::lzw_ks                                   p = 3.550e-10  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1761859 U=0.000012 z=-4.2153
  [INFO] testu01::lzw_rep02                    W=1761837 U=0.000017 z=-4.1440
  [INFO] testu01::lzw_rep03                    W=1761768 U=0.000062 z=-3.8372
  [INFO] testu01::lzw_rep04                    W=1761798 U=0.000076 z=-3.7887
  [INFO] testu01::lzw_rep05                    W=1761764 U=0.000052 z=-3.8801
  [INFO] testu01::lzw_rep06                    W=1761816 U=0.000024 z=-4.0687
  [INFO] testu01::lzw_rep07                    W=1761741 U=0.000054 z=-3.8727
  [INFO] testu01::lzw_rep08                    W=1761768 U=0.000037 z=-3.9607
  [INFO] testu01::lzw_rep09                    W=1761800 U=0.000097 z=-3.7268
  [INFO] testu01::lzw_rep10                    W=1761780 U=0.000078 z=-3.7821

LCG MINSTD
  [FAIL] testu01::lzw_sum                                  p = 2.029e-35  (N=10, k=25, r=0, s=30, pit_seed=1, Z=-12.4203)
  [FAIL] testu01::lzw_ks                                   p = 3.550e-10  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1761746 U=0.000012 z=-4.2153
  [INFO] testu01::lzw_rep02                    W=1761663 U=0.000017 z=-4.1440
  [INFO] testu01::lzw_rep03                    W=1761733 U=0.000062 z=-3.8372
  [INFO] testu01::lzw_rep04                    W=1761659 U=0.000076 z=-3.7887
  [INFO] testu01::lzw_rep05                    W=1761751 U=0.000052 z=-3.8801
  [INFO] testu01::lzw_rep06                    W=1761622 U=0.000024 z=-4.0687
  [INFO] testu01::lzw_rep07                    W=1761706 U=0.000054 z=-3.8727
  [INFO] testu01::lzw_rep08                    W=1761722 U=0.000037 z=-3.9607
  [INFO] testu01::lzw_rep09                    W=1761708 U=0.000097 z=-3.7268
  [INFO] testu01::lzw_rep10                    W=1761669 U=0.000078 z=-3.7821

AES-128-CTR
  [PASS] testu01::lzw_sum                                  p = 0.060670  (N=10, k=25, r=0, s=30, pit_seed=1, Z=1.8759)
  [PASS] testu01::lzw_ks                                   p = 0.244179  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1762990 U=0.764932 z=0.7223
  [INFO] testu01::lzw_rep02                    W=1762977 U=0.633934 z=0.3423
  [INFO] testu01::lzw_rep03                    W=1762968 U=0.534110 z=0.0856
  [INFO] testu01::lzw_rep04                    W=1763003 U=0.869148 z=1.1224
  [INFO] testu01::lzw_rep05                    W=1762950 U=0.321180 z=-0.4644
  [INFO] testu01::lzw_rep06                    W=1762949 U=0.307879 z=-0.5019
  [INFO] testu01::lzw_rep07                    W=1763021 U=0.949640 z=1.6414
  [INFO] testu01::lzw_rep08                    W=1763026 U=0.962255 z=1.7775
  [INFO] testu01::lzw_rep09                    W=1762964 U=0.489013 z=-0.0275
  [INFO] testu01::lzw_rep10                    W=1763007 U=0.891499 z=1.2345

cryptography::CtrDrbgAes256
  [PASS] testu01::lzw_sum                                  p = 0.573631  (N=10, k=25, r=0, s=30, pit_seed=1, Z=0.5627)
  [PASS] testu01::lzw_ks                                   p = 0.814483  (N=10, k=25, r=0, s=30, pit_seed=1)
  [INFO] testu01::lzw_rep01                    W=1762988 U=0.745911 z=0.6617
  [INFO] testu01::lzw_rep02                    W=1763007 U=0.888707 z=1.2197
  [INFO] testu01::lzw_rep03                    W=1763003 U=0.868272 z=1.1183
  [INFO] testu01::lzw_rep04                    W=1762932 U=0.153373 z=-1.0221
  [INFO] testu01::lzw_rep05                    W=1762953 U=0.354369 z=-0.3736
  [INFO] testu01::lzw_rep06                    W=1762982 U=0.686947 z=0.4872
  [INFO] testu01::lzw_rep07                    W=1762965 U=0.495752 z=-0.0106
  [INFO] testu01::lzw_rep08                    W=1762954 U=0.364049 z=-0.3477
  [INFO] testu01::lzw_rep09                    W=1763001 U=0.857236 z=1.0680
  [INFO] testu01::lzw_rep10                    W=1762932 U=0.153524 z=-1.0214


========================================================================
webster_tavares  (SAC / BIC avalanche  samples=4096  bits=32)
========================================================================

RNG                                       samples  SACmean   SACmax  BICmean   BICmax BICdegen
-------------------------------------------------------------------------------------------------
MT19937                                      4096   0.0065   0.0281   0.0124   0.0611        0
Xorshift32                                   4096   0.5000   0.5000      NaN      NaN    15872
Xorshift64                                   4096   0.5000   0.5000      NaN      NaN    15872
BAD Unix System V rand()                     4096   0.3334   0.5000   0.0752   0.7659     5877
BAD Unix System V mrand48()                  4096   0.3782   0.5000   0.1124   0.9011    10942
BAD Unix BSD random()                        4096   0.0428   0.4878   0.0206   0.7164        0
BAD Unix Linux glibc rand()/random()         4096   0.0428   0.4878   0.0206   0.7164        0
BAD Windows CRT rand()                       4096   0.3310   0.5000   0.0771   0.8559     5877
warning: BAD Windows VB6/VBA Rnd(): --input-bits 32 exceeds RNG seed width (24 bits); bits 24..32 are silently truncated — results are misleading
BAD Windows VB6/VBA Rnd()                    4096   0.4173   0.5000   0.1049   0.8228    11986
BAD Windows .NET Random(seed)                4096   0.2346   0.4944   0.0697   0.7415        0
warning: ANSI C sample LCG: --input-bits 32 exceeds RNG seed width (31 bits); bits 31..32 are silently truncated — results are misleading
ANSI C sample LCG                            4096   0.3617   0.5000   0.1012   0.7015    11489
LCG MINSTD                                   4096   0.2637   0.5000   0.0807   0.7608      992
AES-128-CTR                                  4096   0.0061   0.0232   0.0125   0.0663        0
cryptography::CtrDrbgAes256                  4096   0.0063   0.0308   0.0125   0.0610        0

========================================================================
gorilla  (Marsaglia-Tsang Gorilla  all 32 bit positions)
========================================================================

RNG                                          min_p     max_p worst_bit  worst_|z|   agg_ad_A   agg_ad_p
----------------------------------------------------------------------------------------------------------
MT19937                                   0.025510  0.963248        28      1.951     0.6824   0.572419
Xorshift32                                1.000000  1.000000        24     47.913  2178.4817   0.000000
Xorshift64                                0.042486  0.959682        19      1.747     0.4842   0.761690
BAD Unix System V rand()                  0.103973  0.999980        18      4.111    27.7585   0.000000
BAD Unix System V mrand48()               0.000000  1.000000        31  10141.521   163.7581   0.000000
BAD Unix BSD random()                     0.114651  0.975209        13      1.964     1.0889   0.313288
BAD Unix Linux glibc rand()/random()      0.114651  0.975209        13      1.964     1.0889   0.313288
BAD Windows CRT rand()                    0.000000  0.999969         3     12.004    30.5478   0.000000
BAD Windows VB6/VBA Rnd()                 0.000000  0.000000         7  10162.438  2178.4817   0.000000
BAD Windows .NET Random(seed)             0.026731  0.969952        28      1.931     0.2187   0.984438
ANSI C sample LCG                         0.000000  1.000000         0  10172.876  1427.2633   0.000000
LCG MINSTD                                0.000000  1.000000         0  10172.876   102.5270   0.000000
AES-128-CTR                               0.050697  0.987885         9      2.253     0.4736   0.772588
cryptography::CtrDrbgAes256               0.000534  0.993353        31      3.272     0.9383   0.390523
```
