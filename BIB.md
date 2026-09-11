# Bibliography

References used or surveyed for this project. Entries marked **[pubs/]** have a local copy in `pubs/`.
Entries marked **[not in pubs/]** have no local copy; entries marked **[TODO: library]** still need
to be fetched from a library or publisher site.

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
  note   = {Florida State University. Source from the Internet Archive's copy of
            stat.fsu.edu/pub/diehard: Marsaglia's Fortran, diehard.f (January 1996), with
            tests.txt, diehard.doc and operm5d.ata [pubs/diehard-fortran-1996.tar.gz]; the f2c
            translation the DOS executables were built from, with four PostScript papers
            [pubs/diehard-f2c-source-1996.tar.gz]; and Dagang Wang's 1998 C translation, whose
            NOTES file lists where it differs from the Fortran [pubs/diehard-c-wang-1998.tar.gz].
            DOS executables and documentation: [pubs/diehard-doc.txt, pubs/diehard-tests.txt, pubs/Diehard.zip]}
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
  note    = {[pubs/marsaglia-tsang-2002-difficult-tests.pdf]; the C attached to the article, whose ADKS aggregate defines the Gorilla second stage: [pubs/marsaglia-tsang-2002-tuftests.c]}
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
  note    = {[pubs/matsumoto-nishimura-1998-mersenne-twister.pdf] (authors' preprint); reference code and its output check file: [pubs/mt19937ar.c], [pubs/mt19937ar.out] Period 2^{19937}-1; state recovery from 624 consecutive
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
  note    = {[pubs/blackman-vigna-2021-scrambled-linear-prngs.pdf] (arXiv:1805.01407v3); reference C: [pubs/vigna-xoshiro256starstar.c], [pubs/vigna-xoshiro256plusplus.c], [pubs/vigna-xoroshiro128plus.c], [pubs/vigna-xoroshiro128starstar.c], [pubs/vigna-xoroshiro128plusplus.c], [pubs/vigna-splitmix64.c] Xoshiro256** and
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

@misc{oneill-pcg-c,
  author = {O'Neill, M. E.},
  title  = {pcg-c: the reference C implementation of the {PCG} family},
  url    = {https://github.com/imneme/pcg-c},
  note   = {include/pcg_variants.h and the test-high expected outputs that src/rng/pcg.rs pins. [pubs/pcg-c-83252d9c23df.tar.gz] (commit 83252d9c23df9c82ecb42210afed61a7b42402d7)}
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

@misc{wangyi2022wyhash,
  author = {Wang, Yi},
  title  = {wyhash and wyrand},
  year   = {2022},
  url    = {https://github.com/wangyi-fudan/wyhash},
  note   = {[pubs/wyhash-e4764a0b637d.tar.gz] (commit e4764a0b637d34d3421a7760affada9288b625a8, whose wyhash.h is final version 4.3) Weyl-sequence counter with 128-bit
             multiply-xorfolded finaliser; passes BigCrush and PractRand > 8 TiB.  src/rng/wyrand.rs uses the
             wyrand constants of old\_versions/wyhash\_final2.h and wyhash\_final4.h, not those of 4.3.}
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
  note    = {MINSTD: a=16807, c=0, m=2^{31}-1 (Lehmer generator).  Also defines
             the Park-Miller test used by FreeBSD rand_r() compatibility path. [not in pubs/: the ACM Digital Library refuses automated download]}
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
  note   = {[TODO: fetch docs] Version pre-0.95 (the source the crate's FPF port follows).
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
  note    = {[not in pubs/: the ACM Digital Library refuses automated download] ACM paywalled; no author preprint found. ResearchGate listing:
             https://www.researchgate.net/publication/2953435
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
  note    = {[TODO: open-access PDF at PMC: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC51218/]
             Original ApEn(m) definition: φ(m) − φ(m+1) over overlapping patterns.
             NIST SP 800-22 §2.12 and `src/nist/approximate_entropy.rs` implement this statistic.
             Multi-scale sweep over m=2..6 is in `src/research/approx_entropy.rs`. [not in pubs/: PNAS and PubMed Central (PMC51218) refuse automated download]}
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
  note    = {[pubs/marsaglia-marsaglia-2004-anderson-darling.pdf]; the C attached to the article: [pubs/marsaglia-marsaglia-2004-ADinf.c], [pubs/marsaglia-marsaglia-2004-AnDarl.c]}
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
  note    = {[not in pubs/: the ACM Digital Library refuses automated download]}
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
  note    = {Taylor-series evaluation of Phi(x) and the complementary cPhi(x) in double precision.
             [pubs/marsaglia-2004-normal-distribution.pdf]; the C attached to the article:
             [pubs/marsaglia-2004-normal-distribution-sources.c]}
}

@misc{nist-sts-2.1.2,
  author       = {{National Institute of Standards and Technology}},
  title        = {{NIST} Statistical Test Suite, version 2.1.2},
  howpublished = {https://csrc.nist.gov/CSRC/media/Projects/Random-Bit-Generation/documents/sts-2\_1\_2.zip},
  note         = {Source code, templates and the constant expansions data.e, data.pi, data.sqrt2 and data.sqrt3; repacked without the generator-output files in data/ and the empty experiments/ tree. Original sha256 0238d2f1d26e120e3cc748ed2d4c674cdc636de37fc4027c76cc2a394fff9157. tests/data/e\_1e6\_bits.bin packs the first 10\^6 digits of data.e for the SP 800-22 worked examples. [pubs/NIST-STS-2.1.2-src-and-constants.zip]}
}

@misc{testu01-source,
  author       = {L'Ecuyer, Pierre and Simard, Richard},
  title        = {{TestU01} source code},
  howpublished = {https://github.com/umontreal-simul/TestU01-2009, commit 57e98bf33880daedc930739c81e39b89a8d24dba},
  note         = {Includes testu01/sstring.c (HammingCorr, HammingIndep) and testu01/scomp.c (LempelZiv). [pubs/TestU01-2009-57e98bf33880.tar.gz]}
}

@misc{gsl-2.8,
  author       = {Galassi, M. and others},
  title        = {{GNU} Scientific Library 2.8, random number generator sources},
  howpublished = {https://ftp.gnu.org/gnu/gsl/gsl-2.8.tar.gz},
  note         = {rng/, COPYING, AUTHORS and README only; gsl\_rng\_uniform\_int is in rng/gsl\_rng.h. Original sha256 6a99eeed15632c6354895b1dd542ed5a855c0f15d9ad1326c6fe2b2c9e423190. [pubs/gsl-2.8-rng-subset.tar.gz]}
}

@misc{glibc-2.40-random,
  author       = {{The GNU C Library contributors}},
  title        = {{GNU} C Library 2.40, stdlib/random.c, stdlib/random\_r.c and stdlib/rand.c},
  howpublished = {https://ftp.gnu.org/gnu/glibc/glibc-2.40.tar.xz},
  note         = {The srandom/random implementation LinuxLibcRandom emulates: \_\_initstate\_r and \_\_random\_r compiled from random\_r.c match it, and its TYPE\_0 generator matches LcgVariant::AnsiC for every seed whose low 32 bits are nonzero, which covers 1 to 2^{32}-1. Tarball sha256 19a890175e9263d748f627993de6f4b1af9cd21e03f080e4bfb3a1fac10205a2. [pubs/glibc-2.40-random_r.c], [pubs/glibc-2.40-random.c], [pubs/glibc-2.40-rand.c], whose rand() returns (int) \_\_random(), license [pubs/glibc-2.40-COPYING.LIB]}
}

@misc{freebsd-libc-random,
  author       = {{The FreeBSD Project}},
  title        = {{FreeBSD} libc stdlib/random.c and stdlib/rand.c},
  howpublished = {https://github.com/freebsd/freebsd-src, commit 0d022baa047aea6499e394d2cd8d097820ecf486},
  note         = {rand\_r() and random() at this commit. BsdRandCompat matches rand\_r(); random() seeds through parkmiller32, which shifts each word by one around the Park-Miller step, so its stream differs from BsdRandom's, which follows glibc. [pubs/freebsd-0d022baa047a-random.c], [pubs/freebsd-0d022baa047a-rand.c]}
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
