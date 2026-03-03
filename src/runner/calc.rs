// ── CalcRunner ───────────────────────────────────────────────────────────────
//
// Detects if the user's input looks like a math expression and evaluates it
// in real-time using `meval`.  The result is shown as a search result.
// Executing the result copies it to the clipboard via `wl-copy`.
// ─────────────────────────────────────────────────────────────────────────────

use super::{Runner, RunnerResult};
use crate::search::SearchEngine;

pub struct CalcRunner;

impl CalcRunner {
    pub fn new() -> Self {
        Self
    }

    /// Heuristic: the input looks like math if it contains at least one digit
    /// AND at least one math operator or function call.
    fn looks_like_math(input: &str) -> bool {
        let has_digit = input.chars().any(|c| c.is_ascii_digit());
        let has_operator = input
            .chars()
            .any(|c| matches!(c, '+' | '-' | '*' | '/' | '^' | '%' | '('));
        let is_pure_number = input.trim().parse::<f64>().is_ok();

        has_digit && (has_operator || is_pure_number)
    }

    /// Format the result nicely: strip trailing zeros, handle integers.
    fn format_result(value: f64) -> String {
        if value.fract() == 0.0 && value.abs() < i64::MAX as f64 {
            format!("{}", value as i64)
        } else {
            let s = format!("{:.10}", value);
            let s = s.trim_end_matches('0');
            let s = s.trim_end_matches('.');
            s.to_string()
        }
    }
}

impl Runner for CalcRunner {
    fn name(&self) -> &str {
        "Calculator"
    }

    fn slug(&self) -> &str {
        "calc"
    }

    fn matches_input(&self, input: &str) -> bool {
        Self::looks_like_math(input)
    }

    fn query(
        &self,
        input: &str,
        _max_results: usize,
        _engine: &mut SearchEngine,
    ) -> Vec<RunnerResult> {
        match meval::eval_str(input) {
            Ok(value) => {
                let formatted = Self::format_result(value);
                vec![RunnerResult {
                    icon: Some("accessories-calculator".to_string()),
                    title: format!("= {}", formatted),
                    description: format!("{} = {}", input.trim(), formatted),
                    relevance: 1000, // Always rank calculator results high.
                    id: formatted,
                    source: "calc".to_string(),
                }]
            }
            Err(_) => vec![], // Not a valid expression — silently produce nothing.
        }
    }

    fn execute(&self, id: &str) -> anyhow::Result<()> {
        // Copy the result to the Wayland clipboard via wl-copy.
        log::info!("Copying to clipboard: {}", id);
        std::process::Command::new("wl-copy")
            .arg(id)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| {
                anyhow::anyhow!("Failed to run wl-copy: {}. Is wl-clipboard installed?", e)
            })?;
        Ok(())
    }
}
