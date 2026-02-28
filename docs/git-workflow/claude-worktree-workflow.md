# Claude Code — Agent Teams & Worktree Workflow

**วันที่:** 28 กุมภาพันธ์ 2026

คู่มือการใช้ Claude Code Agent Teams + Git Worktrees เพื่อพัฒนา project นี้แบบ parallel

---

## 1. Overview

Claude Code มีสองวิธีในการทำงานแบบ parallel:

| วิธี | เหมาะกับ | Coordination |
|------|---------|-------------|
| **Manual Worktrees** (`claude --worktree`) | งานแยกอิสระ, debug, hotfix | Manual (คุณ merge เอง) |
| **Agent Teams** (experimental) | Feature sprint ทั้ง phase | Auto (agents สื่อสารกันได้) |

---

## 2. Approach A — Manual Worktrees (Simplest)

เปิด Terminal แยกต่างหากสำหรับแต่ละ module:

```bash
# Terminal 1 — Config module (ต้องทำก่อน)
claude --worktree feat/config-settings

# Terminal 2 — Audio module
claude --worktree feat/audio-pipeline

# Terminal 3 — STT module
claude --worktree feat/stt-whisper-engine

# Terminal 4 — LLM module
claude --worktree feat/llm-api-corrector
```

**ผลลัพธ์:** Claude Code สร้าง worktree ที่ `.claude/worktrees/<name>/` และ branch `worktree-<name>` โดยอัตโนมัติ

**Cleanup:**
- ถ้าไม่มีการเปลี่ยนแปลง → worktree ถูกลบอัตโนมัติเมื่อ session ปิด
- ถ้ามี changes → Claude ถามว่าจะเก็บหรือลบ

---

## 3. Approach B — Agent Teams (Parallel + Coordinated)

### 3.1 Enable Agent Teams

```bash
export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1
claude
```

หรือตั้งใน `.claude/settings.json`:

```json
{
  "env": {
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
  },
  "teammateMode": "in-process"
}
```

`teammateMode` options:
- `"in-process"` — ทุก agent แสดงใน terminal เดียว (default)
- `"tmux"` — แยก pane (ต้องมี tmux หรือ iTerm2)

### 3.2 Launch a Team Sprint

ตัวอย่าง Phase 1 MVP sprint — พิมพ์ใน Claude session:

```
Create an agent team for Phase 1 MVP implementation.

Team structure:
- config-agent: implement src/config/ — AppConfig, LlmConfig, paths
- audio-agent: implement src/audio/ — cpal capture, ring buffer, VAD, resample
- stt-agent: implement src/stt/ — WhisperEngine, model loading, transcribe params

Each agent must:
1. Read docs/DOCUMENT-ROUTER.md to find the relevant design doc
2. Read their assigned design doc before writing any code
3. Work only in their assigned module — never touch other src/ folders
4. Follow docs/git-workflow/commit-conventions.md for all commits
5. Work in a separate worktree

Merge order when done: config → audio → stt
```

---

## 4. Agent Definitions (`.claude/agents/`)

บันทึกไฟล์นี้เพื่อ reuse ได้ทุก session:

```bash
mkdir -p .claude/agents
```

### `config-agent.md`

```yaml
---
name: config-agent
description: Implements src/config/ — AppConfig, LlmConfig, paths, settings persistence
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---

You implement the config module for the Thai Voice-to-Text project.

Your files: src/config/ only
Design doc: docs/designs/configuration-and-modes.md
Commit scope: config — e.g. feat(config): add AppConfig struct

Rules:
- Read docs/DOCUMENT-ROUTER.md first, then your design doc
- Never touch src/audio/, src/stt/, src/llm/, or other modules
- Follow docs/git-workflow/commit-conventions.md for every commit
- cargo check must pass before finishing
```

### `audio-agent.md`

```yaml
---
name: audio-agent
description: Implements src/audio/ — cpal capture, ring buffer, VAD, resampling, waveform
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---

You implement the audio pipeline module for the Thai Voice-to-Text project.

Your files: src/audio/ only
Design doc: docs/designs/audio-pipeline-design.md
Commit scope: audio — e.g. feat(audio): add ring buffer

Rules:
- Read docs/DOCUMENT-ROUTER.md first, then your design doc
- Never touch src/stt/, src/llm/, src/config/, or other modules
- Follow docs/git-workflow/commit-conventions.md for every commit
- cargo check must pass before finishing
```

### `stt-agent.md`

```yaml
---
name: stt-agent
description: Implements src/stt/ — WhisperEngine, model loading, transcription params
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---

You implement the STT engine module for the Thai Voice-to-Text project.

Your files: src/stt/ only
Design doc: docs/designs/stt-engine-design.md
Commit scope: stt — e.g. feat(stt): add WhisperEngine

Rules:
- Read docs/DOCUMENT-ROUTER.md first, then your design doc
- Never touch src/audio/, src/llm/, src/config/, or other modules
- Follow docs/git-workflow/commit-conventions.md for every commit
- cargo check must pass before finishing
```

### `llm-agent.md`

```yaml
---
name: llm-agent
description: Implements src/llm/ — ApiCorrector, prompt builder, context manager, domain detection
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---

You implement the LLM correction module for the Thai Voice-to-Text project.

Your files: src/llm/ only
Design doc: docs/designs/llm-correction-design.md
Commit scope: llm — e.g. feat(llm): add ApiCorrector

Rules:
- Read docs/DOCUMENT-ROUTER.md first, then your design doc
- LLM is config-driven: use base_url + api_key from LlmConfig — never hardcode provider URLs
- Never touch src/audio/, src/stt/, src/config/, or other modules
- Follow docs/git-workflow/commit-conventions.md for every commit
- cargo check must pass before finishing
```

### `pipeline-agent.md`

```yaml
---
name: pipeline-agent
description: Implements src/pipeline/ — PipelineOrchestrator wiring audio→STT→LLM→inject
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---

You implement the pipeline orchestrator for the Thai Voice-to-Text project.

Your files: src/pipeline/ only
Design doc: docs/designs/threading-and-data-flow.md
Commit scope: pipeline — e.g. feat(pipeline): wire STT into orchestrator

Rules:
- Read docs/DOCUMENT-ROUTER.md first, then your design doc
- Pipeline calls audio, STT, and LLM modules — read their interfaces but do NOT modify them
- Must merge after config, audio, stt, llm agents are all done
- Follow docs/git-workflow/commit-conventions.md for every commit
- cargo check must pass before finishing
```

### `ui-agent.md`

```yaml
---
name: ui-agent
description: Implements src/app.rs and assets/ — egui floating widget, push-to-talk UI
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---

You implement the UI widget for the Thai Voice-to-Text project.

Your files: src/app.rs, assets/ only
Design doc: docs/designs/ui-widget-design.md
Commit scope: ui — e.g. feat(ui): add floating egui widget

Rules:
- Read docs/DOCUMENT-ROUTER.md first, then your design doc
- UI depends on pipeline state — coordinate with pipeline-agent owner before merging
- Must merge last (depends on all other modules)
- Follow docs/git-workflow/commit-conventions.md for every commit
- cargo check must pass before finishing
```

---

## 5. Phase Sprint Plan

### Phase 1 MVP — Parallel Execution

```
Step 1 (sequential — must be first):
  └── config-agent          src/config/

Step 2 (fully parallel — run all at once):
  ├── audio-agent           src/audio/
  ├── stt-agent             src/stt/
  ├── llm-agent             src/llm/
  └── inject-agent          src/inject/ + src/hotkey/

Step 3 (after Step 2 complete):
  └── pipeline-agent        src/pipeline/

Step 4 (last — depends on all):
  └── ui-agent              src/app.rs
```

**เหตุผล:** `config` ถูก import โดยทุก module → ต้อง merge ก่อนเสมอ
ดูรายละเอียด → `multi-agent-workflow.md` §4

---

## 6. Worktree File Structure

หลังจาก agents ทำงาน:

```
voice-to-text/
├── .claude/
│   ├── agents/
│   │   ├── config-agent.md
│   │   ├── audio-agent.md
│   │   ├── stt-agent.md
│   │   ├── llm-agent.md
│   │   ├── pipeline-agent.md
│   │   └── ui-agent.md
│   ├── worktrees/
│   │   ├── feat-config-settings/     ← isolated copy for config-agent
│   │   ├── feat-audio-pipeline/      ← isolated copy for audio-agent
│   │   └── feat-stt-whisper-engine/  ← isolated copy for stt-agent
│   └── settings.json
├── .git/
└── docs/
```

---

## 7. Merge Flow (After Agents Finish)

ทำตาม dependency order — `config → pipeline → audio → stt → llm → inject → hotkey → ui`:

```bash
git switch main

# 1. Config first
git merge --squash worktree-feat-config-settings
git commit -m "feat(config): add AppConfig, LlmConfig, paths"
git branch -d worktree-feat-config-settings

# 2. Parallel modules (any order, all depend on config only)
git merge --squash worktree-feat-audio-pipeline
git commit -m "feat(audio): add cpal capture, ring buffer, VAD, resampling"
git branch -d worktree-feat-audio-pipeline

git merge --squash worktree-feat-stt-whisper-engine
git commit -m "feat(stt): add WhisperEngine, Thonburian model loading"
git branch -d worktree-feat-stt-whisper-engine

git merge --squash worktree-feat-llm-api-corrector
git commit -m "feat(llm): add ApiCorrector with OpenAI-compatible endpoint"
git branch -d worktree-feat-llm-api-corrector

# 3. Pipeline (after all above merged)
git merge --squash worktree-feat-pipeline-orchestrator
git commit -m "feat(pipeline): wire audio→STT→LLM→inject orchestrator"
git branch -d worktree-feat-pipeline-orchestrator

# 4. UI last
git merge --squash worktree-feat-ui-widget
git commit -m "feat(ui): add egui floating widget with push-to-talk"
git branch -d worktree-feat-ui-widget
```

ดู merge strategy เพิ่มเติม → `pr-workflow.md` §9

---

## 8. Quick Reference

| ต้องการทำอะไร | คำสั่ง |
|---|---|
| เปิด worktree session | `claude --worktree feat/<module>-<desc>` |
| เปิด agent team | `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1 claude` |
| ดู worktrees ทั้งหมด | `git worktree list` |
| ลบ worktree เอง | `git worktree remove .claude/worktrees/<name>` |
| ดู isolation map | `multi-agent-workflow.md` §2 |
| ดู merge order | `multi-agent-workflow.md` §4 |
