//! Waveform amplitude data for the UI visualisation bar.
//!
//! The egui widget polls [`WaveformData::compute`] on each frame to render a
//! bar chart showing the recording's amplitude envelope.
//!
//! # Example
//!
//! ```rust
//! use voice_to_text::audio::WaveformData;
//!
//! // Simulate 1 second of audio at 16 kHz
//! let audio: Vec<f32> = (0..16_000)
//!     .map(|i| (i as f32 * 0.001).sin() * 0.5)
//!     .collect();
//!
//! let waveform = WaveformData::compute(&audio, 20);
//! assert_eq!(waveform.bars.len(), 20);
//! // Every bar should be in [0.0, 1.0]
//! for &bar in &waveform.bars {
//!     assert!(bar >= 0.0 && bar <= 1.0);
//! }
//! ```

// ---------------------------------------------------------------------------
// WaveformData
// ---------------------------------------------------------------------------

/// Amplitude snapshot for the UI waveform bar chart.
///
/// Each element of `bars` is an RMS amplitude value in `[0.0, 1.0]`
/// representing an equal-width chunk of the input audio.
#[derive(Debug, Clone)]
pub struct WaveformData {
    /// RMS amplitude per bar, clamped to `[0.0, 1.0]`.
    pub bars: Vec<f32>,
}

impl WaveformData {
    /// Compute `num_bars` RMS amplitude values from `audio`.
    ///
    /// The audio is divided into `num_bars` equal-sized chunks; the RMS of
    /// each chunk becomes one bar value.  If `audio` is shorter than
    /// `num_bars` the remaining bars are padded with `0.0`.
    ///
    /// # Arguments
    ///
    /// * `audio` — 16 kHz mono `f32` samples.
    /// * `num_bars` — number of bars to produce (e.g. `20` for a 20-column
    ///   waveform widget).  If `0`, an empty `WaveformData` is returned.
    pub fn compute(audio: &[f32], num_bars: usize) -> Self {
        if num_bars == 0 {
            return Self { bars: Vec::new() };
        }

        if audio.is_empty() {
            return Self {
                bars: vec![0.0; num_bars],
            };
        }

        let chunk_size = (audio.len() / num_bars).max(1);

        let mut bars: Vec<f32> = audio
            .chunks(chunk_size)
            .take(num_bars)
            .map(|chunk| {
                let mean_sq: f32 =
                    chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32;
                mean_sq.sqrt().min(1.0) // clamp to [0.0, 1.0]
            })
            .collect();

        // Pad any remaining bars with 0.0
        bars.resize(num_bars, 0.0);

        Self { bars }
    }

    /// Number of bars.
    pub fn len(&self) -> usize {
        self.bars.len()
    }

    /// Returns `true` when there are no bars.
    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }

    /// Peak bar value across the waveform (useful for normalisation).
    pub fn peak(&self) -> f32 {
        self.bars.iter().cloned().fold(0.0_f32, f32::max)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_number_of_bars() {
        let audio = vec![0.3_f32; 16_000];
        let w = WaveformData::compute(&audio, 20);
        assert_eq!(w.bars.len(), 20);
    }

    #[test]
    fn bars_clamped_to_unit_range() {
        // Samples at full scale — RMS = 1.0 → clamped to 1.0
        let audio = vec![1.0_f32; 1_600];
        let w = WaveformData::compute(&audio, 10);
        for &b in &w.bars {
            assert!(b >= 0.0 && b <= 1.0, "bar out of range: {b}");
        }
    }

    #[test]
    fn silent_audio_all_zero_bars() {
        let audio = vec![0.0_f32; 1_600];
        let w = WaveformData::compute(&audio, 10);
        for &b in &w.bars {
            assert_eq!(b, 0.0);
        }
    }

    #[test]
    fn empty_audio_returns_zero_bars() {
        let w = WaveformData::compute(&[], 10);
        assert_eq!(w.bars.len(), 10);
        for &b in &w.bars {
            assert_eq!(b, 0.0);
        }
    }

    #[test]
    fn zero_num_bars_returns_empty() {
        let audio = vec![0.5_f32; 1_000];
        let w = WaveformData::compute(&audio, 0);
        assert!(w.is_empty());
    }

    #[test]
    fn peak_reflects_max_bar() {
        let audio = vec![0.5_f32; 1_600]; // constant 0.5 → RMS = 0.5
        let w = WaveformData::compute(&audio, 10);
        let peak = w.peak();
        assert!((peak - 0.5).abs() < 1e-4, "peak = {peak}");
    }

    #[test]
    fn bars_shorter_than_num_bars_padded_with_zeros() {
        // Only 1 sample — cannot fill 10 bars; remaining should be 0
        let audio = vec![0.5_f32; 1];
        let w = WaveformData::compute(&audio, 10);
        assert_eq!(w.bars.len(), 10);
        // The last bars must be 0.0 (padding)
        assert!(w.bars.iter().skip(1).all(|&b| b == 0.0));
    }

    #[test]
    fn len_and_is_empty() {
        let w = WaveformData::compute(&[], 5);
        assert_eq!(w.len(), 5);
        assert!(!w.is_empty());

        let empty = WaveformData::compute(&[], 0);
        assert!(empty.is_empty());
    }
}
