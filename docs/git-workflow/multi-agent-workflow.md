# Multi-Agent Git Workflow

**วันที่:** 28 กุมภาพันธ์ 2026

คู่มือการใช้ Claude Code subagents หลายตัวพัฒนาพร้อมกัน บน isolated git branches

---

## 1. Why Multi-Agent Git Workflow

Claude Code สามารถ spawn subagents หลายตัวพร้อมกัน — แต่ละตัวทำงานบน feature branch ของตัวเอง
ผลลัพธ์: งานที่ปกติใช้เวลาหลายชั่วโมงอาจเสร็จใน 1 session เดียว

**ข้อดี:**
- Agents ทำงานแบบ parallel แทน sequential
- Module isolation ป้องกัน merge conflicts
- แต่ละ agent มี context ชัดเจน ไม่รบกวนกัน
- Orchestrator (main agent) review และ merge ตาม dependency order

---

## 2. Module Isolation Map

แต่ละ agent ครอบครอง files ของตัวเอง — ไม่แตะ files ของ agent อื่น:

| Agent | Branch Pattern | Files Owned | Shared Files (coordinate) |
|-------|---------------|-------------|--------------------------|
| **Audio Agent** | `feat/audio-*` | `src/audio/**` | `Cargo.toml` (audio deps) |
| **STT Agent** | `feat/stt-*` | `src/stt/**` | `Cargo.toml` (whisper-rs) |
| **LLM Agent** | `feat/llm-*` | `src/llm/**` | `Cargo.toml` (reqwest, ollama) |
| **UI Agent** | `feat/ui-*` | `src/app.rs`, `assets/**` | `src/main.rs` |
| **Inject Agent** | `feat/inject-*` | `src/inject/**`, `src/hotkey/**` | `Cargo.toml` (arboard, enigo) |
| **Pipeline Agent** | `feat/pipeline-*` | `src/pipeline/**` | `src/main.rs` |
| **Config Agent** | `feat/config-*` | `src/config/**` | `Cargo.toml` |
| **Docs Agent** | `docs/*` | `docs/**`, `CLAUDE.md` | — |

> **Rule:** ถ้า agent ต้องแก้ `Cargo.toml` หรือ `src/main.rs` ต้อง coordinate ก่อน merge

---

## 3. Parallel Session Protocol

### Step-by-Step

```
1. ORCHESTRATOR: วิเคราะห์ task → แบ่ง subtasks ตาม module
2. ORCHESTRATOR: สร้าง branches สำหรับแต่ละ agent
   git switch -c feat/audio-vad main
   git switch -c feat/stt-model-cache main
   git switch -c feat/llm-ollama-client main

3. AGENTS (parallel): แต่ละ agent ทำงานบน branch ของตัวเอง
   - อ่าน design doc ของ module
   - implement
   - commit ด้วย conventional commits
   - push branch

4. AGENTS: เปิด PR เมื่อเสร็จ
   - ใส่ PR description ตาม template
   - ระบุ module ที่แก้
   - link ไปยัง design doc

5. ORCHESTRATOR: review PRs ตาม dependency order
   config → pipeline → audio → stt → llm → inject → ui

6. ORCHESTRATOR: merge (squash) และ delete branches
```

---

## 4. Dependency Merge Order

```
config ──► pipeline ──► audio ──► stt ──► llm ──► inject ──► hotkey ──► ui
  │           │           │         │        │         │          │        │
  │         (uses       (uses     (uses    (uses     (uses      (uses    (uses
  │         config)    config)   audio)   config)   config)   config)   all)
  │
  └── Always merge first (other modules import from config)
```

**เหตุผล:**
- `config` — ทุก module import `AppConfig`, `AppPaths`; ต้อง merge ก่อนเสมอ
- `pipeline` — orchestrates ทุก module; ต้องรู้ interfaces ก่อน implement
- `audio/stt/llm/inject` — independent แต่ต้อง config พร้อมก่อน
- `ui` — depend on ทุก module; merge สุดท้าย

---

## 5. Conflict Resolution

### Cargo.toml Conflicts
```bash
# เมื่อหลาย agents แก้ Cargo.toml พร้อมกัน:
git fetch origin main
git rebase origin/main

# แก้ conflict ใน [dependencies] section ด้วยมือ:
# รักษา version ที่สูงกว่า หรือ compatible range
# ตรวจสอบด้วย: cargo check
```

### src/main.rs หรือ src/app.rs Conflicts
- UI Agent เป็น owner หลัก
- Agents อื่นที่ต้องแก้ `main.rs` → coordinate ผ่าน PR description ก่อน
- ถ้า conflict: UI Agent resolves เป็น final say

### General Conflict Resolution Steps
1. `git fetch origin main`
2. `git rebase origin/main` (ดีกว่า merge สำหรับ feature branches)
3. Fix conflicts — อย่า drop code ของ agent อื่น โดยไม่ตรวจสอบ
4. `cargo check && cargo test` ก่อน `git rebase --continue`
5. Force push branch: `git push --force-with-lease origin feat/my-branch`

---

## 6. Example: Phase 2 Parallel Sprint

สมมติว่าต้องการทำ Phase 2 LLM Integration — 3 agents ทำงานพร้อมกัน:

```
main
  │
  ├──► feat/llm-ollama-client    (Agent A)
  │     Files: src/llm/corrector.rs
  │     Task: Implement Ollama REST API client
  │     Commits:
  │       feat(llm): add ollama client struct
  │       feat(llm): implement correct() with reqwest
  │       test(llm): mock ollama server in tests
  │
  ├──► feat/pipeline-llm-integration    (Agent B)
  │     Files: src/pipeline/mod.rs
  │     Task: Wire LLM corrector into pipeline orchestrator
  │     Commits:
  │       feat(pipeline): add llm correction step
  │       feat(pipeline): implement Standard Mode toggle
  │       fix(pipeline): handle llm timeout gracefully
  │
  └──► docs/llm-correction-update    (Agent C)
        Files: docs/designs/llm-correction-design.md
        Task: Update design doc with implementation notes
        Commits:
          docs(llm): document ollama client design decisions
          docs(llm): add performance budget section
```

**Merge order:** Agent C (docs, no dep) → Agent A (llm client) → Agent B (pipeline, depends on llm)

---

## 7. Git Commands for Multi-Agent Setup

```bash
# Orchestrator: สร้าง branches สำหรับ agents
git switch main
git pull origin main

git switch -c feat/llm-ollama-client
git push -u origin feat/llm-ollama-client
git switch main

git switch -c feat/pipeline-llm-integration
git push -u origin feat/pipeline-llm-integration
git switch main

git switch -c docs/llm-correction-update
git push -u origin docs/llm-correction-update
git switch main

# ───── Agents work in parallel ─────

# After agents push: review and merge in order
# 1. Merge docs (no deps)
git switch main
git merge --squash docs/llm-correction-update
git commit -m "docs(llm): update correction design with ollama client details"
git branch -d docs/llm-correction-update

# 2. Merge llm client
git merge --squash feat/llm-ollama-client
git commit -m "feat(llm): implement ollama rest api client"
git branch -d feat/llm-ollama-client

# 3. Merge pipeline (last, depends on llm)
git merge --squash feat/pipeline-llm-integration
git commit -m "feat(pipeline): integrate llm correction, add standard mode"
git branch -d feat/pipeline-llm-integration
```

---

## 8. Agent Briefing Template

เมื่อ launch subagent ให้ใส่ข้อมูลนี้ใน prompt:

```
You are working on branch: feat/<module>-<desc>
Your assigned files: src/<module>/
DO NOT touch: src/<other-modules>/, Cargo.toml (unless adding deps for your module only)

Design reference: docs/designs/<module>-design.md §<section>
Commit convention: feat(<module>): <imperative description>

When done:
1. Run: cargo check && cargo test
2. Commit all changes with conventional commits
3. Report: list of files changed + summary of what was implemented
```

---

## 9. Quick Reference

```bash
# สร้าง branch สำหรับ agent
git switch -c feat/<module>-<desc> main

# Rebase ก่อน merge (keep history clean)
git fetch origin && git rebase origin/main

# Squash merge (clean history on main)
git switch main
git merge --squash feat/<module>-<desc>
git commit -m "feat(<module>): <description>"

# Delete branch after merge
git branch -d feat/<module>-<desc>
git push origin --delete feat/<module>-<desc>
```
