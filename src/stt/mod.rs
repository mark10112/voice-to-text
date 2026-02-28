//! STT (Speech-to-Text) engine module.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │                  SttEngine (trait)                    │
//! │                                                      │
//! │   ┌─────────────┐    ┌──────────────┐               │
//! │   │  ModelPaths  │    │ WhisperEngine│               │
//! │   │ - resolve    │───▶│ - ctx        │               │
//! │   │ - exists?    │    │ - params     │               │
//! │   └─────────────┘    └──────┬───────┘               │
//! │                              │                       │
//! │                              ▼                       │
//! │                    ┌──────────────────┐              │
//! │                    │  transcribe()    │              │
//! │                    │  audio → text    │              │
//! │                    └──────────────────┘              │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! # Quick start
//!
//! ```rust,no_run
//! use voice_to_text::stt::{WhisperEngine, TranscribeParams, SttEngine};
//!
//! let params = TranscribeParams::default(); // language = "th", Greedy { best_of: 1 }
//! let engine = WhisperEngine::load("models/ggml-thonburian-medium.bin", params)
//!     .expect("model not found — run the setup wizard first");
//!
//! // audio: 16 kHz, mono, f32 PCM from the audio module
//! let audio: Vec<f32> = vec![0.0; 16_000]; // 1 s of silence
//! let text = engine.transcribe(&audio).unwrap();
//! println!("{text}");
//! ```

pub mod engine;
pub mod model;
pub mod transcribe;

// ── Public re-exports ──────────────────────────────────────────────────────

pub use engine::{SttEngine, SttError, WhisperEngine};
pub use model::{
    find_model_by_id, models_for_language, ModelInfo, ModelPaths, ModelSize, THAI_MODELS,
    WHISPER_MODELS,
};
pub use transcribe::{SamplingStrategy, Segment, TranscribeParams, TranscriptionResult};

// test-only re-export so the pipeline test module can import MockSttEngine
// without `use voice_to_text::stt::engine::MockSttEngine`.
#[cfg(test)]
pub use engine::MockSttEngine;
