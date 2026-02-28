# STT Engine Design

**วันที่:** 28 กุมภาพันธ์ 2026
**ขอบเขต:** Whisper integration, model management, transcription pipeline

---

## 1. Engine Overview

```
┌──────────────────────────────────────────────────────┐
│                  SttEngine                            │
│                                                      │
│   ┌─────────────┐    ┌──────────────┐               │
│   │ ModelManager │    │ WhisperEngine│               │
│   │ - download   │───▶│ - context    │               │
│   │ - verify     │    │ - state      │               │
│   │ - list       │    │ - params     │               │
│   └─────────────┘    └──────┬───────┘               │
│                              │                       │
│                              ▼                       │
│                    ┌──────────────────┐              │
│                    │ transcribe()     │              │
│                    │ audio → text     │              │
│                    └──────────────────┘              │
└──────────────────────────────────────────────────────┘
```

---

## 2. WhisperEngine

### 2.1 Initialization

```rust
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

pub struct WhisperEngine {
    ctx: WhisperContext,
    model_size: ModelSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModelSize {
    Small,   // 242 MB, ~900 MB RAM, CER สูงกว่า
    Medium,  // 769 MB, ~3 GB RAM,   balanced ✅
    Large,   // 1.5 GB, ~6 GB RAM,   accuracy สูงสุด
}

impl WhisperEngine {
    pub fn new(model_path: &str) -> Result<Self> {
        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

        let model_size = Self::detect_model_size(model_path);

        Ok(Self { ctx, model_size })
    }
}
```

### 2.2 Transcription

```rust
impl WhisperEngine {
    /// Transcribe audio samples (16kHz, mono, f32) → text in the configured language.
    /// `language`: ISO-639-1 code ("th", "en", "zh", etc.) or "auto" for Whisper detection.
    pub fn transcribe(&self, audio: &[f32], language: &str) -> Result<TranscriptionResult> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Language — None means Whisper auto-detects
        let lang_opt = if language == "auto" { None } else { Some(language) };
        params.set_language(lang_opt);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Performance tuning
        params.set_n_threads(Self::optimal_threads());
        params.set_single_segment(false);

        // Create state and run
        let mut state = self.ctx.create_state()
            .map_err(|e| format!("Failed to create state: {}", e))?;

        let start = std::time::Instant::now();
        state.full(params, audio)
            .map_err(|e| format!("Transcription failed: {}", e))?;

        // Extract segments
        let num_segments = state.full_n_segments()
            .map_err(|e| format!("Failed to get segments: {}", e))?;

        let mut text = String::new();
        let mut segments = Vec::new();

        for i in 0..num_segments {
            let segment_text = state.full_get_segment_text(i)
                .map_err(|e| format!("Failed to get segment {}: {}", i, e))?;
            let t0 = state.full_get_segment_t0(i).unwrap_or(0);
            let t1 = state.full_get_segment_t1(i).unwrap_or(0);

            segments.push(Segment {
                text: segment_text.clone(),
                start_ms: t0 as u64 * 10,
                end_ms: t1 as u64 * 10,
            });

            text.push_str(&segment_text);
        }

        Ok(TranscriptionResult {
            text: text.trim().to_string(),
            segments,
            duration_ms: start.elapsed().as_millis(),
            model_size: self.model_size,
        })
    }

    fn optimal_threads() -> i32 {
        let cpus = num_cpus::get_physical();
        // ใช้ physical cores ทั้งหมด แต่ไม่เกิน 8
        cpus.min(8) as i32
    }
}
```

### 2.3 Result Types

```rust
pub struct TranscriptionResult {
    pub text: String,
    pub segments: Vec<Segment>,
    pub duration_ms: u128,
    pub model_size: ModelSize,
}

pub struct Segment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}
```

---

## 3. Model Management

### 3.1 Model Registry

```rust
pub struct ModelInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub size: ModelSize,
    pub file_name: &'static str,
    pub file_size_mb: u64,
    pub ram_required_mb: u64,
    pub source_url: &'static str,
}

pub struct ModelInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub size: ModelSize,
    pub file_name: &'static str,
    pub file_size_mb: u64,
    pub ram_required_mb: u64,
    pub source_url: &'static str,
    /// ISO-639-1 language code this model is optimised for, or "multilingual"
    pub language: &'static str,
}

/// Thai-optimised models (Thonburian Whisper — fine-tuned on Thai, ICNLSP 2024)
/// Best accuracy for Thai. Use these when stt_language = "th".
pub const THAI_MODELS: &[ModelInfo] = &[
    ModelInfo {
        id: "thonburian-small",
        display_name: "Thonburian Whisper Small (Thai)",
        size: ModelSize::Small,
        file_name: "ggml-thonburian-small.bin",
        file_size_mb: 242,
        ram_required_mb: 900,
        source_url: "https://huggingface.co/biodatlab/whisper-small-th-combined",
        language: "th",
    },
    ModelInfo {
        id: "thonburian-medium",
        display_name: "Thonburian Whisper Medium (Thai) [Recommended]",
        size: ModelSize::Medium,
        file_name: "ggml-thonburian-medium.bin",
        file_size_mb: 769,
        ram_required_mb: 3000,
        source_url: "https://huggingface.co/biodatlab/whisper-th-medium-combined",
        language: "th",
    },
    ModelInfo {
        id: "thonburian-large",
        display_name: "Thonburian Whisper Large (Thai)",
        size: ModelSize::Large,
        file_name: "ggml-thonburian-large.bin",
        file_size_mb: 1500,
        ram_required_mb: 6000,
        source_url: "https://huggingface.co/biodatlab/whisper-th-large-combined",
        language: "th",
    },
];

/// Standard Whisper models (99-language multilingual)
/// Use for non-Thai languages or when stt_language = "auto".
pub const WHISPER_MODELS: &[ModelInfo] = &[
    ModelInfo {
        id: "whisper-small",
        display_name: "Whisper Small (Multilingual, 99 langs)",
        size: ModelSize::Small,
        file_name: "ggml-whisper-small.bin",
        file_size_mb: 244,
        ram_required_mb: 1000,
        source_url: "https://huggingface.co/ggerganov/whisper.cpp",
        language: "multilingual",
    },
    ModelInfo {
        id: "whisper-medium",
        display_name: "Whisper Medium (Multilingual, 99 langs)",
        size: ModelSize::Medium,
        file_name: "ggml-whisper-medium.bin",
        file_size_mb: 769,
        ram_required_mb: 3000,
        source_url: "https://huggingface.co/ggerganov/whisper.cpp",
        language: "multilingual",
    },
    ModelInfo {
        id: "whisper-large-v3",
        display_name: "Whisper Large-v3 (Multilingual, 99 langs)",
        size: ModelSize::Large,
        file_name: "ggml-whisper-large-v3.bin",
        file_size_mb: 1550,
        ram_required_mb: 6000,
        source_url: "https://huggingface.co/ggerganov/whisper.cpp",
        language: "multilingual",
    },
];

/// Combined registry of all available models
pub const ALL_MODELS: &[&[ModelInfo]] = &[THAI_MODELS, WHISPER_MODELS];

/// Return recommended models for a given language code.
/// "th" → Thonburian models; everything else → standard Whisper.
pub fn models_for_language(language: &str) -> &'static [ModelInfo] {
    if language == "th" { THAI_MODELS } else { WHISPER_MODELS }
}
```

### 3.2 Model Storage

```
Platform-specific model directory:
  Windows: %LOCALAPPDATA%\voice-to-text\models\
  macOS:   ~/Library/Application Support/voice-to-text/models/
  Linux:   ~/.local/share/voice-to-text/models/
```

### 3.3 Model Download Flow

```
┌──────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────┐
│ UI: user │───▶│ Check if     │───▶│ Download     │───▶│ Verify   │
│ selects  │    │ model exists │    │ from HF Hub  │    │ checksum │
│ model    │    │ locally      │    │ (progress %) │    │          │
└──────────┘    └──────────────┘    └──────────────┘    └──────────┘
```

```rust
pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new() -> Result<Self> {
        let models_dir = Self::platform_models_dir()?;
        std::fs::create_dir_all(&models_dir)?;
        Ok(Self { models_dir })
    }

    /// ตรวจสอบว่า model มีอยู่ในเครื่องหรือไม่
    pub fn is_model_available(&self, model: &ModelInfo) -> bool {
        self.models_dir.join(model.file_name).exists()
    }

    /// path ไปยัง model file
    pub fn model_path(&self, model: &ModelInfo) -> PathBuf {
        self.models_dir.join(model.file_name)
    }

    /// list models ที่มีอยู่ในเครื่อง
    pub fn list_local_models(&self) -> Vec<&ModelInfo> {
        AVAILABLE_MODELS.iter()
            .filter(|m| self.is_model_available(m))
            .collect()
    }

    fn platform_models_dir() -> Result<PathBuf> {
        let base = dirs::data_dir()
            .ok_or("Cannot determine data directory")?;
        Ok(base.join("voice-to-text").join("models"))
    }
}
```

### 3.4 GGML Format Conversion

Thonburian Whisper models บน HuggingFace เป็น PyTorch format
ต้อง convert เป็น GGML ก่อนใช้กับ whisper-rs:

```bash
# ใช้ script จาก whisper.cpp
python whisper.cpp/models/convert-pt-to-ggml.py \
    biodatlab/whisper-th-medium-combined \
    models/ \
    models/ggml-thonburian-medium.bin
```

**ทางเลือก:** Pre-convert แล้ว host GGML files ไว้เอง ให้ user download ได้ทันที

---

## 4. Performance Characteristics

### 4.1 Latency by Model Size (10s audio)

| Model | CPU (8-core) | GPU (RTX 3060) | Apple M2 |
|-------|-------------|---------------|----------|
| Small | 2-5s | <1s | 1-2s |
| Medium | 5-15s | 1-3s | 2-4s |
| Large | 15-30s | 2-5s | 4-8s |

### 4.2 Accuracy (CER) by Model Size

| Model | General Thai | Medical Thai | Code-switched |
|-------|-------------|-------------|---------------|
| Small | ~15-20% CER | ~25-30% CER | ~30% CER |
| Medium | ~8-12% CER | ~15-20% CER | ~20% CER |
| Large | ~5-8% CER | ~10-15% CER | ~15% CER |

> หมายเหตุ: ตัวเลขเป็นค่าประมาณ จะ benchmark จริงใน Phase 1

---

## 5. GPU Acceleration

### 5.1 Feature Flags

```toml
[dependencies]
whisper-rs = { version = "0.13", features = [] }

# เปิด CUDA support:
# whisper-rs = { version = "0.13", features = ["cuda"] }

# เปิด Metal support (macOS):
# whisper-rs = { version = "0.13", features = ["metal"] }
```

### 5.2 GPU Detection

```rust
impl WhisperEngine {
    pub fn detect_gpu() -> GpuInfo {
        // ตรวจสอบ GPU availability ตอน startup
        // ใช้เพื่อแนะนำ model size ให้ user
        #[cfg(feature = "cuda")]
        {
            // Check CUDA availability
        }
        #[cfg(target_os = "macos")]
        {
            // Metal always available on macOS
            return GpuInfo::Metal;
        }
        GpuInfo::None
    }
}

pub enum GpuInfo {
    Cuda { vram_mb: u64 },
    Metal,
    None,
}
```

---

## 6. Error Handling

```rust
#[derive(Debug)]
pub enum SttError {
    ModelNotFound(String),
    ModelLoadFailed(String),
    TranscriptionFailed(String),
    InvalidAudio(String),
    Timeout,
}

impl std::fmt::Display for SttError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelNotFound(path) => write!(f, "Model not found: {}", path),
            Self::ModelLoadFailed(e) => write!(f, "Failed to load model: {}", e),
            Self::TranscriptionFailed(e) => write!(f, "Transcription error: {}", e),
            Self::InvalidAudio(e) => write!(f, "Invalid audio: {}", e),
            Self::Timeout => write!(f, "Transcription timed out"),
        }
    }
}
```

---

## 7. Dependencies

```toml
[dependencies]
whisper-rs = "0.13"
num_cpus = "1.16"
dirs = "6.0"       # Platform-specific directories
```
