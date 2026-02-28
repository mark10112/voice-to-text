//! Cross-platform application paths using the `dirs` crate.
//!
//! Layout:
//!
//! Config dir (settings + user data):
//!   Windows: %APPDATA%\voice-to-text\
//!   macOS:   ~/Library/Application Support/voice-to-text/
//!   Linux:   ~/.config/voice-to-text/
//!
//! Data dir (models):
//!   Windows: %LOCALAPPDATA%\voice-to-text\
//!   macOS:   ~/Library/Application Support/voice-to-text/
//!   Linux:   ~/.local/share/voice-to-text/

use std::path::PathBuf;

/// Holds all resolved application directory/file paths.
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// Directory for `settings.toml` and `user-vocab.json`.
    pub config_dir: PathBuf,
    /// Full path to `settings.toml`.
    pub settings_file: PathBuf,
    /// Full path to `user-vocab.json`.
    pub user_vocab_file: PathBuf,
    /// Directory for downloaded GGML model files.
    pub models_dir: PathBuf,
}

impl AppPaths {
    const APP_NAME: &'static str = "voice-to-text";

    /// Resolves all paths using the `dirs` crate.
    ///
    /// Falls back to the current directory if the platform cannot provide a
    /// standard path (should be extremely rare in practice).
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(Self::APP_NAME);

        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(Self::APP_NAME);

        let settings_file = config_dir.join("settings.toml");
        let user_vocab_file = config_dir.join("user-vocab.json");
        let models_dir = data_dir.join("models");

        Self {
            config_dir,
            settings_file,
            user_vocab_file,
            models_dir,
        }
    }
}

impl Default for AppPaths {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_non_empty() {
        let paths = AppPaths::new();
        assert!(paths.config_dir.to_str().is_some_and(|s| !s.is_empty()));
        assert!(paths.models_dir.to_str().is_some_and(|s| !s.is_empty()));
        assert!(paths
            .settings_file
            .file_name()
            .is_some_and(|n| n == "settings.toml"));
        assert!(paths
            .user_vocab_file
            .file_name()
            .is_some_and(|n| n == "user-vocab.json"));
    }
}
