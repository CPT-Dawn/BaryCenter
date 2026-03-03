// ── Barycenter Configuration Bootloader ──────────────────────────────────────
//
// Uses the "Embedded Asset Pattern":
// 1. The beautifully commented default_config.toml is baked into the binary
//    at compile time via `include_str!`.
// 2. On startup, if ~/.config/barycenter/config.toml doesn't exist, we write
//    the embedded default to disk.
// 3. We always parse from the on-disk file so user edits are respected.
// ─────────────────────────────────────────────────────────────────────────────

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// The default config baked into the binary.
const DEFAULT_CONFIG: &str = include_str!("../assets/default_config.toml");

/// Parsed, validated configuration for the entire application.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    // ── Window geometry ──────────────────────────────────────────────────
    pub width: u32,
    pub height: u32,

    // ── Colors ───────────────────────────────────────────────────────────
    pub border_color: String,
    pub background_color: String,
    pub text_color: String,
    pub accent_color: String,

    // ── Typography ───────────────────────────────────────────────────────
    #[allow(dead_code)]
    pub font_family: String,
    pub font_size: f32,

    // ── Behavior ─────────────────────────────────────────────────────────
    pub max_results: usize,
    pub border_width: f32,
    pub border_radius: f32,

    // ── Shell runner ─────────────────────────────────────────────────────
    #[serde(default = "default_terminal")]
    pub terminal: String,

    // ── Frecency ─────────────────────────────────────────────────────────
    #[serde(default = "default_frecency_enabled")]
    pub frecency_enabled: bool,
    #[serde(default = "default_frecency_decay")]
    pub frecency_decay: f64,
}

fn default_terminal() -> String {
    std::env::var("TERMINAL").unwrap_or_else(|_| "kitty".to_string())
}
fn default_frecency_enabled() -> bool {
    true
}
fn default_frecency_decay() -> f64 {
    0.1
}

impl AppConfig {
    /// Resolve the path: `~/.config/barycenter/config.toml`
    fn config_path() -> Result<PathBuf> {
        let base =
            directories::BaseDirs::new().context("Could not determine home / XDG directories")?;
        Ok(base.config_dir().join("barycenter").join("config.toml"))
    }

    /// The full bootloader sequence:
    /// 1. Ensure config dir + file exist (write embedded default if missing).
    /// 2. Read from disk.
    /// 3. Deserialize into the strict struct.
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        // ── Step 1: Bootstrap on-disk config if absent ───────────────────
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create config directory: {}", parent.display())
                })?;
            }
            fs::write(&path, DEFAULT_CONFIG)
                .with_context(|| format!("Failed to write default config to {}", path.display()))?;
            log::info!("Wrote default config to {}", path.display());
        }

        // ── Step 2: Read the on-disk file ────────────────────────────────
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;

        // ── Step 3: Parse into the strict struct ─────────────────────────
        let config: AppConfig = toml::from_str(&raw)
            .with_context(|| format!("Failed to parse config at {}", path.display()))?;

        log::info!("Loaded config from {}", path.display());
        Ok(config)
    }

    /// Load the embedded fallback without disk I/O.
    pub fn load_embedded() -> Self {
        toml::from_str(DEFAULT_CONFIG)
            .expect("Embedded default_config.toml is invalid — this is a compile-time bug")
    }
}

/// Parse a hex color string (#RRGGBB or #RRGGBBAA) into an iced Color.
///
/// Validates that all characters are ASCII hex digits before slicing
/// to prevent panics on malformed multi-byte UTF-8 input.
pub fn parse_hex_color(hex: &str) -> iced::Color {
    let hex = hex.trim_start_matches('#');

    // Guard: every char must be an ASCII hex digit.
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        log::warn!(
            "Invalid hex color '{}' (non-hex chars), falling back to white",
            hex
        );
        return iced::Color::WHITE;
    }

    let parse = |s: &str| u8::from_str_radix(s, 16).unwrap_or(0);

    match hex.len() {
        6 => {
            let r = parse(&hex[0..2]);
            let g = parse(&hex[2..4]);
            let b = parse(&hex[4..6]);
            iced::Color::from_rgba8(r, g, b, 1.0)
        }
        8 => {
            let r = parse(&hex[0..2]);
            let g = parse(&hex[2..4]);
            let b = parse(&hex[4..6]);
            let a = parse(&hex[6..8]);
            iced::Color::from_rgba8(r, g, b, a as f32 / 255.0)
        }
        _ => {
            log::warn!(
                "Invalid hex color '{}' (expected 6 or 8 chars), falling back to white",
                hex
            );
            iced::Color::WHITE
        }
    }
}
