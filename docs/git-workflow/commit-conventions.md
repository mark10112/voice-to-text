# Commit Conventions

**วันที่:** 28 กุมภาพันธ์ 2026

ใช้ **Conventional Commits 1.0** ปรับให้เข้ากับ project นี้

---

## 1. Commit Message Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Rules
- **type** — required, lowercase
- **scope** — required for this project (module name)
- **description** — imperative mood, no period, ≤72 chars, English
- **body** — optional; Thai ก็ได้; อธิบาย *why* ไม่ใช่ *what*
- **footer** — optional; `BREAKING CHANGE:`, `Fixes #123`, `Co-authored-by:`

---

## 2. Types

| Type | When to Use | Example |
|------|------------|---------|
| `feat` | New feature or capability | `feat(audio): add 16kHz resampling` |
| `fix` | Bug fix | `fix(stt): handle empty transcription result` |
| `perf` | Performance improvement | `perf(stt): reduce whisper beam size for latency` |
| `refactor` | Code restructure, no behavior change | `refactor(pipeline): extract state machine enum` |
| `test` | Tests only — no production code | `test(audio): add ring buffer overflow tests` |
| `docs` | Documentation only | `docs(llm): update ollama integration notes` |
| `chore` | Maintenance — deps, build, scripts | `chore(deps): upgrade whisper-rs to 0.12` |
| `style` | Formatting, no logic change | `style(audio): run cargo fmt on capture.rs` |
| `ci` | CI/CD configuration | `ci: add cargo clippy to github actions` |
| `build` | Build system changes | `build: add release profile to Cargo.toml` |

---

## 3. Scopes (Project-Specific)

| Scope | Maps To | Files |
|-------|---------|-------|
| `audio` | Audio pipeline | `src/audio/` (capture, vad, buffer) |
| `stt` | STT engine | `src/stt/` (engine, model) |
| `llm` | LLM correction | `src/llm/` (corrector, prompt, context) |
| `inject` | Text injection | `src/inject/` (clipboard, keyboard) |
| `hotkey` | Global hotkey | `src/hotkey/` |
| `pipeline` | Orchestrator | `src/pipeline/` |
| `config` | Configuration | `src/config/` |
| `ui` | egui widget | `src/app.rs`, `assets/` |
| `deps` | Dependencies | `Cargo.toml` only |
| `ci` | CI/CD | `.github/workflows/` |
| `docs` | Documentation | `docs/**`, `CLAUDE.md` |

---

## 4. Breaking Changes

```bash
# Option A: exclamation mark after type
feat(stt)!: change transcribe() to return Result<Vec<String>>

# Option B: footer
feat(stt): refactor transcription API

BREAKING CHANGE: transcribe() now returns Vec<String> instead of String.
Update all callers to handle multiple segments.
```

---

## 5. Real Examples — Phase 1 (MVP)

```bash
# Audio module
feat(audio): add cpal device enumeration
feat(audio): implement f32 ring buffer with 16kHz mono
feat(audio): integrate silero vad for silence detection
feat(audio): add waveform amplitude data for ui
fix(audio): prevent buffer overflow on long recordings
perf(audio): reduce ring buffer lock contention

# STT module
feat(stt): add whisper-rs engine struct
feat(stt): implement thonburian whisper ggml loading
feat(stt): add thai language hint to transcription params
fix(stt): handle missing model file with user prompt
fix(stt): return error on audio shorter than 1 second
perf(stt): use greedy strategy to reduce latency

# UI module
feat(ui): add floating egui widget with always-on-top
feat(ui): implement push-to-talk button
feat(ui): add waveform visualization bar
feat(ui): show raw transcription in gray during processing

# Hotkey + Inject
feat(hotkey): add rdev global listener for configurable key
feat(inject): implement arboard clipboard save and restore
feat(inject): add enigo ctrl+v text injection
fix(inject): restore clipboard after 50ms delay on windows
```

## 5b. Real Examples — Phase 2-4

```bash
# Phase 2 — LLM
feat(llm): add ollama rest api client with reqwest
feat(llm): implement thai correction prompt builder
feat(llm): add filler word removal (เอ่อ, อ่า, ก็คือ)
feat(llm): add punctuation restoration to prompt
feat(pipeline): add standard mode with llm correction step
fix(llm): fallback to raw stt on ollama timeout

# Phase 3 — Context
feat(llm): add rolling context window (3 sentences)
feat(llm): implement domain detection (medical/legal/tech/casual)
feat(llm): persist user vocabulary to toml file
feat(llm): inject few-shot examples per detected domain
feat(pipeline): add context mode with full rolling window

# Phase 4 — Polish
feat(ui): add model selector dropdown (small/medium/large)
feat(config): implement toml settings persistence with dirs crate
feat(config): add first-run setup wizard with model download
build: add windows msi packaging via cargo-wix
build: add macos dmg packaging via cargo-bundle
docs(config): document toml schema and default values
```

---

## 6. Bad vs Good

| ❌ Bad | ✅ Good |
|--------|---------|
| `fix bug` | `fix(stt): handle empty transcription result` |
| `update stuff` | `refactor(pipeline): extract state transitions to enum` |
| `WIP` | Use draft PR instead; commit: `feat(audio): wip - ring buffer skeleton` |
| `feat: add lots of things` | Split into separate atomic commits per feature |
| `fixed the audio thing` | `fix(audio): prevent ring buffer overflow on 60s+ recordings` |

---

## 7. Git Hook: commit-msg Validator

บันทึกไฟล์นี้เป็น `.git/hooks/commit-msg` แล้ว `chmod +x`:

```bash
#!/bin/sh
# Validates Conventional Commits format
# Pattern: type(scope): description

commit_regex='^(feat|fix|perf|refactor|test|docs|chore|style|ci|build)(\([a-z]+\))!?: .{1,72}$'

commit_msg=$(head -n1 "$1")

if ! echo "$commit_msg" | grep -qE "$commit_regex"; then
  echo ""
  echo "ERROR: Invalid commit message format."
  echo ""
  echo "Expected: <type>(<scope>): <description>"
  echo "Example:  feat(audio): add ring buffer implementation"
  echo ""
  echo "Types:  feat fix perf refactor test docs chore style ci build"
  echo "Scopes: audio stt llm inject hotkey pipeline config ui deps ci docs"
  echo ""
  echo "Your message: $commit_msg"
  echo ""
  exit 1
fi
```

**ติดตั้ง:**
```bash
cp .git/hooks/commit-msg.sample .git/hooks/commit-msg  # if exists
# or create new:
cat > .git/hooks/commit-msg << 'EOF'
#!/bin/sh
# ... (paste script above)
EOF
chmod +x .git/hooks/commit-msg
```

---

## 8. Quick Reference Card

```
feat(scope):    New feature
fix(scope):     Bug fix
perf(scope):    Performance
refactor(scope): Restructure
test(scope):    Tests only
docs(scope):    Docs only
chore(scope):   Maintenance
ci:             CI/CD
build:          Build system

Scopes: audio | stt | llm | inject | hotkey | pipeline | config | ui | deps | ci | docs

Format:  feat(audio): add ring buffer   ← ≤72 chars, imperative, no period
Breaking: feat(stt)!: change api        ← add ! for breaking changes
```
