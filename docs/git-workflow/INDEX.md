# Git Workflow — Index

**วันที่:** 28 กุมภาพันธ์ 2026

---

## Files in This Folder

| File | Description | Key Sections |
|------|-------------|-------------|
| `branching-strategy.md` | Branch types, naming convention, multi-agent isolation | Branch types, naming rules, ASCII flow, module isolation |
| `commit-conventions.md` | Conventional commits adapted for this project | Types, scopes, 20+ examples, git hook validator |
| `pr-workflow.md` | PR template, review checklist, CI config, merge strategy | PR template, checklist, GitHub Actions YAML |
| `release-process.md` | Versioning, phase→version map, CHANGELOG, hotfix flow | Semver map, release steps, platform artifacts |
| `multi-agent-workflow.md` | Parallel Claude Code subagent development protocol | Module isolation, merge order, parallel sprint example |
| `claude-worktree-workflow.md` | Claude Code Agent Teams + Git Worktrees setup guide | Agent definitions, phase sprint plan, merge flow |
| `run-agent-team.md` | Step-by-step runbook for Phase 1 MVP agent team sprint | Copy-paste prompts, merge commands, troubleshooting |

---

## Quick Start — 5 Most Important Rules

1. **Branch naming:** `<type>/<module>-<short-desc>` — e.g., `feat/audio-ring-buffer`, `fix/stt-empty-transcription`
2. **Commit format:** Conventional commits — `feat(audio): add 16kHz resampling` (imperative, ≤72 chars)
3. **PR size:** Keep PRs to one module, ≤500 lines changed; anything larger → split
4. **Merge strategy:** Squash merge feature branches into `main` (clean linear history)
5. **Tagging:** Annotated tags only — `git tag -a v0.1.0 -m "Phase 1 MVP release"`

---

## Workflow at a Glance

```
  main ─────────────────────────────────────────────────────────►
         │                                              ▲
         │ git switch -c feat/audio-ring-buffer         │ squash merge
         │                                              │
         ▼                                              │
  feat/audio-ring-buffer                               │
         │                                              │
         ├─ commit: feat(audio): add f32 ring buffer    │
         ├─ commit: feat(audio): add VAD integration    │
         ├─ commit: test(audio): unit tests for buffer  │
         │                                              │
         └──── open PR ──► CI passes ──► review ────────┘
                              │
                      ┌───────▼──────┐
                      │ cargo check  │
                      │ cargo test   │
                      │ cargo clippy │
                      │ cargo fmt    │
                      └─────────────┘

  Hotfix flow:
  main ──► git switch -c hotfix/stt-crash ──► fix ──► tag v0.1.1 ──► merge ──► main
```

---

## Phase → Branch → Version Map

| Phase | Long-lived Branch | Release Tag | Status |
|-------|------------------|-------------|--------|
| Phase 1 MVP | `phase/1-mvp` | `v0.1.0` | Core pipeline |
| Phase 2 LLM | `phase/2-llm-integration` | `v0.2.0` | Ollama + correction |
| Phase 3 Context | `phase/3-context-intelligence` | `v0.3.0` | Rolling window + domain |
| Phase 4 Polish | `phase/4-polish-distribution` | `v1.0.0` | Packaging + installer |
