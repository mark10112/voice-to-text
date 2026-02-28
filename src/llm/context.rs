//! Rolling-window context manager for LLM-based STT correction.
//!
//! [`ContextManager`] keeps the last *N* corrected sentences and produces a
//! compact context string that [`PromptBuilder`](crate::llm::PromptBuilder)
//! embeds in every correction prompt.  The context includes:
//!
//! * Detected domain (if any) from [`DomainDetector`]
//! * Top user vocabulary entries from [`UserVocabulary`]
//! * Previous sentences (most recent first)
//!
//! The context window is cleared automatically when the user is silent for
//! longer than `silence_reset` seconds (default 120 s), which signals a topic
//! change.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::llm::domain::DomainDetector;
use crate::llm::vocabulary::UserVocabulary;

// ---------------------------------------------------------------------------
// ContextManager
// ---------------------------------------------------------------------------

/// Maintains a rolling window of corrected sentences for context injection.
///
/// # Example
/// ```rust
/// use voice_to_text::llm::ContextManager;
///
/// let mut mgr = ContextManager::new();
/// mgr.push_sentence("ผู้ป่วยมีไข้สูง".to_string());
/// let ctx = mgr.build_context();
/// assert!(ctx.is_some());
/// ```
pub struct ContextManager {
    sentences: VecDeque<String>,
    max_sentences: usize,
    domain_detector: DomainDetector,
    user_vocab: UserVocabulary,
    last_activity: Instant,
    silence_reset: Duration,
}

impl ContextManager {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Create a context manager with default settings:
    /// * `max_sentences` = 3
    /// * `silence_reset` = 120 seconds
    pub fn new() -> Self {
        Self {
            sentences: VecDeque::with_capacity(5),
            max_sentences: 3,
            domain_detector: DomainDetector::new(),
            user_vocab: UserVocabulary::load_or_default(),
            last_activity: Instant::now(),
            silence_reset: Duration::from_secs(120),
        }
    }

    /// Create a context manager with custom limits (useful for testing).
    pub fn with_capacity(max_sentences: usize, silence_reset_secs: u64) -> Self {
        Self {
            sentences: VecDeque::with_capacity(max_sentences + 2),
            max_sentences,
            domain_detector: DomainDetector::new(),
            user_vocab: UserVocabulary::load_or_default(),
            last_activity: Instant::now(),
            silence_reset: Duration::from_secs(silence_reset_secs),
        }
    }

    // -----------------------------------------------------------------------
    // Mutation
    // -----------------------------------------------------------------------

    /// Add a corrected sentence to the rolling window.
    ///
    /// If the elapsed time since the last sentence exceeds `silence_reset`
    /// the window is cleared first (topic-change heuristic).  Oldest entries
    /// are dropped once the window exceeds `max_sentences`.
    pub fn push_sentence(&mut self, sentence: String) {
        // Silence-based context reset
        if self.last_activity.elapsed() > self.silence_reset {
            self.sentences.clear();
        }

        self.sentences.push_back(sentence);

        // Trim to max window size
        while self.sentences.len() > self.max_sentences {
            self.sentences.pop_front();
        }

        self.last_activity = Instant::now();
    }

    /// Clear the rolling window immediately (e.g. user pressed a "clear" button).
    pub fn reset(&mut self) {
        self.sentences.clear();
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Build a compact context string for injection into the correction prompt.
    ///
    /// Returns `None` when the window is empty (nothing to inject).
    ///
    /// The returned string has sections:
    /// ```text
    /// Domain: Medical          ← present only when domain detected
    /// User-specific terms:     ← present only when vocab is non-empty
    /// - "ผิด" → "ถูก"
    /// Previous context:
    /// - ประโยคล่าสุด
    /// - ประโยคก่อนหน้า
    /// ```
    pub fn build_context(&self) -> Option<String> {
        if self.sentences.is_empty() {
            return None;
        }

        let all_text: String = self.sentences.iter().cloned().collect::<Vec<_>>().join(" ");

        let mut ctx = String::with_capacity(512);

        // Domain hint
        if let Some(domain) = self.domain_detector.detect(&all_text) {
            ctx.push_str(&format!("Domain: {}\n", domain));
        }

        // User vocabulary (top 5)
        let vocab = self.user_vocab.top_entries(5);
        if !vocab.is_empty() {
            ctx.push_str("User-specific terms:\n");
            for (error, correction) in &vocab {
                ctx.push_str(&format!("- \"{}\" → \"{}\"\n", error, correction));
            }
        }

        // Previous sentences (oldest first, most recent last)
        ctx.push_str("Previous context:\n");
        for sentence in &self.sentences {
            ctx.push_str(&format!("- {}\n", sentence));
        }

        Some(ctx)
    }

    /// Number of sentences currently held in the window.
    pub fn len(&self) -> usize {
        self.sentences.len()
    }

    /// Returns `true` when the window is empty.
    pub fn is_empty(&self) -> bool {
        self.sentences.is_empty()
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty() {
        let mgr = ContextManager::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert_eq!(mgr.build_context(), None);
    }

    #[test]
    fn push_single_sentence_produces_context() {
        let mut mgr = ContextManager::new();
        mgr.push_sentence("สวัสดีครับ".to_string());
        let ctx = mgr.build_context().expect("context should be Some");
        assert!(ctx.contains("สวัสดีครับ"));
        assert!(ctx.contains("Previous context:"));
    }

    #[test]
    fn rolling_window_caps_at_max_sentences() {
        let mut mgr = ContextManager::with_capacity(3, 120);
        for i in 0..6 {
            mgr.push_sentence(format!("ประโยค{}", i));
        }
        // Window must not exceed 3
        assert_eq!(mgr.len(), 3);

        let ctx = mgr.build_context().unwrap();
        // Oldest entries should be gone
        assert!(!ctx.contains("ประโยค0"));
        assert!(!ctx.contains("ประโยค1"));
        assert!(!ctx.contains("ประโยค2"));
        // Most recent three should be present
        assert!(ctx.contains("ประโยค3"));
        assert!(ctx.contains("ประโยค4"));
        assert!(ctx.contains("ประโยค5"));
    }

    #[test]
    fn reset_clears_window() {
        let mut mgr = ContextManager::new();
        mgr.push_sentence("ประโยคทดสอบ".to_string());
        mgr.reset();
        assert!(mgr.is_empty());
        assert_eq!(mgr.build_context(), None);
    }

    #[test]
    fn context_includes_domain_when_detected() {
        let mut mgr = ContextManager::new();
        // Two medical keywords trigger the Medical domain
        mgr.push_sentence("ผู้ป่วยมีความดันสูง วินิจฉัยโดยแพทย์".to_string());
        let ctx = mgr.build_context().unwrap();
        assert!(ctx.contains("Domain: Medical"), "context: {}", ctx);
    }

    #[test]
    fn context_omits_domain_when_not_detected() {
        let mut mgr = ContextManager::new();
        mgr.push_sentence("สวัสดีครับ ขอบคุณมาก".to_string());
        let ctx = mgr.build_context().unwrap();
        assert!(!ctx.contains("Domain:"), "no domain expected; context: {}", ctx);
    }
}
