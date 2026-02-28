# Changelog

All notable changes to Thai Voice-to-Text will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project skeleton
- Config module: AppConfig, TOML persistence, cross-platform paths
- Audio module: cpal capture, ring buffer, VAD, resampling, waveform
- STT module: whisper-rs engine, Thai language transcription
- Inject module: arboard clipboard, enigo keyboard simulation, Thai validation
- Hotkey module: rdev global push-to-talk listener
- Pipeline module: orchestrator, state machine, channel wiring
- UI module: egui floating widget, waveform visualization
- LLM module: Ollama client, Thai correction prompt, fallback
- LLM Phase 3: context window, domain detection, user vocabulary

## [0.1.0] - TBD

### Added
- MVP: Audio capture → VAD → Whisper STT → Text injection
- Global hotkey (F9) push-to-talk
- egui floating always-on-top widget
- Fast mode (no LLM correction)
