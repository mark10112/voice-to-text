//! Simple energy-based Voice Activity Detection (VAD).
//!
//! [`VadDetector`] trims leading and trailing silence from a 16 kHz mono
//! audio clip before it is sent to the STT engine.  Removing silence:
//!
//! * reduces Whisper processing time, and
//! * prevents Whisper from hallucinating text during quiet periods.
//!
//! ## Algorithm
//!
//! Audio is split into 30 ms frames (480 samples @ 16 kHz).  A frame is
//! classified as *voice* when its RMS amplitude exceeds the configured
//! threshold.  The output slice is trimmed to the first and last voice frame.
//!
//! ## Phase 2 upgrade
//!
//! Replace this module with Silero VAD (ONNX) or whisper-rs built-in VAD
//! for language-aware, more accurate voice detection.

// ---------------------------------------------------------------------------
// VadDetector
// ---------------------------------------------------------------------------

/// Energy-based silence trimmer.
///
/// # Example
///
/// ```rust
/// use voice_to_text::audio::VadDetector;
///
/// // 0.01 RMS threshold — typical for a quiet room
/// let vad = VadDetector::new(0.01);
///
/// // Build a signal: 480 silent samples, then 480 loud samples, then 480 silent
/// let mut audio = vec![0.0_f32; 480];
/// audio.extend(vec![0.5_f32; 480]);
/// audio.extend(vec![0.0_f32; 480]);
///
/// let trimmed = vad.trim_silence(&audio);
/// assert_eq!(trimmed.len(), 480); // only the loud middle section
/// ```
pub struct VadDetector {
    /// RMS amplitude threshold; frames below this are considered silence.
    rms_threshold: f32,
    /// Frame size in samples.  Default: 480 samples = 30 ms at 16 kHz.
    frame_size: usize,
}

impl VadDetector {
    /// Create a [`VadDetector`] with the given RMS threshold.
    ///
    /// `rms_threshold` should be in `[0.0, 1.0]`.  A typical value is
    /// `0.01` for quiet microphones; use `0.02`–`0.05` in noisy environments.
    pub fn new(rms_threshold: f32) -> Self {
        Self {
            rms_threshold,
            frame_size: 480, // 30 ms at 16 kHz
        }
    }

    /// Create a [`VadDetector`] with a custom frame size.
    ///
    /// Useful for sample rates other than 16 kHz.  Prefer [`VadDetector::new`]
    /// for standard 16 kHz audio.
    pub fn with_frame_size(rms_threshold: f32, frame_size: usize) -> Self {
        assert!(frame_size > 0, "frame_size must be > 0");
        Self {
            rms_threshold,
            frame_size,
        }
    }

    /// RMS threshold currently in use.
    pub fn threshold(&self) -> f32 {
        self.rms_threshold
    }

    /// Returns `true` when the frame contains voice activity.
    fn is_voice_frame(&self, chunk: &[f32]) -> bool {
        if chunk.is_empty() {
            return false;
        }
        let mean_sq: f32 = chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32;
        mean_sq.sqrt() > self.rms_threshold
    }

    /// Trim leading and trailing silence from `audio`.
    ///
    /// Returns a sub-slice of the original buffer — no allocation.
    /// If the entire signal is silent, a zero-length slice is returned.
    ///
    /// # Arguments
    ///
    /// * `audio` — 16 kHz mono `f32` samples (pre-processed with
    ///   [`crate::audio::resample_to_16k`] and
    ///   [`crate::audio::stereo_to_mono`]).
    pub fn trim_silence<'a>(&self, audio: &'a [f32]) -> &'a [f32] {
        if audio.is_empty() {
            return audio;
        }

        let frame_size = self.frame_size;
        let total_frames = (audio.len() + frame_size - 1) / frame_size;

        // Find the first voice frame (left trim boundary)
        let start_frame = match (0..total_frames).find(|&i| {
            let s = i * frame_size;
            let e = ((i + 1) * frame_size).min(audio.len());
            self.is_voice_frame(&audio[s..e])
        }) {
            Some(f) => f,
            None => return &audio[0..0], // entire signal is silence
        };

        // Find the last voice frame (right trim boundary)
        let end_frame = (0..total_frames)
            .rfind(|&i| {
                let s = i * frame_size;
                let e = ((i + 1) * frame_size).min(audio.len());
                self.is_voice_frame(&audio[s..e])
            })
            .unwrap_or(start_frame);

        let start = start_frame * frame_size;
        let end = ((end_frame + 1) * frame_size).min(audio.len());

        &audio[start..end]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signal(silent_pre: usize, voice: usize, silent_post: usize) -> Vec<f32> {
        let mut v = vec![0.0_f32; silent_pre];
        v.extend(vec![0.5_f32; voice]);
        v.extend(vec![0.0_f32; silent_post]);
        v
    }

    #[test]
    fn trims_leading_and_trailing_silence() {
        let audio = make_signal(480, 480, 480); // 30ms silence / 30ms voice / 30ms silence
        let vad = VadDetector::new(0.01);
        let trimmed = vad.trim_silence(&audio);
        assert_eq!(trimmed.len(), 480);
    }

    #[test]
    fn all_silence_returns_empty() {
        let audio = vec![0.0_f32; 1440]; // 90 ms of silence
        let vad = VadDetector::new(0.01);
        let trimmed = vad.trim_silence(&audio);
        assert_eq!(trimmed.len(), 0);
    }

    #[test]
    fn no_silence_returns_full_signal() {
        let audio = vec![0.5_f32; 960]; // 60 ms of voice
        let vad = VadDetector::new(0.01);
        let trimmed = vad.trim_silence(&audio);
        assert_eq!(trimmed.len(), audio.len());
    }

    #[test]
    fn empty_input_returns_empty() {
        let vad = VadDetector::new(0.01);
        let trimmed = vad.trim_silence(&[]);
        assert_eq!(trimmed.len(), 0);
    }

    #[test]
    fn threshold_getter() {
        let vad = VadDetector::new(0.05);
        assert!((vad.threshold() - 0.05).abs() < 1e-7);
    }

    #[test]
    fn custom_frame_size() {
        // Use 160-sample frames (10 ms @ 16 kHz)
        let vad = VadDetector::with_frame_size(0.01, 160);
        let audio = make_signal(160, 160, 160);
        let trimmed = vad.trim_silence(&audio);
        assert_eq!(trimmed.len(), 160);
    }

    #[test]
    #[should_panic(expected = "frame_size must be > 0")]
    fn zero_frame_size_panics() {
        VadDetector::with_frame_size(0.01, 0);
    }
}
