//! Deliberately bad "generators" used to verify that each test detects failure.

use super::Rng;

/// Always returns the same 32-bit constant.
///
/// Every statistical test should FAIL against this generator.
#[derive(Debug, Clone)]
pub struct ConstantRng {
    value: u32,
}

impl ConstantRng {
    /// Construct a generator that returns `value` from every `next_u32` call.
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl Rng for ConstantRng {
    fn next_u32(&mut self) -> u32 {
        self.value
    }
}

/// Returns an incrementing counter: 0, 1, 2, 3, …
///
/// Every statistical test should FAIL against this generator.
#[derive(Debug, Clone)]
pub struct CounterRng {
    state: u32,
}

impl CounterRng {
    /// Construct a counter that emits `start, start+1, start+2, …` (wrapping).
    pub fn new(start: u32) -> Self {
        Self { state: start }
    }
}

impl Rng for CounterRng {
    fn next_u32(&mut self) -> u32 {
        let v = self.state;
        self.state = self.state.wrapping_add(1);
        v
    }
}
