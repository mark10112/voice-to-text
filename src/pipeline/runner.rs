//! Pipeline orchestrator — drives the full audio → STT → LLM → inject loop.
//!
//! [`PipelineOrchestrator`] owns the [`SharedState`] and responds to
//! [`HotkeyEvent`]s received over a `tokio::sync::mpsc` channel.
//!
//! # Pipeline flow
//!
//! ```text
//! HotkeyEvent::PushToTalkPressed
//!   └─▶ clear audio buffer, set state = Recording
//!
//! HotkeyEvent::PushToTalkReleased
//!   └─▶ drain buffer → spawn_blocking(stt.transcribe)   [Transcribing]
//!         └─▶ Fast mode  → spawn_blocking(inject_text)  [Result]
//!         └─▶ Std/Ctx    → llm.correct (async)          [Correcting]
//!               ├─ Ok  → push context, spawn_blocking(inject_text) [Result]
//!               └─ Err → warn + spawn_blocking(inject raw text)    [Result]
//! ```
//!
//! All blocking work (Whisper inference, clipboard I/O) is pushed onto
//! `tokio::task::spawn_blocking` so the async runtime never stalls.

use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

use crate::audio::RingBuffer;
use crate::config::OperatingMode;
use crate::hotkey::HotkeyEvent;
use crate::inject::inject_text;
use crate::llm::{ContextManager, LlmCorrector};
use crate::stt::SttEngine;

use super::state::{PipelineState, SharedState};

// ---------------------------------------------------------------------------
// PipelineError
// ---------------------------------------------------------------------------

/// Errors that can surface inside the pipeline.
///
/// All variants carry a human-readable description so the UI can display them
/// without knowing the internal cause.
#[derive(Debug)]
pub enum PipelineError {
    /// Audio buffer was empty when transcription was attempted.
    EmptyAudio,
    /// STT engine failed or returned an error.
    Stt(String),
    /// Text injection failed.
    Inject(String),
    /// Internal / unexpected error (e.g. tokio join failure).
    Internal(String),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::EmptyAudio => write!(f, "No audio captured — hold the hotkey longer"),
            PipelineError::Stt(msg) => write!(f, "Transcription failed: {msg}"),
            PipelineError::Inject(msg) => write!(f, "Text injection failed: {msg}"),
            PipelineError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

// ---------------------------------------------------------------------------
// SharedAudioBuffer
// ---------------------------------------------------------------------------

/// Thread-safe audio ring buffer shared between the cpal callback and the
/// pipeline orchestrator.
///
/// The orchestrator drains it on `PushToTalkReleased`; the cpal callback
/// pushes chunks on every callback invocation while recording is active.
pub type SharedAudioBuffer = Arc<Mutex<RingBuffer<f32>>>;

// ---------------------------------------------------------------------------
// PipelineOrchestrator
// ---------------------------------------------------------------------------

/// Drives the complete voice-to-text pipeline.
///
/// Create with [`PipelineOrchestrator::new`], then call [`run`](Self::run)
/// inside a tokio task.
///
/// ```rust,no_run
/// use std::sync::{Arc, Mutex};
/// use voice_to_text::audio::RingBuffer;
/// use voice_to_text::config::AppConfig;
/// use voice_to_text::pipeline::{new_shared_state, PipelineOrchestrator};
///
/// // (stt and llm are Arc<dyn …> created elsewhere)
/// # async fn example() {
/// # use voice_to_text::stt::SttEngine;
/// # use voice_to_text::llm::LlmCorrector;
/// # fn make_stt() -> Arc<dyn SttEngine> { unimplemented!() }
/// # fn make_llm() -> Arc<dyn LlmCorrector> { unimplemented!() }
/// let config = AppConfig::default();
/// let shared_state = new_shared_state(config.clone());
/// let audio_buf = Arc::new(Mutex::new(RingBuffer::new(16_000 * 60)));
///
/// let (hotkey_tx, hotkey_rx) = tokio::sync::mpsc::channel(16);
/// let orchestrator = PipelineOrchestrator::new(
///     shared_state,
///     audio_buf,
///     make_stt(),
///     make_llm(),
/// );
/// orchestrator.run(hotkey_rx).await;
/// # }
/// ```
pub struct PipelineOrchestrator {
    state: SharedState,
    audio_buf: SharedAudioBuffer,
    stt: Arc<dyn SttEngine>,
    llm: Arc<dyn LlmCorrector>,
    context: ContextManager,
}

impl PipelineOrchestrator {
    /// Create a new orchestrator.
    ///
    /// # Arguments
    ///
    /// * `state`     — shared application state (also read by the UI).
    /// * `audio_buf` — ring buffer filled by the cpal audio callback.
    /// * `stt`       — STT engine (e.g. `WhisperEngine`).
    /// * `llm`       — LLM corrector (e.g. `ApiCorrector` or `FallbackCorrector`).
    pub fn new(
        state: SharedState,
        audio_buf: SharedAudioBuffer,
        stt: Arc<dyn SttEngine>,
        llm: Arc<dyn LlmCorrector>,
    ) -> Self {
        Self {
            state,
            audio_buf,
            stt,
            llm,
            context: ContextManager::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Main async loop
    // -----------------------------------------------------------------------

    /// Run the orchestrator until `hotkey_rx` is closed.
    ///
    /// This is an `async fn` and should be spawned as a tokio task from
    /// `main()`.  It never returns while the channel is open.
    pub async fn run(mut self, mut hotkey_rx: mpsc::Receiver<HotkeyEvent>) {
        while let Some(event) = hotkey_rx.recv().await {
            match event {
                HotkeyEvent::PushToTalkPressed => {
                    self.handle_pressed();
                }
                HotkeyEvent::PushToTalkReleased => {
                    self.handle_released().await;
                }
                HotkeyEvent::ToggleVisibility => {
                    // Visibility toggle is handled by the UI layer — ignore.
                }
            }
        }

        log::info!("pipeline: hotkey channel closed, orchestrator shutting down");
    }

    // -----------------------------------------------------------------------
    // Event handlers
    // -----------------------------------------------------------------------

    /// Handle push-to-talk press: clear the audio buffer and enter Recording.
    fn handle_pressed(&mut self) {
        log::debug!("pipeline: PushToTalkPressed → Recording");

        // Clear any leftover audio from a previous session.
        if let Ok(mut buf) = self.audio_buf.lock() {
            buf.clear();
        }

        // Reset recording timer.
        let mut st = self.state.lock().unwrap();
        st.pipeline = PipelineState::Recording;
        st.recording_secs = 0.0;
        st.error_message = None;
    }

    /// Handle push-to-talk release: drain audio → STT → (LLM) → inject.
    async fn handle_released(&mut self) {
        log::debug!("pipeline: PushToTalkReleased → draining audio");

        // ── 1. Drain audio buffer ────────────────────────────────────────
        let audio: Vec<f32> = {
            match self.audio_buf.lock() {
                Ok(mut buf) => {
                    let samples = buf.drain();
                    samples
                }
                Err(e) => {
                    self.set_error(format!("audio buffer lock poisoned: {e}"));
                    return;
                }
            }
        };

        if audio.is_empty() {
            log::warn!("pipeline: audio buffer was empty after release");
            self.set_error(PipelineError::EmptyAudio.to_string());
            return;
        }

        let recording_secs = audio.len() as f32 / 16_000.0;
        {
            let mut st = self.state.lock().unwrap();
            st.recording_secs = recording_secs;
        }

        // ── 2. STT transcription (blocking → thread pool) ────────────────
        self.set_pipeline(PipelineState::Transcribing);

        let stt = Arc::clone(&self.stt);
        let audio_clone = audio.clone();

        let stt_result = tokio::task::spawn_blocking(move || stt.transcribe(&audio_clone)).await;

        let raw_text = match stt_result {
            Ok(Ok(text)) => text,
            Ok(Err(e)) => {
                self.set_error(PipelineError::Stt(e.to_string()).to_string());
                return;
            }
            Err(e) => {
                self.set_error(PipelineError::Internal(e.to_string()).to_string());
                return;
            }
        };

        log::debug!("pipeline: STT result = {:?}", raw_text);

        // Store raw text so the UI can show it while the LLM runs.
        {
            let mut st = self.state.lock().unwrap();
            st.raw_text = Some(raw_text.clone());
        }

        // ── 3. Operating mode check ──────────────────────────────────────
        let mode = {
            let st = self.state.lock().unwrap();
            st.config.operating_mode
        };

        let final_text = if mode == OperatingMode::Fast {
            // Fast mode: skip LLM entirely.
            log::debug!("pipeline: Fast mode — skipping LLM");
            raw_text.clone()
        } else {
            // Standard / Context mode: call LLM corrector.
            self.set_pipeline(PipelineState::Correcting);

            let context = self.context.build_context();
            let context_ref = context.as_deref();

            match self.llm.correct(&raw_text, context_ref).await {
                Ok(corrected) => {
                    log::debug!("pipeline: LLM corrected = {:?}", corrected);
                    // Push corrected sentence into rolling context window.
                    self.context.push_sentence(corrected.clone());
                    corrected
                }
                Err(e) => {
                    log::warn!("pipeline: LLM failed ({e}), falling back to raw STT text");
                    // Graceful fallback — inject raw text, do NOT crash.
                    raw_text.clone()
                }
            }
        };

        // ── 4. Text injection (blocking → thread pool) ───────────────────
        let text_to_inject = final_text.clone();
        let inject_result =
            tokio::task::spawn_blocking(move || inject_text(&text_to_inject)).await;

        match inject_result {
            Ok(Ok(())) => {
                log::debug!("pipeline: injection succeeded");
            }
            Ok(Err(e)) => {
                // Injection failure is non-fatal — we still show the text.
                log::warn!("pipeline: injection failed: {e}");
            }
            Err(e) => {
                log::warn!("pipeline: inject task panicked: {e}");
            }
        }

        // ── 5. Finalise state ────────────────────────────────────────────
        {
            let mut st = self.state.lock().unwrap();
            st.pipeline = PipelineState::Result;
            st.last_text = Some(final_text);
            st.raw_text = None;
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn set_pipeline(&self, state: PipelineState) {
        let mut st = self.state.lock().unwrap();
        st.pipeline = state;
    }

    fn set_error(&self, message: String) {
        let mut st = self.state.lock().unwrap();
        st.pipeline = PipelineState::Error;
        st.error_message = Some(message.clone());
        log::error!("pipeline error: {message}");
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::hotkey::HotkeyEvent;
    use crate::pipeline::state::new_shared_state;
    use async_trait::async_trait;

    // -----------------------------------------------------------------------
    // Test doubles
    // -----------------------------------------------------------------------

    /// Mock LLM corrector that always succeeds with a fixed string.
    struct OkLlm(String);

    #[async_trait]
    impl LlmCorrector for OkLlm {
        async fn correct(
            &self,
            _raw: &str,
            _ctx: Option<&str>,
        ) -> Result<String, crate::llm::LlmError> {
            Ok(self.0.clone())
        }
    }

    /// Mock LLM corrector that always fails.
    struct FailLlm;

    #[async_trait]
    impl LlmCorrector for FailLlm {
        async fn correct(
            &self,
            raw: &str,
            _ctx: Option<&str>,
        ) -> Result<String, crate::llm::LlmError> {
            // We return an error; the orchestrator must fall back to raw text.
            let _ = raw;
            Err(crate::llm::LlmError::Timeout)
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// 1 second of silence at 16 kHz — passes the AudioTooShort check.
    fn one_second_of_silence() -> Vec<f32> {
        vec![0.0f32; 16_000]
    }

    fn make_audio_buf(samples: &[f32]) -> SharedAudioBuffer {
        let buf = Arc::new(Mutex::new(RingBuffer::new(16_000 * 60)));
        buf.lock().unwrap().push_slice(samples);
        buf
    }

    fn make_orchestrator(
        config: AppConfig,
        samples: &[f32],
        llm: Arc<dyn LlmCorrector>,
    ) -> (PipelineOrchestrator, SharedState) {
        use crate::stt::MockSttEngine;

        let state = new_shared_state(config);
        let audio_buf = make_audio_buf(samples);
        let stt: Arc<dyn SttEngine> = Arc::new(MockSttEngine::ok("สวัสดี"));

        let orc = PipelineOrchestrator::new(Arc::clone(&state), audio_buf, stt, llm);
        (orc, state)
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    /// `PushToTalkPressed` should move the pipeline to `Recording`.
    #[tokio::test]
    async fn pressed_sets_recording_state() {
        let (tx, rx) = mpsc::channel(4);
        let config = AppConfig::default();
        let (orc, state) = make_orchestrator(
            config,
            &one_second_of_silence(),
            Arc::new(OkLlm("fixed".into())),
        );

        tx.send(HotkeyEvent::PushToTalkPressed).await.unwrap();
        drop(tx); // close channel so run() returns

        orc.run(rx).await;

        // After channel is closed the last state was Recording (no release sent).
        let st = state.lock().unwrap();
        assert_eq!(st.pipeline, PipelineState::Recording);
    }

    /// Full press + release in Fast mode should reach `Result` state.
    #[tokio::test]
    async fn fast_mode_reaches_result_state() {
        let (tx, rx) = mpsc::channel(4);
        let mut config = AppConfig::default();
        config.operating_mode = OperatingMode::Fast;

        let (orc, state) = make_orchestrator(
            config,
            &one_second_of_silence(),
            Arc::new(OkLlm("should not be called".into())),
        );

        tx.send(HotkeyEvent::PushToTalkPressed).await.unwrap();
        tx.send(HotkeyEvent::PushToTalkReleased).await.unwrap();
        drop(tx);

        orc.run(rx).await;

        let st = state.lock().unwrap();
        assert_eq!(st.pipeline, PipelineState::Result);
        // Fast mode should inject the raw STT text directly.
        assert_eq!(st.last_text.as_deref(), Some("สวัสดี"));
    }

    /// Standard mode with a successful LLM should inject the corrected text.
    #[tokio::test]
    async fn standard_mode_llm_ok_injects_corrected_text() {
        let (tx, rx) = mpsc::channel(4);
        let mut config = AppConfig::default();
        config.operating_mode = OperatingMode::Standard;

        let (orc, state) = make_orchestrator(
            config,
            &one_second_of_silence(),
            Arc::new(OkLlm("แก้ไขแล้ว".into())),
        );

        tx.send(HotkeyEvent::PushToTalkPressed).await.unwrap();
        tx.send(HotkeyEvent::PushToTalkReleased).await.unwrap();
        drop(tx);

        orc.run(rx).await;

        let st = state.lock().unwrap();
        assert_eq!(st.pipeline, PipelineState::Result);
        assert_eq!(st.last_text.as_deref(), Some("แก้ไขแล้ว"));
    }

    /// When LLM fails, the orchestrator must fall back to the raw STT text and
    /// NOT crash or enter `Error` state.
    #[tokio::test]
    async fn llm_failure_falls_back_to_raw_text() {
        let (tx, rx) = mpsc::channel(4);
        let mut config = AppConfig::default();
        config.operating_mode = OperatingMode::Standard;

        let (orc, state) = make_orchestrator(
            config,
            &one_second_of_silence(),
            Arc::new(FailLlm),
        );

        tx.send(HotkeyEvent::PushToTalkPressed).await.unwrap();
        tx.send(HotkeyEvent::PushToTalkReleased).await.unwrap();
        drop(tx);

        orc.run(rx).await;

        let st = state.lock().unwrap();
        // Must reach Result (not Error) with the raw STT text.
        assert_eq!(st.pipeline, PipelineState::Result);
        assert_eq!(st.last_text.as_deref(), Some("สวัสดี"));
    }

    /// When audio buffer is empty on release, the pipeline should enter `Error`.
    #[tokio::test]
    async fn empty_audio_on_release_sets_error_state() {
        let (tx, rx) = mpsc::channel(4);
        let config = AppConfig::default();

        // Build orchestrator with empty audio buffer.
        let state = new_shared_state(config);
        let audio_buf = Arc::new(Mutex::new(RingBuffer::new(16_000 * 60)));
        // Do NOT push any samples — buffer stays empty.

        use crate::stt::MockSttEngine;
        let stt: Arc<dyn SttEngine> = Arc::new(MockSttEngine::ok("text"));
        let llm: Arc<dyn LlmCorrector> = Arc::new(OkLlm("fixed".into()));

        let orc = PipelineOrchestrator::new(Arc::clone(&state), audio_buf, stt, llm);

        tx.send(HotkeyEvent::PushToTalkPressed).await.unwrap();
        tx.send(HotkeyEvent::PushToTalkReleased).await.unwrap();
        drop(tx);

        orc.run(rx).await;

        let st = state.lock().unwrap();
        assert_eq!(st.pipeline, PipelineState::Error);
        assert!(st.error_message.is_some());
    }

    /// `ToggleVisibility` events must be silently ignored without panicking.
    #[tokio::test]
    async fn toggle_visibility_is_ignored() {
        let (tx, rx) = mpsc::channel(4);
        let config = AppConfig::default();
        let (orc, state) = make_orchestrator(
            config,
            &one_second_of_silence(),
            Arc::new(OkLlm("ok".into())),
        );

        tx.send(HotkeyEvent::ToggleVisibility).await.unwrap();
        drop(tx);

        orc.run(rx).await;

        // State should remain Idle — only ToggleVisibility was sent.
        let st = state.lock().unwrap();
        assert_eq!(st.pipeline, PipelineState::Idle);
    }
}
