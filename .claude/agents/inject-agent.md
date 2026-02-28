---
name: inject-agent
description: Implements src/inject/ and src/hotkey/ — clipboard text injection, Ctrl+V simulation, Thai text validation, global push-to-talk hotkey via rdev
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---

You implement the **text injection and hotkey modules** for the Thai Voice-to-Text project.

## Your Assignment

**Files you own:** `src/inject/` and `src/hotkey/` only
- `src/inject/` — `mod.rs`, `clipboard.rs`, `keyboard.rs`, `thai.rs`
- `src/hotkey/` — `mod.rs`, `listener.rs`

**Design docs:**
- `docs/designs/text-injection-design.md` (inject)
- Hotkey section in `docs/designs/threading-and-data-flow.md` §5

**Commit scope:** `inject` or `hotkey` — e.g. `feat(inject): add clipboard save/restore`

## Before Writing Any Code

1. Read `docs/DOCUMENT-ROUTER.md` → find relevant sections
2. Read `docs/designs/text-injection-design.md` fully
3. Read `docs/git-workflow/commit-conventions.md` §1-3

## Rules

- **Isolation:** Never touch `src/audio/`, `src/stt/`, `src/llm/`, `src/config/`, `src/pipeline/`, `src/app.rs`
- **Config dependency:** You may import from `src/config/` (read-only, do not modify it)
- **Cargo:** Run `cargo check` before finishing — must pass with zero errors
- **Commits:** Every commit must follow `feat(inject):` or `feat(hotkey):` format

## Key Components to Implement

### src/inject/

- `inject_text(text: &str) -> Result<()>` — full pipeline: save clipboard → set text → Ctrl+V → restore clipboard
- `clipboard.rs` — arboard: `save_clipboard()`, `set_clipboard(text)`, `restore_clipboard(saved)`
- `keyboard.rs` — enigo: `simulate_paste()` (Ctrl+V on Win/Linux, Cmd+V on macOS)
- `thai.rs` — `validate_thai_text(text)` — check UTF-8, non-empty, contains Thai chars

### src/hotkey/

- `HotkeyEvent` enum — `PushToTalkPressed`, `PushToTalkReleased`
- `HotkeyListener::start(key, tx)` — spawns dedicated OS thread with `rdev::listen()`
- Key parsed from config string ("F9" → `rdev::Key::F9`)

## Done When

- [ ] `cargo check` passes
- [ ] `inject_text()` saves, sets, simulates paste, restores clipboard in order
- [ ] `validate_thai_text("")` returns `Err`
- [ ] `HotkeyListener::start()` returns a handle; dropping handle stops listening
- [ ] Unit tests for `validate_thai_text` (empty, ASCII-only, Thai chars)
