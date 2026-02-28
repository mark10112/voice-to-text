//! Pipeline orchestrator module for Thai Voice-to-Text.
//!
//! This module wires the full audio → STT → LLM → text-injection pipeline and
//! exposes the shared state that the UI reads every frame.
//!
//! # Architecture
//!
//! ```text
//! HotkeyEvent (mpsc)
//!        │
//!        ▼
//! PipelineOrchestrator::run()  ← async tokio task
//!        │
//!        ├─ PushToTalkPressed  → clear RingBuffer, set Recording
//!        │
//!        └─ PushToTalkReleased
//!              │
//!              ├─ drain RingBuffer
//!              ├─ spawn_blocking(SttEngine::transcribe)   → Transcribing
//!              ├─ [Standard/Context] LlmCorrector::correct → Correcting
//!              └─ spawn_blocking(inject_text)              → Result
//!
//! SharedState (Arc<Mutex<AppState>>) ←─── read by egui update() each frame
//! ```
//!
//! # Quick start
//!
//! ```rust,no_run
//! use std::sync::{Arc, Mutex};
//! use tokio::sync::mpsc;
//! use voice_to_text::audio::RingBuffer;
//! use voice_to_text::config::AppConfig;
//! use voice_to_text::pipeline::{new_shared_state, PipelineOrchestrator};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = AppConfig::default();
//!     let shared_state = new_shared_state(config.clone());
//!     let audio_buf = Arc::new(Mutex::new(RingBuffer::<f32>::new(16_000 * 60)));
//!
//!     // (stt and llm constructed from config)
//!     # use voice_to_text::stt::SttEngine;
//!     # use voice_to_text::llm::LlmCorrector;
//!     # fn make_stt() -> Arc<dyn SttEngine> { unimplemented!() }
//!     # fn make_llm() -> Arc<dyn LlmCorrector> { unimplemented!() }
//!
//!     let (hotkey_tx, hotkey_rx) = mpsc::channel(16);
//!     let orchestrator = PipelineOrchestrator::new(
//!         shared_state.clone(),
//!         audio_buf,
//!         make_stt(),
//!         make_llm(),
//!     );
//!
//!     tokio::spawn(async move { orchestrator.run(hotkey_rx).await });
//!
//!     // hotkey_tx is passed to HotkeyListener::start(...)
//! }
//! ```

pub mod runner;
pub mod state;

// ---------------------------------------------------------------------------
// Public re-exports
// ---------------------------------------------------------------------------

pub use runner::{PipelineError, PipelineOrchestrator, SharedAudioBuffer};
pub use state::{new_shared_state, AppState, PipelineState, SharedState};
