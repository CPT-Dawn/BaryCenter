// ── SysRunner ────────────────────────────────────────────────────────────────
//
// Detects system commands (lock, logout, reboot, shutdown, suspend) and
// executes them via systemctl / loginctl.
// ─────────────────────────────────────────────────────────────────────────────

use super::{Runner, RunnerResult};
use crate::search::SearchEngine;

struct SysCommand {
    keyword: &'static str,
    display: &'static str,
    description: &'static str,
    icon: &'static str,
    command: &'static [&'static str],
}

const COMMANDS: &[SysCommand] = &[
    SysCommand {
        keyword: "lock",
        display: "Lock Screen",
        description: "Lock the current session",
        icon: "system-lock-screen",
        command: &["loginctl", "lock-session"],
    },
    SysCommand {
        keyword: "logout",
        display: "Log Out",
        description: "End the current session",
        icon: "system-log-out",
        command: &["loginctl", "terminate-session", "self"],
    },
    SysCommand {
        keyword: "reboot",
        display: "Reboot",
        description: "Restart the system",
        icon: "system-reboot",
        command: &["systemctl", "reboot"],
    },
    SysCommand {
        keyword: "shutdown",
        display: "Shut Down",
        description: "Power off the system",
        icon: "system-shutdown",
        command: &["systemctl", "poweroff"],
    },
    SysCommand {
        keyword: "suspend",
        display: "Suspend",
        description: "Suspend the system to RAM",
        icon: "system-suspend",
        command: &["systemctl", "suspend"],
    },
    SysCommand {
        keyword: "hibernate",
        display: "Hibernate",
        description: "Hibernate the system to disk",
        icon: "system-hibernate",
        command: &["systemctl", "hibernate"],
    },
];

pub struct SysRunner {
    keywords: Vec<String>,
}

impl SysRunner {
    pub fn new() -> Self {
        let keywords = COMMANDS.iter().map(|c| c.keyword.to_string()).collect();
        Self { keywords }
    }
}

impl Runner for SysRunner {
    fn name(&self) -> &str {
        "System"
    }

    fn slug(&self) -> &str {
        "sys"
    }

    fn matches_input(&self, input: &str) -> bool {
        // Cheap prefix check — avoids the old double-query.
        if input.is_empty() {
            return false;
        }
        let lower = input.to_ascii_lowercase();
        COMMANDS
            .iter()
            .any(|c| c.keyword.starts_with(&lower) || lower.starts_with(c.keyword))
    }

    fn query(
        &self,
        input: &str,
        max_results: usize,
        engine: &mut SearchEngine,
    ) -> Vec<RunnerResult> {
        let ranked = engine.rank(input, &self.keywords);

        ranked
            .into_iter()
            .take(max_results)
            .filter(|(_, score)| *score > 0)
            .map(|(idx, score)| {
                let cmd = &COMMANDS[idx];
                RunnerResult {
                    icon: Some(cmd.icon.to_string()),
                    title: cmd.display.to_string(),
                    description: cmd.description.to_string(),
                    relevance: score + 500,
                    id: cmd.keyword.to_string(),
                    source: "sys".to_string(),
                }
            })
            .collect()
    }

    fn execute(&self, id: &str) -> anyhow::Result<()> {
        let cmd = COMMANDS
            .iter()
            .find(|c| c.keyword == id)
            .ok_or_else(|| anyhow::anyhow!("Unknown system command: {}", id))?;

        log::info!(
            "Executing system command: {} {:?}",
            cmd.display,
            cmd.command
        );

        let parts = cmd.command;
        std::process::Command::new(parts[0])
            .args(&parts[1..])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", cmd.display, e))?;

        Ok(())
    }
}
