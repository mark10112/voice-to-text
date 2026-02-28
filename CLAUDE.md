# Thai Voice-to-Text — Claude Instructions

Desktop widget สำหรับ voice-to-text ภาษาไทย | Stack: Rust · Whisper STT · LLM Post-processing
ทำงาน offline-first, cross-platform (Win/macOS/Linux) | UI: egui floating widget

---

## Document Router

เมื่อได้รับ task ที่ต้องอ่าน doc — **Read `docs/DOCUMENT-ROUTER.md`** แล้วเลือกเฉพาะไฟล์ที่ match (1-2 ไฟล์)
ครอบคลุม: Audio · STT · LLM · UI · Inject · Threading · Config · Research · Git Workflow

---

## Tech Stack (Settled)

```
Audio Capture    → cpal 0.15
VAD              → Silero VAD (ONNX) / whisper-rs built-in VAD
STT Engine       → whisper-rs (whisper.cpp bindings) + Thonburian Whisper GGML
LLM Correction   → Ollama REST API (MVP) / llama_cpp crate (Phase 2)
LLM Model        → Qwen2.5-3B GGUF Q4 (default) / Typhoon2-Qwen2.5-7B (GPU)
Context Manager  → Rolling window 3 sentences + Domain detection + User vocab
UI Widget        → egui 0.31 + eframe (always-on-top, transparent, borderless)
Global Hotkey    → rdev 0.5 (push-to-talk)
Text Injection   → arboard 3.4 (clipboard) + enigo 0.3 (Ctrl+V simulation)
Async Runtime    → tokio
Channels         → tokio::sync::mpsc / crossbeam-channel
```

---

## Git Workflow Rules (MANDATORY)

**ทุก task ที่เขียนโค้ดหรือแก้ไขไฟล์ ต้องทำตามนี้เสมอ — ไม่มีข้อยกเว้น**

### Before Starting Any Task
1. **Branch** — ต้องทำงานบน branch ที่ถูกต้อง ไม่ push direct ไป `main`
   - ดูรูปแบบชื่อ branch: `git-workflow/branching-strategy.md` §2
   - Format: `feat/<module>-<desc>` | `fix/<module>-<desc>` | `docs/<desc>` | `chore/<desc>`

### Every Commit
2. **Conventional Commits** — ทุก commit ต้องเป็น format `<type>(<scope>): <description>`
   - ดู types และ scopes: `git-workflow/commit-conventions.md` §2-3
   - Scopes: `audio` | `stt` | `llm` | `inject` | `hotkey` | `pipeline` | `config` | `ui` | `docs`
   - Example: `feat(audio): add 16kHz resampling` | `fix(stt): handle empty transcription`

### Before Opening PR
3. **PR Checklist** — ต้องผ่านทุกข้อก่อน open PR
   - `cargo build` ✅ | `cargo test` ✅ | `cargo clippy -- -D warnings` ✅ | `cargo fmt --check` ✅
   - ดู checklist เต็ม: `git-workflow/pr-workflow.md` §4
4. **PR Description** — ต้องใส่ Summary + Design Reference ทุกครั้ง
   - ดู template: `git-workflow/pr-workflow.md` §3

### Module Isolation (Multi-Agent)
5. **แต่ละ agent แตะเฉพาะ module ของตัวเอง** — ห้ามแก้ไข module อื่น
   - ดู isolation map: `git-workflow/multi-agent-workflow.md` §2
   - Merge order: `config → pipeline → audio → stt → llm → inject → hotkey → ui`

### Quick Reference
| ต้องการทำอะไร | อ่านไฟล์นี้ |
|---|---|
| สร้าง branch | `git-workflow/branching-strategy.md` §2 |
| เขียน commit message | `git-workflow/commit-conventions.md` §1-3 |
| เปิด PR | `git-workflow/pr-workflow.md` §3-4 |
| Release / Tag | `git-workflow/release-process.md` §3,5 |
| Hotfix | `git-workflow/release-process.md` §6 |
| Multi-agent parallel work | `git-workflow/multi-agent-workflow.md` §3-4 |

---

## Project Rules

### Folder Structure
```
voice-to-text/
├── CLAUDE.md
└── docs/
    ├── DOCUMENT-ROUTER.md  ← index of all docs → which file to read per task
    ├── research/           ← งานวิจัย, benchmark, competitor analysis
    ├── designs/            ← UI/UX wireframes, specs, design decisions
    ├── plan/               ← roadmap, milestones, sprint plans
    └── git-workflow/       ← branching strategy, commit conventions
```

### Naming Conventions
| Type | Format | Example |
|------|--------|---------|
| Markdown files | `kebab-case.md` | `stt-engine-design.md` |
| Rust files | `snake_case.rs` | `audio_capture.rs` |
| Folders (docs) | `kebab-case/` | `docs/research/` |
| Folders (Rust) | `snake_case/` | `src/audio/` |

### Filing Rules
- ทุก folder ต้องมี `INDEX.md` เป็นสารบัญ
- ไม่สร้าง folder ใหม่โดยไม่อัปเดต CLAUDE.md ก่อน
- `docs/research/` — เฉพาะงานวิจัย, benchmark, competitor analysis
- `docs/designs/` — เฉพาะ wireframes, UI specs, design decisions
- `docs/plan/` — เฉพาะ roadmap, milestones, sprint plans
- `docs/git-workflow/` — เฉพาะ branching strategy, commit conventions, PR workflow, release process, multi-agent workflow
- ไฟล์ใน `models/` ต้อง gitignore เสมอ (ขนาดใหญ่)

---

## Deep Search

ถ้า `docs/DOCUMENT-ROUTER.md` ไม่ตรงกับ task — grep knowledge map ระดับ section:
```
Grep pattern="<keyword>" path="memory/knowledge-map.md"
```
สำหรับ keyword quick-lookup → `MEMORY.md` (auto-loaded)
