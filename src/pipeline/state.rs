//! Pipeline state machine and shared application state.
//!
//! [`PipelineState`] drives the orchestrator's state machine.  The UI reads
//! it via [`SharedState`] to render the appropriate widget view.
//!
//! [`AppState`] is the single source of truth for everything the UI needs:
//! current pipeline phase, last injected text, waveform data, config snapshot,
//! and any error message.
//!
//! [`SharedState`] is a type alias for `Arc<Mutex<AppState>>` — cheap to clone
//! and safe to share across threads.

use std::sync::{Arc, Mutex};

use crate::config::AppConfig;

// ---------------------------------------------------------------------------
// PipelineState
// ---------------------------------------------------------------------------

/// States of the voice-to-text pipeline.
///
/// The state machine transitions are:
///
/// ```text
/// Idle ──hotkey press──▶ Recording
///      ──hotkey release─▶ Transcribing
///                         ──STT done──▶ Correcting  (Standard / Context mode)
///                                       ──LLM done──▶ Result
///                         ──STT done──▶ Result       (Fast mode)
/// any state ──error──▶ Error
/// Error / Result ──next cycle──▶ Idle
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineState {
    /// Waiting for the user to press the push-to-talk hotkey.
    Idle,

    /// Microphone is active; audio is being captured into the ring buffer.
    Recording,

    /// Audio has been drained; Whisper is running on the blocking thread pool.
    Transcribing,

    /// STT is complete; the LLM corrector is running (Standard / Context mode).
    Correcting,

    /// The final text is ready and has been injected into the active window.
    Result,

    /// A recoverable error occurred.  The pipeline will return to `Idle` on
    /// the next hotkey press.
    Error,
}

impl PipelineState {
    /// Returns `true` while the pipeline is actively processing audio or text.
    ///
    /// The UI uses this to disable the push-to-talk button while busy.
    ///
    /// ```
    /// use voice_to_text::pipeline::PipelineState;
    ///
    /// assert!(!PipelineState::Idle.is_busy());
    /// assert!(PipelineState::Recording.is_busy());
    /// assert!(PipelineState::Transcribing.is_busy());
    /// assert!(PipelineState::Correcting.is_busy());
    /// assert!(!PipelineState::Result.is_busy());
    /// assert!(!PipelineState::Error.is_busy());
    /// ```
    pub fn is_busy(&self) -> bool {
        matches!(
            self,
            PipelineState::Recording | PipelineState::Transcribing | PipelineState::Correcting
        )
    }

    /// A short human-readable label suitable for display in the UI status bar.
    pub fn label(&self) -> &'static str {
        match self {
            PipelineState::Idle => "Idle",
            PipelineState::Recording => "Recording",
            PipelineState::Transcribing => "Transcribing",
            PipelineState::Correcting => "Correcting",
            PipelineState::Result => "Done",
            PipelineState::Error => "Error",
        }
    }
}

impl Default for PipelineState {
    fn default() -> Self {
        PipelineState::Idle
    }
}

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

/// Shared application state — the single source of truth for the UI.
///
/// Held behind [`SharedState`] (`Arc<Mutex<AppState>>`).  The pipeline
/// orchestrator mutates it; the egui update loop reads it each frame.
pub struct AppState {
    /// Current phase of the voice-to-text pipeline.
    pub pipeline: PipelineState,

    /// The most-recently injected (or corrected) text.
    ///
    /// `None` until at least one transcription has completed.
    pub last_text: Option<String>,

    /// Raw STT output shown in the UI while LLM correction is running.
    ///
    /// `None` when in Fast mode or before STT completes.
    pub raw_text: Option<String>,

    /// Waveform amplitude data for the live recording visualisation.
    ///
    /// Updated by the audio callback; displayed as a bar graph in the widget.
    pub waveform: Vec<f32>,

    /// Current application configuration.
    ///
    /// The pipeline reads `operating_mode` to decide whether to call the LLM.
    pub config: AppConfig,

    /// Error message to display when `pipeline == PipelineState::Error`.
    pub error_message: Option<String>,

    /// Duration of the current recording in seconds.
    ///
    /// Reset to `0.0` when a new recording starts; updated in real time by
    /// the audio accumulation loop.
    pub recording_secs: f32,
}

impl AppState {
    /// Create a new `AppState` with sensible defaults.
    pub fn new(config: AppConfig) -> Self {
        Self {
            pipeline: PipelineState::Idle,
            last_text: None,
            raw_text: None,
            waveform: Vec::new(),
            config,
            error_message: None,
            recording_secs: 0.0,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(AppConfig::default())
    }
}

// ---------------------------------------------------------------------------
// SharedState
// ---------------------------------------------------------------------------

/// Thread-safe handle to [`AppState`].
///
/// Cheap to clone (`Arc` clone).  Lock with `.lock().unwrap()` for a short
/// critical section; do **not** hold the lock across `.await` points.
pub type SharedState = Arc<Mutex<AppState>>;

/// Construct a new [`SharedState`] wrapping a default [`AppState`].
pub fn new_shared_state(config: AppConfig) -> SharedState {
    Arc::new(Mutex::new(AppState::new(config)))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- PipelineState::is_busy ---

    #[test]
    fn idle_is_not_busy() {
        assert!(!PipelineState::Idle.is_busy());
    }

    #[test]
    fn recording_is_busy() {
        assert!(PipelineState::Recording.is_busy());
    }

    #[test]
    fn transcribing_is_busy() {
        assert!(PipelineState::Transcribing.is_busy());
    }

    #[test]
    fn correcting_is_busy() {
        assert!(PipelineState::Correcting.is_busy());
    }

    #[test]
    fn result_is_not_busy() {
        assert!(!PipelineState::Result.is_busy());
    }

    #[test]
    fn error_is_not_busy() {
        assert!(!PipelineState::Error.is_busy());
    }

    // ---- PipelineState::label ---

    #[test]
    fn label_idle() {
        assert_eq!(PipelineState::Idle.label(), "Idle");
    }

    #[test]
    fn label_recording() {
        assert_eq!(PipelineState::Recording.label(), "Recording");
    }

    #[test]
    fn label_transcribing() {
        assert_eq!(PipelineState::Transcribing.label(), "Transcribing");
    }

    #[test]
    fn label_correcting() {
        assert_eq!(PipelineState::Correcting.label(), "Correcting");
    }

    #[test]
    fn label_result() {
        assert_eq!(PipelineState::Result.label(), "Done");
    }

    #[test]
    fn label_error() {
        assert_eq!(PipelineState::Error.label(), "Error");
    }

    // ---- Default ---

    #[test]
    fn default_pipeline_state_is_idle() {
        assert_eq!(PipelineState::default(), PipelineState::Idle);
    }

    // ---- AppState / SharedState ---

    #[test]
    fn app_state_default_pipeline_is_idle() {
        let state = AppState::default();
        assert_eq!(state.pipeline, PipelineState::Idle);
        assert!(state.last_text.is_none());
        assert!(state.error_message.is_none());
        assert!((state.recording_secs - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn shared_state_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SharedState>();
    }

    #[test]
    fn shared_state_can_be_cloned_and_mutated() {
        let state = new_shared_state(AppConfig::default());
        let state2 = Arc::clone(&state);

        state.lock().unwrap().pipeline = PipelineState::Recording;
        assert_eq!(state2.lock().unwrap().pipeline, PipelineState::Recording);
    }
}
