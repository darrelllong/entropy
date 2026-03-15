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

@article{pincus1991approximate,
  author  = {Pincus, Steven M.},
  title   = {Approximate Entropy as a Measure of System Complexity},
  journal = {Proceedings of the National Academy of Sciences},
  volume  = {88},
  pages   = {2297--2301},
  year    = {1991},
  doi     = {10.1073/pnas.88.6.2297},
  note    = {[pubs/pincus-1991-approximate-entropy.pdf] Multi-scale ApEn(m) vs. m profile detects correlation length.
             NIST uses only a single scale; the full sweep is more sensitive.}
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
  note      = {[pubs/doganaksoy-gologlu-2006-bent-functions.pdf] L1-norm DFT variant: sum of all |DFT coefficients| rather
               than peak count. Catches diffuse periodic structure across many frequencies
               that NIST's threshold-exceedance statistic misses.}
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
| 7 | `pincus1991approximate` | Multi-scale ApEn |
| 8 | `doganaksoy2006bent` | L1-norm DFT variant |
| 9 | `webster1985sboxes` | SAC / bit independence |
