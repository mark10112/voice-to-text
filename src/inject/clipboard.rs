//! Clipboard save / restore helpers backed by the `arboard` crate.
//!
//! All three functions create a short-lived [`arboard::Clipboard`] handle
//! rather than sharing one across calls, because `arboard::Clipboard` is not
//! `Send` on all platforms and the clipboard handle is cheap to create.

use arboard::Clipboard;

use super::InjectError;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Capture the current clipboard plain-text content.
///
/// Returns `Ok(None)` when the clipboard is empty or contains non-text data
/// (e.g. an image).  Never returns an error just because the clipboard is
/// empty.
///
/// # Errors
///
/// Returns [`InjectError::ClipboardAccess`] if the OS clipboard cannot be
/// opened.
pub fn save_clipboard() -> Result<Option<String>, InjectError> {
    let mut clipboard = open_clipboard()?;
    // `get_text` returns Err if empty or non-text — treat both as None
    Ok(clipboard.get_text().ok())
}

/// Write `text` into the system clipboard, replacing whatever was there.
///
/// # Errors
///
/// Returns [`InjectError::ClipboardAccess`] if the clipboard cannot be opened,
/// or [`InjectError::ClipboardSet`] if writing fails.
pub fn set_clipboard(text: &str) -> Result<(), InjectError> {
    let mut clipboard = open_clipboard()?;
    clipboard
        .set_text(text)
        .map_err(|e| InjectError::ClipboardSet(e.to_string()))
}

/// Restore the clipboard to a previously saved value.
///
/// * `Some(text)` — writes `text` back to the clipboard.
/// * `None` — nothing was saved (clipboard was empty or non-text before the
///   injection); this function returns `Ok(())` immediately without modifying
///   the clipboard.
///
/// # Errors
///
/// Propagates errors from [`set_clipboard`].
pub fn restore_clipboard(saved: Option<String>) -> Result<(), InjectError> {
    match saved {
        Some(text) => set_clipboard(&text),
        None => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Open an `arboard::Clipboard` handle, mapping the error to [`InjectError`].
fn open_clipboard() -> Result<Clipboard, InjectError> {
    Clipboard::new().map_err(|e| InjectError::ClipboardAccess(e.to_string()))
}
