// ── Fuzzy Search Engine ──────────────────────────────────────────────────────
//
// Performance-critical wrapper around `nucleo-matcher`.
//
// Key optimizations:
//   • The `Matcher` struct is created once and reused across all queries.
//     nucleo allocates internal scratch buffers on construction — re-creating
//     it per call defeats the caching that makes nucleo 6-8x faster than skim.
//   • A single `Vec<char>` buffer is reused for UTF-32 conversion instead of
//     allocating a fresh one per haystack.
// ─────────────────────────────────────────────────────────────────────────────

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Reusable search context.  Create once, call `score` / `rank` many times.
pub struct SearchEngine {
    matcher: Matcher,
    /// Shared buffer for Utf32Str conversion — avoids per-call heap allocation.
    buf: Vec<char>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
            buf: Vec::with_capacity(256),
        }
    }

    /// Score a single `haystack` against a `pattern` string.
    /// Returns `Some(score)` if there is a match, `None` otherwise.
    #[allow(dead_code)]
    pub fn score(&mut self, pattern: &str, haystack: &str) -> Option<u32> {
        if pattern.is_empty() {
            return Some(0);
        }

        let pat = Pattern::new(
            pattern,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        self.buf.clear();
        pat.score(Utf32Str::new(haystack, &mut self.buf), &mut self.matcher)
    }

    /// Score a slice of candidates, returning `(index, score)` pairs sorted
    /// descending by score.
    pub fn rank(&mut self, pattern: &str, candidates: &[String]) -> Vec<(usize, u32)> {
        if pattern.is_empty() {
            // Empty pattern matches everything with neutral score.
            return candidates.iter().enumerate().map(|(i, _)| (i, 0)).collect();
        }

        let pat = Pattern::new(
            pattern,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        let mut results: Vec<(usize, u32)> = candidates
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                self.buf.clear();
                pat.score(Utf32Str::new(c, &mut self.buf), &mut self.matcher)
                    .map(|s| (i, s))
            })
            .collect();

        results.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        results
    }
}
