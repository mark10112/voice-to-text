//! Global hotkey listener for push-to-talk, backed by `rdev`.
//!
//! # Design
//!
//! `rdev::listen()` is a blocking OS-level call that never returns while the
//! process is alive.  It must run on a **dedicated OS thread** — it cannot be
//! used inside a tokio task.
//!
//! [`HotkeyListener::start`] spawns that dedicated thread and returns a
//! [`HotkeyListener`] handle.  Dropping the handle sets a stop flag so the
//! callback silently discards further events.  The underlying thread will
//! continue to exist until the process exits (rdev has no graceful shutdown
//! API), but it will consume no meaningful CPU while blocked waiting for
//! keyboard events.
//!
//! # Usage
//!
//! ```no_run
//! use tokio::sync::mpsc;
//! use voice_to_text::hotkey::{HotkeyEvent, HotkeyListener, parse_key};
//!
//! let (tx, mut rx) = mpsc::channel(16);
//! let key = parse_key("F9").expect("unknown key");
//! let _listener = HotkeyListener::start(key, tx);
//!
//! // In your async loop:
//! // while let Some(ev) = rx.recv().await { ... }
//! ```

pub mod listener;

pub use listener::HotkeyListener;

// ---------------------------------------------------------------------------
// HotkeyEvent
// ---------------------------------------------------------------------------

/// Events emitted by the hotkey listener thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotkeyEvent {
    /// The push-to-talk key was pressed down.
    PushToTalkPressed,
    /// The push-to-talk key was released.
    PushToTalkReleased,
    /// The visibility-toggle shortcut was activated.
    ToggleVisibility,
}

// ---------------------------------------------------------------------------
// parse_key
// ---------------------------------------------------------------------------

/// Parse a hotkey name from a config string into an [`rdev::Key`].
///
/// Supports F1–F12, common named keys, and single uppercase or lowercase ASCII
/// letters.
///
/// Returns `None` for unrecognised names so callers can fall back to a default
/// or display an error to the user.
///
/// # Examples
///
/// ```
/// use voice_to_text::hotkey::parse_key;
///
/// assert_eq!(parse_key("F9"),      Some(rdev::Key::F9));
/// assert_eq!(parse_key("Escape"),  Some(rdev::Key::Escape));
/// assert_eq!(parse_key("a"),       Some(rdev::Key::KeyA));
/// assert_eq!(parse_key("xyz"),     None);
/// ```
pub fn parse_key(key_str: &str) -> Option<rdev::Key> {
    match key_str {
        // Function keys
        "F1" => Some(rdev::Key::F1),
        "F2" => Some(rdev::Key::F2),
        "F3" => Some(rdev::Key::F3),
        "F4" => Some(rdev::Key::F4),
        "F5" => Some(rdev::Key::F5),
        "F6" => Some(rdev::Key::F6),
        "F7" => Some(rdev::Key::F7),
        "F8" => Some(rdev::Key::F8),
        "F9" => Some(rdev::Key::F9),
        "F10" => Some(rdev::Key::F10),
        "F11" => Some(rdev::Key::F11),
        "F12" => Some(rdev::Key::F12),

        // Navigation / control
        "Escape" | "Esc" => Some(rdev::Key::Escape),
        "Space" => Some(rdev::Key::Space),
        "Return" | "Enter" => Some(rdev::Key::Return),
        "Tab" => Some(rdev::Key::Tab),
        "Backspace" => Some(rdev::Key::Backspace),
        "Delete" | "Del" => Some(rdev::Key::Delete),
        "Home" => Some(rdev::Key::Home),
        "End" => Some(rdev::Key::End),
        "PageUp" => Some(rdev::Key::PageUp),
        "PageDown" => Some(rdev::Key::PageDown),
        "UpArrow" | "Up" => Some(rdev::Key::UpArrow),
        "DownArrow" | "Down" => Some(rdev::Key::DownArrow),
        "LeftArrow" | "Left" => Some(rdev::Key::LeftArrow),
        "RightArrow" | "Right" => Some(rdev::Key::RightArrow),

        // Lock / special
        "CapsLock" => Some(rdev::Key::CapsLock),
        "NumLock" => Some(rdev::Key::NumLock),
        "ScrollLock" => Some(rdev::Key::ScrollLock),
        "PrintScreen" => Some(rdev::Key::PrintScreen),
        "Pause" => Some(rdev::Key::Pause),

        // Letter keys (case-insensitive)
        "A" | "a" => Some(rdev::Key::KeyA),
        "B" | "b" => Some(rdev::Key::KeyB),
        "C" | "c" => Some(rdev::Key::KeyC),
        "D" | "d" => Some(rdev::Key::KeyD),
        "E" | "e" => Some(rdev::Key::KeyE),
        "F" | "f" => Some(rdev::Key::KeyF),
        "G" | "g" => Some(rdev::Key::KeyG),
        "H" | "h" => Some(rdev::Key::KeyH),
        "I" | "i" => Some(rdev::Key::KeyI),
        "J" | "j" => Some(rdev::Key::KeyJ),
        "K" | "k" => Some(rdev::Key::KeyK),
        "L" | "l" => Some(rdev::Key::KeyL),
        "M" | "m" => Some(rdev::Key::KeyM),
        "N" | "n" => Some(rdev::Key::KeyN),
        "O" | "o" => Some(rdev::Key::KeyO),
        "P" | "p" => Some(rdev::Key::KeyP),
        "Q" | "q" => Some(rdev::Key::KeyQ),
        "R" | "r" => Some(rdev::Key::KeyR),
        "S" | "s" => Some(rdev::Key::KeyS),
        "T" | "t" => Some(rdev::Key::KeyT),
        "U" | "u" => Some(rdev::Key::KeyU),
        "V" | "v" => Some(rdev::Key::KeyV),
        "W" | "w" => Some(rdev::Key::KeyW),
        "X" | "x" => Some(rdev::Key::KeyX),
        "Y" | "y" => Some(rdev::Key::KeyY),
        "Z" | "z" => Some(rdev::Key::KeyZ),

        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_function_keys() {
        assert_eq!(parse_key("F9"), Some(rdev::Key::F9));
        assert_eq!(parse_key("F1"), Some(rdev::Key::F1));
        assert_eq!(parse_key("F12"), Some(rdev::Key::F12));
    }

    #[test]
    fn parse_named_keys() {
        assert_eq!(parse_key("Escape"), Some(rdev::Key::Escape));
        assert_eq!(parse_key("Esc"), Some(rdev::Key::Escape));
        assert_eq!(parse_key("Space"), Some(rdev::Key::Space));
        assert_eq!(parse_key("Return"), Some(rdev::Key::Return));
        assert_eq!(parse_key("Enter"), Some(rdev::Key::Return));
    }

    #[test]
    fn parse_letter_keys_case_insensitive() {
        assert_eq!(parse_key("A"), Some(rdev::Key::KeyA));
        assert_eq!(parse_key("a"), Some(rdev::Key::KeyA));
        assert_eq!(parse_key("Z"), Some(rdev::Key::KeyZ));
        assert_eq!(parse_key("z"), Some(rdev::Key::KeyZ));
    }

    #[test]
    fn parse_unknown_key_returns_none() {
        assert_eq!(parse_key("xyz"), None);
        assert_eq!(parse_key(""), None);
        assert_eq!(parse_key("Ctrl+V"), None);
    }
}
