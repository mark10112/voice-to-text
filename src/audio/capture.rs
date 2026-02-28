//! Microphone capture via `cpal`.
//!
//! [`AudioCapture`] wraps the cpal host/device/stream lifecycle.  Call
//! [`AudioCapture::start`] to begin streaming [`AudioChunk`]s over an mpsc
//! channel.  The returned [`StreamHandle`] is a RAII guard — dropping it
//! stops the underlying cpal stream.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
use thiserror::Error;

// ---------------------------------------------------------------------------
// AudioChunk
// ---------------------------------------------------------------------------

/// A single buffer of raw audio as delivered by the cpal callback.
///
/// Samples are interleaved `f32` in the range `[-1.0, 1.0]`.
/// Use [`crate::audio::stereo_to_mono`] to downmix channels and
/// [`crate::audio::resample_to_16k`] to convert to 16 kHz before passing
/// audio to the STT engine.
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// Interleaved PCM samples in `[-1.0, 1.0]`.
    pub samples: Vec<f32>,
    /// Sample rate of this chunk in Hz (e.g. 44100, 48000, 16000).
    pub sample_rate: u32,
    /// Number of interleaved channels (1 = mono, 2 = stereo, …).
    pub channels: u16,
}

// ---------------------------------------------------------------------------
// StreamHandle
// ---------------------------------------------------------------------------

/// RAII guard that keeps the cpal stream alive.
///
/// Dropping this value calls `cpal::Stream::drop` which pauses/stops the
/// underlying hardware stream.
pub struct StreamHandle {
    _stream: cpal::Stream,
}

// ---------------------------------------------------------------------------
// CaptureError
// ---------------------------------------------------------------------------

/// Errors that can occur while setting up or running the audio capture.
#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("no input device found on the default audio host")]
    NoDevice,

    #[error("failed to query default input config: {0}")]
    DefaultConfig(#[from] cpal::DefaultStreamConfigError),

    #[error("failed to build input stream: {0}")]
    BuildStream(#[from] cpal::BuildStreamError),

    #[error("failed to start audio stream: {0}")]
    PlayStream(#[from] cpal::PlayStreamError),
}

// ---------------------------------------------------------------------------
// AudioCapture
// ---------------------------------------------------------------------------

/// Microphone capture device wrapper built on top of `cpal`.
///
/// # Example
///
/// ```rust,no_run
/// use std::sync::mpsc;
/// use voice_to_text::audio::{AudioCapture, AudioChunk};
///
/// let (tx, rx) = mpsc::channel::<AudioChunk>();
/// let capture = AudioCapture::new().unwrap();
/// let _handle = capture.start(tx).unwrap();
/// // `_handle` keeps the stream alive; drop it to stop recording.
/// ```
pub struct AudioCapture {
    device: cpal::Device,
    config: cpal::StreamConfig,
    /// Native sample rate reported by the device (Hz).
    sample_rate: u32,
    /// Number of interleaved channels reported by the device.
    channels: u16,
}

impl AudioCapture {
    /// Create a new [`AudioCapture`] using the system default input device.
    ///
    /// Queries the device's preferred stream configuration (sample rate,
    /// channels, buffer size) so no manual configuration is required.
    ///
    /// # Errors
    ///
    /// Returns [`CaptureError::NoDevice`] when no input device is available,
    /// or [`CaptureError::DefaultConfig`] when the device cannot report a
    /// default stream configuration.
    pub fn new() -> Result<Self, CaptureError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(CaptureError::NoDevice)?;

        let supported = device.default_input_config()?;

        let channels = supported.channels();
        let sample_rate = supported.sample_rate().0;
        let config: cpal::StreamConfig = supported.into();

        Ok(Self {
            device,
            config,
            sample_rate,
            channels,
        })
    }

    /// Start recording and send [`AudioChunk`]s to `tx`.
    ///
    /// The cpal callback runs on a dedicated audio thread; each time the
    /// hardware delivers a buffer the raw `f32` samples are wrapped in an
    /// [`AudioChunk`] and forwarded over the channel.  Send errors (receiver
    /// dropped) are silently ignored so the audio thread never panics.
    ///
    /// # Errors
    ///
    /// Returns [`CaptureError::BuildStream`] or [`CaptureError::PlayStream`]
    /// if the platform rejects the stream configuration.
    pub fn start(&self, tx: mpsc::Sender<AudioChunk>) -> Result<StreamHandle, CaptureError> {
        let sample_rate = self.sample_rate;
        let channels = self.channels;

        let stream = self.device.build_input_stream(
            &self.config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let chunk = AudioChunk {
                    samples: data.to_vec(),
                    sample_rate,
                    channels,
                };
                // Ignore send errors; the receiver may have been dropped.
                let _ = tx.send(chunk);
            },
            |err: cpal::StreamError| {
                log::error!("cpal stream error: {err}");
            },
            None, // no timeout
        )?;

        stream.play()?;
        Ok(StreamHandle { _stream: stream })
    }

    /// Native sample rate of the capture stream in Hz.
    ///
    /// This is the rate reported by the device (commonly 44 100 or 48 000 Hz).
    /// Use [`crate::audio::resample_to_16k`] to downsample before STT.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Number of interleaved channels in each [`AudioChunk`].
    pub fn channels(&self) -> u16 {
        self.channels
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// `AudioChunk` must be `Send` so it can cross thread boundaries.
    #[test]
    fn audio_chunk_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<AudioChunk>();
    }

    /// `StreamHandle` wraps `cpal::Stream` which is not Send on all
    /// platforms — we only assert it is `'static` here (no Send bound).
    #[test]
    fn audio_chunk_fields() {
        let chunk = AudioChunk {
            samples: vec![0.0_f32; 512],
            sample_rate: 48_000,
            channels: 2,
        };
        assert_eq!(chunk.samples.len(), 512);
        assert_eq!(chunk.sample_rate, 48_000);
        assert_eq!(chunk.channels, 2);
    }
}
