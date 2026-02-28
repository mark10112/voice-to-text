# Pull Request Workflow

**วันที่:** 28 กุมภาพันธ์ 2026

---

## 1. PR Philosophy

- **Small, focused PRs** — one module, one feature per PR
- **Reviewable in <30 minutes** — if larger, split into smaller PRs
- **Always link to design doc** — PR description must reference `docs/designs/`
- **CI must pass** before requesting review
- **Draft PRs** for WIP or early feedback — don't leave draft > 3 days

---

## 2. PR Title Format

Follow conventional commits exactly:

```
feat(audio): add silero vad integration
fix(stt): handle empty transcription result
refactor(pipeline): extract state machine to enum
docs(llm): update ollama integration design
```

---

## 3. PR Description Template

```markdown
## Summary
- <!-- What does this PR do? 1-3 bullet points -->
-
-

## Module(s) Affected
<!-- audio / stt / llm / inject / hotkey / pipeline / config / ui -->

## Design Reference
<!-- Link to relevant section: docs/designs/<file>.md §N Section Name -->

## Changes Made
<!-- Bullet list of files changed and what was done -->

## Test Coverage
- [ ] Unit tests added/updated
- [ ] Tested manually on: Windows / macOS / Linux
- [ ] Edge cases covered: <!-- list them -->

## Platform Testing
| Platform | Tested | Notes |
|----------|--------|-------|
| Windows 11 | ✅ / ❌ / — | |
| macOS | ✅ / ❌ / — | |
| Linux X11 | ✅ / ❌ / — | |

## Breaking Changes
<!-- None / Describe what breaks and migration path -->

## Screenshots / Recordings
<!-- For UI changes only — attach screenshot or screen recording -->
```

---

## 4. PR Checklist (Author)

Before opening PR, verify all of these:

- [ ] `cargo build` passes without warnings
- [ ] `cargo test` — all tests pass
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo fmt --check` — code is formatted
- [ ] No `.gguf`, `.bin`, `.onnx` model files committed (check `.gitignore`)
- [ ] No hardcoded Windows paths (use `dirs` crate or `config::paths`)
- [ ] PR title follows conventional commits format
- [ ] PR description filled with Summary + Design Reference
- [ ] Branch rebased on latest `main` (`git rebase origin/main`)
- [ ] Branch name follows naming convention (`feat/<module>-<desc>`)
- [ ] Commits are atomic — each commit compiles and makes sense alone
- [ ] No `println!` debug statements left in production code (use `log::debug!`)
- [ ] Thai text is valid UTF-8 (no corrupted string literals)
- [ ] PR touches only one module (or has clear justification)
- [ ] CHANGELOG.md updated if this is a user-facing change

---

## 5. Review Checklist (Reviewer)

When reviewing a PR:

- [ ] **Correctness** — Does the implementation match the design doc?
- [ ] **Error handling** — Errors propagated with `?`, not silently swallowed
- [ ] **Thread safety** — `Arc<Mutex<T>>` used correctly, no deadlock risk
- [ ] **Thai text safety** — Thai strings handled as UTF-8, not bytes
- [ ] **No hardcoded paths** — Uses `dirs` / `config::paths` for platform paths
- [ ] **No blocking in async** — No `std::thread::sleep` inside `tokio::spawn`
- [ ] **Memory** — No unbounded Vec growth; ring buffers have max capacity
- [ ] **Trait consistency** — New code implements existing traits (`SttEngine`, `LlmCorrector`, etc.)
- [ ] **Test coverage** — At least one unit test for new public functions
- [ ] **Design alignment** — Implementation matches `docs/designs/` spec

---

## 6. PR Size Guidelines

| Size | Lines Changed | Guideline |
|------|--------------|-----------|
| XS | < 50 | Merge same day; minimal review |
| S | 50–200 | Review within 24 hours |
| M | 200–500 | Requires design doc reference in PR |
| L | 500–1000 | Split into 2+ smaller PRs strongly preferred |
| XL | > 1000 | Must split — will be rejected without justification |

**Exception:** Cargo.lock changes don't count toward line total.

---

## 7. Multi-Agent PR Strategy

When multiple Claude Code agents work in parallel, each agent opens its own PR:

### Merge Order (follow dependency order)
```
docs/* → config/* → pipeline/* → audio/* → stt/* → llm/* → inject/* → hotkey/* → ui/*
```

### Conflict Prevention
- Each agent branch touches only its assigned module files (see `branching-strategy.md §7`)
- `Cargo.toml` conflicts: manually reconcile dependency versions, then `cargo check`
- `src/main.rs` conflicts: UI Agent is the owner — coordinate before merging

### Multi-Agent PR Example (Phase 2 Sprint)
```
PR #1  feat(llm): add ollama client        ← Agent A, no deps
PR #2  feat(pipeline): standard mode       ← Agent B, depends on PR #1
PR #3  docs(llm): update design notes      ← Agent C, no deps

Merge order: PR #3 → PR #1 → PR #2
```

---

## 8. GitHub Actions CI

บันทึกเป็น `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: Check & Test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: cargo check
        run: cargo check --all-features

      - name: cargo test
        run: cargo test --all-features

      - name: cargo clippy
        run: cargo clippy --all-features -- -D warnings

      - name: cargo fmt
        run: cargo fmt --check
```

---

## 9. Merge Strategy

| Branch Type | Merge Strategy | Reason |
|-------------|---------------|--------|
| `feat/*` → `main` | **Squash merge** | Clean linear history on `main` |
| `fix/*` → `main` | **Squash merge** | Single atomic fix commit |
| `hotfix/*` → `main` | **Merge commit** | Preserve hotfix context |
| `phase/*` → `main` | **Merge commit** | Preserve phase development history |
| `docs/*` → `main` | **Squash merge** | Single doc update commit |

**After merge: always delete the source branch.**

```bash
# Squash merge (recommended for features)
git switch main
git merge --squash feat/audio-vad
git commit -m "feat(audio): add silero vad integration"
git branch -d feat/audio-vad
git push origin --delete feat/audio-vad
```

---

## 10. Draft PRs

เปิด Draft PR เมื่อ:
- งานยัง WIP แต่อยากให้ CI ทำงาน
- ต้องการ early feedback ก่อน implementation เสร็จ
- Branch blocked by ส่วนอื่น (ใส่ `Blocked by #<PR>` ใน description)

**ข้อควรระวัง:**
- Draft PR อายุไม่เกิน 3 วัน ถ้านานกว่านั้น → close และเปิดใหม่เมื่อพร้อม
- ห้าม mark as Ready for Review ถ้า CI ยังไม่ผ่าน
