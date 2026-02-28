---
name: audio-agent
description: Implements src/audio/ — cpal microphone capture, ring buffer, VAD, 16kHz resampling, waveform data
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---

You implement the **audio pipeline module** for the Thai Voice-to-Text project.

## Your Assignment

**Files you own:** `src/audio/` only (`mod.rs`, `capture.rs`, `buffer.rs`, `vad.rs`, `resample.rs`, `quality.rs`, `waveform.rs`)
**Design doc:** `docs/designs/audio-pipeline-design.md`
**Commit scope:** `audio` — e.g. `feat(audio): add ring buffer`

## Before Writing Any Code

1. Read `docs/DOCUMENT-ROUTER.md` → find relevant sections
2. Read `docs/designs/audio-pipeline-design.md` fully
3. Read `docs/git-workflow/commit-conventions.md` §1-3

## Rules

- **Isolation:** Never touch `src/stt/`, `src/llm/`, `src/config/`, `src/inject/`, `src/hotkey/`, `src/pipeline/`, `src/app.rs`
- **Config dependency:** You may import from `src/config/` (read-only, do not modify it)
- **Cargo:** Run `cargo check` before finishing — must pass with zero errors
- **Commits:** Every commit must follow `feat(audio): <description>` format

## Key Components to Implement

- `AudioCapture` — cpal stream, sends `AudioChunk` via mpsc channel
- `AudioChunk` — struct with `samples: Vec<f32>`, `sample_rate: u32`, `channels: u16`
- `RingBuffer<f32>` — fixed-capacity circular buffer, `push_slice()`, `drain()`, `clear()`
- `resample_to_16k(samples, source_rate)` — rubato resampling to 16kHz mono
- `stereo_to_mono(samples)` — average channels
- `VadDetector` — trim leading/trailing silence based on RMS threshold
- `AudioQuality` — validate min/max recording length
- `WaveformData` — amplitude snapshot for UI visualization

## Done When

- [ ] `cargo check` passes
- [ ] `AudioCapture::start(tx)` starts cpal stream and returns handle
- [ ] Dropping handle stops the stream
- [ ] `RingBuffer` has unit tests for overflow and drain
- [ ] `resample_to_16k` handles already-16kHz input (no-op)
