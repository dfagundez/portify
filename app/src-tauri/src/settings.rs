//! User settings, persisted as JSON in the OS config directory.
//!
//! Deliberately not a plugin: it is one small struct, and reading a file on
//! demand is simpler to reason about than a key-value store with its own
//! lifecycle.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Seconds between automatic refreshes. Zero disables auto-refresh.
    pub refresh_interval_secs: u32,
    /// Show a desktop notification with the result of a kill.
    pub notifications: bool,
    /// Require a second click before terminating anything.
    pub confirm_before_kill: bool,
    /// Include established connections, not just ports being served.
    pub include_all: bool,
    /// Hide the window as soon as it loses focus (menu-bar style).
    pub hide_on_blur: bool,
    /// System-wide hotkey that shows or hides the window, in Tauri's
    /// accelerator syntax. Empty disables it.
    pub shortcut: String,
}

/// Default hotkey.
///
/// Ctrl+Alt+P rather than the more obvious Ctrl+Shift+P: a global shortcut wins
/// over whatever application has focus, and Ctrl+Shift+P would silently break
/// the command palette in every editor on the machine.
pub const DEFAULT_SHORTCUT: &str = "CmdOrCtrl+Alt+P";

impl Default for Settings {
    fn default() -> Self {
        Self {
            refresh_interval_secs: 5,
            notifications: true,
            // Defaults to on: the whole app is one click away from killing a
            // process, so the safe default is the deliberate one.
            confirm_before_kill: true,
            include_all: false,
            // Off by default because a window that vanishes when you alt-tab is
            // surprising the first time you meet it.
            hide_on_blur: false,
            shortcut: DEFAULT_SHORTCUT.to_string(),
        }
    }
}

impl Settings {
    /// Clamp anything a hand-edited file could get wrong.
    fn sanitised(mut self) -> Self {
        if self.refresh_interval_secs > 3600 {
            self.refresh_interval_secs = 3600;
        }
        self
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|err| format!("no config directory available: {err}"))?;
    Ok(dir.join("settings.json"))
}

/// Read settings, falling back to defaults for a missing or unreadable file.
///
/// A corrupt settings file must never stop the app from starting — the worst
/// acceptable outcome is losing preferences.
pub fn load(app: &AppHandle) -> Settings {
    let Ok(path) = settings_path(app) else {
        return Settings::default();
    };
    let Ok(contents) = fs::read_to_string(&path) else {
        return Settings::default();
    };
    serde_json::from_str::<Settings>(&contents)
        .map(Settings::sanitised)
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("could not create {}: {err}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&settings.clone().sanitised())
        .map_err(|err| format!("could not serialise settings: {err}"))?;
    fs::write(&path, json).map_err(|err| format!("could not write {}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_conservative() {
        let settings = Settings::default();
        assert!(settings.confirm_before_kill, "killing must be deliberate");
        assert!(!settings.hide_on_blur);
        assert_eq!(settings.refresh_interval_secs, 5);
    }

    #[test]
    fn the_default_hotkey_avoids_the_editor_command_palette() {
        // A global shortcut outranks the focused application, so colliding with
        // Ctrl+Shift+P would break VS Code everywhere on the machine.
        assert_eq!(Settings::default().shortcut, DEFAULT_SHORTCUT);
        assert!(!DEFAULT_SHORTCUT.contains("Shift"));
    }

    #[test]
    fn an_older_settings_file_gains_the_default_hotkey() {
        let settings: Settings = serde_json::from_str(r#"{ "notifications": true }"#).unwrap();
        assert_eq!(settings.shortcut, DEFAULT_SHORTCUT);
    }

    #[test]
    fn unknown_and_missing_fields_fall_back_to_defaults() {
        // Forward compatibility: a settings file written by a newer build must
        // not brick an older one.
        let json = r#"{ "notifications": false, "something_new": 42 }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert!(!settings.notifications);
        assert_eq!(settings.refresh_interval_secs, 5);
        assert!(settings.confirm_before_kill);
    }

    #[test]
    fn absurd_intervals_are_clamped() {
        let settings = Settings {
            refresh_interval_secs: 999_999,
            ..Settings::default()
        }
        .sanitised();
        assert_eq!(settings.refresh_interval_secs, 3600);
    }

    #[test]
    fn round_trips_through_json() {
        let original = Settings {
            refresh_interval_secs: 10,
            notifications: false,
            confirm_before_kill: false,
            include_all: true,
            hide_on_blur: true,
            shortcut: "CmdOrCtrl+Alt+K".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }
}
