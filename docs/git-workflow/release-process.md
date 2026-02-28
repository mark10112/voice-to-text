# Release Process

**วันที่:** 28 กุมภาพันธ์ 2026

---

## 1. Versioning Scheme

ใช้ **Semantic Versioning (SemVer)**: `MAJOR.MINOR.PATCH`

| Part | When to Bump | Example |
|------|-------------|---------|
| `MAJOR` | Breaking UX or API changes | `0.x.x` → `1.0.0` |
| `MINOR` | New phase / feature complete | `0.1.0` → `0.2.0` |
| `PATCH` | Bug fixes, minor improvements | `0.1.0` → `0.1.1` |

**Pre-1.0:** MAJOR stays `0` until Phase 4 polish complete

---

## 2. Version → Phase Mapping

| Version | Phase | Description | Status |
|---------|-------|-------------|--------|
| `v0.1.0` | Phase 1 MVP | Audio + STT + egui widget + hotkey + inject (Fast Mode only) | Target |
| `v0.1.x` | Phase 1 patches | Bug fixes for MVP | — |
| `v0.2.0` | Phase 2 LLM | Ollama integration, filler removal, Standard Mode | — |
| `v0.2.x` | Phase 2 patches | LLM prompt improvements, timeout tuning | — |
| `v0.3.0` | Phase 3 Context | Rolling window, domain detection, user vocab, Context Mode | — |
| `v0.3.x` | Phase 3 patches | Vocabulary improvements, domain tuning | — |
| `v1.0.0` | Phase 4 Polish | Model selector UI, settings persistence, first-run wizard, packaging | — |
| `v1.x.x` | Post-launch | Feature additions, platform improvements | — |

---

## 3. Release Branch Workflow

```
main ──────────────────────────────────────────────────────────────►
  │                   ▲              ▲              ▲
  │                   │              │              │
  └─► release/v0.1.0 ─┘  └─ hotfix ─┘  └─ release/v0.2.0
         (prep only)        v0.1.1           (prep only)
```

### Steps (Standard Release)

```bash
# 1. Create release branch from main
git switch -c release/v0.1.0 main

# 2. Bump version in Cargo.toml
#    version = "0.1.0"
# (edit Cargo.toml)

# 3. Update CHANGELOG.md (see §4)

# 4. Final testing on release branch
cargo test --release
# manual testing on Windows, macOS, Linux

# 5. Commit version bump
git add Cargo.toml CHANGELOG.md
git commit -m "chore(release): bump version to v0.1.0"

# 6. Merge to main (merge commit to preserve release context)
git switch main
git merge --no-ff release/v0.1.0 -m "release: v0.1.0 Phase 1 MVP"

# 7. Create annotated tag
git tag -a v0.1.0 -m "Phase 1 MVP: Audio capture + Thonburian Whisper STT + egui widget + hotkey + text injection"

# 8. Push
git push origin main
git push origin --tags

# 9. Delete release branch
git branch -d release/v0.1.0
git push origin --delete release/v0.1.0
```

---

## 4. CHANGELOG Format

บันทึกเป็น `CHANGELOG.md` ที่ root — ตาม [Keep a Changelog](https://keepachangelog.com):

```markdown
# Changelog

All notable changes to Thai Voice-to-Text are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

---

## [Unreleased]

### Added
- ...

---

## [0.1.0] — 2026-03-15 — Phase 1 MVP

### Added
- `feat(audio)` — cpal microphone capture at 16kHz mono via WASAPI/CoreAudio/ALSA
- `feat(audio)` — f32 ring buffer with configurable capacity
- `feat(audio)` — Silero VAD integration for silence trimming
- `feat(stt)` — Thonburian Whisper Medium (GGML) via whisper-rs
- `feat(stt)` — Thai language STT with greedy decoding strategy
- `feat(ui)` — Always-on-top floating egui widget
- `feat(ui)` — Push-to-talk button with waveform visualization
- `feat(hotkey)` — Global hotkey listener via rdev (F9 default)
- `feat(inject)` — Clipboard-based text injection via arboard + enigo
- Fast Mode: direct STT output without LLM correction

### Fixed
- Nothing yet (first release)

### Known Issues
- Wayland hotkey support limited (rdev limitation)
- macOS requires Accessibility permission on first launch

---

## [0.0.1] — 2026-02-28 — Initial project setup

### Added
- Project structure, CLAUDE.md, design documents
- Research documentation for Thai STT and LLM correction
```

---

## 5. Git Tag Commands

```bash
# สร้าง annotated tag (required — ห้ามใช้ lightweight tag สำหรับ releases)
git tag -a v0.1.0 -m "Phase 1 MVP: Audio + STT + Widget + Hotkey + Inject"

# รายการ tags ทั้งหมด
git tag -l "v*" --sort=version:refname

# ดู tag details
git show v0.1.0

# Push tags ไป remote
git push origin --tags

# Push tag เดียว
git push origin v0.1.0

# ลบ tag ที่ผิด (local + remote)
git tag -d v0.1.0
git push origin :refs/tags/v0.1.0

# สร้างใหม่ที่ถูกต้อง
git tag -a v0.1.0 -m "Phase 1 MVP release"
git push origin v0.1.0

# Checkout code ณ version ใดๆ
git checkout v0.1.0
```

---

## 6. Hotfix Release Process

เมื่อพบ critical bug ใน production:

```bash
# 1. Branch จาก tag ที่ affected
git switch -c hotfix/stt-null-crash v0.1.0

# 2. Fix the bug
# edit src/stt/engine.rs
git add src/stt/engine.rs
git commit -m "fix(stt): handle null model path on startup"

# 3. Bump patch version
# Cargo.toml: version = "0.1.1"
# CHANGELOG.md: add [0.1.1] section
git add Cargo.toml CHANGELOG.md
git commit -m "chore(release): bump version to v0.1.1"

# 4. Merge to main
git switch main
git merge --no-ff hotfix/stt-null-crash -m "hotfix: v0.1.1 fix stt null crash"

# 5. Tag
git tag -a v0.1.1 -m "Hotfix: fix STT null model path crash on startup"
git push origin main --tags

# 6. Cleanup
git branch -d hotfix/stt-null-crash
git push origin --delete hotfix/stt-null-crash
```

---

## 7. Pre-Release Checklist

ก่อน tag และ push release:

- [ ] `cargo build --release` passes on Windows
- [ ] `cargo build --release` passes on macOS (if available)
- [ ] `cargo build --release` passes on Linux (if available)
- [ ] `cargo test --release` — all tests pass
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] No model files (`.gguf`, `.bin`, `.onnx`) in git: `git ls-files | grep -E '\.(gguf|bin|onnx)$'`
- [ ] `Cargo.toml` version matches release tag
- [ ] `CHANGELOG.md` updated with release date and all notable changes
- [ ] Push-to-talk hotkey works (Windows manual test)
- [ ] Thai text transcription works end-to-end
- [ ] Text injection into Notepad works
- [ ] Widget stays always-on-top
- [ ] Widget is draggable
- [ ] `README.md` updated with new version (if applicable)
- [ ] GitHub Release draft ready with release notes
- [ ] Platform artifacts built and tested (see §8)

---

## 8. Platform Artifacts

| Platform | Artifact | Build Command |
|----------|----------|---------------|
| Windows | `voice-to-text.exe` | `cargo build --release --target x86_64-pc-windows-msvc` |
| Windows installer | `voice-to-text.msi` | `cargo wix` (cargo-wix) |
| macOS (Apple Silicon) | `voice-to-text` | `cargo build --release --target aarch64-apple-darwin` |
| macOS (Intel) | `voice-to-text` | `cargo build --release --target x86_64-apple-darwin` |
| macOS bundle | `voice-to-text.app` + `.dmg` | `cargo bundle --release` |
| Linux | `voice-to-text` | `cargo build --release --target x86_64-unknown-linux-gnu` |
| Linux portable | `voice-to-text.AppImage` | `cargo appimage` (or manual) |

**Phase 1 target:** Windows binary only (`.exe`)
**Phase 4 target:** All platforms with installers

---

## 9. GitHub Release Notes Template

```markdown
## Thai Voice-to-Text v0.1.0 — Phase 1 MVP

ครั้งแรกที่ Thai Voice-to-Text พร้อมใช้งาน 🎙️

### ✨ Features
- **Audio Capture** — microphone via cpal (WASAPI on Windows)
- **Thai STT** — Thonburian Whisper Medium (GGML) via whisper-rs
- **Floating Widget** — always-on-top egui widget
- **Push-to-Talk** — global hotkey (F9 default) via rdev
- **Text Injection** — clipboard + Ctrl+V simulation

### 📦 Download
| Platform | File | Size |
|----------|------|------|
| Windows x64 | `voice-to-text-v0.1.0-windows-x64.exe` | ~XX MB |

### ⚙️ Requirements
- Windows 10/11 x64
- RAM: 4GB minimum, 8GB recommended
- Microphone

### 🚫 Known Limitations
- Fast Mode only (no LLM correction yet — coming in v0.2.0)
- Wayland hotkey support limited

### 📋 Full Changelog
See [CHANGELOG.md](./CHANGELOG.md)

---
*Built with Rust 🦀 | Thonburian Whisper STT | egui*
```
