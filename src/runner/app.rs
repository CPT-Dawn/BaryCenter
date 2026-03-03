// ── AppRunner ────────────────────────────────────────────────────────────────
//
// Parses `.desktop` files from standard directories on startup, caches them
// in memory, and provides instant fuzzy searching via nucleo-matcher.
// Launches applications via `std::process::Command`.
//
// v1.0: query() takes `&mut SearchEngine` to reuse the Matcher.
// ─────────────────────────────────────────────────────────────────────────────

use super::{Runner, RunnerResult};
use crate::search::SearchEngine;
use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A cached desktop entry.
#[derive(Debug, Clone)]
struct DesktopEntry {
    name: String,
    exec: String,
    description: String,
    icon: Option<String>,
    hidden: bool,
    path: String,
}

pub struct AppRunner {
    entries: Vec<DesktopEntry>,
    /// Pre-built search keys: "name  description" for fuzzy matching.
    search_keys: Vec<String>,
}

impl AppRunner {
    pub fn new() -> Self {
        let mut entries = Vec::new();

        let dirs: Vec<PathBuf> = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            home_dir()
                .map(|h| h.join(".local/share/applications"))
                .unwrap_or_default(),
        ];

        for dir in &dirs {
            if dir.as_os_str().is_empty() || !dir.exists() {
                continue;
            }
            Self::scan_directory(dir, &mut entries);
        }

        // Deduplicate by name (last entry wins — user overrides system).
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut deduped: Vec<DesktopEntry> = Vec::with_capacity(entries.len());
        for entry in entries {
            if entry.hidden {
                continue;
            }
            let key = entry.name.to_lowercase();
            if let Some(&idx) = seen.get(&key) {
                deduped[idx] = entry;
            } else {
                seen.insert(key, deduped.len());
                deduped.push(entry);
            }
        }

        let search_keys: Vec<String> = deduped
            .iter()
            .map(|e| format!("{} {}", e.name, e.description))
            .collect();

        log::info!("AppRunner: cached {} desktop entries", deduped.len());

        Self {
            entries: deduped,
            search_keys,
        }
    }

    fn scan_directory(dir: &Path, entries: &mut Vec<DesktopEntry>) {
        let read_dir = match fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => return,
        };

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::scan_directory(&path, entries);
            } else if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                if let Some(de) = Self::parse_desktop_file(&path) {
                    entries.push(de);
                }
            }
        }
    }

    fn parse_desktop_file(path: &Path) -> Option<DesktopEntry> {
        let content = fs::read_to_string(path).ok()?;

        let mut name = None;
        let mut exec = None;
        let mut comment = None;
        let mut generic_name = None;
        let mut icon = None;
        let mut no_display = false;
        let mut hidden = false;
        let mut in_desktop_entry = false;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with('[') {
                in_desktop_entry = line == "[Desktop Entry]";
                continue;
            }

            if !in_desktop_entry {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "Name" if name.is_none() => name = Some(value.to_string()),
                    "Exec" => exec = Some(Self::clean_exec(value)),
                    "Comment" if comment.is_none() => comment = Some(value.to_string()),
                    "GenericName" if generic_name.is_none() => {
                        generic_name = Some(value.to_string())
                    }
                    "Icon" => icon = Some(value.to_string()),
                    "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
                    "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
                    _ => {}
                }
            }
        }

        let name = name?;
        let exec = exec?;
        let description = generic_name.or(comment).unwrap_or_default();

        Some(DesktopEntry {
            name,
            exec,
            description,
            icon,
            hidden: no_display || hidden,
            path: path.to_string_lossy().into_owned(),
        })
    }

    fn clean_exec(exec: &str) -> String {
        exec.split_whitespace()
            .filter(|token| {
                !matches!(
                    *token,
                    "%f" | "%F"
                        | "%u"
                        | "%U"
                        | "%d"
                        | "%D"
                        | "%n"
                        | "%N"
                        | "%i"
                        | "%c"
                        | "%k"
                        | "%v"
                        | "%m"
                )
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

impl Runner for AppRunner {
    fn name(&self) -> &str {
        "Applications"
    }

    fn slug(&self) -> &str {
        "app"
    }

    fn matches_input(&self, _input: &str) -> bool {
        true
    }

    fn query(
        &self,
        input: &str,
        max_results: usize,
        engine: &mut SearchEngine,
    ) -> Vec<RunnerResult> {
        let ranked = engine.rank(input, &self.search_keys);

        ranked
            .into_iter()
            .take(max_results)
            .map(|(idx, score)| {
                let entry = &self.entries[idx];
                RunnerResult {
                    icon: entry.icon.clone(),
                    title: entry.name.clone(),
                    description: entry.description.clone(),
                    relevance: score,
                    id: entry.path.clone(),
                    source: "app".to_string(),
                }
            })
            .collect()
    }

    fn execute(&self, id: &str) -> anyhow::Result<()> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.path == id)
            .ok_or_else(|| anyhow::anyhow!("Desktop entry not found: {}", id))?;

        let parts: Vec<&str> = entry.exec.split_whitespace().collect();
        if parts.is_empty() {
            anyhow::bail!("Empty Exec line for {}", entry.name);
        }

        log::info!("Launching: {}", entry.exec);

        Command::new(parts[0])
            .args(&parts[1..])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to launch {}", entry.exec))?;

        Ok(())
    }
}
