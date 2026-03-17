# Bibliography

References used or surveyed for this project. Entries marked **[pubs/]** have a local copy in `pubs/`.
Entries marked **[TODO: library]** still need to be fetched from a library or publisher site.

The point of keeping these files in-tree is auditability: readers should be able to check the implementation claims against the exact standards, manuals, source drops, and papers used here.

---

## Implemented batteries

```bibtex
@techreport{nist800-22,
  author      = {Rukhin, Andrew and Soto, Juan and Nechvatal, James and Smid, Miles and Barker, Elaine and Leigh, Stefan and Levenson, Mark and Vangel, Mark and Banks, David and Heckert, Alan and Dray, James and Vo, San},
  title       = {A Statistical Test Suite for Random and Pseudorandom Number Generators for Cryptographic Applications},
  institution = {National Institute of Standards and Technology},
  number      = {SP 800-22 Rev. 1a},
  year        = {2010},
  note        = {[pubs/NIST-SP-800-22r1a.pdf]}
}

@misc{marsaglia1995diehard,
  author = {Marsaglia, George},
  title  = {{DIEHARD}: A Battery of Tests of Randomness},
  year   = {1995},
  note   = {Florida State University. [pubs/diehard-doc.txt, pubs/diehard-tests.txt, pubs/Diehard.zip]}
}

@misc{brown2004dieharder,
  author = {Brown, Robert G.},
  title  = {Dieharder: A Random Number Test Suite},
  year   = {2004},
  note   = {Version 3.31.x. [pubs/dieharder-manual.pdf, pubs/dieharder-3.31.1.tgz]}
}

@article{marsaglia2002difficult,
  author  = {Marsaglia, George and Tsang, Wai Wan},
  title   = {Some Difficult-to-pass Tests of Randomness},
  journal = {Journal of Statistical Software},
  volume  = {7},
  number  = {3},
  year    = {2002},
  doi     = {10.18637/jss.v007.i03},
  note    = {[pubs/marsaglia-tsang-2002-difficult-tests.pdf]}
}
```

---

## NIST SP 800-90 series (RNG standards)

```bibtex
@techreport{nist800-90a,
  author      = {{National Institute of Standards and Technology}},
  title       = {Recommendation for Random Number Generation Using Deterministic Random Bit Generators},
  institution = {NIST},
  number      = {SP 800-90A Rev. 1},
  year        = {2015},
  note        = {[pubs/NIST-SP-800-90Ar1.pdf]}
}

@techreport{nist800-90b,
  author      = {{National Institute of Standards and Technology}},
  title       = {Recommendation for the Entropy Sources Used for Random Bit Generation},
  institution = {NIST},
  number      = {SP 800-90B},
  year        = {2018},
  note        = {[pubs/NIST-SP-800-90B.pdf]}
}

@techreport{nist800-90c,
  author      = {{National Institute of Standards and Technology}},
  title       = {Recommendation for Random Bit Generator (RBG) Constructions},
  institution = {NIST},
  number      = {SP 800-90C (Draft)},
  year        = {2022},
  note        = {[pubs/NIST-SP-800-90C.pdf]}
}

@techreport{nist-fips-140-3,
  author      = {{National Institute of Standards and Technology}},
  title       = {Security Requirements for Cryptographic Modules},
  institution = {NIST},
  number      = {FIPS 140-3},
  year        = {2019},
  note        = {[pubs/NIST-FIPS-140-3.pdf]}
}
```

---

## Generator algorithms

Papers that define the RNG algorithms implemented in `src/rng/`.

```bibtex
@article{matsumoto1998mersenne,
  author  = {Matsumoto, Makoto and Nishimura, Takuji},
  title   = {Mersenne Twister: A 623-Dimensionally Equidistributed Uniform
             Pseudo-Random Number Generator},
  journal = {ACM Transactions on Modeling and Computer Simulation},
  volume  = {8},
  number  = {1},
  pages   = {3--30},
  year    = {1998},
  doi     = {10.1145/272991.272995},
  note    = {[TODO: library] Period 2^{19937}-1; state recovery from 624 consecutive
             outputs is documented in §3.  Default generator in NumPy, MATLAB, R.}
}

@article{blackman2021xoshiro,
  author  = {Blackman, David and Vigna, Sebastiano},
  title   = {Scrambled Linear Pseudorandom Number Generators},
  journal = {ACM Transactions on Mathematical Software},
  volume  = {47},
  number  = {4},
  pages   = {36:1--36:32},
  year    = {2021},
  doi     = {10.1145/3460772},
  note    = {[pubs/blackman-vigna-2021-scrambled-linear.pdf] Xoshiro256** and
             Xoroshiro128** scrambler definitions.}
}

@techreport{oneill2014pcg,
  author      = {O'Neill, Melissa E.},
  title       = {{PCG}: A Family of Simple Fast Space-Efficient Statistically Good
                 Algorithms for Random Number Generation},
  institution = {Harvey Mudd College},
  number      = {HMC-CS-2014-0905},
  year        = {2014},
  note        = {[pubs/oneill-2014-pcg.pdf] PCG32 (XSH-RR) and PCG64 (XSL-RR).}
}

@article{marsaglia2003xorshift,
  author  = {Marsaglia, George},
  title   = {Xorshift {RNG}s},
  journal = {Journal of Statistical Software},
  volume  = {8},
  number  = {14},
  year    = {2003},
  doi     = {10.18637/jss.v008.i14},
  note    = {32-bit and 64-bit Xorshift generators; listing 1 defines xorshift32.}
}

@misc{wangyi2022wyhash,
  author = {Wang, Yi},
  title  = {wyhash and wyrand, version 4.2},
  year   = {2022},
  url    = {https://github.com/wangyi-fudan/wyhash},
  note   = {[pubs/wang-2022-wyhash.pdf] Weyl-sequence counter with 128-bit
             multiply-xorfolded finaliser; passes BigCrush and PractRand > 8 TiB.}
}

@misc{jenkins2007smallprng,
  author = {Jenkins, Bob},
  title  = {A Small Noncryptographic {PRNG}},
  year   = {2007},
  url    = {http://burtleburtle.net/bob/rand/smallprng.html},
  note   = {[pubs/jenkins-2007-smallprng.html] JSF64 (Jenkins Small Fast),
             four-word 64-bit chaotic generator.}
}

@inproceedings{bernstein2008chacha,
  author    = {Bernstein, Daniel J.},
  title     = {{ChaCha}, a Variant of {Salsa20}},
  booktitle = {Workshop Record of SASC 2008: The State of the Art of Stream Ciphers},
  year      = {2008},
  note      = {[pubs/bernstein-2008-chacha.pdf] ChaCha20 stream cipher; 20-round
               variant used in Linux /dev/urandom, macOS arc4random, and TLS 1.3.}
}

@article{park1988minstd,
  author  = {Park, Stephen K. and Miller, Keith W.},
  title   = {Random Number Generators: Good Ones Are Hard to Find},
  journal = {Communications of the ACM},
  volume  = {31},
  number  = {10},
  pages   = {1192--1201},
  year    = {1988},
  doi     = {10.1145/63039.63042},
  note    = {MINSTD: a=16807, c=0, m=2^{31}-1 (Lehmer generator).  Also defines
             the Park-Miller test used by FreeBSD rand_r() compatibility path.}
}

@misc{unix-v7-manual,
  author = {Thompson, Ken and Ritchie, Dennis M.},
  title  = {Unix Programmer's Manual, 7th Edition},
  year   = {1979},
  note   = {Bell Laboratories. rand(3) entry defines the LCG parameters
             a=1103515245, c=12345 that became the de facto ANSI C / System V
             rand() implementation.  Available at
             https://www.tuhs.org/Archive/Distributions/Research/V7/}
}

@incollection{bernstein2015dualec,
  author    = {Bernstein, Daniel J. and Lange, Tanja and Niederhagen, Ruben},
  title     = {Dual {EC}: A Standardized Back Door},
  booktitle = {The New Codebreakers: Essays Dedicated to David Kahn on the
               Occasion of His 85th Birthday},
  series    = {Lecture Notes in Computer Science},
  volume    = {9100},
  publisher = {Springer},
  year      = {2016},
  pages     = {256--281},
  doi       = {10.1007/978-3-662-49301-4_17},
  note      = {Demonstrates that the NIST-specified Q points in SP 800-90 Appendix A.1
               are likely NSA-chosen with a discrete-log trapdoor.  State recovery
               from 30 bytes of output.}
}
```

---

## Cryptographic primitives

FIPS standards and mode-of-operation documents underlying the cipher-based generators.

```bibtex
@techreport{nist-fips-197,
  author      = {{National Institute of Standards and Technology}},
  title       = {Advanced Encryption Standard ({AES})},
  institution = {NIST},
  number      = {FIPS PUB 197, Update 1},
  year        = {2023},
  note        = {[pubs/NIST-FIPS-197.pdf] AES block cipher specification.
                 Underlying cipher for AesCtr, BlockCtrRng<Aes>, and CryptoCtrDrbg.}
}

@techreport{nist-sp800-38a,
  author      = {Dworkin, Morris},
  title       = {Recommendation for Block Cipher Modes of Operation},
  institution = {NIST},
  number      = {SP 800-38A},
  year        = {2001},
  note        = {[pubs/NIST-SP-800-38A.pdf] §6.5 defines CTR mode used by
                 AesCtr and BlockCtrRng.}
}

@techreport{nist-fips-180-4,
  author      = {{National Institute of Standards and Technology}},
  title       = {Secure Hash Standard ({SHS})},
  institution = {NIST},
  number      = {FIPS PUB 180-4},
  year        = {2015},
  note        = {[pubs/NIST-FIPS-180-4.pdf] SHA-256 specification.
                 Used by Squidward hash-chain generator and Hash_DRBG.}
}

@techreport{nist-fips-202,
  author      = {{National Institute of Standards and Technology}},
  title       = {{SHA-3} Standard: Permutation-Based Hash and Extendable-Output
                 Functions},
  institution = {NIST},
  number      = {FIPS PUB 202},
  year        = {2015},
  note        = {[pubs/NIST-FIPS-202.pdf] SHA3-512 specification.
                 Used by SpongeBob hash-chain generator.}
}
```

---

## Domain context

```bibtex
@phdthesis{hughes2021badrandom,
  author = {Hughes, James Prescott},
  title  = {{BADRANDOM}: The Effect and Mitigations for Low Entropy Random Numbers in {TLS}},
  school = {University of California, Santa Cruz},
  year   = {2021},
  note   = {[pubs/hughes-2022-badrandom-the-effect-and-mitigations-for-low-entropy-random-numbers-in-tls.pdf]}
}
```

---

## Suites to implement — high priority

```bibtex
@article{lecuyer2007testu01,
  author  = {L'Ecuyer, Pierre and Simard, Richard},
  title   = {{TestU01}: A {C} Library for Empirical Testing of Random Number Generators},
  journal = {ACM Transactions on Mathematical Software},
  volume  = {33},
  number  = {4},
  pages   = {22:1--22:40},
  year    = {2007},
  doi     = {10.1145/1268776.1268777},
  note    = {[pubs/lecuyer-simard-2007-testu01.pdf] Novel tests vs our batteries:
             BirthdaySpacings (Poisson form), LempelZiv (LZ78 phrase count),
             HammingCorr/HammingIndep, RandomWalk1, LinearComplexity profile (streaming
             Berlekamp-Massey), MaxOft (order statistics), CouponCollector (exact
             waiting-time dist.), ClosePairs (N-dim Poisson), PowerDivergence multinomial.
             Source: http://simul.iro.umontreal.ca/testu01/}
}

@misc{practrand,
  author = {Doty-Humphrey, Chris},
  title  = {{PractRand}: Practically Random --- A C++ Library of Statistical Tests for {RNG}s},
  year   = {2018},
  note   = {[TODO: fetch docs] Version 0.95. Novel tests: BCFN (DFT of Hamming-weight block
             counts), DC6 (lagged difference patterns for small-state generators), FPF
             (leading-bit frequency chi-square), TMFn (N-dim spectral), streaming linear
             complexity. Source: http://pracrand.sourceforge.net/}
}
```

---

## Individual tests to implement — peer-reviewed

```bibtex
@article{maurer1992universal,
  author  = {Maurer, Ueli M.},
  title   = {A Universal Statistical Test for Random Bit Generators},
  journal = {Journal of Cryptology},
  volume  = {5},
  number  = {2},
  pages   = {89--105},
  year    = {1992},
  doi     = {10.1007/BF00193563},
  note    = {[pubs/maurer-1992-universal-test.pdf] Full parametric form at L=10--16 is
             substantially more sensitive than the NIST-locked implementation at L=7.
             Requires the exact asymptotic variance formula from this paper for correct
             p-values at higher L. PDF from author page: https://crypto.ethz.ch/publications/}
}

@book{knuth1997taocp2,
  author    = {Knuth, Donald E.},
  title     = {The Art of Computer Programming, Volume 2: Seminumerical Algorithms},
  edition   = {3rd},
  publisher = {Addison-Wesley},
  year      = {1997},
  isbn      = {0-201-89684-2},
  note      = {§3.3.2: Poker test (hand-type multinomial over t-symbol groups), Permutation test
               (all t! orderings), Wald-Wolfowitz runs above/below median (distinct from NIST
               bit-level runs), Serial Correlation Coefficient with exact variance. None of
               these are in NIST/DIEHARD/DIEHARDER.}
}

@article{golic1997linear,
  author  = {Goli\'{c}, Jovan Dj.},
  title   = {On the Linear Complexity and Multidimensional Distribution of Decimated $m$-Sequences},
  journal = {IEEE Transactions on Information Theory},
  volume  = {43},
  number  = {3},
  pages   = {1054--1059},
  year    = {1997},
  doi     = {10.1109/18.568717},
  note    = {[TODO: library] IEEE paywalled; DOI needs verification (10.1109/18.568717 may be
             incorrect — verify against TIT vol.43 no.3 May 1997 pp.1054-1059 on IEEE Xplore).
             Decimated linear complexity: take every d-th output bit and run Berlekamp-Massey;
             complexity collapses at specific decimation factors for LFSR-based generators.}
}

@article{hellekalek2003aes,
  author  = {Hellekalek, Peter and Wegenkittl, Stefan},
  title   = {Empirical Evidence Concerning {AES}},
  journal = {ACM Transactions on Modeling and Computer Simulation},
  volume  = {13},
  number  = {4},
  pages   = {322--333},
  year    = {2003},
  doi     = {10.1145/945511.945515},
  note    = {[TODO: library] ACM paywalled; no author preprint found. ResearchGate listing:
             https://www.researchgate.net/publication/2953435
             Walsh-Hadamard spectral test; sensitive to nonlinear Boolean structure in
             keystream generators. Specifically applied to AES-based PRNGs, making it a
             natural complement to our AesCtr and CryptoCtrDrbg results.}
}

@inproceedings{doganaksoy2006bent,
  author    = {Doganaksoy, Ali and G\"{o}loglu, Fatih},
  title     = {On the Weakness of Non-Dual Bent Functions},
  booktitle = {Selected Areas in Cryptography (SAC 2005)},
  series    = {Lecture Notes in Computer Science},
  volume    = {3897},
  publisher = {Springer},
  year      = {2006},
  pages     = {50--64},
  doi       = {10.1007/11693383_4},
  note      = {L1-norm DFT variant: sum of all |DFT coefficients| rather
               than peak count. Catches diffuse periodic structure across many frequencies
               that NIST's threshold-exceedance statistic misses.
               Local copy not available.}
}

@inproceedings{webster1985sboxes,
  author    = {Webster, A. F. and Tavares, Stafford E.},
  title     = {On the Design of S-Boxes},
  booktitle = {Advances in Cryptology --- {CRYPTO} 1985},
  series    = {Lecture Notes in Computer Science},
  volume    = {218},
  publisher = {Springer},
  year      = {1986},
  pages     = {523--534},
  doi       = {10.1007/3-540-39799-X_41},
  note      = {[pubs/webster-tavares-1985-sbox-design.pdf] Strict Avalanche Criterion and
               Bit Independence Criterion. Requires a reseedable RNG interface; applicable
               to all seeded generators here to test differential output behavior under
               single-bit seed perturbations. Initial implementation now lives in
               `src/research/webster_tavares.rs` and `src/bin/webster_tavares.rs`.}
}
```

## Already-implemented test algorithms

Papers whose algorithms are **fully implemented** in this crate but that were
previously missing from this bibliography.

```bibtex
@article{pincus1991apen,
  author  = {Pincus, Steven M.},
  title   = {Approximate Entropy as a Measure of System Complexity},
  journal = {Proceedings of the National Academy of Sciences},
  volume  = {88},
  number  = {6},
  pages   = {2297--2301},
  year    = {1991},
  doi     = {10.1073/pnas.88.6.2297},
  note    = {[TODO: open-access PDF at PMC: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC51218/]
             Original ApEn(m) definition: φ(m) − φ(m+1) over overlapping patterns.
             NIST SP 800-22 §2.12 and `src/nist/approximate_entropy.rs` implement this statistic.
             Multi-scale sweep over m=2..6 is in `src/research/approx_entropy.rs`.}
}

@article{massey1969lfsr,
  author  = {Massey, James L.},
  title   = {Shift-Register Synthesis and {BCH} Decoding},
  journal = {IEEE Transactions on Information Theory},
  volume  = {15},
  number  = {1},
  pages   = {122--127},
  year    = {1969},
  doi     = {10.1109/TIT.1969.1054260},
  note    = {[TODO: library] The Berlekamp-Massey algorithm for computing the minimal LFSR
             that generates a given sequence.  Used in NIST SP 800-22 §2.10
             (`src/nist/linear_complexity.rs`) and implicitly in the linear-complexity
             profile test in TestU01 BigCrush.}
}

@article{grafton1981runs,
  author  = {Grafton, R. G. T.},
  title   = {Algorithm {AS} 157: The Runs-Up and Runs-Down Tests},
  journal = {Applied Statistics},
  volume  = {30},
  number  = {1},
  pages   = {81--85},
  year    = {1981},
  doi     = {10.2307/2346560},
  note    = {[TODO: library] Covariance matrix and expected proportions for the
             runs-up/down chi-square statistic.  Used verbatim in
             `src/diehard/runs_float.rs` (constant PSEUDO_INV_COV matrix) and
             `src/research/knuth.rs` (runs-above/below-median test).
             See also Knuth TAOCP Vol. 2 §3.3.2.}
}
```

---

## Implementation priority

| Priority | Key | What it adds |
|---|---|---|
| 1 | `lecuyer2007testu01` | LempelZiv, BirthdaySpacings, HammingCorr/HammingIndep — partially implemented; keep pushing toward BigCrush coverage |
| 2 | `practrand` | FPF core implemented; next priority is BCFN and DC6 to catch small-state generators that pass everything else |
| 3 | `maurer1992universal` | Full parametric universal test at L=10+ |
| 4 | `knuth1997taocp2` | Poker, Permutation, Wald-Wolfowitz, Serial Correlation |
| 5 | `golic1997linear` | Decimated linear complexity — relevant to Dual_EC analysis |
| 6 | `hellekalek2004aes` | Walsh-Hadamard spectral — validates AesCtr / CryptoCtrDrbg |
| 7 | `doganaksoy2006bent` | L1-norm DFT variant |
| 8 | `webster1985sboxes` | SAC / bit independence |
