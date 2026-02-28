# Configuration & Modes Design

**วันที่:** 28 กุมภาพันธ์ 2026
**ขอบเขต:** Operating modes, user settings, persistence, first-run experience

---

## 1. Operating Modes

### 1.1 Overview

```
┌─────────────┬──────────────────────────────────────────────────┐
│  Fast Mode  │  STT → Inject                                   │
│             │  ไม่ใช้ LLM, เร็วที่สุด                          │
│             │  Latency: ~5-15s (CPU)                           │
├─────────────┼──────────────────────────────────────────────────┤
│  Standard   │  STT → LLM (zero-shot) → Inject                │
│  Mode ✅     │  แก้ไข filler, punctuation, tone marks           │
│             │  Latency: ~8-20s (CPU)                           │
├─────────────┼──────────────────────────────────────────────────┤
│  Context    │  STT → LLM (context + vocab) → Inject           │
│  Mode       │  rolling context, domain detection, user vocab  │
│             │  Latency: ~10-25s (CPU)                          │
└─────────────┴──────────────────────────────────────────────────┘
```

### 1.2 Mode Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OperatingMode {
    Fast,      // STT only
    Standard,  // STT + LLM (no context)
    Context,   // STT + LLM + rolling context
}

impl Default for OperatingMode {
    fn default() -> Self {
        Self::Standard
    }
}
```

### 1.3 Feature Matrix

| Feature | Fast | Standard | Context |
|---------|------|----------|---------|
| Whisper STT | ✅ | ✅ | ✅ |
| LLM correction | ❌ | ✅ | ✅ |
| Filler word removal | ❌ | ✅ | ✅ |
| Punctuation | ❌ | ✅ | ✅ |
| Previous context | ❌ | ❌ | ✅ (3 sentences) |
| Domain detection | ❌ | ❌ | ✅ |
| User vocabulary | ❌ | ❌ | ✅ |
| Requires Ollama | ❌ | ✅ | ✅ |

---

## 2. Settings Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // Operating mode
    pub operating_mode: OperatingMode,

    // STT settings
    pub stt_model: String,          // "thonburian-medium" (Thai default)
    pub stt_language: String,       // "th" (default), "en", "zh", "ja", "auto", etc.

    // LLM settings
    pub llm_enabled: bool,
    pub llm_provider: LlmProvider,
    pub llm_model: String,          // "qwen2.5:3b" (default for Ollama)
    pub llm_base_url: String,       // "http://localhost:11434" (Ollama) or any OpenAI-compatible URL
    pub llm_api_key: Option<String>, // None for local (Ollama/LlamaCpp), required for cloud APIs
    pub llm_temperature: f32,       // 0.3
    pub llm_timeout_secs: u64,      // 10

    // Hotkey settings
    pub push_to_talk_key: String,   // "F9"
    pub toggle_visibility_key: String, // "Ctrl+Shift+T"

    // Context settings
    pub context_window_size: usize, // 3 sentences
    pub context_reset_silence_secs: u64, // 120

    // UI settings
    pub widget_position: Option<(f32, f32)>,  // last known position
    pub auto_inject: bool,          // true = auto-inject after correction
    pub show_raw_text: bool,        // true = show raw STT before correction

    // Audio settings
    pub audio_device: Option<String>, // None = system default
    pub max_recording_secs: u64,    // 60
}

/// LLM provider selection — determines API format and auth mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    /// Ollama running locally — uses Ollama native REST API, no auth required
    Ollama,
    /// Any OpenAI-compatible API — e.g. OpenAI, Groq, Together.ai, LM Studio, vLLM
    /// Set llm_base_url + llm_api_key as needed
    OpenAiCompatible,
    /// In-process inference via llama_cpp crate — no network, no auth (Phase 2)
    LlamaCpp,
    /// LLM disabled — Fast mode only
    Disabled,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            operating_mode: OperatingMode::Standard,
            stt_model: "thonburian-medium".into(),
            stt_language: "th".into(),
            llm_enabled: true,
            llm_provider: LlmProvider::Ollama,
            llm_model: "qwen2.5:3b".into(),
            llm_base_url: "http://localhost:11434".into(),
            llm_api_key: None,
            llm_temperature: 0.3,
            llm_timeout_secs: 10,
            push_to_talk_key: "F9".into(),
            toggle_visibility_key: "Ctrl+Shift+T".into(),
            context_window_size: 3,
            context_reset_silence_secs: 120,
            widget_position: None,
            auto_inject: true,
            show_raw_text: true,
            audio_device: None,
            max_recording_secs: 60,
        }
    }
}
```

---

## 3. Settings Persistence

### 3.1 File Format: TOML

```toml
# ~/.config/voice-to-text/settings.toml (Linux)
# %APPDATA%\voice-to-text\settings.toml (Windows)
# ~/Library/Application Support/voice-to-text/settings.toml (macOS)

[general]
operating_mode = "Standard"

[stt]
model = "thonburian-medium"
language = "th"   # ISO-639-1 code, or "auto" for Whisper language detection
                  # Common values: "th", "en", "zh", "ja", "ko", "fr", "de", "es"

[llm]
enabled = true
provider = "Ollama"             # "Ollama" | "OpenAiCompatible" | "LlamaCpp" | "Disabled"
model = "qwen2.5:3b"
base_url = "http://localhost:11434"   # Override for cloud: "https://api.openai.com"
# api_key = ""                 # Required for cloud providers; omit for local Ollama
temperature = 0.3
timeout_secs = 10

[hotkey]
push_to_talk = "F9"
toggle_visibility = "Ctrl+Shift+T"

[context]
window_size = 3
reset_silence_secs = 120

[ui]
auto_inject = true
show_raw_text = true

[audio]
max_recording_secs = 60
```

### 3.2 Load / Save

```rust
impl AppSettings {
    pub fn load() -> Self {
        let path = Self::settings_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn settings_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("voice-to-text")
            .join("settings.toml")
    }
}
```

---

## 4. Data Directory Layout

```
Config dir (settings + user data):
  Windows: %APPDATA%\voice-to-text\
  macOS:   ~/Library/Application Support/voice-to-text/
  Linux:   ~/.config/voice-to-text/

  voice-to-text/
  ├── settings.toml        # Application settings
  └── user-vocab.json      # User vocabulary

Data dir (models):
  Windows: %LOCALAPPDATA%\voice-to-text\models\
  macOS:   ~/Library/Application Support/voice-to-text/models/
  Linux:   ~/.local/share/voice-to-text/models/

  models/
  ├── ggml-thonburian-small.bin       # Thai (default)
  ├── ggml-thonburian-medium.bin      # Thai (default, recommended)
  ├── ggml-thonburian-large.bin       # Thai
  ├── ggml-whisper-small.bin          # Multilingual (for non-Thai)
  ├── ggml-whisper-medium.bin         # Multilingual
  └── ggml-whisper-large-v3.bin       # Multilingual
```

---

## 5. First-Run Experience

### 5.1 Detection

```rust
pub fn is_first_run() -> bool {
    !AppSettings::settings_path().exists()
}
```

### 5.2 First-Run Flow

```
App starts → is_first_run()?
  │
  ├── Yes ─────────────────────────────────────────┐
  │   ┌──────────────────────────────────────────┐  │
  │   │  Welcome to Voice-to-Text!               │  │
  │   │                                          │  │
  │   │  Step 1: Language                        │  │
  │   │  Primary language for speech:            │  │
  │   │  [● Thai (ภาษาไทย)] ← default           │  │
  │   │  [○ English]  [○ Chinese]  [○ Japanese]  │  │
  │   │  [○ Auto-detect]  [○ Other (ISO code)] │  │
  │   │                                          │  │
  │   │  Step 2: Download STT model              │  │
  │   │  (shows models for selected language)    │  │
  │   │  Thai:  [Thonburian Small/Medium/Large]  │  │
  │   │  Other: [Whisper Small/Medium/Large-v3]  │  │
  │   │                                          │  │
  │   │  Step 3: LLM Setup (optional)            │  │
  │   │  ○ Ollama (local, free)                  │  │
  │   │  ○ OpenAI-compatible API (cloud)         │  │
  │   │  ○ Skip (use Fast Mode)                  │  │
  │   │                                          │  │
  │   │  Step 4: Choose hotkey                   │  │
  │   │  [F9] (default)                          │  │
  │   │                                          │  │
  │   │  [Get Started]                           │  │
  │   └──────────────────────────────────────────┘  │
  │                                                  │
  └── No → Normal startup ─────────────────────────┘
```

### 5.3 LLM Setup Guide (in-app)

```
┌──────────────────────────────────────────┐
│  LLM Setup                               │
│                                          │
│  Provider: [● Ollama (local)]            │
│            [○ OpenAI-compatible API]     │
│            [○ Skip (Fast Mode)]          │
│                                          │
│  ── Ollama ──────────────────────────── │
│  1. Download Ollama:                     │
│     https://ollama.com/download          │
│  2. Run in terminal:                     │
│     ollama pull qwen2.5:3b              │
│  Status: ● Ollama connected              │
│          ● qwen2.5:3b loaded             │
│                                          │
│  ── OpenAI-Compatible API ────────────── │
│  Base URL: [https://api.openai.com    ]  │
│  API Key:  [sk-...                    ]  │
│  Model:    [gpt-4o-mini               ]  │
│  Status: ● Connected                     │
│                                          │
│  [Check Connection]  [Skip]              │
└──────────────────────────────────────────┘
```

**Supported providers via OpenAI-compatible API:**
- OpenAI (`https://api.openai.com`) — gpt-4o, gpt-4o-mini
- Groq (`https://api.groq.com/openai`) — llama-3.3-70b, mixtral-8x7b
- Together.ai (`https://api.together.xyz`) — Qwen, Llama family
- LM Studio (`http://localhost:1234`) — any GGUF model locally
- vLLM / any self-hosted endpoint — custom server

---

## 6. Model Selection UI

### 6.1 STT Model Selector

```
┌──────────────────────────────────────────┐
│  STT Model                               │
│                                          │
│  Language: [Thai ▾]                      │
│                                          │
│  ── Thai-optimized models ────────────── │
│  ● Thonburian Small (242 MB)             │
│    Speed: fast, Accuracy: good           │
│    RAM: ~1 GB                            │
│                                          │
│  ○ Thonburian Medium (769 MB) [Recommended] │
│    Speed: moderate, Accuracy: very good  │
│    RAM: ~3 GB                            │
│                                          │
│  ○ Thonburian Large (1.5 GB)             │
│    Speed: slow, Accuracy: best           │
│    RAM: ~6 GB  ⚠️ Requires GPU           │
│                                          │
│  ── Standard Whisper (multilingual) ──── │
│  ○ Whisper Small  (244 MB)  — 99 langs  │
│  ○ Whisper Medium (769 MB)  — 99 langs  │
│  ○ Whisper Large-v3 (1.5 GB)— 99 langs  │
│    ⚠️ For non-Thai, use standard Whisper  │
│                                          │
│  [Download Selected]                     │
└──────────────────────────────────────────┘
```

### 6.2 LLM Provider Selector

```
┌──────────────────────────────────────────┐
│  LLM Provider                            │
│                                          │
│  ● Ollama (local, offline)               │
│  ○ OpenAI-compatible API                 │
│  ○ Disabled (Fast Mode only)             │
│                                          │
│  ── Ollama Models ──────────────────── │
│  ○ Qwen2.5-1.5B (1.1 GB)               │
│    Fast. RAM: ~2.5 GB                    │
│                                          │
│  ● Qwen2.5-3B (2.0 GB) [Recommended]   │
│    Balanced. RAM: ~4 GB                  │
│                                          │
│  ○ Typhoon2-Qwen2.5-7B (4.7 GB)        │
│    Best Thai. RAM: ~7.5 GB  ⚠️ Needs GPU │
│                                          │
│  Run: ollama pull qwen2.5:3b           │
│  [Copy Command]                          │
│                                          │
│  ── OpenAI-compatible API ────────────── │
│  Base URL: [________________________________] │
│  API Key:  [________________________________] │
│  Model:    [________________________________] │
│  [Test Connection]                       │
└──────────────────────────────────────────┘
```

---

## 7. System Requirements Check

```rust
pub struct SystemCheck {
    pub ram_gb: f32,
    pub cpu_cores: usize,
    pub has_avx2: bool,
    pub gpu: Option<GpuInfo>,
}

impl SystemCheck {
    pub fn run() -> Self {
        Self {
            ram_gb: sys_info::mem_info()
                .map(|m| m.total as f32 / 1_048_576.0)
                .unwrap_or(0.0),
            cpu_cores: num_cpus::get_physical(),
            has_avx2: is_x86_feature_detected!("avx2"),
            gpu: detect_gpu(),
        }
    }

    /// แนะนำ configuration ตาม hardware
    pub fn recommend(&self) -> RecommendedConfig {
        if self.ram_gb >= 16.0 {
            RecommendedConfig {
                stt_model: "thonburian-medium",
                llm_model: "qwen2.5:3b",
                mode: OperatingMode::Standard,
            }
        } else if self.ram_gb >= 8.0 {
            RecommendedConfig {
                stt_model: "thonburian-small",
                llm_model: "qwen2.5:1.5b",
                mode: OperatingMode::Standard,
            }
        } else {
            RecommendedConfig {
                stt_model: "thonburian-small",
                llm_model: "",
                mode: OperatingMode::Fast,
            }
        }
    }
}

pub struct RecommendedConfig {
    pub stt_model: &'static str,
    pub llm_model: &'static str,
    pub mode: OperatingMode,
}
```

---

## 8. Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
dirs = "6.0"
num_cpus = "1.16"
```
