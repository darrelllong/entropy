//! `entropy` — pure, safe Rust statistical test suite for pseudorandom number generators.
//!
//! # Modules
//!
//! | Module | Source |
//! |--------|--------|
//! | [`nist`] | NIST SP 800-22 Rev 1a \[`nist800-22`\] |
//! | [`diehard`] | DIEHARD (Marsaglia, 1995) \[`marsaglia1995diehard`\] |
//! | [`dieharder`] | DIEHARDER (Brown, 2004) \[`brown2004dieharder`\] |
//! | [`research`] | Research-grade tests: Knuth TAOCP, Marsaglia–Tsang, TestU01, PractRand |
//! | [`rng`] | The generators under test, all implementing [`rng::Rng`] |
//! | [`math`] | Special functions: erfc, igamc, KS, chi-square, FFT, GF(2) rank |
//! | [`seed`] | Deterministic seed-expansion helpers and fixed cipher test keys |
//! | [`result`] | [`result::TestResult`] and the shared significance level [`result::ALPHA`] |
//!
//! Citation keys refer to `BIB.md` in the repository root.
//!
//! # Example
//!
//! Seed a deterministic generator, collect bits, and run one NIST test:
//!
//! ```
//! use entropy::{nist, rng::{Mt19937, Rng}};
//!
//! let mut rng = Mt19937::new(5489);
//! let bits = rng.collect_bits(20_000);
//! let result = nist::frequency::frequency(&bits);
//! assert!(result.passed());
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod math;
pub mod result;
pub mod rng;
pub mod seed;

pub mod diehard;
pub mod dieharder;
pub mod nist;
pub mod research;
