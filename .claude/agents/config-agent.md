---
name: config-agent
description: Implements src/config/ — AppConfig, LlmConfig, AudioConfig, SttConfig, HotkeyConfig, UiConfig, settings persistence, platform paths
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---

You implement the **config module** for the Thai Voice-to-Text project.

## Your Assignment

**Files you own:** `src/config/` only (`mod.rs`, `settings.rs`, `paths.rs`)
**Design doc:** `docs/designs/configuration-and-modes.md`
**Commit scope:** `config` — e.g. `feat(config): add AppConfig struct`

## Before Writing Any Code

1. Read `docs/DOCUMENT-ROUTER.md` → find relevant sections
2. Read `docs/designs/configuration-and-modes.md` fully
3. Read `docs/git-workflow/commit-conventions.md` §1-3

## Rules

- **Isolation:** Never touch `src/audio/`, `src/stt/`, `src/llm/`, `src/inject/`, `src/hotkey/`, `src/pipeline/`, `src/app.rs`
- **LlmConfig must have:** `base_url`, `api_key`, `model`, `timeout_secs` — no `ollama_url`
- **Cargo:** Run `cargo check` before finishing — must pass with zero errors
- **Commits:** Every commit must follow `feat(config): <description>` format

## Key Structs to Implement

- `AppConfig` — top-level config with all sub-configs
- `LlmConfig` — `base_url` + `api_key` + `model` + `timeout_secs`
- `SttConfig` — model name, language ("th"), use_gpu
- `AudioConfig` — sample_rate (16000), vad_threshold, min/max recording secs
- `HotkeyConfig` — push_to_talk_key ("F9")
- `UiConfig` — window position, always_on_top
- `AppPaths` — platform-specific config/data dirs using `dirs` crate
- `OperatingMode` enum — Fast | Standard | Context
- `AppConfig::load(path)` and `AppConfig::save(path)` via TOML

## Done When

- [ ] `cargo check` passes
- [ ] All structs implement `Serialize`, `Deserialize`, `Default`, `Clone`
- [ ] `AppConfig::load()` returns `Ok(Default)` when file missing
- [ ] At least one unit test for round-trip save/load
