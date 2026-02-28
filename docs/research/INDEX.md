# Research Index — Thai Voice-to-Text (Rust)

**อัปเดตล่าสุด:** 28 กุมภาพันธ์ 2026

โปรเจค: Desktop widget สำหรับ voice-to-text ภาษาไทย
Stack: Rust · Whisper STT · LLM Post-processing · Cross-platform (Win/macOS/Linux)

---

## ไฟล์ทั้งหมด

| ไฟล์ | เนื้อหา | อ่านก่อน |
|------|---------|----------|
| [research-thai-voice-to-text.md](./research-thai-voice-to-text.md) | Tech stack พื้นฐาน: Whisper, cpal, egui, rdev, arboard | ✅ เริ่มต้นที่นี่ |
| [comparison-thai-stt.md](./comparison-thai-stt.md) | เปรียบเทียบ STT engines (ElevenLabs, Whisper, Google ฯลฯ) และ competitor apps | 2 |
| [llm-post-processing-research.md](./llm-post-processing-research.md) | LLM เกลาคำ, context window, local LLM สำหรับ Thai | 3 |
| [system-requirements.md](./system-requirements.md) | System requirements: RAM/CPU/GPU/Storage แยก OS + Cloud options | 4 |
| [research-compilation.md](./research-compilation.md) | งานวิจัยเชิงลึก: error patterns, GEC, architecture | 5 |
| [implementation-guide.md](./implementation-guide.md) | Code examples ใน Rust, pipeline design | 6 |
| [papers-and-references.md](./papers-and-references.md) | Papers, GitHub repos, และ references ทั้งหมด | อ้างอิง |

---

## สรุป Tech Stack ที่เลือก

```
Audio Capture    → cpal
STT Model        → Thonburian Whisper Medium (whisper-rs)
LLM Correction   → Qwen2.5-3B via Ollama (หรือ llama_cpp crate)
Context Manager  → Rolling window (3 ประโยค) + Domain detection + User vocab
UI Widget        → egui + eframe (always-on-top, transparent)
Global Hotkey    → rdev (push-to-talk)
Text Injection   → arboard + enigo (clipboard → Ctrl+V)
```

---

## สรุปการค้นพบสำคัญ

**STT:** Thonburian Whisper คือตัวเลือก open-source ที่ดีที่สุดสำหรับ Thai โดยเฉพาะ (fine-tuned จาก Whisper, ICNLSP 2024)

**LLM Post-processing:** งานวิจัยพิสูจน์ว่า LLM correction ลด CER ได้ 40–60% สำหรับปัญหา tone mark, homophone, filler words และ punctuation ของภาษาไทย (อ้างอิง: HyPoradise NeurIPS 2023, Whispering LLaMA EMNLP 2023)

**Context Window:** เก็บ 2–3 ประโยคก่อนหน้า + detect domain อัตโนมัติ (Medical/Legal/Technical/Casual) ให้ผลดีที่สุดโดยไม่เพิ่ม latency มากนัก

**Competitor Gap:** ไม่มี app ใดในตลาดที่ครบทั้ง Thai + Cross-platform + Offline + Open source พร้อมกัน

---

## Roadmap

```
Phase 1 — MVP
  └── Whisper STT + Push-to-talk widget + Text inject

Phase 2 — LLM Integration
  └── Ollama (Qwen 3B) + filler removal + punctuation

Phase 3 — Context
  └── Rolling window + domain detection + user vocabulary

Phase 4 — Polish
  └── Model selector UI, fast/standard/context mode, auto-learn vocab
```
