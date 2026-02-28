---
name: pipeline-agent
description: Implements src/pipeline/ — PipelineOrchestrator wiring audio→STT→LLM→inject, PipelineState machine, AppState, channels
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---

You implement the **pipeline orchestrator module** for the Thai Voice-to-Text project.

## Your Assignment

**Files you own:** `src/pipeline/` only (`mod.rs`, `runner.rs`, `state.rs`)
**Design docs:**
- `docs/designs/threading-and-data-flow.md` (primary)
- `docs/designs/architecture-overview.md` §4-6 (data flow, state machine)

**Commit scope:** `pipeline` — e.g. `feat(pipeline): wire STT into orchestrator`

## Before Writing Any Code

1. Read `docs/DOCUMENT-ROUTER.md` → find relevant sections
2. Read `docs/designs/threading-and-data-flow.md` fully
3. Read `docs/designs/architecture-overview.md` §4-6
4. Read `docs/git-workflow/commit-conventions.md` §1-3

## Rules

- **Isolation:** Never touch `src/audio/`, `src/stt/`, `src/llm/`, `src/inject/`, `src/hotkey/`, `src/config/`, `src/app.rs`
- **Dependency:** Imports interfaces from other modules — do NOT modify them
- **Must merge after:** config, audio, stt, llm, inject agents are all done
- **Cargo:** Run `cargo check` before finishing — must pass with zero errors
- **Commits:** Every commit must follow `feat(pipeline): <description>` format

## Key Components to Implement

- `PipelineState` enum — Idle | Recording | Transcribing | Correcting | Result | Error
- `AppState` — shared struct: `pipeline`, `last_text`, `waveform`, `config`, `error_message`, `recording_secs`
- `SharedState` — `Arc<Mutex<AppState>>`
- `PipelineOrchestrator` — owns `SharedState`, drives the full pipeline loop
- `PipelineOrchestrator::run(stt: Arc<dyn SttEngine>, llm: Arc<dyn LlmCorrector>)` — async loop
  - Listens for `HotkeyEvent` via mpsc
  - On `PushToTalkPressed` → start `AudioCapture`
  - On `PushToTalkReleased` → drain buffer → `spawn_blocking(stt.transcribe)` → LLM correct (if mode != Fast) → `spawn_blocking(inject_text)`
  - Updates `AppState.pipeline` at each stage
  - Graceful fallback: if LLM fails → inject raw STT text

## LLM Wiring (Phase 2 — must implement, not stub)

```rust
// Standard / Context mode: call LLM corrector
match llm.correct(&raw_text, context).await {
    Ok(corrected) => corrected,
    Err(e) => { log::warn!("LLM failed, using raw: {e}"); raw_text }
}
```

## Done When

- [ ] `cargo check` passes
- [ ] `PipelineState::is_busy()` returns true for Recording/Transcribing/Correcting
- [ ] Fast mode skips LLM call entirely
- [ ] LLM error falls back to raw text (no crash)
- [ ] `AppState.pipeline` transitions correctly through all states
- [ ] Unit tests for `PipelineState` labels and `is_busy()`
