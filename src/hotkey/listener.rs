//! Dedicated OS-thread hotkey listener using `rdev::listen`.
//!
//! `rdev::listen` is a blocking call that must live on its own OS thread.
//! [`HotkeyListener`] owns that thread and a stop flag; dropping it sets the
//! flag so the callback silently ignores further events.
//!
//! # Shutdown caveat
//!
//! `rdev::listen` has **no graceful shutdown API**.  Setting the stop flag
//! prevents events from being forwarded, but the OS thread itself will remain
//! blocked in the rdev event loop until the process exits.  This is safe and
//! expected — rdev holds no resources that need explicit cleanup.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tokio::sync::mpsc;

use super::HotkeyEvent;

// ---------------------------------------------------------------------------
// HotkeyListener
// ---------------------------------------------------------------------------

/// Handle to a running hotkey listener thread.
///
/// Construct one with [`HotkeyListener::start`].  Drop it to stop forwarding
/// events.
///
/// The underlying OS thread will continue to exist until the process exits
/// because `rdev::listen` cannot be interrupted, but it will silently discard
/// all events once the stop flag is set.
pub struct HotkeyListener {
    /// Shared stop flag — set `true` on [`Drop`].
    stop: Arc<AtomicBool>,
    /// The thread handle.  Kept alive so the thread is not detached
    /// prematurely; we never `join` it because `rdev::listen` never returns.
    _thread: std::thread::JoinHandle<()>,
}

impl HotkeyListener {
    /// Spawn a dedicated OS thread that listens for global key events and
    /// forwards `PushToTalkPressed` / `PushToTalkReleased` events on `tx`
    /// whenever `key` is pressed or released.
    ///
    /// # Arguments
    ///
    /// * `key` — The [`rdev::Key`] to watch (e.g. `rdev::Key::F9`).
    ///   Use [`crate::hotkey::parse_key`] to obtain this from a config string.
    /// * `tx`  — A `tokio::sync::mpsc` sender.  The background thread uses
    ///   `blocking_send` so it works correctly from a non-async context.
    ///
    /// # Returns
    ///
    /// A [`HotkeyListener`] whose drop will stop event forwarding.
    ///
    /// # Panics
    ///
    /// Panics if the OS refuses to create the thread (extremely unlikely).
    pub fn start(key: rdev::Key, tx: mpsc::Sender<HotkeyEvent>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);

        let thread = std::thread::Builder::new()
            .name("hotkey-listener".into())
            .spawn(move || {
                let result = rdev::listen(move |event| {
                    // Bail out if the listener has been stopped.
                    if stop_clone.load(Ordering::Relaxed) {
                        return;
                    }

                    match event.event_type {
                        rdev::EventType::KeyPress(k) if k == key => {
                            // blocking_send is safe to call from non-async threads.
                            let _ = tx.blocking_send(HotkeyEvent::PushToTalkPressed);
                        }
                        rdev::EventType::KeyRelease(k) if k == key => {
                            let _ = tx.blocking_send(HotkeyEvent::PushToTalkReleased);
                        }
                        _ => {}
                    }
                });

                if let Err(e) = result {
                    log::error!("hotkey-listener: rdev::listen exited with error: {:?}", e);
                }
            })
            .expect("failed to spawn hotkey-listener thread");

        Self {
            stop,
            _thread: thread,
        }
    }
}

impl Drop for HotkeyListener {
    /// Set the stop flag so the rdev callback stops forwarding events.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        // The OS thread continues to exist blocked inside rdev::listen until
        // the process exits — this is safe and requires no further cleanup.
    }
}
