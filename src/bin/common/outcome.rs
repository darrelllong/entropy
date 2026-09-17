//! Whether an auxiliary run completed.
//!
//! A run is incomplete when any result is an ERROR (a p-value that is not a
//! probability) or UNSUPPORTED (parameters outside the test's domain).  A
//! statistical rejection is a completed result.  Incomplete runs exit 3, as
//! `run_tests` does.

use entropy::result::TestResult;

#[derive(Default)]
pub struct Outcome {
    incomplete: Vec<String>,
}

impl Outcome {
    /// Note `result` of generator `label` if it did not complete.
    #[allow(dead_code)]
    pub fn record(&mut self, label: &str, result: &TestResult) {
        if result.errored() || result.is_unsupported() {
            self.incomplete.push(format!("{label}: {}", result.name));
        }
    }

    /// Note a bare p-value of `name` that is not a probability.
    #[allow(dead_code)]
    pub fn record_p(&mut self, label: &str, name: &str, p_value: f64) {
        if !(0.0..=1.0).contains(&p_value) {
            self.incomplete
                .push(format!("{label}: {name} p = {p_value}"));
        }
    }

    /// Exit 3 after listing the incomplete results, if there are any.
    pub fn exit_if_incomplete(self) {
        if !self.incomplete.is_empty() {
            eprintln!(
                "error: {} result(s) incomplete (ERROR, UNSUPPORTED or an invalid p-value):",
                self.incomplete.len()
            );
            for line in &self.incomplete {
                eprintln!("  {line}");
            }
            std::process::exit(3);
        }
    }
}
