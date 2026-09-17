# Bibliography

References used or surveyed for this project. Entries marked **[pubs/]** have a local copy in `pubs/`.
Entries marked **[not in pubs/]** have no local copy; entries marked **[TODO: library]** still need
to be fetched from a library or publisher site.

The point of keeping these files in-tree is that readers can check each implementation against the standard, manual or paper it follows.

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
  note   = {Florida State University, stat.fsu.edu/pub/diehard (via the Internet Archive).
            Documentation: [pubs/diehard-doc.txt, pubs/diehard-tests.txt]}
}

@article{marsaglia1993monkey,
  author  = {Marsaglia, George and Zaman, Arif},
  title   = {Monkey Tests for Random Number Generators},
  journal = {Computers \& Mathematics with Applications},
  volume  = {26},
  number  = {9},
  pages   = {1--10},
  year    = {1993},
  note    = {OPSO, OQSO and DNA, implemented in src/diehard/monkey.rs. Marsaglia's own extract of
             the article, distributed with DIEHARD as source/monkey.ps and converted with ps2pdf; it
             has no byline and its own pagination. [pubs/marsaglia-zaman-1993-monkey-tests.pdf]}
}

@misc{brown2004dieharder,
  author = {Brown, Robert G.},
  title  = {Dieharder: A Random Number Test Suite},
  year   = {2004},
  note   = {Version 3.31.x. [pubs/dieharder-manual.pdf]}
}

@article{marsaglia2002difficult,
  author  = {Marsaglia, George and Tsang, Wai Wan},
  title   = {Some Difficult-to-pass Tests of Randomness},
  journal = {Journal of Statistical Software},
  volume  = {7},
  number  = {3},
  year    = {2002},
  doi     = {10.18637/jss.v007.i03},
  note    = {Gorilla test behind research::marsaglia_tsang, and the GCD test. [pubs/marsaglia-tsang-2002-difficult-tests.pdf]}
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

@misc{nist-cavp-drbgvs,
  author       = {{National Institute of Standards and Technology}},
  title        = {{DRBG} Test Vectors ({CAVS} 14.3)},
  howpublished = {Cryptographic Algorithm Validation Program},
  year         = {2013},
  url          = {https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Algorithm-Validation-Program/documents/drbg/drbgtestvectors.zip},
  note         = {Known-answer vectors for the SP 800-90A DRBGs, generated 2 April 2013.  Kept here: HMAC\_DRBG.rsp from drbgvectors\_no\_reseed.zip.  Four of its sections open with the header [SHA-256] [PredictionResistance = False] [EntropyInputLen = 256] [NonceLen = 128] [PersonalizationStringLen = 0] [AdditionalInputLen = 0] [ReturnedBitsLen = 1024], each with its own COUNT = 0; the HMAC\_DRBG known-answer test in src/rng/hmac\_drbg.rs is in the first of them, header at line 4104, and is its COUNT = 0 record at line 4112, the one whose EntropyInput is DRBGVS\_ENTROPY\_INPUT there. [pubs/NIST-CAVP-drbgtestvectors-no_reseed-HMAC_DRBG.rsp]}
}

@techreport{nist800-90-2006,
  author      = {{National Institute of Standards and Technology}},
  title       = {Recommendation for Random Number Generation Using Deterministic Random Bit Generators},
  institution = {NIST},
  number      = {SP 800-90},
  year        = {2006},
  month       = jun,
  note        = {Original edition, withdrawn March 2007 by SP 800-90 Revised. Specifies Dual\_EC\_DRBG. [pubs/NIST-SP-800-90-2006.pdf]}
}

@techreport{nist800-90-2007,
  author      = {{National Institute of Standards and Technology}},
  title       = {Recommendation for Random Number Generation Using Deterministic Random Bit Generators (Revised)},
  institution = {NIST},
  number      = {SP 800-90 Revised},
  year        = {2007},
  month       = mar,
  note        = {Specifies Dual\_EC\_DRBG (section 10.3, Appendix A.1). Superseded by SP 800-90A (January 2012). [pubs/NIST-SP-800-90-2007.pdf]}
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
  note    = {[pubs/matsumoto-nishimura-1998-mersenne-twister.pdf] (authors' preprint). Period 2^{19937}-1; state recovery from 624 consecutive
             outputs is documented in §3.  Default generator in MATLAB and R;
             NumPy's legacy RandomState uses it, but NumPy's default_rng has been
             PCG64 since NumPy 1.17.}
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
  note    = {[pubs/blackman-vigna-2021-scrambled-linear-prngs.pdf] (arXiv:1805.01407v3) Xoshiro256** and
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

@misc{oneill-pcg-web,
  author = {O'Neill, M. E.},
  title  = {{PCG}, A Family of Better Random Number Generators},
  url    = {https://www.pcg-random.org},
  note   = {Default multipliers, stream seeding and the published outputs for seed (42, 54) that src/rng/pcg.rs pins.}
}
}

@article{marsaglia2003xorshift,
  author  = {Marsaglia, George},
  title   = {Xorshift {RNG}s},
  journal = {Journal of Statistical Software},
  volume  = {8},
  number  = {14},
  year    = {2003},
  doi     = {10.18637/jss.v008.i14},
  note    = {32-bit and 64-bit Xorshift generators; section 3 (p. 4) gives xor(), the 32-bit [13,17,5] generator,
             whose middle step is misprinted as y=(y>>17) without the xor, and xor64() with [13,7,17]. [pubs/marsaglia-2003-xorshift-rngs.pdf]}
}

@misc{jenkins2007smallprng,
  author = {Jenkins, Bob},
  title  = {A Small Noncryptographic {PRNG}},
  year   = {2007},
  url    = {http://burtleburtle.net/bob/rand/smallprng.html},
  note   = {[pubs/jenkins-2007-smallprng.html] (page saved 2026-09-11) JSF64 (Jenkins Small Fast),
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

@misc{rfc8439,
  author       = {Nir, Y. and Langley, A.},
  title        = {{ChaCha20} and {Poly1305} for {IETF} Protocols},
  howpublished = {RFC 8439},
  year         = {2018},
  month        = jun,
  note         = {The 96-bit-nonce, 32-bit-counter layout that cryptography::ChaCha20 follows. [pubs/rfc8439-chacha20-poly1305.txt]}
}

@misc{bernstein2005salsa20,
  author = {Bernstein, D. J.},
  title  = {{Salsa20} specification},
  year   = {2005},
  url    = {https://cr.yp.to/snuffle/spec.pdf},
  note   = {The Salsa20 cipher wrapped by src/rng/stream_rng.rs. [pubs/bernstein-2005-salsa20-spec.pdf]}
}

@misc{boesgaard2006rabbit,
  author       = {Boesgaard, M. and Vesterager, M. and Zenner, E.},
  title        = {A Description of the {Rabbit} Stream Cipher Algorithm},
  howpublished = {RFC 4503},
  year         = {2006},
  month        = may,
  note         = {Rabbit, wrapped by src/rng/stream_rng.rs; its known-answer test uses Appendix A.2. [pubs/rfc4503-rabbit.txt]; the eSTREAM description is [pubs/rabbit-estream-description.pdf]}
}

@misc{etsi-snow3g,
  author = {{ETSI/SAGE}},
  title  = {Specification of the 3GPP Confidentiality and Integrity Algorithms UEA2 \& UIA2, Document 2: {SNOW 3G} Specification},
  note   = {Version 1.1. The SNOW 3G cipher wrapped by src/rng/stream_rng.rs. [pubs/etsi-sage-snow3g-spec-v1.1.pdf]; Document 3, Implementors' Test Data v1.1, as GSMA publishes it in Word format: [pubs/etsi-sage-snow3g-testdata-v1.1.doc]}
}

@misc{etsi-zuc,
  author = {{ETSI/SAGE}},
  title  = {Specification of the 3GPP Confidentiality and Integrity Algorithms 128-EEA3 \& 128-EIA3, Document 2: {ZUC} Specification},
  note   = {Version 1.6. The ZUC-128 cipher wrapped by src/rng/stream_rng.rs. [pubs/etsi-sage-zuc-spec-v1.6.pdf]; Document 3, Implementor's Test Data v1.1: [pubs/etsi-sage-zuc-testdata-v1.1.pdf]}
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
  note    = {[pubs/park-miller-1988-good-ones-hard-to-find.pdf] MINSTD: a=16807, c=0, m=2^{31}-1 (Lehmer generator).  Also defines
             the Park-Miller test used by FreeBSD rand_r() compatibility path.}
}

@misc{unix-v7-manual,
  author = {Thompson, Ken and Ritchie, Dennis M.},
  title  = {Unix Programmer's Manual, 7th Edition},
  year   = {1979},
  note   = {Bell Laboratories. The rand(3) entry describes a multiplicative
             congruential generator with period 2^{32} returning 0 to 2^{15}-1;
             it does not print the a=1103515245, c=12345 parameters that
             SystemVRand and LcgVariant::AnsiC use.  Available at
             https://www.tuhs.org/Archive/Distributions/Research/V7/ [pubs/v7-unix-programmers-manual-vol1.pdf]}
}

@incollection{bernstein2016dualec,
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
               from 30 bytes of output. [pubs/bernstein-lange-niederhagen-2015-dual-ec.pdf] (IACR ePrint 2015/767 version)}
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
  note   = {[pubs/hughes-2022-badrandom-the-effect-and-mitigations-for-low-entropy-random-numbers-in-tls.pdf]
            The dissertation year is 2021 (UCSC / ProQuest record); the "2022" in the
            local filename is a filename artifact, not the publication year.}
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
  note   = {[TODO: fetch docs] The design of the FPF test in research::practrand_fpf.
             Novel tests: BCFN (DFT of Hamming-weight block
             counts), DC6 (lagged difference patterns for small-state generators), FPF
             (leading-bit frequency chi-square), TMFn (N-dim spectral), streaming linear
             complexity. Source: http://pracrand.sourceforge.net/ [not in pubs/: SourceForge refuses automated download]}
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
             substantially more sensitive than the single NIST-selected setting (SP 800-22
             chooses L from the sample size over L=6..16; L=7 is only the worked example
             in §2.9).  The crate's `maurer::` family covers L=5..16.
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
               (all t! orderings), Gap test, Serial Correlation Coefficient with exact variance.
               DIEHARDER's rgb_permutations also scores t! orderings (src/dieharder/permutations.rs);
               the others are not in NIST/DIEHARD/DIEHARDER. The runs test above/below the median
               that bib_tests runs is Wald and Wolfowitz's (wald1940runs), not the §3.3.2 run test. [TODO: library] (copyrighted book)}
}

@article{wald1940runs,
  author  = {Wald, Abraham and Wolfowitz, Jacob},
  title   = {On a Test Whether Two Samples are from the Same Population},
  journal = {Annals of Mathematical Statistics},
  volume  = {11},
  number  = {2},
  pages   = {147--162},
  year    = {1940},
  note    = {Conditional moments of the runs-above/below-median statistic in
             research::knuth::runs_above_below_median_test. [pubs/wald-wolfowitz-1940-runs.pdf]}
}

@article{golic1988decimated,
  author  = {Goli\'{c}, Jovan Dj. and \v{Z}ivkovi\'{c}, Miodrag V.},
  title   = {On the Linear Complexity of Nonuniformly Decimated {PN}-Sequences},
  journal = {IEEE Transactions on Information Theory},
  volume  = {34},
  number  = {5},
  pages   = {1077--1079},
  year    = {1988},
  note    = {[TODO: library] (IEEE, paywalled) IEEE paywalled (DOI not verified; omitted).
             Decimated linear complexity: take every d-th output bit and run Berlekamp-Massey;
             complexity collapses at specific decimation factors for LFSR-based generators.
             (This entry replaces an earlier citation of a 1997 Goli\'{c} single-author
             TIT paper that could not be verified to exist.)}
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
  note    = {[pubs/hellekalek-wegenkittl-2003-empirical-evidence-aes.pdf]
             Walsh-Hadamard spectral test; sensitive to nonlinear Boolean structure in
             keystream generators. Specifically applied to AES-based PRNGs, making it a
             natural complement to our AesCtr and CryptoCtrDrbg results.}
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
  note    = {[pubs/pincus-1991-approximate-entropy.pdf] (PubMed Central PMC51218)
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
  note    = {[TODO: library] (IEEE, paywalled) The Berlekamp-Massey algorithm for computing the minimal LFSR
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
  note    = {[TODO: library] (JSTOR, paywalled) Covariance matrix and expected proportions for the
             runs-up/down chi-square statistic.  Used verbatim in
             `src/diehard/runs_float.rs` (constant PSEUDO_INV_COV matrix).
             (The runs test in `src/research/knuth.rs` is the distinct
             Wald-Wolfowitz runs-above/below-median statistic, not this one.)
             See also Knuth TAOCP Vol. 2 §3.3.2.}
}
```

---

## Reference sources and supporting papers

```bibtex
@article{marsaglia2004anderson,
  author  = {Marsaglia, George and Marsaglia, John C. W.},
  title   = {Evaluating the {Anderson-Darling} Distribution},
  journal = {Journal of Statistical Software},
  volume  = {9},
  number  = {2},
  year    = {2004},
  doi     = {10.18637/jss.v009.i02},
  note    = {Anderson-Darling distribution behind math::anderson_darling_cdf and the Gorilla aggregate. [pubs/marsaglia-marsaglia-2004-anderson-darling.pdf]}
}

@article{marsaglia2003kolmogorov,
  author  = {Marsaglia, George and Tsang, Wai Wan and Wang, Jingbo},
  title   = {Evaluating {Kolmogorov}'s Distribution},
  journal = {Journal of Statistical Software},
  volume  = {8},
  number  = {18},
  year    = {2003},
  doi     = {10.18637/jss.v008.i18},
  note    = {Exact KS distribution behind math::ks_pvalue. [pubs/marsaglia-tsang-wang-2003-kolmogorov-distribution.pdf]}
}

@misc{kim2004niststs,
  author       = {Kim, Song-Ju and Umeno, Ken and Hasegawa, Akio},
  title        = {Corrections of the {NIST} Statistical Test Suite for Randomness},
  howpublished = {IACR Cryptology ePrint Archive, Report 2004/018},
  year         = {2004},
  note         = {[pubs/kim-umeno-hasegawa-2004-nist-sts-corrections.pdf]}
}

@article{hamano2007overlapping,
  author  = {Hamano, Kenji and Kaneko, Toshinobu},
  title   = {Correction of Overlapping Template Matching Test Included in {NIST} Randomness Test Suite},
  journal = {IEICE Transactions on Fundamentals of Electronics, Communications and Computer Sciences},
  volume  = {E90-A},
  number  = {9},
  pages   = {1788--1792},
  year    = {2007},
  doi     = {10.1093/ietfec/e90-a.9.1788},
  note    = {[not in pubs/: the IEICE site refuses automated download]}
}

@misc{killmann2004dft,
  author       = {Killmann, W. and Sch{\"u}th, J. and Thumser, W. and Uludag, I.},
  title        = {A Note Concerning the {DFT} Test in {NIST} Special Publication 800-22},
  howpublished = {T-Systems, Systems Integration},
  year         = {2004},
  note         = {Cited by SP 800-22 Rev. 1a. [not in pubs/: no public copy found]}
}

@article{lecuyer1999lcg,
  author  = {L'Ecuyer, Pierre and Simard, Richard},
  title   = {Beware of Linear Congruential Generators with Multipliers of the Form $a = \pm 2^q \pm 2^r$},
  journal = {ACM Transactions on Mathematical Software},
  volume  = {25},
  number  = {3},
  pages   = {367--374},
  year    = {1999},
  doi     = {10.1145/326147.326156},
  note    = {[pubs/lecuyer-simard-1999-beware-lcg-multipliers.pdf]}
}

@inproceedings{marsaglia1985currentview,
  author    = {Marsaglia, George},
  title     = {A Current View of Random Number Generators},
  booktitle = {Computer Science and Statistics: Proceedings of the 16th Symposium on the Interface},
  address   = {Atlanta},
  publisher = {Elsevier},
  year      = {1985},
  note      = {Keynote address, 1984; background for several DIEHARD tests. Marsaglia's retypeset
               copy, distributed with DIEHARD as source/keynote.ps and converted with ps2pdf.
               [pubs/marsaglia-1985-current-view-keynote.pdf]}
}

@article{marsaglia2004normal,
  author  = {Marsaglia, George},
  title   = {Evaluating the Normal Distribution},
  journal = {Journal of Statistical Software},
  volume  = {11},
  number  = {4},
  year    = {2004},
  doi     = {10.18637/jss.v011.i04},
  note    = {Taylor-series evaluation of the normal tail through Mills' ratio, behind math::erfc
             and math::normal_cdf. [pubs/marsaglia-2004-normal-distribution.pdf]}
}

@techreport{fischler2002mindist,
  author      = {Fischler, Mark},
  title       = {Distribution of Minimum Distance among {N} Random Points in d Dimensions},
  institution = {Fermi National Accelerator Laboratory},
  year        = {2002},
  url         = {https://www.osti.gov/biblio/794005},
  note        = {The second-order approximation and Q\_d coefficients behind dieharder::minimum\_distance\_nd. [not in pubs/]}
}

@article{gilpelaez1951inversion,
  author  = {Gil-Pelaez, J.},
  title   = {Note on the Inversion Theorem},
  journal = {Biometrika},
  volume  = {38},
  number  = {3--4},
  pages   = {481--482},
  year    = {1951},
  doi     = {10.1093/biomet/38.3-4.481},
  note    = {Characteristic-function inversion behind the correction table of diehard::historical::overlapping\_sums. [not in pubs/]}
}

@article{lentz1976continued,
  author  = {Lentz, W. J.},
  title   = {Generating {Bessel} Functions in {Mie} Scattering Calculations Using Continued Fractions},
  journal = {Applied Optics},
  volume  = {15},
  number  = {3},
  pages   = {668--671},
  year    = {1976},
  doi     = {10.1364/AO.15.000668},
  note    = {Continued-fraction evaluation behind math::igamc, with the modification of thompson1986coulomb. [not in pubs/]}
}

@article{thompson1986coulomb,
  author  = {Thompson, I. J. and Barnett, A. R.},
  title   = {Coulomb and {Bessel} Functions of Complex Arguments and Order},
  journal = {Journal of Computational Physics},
  volume  = {64},
  number  = {2},
  pages   = {490--509},
  year    = {1986},
  doi     = {10.1016/0021-9991(86)90046-X},
  note    = {The modified Lentz algorithm. [not in pubs/]}
}

@misc{dlmf8,
  author       = {Paris, R. B.},
  title        = {Incomplete Gamma and Related Functions},
  howpublished = {NIST Digital Library of Mathematical Functions, Chapter 8},
  url          = {https://dlmf.nist.gov/8},
  note         = {Series 8.7.1, continued fraction 8.9.2 and uniform expansion 8.12.3--8.12.8 behind math::igamc.}
}

@misc{dlmf5,
  author       = {Askey, R. A. and Roy, R.},
  title        = {Gamma Function},
  howpublished = {NIST Digital Library of Mathematical Functions, Chapter 5},
  url          = {https://dlmf.nist.gov/5},
  note         = {Stirling's series 5.11.1, the large-shape prefactor of math::igamc.}
}

@article{temme1979incomplete,
  author  = {Temme, N. M.},
  title   = {The Asymptotic Expansion of the Incomplete Gamma Functions},
  journal = {SIAM Journal on Mathematical Analysis},
  volume  = {10},
  number  = {4},
  pages   = {757--766},
  year    = {1979},
  doi     = {10.1137/0510071},
  note    = {The uniform expansion behind math::igamc for a $\geq 10^5$; the c$_0$ and c$_1$ power series in $\mu$ were derived here. [not in pubs/]}
}

@article{stephens1974edf,
  author  = {Stephens, M. A.},
  title   = {{EDF} Statistics for Goodness of Fit and Some Comparisons},
  journal = {Journal of the American Statistical Association},
  volume  = {69},
  number  = {347},
  pages   = {730--737},
  year    = {1974},
  doi     = {10.1080/01621459.1974.10480196},
  note    = {The modified Kolmogorov--Smirnov argument behind math::ks\_pvalue for large n. [not in pubs/]}
}

@article{marsaglia1985matrices,
  author  = {Marsaglia, George and Tsay, L. H.},
  title   = {Matrices and the Structure of Random Number Sequences},
  journal = {Linear Algebra and its Applications},
  volume  = {67},
  pages   = {147--156},
  year    = {1985},
  doi     = {10.1016/0024-3795(85)90192-2},
  note    = {The rank distribution of random binary matrices behind diehard::binary\_rank and nist::matrix\_rank. [not in pubs/]}
}
@article{lemire2019interval,
  author  = {Lemire, Daniel},
  title   = {Fast Random Integer Generation in an Interval},
  journal = {ACM Transactions on Modeling and Computer Simulation},
  volume  = {29},
  number  = {1},
  pages   = {Article 3},
  year    = {2019},
  doi     = {10.1145/3230636},
  note    = {Algorithm 5, the multiply-and-reject method behind rng::Sample::below. [pubs/lemire-2019-fast-random-integer-in-interval.pdf]}
}

@article{durstenfeld1964permutation,
  author  = {Durstenfeld, Richard},
  title   = {Algorithm 235: Random Permutation},
  journal = {Communications of the ACM},
  volume  = {7},
  number  = {7},
  pages   = {420},
  year    = {1964},
  doi     = {10.1145/364520.364540},
  note    = {The in-place shuffle behind rng::Sample::shuffle and partial\_shuffle. [not in pubs/: the ACM Digital Library refuses automated download]}
}

@article{bentley1987sample,
  author  = {Bentley, Jon and Floyd, Robert},
  title   = {Programming Pearls: A Sample of Brilliance},
  journal = {Communications of the ACM},
  volume  = {30},
  number  = {9},
  pages   = {754--757},
  year    = {1987},
  doi     = {10.1145/30401.315746},
  note    = {Floyd's algorithm for distinct samples, behind rng::Sample::sample\_indices. [not in pubs/: the ACM Digital Library refuses automated download]}
}

@article{vitter1985reservoir,
  author  = {Vitter, Jeffrey Scott},
  title   = {Random Sampling with a Reservoir},
  journal = {ACM Transactions on Mathematical Software},
  volume  = {11},
  number  = {1},
  pages   = {37--57},
  year    = {1985},
  doi     = {10.1145/3147.3165},
  note    = {Algorithm R, behind rng::Sample::choose\_from\_iter and sample\_from\_iter. [pubs/vitter-1985-reservoir-sampling.pdf]}
}

@misc{bernstein2017fastkeyerasure,
  author = {Bernstein, Daniel J.},
  title  = {Fast-key-erasure random-number generators},
  year   = {2017},
  month  = jul,
  url    = {https://blog.cr.yp.to/20170723-random.html},
  note   = {The design of rng::FastKeyErasureRng and thread\_rng. [pubs/bernstein-2017-fast-key-erasure-rng.html]}
}

@inproceedings{steele2014splittable,
  author    = {Steele, Guy L., Jr. and Lea, Doug and Flood, Christine H.},
  title     = {Fast Splittable Pseudorandom Number Generators},
  booktitle = {Proceedings of the 2014 ACM International Conference on Object Oriented Programming Systems Languages \& Applications (OOPSLA)},
  pages     = {453--472},
  year      = {2014},
  doi       = {10.1145/2660193.2660195},
  note      = {SplitMix64, behind seed::splitmix64 and rng::Seedable::seed\_from\_u64. [not in pubs/: the ACM Digital Library refuses automated download]}
}

@article{marsaglia2000ziggurat,
  author  = {Marsaglia, George and Tsang, Wai Wan},
  title   = {The Ziggurat Method for Generating Random Variables},
  journal = {Journal of Statistical Software},
  volume  = {5},
  number  = {8},
  year    = {2000},
  doi     = {10.18637/jss.v005.i08},
  note    = {The ziggurat method. [pubs/marsaglia-tsang-2000-ziggurat.pdf]}
}

@phdthesis{ville1939collectif,
  author = {Ville, Jean},
  title  = {{\'E}tude critique de la notion de collectif},
  school = {Universit{\'e} de Paris},
  year   = {1939},
  note   = {Published by Gauthier-Villars.  The maximal inequality for nonnegative martingales behind research::sequential. [pubs/ville-1939-etude-critique-collectif.pdf]}
}

@article{howard2021timeuniform,
  author  = {Howard, Steven R. and Ramdas, Aaditya and McAuliffe, Jon and Sekhon, Jasjeet},
  title   = {Time-uniform, nonparametric, nonasymptotic confidence sequences},
  journal = {Annals of Statistics},
  volume  = {49},
  number  = {2},
  pages   = {1055--1080},
  year    = {2021},
  doi     = {10.1214/20-AOS1991},
  note    = {Anytime-valid inference, research::sequential. [pubs/howard-ramdas-mcauliffe-sekhon-2021-time-uniform-confidence-sequences.pdf]}
}

@article{krichevsky1981universal,
  author  = {Krichevsky, Raphail E. and Trofimov, Victor K.},
  title   = {The performance of universal encoding},
  journal = {IEEE Transactions on Information Theory},
  volume  = {27},
  number  = {2},
  pages   = {199--207},
  year    = {1981},
  doi     = {10.1109/TIT.1981.1056331},
  note    = {The estimator (n\_1 + 1/2)/(n + 1) behind research::sequential. [not in pubs/: IEEE Xplore requires a subscription]}
}

@article{willems1995ctw,
  author  = {Willems, Frans M. J. and Shtarkov, Yuri M. and Tjalkens, Tjalling J.},
  title   = {The context-tree weighting method: basic properties},
  journal = {IEEE Transactions on Information Theory},
  volume  = {41},
  number  = {3},
  pages   = {653--664},
  year    = {1995},
  doi     = {10.1109/18.382012},
  note    = {Mixtures of context models, research::sequential. [not in pubs/: IEEE Xplore requires a subscription]}
}

@article{makhoul1980fastcosine,
  author  = {Makhoul, John},
  title   = {A fast cosine transform in one and multiple dimensions},
  journal = {IEEE Transactions on Acoustics, Speech, and Signal Processing},
  volume  = {28},
  number  = {1},
  pages   = {27--34},
  year    = {1980},
  doi     = {10.1109/TASSP.1980.1163351},
  note    = {The DCT-II through one FFT, behind dieharder::dct. [not in pubs/: IEEE Xplore requires a subscription]}
}

@book{bartlett1955stochastic,
  author    = {Bartlett, M. S.},
  title     = {An Introduction to Stochastic Processes},
  publisher = {Cambridge University Press},
  year      = {1955},
  note      = {The cumulative periodogram test in scripts/r\_rng\_tests.R. [not in pubs/: in copyright, no open copy]}
}

@article{park1993response,
  author  = {Park, Stephen K. and Miller, Keith W. and Stockmeyer, Paul K.},
  title   = {Technical Correspondence: Response},
  journal = {Communications of the ACM},
  volume  = {36},
  number  = {7},
  pages   = {108--110},
  year    = {1993},
  note    = {The MINSTD multiplier 48271 behind rng::LcgVariant::Minstd. [not in pubs/: the ACM Digital Library refuses automated download]}
}
```

---

## Implementation priority

| Priority | Key | What it adds |
|---|---|---|
| 1 | `lecuyer2007testu01` | LempelZiv, BirthdaySpacings, HammingCorr/HammingIndep — partially implemented; keep pushing toward BigCrush coverage |
| 2 | `practrand` | FPF core implemented; next priority is BCFN and DC6 to catch small-state generators that pass everything else |
| 3 | `maurer1992universal` | Full parametric universal test at L=10+ |
| 4 | `knuth1997taocp2` | Poker, Permutation, Serial Correlation |
| 5 | `golic1988decimated` | Decimated linear complexity — relevant to Dual_EC analysis |
| 6 | `hellekalek2003aes` | Walsh-Hadamard spectral — validates AesCtr / CryptoCtrDrbg |
| 7 | `webster1985sboxes` | SAC / bit independence |
