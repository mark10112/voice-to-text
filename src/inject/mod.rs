//! Text injection module — clipboard-based text injection for Thai voice-to-text.
//!
//! # Overview
//!
//! Thai has combining characters (floating vowels, tone marks) that are very
//! difficult to inject via raw key events.  Instead we use the clipboard:
//!
//! 1. **Save** the original clipboard content.
//! 2. **Set** the corrected text into the clipboard.
//! 3. **Simulate** Ctrl+V (or ⌘V on macOS) to paste into the focused window.
//! 4. **Restore** the original clipboard content (best-effort).
//!
//! # Usage
//!
//! ```no_run
//! use voice_to_text::inject::inject_text;
//!
//! inject_text("สวัสดีครับ").expect("injection failed");
//! ```

pub mod clipboard;
pub mod keyboard;
pub mod thai;

pub use clipboard::{restore_clipboard, save_clipboard, set_clipboard};
pub use keyboard::simulate_paste;
pub use thai::validate_thai_text;

use thiserror::Error;

// ---------------------------------------------------------------------------
// InjectError
// ---------------------------------------------------------------------------

/// All errors that can surface during text injection.
#[derive(Debug, Error)]
pub enum InjectError {
    /// Could not open or read the system clipboard.
    #[error("cannot access clipboard: {0}")]
    ClipboardAccess(String),

    /// Could not write text to the system clipboard.
    #[error("cannot set clipboard text: {0}")]
    ClipboardSet(String),

    /// Could not simulate a key press/release event.
    #[error("cannot simulate key press: {0}")]
    KeySimulation(String),

    /// The target window lost focus before the paste completed.
    #[error("target window lost focus before paste")]
    TargetWindowLost,

    /// The supplied text did not pass Thai validation.
    #[error("text validation failed: {0}")]
    Validation(String),
}

// ---------------------------------------------------------------------------
// inject_text  — free-function convenience API
// ---------------------------------------------------------------------------

/// Full clipboard-paste injection pipeline.
///
/// Steps (in order):
/// 1. Save the current clipboard plain-text content.
/// 2. Write `text` into the clipboard.
/// 3. Wait 50 ms (clipboard flush).
/// 4. Simulate Ctrl+V / ⌘V.
/// 5. Wait 100 ms (let the target app complete the paste).
/// 6. Restore the original clipboard content (best-effort; errors ignored).
///
/// # Errors
///
/// Returns the first [`InjectError`] encountered in steps 1–4.  The restore
/// in step 6 is always attempted but its result is discarded.
pub fn inject_text(text: &str) -> Result<(), InjectError> {
    // 1. Save
    let saved = save_clipboard()?;

    // 2. Set
    set_clipboard(text)?;

    // 3. Small delay so the clipboard manager flushes before the target reads it
    std::thread::sleep(std::time::Duration::from_millis(50));

    // 4. Paste
    simulate_paste()?;

    // 5. Let the target app finish pasting before we clobber the clipboard
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 6. Restore (best-effort)
    let _ = restore_clipboard(saved);

    Ok(())
}

// ---------------------------------------------------------------------------
// TextInjector  — struct API with configurable delays
// ---------------------------------------------------------------------------

/// Configurable text injector.
///
/// For most callers the free-function [`inject_text`] is sufficient.  Use
/// `TextInjector` when you need to customise the inter-step delays (e.g. on
/// slow systems or when targeting apps with sluggish clipboard handling).
#[derive(Debug, Clone)]
pub struct TextInjector {
    /// Milliseconds to wait after setting the clipboard before simulating paste.
    pub delay_ms: u64,
    /// Milliseconds to wait after simulating paste before restoring the
    /// original clipboard.
    pub restore_delay_ms: u64,
}

impl Default for TextInjector {
    fn default() -> Self {
        Self {
            delay_ms: 50,
            restore_delay_ms: 100,
        }
    }
}

impl TextInjector {
    /// Create a `TextInjector` with the default delays (50 ms / 100 ms).
    pub fn new() -> Self {
        Self::default()
    }

    /// Inject `text` using this injector's configured delays.
    pub fn inject(&self, text: &str) -> Result<(), InjectError> {
        let saved = save_clipboard()?;
        set_clipboard(text)?;
        std::thread::sleep(std::time::Duration::from_millis(self.delay_ms));
        simulate_paste()?;
        std::thread::sleep(std::time::Duration::from_millis(self.restore_delay_ms));
        let _ = restore_clipboard(saved);
        Ok(())
    }
}
