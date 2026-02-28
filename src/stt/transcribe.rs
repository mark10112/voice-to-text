//! Transcription parameter types and result types.
//!
//! [`TranscribeParams`] carries all settings that control a single Whisper
//! inference run.  [`TranscriptionResult`] is returned by
//! [`WhisperEngine::transcribe_full`].

// ---------------------------------------------------------------------------
// SamplingStrategy
// ---------------------------------------------------------------------------

/// Mirrors `whisper_rs::SamplingStrategy` but is owned and `Clone`.
///
/// Use [`SamplingStrategy::Greedy`] for low-latency, single-pass decoding
/// (recommended for Thai real-time use).  [`SamplingStrategy::BeamSearch`]
/// gives slightly better accuracy at the cost of 2-4× higher latency.
#[derive(Debug, Clone, PartialEq)]
pub enum SamplingStrategy {
    /// Greedy (single-pass) decoding.
    Greedy {
        /// Number of candidate tokens evaluated per step.  1 is fastest.
        best_of: i32,
    },
    /// Beam-search decoding.
    BeamSearch {
        /// Number of beams to maintain in parallel.
        beam_size: i32,
        /// Beam-search patience factor (≥1.0 = standard beam search).
        patience: f32,
    },
}

impl Default for SamplingStrategy {
    fn default() -> Self {
        Self::Greedy { best_of: 1 }
    }
}

// ---------------------------------------------------------------------------
// TranscribeParams
// ---------------------------------------------------------------------------

/// All parameters for a single Whisper transcription run.
///
/// Build with [`TranscribeParams::default()`] for Thai and override fields as
/// needed:
///
/// ```
/// use voice_to_text::stt::TranscribeParams;
///
/// let params = TranscribeParams {
///     language: "en".into(),
///     ..TranscribeParams::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct TranscribeParams {
    /// ISO-639-1 language code (e.g. `"th"`, `"en"`), or `"auto"` to let
    /// Whisper detect the language automatically.
    pub language: String,

    /// Decoding strategy — Greedy is fastest, BeamSearch is more accurate.
    pub strategy: SamplingStrategy,

    /// Number of CPU threads handed to Whisper.  Defaults to
    /// [`optimal_threads()`], capped at 8.
    pub n_threads: i32,

    /// Suppress Whisper's progress output to stderr.
    pub suppress_progress: bool,
}

impl Default for TranscribeParams {
    fn default() -> Self {
        Self {
            language: "th".into(),
            strategy: SamplingStrategy::default(),
            n_threads: optimal_threads(),
            suppress_progress: true,
        }
    }
}

/// Returns the number of physical CPU threads to use for inference,
/// capped at 8 to avoid diminishing returns on Whisper.
pub(crate) fn optimal_threads() -> i32 {
    std::thread::available_parallelism()
        .map(|n| n.get().min(8) as i32)
        .unwrap_or(4)
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// The output of a successful transcription.
#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    /// Full concatenated transcript text (trimmed of leading/trailing
    /// whitespace).
    pub text: String,

    /// Individual time-aligned segments produced by Whisper.
    pub segments: Vec<Segment>,

    /// Wall-clock time the inference took, in milliseconds.
    pub duration_ms: u128,
}

/// A single time-aligned text chunk produced by Whisper.
#[derive(Debug, Clone)]
pub struct Segment {
    /// Segment text (may include punctuation inserted by Whisper).
    pub text: String,
    /// Segment start time in milliseconds from the start of the audio.
    pub start_ms: u64,
    /// Segment end time in milliseconds from the start of the audio.
    pub end_ms: u64,
}
