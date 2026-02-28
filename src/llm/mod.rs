//! LLM post-processing module for Thai Voice-to-Text.
//!
//! This module provides:
//! * [`LlmCorrector`] — async trait implemented by all corrector backends.
//! * [`ApiCorrector`] — OpenAI-compatible REST API corrector (MVP backend).
//! * [`FallbackCorrector`] — wraps any corrector; returns raw text on failure.
//! * [`PromptBuilder`] — builds Thai/English correction prompts.
//! * [`ContextManager`] — rolling window of previous sentences + domain hint.
//! * [`DomainDetector`] — keyword-based Thai domain detection.
//! * [`UserVocabulary`] / [`VocabEntry`] — custom correction entries.
//! * [`LlmError`] — error variants for LLM operations.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use voice_to_text::config::AppConfig;
//! use voice_to_text::llm::{ApiCorrector, ContextManager, FallbackCorrector, LlmCorrector};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = AppConfig::default();
//!
//!     // Build a corrector that never fails (falls back to raw text).
//!     let corrector = FallbackCorrector::new(ApiCorrector::from_config(&config.llm));
//!
//!     // Maintain rolling context between calls.
//!     let mut ctx_mgr = ContextManager::new();
//!
//!     let raw = "เอ่อ ผม ทำงาน เสร็จ แล้ว ครับ";
//!     let context = ctx_mgr.build_context();
//!     let corrected = corrector
//!         .correct(raw, context.as_deref())
//!         .await
//!         .unwrap();
//!
//!     ctx_mgr.push_sentence(corrected.clone());
//!     println!("{}", corrected);
//! }
//! ```

pub mod context;
pub mod corrector;
pub mod domain;
pub mod fallback;
pub mod prompt;
pub mod vocabulary;

// ---------------------------------------------------------------------------
// Public re-exports
// ---------------------------------------------------------------------------

pub use context::ContextManager;
pub use corrector::{ApiCorrector, LlmCorrector, LlmError};
pub use domain::DomainDetector;
pub use fallback::FallbackCorrector;
pub use prompt::PromptBuilder;
pub use vocabulary::{UserVocabulary, VocabEntry};
