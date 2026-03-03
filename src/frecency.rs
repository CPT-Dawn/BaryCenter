// ── Frecency Engine ──────────────────────────────────────────────────────────
//
// Tracks launch history and computes a frecency score that blends frequency
// (how often) with recency (how recently).  Stored as a JSON file at:
//
//     ~/.local/share/barycenter/frecency.json
//
// Format: { "app_id": [timestamp_epoch_secs, ...], ... }
//
// Boost formula per entry:
//     boost = Σ  e^(-decay * age_in_hours)
//
// where `decay` is configurable (default 0.1).
// ─────────────────────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of timestamps stored per app ID to prevent unbounded growth.
const MAX_TIMESTAMPS: usize = 50;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FrecencyDb {
    /// Map from app IDs to lists of launch timestamps (epoch seconds).
    entries: HashMap<String, Vec<u64>>,

    /// Filesystem path — not serialised.
    #[serde(skip)]
    path: PathBuf,

    /// Decay parameter — not serialised (comes from config).
    #[serde(skip)]
    decay: f64,
}

impl FrecencyDb {
    /// Load from disk, or create an empty DB if the file doesn't exist / is
    /// malformed.  Never panics.
    pub fn load(decay: f64) -> Self {
        let path = Self::db_path();
        let mut db = match std::fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str::<FrecencyDb>(&data).unwrap_or_default(),
            Err(_) => FrecencyDb::default(),
        };
        db.path = path;
        db.decay = decay;
        log::info!("FrecencyDb: loaded {} entries", db.entries.len());
        db
    }

    /// Record a launch event for the given app ID.  Persists to disk.
    pub fn record_launch(&mut self, id: &str) {
        let now = Self::now_secs();
        let timestamps = self.entries.entry(id.to_string()).or_default();
        timestamps.push(now);

        // Evict oldest entries if over the cap.
        if timestamps.len() > MAX_TIMESTAMPS {
            let drain = timestamps.len() - MAX_TIMESTAMPS;
            timestamps.drain(..drain);
        }

        self.save();
    }

    /// Compute the frecency boost for a given app ID.
    /// Returns 0 if no history exists.
    pub fn boost(&self, id: &str) -> u32 {
        let timestamps = match self.entries.get(id) {
            Some(ts) => ts,
            None => return 0,
        };

        let now = Self::now_secs();
        let score: f64 = timestamps
            .iter()
            .map(|&ts| {
                let age_hours = (now.saturating_sub(ts) as f64) / 3600.0;
                (-self.decay * age_hours).exp()
            })
            .sum();

        // Scale to a reasonable integer range (0–500).
        (score * 100.0).min(500.0) as u32
    }

    // ── Private helpers ──────────────────────────────────────────────────

    fn db_path() -> PathBuf {
        directories::ProjectDirs::from("", "", "barycenter")
            .map(|p| p.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("frecency.json")
    }

    fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(&self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&self.path, json) {
                    log::warn!("FrecencyDb: failed to write {}: {}", self.path.display(), e);
                }
            }
            Err(e) => log::warn!("FrecencyDb: serialization error: {}", e),
        }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}
