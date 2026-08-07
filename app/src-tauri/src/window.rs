//! Window visibility.
//!
//! The tray owns whether the window is on screen, and the UI needs to know so
//! it can stop polling while nobody is looking. Every transition therefore goes
//! through here and emits an event the frontend listens for.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

pub const MAIN: &str = "main";

/// A blur this recent was almost certainly caused by the click being handled.
///
/// Clicking the tray icon takes focus away from the window as a side effect, so
/// by the time the handler runs the window always looks unfocused. Without this
/// grace period the tray click can never mean "hide".
const BLUR_GRACE: Duration = Duration::from_millis(400);

/// When the window last lost focus.
///
/// A module-level cell rather than Tauri state: there is exactly one window, and
/// threading a managed struct through the tray callbacks buys nothing.
fn last_blur() -> &'static Mutex<Option<Instant>> {
    static CELL: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(None))
}

/// Record that the window just lost focus. Called from the window event hook.
pub fn note_blur() {
    if let Ok(mut slot) = last_blur().lock() {
        *slot = Some(Instant::now());
    }
}

/// True when the window has focus, or lost it so recently that the loss is
/// attributable to the interaction currently being handled.
fn effectively_focused(window: &WebviewWindow) -> bool {
    if window.is_focused().unwrap_or(false) {
        return true;
    }
    last_blur()
        .lock()
        .ok()
        .and_then(|slot| *slot)
        .is_some_and(|at| at.elapsed() < BLUR_GRACE)
}

/// Event names shared with `src/main.ts`.
pub const EVENT_SHOWN: &str = "portify://shown";
pub const EVENT_HIDDEN: &str = "portify://hidden";
pub const EVENT_REFRESH: &str = "portify://refresh";

fn main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(MAIN)
}

pub fn show(app: &AppHandle) {
    let Some(window) = main_window(app) else {
        return;
    };
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
    let _ = app.emit(EVENT_SHOWN, ());
}

pub fn hide(app: &AppHandle) {
    let Some(window) = main_window(app) else {
        return;
    };
    let _ = window.hide();
    let _ = app.emit(EVENT_HIDDEN, ());
}

/// Show, raise, or hide — whichever the user most likely meant.
///
/// Visibility alone is the wrong question. After alt-tabbing away the window is
/// still *visible*, just buried behind whatever has focus, so a visibility-only
/// toggle hides something the user cannot see and the hotkey appears to need
/// two presses. Hiding is only right when the window is both visible and
/// focused; otherwise the intent is "bring it to me".
pub fn toggle(app: &AppHandle) {
    let Some(window) = main_window(app) else {
        return;
    };

    // `is_visible` can fail while the window is being torn down. Defaulting to
    // false means the fallback is to show it, which is the recoverable outcome.
    let visible = window.is_visible().unwrap_or(false);

    if visible && effectively_focused(&window) {
        hide(app);
    } else {
        show(app);
    }
}

/// Ask the UI to rescan now.
pub fn request_refresh(app: &AppHandle) {
    let _ = app.emit(EVENT_REFRESH, ());
}
