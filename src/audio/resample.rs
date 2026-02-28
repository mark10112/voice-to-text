//! Audio resampling and channel mixing utilities.
//!
//! The Whisper STT engine requires **16 kHz mono `f32`** audio.  This module
//! provides the two conversion steps:
//!
//! 1. [`stereo_to_mono`] — downmix any number of interleaved channels to mono.
//! 2. [`resample_to_16k`] — resample from any source rate to 16 000 Hz.
//!
//! ## Phase 2 note
//!
//! The current resampler uses linear interpolation (fast, zero extra deps).
//! For better audio quality replace the inner loop with the `rubato` crate
//! (`SincFixedIn` + `BlackmanHarris2` window) — rubato is already listed in
//! `Cargo.toml` for that upgrade path.

// ---------------------------------------------------------------------------
// stereo_to_mono
// ---------------------------------------------------------------------------

/// Mix interleaved multi-channel audio down to mono by averaging all channels.
///
/// The output length is `samples.len() / channels`.
///
/// * If `channels == 1` the input slice is returned as an owned `Vec` with no
///   averaging (fast path — avoids an extra allocation when already mono).
/// * If `channels == 0` an empty vector is returned.
///
/// # Example
///
/// ```rust
/// use voice_to_text::audio::stereo_to_mono;
///
/// let stereo = vec![0.5_f32, -0.5, 0.2, -0.2]; // L R L R
/// let mono = stereo_to_mono(&stereo, 2);
/// assert_eq!(mono.len(), 2);
/// // first frame: (0.5 + -0.5) / 2 = 0.0
/// assert!((mono[0] - 0.0).abs() < 1e-6);
/// // second frame: (0.2 + -0.2) / 2 = 0.0
/// assert!((mono[1] - 0.0).abs() < 1e-6);
/// ```
pub fn stereo_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    match channels {
        0 => Vec::new(),
        1 => samples.to_vec(),
        n => {
            let n = n as usize;
            samples
                .chunks_exact(n)
                .map(|frame| frame.iter().sum::<f32>() / n as f32)
                .collect()
        }
    }
}

// ---------------------------------------------------------------------------
// resample_to_16k
// ---------------------------------------------------------------------------

/// Resample `samples` from `source_rate` Hz to 16 000 Hz using linear
/// interpolation.
///
/// * If `source_rate` is already `16_000` the input is cloned and returned
///   unchanged (no-op fast path — no interpolation performed).
/// * If `samples` is empty an empty vector is returned.
///
/// The output length is approximately
/// `samples.len() * 16_000 / source_rate`.
///
/// # Example
///
/// ```rust
/// use voice_to_text::audio::resample_to_16k;
///
/// // Already 16 kHz — no-op
/// let mono_16k = vec![0.1_f32; 160];
/// let out = resample_to_16k(&mono_16k, 16_000);
/// assert_eq!(out.len(), mono_16k.len());
///
/// // Downsample from 48 kHz to 16 kHz (ratio = 1/3)
/// let hi = vec![0.5_f32; 480];
/// let lo = resample_to_16k(&hi, 48_000);
/// assert_eq!(lo.len(), 160);
/// ```
pub fn resample_to_16k(samples: &[f32], source_rate: u32) -> Vec<f32> {
    const TARGET_RATE: u32 = 16_000;

    if source_rate == TARGET_RATE {
        return samples.to_vec();
    }

    if samples.is_empty() {
        return Vec::new();
    }

    let ratio = TARGET_RATE as f64 / source_rate as f64;
    let output_len = (samples.len() as f64 * ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 / ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f64;

        let sample = if idx + 1 < samples.len() {
            // Linear interpolation between adjacent samples
            samples[idx] * (1.0 - frac as f32) + samples[idx + 1] * frac as f32
        } else if idx < samples.len() {
            samples[idx]
        } else {
            0.0
        };

        output.push(sample);
    }

    output
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- stereo_to_mono ----------------------------------------------------

    #[test]
    fn stereo_to_mono_already_mono() {
        let input = vec![0.1_f32, 0.2, 0.3];
        let out = stereo_to_mono(&input, 1);
        assert_eq!(out, input);
    }

    #[test]
    fn stereo_to_mono_two_channel() {
        let input = vec![1.0_f32, -1.0, 0.5, 0.5];
        let out = stereo_to_mono(&input, 2);
        assert_eq!(out.len(), 2);
        assert!((out[0] - 0.0).abs() < 1e-6); // (1.0 + -1.0) / 2
        assert!((out[1] - 0.5).abs() < 1e-6); // (0.5 + 0.5) / 2
    }

    #[test]
    fn stereo_to_mono_four_channel() {
        // 4 interleaved channels: frame [0.4, 0.4, 0.4, 0.4] → 0.4
        let input = vec![0.4_f32; 4];
        let out = stereo_to_mono(&input, 4);
        assert_eq!(out.len(), 1);
        assert!((out[0] - 0.4).abs() < 1e-6);
    }

    #[test]
    fn stereo_to_mono_zero_channels() {
        let out = stereo_to_mono(&[1.0_f32, 2.0], 0);
        assert!(out.is_empty());
    }

    // ---- resample_to_16k ---------------------------------------------------

    #[test]
    fn resample_already_16k_is_noop() {
        let input: Vec<f32> = (0..160).map(|i| i as f32 / 160.0).collect();
        let out = resample_to_16k(&input, 16_000);
        assert_eq!(out.len(), input.len());
        for (a, b) in input.iter().zip(out.iter()) {
            assert!((a - b).abs() < 1e-6, "sample mismatch: {a} vs {b}");
        }
    }

    #[test]
    fn resample_empty_input() {
        let out = resample_to_16k(&[], 48_000);
        assert!(out.is_empty());
    }

    #[test]
    fn resample_48k_to_16k_output_length() {
        // 480 samples @ 48 kHz = 10 ms → should become 160 samples @ 16 kHz
        let input = vec![0.5_f32; 480];
        let out = resample_to_16k(&input, 48_000);
        assert_eq!(out.len(), 160);
    }

    #[test]
    fn resample_44100_to_16k_output_length() {
        // 44100 samples @ 44.1 kHz = 1 second → ~16000 output samples
        let input = vec![0.0_f32; 44_100];
        let out = resample_to_16k(&input, 44_100);
        // Allow ±1 sample rounding tolerance
        let expected = 16_000usize;
        assert!(
            out.len().abs_diff(expected) <= 1,
            "expected ~{expected}, got {}",
            out.len()
        );
    }

    #[test]
    fn resample_constant_signal_preserves_amplitude() {
        // A DC signal (all 0.5) should remain 0.5 after resampling
        let input = vec![0.5_f32; 480];
        let out = resample_to_16k(&input, 48_000);
        for &s in &out {
            assert!((s - 0.5).abs() < 1e-5, "amplitude drift: {s}");
        }
    }

    #[test]
    fn resample_upsample_from_8k_to_16k() {
        // 8 kHz → 16 kHz (upsampling): output should be ~2× length
        let input = vec![0.0_f32; 80]; // 10 ms @ 8 kHz
        let out = resample_to_16k(&input, 8_000);
        assert_eq!(out.len(), 160); // 10 ms @ 16 kHz
    }
}
