//! Application settings structs, defaults and TOML persistence.
//!
//! All structs implement `Serialize`, `Deserialize`, `Default` and `Clone`
//! so they can be round-tripped through TOML files and shared across threads.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::AppPaths;

// ---------------------------------------------------------------------------
// OperatingMode
// ---------------------------------------------------------------------------

/// Selects the processing pipeline complexity.
///
/// | Variant  | Pipeline                                | Requires Ollama |
/// |----------|-----------------------------------------|-----------------|
/// | Fast     | STT → Inject (no LLM)                  | No              |
/// | Standard | STT → LLM (zero-shot) → Inject         | Yes             |
/// | Context  | STT → LLM (context + vocab) → Inject   | Yes             |
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OperatingMode {
    /// STT only — lowest latency (~5-15 s on CPU).
    Fast,
    /// STT + LLM correction (zero-shot) — balanced latency (~8-20 s on CPU).
    Standard,
    /// STT + LLM correction with rolling context, domain detection, and user
    /// vocabulary — highest quality (~10-25 s on CPU).
    Context,
}

impl Default for OperatingMode {
    fn default() -> Self {
        Self::Standard
    }
}

// ---------------------------------------------------------------------------
// LlmProvider
// ---------------------------------------------------------------------------

/// Selects which LLM backend handles post-processing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LlmProvider {
    /// Ollama running locally — no authentication required.
    Ollama,
    /// Any OpenAI-compatible REST API (OpenAI, Groq, Together.ai, LM Studio …).
    OpenAiCompatible,
    /// In-process inference via the `llama_cpp` crate (Phase 2).
    LlamaCpp,
    /// LLM disabled — forces Fast mode.
    Disabled,
}

impl Default for LlmProvider {
    fn default() -> Self {
        Self::Ollama
    }
}

// ---------------------------------------------------------------------------
// LlmConfig
// ---------------------------------------------------------------------------

/// Settings for the LLM post-processing step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Whether LLM correction is active at all.
    pub enabled: bool,
    /// Which backend to use.
    pub provider: LlmProvider,
    /// Base URL of the API endpoint.
    ///
    /// - Ollama default: `http://localhost:11434`
    /// - OpenAI: `https://api.openai.com`
    pub base_url: String,
    /// API key — `None` for local providers (Ollama / LlamaCpp).
    pub api_key: Option<String>,
    /// Model identifier sent to the API (e.g. `"qwen2.5:3b"`, `"gpt-4o-mini"`).
    pub model: String,
    /// Sampling temperature (0.0 – 1.0).  Lower = more deterministic.
    pub temperature: f32,
    /// Maximum seconds to wait for an LLM response before timing out.
    pub timeout_secs: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: LlmProvider::default(),
            base_url: "http://localhost:11434".into(),
            api_key: None,
            model: "qwen2.5:3b".into(),
            temperature: 0.3,
            timeout_secs: 10,
        }
    }
}

// ---------------------------------------------------------------------------
// SttConfig
// ---------------------------------------------------------------------------

/// Settings for the Whisper STT engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    /// GGML model name / file stem (e.g. `"thonburian-medium"`).
    pub model: String,
    /// Primary speech language as an ISO-639-1 code, or `"auto"` for
    /// Whisper's built-in language detection.
    pub language: String,
    /// Attempt GPU-accelerated inference when available.
    pub use_gpu: bool,
    /// Audio input device name — `None` means the system default.
    pub audio_device: Option<String>,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            model: "thonburian-medium".into(),
            language: "th".into(),
            use_gpu: false,
            audio_device: None,
        }
    }
}

// ---------------------------------------------------------------------------
// AudioConfig
// ---------------------------------------------------------------------------

/// Settings for audio capture and voice-activity detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Target sample rate in Hz passed to Whisper (must be 16 000).
    pub sample_rate: u32,
    /// Silero VAD confidence threshold (0.0 – 1.0); speech above this level
    /// is considered voice activity.
    pub vad_threshold: f32,
    /// Minimum recording length in seconds before transcription is attempted.
    pub min_recording_secs: f32,
    /// Maximum recording length in seconds; recording stops automatically.
    pub max_recording_secs: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16_000,
            vad_threshold: 0.5,
            min_recording_secs: 0.5,
            max_recording_secs: 60.0,
        }
    }
}

// ---------------------------------------------------------------------------
// HotkeyConfig
// ---------------------------------------------------------------------------

/// Global hotkey bindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Push-to-talk key name (e.g. `"F9"`).
    pub push_to_talk_key: String,
    /// Key combination that toggles widget visibility (e.g. `"Ctrl+Shift+T"`).
    pub toggle_visibility_key: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            push_to_talk_key: "F9".into(),
            toggle_visibility_key: "Ctrl+Shift+T".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// UiConfig
// ---------------------------------------------------------------------------

/// egui widget appearance and behaviour settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Last saved widget position `(x, y)` in screen pixels.  `None` means
    /// let the OS / window manager pick a position on first launch.
    pub window_position: Option<(f32, f32)>,
    /// Keep the widget floating above all other windows.
    pub always_on_top: bool,
    /// Automatically inject the final text without requiring a manual
    /// "inject" button press.
    pub auto_inject: bool,
    /// Display the raw (pre-correction) STT transcript while LLM is running.
    pub show_raw_text: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            window_position: None,
            always_on_top: true,
            auto_inject: true,
            show_raw_text: true,
        }
    }
}

// ---------------------------------------------------------------------------
// ContextConfig  (rolling-context window, used in Context mode)
// ---------------------------------------------------------------------------

/// Settings that control the rolling context window used in Context mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Number of previous sentences kept in the rolling context window.
    pub window_size: usize,
    /// Seconds of silence after which the context window is automatically
    /// cleared (topic change assumed).
    pub reset_silence_secs: u64,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            window_size: 3,
            reset_silence_secs: 120,
        }
    }
}

// ---------------------------------------------------------------------------
// AppConfig  (top-level)
// ---------------------------------------------------------------------------

/// Top-level application configuration, serialised as `settings.toml`.
///
/// # Persistence
///
/// ```rust,no_run
/// use voice_to_text::config::AppConfig;
///
/// // Load (returns Default when file is missing)
/// let config = AppConfig::load().unwrap();
///
/// // Modify and save
/// // config.save().unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Selected operating mode.
    pub operating_mode: OperatingMode,
    /// LLM post-processing settings.
    pub llm: LlmConfig,
    /// STT engine settings.
    pub stt: SttConfig,
    /// Audio capture / VAD settings.
    pub audio: AudioConfig,
    /// Global hotkey bindings.
    pub hotkey: HotkeyConfig,
    /// UI / widget settings.
    pub ui: UiConfig,
    /// Rolling context window settings (Context mode only).
    pub context: ContextConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            operating_mode: OperatingMode::default(),
            llm: LlmConfig::default(),
            stt: SttConfig::default(),
            audio: AudioConfig::default(),
            hotkey: HotkeyConfig::default(),
            ui: UiConfig::default(),
            context: ContextConfig::default(),
        }
    }
}

impl AppConfig {
    /// Load configuration from the platform-appropriate `settings.toml`.
    ///
    /// Returns `Ok(AppConfig::default())` when the file does not exist yet
    /// (first-run scenario) so callers never need to special-case a missing
    /// file.
    pub fn load() -> Result<Self> {
        Self::load_from(&AppPaths::new().settings_file)
    }

    /// Load from an explicit path (useful for tests).
    pub fn load_from(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to the platform-appropriate `settings.toml`,
    /// creating parent directories as needed.
    pub fn save(&self) -> Result<()> {
        self.save_to(&AppPaths::new().settings_file)
    }

    /// Save to an explicit path (useful for tests).
    pub fn save_to(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Returns `true` when no `settings.toml` file exists yet — first-run
    /// detection used by the setup wizard.
    pub fn is_first_run() -> bool {
        !AppPaths::new().settings_file.exists()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Verify that a default `AppConfig` can be serialised to TOML and
    /// deserialised back without any data loss.
    #[test]
    fn round_trip_toml() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("settings.toml");

        let original = AppConfig::default();
        original.save_to(&path).expect("save");

        let loaded = AppConfig::load_from(&path).expect("load");

        // OperatingMode
        assert_eq!(original.operating_mode, loaded.operating_mode);

        // LlmConfig
        assert_eq!(original.llm.base_url, loaded.llm.base_url);
        assert_eq!(original.llm.api_key, loaded.llm.api_key);
        assert_eq!(original.llm.model, loaded.llm.model);
        assert_eq!(original.llm.timeout_secs, loaded.llm.timeout_secs);
        assert_eq!(original.llm.temperature, loaded.llm.temperature);

        // SttConfig
        assert_eq!(original.stt.model, loaded.stt.model);
        assert_eq!(original.stt.language, loaded.stt.language);
        assert_eq!(original.stt.use_gpu, loaded.stt.use_gpu);

        // AudioConfig
        assert_eq!(original.audio.sample_rate, loaded.audio.sample_rate);
        assert_eq!(original.audio.vad_threshold, loaded.audio.vad_threshold);
        assert_eq!(
            original.audio.min_recording_secs,
            loaded.audio.min_recording_secs
        );
        assert_eq!(
            original.audio.max_recording_secs,
            loaded.audio.max_recording_secs
        );

        // HotkeyConfig
        assert_eq!(
            original.hotkey.push_to_talk_key,
            loaded.hotkey.push_to_talk_key
        );

        // UiConfig
        assert_eq!(original.ui.always_on_top, loaded.ui.always_on_top);
        assert_eq!(original.ui.auto_inject, loaded.ui.auto_inject);
    }

    /// `load_from` on a non-existent path must return `Default` without error.
    #[test]
    fn load_missing_returns_default() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("nonexistent.toml");

        let config = AppConfig::load_from(&path).expect("should not error");
        let default = AppConfig::default();

        assert_eq!(config.operating_mode, default.operating_mode);
        assert_eq!(config.llm.model, default.llm.model);
        assert_eq!(config.stt.language, default.stt.language);
        assert_eq!(config.audio.sample_rate, default.audio.sample_rate);
        assert_eq!(
            config.hotkey.push_to_talk_key,
            default.hotkey.push_to_talk_key
        );
    }

    /// Verify default values match the design spec.
    #[test]
    fn default_values_match_spec() {
        let cfg = AppConfig::default();

        assert_eq!(cfg.operating_mode, OperatingMode::Standard);
        assert_eq!(cfg.llm.base_url, "http://localhost:11434");
        assert_eq!(cfg.llm.model, "qwen2.5:3b");
        assert_eq!(cfg.llm.timeout_secs, 10);
        assert!(cfg.llm.api_key.is_none());
        assert_eq!(cfg.stt.model, "thonburian-medium");
        assert_eq!(cfg.stt.language, "th");
        assert_eq!(cfg.audio.sample_rate, 16_000);
        assert_eq!(cfg.hotkey.push_to_talk_key, "F9");
        assert!(cfg.ui.always_on_top);
        assert_eq!(cfg.context.window_size, 3);
    }

    /// Verify that modified non-default values survive a round trip.
    #[test]
    fn round_trip_modified_values() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("modified.toml");

        let mut cfg = AppConfig::default();
        cfg.operating_mode = OperatingMode::Context;
        cfg.llm.base_url = "https://api.openai.com".into();
        cfg.llm.api_key = Some("sk-test".into());
        cfg.llm.model = "gpt-4o-mini".into();
        cfg.llm.timeout_secs = 30;
        cfg.stt.language = "en".into();
        cfg.audio.sample_rate = 16_000;
        cfg.ui.window_position = Some((100.0, 200.0));
        cfg.hotkey.push_to_talk_key = "F10".into();

        cfg.save_to(&path).expect("save");
        let loaded = AppConfig::load_from(&path).expect("load");

        assert_eq!(loaded.operating_mode, OperatingMode::Context);
        assert_eq!(loaded.llm.base_url, "https://api.openai.com");
        assert_eq!(loaded.llm.api_key, Some("sk-test".into()));
        assert_eq!(loaded.llm.model, "gpt-4o-mini");
        assert_eq!(loaded.llm.timeout_secs, 30);
        assert_eq!(loaded.stt.language, "en");
        assert_eq!(loaded.ui.window_position, Some((100.0, 200.0)));
        assert_eq!(loaded.hotkey.push_to_talk_key, "F10");
    }
}
