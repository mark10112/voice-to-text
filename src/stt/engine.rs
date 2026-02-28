//! Core STT engine trait and implementations.
//!
//! # Overview
//!
//! [`SttEngine`] is the public interface used by the pipeline.  It is
//! object-safe and `Send + Sync` so it can be held behind an `Arc<dyn SttEngine>`.
//!
//! [`WhisperEngine`] is the production implementation that wraps a
//! `whisper_rs::WhisperContext`.  Construct it with [`WhisperEngine::load`].
//!
//! [`MockSttEngine`] (available under `#[cfg(test)]`) is a zero-dependency stub
//! that returns a pre-configured response — useful for unit-testing the pipeline
//! without a real GGML model file.

use std::path::Path;

use thiserror::Error;
use whisper_rs::{FullParams, WhisperContext, WhisperContextParameters};

use crate::stt::transcribe::{SamplingStrategy, Segment, TranscribeParams, TranscriptionResult};

// ---------------------------------------------------------------------------
// SttError
// ---------------------------------------------------------------------------

/// All errors that can arise from the STT subsystem.
#[derive(Debug, Clone, Error)]
pub enum SttError {
    /// The GGML model file was not found at the given path.
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// `whisper_rs` failed to initialise a `WhisperContext` or `WhisperState`.
    #[error("Whisper context initialisation failed: {0}")]
    ContextInit(String),

    /// An error occurred during the inference pass.
    #[error("Transcription error: {0}")]
    Transcription(String),

    /// The supplied audio buffer is shorter than the minimum 0.5 s
    /// (8 000 samples at 16 kHz).
    #[error("Audio too short — minimum 0.5 s (8 000 samples at 16 kHz)")]
    AudioTooShort,

    /// The supplied audio buffer exceeds the maximum 60 s
    /// (960 000 samples at 16 kHz).
    #[error("Audio too long — maximum 60 s (960 000 samples at 16 kHz)")]
    AudioTooLong,
}

// ---------------------------------------------------------------------------
// SttEngine trait
// ---------------------------------------------------------------------------

/// Object-safe, thread-safe interface for speech-to-text engines.
///
/// Implementations must be `Send + Sync` so that they can be held behind an
/// `Arc<dyn SttEngine>` and called from any thread.
///
/// # Contract
///
/// - `audio` must be **16 kHz, mono, f32** PCM samples.
/// - Returns `Err(SttError::AudioTooShort)` when `audio.len() < 8_000`.
/// - Returns `Err(SttError::AudioTooLong)` when `audio.len() > 960_000`.
pub trait SttEngine: Send + Sync {
    /// Transcribe `audio` and return the text transcript.
    fn transcribe(&self, audio: &[f32]) -> Result<String, SttError>;
}

// Compile-time assertion: Box<dyn SttEngine> must be constructible.
const _: fn() = || {
    fn _assert_object_safe(_: Box<dyn SttEngine>) {}
};

// ---------------------------------------------------------------------------
// Audio length constants (16 kHz mono f32)
// ---------------------------------------------------------------------------

/// Minimum audio length: 0.5 s × 16 000 Hz = 8 000 samples.
const MIN_AUDIO_SAMPLES: usize = 8_000;
/// Maximum audio length: 60 s × 16 000 Hz = 960 000 samples.
const MAX_AUDIO_SAMPLES: usize = 960_000;

// ---------------------------------------------------------------------------
// WhisperEngine
// ---------------------------------------------------------------------------

/// Production STT engine that wraps a `whisper_rs::WhisperContext`.
///
/// A new `WhisperState` is created for every [`transcribe`] call so the
/// engine can be shared across threads without any locking.
///
/// [`transcribe`]: WhisperEngine::transcribe
pub struct WhisperEngine {
    ctx: WhisperContext,
    params: TranscribeParams,
}

impl std::fmt::Debug for WhisperEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WhisperEngine")
            .field("params", &self.params)
            .finish_non_exhaustive()
    }
}

// `WhisperContext` holds a raw pointer internally but declares
// `unsafe impl Send` and `unsafe impl Sync` in whisper-rs — the model
// weights are read-only after loading.  `TranscribeParams` is fully owned
// and trivially Send+Sync.
// SAFETY: WhisperContext is Send+Sync as declared by whisper-rs.
unsafe impl Send for WhisperEngine {}
unsafe impl Sync for WhisperEngine {}

impl WhisperEngine {
    /// Load a GGML model from `model_path` and prepare it for inference.
    ///
    /// # Errors
    ///
    /// - [`SttError::ModelNotFound`] — `model_path` does not exist.
    /// - [`SttError::ContextInit`]  — whisper-rs failed to load the file.
    pub fn load(
        model_path: impl AsRef<Path>,
        params: TranscribeParams,
    ) -> Result<Self, SttError> {
        let path = model_path.as_ref();

        if !path.exists() {
            return Err(SttError::ModelNotFound(path.display().to_string()));
        }

        let path_str = path.to_str().ok_or_else(|| {
            SttError::ModelNotFound(format!(
                "model path contains non-UTF-8 characters: {}",
                path.display()
            ))
        })?;

        let ctx_params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(path_str, ctx_params)
            .map_err(|e| SttError::ContextInit(e.to_string()))?;

        Ok(Self { ctx, params })
    }

    /// Transcribe `audio` and return a [`TranscriptionResult`] with
    /// per-segment timing information.
    ///
    /// Prefer [`SttEngine::transcribe`] when only the text is needed.
    pub fn transcribe_full(&self, audio: &[f32]) -> Result<TranscriptionResult, SttError> {
        // ── Audio length guards ───────────────────────────────────────────
        if audio.len() < MIN_AUDIO_SAMPLES {
            return Err(SttError::AudioTooShort);
        }
        if audio.len() > MAX_AUDIO_SAMPLES {
            return Err(SttError::AudioTooLong);
        }

        // ── Build FullParams ──────────────────────────────────────────────
        // Convert our SamplingStrategy → whisper-rs's SamplingStrategy.
        use whisper_rs::SamplingStrategy as WS;
        let ws = match self.params.strategy {
            SamplingStrategy::Greedy { best_of } => WS::Greedy { best_of },
            SamplingStrategy::BeamSearch { beam_size, patience } => {
                WS::BeamSearch { beam_size, patience }
            }
        };

        let mut fp = FullParams::new(ws);

        // set_language takes an Option<&str> whose lifetime is tied to fp.
        // Both `fp` and the borrow of `self.params.language` remain alive
        // until state.full() returns, so the borrow is valid.
        let lang: Option<&str> = if self.params.language == "auto" {
            None
        } else {
            Some(self.params.language.as_str())
        };
        fp.set_language(lang);
        fp.set_n_threads(self.params.n_threads);

        if self.params.suppress_progress {
            fp.set_print_progress(false);
            fp.set_print_realtime(false);
        }

        // ── Create per-call state and run inference ───────────────────────
        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| SttError::ContextInit(e.to_string()))?;

        let wall_start = std::time::Instant::now();

        state
            .full(fp, audio)
            .map_err(|e| SttError::Transcription(e.to_string()))?;

        // ── Collect segments ──────────────────────────────────────────────
        let n_segments = state
            .full_n_segments()
            .map_err(|e| SttError::Transcription(e.to_string()))?;

        let mut text = String::new();
        let mut segments: Vec<Segment> = Vec::with_capacity(n_segments as usize);

        for i in 0..n_segments {
            let seg_text = state
                .full_get_segment_text(i)
                .map_err(|e| SttError::Transcription(format!("segment {i}: {e}")))?;

            // Timestamps are in centiseconds → multiply by 10 for ms.
            let t0 = state.full_get_segment_t0(i).unwrap_or(0).max(0) as u64 * 10;
            let t1 = state.full_get_segment_t1(i).unwrap_or(0).max(0) as u64 * 10;

            text.push_str(&seg_text);
            segments.push(Segment {
                text: seg_text,
                start_ms: t0,
                end_ms: t1,
            });
        }

        Ok(TranscriptionResult {
            text: text.trim().to_string(),
            segments,
            duration_ms: wall_start.elapsed().as_millis(),
        })
    }
}

impl SttEngine for WhisperEngine {
    fn transcribe(&self, audio: &[f32]) -> Result<String, SttError> {
        self.transcribe_full(audio).map(|r| r.text)
    }
}

// ---------------------------------------------------------------------------
// MockSttEngine  (test-only)
// ---------------------------------------------------------------------------

/// A test double that returns a pre-configured response without loading any
/// model file.
///
/// # Example
///
/// ```rust
/// # use voice_to_text::stt::{SttEngine, MockSttEngine};
/// let engine = MockSttEngine::ok("สวัสดี");
/// let result = engine.transcribe(&vec![0.0f32; 8_000]);
/// assert_eq!(result.unwrap(), "สวัสดี");
/// ```
#[cfg(test)]
pub struct MockSttEngine {
    response: Result<String, SttError>,
}

#[cfg(test)]
impl MockSttEngine {
    /// Create a mock that always returns `Ok(text)`.
    pub fn ok(text: impl Into<String>) -> Self {
        Self {
            response: Ok(text.into()),
        }
    }

    /// Create a mock that always returns `Err(error)`.
    pub fn err(error: SttError) -> Self {
        Self {
            response: Err(error),
        }
    }
}

#[cfg(test)]
impl SttEngine for MockSttEngine {
    fn transcribe(&self, audio: &[f32]) -> Result<String, SttError> {
        // Enforce the audio-length contract even in the mock so that callers
        // are tested against it.
        if audio.len() < MIN_AUDIO_SAMPLES {
            return Err(SttError::AudioTooShort);
        }
        if audio.len() > MAX_AUDIO_SAMPLES {
            return Err(SttError::AudioTooLong);
        }
        self.response.clone()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::stt::transcribe::optimal_threads;
    use super::*;

    // --- MockSttEngine ---

    #[test]
    fn mock_ok_returns_configured_text() {
        let engine = MockSttEngine::ok("สวัสดีครับ");
        let audio = vec![0.0f32; MIN_AUDIO_SAMPLES];
        assert_eq!(engine.transcribe(&audio).unwrap(), "สวัสดีครับ");
    }

    #[test]
    fn mock_err_returns_configured_error() {
        let engine = MockSttEngine::err(SttError::Transcription("boom".into()));
        let audio = vec![0.0f32; MIN_AUDIO_SAMPLES];
        let err = engine.transcribe(&audio).unwrap_err();
        assert!(matches!(err, SttError::Transcription(_)));
    }

    #[test]
    fn mock_short_audio_returns_audio_too_short() {
        let engine = MockSttEngine::ok("text");
        let short = vec![0.0f32; MIN_AUDIO_SAMPLES - 1];
        let err = engine.transcribe(&short).unwrap_err();
        assert!(matches!(err, SttError::AudioTooShort));
    }

    #[test]
    fn mock_long_audio_returns_audio_too_long() {
        let engine = MockSttEngine::ok("text");
        let long = vec![0.0f32; MAX_AUDIO_SAMPLES + 1];
        let err = engine.transcribe(&long).unwrap_err();
        assert!(matches!(err, SttError::AudioTooLong));
    }

    // --- WhisperEngine::load missing path ---

    #[test]
    fn load_missing_model_returns_model_not_found() {
        let params = TranscribeParams::default();
        let result = WhisperEngine::load("/nonexistent/model.bin", params);
        assert!(
            matches!(result, Err(SttError::ModelNotFound(_))),
            "expected ModelNotFound, got: {result:?}"
        );
    }

    // --- SttEngine object safety ---

    #[test]
    fn box_dyn_stt_engine_compiles() {
        // If this test compiles, the trait is object-safe.
        let engine: Box<dyn SttEngine> = Box::new(MockSttEngine::ok("ok"));
        let audio = vec![0.0f32; MIN_AUDIO_SAMPLES];
        let _ = engine.transcribe(&audio);
    }

    // --- AudioTooShort boundary ---

    #[test]
    fn exactly_min_audio_does_not_error_in_mock() {
        let engine = MockSttEngine::ok("ok");
        let audio = vec![0.0f32; MIN_AUDIO_SAMPLES];
        assert!(engine.transcribe(&audio).is_ok());
    }

    #[test]
    fn one_below_min_audio_errors_in_mock() {
        let engine = MockSttEngine::ok("ok");
        let audio = vec![0.0f32; MIN_AUDIO_SAMPLES - 1];
        assert!(matches!(
            engine.transcribe(&audio).unwrap_err(),
            SttError::AudioTooShort
        ));
    }

    // --- SttError display ---

    #[test]
    fn stt_error_display_model_not_found() {
        let e = SttError::ModelNotFound("/some/path.bin".into());
        assert!(e.to_string().contains("/some/path.bin"));
    }

    #[test]
    fn stt_error_display_audio_too_short() {
        let e = SttError::AudioTooShort;
        assert!(e.to_string().contains("short"));
    }

    // --- optimal_threads sanity check ---

    #[test]
    fn optimal_threads_is_positive_and_at_most_8() {
        let t = optimal_threads();
        assert!(t >= 1 && t <= 8);
    }
}
