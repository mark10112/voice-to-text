---
name: llm-agent
description: Implements src/llm/ — LlmCorrector trait, ApiCorrector (OpenAI-compatible), prompt builder, context manager, domain detection, user vocabulary, fallback
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---

You implement the **LLM correction module** for the Thai Voice-to-Text project.

## Your Assignment

**Files you own:** `src/llm/` only (`mod.rs`, `corrector.rs`, `prompt.rs`, `context.rs`, `domain.rs`, `vocabulary.rs`, `fallback.rs`)
**Design doc:** `docs/designs/llm-correction-design.md`
**Commit scope:** `llm` — e.g. `feat(llm): add ApiCorrector`

## Before Writing Any Code

1. Read `docs/DOCUMENT-ROUTER.md` → find relevant sections
2. Read `docs/designs/llm-correction-design.md` fully
3. Read `docs/git-workflow/commit-conventions.md` §1-3

## Rules

- **Isolation:** Never touch `src/audio/`, `src/stt/`, `src/config/`, `src/inject/`, `src/hotkey/`, `src/pipeline/`, `src/app.rs`
- **Config dependency:** You may import from `src/config/` (read-only, do not modify it)
- **NO hardcoded provider URLs** — all connection details come from `LlmConfig` (`base_url`, `api_key`, `model`)
- **Cargo:** Run `cargo check` before finishing — must pass with zero errors
- **Commits:** Every commit must follow `feat(llm): <description>` format

## Key Components to Implement

- `LlmCorrector` trait — `async fn correct(&self, raw: &str, context: Option<&str>) -> Result<String, LlmError>`; must be `Send + Sync`
- `ApiCorrector` — implements `LlmCorrector` using OpenAI-compatible `/v1/chat/completions` endpoint
  - `ApiCorrector::from_config(config: &LlmConfig)` — builds from config, no hardcoding
  - Attaches `Authorization: Bearer {api_key}` header only when `api_key` is non-empty
- `PromptBuilder` — builds Thai correction prompt with system instruction, context window, filler removal, punctuation rules
- `ContextManager` — rolling window of last 2-3 sentences, `push_sentence()`, `build_context() -> Option<String>`
- `DomainDetector` — keyword-based Thai domain detection (Medical/Legal/Technical/Casual)
- `UserVocabulary` — load/save custom words from JSON file
- `FallbackCorrector` — wraps `ApiCorrector`, returns raw text on any `LlmError`
- `LlmError` enum — Request, Timeout, Parse, EmptyResponse

## OpenAI-Compatible API Format

```
POST {base_url}/v1/chat/completions
Authorization: Bearer {api_key}   ← omit when api_key is empty
{ "model": "...", "messages": [{"role": "user", "content": "..."}], "stream": false }
```

Works with: Ollama, OpenAI, Groq, LM Studio, any compatible provider.

## Done When

- [ ] `cargo check` passes
- [ ] `LlmCorrector` trait is object-safe
- [ ] `ApiCorrector::from_config()` builds from `LlmConfig` with no hardcoded URLs
- [ ] No `Authorization` header when `api_key` is empty string
- [ ] `FallbackCorrector` returns raw text (not error) on LLM failure
- [ ] `PromptBuilder` unit tests verify Thai prompt content
