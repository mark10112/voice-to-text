# System Requirements — Thai Voice-to-Text Widget

**วันที่:** 28 กุมภาพันธ์ 2026
**ครอบคลุม:** Local (CPU/GPU) และ Cloud Deployment บน Windows · macOS · Linux

---

## 1. สรุป Configuration Tiers

| Tier | RAM | CPU | GPU | Whisper Model | LLM Model | Latency/utterance |
|------|-----|-----|-----|---------------|-----------|-------------------|
| **Minimal** | 8 GB | 4-core (2019+) | ไม่จำเป็น | small (242 MB) | Qwen2.5-1.5B | ~15–25s |
| **Recommended** ✅ | 16 GB | 8-core (2021+) | ไม่จำเป็น | medium (769 MB) | Qwen2.5-3B | ~8–15s |
| **High-end** | 32 GB | 12-core+ | GPU 6GB+ VRAM | medium/large | Qwen2.5-7B | ~2–5s |
| **Apple Silicon** | 16 GB unified | M1/M2/M3 | Metal (built-in) | medium | Qwen2.5-7B | ~3–6s |

---

## 2. RAM Requirements (ละเอียด)

### Whisper STT Models
| Model | ขนาดไฟล์ | RAM ที่ใช้จริง (runtime) |
|-------|---------|------------------------|
| whisper-small | 242 MB | ~900 MB |
| whisper-medium | 769 MB | ~3.0 GB |
| whisper-large-v3 | 1.5 GB | ~6.0 GB |

### LLM Correction (Ollama / llama.cpp) — Q4_K_M quantization
| Model | ขนาดไฟล์ | RAM ที่ใช้จริง (runtime) |
|-------|---------|------------------------|
| Qwen2.5-1.5B Q4 | ~1.1 GB | ~2.5 GB |
| Qwen2.5-3B Q4 | ~2.0 GB | ~4.0 GB |
| Qwen2.5-7B Q4 | ~4.7 GB | ~7.5 GB |

### RAM รวมแต่ละ Configuration
| Whisper + LLM | Whisper RAM | LLM RAM | App overhead | รวม |
|---------------|------------|---------|--------------|-----|
| small + 1.5B | 0.9 GB | 2.5 GB | ~0.3 GB | **~3.7 GB** |
| small + 3B | 0.9 GB | 4.0 GB | ~0.3 GB | **~5.2 GB** |
| medium + 3B ✅ | 3.0 GB | 4.0 GB | ~0.3 GB | **~7.3 GB** |
| medium + 7B | 3.0 GB | 7.5 GB | ~0.3 GB | **~10.8 GB** |
| large + 7B | 6.0 GB | 7.5 GB | ~0.3 GB | **~13.8 GB** |

> **Note:** RAM ที่แนะนำ = รวม + buffer 20% เพื่อให้ OS และ app อื่นทำงานได้
> → medium + 3B ใช้ ~7.3 GB → ต้องการ RAM จริงอย่างน้อย **16 GB**

---

## 3. CPU Requirements

### ขั้นต่ำที่รันได้
- **Architecture:** x86-64 ที่รองรับ SSE4.2 เป็นอย่างน้อย
- **Cores:** 4 physical cores ขึ้นไป
- **Generation:** Intel Gen 7+ (Kaby Lake, 2017) / AMD Ryzen 1000 (Zen 1, 2017)

### SIMD Acceleration — ผลต่อ whisper.cpp
| Instruction Set | Speedup vs baseline | เริ่มจาก CPU รุ่น |
|----------------|--------------------|--------------------|
| SSE4.2 (baseline) | 1x | Intel Gen 1+, AMD Bulldozer |
| **AVX2** | **2–4x** | Intel Gen 4+ (Haswell 2013), AMD Ryzen 1000+ |
| AVX-512 | 3–6x | Intel Gen 11+ (Rocket Lake 2021) |
| ARM NEON | 2–3x | Apple M1+, ARM64 |

> ✅ **แนะนำ AVX2 ขั้นต่ำ** — ได้ใน CPU ทุกตัวที่ผลิตหลังปี 2015

### Thread Recommendations
| Use Case | Recommended Threads |
|----------|---------------------|
| Whisper inference (CPU) | 4–8 threads (แนะนำ = physical cores) |
| Ollama LLM inference | 4–6 threads (memory bandwidth คือ bottleneck) |
| Audio capture (cpal) | 1 dedicated thread |
| UI (egui) | 1 thread |

> **Note:** สำหรับ LLM บน CPU — **memory bandwidth สำคัญกว่า GHz** เช่น DDR5 เร็วกว่า DDR4 อย่างชัดเจน

---

## 4. GPU Requirements (Optional)

GPU ไม่บังคับ แต่เพิ่ม speed ได้อย่างมาก

### NVIDIA (CUDA)
| Model | VRAM ที่ต้องการ | Speedup vs CPU |
|-------|--------------|----------------|
| whisper-small | 2 GB | 5–8x |
| whisper-medium | 4 GB | 5–10x |
| whisper-large | 8 GB | 5–10x |
| Qwen2.5-3B Q4 | 4 GB | 5–15x |
| Qwen2.5-7B Q4 | 6 GB | 5–15x |
| **medium + 3B รวม** | **6–8 GB** | — |

### AMD (ROCm)
- ต้องการ ROCm 5.6+ (Linux เท่านั้น หรือ Windows ROCm beta)
- VRAM requirements เทียบเท่า NVIDIA
- ⚠️ Windows ROCm ยังไม่ stable เท่า CUDA

### Apple Silicon (Metal / Unified Memory)
- ใช้ Unified Memory — GPU/CPU ใช้ RAM เดียวกัน
- **ข้อดี:** ไม่มี VRAM limit แยก — RAM 16 GB = รัน medium + 7B ได้สบาย
- M1: ~3–5s per utterance (medium model)
- M2/M3: ~2–3s per utterance
- M3 Max/Ultra: <1s per utterance

---

## 5. Storage Requirements

### ขนาดไฟล์
| Component | ขนาด |
|-----------|------|
| App binary (Rust release) | ~15–30 MB |
| whisper-small (GGML) | 242 MB |
| whisper-medium (GGML) | 769 MB |
| whisper-large-v3 (GGML) | 1.5 GB |
| Qwen2.5-1.5B Q4 GGUF | 1.1 GB |
| Qwen2.5-3B Q4 GGUF | 2.0 GB |
| Qwen2.5-7B Q4 GGUF | 4.7 GB |
| Ollama installation | ~500 MB |
| OS dependencies | ~100 MB |

### รวม Disk ที่ต้องการ
| Config | Disk (ไม่รวม OS) |
|--------|----------------|
| Minimal (small + 1.5B) | ~2.0 GB |
| Recommended (medium + 3B) | ~3.5 GB |
| High-end (large + 7B) | ~7.5 GB |

---

## 6. Requirements แยกตาม OS

### Windows

| Component | ข้อกำหนด |
|-----------|----------|
| **OS Version** | Windows 10 (1903+) / Windows 11 |
| **Audio Backend** | WASAPI (built-in, แนะนำ) · ASIO (low-latency, ต้องติดตั้งแยก) |
| **Runtime** | Visual C++ Redistributable 2022 |
| **GPU (CUDA)** | CUDA 11.8+ · cuDNN 8.6+ |
| **GPU (AMD)** | ROCm 5.6+ (beta บน Windows) |
| **Permissions** | ไม่ต้องการ admin สำหรับ hotkey / audio |
| **Hotkey** | Win32 `RegisterHotKey` API ทำงานได้ทุก app |

**⚠️ ข้อควรระวัง Windows:**
- บางแอป (admin process) รับ text inject ไม่ได้หากรัน app ในฐานะ user ปกติ
- Windows Defender อาจ flag `enigo` (keyboard simulation) — ต้อง code-sign binary

---

### macOS

| Component | ข้อกำหนด |
|-----------|----------|
| **OS Version** | macOS 12 Monterey+ (แนะนำ 13 Ventura+) |
| **Audio Backend** | CoreAudio (built-in, zero config) |
| **GPU** | Metal (built-in ทุกเครื่อง) |
| **Permissions** | Microphone + **Accessibility** (บังคับสำหรับ global hotkey + text inject) |
| **App Signing** | Code signing + Notarization ถ้าจะ distribute |
| **Homebrew deps** | ไม่จำเป็น (Rust build self-contained) |

**⚠️ ข้อควรระวัง macOS:**
- ต้องขอ Accessibility permission ครั้งแรก (user ต้อง approve ใน System Settings)
- Notarization บังคับสำหรับ distribute ใน macOS 10.15+ (ต้องมี Apple Developer account $99/ปี)
- Apple Silicon (ARM64) ต้อง build แยกจาก Intel (x86_64) หรือสร้าง Universal Binary

---

### Linux

| Component | ข้อกำหนด |
|-----------|----------|
| **Distro** | Ubuntu 22.04 LTS+ · Fedora 38+ · Arch (rolling) |
| **Audio Backend** | PipeWire (แนะนำ, modern) · PulseAudio · ALSA |
| **Display** | X11 (global hotkey เต็มระบบ) · Wayland (hotkey จำกัด) |
| **GPU (CUDA)** | CUDA 11.8+ · nvidia-cuda-toolkit |
| **GPU (AMD)** | ROCm 5.6+ (Linux รองรับดีกว่า Windows) |
| **Build deps** | `libasound2-dev` · `libssl-dev` · `pkg-config` |

**⚠️ ข้อควรระวัง Linux:**
- **Wayland limitation:** `rdev` global hotkey ทำงานได้จำกัดบน Wayland — ต้องใช้ `evdev` หรือ portal API
- PipeWire ≥ 0.3.32 แนะนำ (มากับ Ubuntu 22.04)
- เพิ่มให้ผู้ใช้อยู่ใน group `audio` และ `input`: `sudo usermod -aG audio,input $USER`

---

## 7. Cloud Deployment Options

สำหรับ user ที่ต้องการ accuracy สูงสุด หรือเครื่องที่ไม่แรงพอ

### Architecture: Cloud STT + Cloud LLM
```
[Local Widget (Rust)]
       │ audio stream / audio file
       ▼
[Cloud STT API]          [Cloud LLM API]
 ElevenLabs Scribe   →   OpenAI GPT-4o-mini
 Google Chirp 3           Typhoon API (Thai)
 AssemblyAI               Gemini Flash
       │
       ▼
[Text กลับมาที่ Widget]
       │
       ▼
[inject → active window]
```

### Cloud STT Cost Comparison
| Provider | Thai Support | ราคา | Latency |
|----------|-------------|------|---------|
| ElevenLabs Scribe | ✅ ดีที่สุด | ~$0.40/ชั่วโมง audio | <1s |
| Google Chirp 3 | ✅ | ~$1.44/ชั่วโมง audio | ~1–2s |
| AssemblyAI | ✅ | ~$0.65/ชั่วโมง audio | ~1–2s |
| Deepgram Nova-3 | ❓ | ~$0.46/ชั่วโมง audio | <300ms |
| OpenAI Whisper API | ✅ | ~$1.20/ชั่วโมง audio | ~1–3s |

### Cloud LLM Cost (Post-processing)
| Provider | Model | ราคา (input/output) | Thai Quality |
|----------|-------|---------------------|-------------|
| OpenAI | gpt-4o-mini | $0.15/$0.60 per 1M tokens | ✅ ดี |
| Google | Gemini 1.5 Flash | $0.075/$0.30 per 1M tokens | ✅ ดี |
| SCB10X | Typhoon API | TBD (Thai-specialized) | ✅ ดีที่สุด (Thai) |

> **ค่าใช้จ่ายโดยประมาณ (Cloud mode):** การใช้งาน ~1 ชั่วโมง/วัน → ประมาณ **$10–20/เดือน**

### Self-hosted Cloud Server (whisper-rs API)
หากต้องการ host Whisper เองบน VPS:

| Spec | ราคาประมาณ | รองรับ |
|------|-----------|--------|
| **CPU VPS:** 8 vCPU, 16 GB RAM | ~$40–60/เดือน | medium model, ~10–20s latency |
| **GPU VPS:** 1x RTX 3090 (24 GB) | ~$200–400/เดือน | large model, ~1–2s latency |
| **AWS g4dn.xlarge** (T4 GPU) | ~$380/เดือน on-demand | medium + fast |
| **AWS g4dn.xlarge** (Spot) | ~$80–120/เดือน | medium + fast |
| **Lambda Labs A10** | ~$75/เดือน | ดีที่สุดราคา/ประสิทธิภาพ |

**Regions ใกล้ไทย (latency ต่ำ):**
- AWS: `ap-southeast-1` (Singapore) ~20ms
- GCP: `asia-southeast1` (Singapore) ~20ms
- Azure: `southeastasia` (Singapore) ~20ms

---

## 8. Build Requirements (สำหรับ Developer)

| Component | Version |
|-----------|---------|
| **Rust toolchain** | 1.82+ (stable) |
| **Cargo** | มาพร้อม Rust |
| **CMake** | 3.15+ (สำหรับ build whisper.cpp) |
| **Git** | ใดก็ได้ (clone submodules) |

### Build Dependencies แยกตาม OS

**Windows:**
```powershell
# Visual Studio Build Tools 2022 (C++ workload)
winget install Microsoft.VisualStudio.2022.BuildTools
```

**macOS:**
```bash
xcode-select --install
brew install cmake
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install build-essential cmake pkg-config \
  libasound2-dev libssl-dev libfontconfig1-dev
```

### Cross-compilation
| จาก → ไป | ความยาก | Notes |
|---------|---------|-------|
| macOS → Windows | ยาก | ต้องใช้ `cross` tool + Docker |
| macOS → Linux | ปานกลาง | `cross` tool |
| Linux → Windows | ปานกลาง | `mingw-w64` หรือ `cross` |
| macOS Intel → Apple Silicon | ง่าย | `cargo build --target aarch64-apple-darwin` |

> **แนะนำ:** ใช้ GitHub Actions CI สำหรับ cross-platform build แทน cross-compile เองทีละตัว

---

## 9. สรุปข้อแนะนำ

### Local Mode — เครื่อง 3 ระดับ

**เครื่องพื้นฐาน (ได้ผลใช้งานได้):**
- CPU: Intel i5/Ryzen 5 (2019+, AVX2), 8+ cores
- RAM: 8 GB → ใช้ whisper-small + Qwen2.5-1.5B
- Disk: 5 GB ว่าง
- Latency: ~15–25 วินาที/utterance

**เครื่องแนะนำ ✅:**
- CPU: Intel i7/Ryzen 7 (2021+), 8+ cores, DDR4-3200+
- RAM: 16 GB → ใช้ whisper-medium + Qwen2.5-3B
- Disk: 10 GB ว่าง
- Latency: ~8–15 วินาที/utterance

**เครื่อง High-end:**
- CPU: Intel i9/Ryzen 9 + GPU 8GB VRAM (RTX 3070+)
- RAM: 32 GB → ใช้ whisper-large + Qwen2.5-7B
- Latency: ~2–5 วินาที/utterance

**Apple Silicon (ดีที่สุดสำหรับ local offline):**
- M1 Pro (18 GB unified) → medium + 7B → ~3–5s
- M2/M3 (16 GB+) → medium + 7B → ~2–3s

### Cloud Mode — สำหรับผู้ใช้ที่ต้องการ accuracy สูงสุด
- ใช้ ElevenLabs Scribe + Typhoon API
- เครื่อง client ต้องการแค่: CPU ใดก็ได้, RAM 4 GB, Internet connection
- ค่าใช้จ่าย: ~$10–20/เดือน สำหรับการใช้งานปกติ
