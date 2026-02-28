# Design Documents Index — Thai Voice-to-Text (Rust)

**อัปเดตล่าสุด:** 28 กุมภาพันธ์ 2026

System design specifications สำหรับ Thai Voice-to-Text Desktop Widget
อ้างอิงจาก research ใน [docs/research/INDEX.md](../research/INDEX.md)

---

## ไฟล์ทั้งหมด

| ไฟล์ | เนื้อหา | ขอบเขต |
|------|---------|--------|
| [architecture-overview.md](./architecture-overview.md) | สถาปัตยกรรมภาพรวม, module boundaries, crate structure | ระบบทั้งหมด |
| [audio-pipeline-design.md](./audio-pipeline-design.md) | Audio capture → resampling → VAD → ring buffer | Audio subsystem |
| [stt-engine-design.md](./stt-engine-design.md) | Whisper integration, model management, GGML format | STT subsystem |
| [llm-correction-design.md](./llm-correction-design.md) | LLM pipeline, prompt design, context manager, domain detection | LLM subsystem |
| [ui-widget-design.md](./ui-widget-design.md) | egui floating widget, states, wireframes, interactions | UI layer |
| [text-injection-design.md](./text-injection-design.md) | Clipboard → paste flow, platform edge cases | Output subsystem |
| [threading-and-data-flow.md](./threading-and-data-flow.md) | Thread architecture, channels, state management, error handling | Infrastructure |
| [configuration-and-modes.md](./configuration-and-modes.md) | User settings, model selection, operating modes, persistence | Configuration |

---

## Quick Reference: Tech Stack

```
Audio Capture    → cpal 0.15
VAD              → Silero VAD (ONNX) หรือ whisper-rs built-in VAD
STT Engine       → whisper-rs (whisper.cpp bindings) + Thonburian Whisper GGML
LLM Correction   → Ollama REST API (MVP) / llama_cpp crate (Phase 2)
LLM Model        → Qwen2.5-3B GGUF Q4 (default) / Typhoon2-Qwen2.5-7B (GPU)
Context Manager  → Rolling window 3 ประโยค + Domain detection + User vocab
UI Widget        → egui 0.31 + eframe (always-on-top, transparent, borderless)
Global Hotkey    → rdev 0.5 (push-to-talk)
Text Injection   → arboard 3.4 (clipboard) + enigo 0.3 (Ctrl+V simulation)
Async Runtime    → tokio
Channels         → tokio::sync::mpsc / crossbeam-channel
```

---

## Design Principles

1. **Offline-First** — ทำงานได้โดยไม่ต้องใช้ internet (Local Mode เป็น default)
2. **Privacy-First** — เสียงและข้อความไม่ออกจากเครื่อง (ยกเว้น Cloud Mode ที่ user เลือกเอง)
3. **Modular** — แต่ละ component เปลี่ยนแทนได้ (เช่น เปลี่ยน STT engine โดยไม่กระทบ LLM)
4. **Progressive Enhancement** — Fast Mode (STT only) → Standard Mode (+LLM) → Context Mode (+context window)
5. **Cross-Platform** — Windows / macOS / Linux ใช้ codebase เดียวกัน
