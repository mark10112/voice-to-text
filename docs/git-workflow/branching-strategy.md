# Branching Strategy

**วันที่:** 28 กุมภาพันธ์ 2026

---

## 1. Overview

ใช้ **GitHub Flow** variant: `main` เป็น branch เดียวที่ stable เสมอ
ไม่มี `develop` branch (solo/small team ไม่จำเป็น)
Feature branches แตกจาก `main` → merge กลับผ่าน PR (squash merge)

```
main ─────────────────────────────────────────────────────────►
  │          ▲         ▲              ▲              ▲
  │          │         │              │              │
  └─ feat ───┘  └─ fix ┘   └─ chore ─┘   └─ docs ──┘
```

---

## 2. Branch Types & Naming Convention

| Type | Pattern | Example | Lifetime |
|------|---------|---------|----------|
| Feature | `feat/<module>-<short-desc>` | `feat/audio-ring-buffer` | Short (≤1 week) |
| Phase | `phase/<N>-<name>` | `phase/2-llm-integration` | Long (entire phase) |
| Bug fix | `fix/<module>-<short-desc>` | `fix/stt-empty-transcription` | Short (≤3 days) |
| Hotfix | `hotfix/<short-desc>` | `hotfix/crash-on-startup` | Very short (hours) |
| Chore | `chore/<desc>` | `chore/update-cpal-deps` | Short |
| Docs | `docs/<desc>` | `docs/llm-correction-design` | Short |
| Release | `release/v<M>.<m>.<p>` | `release/v0.2.0` | Short (release prep only) |

### Naming Rules
- ใช้ lowercase เสมอ
- แยกคำด้วย `-` (kebab-case)
- `<module>` = `audio` | `stt` | `llm` | `inject` | `hotkey` | `pipeline` | `config` | `ui`
- `<short-desc>` ≤ 4 คำ กระชับ อธิบาย feature ที่ทำ
- ห้ามใช้ชื่อ generic เช่น `feat/changes`, `fix/bug`, `feat/update`

---

## 3. Branch Lifetime Rules

| Type | Max Lifetime | Action if Longer |
|------|-------------|-----------------|
| `feat/*` | 1 week | Split into smaller PRs |
| `fix/*` | 3 days | Raise priority, pair review |
| `hotfix/*` | 24 hours | Emergency merge |
| `phase/*` | Duration of phase | Keep, delete after phase PR merged |
| `docs/*` | 3 days | — |
| `release/*` | 1-3 days | Delete after tag created |
| `chore/*` | 3 days | — |

---

## 4. Protected Branch: `main`

| Rule | Setting |
|------|---------|
| Direct push | ❌ Forbidden |
| Require PR | ✅ Always |
| CI must pass | ✅ `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt` |
| History | Linear (squash merge) |
| Signed commits | Optional (recommended) |

---

## 5. Workflow Diagram (ASCII)

### Standard Feature Flow
```
main ──────────────────────────────────────────────────────────►
  │                                                    ▲
  │ git switch -c feat/audio-vad                       │ squash merge
  ▼                                                    │
feat/audio-vad                                         │
  ├─ feat(audio): add silero vad integration           │
  ├─ feat(audio): expose vad sensitivity config        │
  ├─ test(audio): unit tests for vad silence trim      │
  └──────────────────── PR → CI → review ──────────────┘
                             │
                    ┌────────▼────────┐
                    │  cargo check    │
                    │  cargo test     │
                    │  cargo clippy   │
                    │  cargo fmt      │
                    └─────────────────┘
```

### Hotfix Flow
```
main ──────────────────────────────────────►
  │  (at v0.1.0)               ▲
  │                            │ merge + tag v0.1.1
  └─► hotfix/crash-on-startup ─┘
        ├─ fix(stt): handle null model path
        └─ (fast-track: no squash needed)
```

### Phase Branch Flow
```
main ──────────────────────────────────────────────────────────►
  │                                                    ▲
  └─► phase/2-llm-integration                         │ merge commit
        ├─ feat/llm-ollama-client → squash → phase/2   │ (preserve phase history)
        ├─ feat/llm-prompt-builder → squash → phase/2  │
        ├─ feat/pipeline-standard-mode → squash →       │
        └───────────────────────────────── PR ──────────┘
```

---

## 6. Naming Examples for This Project

### Phase 1 — MVP
```
feat/audio-cpal-capture
feat/audio-ring-buffer
feat/audio-vad-silero
feat/stt-whisper-engine
feat/stt-model-download
feat/ui-floating-widget
feat/ui-push-to-talk-button
feat/hotkey-rdev-global
feat/inject-arboard-clipboard
feat/inject-enigo-ctrlv
```

### Phase 2 — LLM Integration
```
feat/llm-ollama-client
feat/llm-prompt-thai-correction
feat/llm-filler-word-removal
feat/pipeline-standard-mode-toggle
fix/llm-timeout-fallback
```

### Phase 3 — Context & Intelligence
```
feat/llm-context-rolling-window
feat/llm-domain-detection
feat/llm-user-vocabulary
feat/llm-few-shot-examples
feat/pipeline-context-mode
```

### Phase 4 — Polish
```
feat/ui-model-selector
feat/config-toml-persistence
feat/config-first-run-wizard
chore/packaging-windows-msi
chore/packaging-macos-dmg
docs/user-guide
```

---

## 7. Multi-Agent Parallel Development

Claude Code subagents สามารถทำงานบน branches แยกกันพร้อมกัน โดยใช้ module isolation:

### Module Isolation (ป้องกัน conflicts)

| Agent | Branch | Files | ห้ามแตะ |
|-------|--------|-------|---------|
| Audio Agent | `feat/audio-*` | `src/audio/**` | `src/stt/`, `src/llm/` |
| STT Agent | `feat/stt-*` | `src/stt/**` | `src/audio/`, `src/llm/` |
| LLM Agent | `feat/llm-*` | `src/llm/**` | `src/audio/`, `src/stt/` |
| UI Agent | `feat/ui-*` | `src/app.rs`, `assets/` | `src/pipeline/` |
| Inject Agent | `feat/inject-*` | `src/inject/**`, `src/hotkey/` | `src/stt/`, `src/llm/` |
| Pipeline Agent | `feat/pipeline-*` | `src/pipeline/` | (coordinate with all) |
| Config Agent | `feat/config-*` | `src/config/` | — |
| Docs Agent | `docs/*` | `docs/**`, `CLAUDE.md` | `src/**` |

### Merge Order (Dependency Order)
```
config → pipeline → audio → stt → llm → inject → hotkey → ui
```

ดูรายละเอียดเพิ่มเติม → [`multi-agent-workflow.md`](./multi-agent-workflow.md)

---

## 8. Quick Reference

```bash
# สร้าง feature branch
git switch -c feat/<module>-<desc> main

# Push และ track remote
git push -u origin feat/<module>-<desc>

# Rebase ก่อน PR (keep history clean)
git fetch origin && git rebase origin/main

# Squash merge (orchestrator เท่านั้น)
git switch main
git merge --squash feat/<module>-<desc>
git commit -m "feat(<module>): <description>"
git branch -d feat/<module>-<desc>
git push origin --delete feat/<module>-<desc>

# List all branches
git branch -a | grep "feat\|fix\|hotfix\|phase"
```

---

## 9. Rules Summary

- `main` ต้อง build และ test ผ่านเสมอ — ห้าม push direct
- Branch names ต้องเป็น kebab-case, ขึ้นต้นด้วย type prefix
- Feature branches อายุไม่เกิน 1 สัปดาห์ — ถ้านานกว่านั้น ต้องแบ่ย
- แต่ละ branch แก้ไข module เดียว (module isolation)
- Squash merge เสมอสำหรับ feature branches → history บน `main` สะอาด
- Hotfixes branch จาก `main` แล้ว merge กลับ main + tag ใหม่
- Phase branches: merge commit (ไม่ squash) เพื่อเก็บ phase history
