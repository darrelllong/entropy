# Peer Review — Seventh Pass
*2026-03-15. Sixth Pass plus new `rng::spongebob` module added post-review.*
*All five prior fixes confirmed in place.*

---

## Fixes Confirmed In This Pass

| Fix | Introduced | Status |
|-----|-----------|--------|
| `nist::linear_complexity` μ: `(9+r)/36` → `(9+(−1)^M)/36` | Pass 4 | ✓ |
| `dieharder::lagged_sums` one-sided p → two-sided `erfc(\|z\|/√2)` | Pass 4 | ✓ |
| `diehard::dna` boffset step 1 → step 2, groups div_ceil(16) | Pass 4 | ✓ |
| `nist::runs` extra `* SQRT_2` removed | Pass 5 | ✓ |
| `nist::random_excursions_variant` extra `* SQRT_2` removed | Pass 5 | ✓ |

---

## Active Issues

### FLAW 1 — `diehard::craps` single-result wrapper inflates false-positive rate

**File:** `src/diehard/craps.rs:59`

```rust
let p_value = p_wins.min(p_throws);
```

`craps()` reports `min(p_wins, p_throws)`. Under H₀ each p-value is U(0,1), so the minimum of two independents has CDF F(p) = 1−(1−p)². At α=0.01, the actual false-positive rate is 1−(0.99)² ≈ 2%. Double the intended rate.

`craps_both()` is the correct form — it reports the two p-values separately, letting the caller apply a Bonferroni or per-test threshold. The fix is to document `craps()` as deprecated or remove it.

**Not a numerical error; a design error in the test-collapsing wrapper.**

---

### FLAW 2 — `nist::matrix_rank` format string says F<30 instead of F≤30

**File:** `src/nist/matrix_rank.rs:55`

```rust
format!("N={num_matrices}, F32={f_32}, F31={f_31}, F<30={f_less}, χ²={chi_sq:.4}")
```

`f_less` accumulates matrices with rank ≤ 30 (rank ∈ {0,1,…,30}), so the label should be `F≤30`. The constant `P_LESS = 0.1336` is labelled in the comment "P(rank ≤ 30)". The display string is inconsistent.

**Display-only bug; does not affect p-values.**

---

### CONCERN — `rng::blum_blum_shub` outputs 32 bits per step

**File:** `src/rng/blum_blum_shub.rs:67–70`

```rust
fn next_u32(&mut self) -> u32 {
    self.state = mul_mod(self.state, self.state, self.n);
    self.state as u32
}
```

The module docstring correctly notes: "extracting up to ⌊log₂ n⌋ bits per step is provably secure for BBS." This is aspirational, not the tight bound. The actual provably-secure per-step bit count is O(log₂ log₂ n). With n a 64-bit modulus (log₂ n ≈ 64), the tight bound is ≈ 6 bits per step, not 32.

Returning the low 32 bits of every state is not within the formal security proof. For a statistical test battery this is fine — BBS is here as a historically important CSPRNG target, not as a security primitive. But the docstring implies the security proof covers 32 bits, which it does not.

**Not a test fidelity issue; a documentation overclaim.**

---

### CONCERN — `rng::spongebob` `take_bytes` silently discards output bytes at refill boundaries

**File:** `src/rng/spongebob.rs:73–84`

```rust
fn take_bytes<const N: usize>(&mut self) -> [u8; N] {
    if self.offset == STATE_BYTES {
        self.refill();
    }
    assert!(N <= STATE_BYTES, "chunk larger than SpongeBob state");
    if self.offset + N > STATE_BYTES {
        self.refill();  // ← discards STATE_BYTES - self.offset trailing bytes
    }
    let out = self.state[self.offset..self.offset + N].try_into().unwrap();
    self.offset += N;
    out
}
```

When `offset + N > STATE_BYTES` (but `offset != STATE_BYTES`) the function refills the state and **silently drops `STATE_BYTES − offset` trailing bytes** from the current block. Up to `N−1 = 7` bytes of SHA3-512 output are thrown away per crossing.

**The module docstring says:** *"Each state contributes 512 output bits."* This is false whenever a crossing occurs.

**When does it trigger?** Only on mixed-width access where the boundary is not aligned. For pure `next_u32()` streams: 64 % 4 = 0 — never triggers. For pure `next_u64()` streams: 64 % 8 = 0 — never triggers. For mixed: e.g., 15 × `next_u32()` (offset=60) then 1 × `next_u64()` (60+8=68 > 64) discards 4 bytes before refilling.

**Is this triggered in the test harness?** No. The `Rng` trait blanket methods (`collect_u32s`, `collect_bits`, `collect_f64s`) all call `next_u32()` exclusively. `next_u64()` is called directly only by a handful of tests. No harness code mixes widths on the same instance mid-stream. So the discarding never occurs in practice.

**But the tests do not cover the mixed-width case at all.** The four unit tests are purely single-width (`next_u64` × 8, or `next_u32` × 16), so the latent behavior is invisible.

**Two sub-issues:**

1. **Module documentation overclaims** — "512 output bits" per state should be "up to 512 output bits; mixed-width calls at refill boundaries may discard up to 7 trailing bytes."

2. **`assert!(N <= STATE_BYTES)` is a runtime assertion on a const generic** — `N` is known at compile time. This should be `const { assert!(N <= STATE_BYTES, "...") }` to catch oversized chunks at compile time rather than at runtime.

**Severity:** Documentation inaccuracy + latent behavioral issue in never-exercised code path. Zero impact on any current test results. The SHA3-512 core is cryptographically sound; the output quality of what *is* emitted is impeccable.

---

### PRECISION CONCERN — `math::igamc` 200-iteration limit for large `a`

**File:** `src/math.rs:95–133`

Both `gamser` (series) and `gammcf` (Lentz CF) iterate for at most 200 steps. The Lentz CF converges in O(√a) steps when x ≈ a (the modal region). For `a ≥ 40000` the 200-step limit may be reached before the 1×10⁻¹³ convergence criterion, producing slightly inaccurate p-values.

**Where it bites:** `nist::approximate_entropy` calls `igamc(2^{m-1}, chi_sq/2)`. The guard `(1 << m) > n/10` limits m, but for m=17 (reached at n≈1.3M), `a = 2^16 = 65536` and `√a ≈ 256` — exceeding the iteration cap.

For the intended operating ranges (n ≤ 1M ⟹ m ≤ 16 ⟹ a ≤ 32768) the 200-step limit is just barely sufficient. For any caller pushing past m=16 the p-value from `approximate_entropy` may silently under-converge. The CF does not detect this — it just returns the best 200-step approximation without warning.

**In practice:** weak RNGs produce chi_sq >> df, so x >> a in igamc, and the CF converges in a handful of steps regardless of a. The precision gap only appears for well-behaved RNGs in the modal region. For a test battery this is unlikely to produce false failures, but it could produce slightly wrong p-values.

---

## Full Function Status Table

### `src/math.rs` — Core Numerics

| Function | Verdict | Notes |
|----------|---------|-------|
| `erfc` | **CORRECT** | Numerical Recipes rational approx, max error 1.2×10⁻⁷ ✓ |
| `normal_cdf` | **CORRECT** | Φ(x) = 0.5·erfc(−x/√2) ✓ |
| `lgamma` | **CORRECT** | Lanczos, accurate to ~15 sig figs ✓ |
| `igamc` | **CORRECT†** | Lentz CF + series; 200-iteration cap can under-converge for a ≥ ~40000 (see Concern above) |
| `ks_test` | **CORRECT** | Correct KS D statistic (max of D⁺ and D⁻) ✓ |
| `ks_pvalue` | **CORRECT** | Marsaglia-Tsang-Wang (2003) exact/speedup hybrid; Stephens asymptotic for n > 4999 ✓ |
| `chi2_pvalue` | **CORRECT** | igamc(df/2, χ²/2) ✓ |
| `fft_magnitudes` | **CORRECT** | rustfft ✓ |
| `dft_magnitudes` | **CORRECT** | Direct O(n²) reference implementation ✓ |

### `src/nist/` — NIST SP 800-22

| Function | File | Verdict | Notes |
|----------|------|---------|-------|
| `frequency` | frequency.rs | **CORRECT** | s_obs = \|S_n\|/√n; erfc(s_obs/√2) matches STS ✓ |
| `block_frequency` | block_frequency.rs | **CORRECT** | χ²=4M·Σ(π_j−0.5)², igamc(N/2,χ²/2) ✓ |
| `runs` | runs.rs | **CORRECT** | erfc(numer/denom) without spurious √2 ✓ |
| `longest_run` | longest_run.rs | **CORRECT** | Table 1 M/K/v_min/π values ✓; df=K ✓ |
| `matrix_rank` | matrix_rank.rs | **CORRECT†** | P32/P31/P≤30 constants ✓; igamc(1,χ²/2) ✓; format "F<30" should be "F≤30" |
| `spectral` | spectral.rs | **CORRECT** | T=√(n·ln20) ✓; n/2 magnitudes ✓; erfc(d/√2) ✓ |
| `non_overlapping_template` | non_overlapping_template.rs | **CORRECT** | N=8, μ=(M−m+1)/2^m ✓, σ² formula ✓, 148 aperiodic templates ✓ |
| `overlapping_template` | overlapping_template.rs | **CORRECT** | m=9 restriction ✓; M=1032 ✓; π table (Table 4) ✓; K=5 ✓ |
| `universal` | universal.rs | **CORRECT** | Maurer Table 3 ✓; c(L,K) correction ✓; erfc(z/√2) ✓ |
| `linear_complexity` | linear_complexity.rs | **CORRECT** | μ uses (9+(−1)^M)/36 ✓; 7 bins ✓; igamc(3.0,χ²/2) ✓ |
| `serial` | serial.rs | **CORRECT** | ψ² formula ✓; ∇²ψ²/∇ψ² ✓; both p-values ✓ |
| `approximate_entropy` | approximate_entropy.rs | **CORRECT†** | φ(m)−φ(m+1) ✓; χ²=2n(ln2−ApEn) ✓; igamc precision concern for m≥17 |
| `cumulative_sums` | cumulative_sums.rs | **CORRECT** | SP 800-22 §2.13.4 two-sum formula ✓; forward/backward ✓ |
| `random_excursions` | random_excursions.rs | **CORRECT** | π_k formula ✓ (including k≥5 tail); 8 states ✓; igamc(2.5,χ²/2) ✓ |
| `random_excursions_variant` | random_excursions_variant.rs | **CORRECT** | erfc(\|ξ−J\|/√(2J(4\|x\|−2))) without spurious √2 ✓ |

### `src/diehard/` — Marsaglia DIEHARD

| Function | File | Verdict | Notes |
|----------|------|---------|-------|
| `binary_rank_32x32` | binary_rank.rs | **CORRECT** | Reference constants (0.2888/0.5776/0.1284/0.0053) ✓; 40k matrices ✓ |
| `binary_rank_31x31` | binary_rank.rs | **CORRECT** | Exact gf2_rank_probability() ✓; sums to 1 ✓ |
| `binary_rank_6x8` | binary_rank.rs | **CORRECT** | Exact probabilities (0.7731/0.2174/0.0094) ✓ |
| `birthday_spacings` | birthday_spacings.rs | **CORRECT** | M=512, YEAR=2^24, λ=2 ✓; interval counting ✓; 9 offsets → KS ✓ |
| `bitstream` | bitstream.rs | **CORRECT** | 20-bit words, 2^21 windows, mean=141909, σ=428 ✓; MSB-first ✓ |
| `count_ones_stream` | count_ones.rs | **CORRECT** | Q5−Q4 diff ✓; mean=2500, σ=√5000 ✓; letter probs [37,56,70,56,37]/256 ✓ |
| `craps` | craps.rs | **FLAW** | p_wins/p_throws stats correct; `craps()` takes min() → 2× false-positive rate. Use `craps_both()` |
| `minimum_distance_2d` | minimum_distance.rs | **WARNED** | Acknowledged-buggy formula explicitly documented; use minimum_distance_nd(d=2) ✓ |
| `opso` | monkey.rs | **CORRECT** | Paired 10-bit fields ✓; constants ✓ |
| `oqso` | monkey.rs | **CORRECT** | 4-word groups, 5-bit fields, 6 positions/group ✓ |
| `dna` | monkey.rs | **CORRECT** | boffset step=2, 16 samples/group, 2-bit fields ✓ |
| `parking_lot` | parking_lot.rs | **CORRECT** | L∞ unit square cars ✓; mean=3523, σ=21.9 ✓; z→CDF→KS ✓ |
| `runs_float` | runs_float.rs | **CORRECT** | Grafton 1981 A/B matrices ✓; cyclic closeout (`next > first`) verified vs diehard_runs.c ✓ |
| `spheres_3d` | spheres_3d.rs | **CORRECT** | min r³ ~ Exp(30) ✓; 1−exp(−r³/30) ✓; KS ✓ |
| `squeeze` | squeeze.rs | **CORRECT** | SDATA from reference ✓; bin j≤6→idx 0, j=7..47→idx 1..41, j≥48→idx 42 ✓ |

### `src/dieharder/` — Brown Dieharder

| Function | File | Verdict | Notes |
|----------|------|---------|-------|
| `bit_distribution` | bit_distribution.rs | **CORRECT** | MSB-first ✓; binomial PMF ✓; Vtest bundling cutoff=20 ✓ |
| `byte_distribution` | byte_distribution.rs | **CORRECT** | 9 streams from 3-word groups; bit positions {0–7,12–19,24–31}; df=255×9=2295 ✓ |
| `dct` | dct.rs | **CORRECT** | DCT-II formula ✓; DC mean=N(2^31−0.5) adjustment ✓; rotation schedule ✓ |
| `fill_tree` | fill_tree.rs | **CORRECT** | BST insertion ✓; TARGET_DATA ✓; position_counts[fail_pos/2] ✓ |
| `gcd` | gcd.rs | **CORRECT** | P(gcd=k)=6/(π²k²) ✓; KPROB from C source ✓; Euclidean step count ✓ |
| `ks_uniform` | ks_uniform.rs | **CORRECT** | Direct KS on float output ✓ |
| `lagged_sums` | lagged_sums.rs | **CORRECT** | Two-sided erfc(\|z\|/√2) ✓; stride=lag+1 ✓ |
| `minimum_distance_nd` | minimum_distance_nd.rs | **CORRECT** | Fischler formula ✓; Q_CORRECTION table ✓; ball_volume even/odd ✓ |
| `monobit2` | monobit2.rs | **CORRECT** | Faithful dab_monobit2.c port ✓; auto_ntuple ✓; chisq_binomial ✓; eval_most_extreme ✓ |
| `permutations` | permutations.rs | **CORRECT** | Lehmer code ✓; factorial ✓; χ² with df=t!−1 ✓ |

### `src/research/` — Research Tests

| Function | File | Verdict | Notes |
|----------|------|---------|-------|
| `gorilla_all` | marsaglia_tsang.rs | **CORRECT** | 26-bit words ✓; mean=24687971, σ=4170 ✓; upper-tail p intentional (excess missing words) ✓ |
| `permutation_test` | knuth.rs | **CORRECT** | Lehmer code ✓; count-less-than verified on all 6 permutations of {0.1,0.2,0.3} ✓ |
| `gap_test` | knuth.rs | **CORRECT** | Geometric P(gap=r)=p(1−p)^r ✓; df=max_gap ✓; first-hit prefix excluded ✓ |
| `runs_above_below_median_test` | knuth.rs | **CORRECT** | Wald-Wolfowitz μ=1+2n1n2/(n1+n2), σ² formula ✓; erfc(\|z\|/√2) ✓ |
| `approximate_entropy_profile` | pincus.rs | **CORRECT** | Same circular φ computation as nist::approximate_entropy ✓ |
| `fpf_test` | practrand_fpf.rs | **APPROX** | G-test ✓; uses asymptotic χ² not PractRand's empirical tables — documented |
| `hamming_corr` | testu01_hamming.rs | **CORRECT** | ρ̂=4·Σ/((n−1)L) ✓; z=ρ̂·√(n−1) ✓; erfc(\|z\|/√2) ✓ |
| `hamming_indep` | testu01_hamming.rs | **CORRECT** | Main χ² with gofs_MinExpected=10 lumping ✓; block dof (if l%2==1 && k==1: dof=1 else 2) ✓ |
| `lempel_ziv` | testu01_lz.rs | **APPROX** | LZ78 trie ✓; μ/σ tables empirical from TestU01 — documented |
| `evaluate_u64` (SAC/BIC) | webster_tavares.rs | **CORRECT** | Dependence matrix ✓; BIC correlation accumulation ✓ |

### `src/rng/` — Reference RNGs

| Module | Verdict | Notes |
|--------|---------|-------|
| `SystemVRand` | **CORRECT** | `(state*1103515245+12345)>>16 & 0x7FFF` ✓; 15-bit output verified ✓ |
| `WindowsMsvcRand` | **CORRECT** | `(state*214013+2531011)>>16 & 0x7FFF` ✓; known-vector verified ✓ |
| `WindowsVb6Rnd` | **CORRECT** | `(state*0x43FD43FD+0x00C39EC3) & 0xFFFFFF` ✓; 24-bit; known-vector verified ✓ |
| `WindowsDotNetRandom` | **CORRECT** | Subtractive generator; 55-element table; known-vector verified ✓ |
| `BsdRandom` | **CORRECT** | TYPE_3 deg=31 sep=3; 310 warm-up steps; known-vector verified ✓ |
| `BsdRandCompat` | **CORRECT** | Park-Miller; known-vector verified ✓ |
| `Rand48` | **CORRECT** | a=0x5DEECE66D, c=0xB, m=2^48 ✓ |
| `Lcg32` variants | **CORRECT** | AnsiC/Minstd/Borland/Msvc; MSVC known-vector verified ✓ |
| `Mt19937` | **CORRECT** | Constants ✓; tempering ✓; pre-twist in constructor produces same output as deferred-twist reference ✓ |
| `Xorshift32` | **CORRECT** | (13,17,5) triple from Marsaglia (2003) Table 1 ✓ |
| `Xorshift64` | **CORRECT** | (13,7,17) triple ✓ |
| `BlumBlumShub` | **CORRECT†** | x_{i+1}=x_i² mod n ✓; 32-bit output exceeds provably-secure ~6 bits/step for 64-bit n (see Concern above) |
| `BlumMicali` | **CORRECT** | x_{i+1}=g^{x_i} mod p ✓; bit=1 iff x≤(p−1)/2 ✓; Mersenne-prime fast path ✓ |
| `SpongeBob` | **CORRECT†** | SHA3-512 hash chain ✓; byte stream consistent for pure-width access; mixed-width calls silently discard up to 7 bytes at refill boundary; `assert!` on const generic should be `const { assert! }` (see Concern above) |
| `AesCtrRng` | **CORRECT** | NIST SP 800-38A CTR-F.5 vector ✓ |
| `DualEcDrbg` | **CORRECT** | t=x(s·P), s←x(t·P), r=x(t·Q), rightmost outlen bits ✓; SP 800-90 Appendix A.1 Q points ✓ |

---

## Priority Issue List

### Priority 1 — Statistical correctness errors (fix)

**`src/diehard/craps.rs` — `craps()` single-result min:**
At α=0.01, actual FPR ≈ 2% instead of 1%. `craps_both()` is the correct API. Either deprecate `craps()` with a doc note, or rewrite it to report the combined p-value using Fisher's method or a proper Bonferroni correction.

### Priority 2 — Documentation / display errors (fix)

**`src/nist/matrix_rank.rs:55`:**
Change `"F<30"` to `"F≤30"` in the format string. The cell accumulates all rank < 31, i.e., rank ∈ {0,…,30}, so ≤30 is correct.

### Priority 3 — Documentation overclaim (clarify)

**`src/rng/blum_blum_shub.rs` module docstring:**
The sentence "extracting up to ⌊log₂ n⌋ bits per step is provably secure" is incorrect. The tight proven bound is O(log₂ log₂ n) ≈ 6 bits for a 64-bit modulus. The full 32-bit per-step output is not within the security proof. This is fine for a test battery, but the docstring overstates the theory.

### Priority 4 — Silent byte discard + documentation overclaim (clarify)

**`src/rng/spongebob.rs` — `take_bytes` mixed-width boundary:**
The module docstring "Each state contributes 512 output bits" is wrong when `next_u32()` and `next_u64()` are mixed across a refill boundary. Up to 7 trailing bytes of SHA3 output are silently discarded. In practice the harness never mixes widths on a live instance, so this is latent. Fix the docstring. Separately, `assert!(N <= STATE_BYTES)` on a `const N` should be a compile-time `const { assert!(...) }`.

### Priority 5 — Precision boundary (note)

**`src/math.rs` — `igamc`/`gamser` 200-iteration cap:**
For `a ≥ 40000` with `x ≈ a` (the modal region), convergence requires O(√a) iterations. 200 steps may be insufficient. Affects `nist::approximate_entropy` for m ≥ 17. In practice only reached for n ≥ 1.3M with recommended parameters; weak-RNG chi_sq >> df values converge fast regardless. Add a `#[cfg(debug_assertions)]` warning or increase the cap to 500 to cover a up to ~62500 (m=16).

### Priority 6 — Known-buggy legacy test (intentional)

**`src/diehard/minimum_distance_2d`:** Formula explicitly documented as wrong and obsolete. Not a flaw, not fixable — it is preserved for historical comparison only. Status quo is correct.

---

## Confirmed Clean

| Suspicion | Resolution |
|-----------|------------|
| `nist::runs` SQRT_2 | Fixed: erfc(numer/denom) ✓ |
| `nist::random_excursions_variant` SQRT_2 | Fixed: erfc(numer/denom) ✓ |
| `nist::linear_complexity` μ formula | Fixed: (9+(−1)^M)/36 ✓ |
| `dieharder::lagged_sums` one-sided p | Fixed: two-sided erfc(|z|/√2) ✓ |
| `diehard::dna` boffset misalignment | Fixed: step=2, 16 samples/group ✓ |
| `nist::frequency` erfc convention | erfc(s_obs/√2) matches STS ✓ |
| `nist::spectral` erfc convention | erfc(d/√2) matches STS ✓ |
| `nist::universal` erfc convention | erfc(z/√2) matches STS ✓ |
| `nist::random_excursions` π_k formula | (1-1/2\|x\|)^4·(1/2\|x\|) = P(J≥5) correct ✓ |
| `nist::serial` psi_sq for l=0 | Returns 0, which is mathematically correct (ψ²₀=0) ✓ |
| `nist::approximate_entropy` chi_sq df | igamc(2^{m-1},…) → df=2^m correct per SP 800-22 §2.12 ✓ |
| `nist::cumulative_sums` series bounds | Matches SP 800-22 §2.13.4 exactly ✓ |
| `diehard::birthday_spacings` λ | 512³/(4·2^24)=2 ✓ |
| `diehard::count_ones_stream` B sums | B sums to 0.5 ≈ expected runs/n (not a probability, a proportion) ✓ |
| `diehard::runs_float` cyclic closeout | `next > first` verified correct vs diehard_runs.c ✓ |
| `diehard::parking_lot` L∞ condition | `(|dx|≥1 or |dy|≥1)` is correct no-overlap for L∞ unit square ✓ |
| `dieharder::byte_distribution` bit positions | {0-7, 12-19, 24-31} per word; verified vs dab_bytedistrib.c ✓ |
| `dieharder::fill_tree` position_counts indexing | [fail_pos/2] verified vs dab_filltree.c ✓ |
| `dieharder::monobit2` multi-scale layout | Verified vs dab_monobit2.c ✓ |
| `research::hamming_indep` dof special case | dof=1 when l%2==1 && k==1 verified vs sstring.c ✓ |
| `rng::mt19937` pre-twist in constructor | Equivalent to standard deferred-twist; same output sequence ✓ |
| `rng::blum_blum_shub` small-example test | Wikipedia BBS sequence (n=253, seed=3) verified ✓ |
| `rng::blum_micali` small reference | p=23, g=5, seed=3: first 3 bits=1,1,1 verified ✓ |
| `rng::dual_ec` output truncation | rightmost(outlen,r) = last outlen/8 bytes matches SP 800-90 ✓ |
| `research::gorilla` upper-tail p | 1−Φ(z) correct: excess missing words = too-uniform = upper tail ✓ |

---

## Summary

**5 prior fixes confirmed in place.**
**0 numerical bugs found in SpongeBob.**
**1 statistical design flaw** (`craps()` min-p wrapper) — priority-1.
**1 display bug** (matrix_rank format "F<30") — trivial.
**1 latent behavioral issue + doc overclaim** (`spongebob` mixed-width byte discard) — priority-4.
**2 documentation concerns** (BBS security claim, igamc iteration cap).

All 55 unit tests pass. SpongeBob cryptographic core (SHA3-512 hash chain) is sound.
