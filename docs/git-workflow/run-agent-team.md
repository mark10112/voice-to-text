# Run Agent Team — Step-by-Step

**วันที่:** 28 กุมภาพันธ์ 2026

คู่มือ copy-paste สำหรับรัน Phase 1 MVP ด้วย Claude Code Agent Teams + Worktrees

---

## Prerequisites

ตรวจสอบก่อนเริ่ม:

```bash
# 1. อยู่ใน project root
cd /path/to/voice-to-text

# 2. ใน main branch และ clean
git status          # ต้องไม่มี uncommitted changes
git branch          # ต้องอยู่บน main

# 3. Cargo.toml มี dependencies ครบ
cat Cargo.toml

# 4. Agent files พร้อม
ls .claude/agents/
# ต้องเห็น: config-agent.md  audio-agent.md  stt-agent.md
#           llm-agent.md     inject-agent.md  pipeline-agent.md  ui-agent.md

# 5. settings.json enable agent teams แล้ว
cat .claude/settings.json
# ต้องเห็น: "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
```

---

## Step 1 — Start Claude Code

```bash
cd /path/to/voice-to-text
claude
```

---

## Step 2 — Run config-agent (ALONE FIRST)

> **ทำไมก่อน:** ทุก module import จาก `src/config/` — ต้อง merge ก่อนเสมอ

คัดลอก prompt นี้ไปวางใน Claude:

```
Use the config-agent to implement src/config/.

The agent should:
1. Read docs/DOCUMENT-ROUTER.md to find the design doc
2. Read docs/designs/configuration-and-modes.md
3. Implement all structs: AppConfig, LlmConfig, SttConfig, AudioConfig, HotkeyConfig, UiConfig, AppPaths, OperatingMode
4. Run cargo check before finishing
5. Commit with: feat(config): <description>

Work in a worktree. Report back when cargo check passes.
```

**รอจนกว่า config-agent รายงานว่าเสร็จและ `cargo check` ผ่าน**

---

## Step 3 — Merge config branch

```bash
# ดู worktree ที่สร้างขึ้น
git worktree list

# Merge config เข้า main (squash)
git switch main
git merge --squash worktree-feat-config-settings
git commit -m "feat(config): add AppConfig, LlmConfig, paths, OperatingMode"

# ลบ worktree
git worktree remove .claude/worktrees/feat-config-settings
git branch -d worktree-feat-config-settings
```

---

## Step 4 — Run 4 agents in parallel

> ทุกตัวรัน **พร้อมกัน** — ไม่ต้องรอกัน เพราะต่างก็ depend แค่ config ซึ่งมีแล้ว

คัดลอก prompt นี้ไปวางใน Claude:

```
Now run these 4 agents in parallel, each in their own worktree:

1. audio-agent — implements src/audio/ (cpal capture, ring buffer, VAD, resampling, waveform)
2. stt-agent   — implements src/stt/  (SttEngine trait, WhisperEngine, model loading)
3. llm-agent   — implements src/llm/  (LlmCorrector trait, ApiCorrector, prompt builder, context, domain, vocabulary)
4. inject-agent — implements src/inject/ and src/hotkey/ (clipboard injection, enigo Ctrl+V, rdev hotkey)

Each agent must:
- Read docs/DOCUMENT-ROUTER.md first
- Read their assigned design doc
- Run cargo check before finishing
- Commit with their correct scope (audio/stt/llm/inject/hotkey)

Run all 4 at the same time. Report when each finishes.
```

**รอจนกว่าทุก agent รายงาน `cargo check` ผ่าน**

---

## Step 5 — Merge 4 parallel branches

Merge ตามลำดับนี้ (ป้องกัน conflict):

```bash
git switch main

# 1. Audio
git merge --squash worktree-feat-audio-pipeline
git commit -m "feat(audio): add cpal capture, ring buffer, VAD, resampling"
git worktree remove .claude/worktrees/feat-audio-pipeline
git branch -d worktree-feat-audio-pipeline

# 2. STT
git merge --squash worktree-feat-stt-whisper-engine
git commit -m "feat(stt): add SttEngine trait, WhisperEngine, model loading"
git worktree remove .claude/worktrees/feat-stt-whisper-engine
git branch -d worktree-feat-stt-whisper-engine

# 3. LLM
git merge --squash worktree-feat-llm-api-corrector
git commit -m "feat(llm): add LlmCorrector trait, ApiCorrector, prompt builder"
git worktree remove .claude/worktrees/feat-llm-api-corrector
git branch -d worktree-feat-llm-api-corrector

# 4. Inject + Hotkey
git merge --squash worktree-feat-inject-hotkey
git commit -m "feat(inject): add clipboard injection, enigo paste, rdev hotkey"
git worktree remove .claude/worktrees/feat-inject-hotkey
git branch -d worktree-feat-inject-hotkey

# ตรวจสอบ
cargo check
```

---

## Step 6 — Run pipeline-agent

> **ทำไมต้องรอ:** pipeline import จากทุก module — ต้อง merge ทั้งหมดก่อน

```
Use the pipeline-agent to implement src/pipeline/.

The agent should:
1. Read docs/DOCUMENT-ROUTER.md first
2. Read docs/designs/threading-and-data-flow.md fully
3. Read docs/designs/architecture-overview.md §4-6
4. Implement: PipelineState, AppState, SharedState, PipelineOrchestrator
5. Wire: audio → STT (spawn_blocking) → LLM (async, Standard/Context mode only) → inject (spawn_blocking)
6. LLM must be fully wired — not a stub. Fallback to raw text on LLM error.
7. Run cargo check before finishing
8. Commit with: feat(pipeline): <description>

Work in a worktree. Report when done.
```

**รอจนกว่า pipeline-agent รายงานว่าเสร็จ**

---

## Step 7 — Merge pipeline branch

```bash
git switch main
git merge --squash worktree-feat-pipeline-orchestrator
git commit -m "feat(pipeline): add PipelineOrchestrator, wire audio→STT→LLM→inject"
git worktree remove .claude/worktrees/feat-pipeline-orchestrator
git branch -d worktree-feat-pipeline-orchestrator

cargo check
```

---

## Step 8 — Run ui-agent (LAST)

> **ทำไมต้องสุดท้าย:** ui import จากทุก module รวมถึง pipeline

```
Use the ui-agent to implement src/app.rs and src/main.rs.

The agent should:
1. Read docs/DOCUMENT-ROUTER.md first
2. Read docs/designs/ui-widget-design.md fully
3. Read docs/designs/threading-and-data-flow.md §6 (startup sequence)
4. Implement src/main.rs — startup sequence: load config → create runtime → load Whisper → build ApiCorrector → spawn PipelineOrchestrator → run eframe
5. Implement src/app.rs — ThaiSttApp: egui always-on-top floating widget, renders all PipelineState variants
6. Run cargo check before finishing
7. Commit with: feat(ui): <description>

Work in a worktree. Report when done.
```

---

## Step 9 — Merge ui branch + final check

```bash
git switch main
git merge --squash worktree-feat-ui-widget
git commit -m "feat(ui): add egui floating widget, push-to-talk UI, waveform display"
git worktree remove .claude/worktrees/feat-ui-widget
git branch -d worktree-feat-ui-widget

# Final verification
cargo check
cargo test
cargo clippy -- -D warnings
```

---

## Step 10 — Tag Phase 1

```bash
git log --oneline -10   # review commits

git tag -a v0.1.0 -m "Phase 1 MVP: Audio + Thonburian Whisper STT + egui widget + hotkey + text injection"
git push origin main --tags
```

---

## Troubleshooting

| ปัญหา | วิธีแก้ |
|-------|---------|
| Agent ไม่รู้จัก worktree ชื่ออะไร | `git worktree list` — ดูชื่อจริง |
| Merge conflict ใน Cargo.toml | เปิดไฟล์ เก็บ dependencies ทั้งหมด แล้ว `cargo check` |
| Merge conflict ใน src/main.rs | ui-agent เป็น owner — ใช้ version ของ ui-agent |
| `cargo check` ยังไม่ผ่านหลัง merge | ดู error → หา agent ที่รับผิดชอบ module นั้น → แก้ใน branch ก่อน merge |
| Agent หยุดกลางคัน | สั่ง agent ใหม่ด้วย: "Continue where you left off in the worktree" |

---

## Summary

```
[Step 1] claude (เปิด session)
[Step 2] config-agent ───────────────────────────────► merge (Step 3)
[Step 4] audio-agent  ─┐
         stt-agent    ─┼─ parallel ───────────────────► merge (Step 5)
         llm-agent    ─┤
         inject-agent ─┘
[Step 6] pipeline-agent ────────────────────────────► merge (Step 7)
[Step 8] ui-agent ──────────────────────────────────► merge (Step 9)
[Step 10] tag v0.1.0
```
