//! Configuration module for Thai Voice-to-Text.
//!
//! Provides `AppConfig` (top-level settings), sub-configs for each subsystem,
//! `AppPaths` for cross-platform data directories, and TOML persistence via
//! `AppConfig::load` / `AppConfig::save`.

pub mod paths;
pub mod settings;

pub use paths::AppPaths;
pub use settings::{
    AppConfig, AudioConfig, HotkeyConfig, LlmConfig, LlmProvider, OperatingMode, SttConfig,
    UiConfig,
};
