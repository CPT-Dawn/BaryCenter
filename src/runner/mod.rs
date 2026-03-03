// ── Runner Trait ─────────────────────────────────────────────────────────────
//
// The core abstraction that makes Barycenter modular.  Each "runner" is an
// independent module that can claim an input query, produce scored results,
// and execute a selection.
//
// v1.0 changes:
//   • Added `slug()` — a stable, machine-readable identifier used for result
//     dispatch and frecency tracking.  Eliminates fragile string matching.
//   • query() now takes `&mut SearchEngine` to avoid re-creating the matcher.
// ─────────────────────────────────────────────────────────────────────────────

pub mod app;
pub mod calc;
pub mod shell;
pub mod sys;

use crate::search::SearchEngine;

/// A single search result produced by a runner.
#[derive(Debug, Clone)]
pub struct RunnerResult {
    /// Optional icon name or path (for future use).
    #[allow(dead_code)]
    pub icon: Option<String>,
    /// Primary display text.
    pub title: String,
    /// Secondary / description text.
    pub description: String,
    /// Fuzzy match score — higher is better.
    pub relevance: u32,
    /// Unique identifier used when calling `execute`.
    pub id: String,
    /// Canonical slug of the runner that produced this result (e.g. "app", "calc").
    pub source: String,
}

/// The trait every module must implement.
pub trait Runner: Send + Sync {
    /// Human-readable module name (e.g. "Applications").
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Machine-readable slug (e.g. "app", "calc", "sys", "shell").
    /// Used for result dispatch and frecency keys.
    fn slug(&self) -> &str;

    /// Return `true` if this runner wants to respond to `input`.
    fn matches_input(&self, input: &str) -> bool;

    /// Produce up to `max_results` scored results for the given `input`.
    fn query(
        &self,
        input: &str,
        max_results: usize,
        engine: &mut SearchEngine,
    ) -> Vec<RunnerResult>;

    /// Execute / launch the result identified by `id`.
    fn execute(&self, id: &str) -> anyhow::Result<()>;
}
