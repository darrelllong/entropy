//! `entropy` — pure, safe Rust statistical test suite for pseudorandom number generators.
//!
//! # Test suites
//!
//! | Module | Source |
//! |--------|--------|
//! | [`nist`] | NIST SP 800-22 Rev 1a \[`nist-sp-800-22`\] |
//! | [`diehard`] | DIEHARD (Marsaglia, 1995) \[`marsaglia1995diehard`\] |
//! | [`dieharder`] | DIEHARDER (Brown, 2006) \[`brown2006dieharder`\] |

#![forbid(unsafe_code)]

pub mod math;
pub mod result;
pub mod rng;

pub mod nist;
pub mod diehard;
pub mod dieharder;
pub mod research;
