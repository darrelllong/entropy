//! `entropy` — pure, safe Rust statistical test suite for pseudorandom number generators.
//!
//! # Modules
//!
//! | Module | Source |
//! |--------|--------|
//! | [`nist`] | NIST SP 800-22 Rev 1a \[`nist800-22`\] (feature `batteries`) |
//! | [`diehard`] | DIEHARD (Marsaglia, 1995) \[`marsaglia1995diehard`\] (feature `batteries`) |
//! | [`dieharder`] | DIEHARDER (Brown, 2004) \[`brown2004dieharder`\] (feature `batteries`) |
//! | [`research`] | Research tests after Knuth, Marsaglia–Tsang, L'Ecuyer–Simard and Doty-Humphrey (feature `batteries`) |
//! | [`rng`] | The generators under test, all implementing [`rng::Rng`] |
//! | [`math`] | Special functions: erfc, igamc, KS, Anderson–Darling, chi-square, FFT, GF(2) rank |
//! | [`seed`] | Deterministic seed-expansion helpers and fixed cipher test keys |
//! | [`result`] | [`result::TestResult`] and the shared significance level [`result::ALPHA`] |
//!
//! Citation keys refer to `BIB.md` in the repository root.
//!
//! # Features
//!
//! `batteries` (default) compiles the four test suites and the fast Fourier
//! transform the spectral test needs.  `cryptography` (default) adds the
//! cipher-, hash- and DRBG-backed generators from the sibling crate.  With
//! neither, the crate is the generators, [`rng::Sample`], [`rng::Seedable`],
//! [`math`] and [`result`], and it has no dependencies at all.
//!
//! # Example
//!
//! Seed a deterministic generator, collect bits, and run one NIST test:
//!
//! ```
//! # #[cfg(feature = "batteries")] {
//! use entropy::{nist, rng::{Mt19937, Rng}};
//!
//! let mut rng = Mt19937::new(5489);
//! let bits = rng.collect_bits(20_000);
//! let result = nist::frequency::frequency(&bits);
//! assert!(result.passed());
//! # }
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod math;
pub mod result;
pub mod rng;
pub mod seed;

/// The batteries, behind the `batteries` feature.
#[cfg(feature = "batteries")]
pub mod diehard;
#[cfg(feature = "batteries")]
pub mod dieharder;
#[cfg(feature = "batteries")]
pub mod nist;
#[cfg(feature = "batteries")]
pub mod research;
