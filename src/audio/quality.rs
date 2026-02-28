//! Pre-transcription audio quality validation.
//!
//! [`AudioQuality`] checks a 16 kHz mono `f32` clip against three criteria
//! before it is passed to the STT engine:
//!
//! | Check | Description |
//! |-------|-------------|
//! | Duration | Clip must be within `[min_secs, max_secs]` |
//! | Silence | At least one sample must exceed an amplitude threshold |
//! | Clipping | Fewer than `clipping_max_pct`% of samples may be clipped |
//!
//! # Example
//!
//! ```rust
//! use voice_to_text::audio::{AudioQuality, AudioError};
//!
//! let validator = AudioQuality::new(0.5, 60.0);
//!
//! // 8000 samples @ 16 kHz = 0.5 s (just at the minimum)
//! let audio = vec![0.1_f32; 8_000];
//! assert!(validator.validate(&audio).is_ok());
//!
//! // Too short
//! let short = vec![0.1_f32; 100];
//! assert!(matches!(validator.validate(&short), Err(AudioError::TooShort { .. })));
//! ```

use thiserror::Error;

// ---------------------------------------------------------------------------
// AudioError
// ---------------------------------------------------------------------------

/// Reason an audio clip failed quality validation.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum AudioError {
    /// Recording is shorter than the configured minimum.
    #[error(
        "recording too short: {got_secs:.2}s (minimum {min_secs:.2}s)"
    )]
    TooShort { min_secs: f32, got_secs: f32 },

    /// Recording is longer than the configured maximum.
    #[error(
        "recording too long: {got_secs:.2}s (maximum {max_secs:.2}s)"
    )]
    TooLong { max_secs: f32, got_secs: f32 },

    /// All samples are below the silence floor.
    #[error(
        "audio too quiet: max amplitude {amplitude:.4} (threshold {threshold:.4})"
    )]
    TooQuiet { amplitude: f32, threshold: f32 },

    /// Too many samples are clipped (at full scale).
    #[error(
        "audio clipping: {clipped_pct:.1}% of samples clipped (max {max_pct:.1}%)"
    )]
    Clipping { clipped_pct: f32, max_pct: f32 },
}

// ---------------------------------------------------------------------------
// AudioQuality
// ---------------------------------------------------------------------------

/// Validates an audio clip before STT transcription.
///
/// All thresholds can be adjusted; the `Default` implementation matches the
/// values recommended in the design document.
pub struct AudioQuality {
    /// Minimum allowed duration in seconds (default: `0.5`).
    pub min_recording_secs: f32,
    /// Maximum allowed duration in seconds (default: `60.0`).
    pub max_recording_secs: f32,
    /// Minimum peak amplitude for the clip to be considered non-silent
    /// (default: `0.01`).
    pub silence_threshold: f32,
    /// Amplitude above which a sample is considered clipped (default: `0.99`).
    pub clipping_threshold: f32,
    /// Maximum fraction of clipped samples (0.0–100.0 %) before the clip is
    /// rejected (default: `10.0` %).
    pub clipping_max_pct: f32,
}

impl Default for AudioQuality {
    fn default() -> Self {
        Self {
            min_recording_secs: 0.5,
            max_recording_secs: 60.0,
            silence_threshold: 0.01,
            clipping_threshold: 0.99,
            clipping_max_pct: 10.0,
        }
    }
}

impl AudioQuality {
    /// Create a validator with the given duration limits and default thresholds.
    pub fn new(min_secs: f32, max_secs: f32) -> Self {
        Self {
            min_recording_secs: min_secs,
            max_recording_secs: max_secs,
            ..Default::default()
        }
    }

    /// Validate `audio` (16 kHz mono `f32`).
    ///
    /// Returns `Ok(())` when all checks pass, or the first [`AudioError`]
    /// encountered otherwise.
    ///
    /// Checks are run in this order:
    /// 1. Duration (too short → too long)
    /// 2. Silence (too quiet)
    /// 3. Clipping
    pub fn validate(&self, audio: &[f32]) -> Result<(), AudioError> {
        const SAMPLE_RATE: f32 = 16_000.0;

        // 1. Duration checks
        let duration_secs = audio.len() as f32 / SAMPLE_RATE;

        if duration_secs < self.min_recording_secs {
            return Err(AudioError::TooShort {
                min_secs: self.min_recording_secs,
                got_secs: duration_secs,
            });
        }

        if duration_secs > self.max_recording_secs {
            return Err(AudioError::TooLong {
                max_secs: self.max_recording_secs,
                got_secs: duration_secs,
            });
        }

        // 2. Silence check
        let max_amplitude = audio
            .iter()
            .map(|s| s.abs())
            .fold(0.0_f32, f32::max);

        if max_amplitude < self.silence_threshold {
            return Err(AudioError::TooQuiet {
                amplitude: max_amplitude,
                threshold: self.silence_threshold,
            });
        }

        // 3. Clipping check
        if !audio.is_empty() {
            let clipped = audio
                .iter()
                .filter(|&&s| s.abs() > self.clipping_threshold)
                .count();
            let clipped_pct = clipped as f32 / audio.len() as f32 * 100.0;

            if clipped_pct > self.clipping_max_pct {
                return Err(AudioError::Clipping {
                    clipped_pct,
                    max_pct: self.clipping_max_pct,
                });
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_audio(secs: f32, amplitude: f32) -> Vec<f32> {
        let n = (secs * 16_000.0) as usize;
        vec![amplitude; n]
    }

    #[test]
    fn valid_audio_passes() {
        let validator = AudioQuality::default();
        let audio = make_audio(1.0, 0.3);
        assert!(validator.validate(&audio).is_ok());
    }

    #[test]
    fn too_short_rejected() {
        let validator = AudioQuality::new(0.5, 60.0);
        let audio = make_audio(0.1, 0.3); // 0.1 s < 0.5 s minimum
        let err = validator.validate(&audio).unwrap_err();
        assert!(matches!(err, AudioError::TooShort { .. }), "{err}");
    }

    #[test]
    fn too_long_rejected() {
        let validator = AudioQuality::new(0.5, 10.0);
        let audio = make_audio(11.0, 0.3); // 11 s > 10 s maximum
        let err = validator.validate(&audio).unwrap_err();
        assert!(matches!(err, AudioError::TooLong { .. }), "{err}");
    }

    #[test]
    fn silent_audio_rejected() {
        let validator = AudioQuality::default();
        let audio = make_audio(1.0, 0.0); // all zeros
        let err = validator.validate(&audio).unwrap_err();
        assert!(matches!(err, AudioError::TooQuiet { .. }), "{err}");
    }

    #[test]
    fn amplitude_just_below_threshold_rejected() {
        let mut validator = AudioQuality::default();
        validator.silence_threshold = 0.05;
        let audio = make_audio(1.0, 0.04); // 0.04 < 0.05
        assert!(matches!(
            validator.validate(&audio).unwrap_err(),
            AudioError::TooQuiet { .. }
        ));
    }

    #[test]
    fn clipping_rejected() {
        let mut validator = AudioQuality::default();
        validator.clipping_max_pct = 5.0;

        // 50% clipped (all samples at 1.0 > 0.99 threshold)
        let audio = make_audio(1.0, 1.0);
        let err = validator.validate(&audio).unwrap_err();
        assert!(matches!(err, AudioError::Clipping { .. }), "{err}");
    }

    #[test]
    fn minor_clipping_within_limit_passes() {
        let validator = AudioQuality::default(); // 10% max
        // Build audio: 90% quiet + 5% clipped (< 10% limit)
        let total = 16_000usize;
        let clipped_count = total * 5 / 100;
        let mut audio = vec![0.3_f32; total - clipped_count];
        audio.extend(vec![1.0_f32; clipped_count]);
        assert!(validator.validate(&audio).is_ok());
    }

    #[test]
    fn at_minimum_duration_passes() {
        let validator = AudioQuality::new(0.5, 60.0);
        // Exactly 0.5 s (8000 samples @ 16 kHz)
        let audio = make_audio(0.5, 0.2);
        assert!(validator.validate(&audio).is_ok());
    }

    #[test]
    fn error_display_is_informative() {
        let err = AudioError::TooShort {
            min_secs: 0.5,
            got_secs: 0.1,
        };
        let msg = err.to_string();
        assert!(msg.contains("0.10"), "message: {msg}");
        assert!(msg.contains("0.50"), "message: {msg}");
    }
}
