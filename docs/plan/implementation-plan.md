# Thai Voice-to-Text — Multi-Agent Implementation Plan

**Date:** 2026-02-28
**Branch Strategy:** GitHub Flow (feature branches → squash merge → main)

## Phase Overview

| Phase | Version | Modules |
|-------|---------|---------|
| Phase 1 | v0.1.0 | config, audio, stt, inject, hotkey, pipeline, ui |
| Phase 2 | v0.2.0 | llm/corrector, llm/prompt, llm/fallback, pipeline (standard mode) |
| Phase 3 | v0.3.0 | llm/context, llm/domain, llm/vocabulary, pipeline (context mode) |
| Phase 4 | v1.0.0 | config/wizard, ui/settings, packaging |

## Module Dependency Order

```
Phase 1:
  config → audio → stt → inject → pipeline → ui

Phase 2:
  llm-ollama-client → pipeline-standard-mode

Phase 3:
  llm-context-window  ─┐
  llm-domain-detection ├─► pipeline-context-mode
  llm-user-vocabulary ─┘

Phase 4:
  config-first-run-wizard  ─┐
  ui-model-selector         ├─► tag v1.0.0
  packaging-*              ─┘
```

## Tech Stack

| Layer | Crate | Version |
|-------|-------|---------|
| Audio capture | cpal | 0.15 |
| Resampling | rubato | 0.15 |
| STT | whisper-rs | 0.12 |
| UI | eframe + egui | 0.31 |
| Hotkey | rdev | 0.5 |
| Clipboard | arboard | 3.4 |
| Keyboard sim | enigo | 0.3 |
| Async | tokio | 1 |
| Config | serde + toml + dirs | latest |
| HTTP (LLM) | reqwest | 0.12 |
| LLM | Ollama REST API | - |

## File Map

```
src/
├── main.rs                   ← eframe entry point
├── app.rs                    ← VoiceApp (egui widget)
├── config/
│   ├── mod.rs               ← AppConfig, OperatingMode enums
│   ├── settings.rs          ← TOML load/save
│   └── paths.rs             ← Cross-platform data dirs
├── audio/
│   ├── mod.rs
│   ├── capture.rs           ← cpal stream
│   ├── resample.rs          ← 16kHz conversion
│   ├── buffer.rs            ← RingBuffer<f32>
│   ├── vad.rs               ← Energy-based VAD
│   ├── quality.rs           ← Audio validation
│   └── waveform.rs          ← RMS chunks for UI
├── stt/
│   ├── mod.rs
│   ├── engine.rs            ← WhisperEngine + SttEngine trait
│   ├── model.rs             ← GGML path resolution
│   └── transcribe.rs        ← TranscribeParams (Thai, greedy)
├── llm/
│   ├── mod.rs
│   ├── corrector.rs         ← OllamaCorrector + LlmCorrector trait
│   ├── prompt.rs            ← Thai correction prompt builder
│   ├── fallback.rs          ← FallbackCorrector wrapper
│   ├── context.rs           ← ContextManager (rolling window)
│   ├── domain.rs            ← DomainDetector (keyword-based)
│   └── vocabulary.rs        ← UserVocabulary (JSON)
├── inject/
│   ├── mod.rs
│   ├── clipboard.rs         ← arboard save/restore
│   ├── keyboard.rs          ← enigo Ctrl+V
│   └── thai.rs              ← Unicode validation
├── hotkey/
│   ├── mod.rs
│   └── listener.rs          ← rdev global listener
└── pipeline/
    ├── mod.rs               ← AppState, SharedState
    ├── state.rs             ← PipelineState enum
    └── runner.rs            ← PipelineOrchestrator async loop
```

## Verification Checklist

### Phase 1 MVP
- [ ] `cargo build` passes without warnings on Windows
- [ ] `cargo test` — all unit tests pass
- [ ] Hold F9 → record voice → release → see Thai text pasted into Notepad
- [ ] Widget stays always-on-top over other windows
- [ ] Widget is draggable with mouse
- [ ] Waveform animates during recording

### Phase 2
- [ ] Ollama running with `qwen2.5:3b` model loaded
- [ ] Enable Standard Mode in settings
- [ ] Record Thai speech with filler words (เอ่อ, อ่า)
- [ ] Verify output has fillers removed and proper punctuation

### Phase 3
- [ ] Record 4 consecutive sentences
- [ ] Verify context window influences 4th correction
- [ ] Say medical Thai terms — verify domain detected as Medical
- [ ] Add custom word to vocabulary — verify it appears in correction

### Phase 4
- [ ] Fresh install → first-run wizard appears
- [ ] Settings panel saves to TOML, persists after restart
- [ ] Windows .exe builds and runs without Rust installed
