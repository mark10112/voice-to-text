//! Model registry, metadata and path resolution.
//!
//! Two const arrays are provided:
//! - [`THAI_MODELS`]   — Thonburian Whisper (fine-tuned for Thai, ICNLSP 2024)
//! - [`WHISPER_MODELS`] — Standard multilingual Whisper models
//!
//! [`ModelPaths`] resolves the on-disk location of a model given an
//! [`crate::config::AppPaths`] instance.

use std::path::PathBuf;

use crate::config::AppPaths;

// ---------------------------------------------------------------------------
// ModelSize
// ---------------------------------------------------------------------------

/// Approximate capacity tier of a Whisper GGML model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelSize {
    /// ~242 MB file / ~900 MB RAM — fastest, lowest accuracy.
    Small,
    /// ~769 MB file / ~3 GB RAM — balanced (recommended). ✅
    Medium,
    /// ~1.5 GB file / ~6 GB RAM — highest accuracy, slowest.
    Large,
}

// ---------------------------------------------------------------------------
// ModelInfo
// ---------------------------------------------------------------------------

/// Static metadata for a single GGML model file.
#[derive(Debug)]
pub struct ModelInfo {
    /// Unique identifier used in `SttConfig::model` (e.g. `"thonburian-medium"`).
    pub id: &'static str,
    /// Human-readable display name shown in the UI.
    pub display_name: &'static str,
    /// Model capacity tier.
    pub size: ModelSize,
    /// File name under the models directory (e.g. `"ggml-thonburian-medium.bin"`).
    pub file_name: &'static str,
    /// Approximate compressed file size in megabytes.
    pub file_size_mb: u64,
    /// Minimum RAM required to run this model (megabytes).
    pub ram_required_mb: u64,
    /// Source URL for downloading the GGML file.
    pub source_url: &'static str,
    /// ISO-639-1 language code this model is optimised for, or
    /// `"multilingual"` for the standard Whisper models.
    pub language: &'static str,
}

// ---------------------------------------------------------------------------
// Thai-optimised models (Thonburian Whisper)
// ---------------------------------------------------------------------------

/// Thonburian Whisper models — fine-tuned on Thai speech (ICNLSP 2024).
///
/// Use these when `SttConfig::language == "th"` for the best CER on Thai.
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
        ram_required_mb: 3_000,
        source_url: "https://huggingface.co/biodatlab/whisper-th-medium-combined",
        language: "th",
    },
    ModelInfo {
        id: "thonburian-large",
        display_name: "Thonburian Whisper Large (Thai)",
        size: ModelSize::Large,
        file_name: "ggml-thonburian-large.bin",
        file_size_mb: 1_500,
        ram_required_mb: 6_000,
        source_url: "https://huggingface.co/biodatlab/whisper-th-large-combined",
        language: "th",
    },
];

// ---------------------------------------------------------------------------
// Standard Whisper models (multilingual)
// ---------------------------------------------------------------------------

/// Standard OpenAI Whisper models (99-language multilingual).
///
/// Use these when `SttConfig::language != "th"` or when `"auto"` language
/// detection is desired.
pub const WHISPER_MODELS: &[ModelInfo] = &[
    ModelInfo {
        id: "whisper-small",
        display_name: "Whisper Small (Multilingual, 99 langs)",
        size: ModelSize::Small,
        file_name: "ggml-whisper-small.bin",
        file_size_mb: 244,
        ram_required_mb: 1_000,
        source_url: "https://huggingface.co/ggerganov/whisper.cpp",
        language: "multilingual",
    },
    ModelInfo {
        id: "whisper-medium",
        display_name: "Whisper Medium (Multilingual, 99 langs)",
        size: ModelSize::Medium,
        file_name: "ggml-whisper-medium.bin",
        file_size_mb: 769,
        ram_required_mb: 3_000,
        source_url: "https://huggingface.co/ggerganov/whisper.cpp",
        language: "multilingual",
    },
    ModelInfo {
        id: "whisper-large-v3",
        display_name: "Whisper Large-v3 (Multilingual, 99 langs)",
        size: ModelSize::Large,
        file_name: "ggml-whisper-large-v3.bin",
        file_size_mb: 1_550,
        ram_required_mb: 6_000,
        source_url: "https://huggingface.co/ggerganov/whisper.cpp",
        language: "multilingual",
    },
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the recommended model list for the given ISO-639-1 language code.
///
/// `"th"` → [`THAI_MODELS`]; everything else → [`WHISPER_MODELS`].
pub fn models_for_language(language: &str) -> &'static [ModelInfo] {
    if language == "th" {
        THAI_MODELS
    } else {
        WHISPER_MODELS
    }
}

/// Find a [`ModelInfo`] by its `id` string, searching both registries.
pub fn find_model_by_id(id: &str) -> Option<&'static ModelInfo> {
    THAI_MODELS
        .iter()
        .chain(WHISPER_MODELS.iter())
        .find(|m| m.id == id)
}

// ---------------------------------------------------------------------------
// ModelPaths
// ---------------------------------------------------------------------------

/// Resolves the on-disk location of model files from [`AppPaths`].
///
/// ```rust,no_run
/// use voice_to_text::config::AppPaths;
/// use voice_to_text::stt::{ModelPaths, THAI_MODELS};
///
/// let paths = ModelPaths::from_app_paths(&AppPaths::new());
/// let available: Vec<_> = THAI_MODELS.iter()
///     .filter(|m| paths.is_available(m))
///     .collect();
/// ```
#[derive(Debug, Clone)]
pub struct ModelPaths {
    /// Directory that contains (or will contain) GGML `.bin` files.
    pub models_dir: PathBuf,
}

impl ModelPaths {
    /// Build a [`ModelPaths`] from the application's [`AppPaths`].
    pub fn from_app_paths(app_paths: &AppPaths) -> Self {
        Self {
            models_dir: app_paths.models_dir.clone(),
        }
    }

    /// Construct directly from a models directory path (useful in tests).
    pub fn new(models_dir: impl Into<PathBuf>) -> Self {
        Self {
            models_dir: models_dir.into(),
        }
    }

    /// Full path to the GGML file for the given model.
    pub fn model_path(&self, model: &ModelInfo) -> PathBuf {
        self.models_dir.join(model.file_name)
    }

    /// Returns `true` if the model file exists on disk.
    pub fn is_available(&self, model: &ModelInfo) -> bool {
        self.model_path(model).exists()
    }

    /// Returns all models (from both registries) that are present on disk.
    pub fn list_local_models(&self) -> Vec<&'static ModelInfo> {
        THAI_MODELS
            .iter()
            .chain(WHISPER_MODELS.iter())
            .filter(|m| self.is_available(m))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thai_models_have_correct_language() {
        for m in THAI_MODELS {
            assert_eq!(m.language, "th", "model {} should be 'th'", m.id);
        }
    }

    #[test]
    fn whisper_models_have_multilingual_language() {
        for m in WHISPER_MODELS {
            assert_eq!(
                m.language, "multilingual",
                "model {} should be 'multilingual'",
                m.id
            );
        }
    }

    #[test]
    fn models_for_language_routes_correctly() {
        let thai = models_for_language("th");
        assert!(!thai.is_empty());
        assert!(thai.iter().all(|m| m.language == "th"));

        let multi = models_for_language("en");
        assert!(!multi.is_empty());
        assert!(multi.iter().all(|m| m.language == "multilingual"));
    }

    #[test]
    fn find_model_by_id_known() {
        let m = find_model_by_id("thonburian-medium");
        assert!(m.is_some());
        assert_eq!(m.unwrap().size, ModelSize::Medium);
    }

    #[test]
    fn find_model_by_id_unknown() {
        assert!(find_model_by_id("does-not-exist").is_none());
    }

    #[test]
    fn model_paths_non_existent_returns_false() {
        let mp = ModelPaths::new("/nonexistent/path");
        let model = &THAI_MODELS[0];
        assert!(!mp.is_available(model));
    }

    #[test]
    fn model_paths_correct_file_name() {
        let mp = ModelPaths::new("/models");
        let model = &THAI_MODELS[1]; // thonburian-medium
        let p = mp.model_path(model);
        assert!(p.to_str().unwrap().ends_with("ggml-thonburian-medium.bin"));
    }
}
