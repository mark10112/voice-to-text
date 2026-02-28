# Architecture Overview — Thai Voice-to-Text

**วันที่:** 28 กุมภาพันธ์ 2026

---

## 1. System Context Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        User's Desktop                           │
│                                                                 │
│   ┌──────────┐    ┌──────────────────────┐    ┌──────────────┐ │
│   │  Active   │◀───│  Voice-to-Text Widget│◀───│  Microphone  │ │
│   │  Window   │    │  (Rust binary)       │    │              │ │
│   │ (any app) │    └──────────────────────┘    └──────────────┘ │
│   └──────────┘              │                                   │
│                             │ (optional)                        │
│                    ┌────────▼────────┐                          │
│                    │  Ollama Server  │                          │
│                    │  (localhost)    │                          │
│                    └─────────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
```

User พูดผ่าน microphone → Widget ถอดเสียง (ภาษาที่กำหนด) → แก้ไขด้วย LLM (provider ใดก็ได้) → inject ข้อความเข้า active window

---

## 2. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Application Layer                            │
│                                                                 │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐ │
│  │   UI Module       │  │  Hotkey Module    │  │  Config      │ │
│  │   (egui/eframe)   │  │  (rdev)           │  │  Module      │ │
│  └────────┬─────────┘  └────────┬─────────┘  └──────┬───────┘ │
│           │                     │                     │         │
├───────────┼─────────────────────┼─────────────────────┼─────────┤
│           │          Pipeline Orchestrator             │         │
│           │                                           │         │
│  ┌────────▼─────────────────────▼─────────────────────▼───────┐ │
│  │                    App State (shared)                       │ │
│  │  PipelineState · RecordingState · Settings · ContextBuffer │ │
│  └────────┬──────────────┬──────────────┬─────────────────────┘ │
│           │              │              │                       │
├───────────┼──────────────┼──────────────┼───────────────────────┤
│           │       Processing Layer      │                       │
│           │              │              │                       │
│  ┌────────▼────┐  ┌──────▼──────┐  ┌───▼────────────┐         │
│  │ Audio       │  │ STT Engine  │  │ LLM Corrector  │         │
│  │ Pipeline    │  │ (whisper-rs)│  │ (Ollama /      │         │
│  │ (cpal+VAD)  │  │ lang config │  │  OpenAI API /  │         │
│  │             │  │             │  │  llama_cpp)    │         │
│  └─────────────┘  └─────────────┘  └────────────────┘         │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                     Output Layer                                │
│                                                                 │
│  ┌───────────────────┐  ┌────────────────────────────┐         │
│  │ Text Injector     │  │ Context Manager            │         │
│  │ (arboard + enigo) │  │ (rolling window + domain   │         │
│  │                   │  │  + user vocab)             │         │
│  └───────────────────┘  └────────────────────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Crate / Module Structure

```
voice-to-text/
├── Cargo.toml                 # workspace root
├── src/
│   ├── main.rs                # entry point, eframe::run_native()
│   ├── app.rs                 # eframe::App implementation, UI loop
│   │
│   ├── audio/
│   │   ├── mod.rs             # pub mod capture, vad, buffer
│   │   ├── capture.rs         # cpal microphone capture
│   │   ├── vad.rs             # Voice Activity Detection (Silero/whisper VAD)
│   │   └── buffer.rs          # Ring buffer (f32, 16kHz, mono)
│   │
│   ├── stt/
│   │   ├── mod.rs             # pub mod engine, model
│   │   ├── engine.rs          # WhisperEngine: transcribe(audio) → text
│   │   └── model.rs           # Model download, path management, GGML loading
│   │
│   ├── llm/
│   │   ├── mod.rs             # pub mod corrector, prompt, context
│   │   ├── corrector.rs       # LLM correction: correct(raw_text, context) → text
│   │   ├── prompt.rs          # Prompt template builder (Thai-specific)
│   │   └── context.rs         # ContextManager: rolling window, domain, user vocab
│   │
│   ├── inject/
│   │   ├── mod.rs             # pub mod clipboard, keyboard
│   │   ├── clipboard.rs       # arboard clipboard operations
│   │   └── keyboard.rs        # enigo Ctrl+V simulation
│   │
│   ├── hotkey/
│   │   └── mod.rs             # rdev global hotkey listener (push-to-talk)
│   │
│   ├── pipeline/
│   │   └── mod.rs             # Orchestrator: audio → STT → LLM → inject
│   │
│   └── config/
│       ├── mod.rs             # Settings struct (AppSettings, LlmProvider), serde, persistence
│       └── paths.rs           # Platform-specific config/data paths (voice-to-text/)
│
├── models/                    # .gitignore'd — GGML/GGUF model files
│   ├── ggml-thonburian-medium.bin   # Thai STT (default)
│   ├── ggml-whisper-medium.bin      # Multilingual STT (non-Thai)
│   └── qwen2.5-3b-q4.gguf          # LLM (Ollama / llama_cpp)
│
└── assets/
    └── icon.png               # App icon
```

---

## 4. Data Flow: Complete Pipeline

```
         User presses hotkey (F9 / configurable)
                        │
                        ▼
    ┌───────────────────────────────────┐
    │  1. AUDIO CAPTURE                 │
    │  cpal: microphone → f32 samples   │
    │  16kHz, mono, streaming to buffer │
    └──────────────┬────────────────────┘
                   │
         User releases hotkey
                   │
                   ▼
    ┌───────────────────────────────────┐
    │  2. VAD + PREPROCESSING           │
    │  Trim leading/trailing silence    │
    │  Validate minimum audio length    │
    │  Output: Vec<f32> (clean audio)   │
    └──────────────┬────────────────────┘
                   │
                   ▼
    ┌───────────────────────────────────┐
    │  3. STT TRANSCRIPTION             │
    │  whisper-rs: audio → raw text     │
    │  Language: from settings          │
    │  ("th" default, "auto", or any    │
    │   ISO-639-1 code)                 │
    │  Strategy: Greedy { best_of: 1 }  │
    │  Output: String (raw_text)        │
    │  ── UI shows raw_text (gray) ──   │
    └──────────────┬────────────────────┘
                   │
          ┌────────┴─── Mode check ───────────┐
          │                                    │
     Fast Mode                          Standard / Context Mode
          │                                    │
          │                                    ▼
          │              ┌───────────────────────────────────┐
          │              │  4. LLM CORRECTION                │
          │              │  Build prompt (language-aware):   │
          │              │   - System instruction (per lang) │
          │              │   - Context (prev 2-3 sentences)  │
          │              │   - Domain hint                   │
          │              │   - User vocab                    │
          │              │   - Few-shot examples (per lang)  │
          │              │   - raw_text                      │
          │              │  Send to configured LLM provider  │
          │              │  (Ollama / OpenAI API / llama_cpp)│
          │              │  Output: corrected_text           │
          │              │  ── UI replaces with corrected ── │
          │              └──────────────┬────────────────────┘
          │                             │
          └──────────┬──────────────────┘
                     │
                     ▼
    ┌───────────────────────────────────┐
    │  5. CONTEXT UPDATE                │
    │  Add corrected_text to rolling    │
    │  window. Re-detect domain.        │
    └──────────────┬────────────────────┘
                   │
                   ▼
    ┌───────────────────────────────────┐
    │  6. TEXT INJECTION                │
    │  Save clipboard → set new text    │
    │  Simulate Ctrl+V (Cmd+V on mac)  │
    │  Restore original clipboard       │
    └───────────────────────────────────┘
```

---

## 5. Key Interfaces (Traits)

```rust
/// STT Engine abstraction — swap Whisper for another engine
pub trait SttEngine: Send + Sync {
    fn transcribe(&self, audio: &[f32], sample_rate: u32) -> Result<String>;
    fn model_info(&self) -> ModelInfo;
}

/// LLM Corrector abstraction — swap Ollama for OpenAI-compatible API, llama_cpp, or cloud
pub trait LlmCorrector: Send + Sync {
    fn correct(&self, raw_text: &str, context: &CorrectionContext) -> Result<String>;
}

/// Text Injector abstraction — platform-specific
pub trait TextInjector: Send + Sync {
    fn inject(&self, text: &str) -> Result<()>;
}
```

แต่ละ trait ช่วยให้สามารถ:
- Mock ได้ใน tests
- เปลี่ยน implementation ได้โดยไม่กระทบส่วนอื่น
- รองรับ Cloud Mode ในอนาคต (implement `SttEngine` ด้วย API call)

---

## 6. State Machine: Pipeline States

```
                    ┌─────────┐
                    │  Idle   │◀──────────────────────────┐
                    └────┬────┘                           │
                         │ hotkey press                   │
                         ▼                                │
                    ┌──────────┐                          │
                    │Recording │                          │
                    └────┬─────┘                          │
                         │ hotkey release                 │
                         ▼                                │
                    ┌──────────────┐                      │
                    │Transcribing  │                      │
                    └────┬─────────┘                      │
                         │ STT complete                   │
                         ▼                                │
                    ┌──────────────┐                      │
                    │ Correcting   │ (skip in Fast Mode)  │
                    └────┬─────────┘                      │
                         │ LLM complete                   │
                         ▼                                │
                    ┌──────────────┐                      │
                    │ Injecting    │                      │
                    └────┬─────────┘                      │
                         │ done                           │
                         └────────────────────────────────┘
```

**State enum:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineState {
    Idle,
    Recording,
    Transcribing { progress: f32 },
    Correcting,
    Injecting,
    Error { message: String },
}
```

---

## 7. Error Handling Strategy

| Layer | Error Type | Recovery |
|-------|-----------|----------|
| Audio | Device not found / permission denied | Show error in UI, retry on hotkey |
| Audio | Stream interrupted | Auto-reconnect, discard current recording |
| STT | Model not loaded | Prompt user to download model |
| STT | Transcription timeout (>30s) | Cancel, show timeout error |
| LLM | Ollama not running / API unreachable | Fallback to Fast Mode (STT only) |
| LLM | API auth failure (invalid key) | Show error in UI, prompt user to check API key |
| LLM | Generation timeout (>10s) | Use raw STT output, warn user |
| Inject | Clipboard access denied | Show text in UI for manual copy |
| Inject | Key simulation blocked | Show text in UI for manual copy |

**กลยุทธ์หลัก:** Graceful degradation — ถ้า LLM ล้มเหลว ใช้ raw STT output แทน ไม่หยุดทำงาน

---

## 8. Cross-Platform Considerations

| Feature | Windows | macOS | Linux (X11) | Linux (Wayland) |
|---------|---------|-------|-------------|-----------------|
| Audio (cpal) | WASAPI | CoreAudio | ALSA/PulseAudio | PipeWire |
| Hotkey (rdev) | Win32 API | Accessibility* | X11 events | evdev (limited) |
| Clipboard | Win32 | NSPasteboard | X11 selection | wl-clipboard |
| Key simulation | SendInput | CGEvent | XTest | wl-keyboard (limited) |
| Always-on-top | SetWindowPos | NSWindow level | X11 _NET_WM | xdg-toplevel |

*macOS: ต้องขอ Accessibility permission ครั้งแรก

---

## 9. Phased Delivery

### Phase 1 — MVP (Core Pipeline)
- [x] cpal audio capture → ring buffer
- [x] whisper-rs STT (Thonburian Whisper small/medium)
- [x] egui floating widget (push-to-talk button + text display)
- [x] rdev global hotkey
- [x] arboard + enigo text injection
- [ ] Fast Mode only (no LLM)

### Phase 2 — LLM Integration
- [ ] Ollama REST API integration
- [ ] Basic prompt (zero-shot Thai correction)
- [ ] Filler word removal + punctuation
- [ ] Standard Mode toggle

### Phase 3 — Context & Intelligence
- [ ] Rolling context window (3 sentences)
- [ ] Domain detection (Medical/Legal/Tech/Casual)
- [ ] User vocabulary persistence
- [ ] Few-shot examples in prompt
- [ ] Context Mode toggle

### Phase 4 — Polish & Distribution
- [ ] Model selector UI (small/medium/large)
- [ ] Settings persistence (TOML/JSON)
- [ ] First-run setup wizard (model download)
- [ ] Platform packaging (MSI / DMG / AppImage)
- [ ] Auto-learn vocabulary from corrections

---

## 10. Performance Targets

| Metric | Target (CPU) | Target (GPU) |
|--------|-------------|-------------|
| Audio → STT latency (10s speech) | < 15s | < 3s |
| LLM correction latency | < 5s | < 1s |
| Total pipeline latency | < 20s | < 5s |
| Idle memory usage | < 50 MB | < 50 MB |
| Active memory (models loaded) | < 8 GB | < 4 GB RAM + 6 GB VRAM |
| Widget render time | < 2ms/frame | < 2ms/frame |
| Binary size (release) | < 30 MB | < 30 MB |
