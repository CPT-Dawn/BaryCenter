// ── ShellRunner ──────────────────────────────────────────────────────────────
//
// Power-user feature: type `> command` to execute an arbitrary shell command
// inside the configured terminal emulator.
//
// Prefix: `>`
// Execution: $TERMINAL -e $SHELL -c "command; exec $SHELL"
// ─────────────────────────────────────────────────────────────────────────────

use super::{Runner, RunnerResult};
use crate::search::SearchEngine;
use std::process::Command;

pub struct ShellRunner {
    terminal: String,
}

impl ShellRunner {
    pub fn new(terminal: String) -> Self {
        Self { terminal }
    }

    /// Strip the leading `>` and any surrounding whitespace.
    fn strip_prefix(input: &str) -> Option<&str> {
        input.strip_prefix('>').map(str::trim)
    }
}

impl Runner for ShellRunner {
    fn name(&self) -> &str {
        "Shell"
    }

    fn slug(&self) -> &str {
        "shell"
    }

    fn matches_input(&self, input: &str) -> bool {
        input.starts_with('>')
    }

    fn query(
        &self,
        input: &str,
        _max_results: usize,
        _engine: &mut SearchEngine,
    ) -> Vec<RunnerResult> {
        let cmd = match Self::strip_prefix(input) {
            Some(c) if !c.is_empty() => c,
            _ => return vec![],
        };

        vec![RunnerResult {
            icon: Some("utilities-terminal".to_string()),
            title: format!("Run: {}", cmd),
            description: format!("Execute in {} ", self.terminal),
            relevance: 2000, // Always top when prefix matches.
            id: cmd.to_string(),
            source: "shell".to_string(),
        }]
    }

    fn execute(&self, id: &str) -> anyhow::Result<()> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());

        log::info!("Shell exec via {}: {}", self.terminal, id);

        // Launch: terminal -e shell -c "command; exec shell"
        // The trailing `exec $SHELL` keeps the terminal open after the command.
        let script = format!("{}; exec {}", id, shell);

        Command::new(&self.terminal)
            .arg("-e")
            .arg(&shell)
            .arg("-c")
            .arg(&script)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to launch '{}': {}. Is '{}' installed?",
                    id,
                    e,
                    self.terminal
                )
            })?;

        Ok(())
    }
}
