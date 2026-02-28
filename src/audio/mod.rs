//! Audio pipeline — microphone capture → resampling → ring buffer → VAD → quality check.
//!
//! # Pipeline
//!
//! ```text
//! Microphone → cpal callback → AudioChunk (mpsc) → resample_to_16k
//!           → stereo_to_mono → RingBuffer → VadDetector → AudioQuality
//! ```
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use std::sync::mpsc;
//! use voice_to_text::audio::{AudioCapture, AudioChunk};
//!
//! let (tx, rx) = mpsc::channel::<AudioChunk>();
//! let capture = AudioCapture::new().unwrap();
//! let _handle = capture.start(tx).unwrap(); // drops handle → stops stream
//!
//! while let Ok(chunk) = rx.recv() {
//!     println!("received {} samples @ {}Hz", chunk.samples.len(), chunk.sample_rate);
//! }
//! ```

pub mod buffer;
pub mod capture;
pub mod quality;
pub mod resample;
pub mod vad;
pub mod waveform;

pub use buffer::RingBuffer;
pub use capture::{AudioCapture, AudioChunk, StreamHandle};
pub use quality::{AudioError, AudioQuality};
pub use resample::{resample_to_16k, stereo_to_mono};
pub use vad::VadDetector;
pub use waveform::WaveformData;
