//! The tray icon and its menu.
//!
//! Built in Rust rather than declared in `tauri.conf.json` so it can carry
//! click handlers, and so the menu is not at the mercy of config-schema drift
//! between Tauri minor versions.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;

use crate::window;

const ID_SHOW: &str = "show";
const ID_REFRESH: &str = "refresh";
const ID_QUIT: &str = "quit";

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, ID_SHOW, "Open Portify", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, ID_REFRESH, "Refresh now", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, ID_QUIT, "Quit Portify", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &refresh, &separator, &quit])?;

    let mut builder = TrayIconBuilder::with_id("portify")
        .tooltip("Portify — ports in use")
        .menu(&menu)
        // Left click belongs to "open the window"; the menu is the right click.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            ID_SHOW => window::show(app),
            ID_REFRESH => {
                window::show(app);
                window::request_refresh(app);
            }
            ID_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                window::toggle(tray.app_handle());
            }
        });

    builder = match tray_icon(app) {
        Some(icon) => builder
            .icon(icon)
            .icon_as_template(cfg!(target_os = "macos")),
        None => builder,
    };

    builder.build(app)?;
    Ok(())
}

/// The image to put in the tray.
///
/// macOS expects a monochrome template image that the system tints for light
/// and dark menu bars; Windows and Linux show the full-colour app icon.
///
/// Two whole functions rather than one with `cfg` blocks inside: a `cfg` block
/// that has to `return` because a sibling block might follow it is exactly the
/// shape clippy rejects, and only the platform being compiled would ever see the
/// complaint.
#[cfg(target_os = "macos")]
fn tray_icon(_app: &AppHandle) -> Option<tauri::image::Image<'static>> {
    tauri::image::Image::from_bytes(include_bytes!("../icons/tray-mono.png")).ok()
}

#[cfg(not(target_os = "macos"))]
fn tray_icon(app: &AppHandle) -> Option<tauri::image::Image<'static>> {
    // The bundled icon is borrowed from the app handle; `to_owned` lifts it to
    // 'static so the tray can outlive this call.
    app.default_window_icon()
        .map(|icon| icon.clone().to_owned())
}
