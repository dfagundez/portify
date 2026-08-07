//! The system-wide hotkey that summons the window.
//!
//! A tray utility you have to go looking for is a tray utility you stop using,
//! so the window is reachable from anywhere without touching the mouse.

use std::str::FromStr;

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::window;

/// Register `accelerator` as the show/hide hotkey, replacing any previous one.
///
/// An empty string disables the hotkey. The accelerator is parsed *before*
/// anything is unregistered, so a typo leaves the working shortcut in place
/// rather than silently removing it.
pub fn apply(app: &AppHandle, accelerator: &str) -> Result<(), String> {
    let trimmed = accelerator.trim();

    let parsed = if trimmed.is_empty() {
        None
    } else {
        // The upstream parse error names a third-party issue tracker and asks
        // the reader to file a bug there, which is noise for someone who simply
        // mistyped. Say what is wrong and what a good value looks like instead.
        Some(Shortcut::from_str(trimmed).map_err(|_| {
            format!("\"{trimmed}\" is not a valid shortcut. Combine modifiers with one key, like Ctrl+Alt+P.")
        })?)
    };

    let manager = app.global_shortcut();
    let _ = manager.unregister_all();

    let Some(shortcut) = parsed else {
        return Ok(());
    };

    manager
        .on_shortcut(shortcut, |app, _shortcut, event| {
            // Key-down only: without this the toggle fires twice per press and
            // the window flashes open and shut.
            if event.state() == ShortcutState::Pressed {
                window::toggle(app);
            }
        })
        .map_err(|err| {
            format!(
                "could not register \"{trimmed}\": {err}. Another application may already own it."
            )
        })
}
