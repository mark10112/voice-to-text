//! Keyboard simulation helpers backed by the `enigo` crate.
//!
//! Provides [`simulate_paste`], which sends the OS-appropriate paste shortcut
//! to the currently focused window:
//!
//! | Platform | Shortcut |
//! |----------|----------|
//! | macOS    | ⌘V (Meta + V) |
//! | Windows  | Ctrl+V |
//! | Linux    | Ctrl+V |

use enigo::{Direction, Enigo, Key, Keyboard, Settings};

use super::InjectError;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Simulate the system paste shortcut in the currently focused window.
///
/// * **macOS** → Meta (⌘) + V
/// * **Windows / Linux** → Ctrl + V
///
/// A new [`Enigo`] instance is created for each call because `Enigo` is not
/// `Send` and the handle is cheap to construct.
///
/// # Errors
///
/// Returns [`InjectError::KeySimulation`] if the enigo backend cannot be
/// initialised or if any key event fails to be delivered.
pub fn simulate_paste() -> Result<(), InjectError> {
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| InjectError::KeySimulation(e.to_string()))?;

    #[cfg(target_os = "macos")]
    {
        // macOS: ⌘V
        enigo
            .key(Key::Meta, Direction::Press)
            .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
        enigo
            .key(Key::Meta, Direction::Release)
            .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Windows / Linux: Ctrl+V
        enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
        enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
    }

    Ok(())
}
