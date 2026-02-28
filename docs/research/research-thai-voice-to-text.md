# Research: Thai Voice-to-Text Desktop App (Rust)

**วันที่:** 28 กุมภาพันธ์ 2026
**โปรเจค:** Thai Voice-to-Text Widget
**เป้าหมาย:** App เบา เร็ว ข้ามแพลตฟอร์ม (Win/macOS/Linux) รองรับภาษาไทย
**การทำงาน:** Widget เล็กแบบ Push-to-Talk (คล้าย Win+H) inject ข้อความเข้า active window

---

## 1. Thai Speech-to-Text Models

### ตัวเลือกหลัก: Thonburian Whisper ⭐⭐⭐⭐⭐

Fine-tuned จาก OpenAI Whisper โดยเฉพาะสำหรับภาษาไทย โดย BioDataLab (ICNLSP 2024)

| Model | Size | VRAM | เหมาะกับ |
|-------|------|------|----------|
| `biodatlab/whisper-small-th-combined` | ~242MB | ~1GB | เครื่องทั่วไป (CPU) |
| `biodatlab/whisper-th-medium-combined` | ~770MB | ~3GB | สมดุลดีที่สุด ✅ |
| `biodatlab/whisper-th-large-combined` | ~1.5GB | ~6GB | ความแม่นยำสูงสุด |

**ข้อดีของ Thonburian Whisper:**
- ฝึกบน CommonVoice 13, Gowajee corpus, Thai Elderly Speech, Thai Dialect
- รองรับเสียงรบกวน (noise-robust)
- ทำงานได้ดีกับ domain เฉพาะ (การเงิน, การแพทย์)
- ใช้ CER (Character Error Rate) วัดผล — เหมาะกับภาษาที่ไม่มี word boundary

**GitHub:** https://github.com/biodatlab/thonburian-whisper
**HuggingFace:** https://huggingface.co/biodatlab

### ตัวเลือกรอง: OpenAI Whisper Large V3

| Variant | Speed | Thai Accuracy | VRAM |
|---------|-------|---------------|------|
| large-v3 | baseline | ⭐⭐⭐⭐ | ~10GB |
| large-v3-turbo | 6x เร็วกว่า | ⭐⭐⭐ (Thai accuracy ลดเล็กน้อย) | ~6GB |

> ⚠️ Turbo variant มี accuracy ลดลงสำหรับภาษาไทยโดยเฉพาะ

### Benchmark เปรียบเทียบ (2025)

| Model | Thai WER/CER | Notes |
|-------|-------------|-------|
| Thonburian Whisper Medium | ดีที่สุดสำหรับ Thai | Fine-tuned เฉพาะ |
| Whisper Large V3 | ดี | General multilingual |
| Whisper Large V3 Turbo | ดีพอใช้ | ช้ากว่าที่คาดสำหรับ Thai |
| ElevenLabs Scribe (cloud) | WER 3.1% FLEURS | Cloud only |

---

## 2. Rust Integration: Whisper Engines

### Option A: `whisper-rs` (แนะนำ)

```toml
[dependencies]
whisper-rs = "0.13"
```

- Rust bindings สำหรับ whisper.cpp (C++)
- GPU support: CUDA, ROCm, Metal (Apple Silicon)
- ประสิทธิภาพดีที่สุด — ใช้ C++ backend ที่ optimize แล้ว
- รองรับ GGML format models
- **GitHub:** https://github.com/tazz4843/whisper-rs
- Downloads: ~8,000/month

**วิธีใช้งาน:**
- Convert Thonburian Whisper → GGML format ด้วย `whisper.cpp/models/convert-pt-to-ggml.py`
- Load model → stream audio → transcribe

### Option B: `candle` (HuggingFace)

```toml
[dependencies]
candle-core = "0.8"
candle-transformers = "0.8"
```

- Pure Rust ML framework จาก HuggingFace
- ไม่ต้องพึ่ง C++ dependency
- Load model จาก HuggingFace Hub โดยตรง (SafeTensors format)
- GPU support ผ่าน CUDA/Metal
- **GitHub:** https://github.com/huggingface/candle

> **แนะนำ:** เริ่มต้นด้วย `whisper-rs` เพราะ performance ดีกว่าและ ecosystem กว้างกว่า

---

## 3. Audio Capture

### `cpal` — Cross-Platform Audio Library ⭐

```toml
[dependencies]
cpal = "0.15"
```

- Pure Rust, callback-based real-time microphone capture
- รองรับ Windows (WASAPI), macOS (CoreAudio), Linux (ALSA/PulseAudio/JACK)
- Downloads: ~8.7M/month
- **GitHub:** https://github.com/RustAudio/cpal

**การติดตั้ง Linux:**
```bash
sudo apt install libasound2-dev  # Ubuntu/Debian
sudo dnf install alsa-lib-devel  # Fedora
```

**ตัวอย่าง flow:**
```
Microphone → cpal callback → ring buffer → whisper-rs → Thai text
```

---

## 4. UI Framework: Floating Widget

### `egui` + `eframe` (แนะนำ) ⭐⭐⭐⭐⭐

```toml
[dependencies]
eframe = "0.31"
egui = "0.31"
```

- Immediate-mode GUI, เบามาก
- Memory overhead: ~30MB
- Performance: ~1-2ms/frame, วาดเฉพาะตอน interact
- Always-on-top window support
- Cross-platform ผ่าน wgpu backend
- **GitHub:** https://github.com/emilk/egui

**ความสามารถที่ต้องการ:**
```rust
// Always-on-top floating window
let mut options = eframe::NativeOptions::default();
options.viewport = egui::ViewportBuilder::default()
    .with_always_on_top()
    .with_decorations(false)  // ไม่มี title bar
    .with_transparent(true)
    .with_inner_size([300.0, 80.0]);
```

### ตัวเลือกอื่น

| Framework | ข้อดี | ข้อเสีย |
|-----------|--------|---------|
| **Slint** | <300KiB binary! | Learning curve สูงกว่า |
| **Tauri** | Web tech (HTML/CSS) | หนักเกินสำหรับ widget เล็ก |
| **iced** | Elm architecture | Less mature กว่า egui |

---

## 5. Global Hotkey (Push-to-Talk)

### Cross-Platform Solution

```toml
[dependencies]
global-hotkey = "0.6"  # Windows/macOS
# Linux Wayland ต้องใช้ hotkey-listener หรือ rdev
rdev = "0.5"
```

**Strategy:**

| OS | Library | Notes |
|----|---------|-------|
| Windows | `global-hotkey` | ทำงานใน background ได้ |
| macOS | `global-hotkey` | ต้องขอ Accessibility permission |
| Linux (X11) | `global-hotkey` | ทำงานได้ |
| Linux (Wayland) | `rdev` + evdev | จำกัดบางส่วน |

**Push-to-Talk Pattern (hold key):**
```rust
// กด: เริ่มอัด / ปล่อย: หยุดอัดและ transcribe
rdev::listen(|event| match event.event_type {
    EventType::KeyPress(Key::F9) => start_recording(),
    EventType::KeyRelease(Key::F9) => stop_and_transcribe(),
    _ => {}
})
```

---

## 6. Text Injection (ส่งข้อความเข้า Active Window)

### Primary: Clipboard + Paste (แนะนำสำหรับภาษาไทย) ⭐

```toml
[dependencies]
arboard = "3.4"  # Maintained by 1Password
enigo = "0.3"
```

**วิธีนี้แนะนำมากกว่าสำหรับภาษาไทย เพราะ:**
- ภาษาไทยมี combining diacritics (สระ, วรรณยุกต์) ที่ simulate keyboard ยาก
- หลีกเลี่ยงปัญหา keyboard layout
- ทำงานได้กับทุก app ที่รับ paste

```rust
// Clipboard approach
let mut clipboard = arboard::Clipboard::new()?;
clipboard.set_text(thai_transcription)?;

// Simulate Ctrl+V
let mut enigo = enigo::Enigo::new(&enigo::Settings::default())?;
enigo.key(enigo::Key::Control, enigo::Direction::Press)?;
enigo.key(enigo::Key::Unicode('v'), enigo::Direction::Click)?;
enigo.key(enigo::Key::Control, enigo::Direction::Release)?;
```

### Alternative: Direct Unicode Input

```rust
// enigo direct typing (อาจมีปัญหากับ Thai combining chars)
enigo.text(&thai_transcription)?;
```

---

## 7. สถาปัตยกรรมที่แนะนำ

```
┌─────────────────────────────────────────────────────┐
│                   Thai STT Widget                    │
├─────────────────────────────────────────────────────┤
│                                                       │
│  [Global Hotkey]                                      │
│   rdev / global-hotkey                                │
│        │                                             │
│        ▼                                             │
│  [Audio Capture]         [UI Widget - egui]          │
│   cpal (microphone)  ←→  Push-to-talk button         │
│        │                 Recording indicator         │
│        ▼                 Waveform display            │
│  [Ring Buffer]                                        │
│   (16kHz, mono, f32)                                 │
│        │                                             │
│        ▼                                             │
│  [STT Engine]                                         │
│   whisper-rs                                         │
│   + Thonburian Whisper model                         │
│   (GGML format)                                      │
│        │                                             │
│        ▼                                             │
│  [Text Post-processing]                               │
│   Thai-specific cleanup                              │
│        │                                             │
│        ▼                                             │
│  [Text Injection]                                     │
│   arboard + enigo                                    │
│   (Clipboard → Ctrl+V)                               │
│                                                       │
└─────────────────────────────────────────────────────┘
```

### Thread Architecture

```
Main Thread: egui UI + event loop
├── Hotkey Thread: rdev::listen()
├── Audio Thread: cpal stream callback → ring buffer
├── STT Thread: whisper-rs inference (blocking, CPU/GPU)
└── Injection Thread: arboard + enigo
```

---

## 8. Tech Stack สรุป

| Component | Library | Crate |
|-----------|---------|-------|
| STT Model | Thonburian Whisper (GGML) | `whisper-rs` |
| Audio Capture | CPAL | `cpal` |
| UI Widget | egui + eframe | `eframe` |
| Global Hotkey | rdev | `rdev` |
| Text Inject | Clipboard | `arboard` + `enigo` |
| Async Runtime | Tokio | `tokio` |
| Channel/Concurrency | Crossbeam | `crossbeam` |

---

## 9. Performance Estimate

| Scenario | Memory | Latency |
|----------|--------|---------|
| Idle (widget shown) | ~30-50MB | - |
| Recording | ~50-80MB | - |
| Whisper Small (CPU) | ~80-120MB | 2-5s / 5s audio |
| Whisper Medium (CPU) | ~100-200MB | 5-15s / 5s audio |
| Whisper Medium (GPU) | ~300MB RAM + 3GB VRAM | <2s / 5s audio |

> **Note:** CPU inference สำหรับ Whisper Medium อาจช้า ควรพิจารณา:
> - ใช้ whisper-small สำหรับ real-time
> - หรือ GPU inference ถ้าผู้ใช้มี GPU

---

## 10. ขั้นตอนการพัฒนาแนะนำ (MVP)

### Phase 1: Core Pipeline
1. Setup Rust project + cargo workspace
2. Audio capture ด้วย `cpal` → WAV file
3. Integrate `whisper-rs` + Thonburian Whisper small model
4. Test Thai transcription accuracy

### Phase 2: UI + Hotkey
5. สร้าง floating widget ด้วย `egui`
6. Implement global hotkey ด้วย `rdev`
7. Push-to-talk → audio buffer → transcribe

### Phase 3: Text Injection + Polish
8. Clipboard injection ด้วย `arboard` + `enigo`
9. Platform testing (Win/Mac/Linux)
10. Model selection UI (small/medium/large)
11. Packaging + distribution

---

## 11. ข้อควรระวังเฉพาะภาษาไทย

1. **ไม่มี Word Boundary** — ภาษาไทยไม่มีช่องว่างระหว่างคำ ใช้ CER แทน WER
2. **Combining Characters** — สระและวรรณยุกต์เป็น combining chars (U+0E30 ถึง U+0E4E) ใช้ clipboard paste แทน keyboard simulation
3. **UTF-8** — ตรวจสอบ encoding ทุกจุดในระบบ
4. **VAD (Voice Activity Detection)** — ควรใช้ Silero VAD เพื่อตัด silence ก่อนส่ง Whisper
5. **Tone Marks** — วรรณยุกต์สำคัญมากในภาษาไทย model ที่ fine-tune เฉพาะจะแม่นกว่า

---

## 12. ทางเลือกสำหรับ Real-time / Low Latency

ถ้า latency ของ Whisper ช้าเกินไป:

| Option | Latency | Trade-off |
|--------|---------|-----------|
| **Whisper Small (CPU)** | 2-5s | ความแม่นยำต่ำกว่า |
| **Whisper Medium (GPU)** | <2s | ต้องมี GPU |
| **faster-whisper (Python service)** | <1s | ต้องมี Python sidecar |
| **Cloud API (Gemini/OpenAI)** | <1s | ต้องใช้ internet + ค่าใช้จ่าย |
| **Streaming Whisper** | ~1s | ซับซ้อนกว่า |

---

## Sources

- [Thonburian Whisper GitHub](https://github.com/biodatlab/thonburian-whisper)
- [Thonburian Whisper Paper (ICNLSP 2024)](https://aclanthology.org/2024.icnlsp-1.17.pdf)
- [whisper-rs (Rust bindings)](https://github.com/tazz4843/whisper-rs)
- [candle (HuggingFace Rust ML)](https://github.com/huggingface/candle)
- [cpal (Rust audio)](https://github.com/RustAudio/cpal)
- [egui (Rust GUI)](https://github.com/emilk/egui)
- [rdev (global hotkey)](https://github.com/Narsil/rdev)
- [arboard (clipboard)](https://github.com/1Password/arboard)
- [enigo (keyboard simulation)](https://github.com/enigo-rs/enigo)
- [Speech-to-text benchmarks 2025 (Soniox)](https://soniox.com/benchmarks)
- [Best open-source STT 2026 (Northflank)](https://northflank.com/blog/best-open-source-speech-to-text-stt-model-in-2026-benchmarks)
