//! Portify desktop app.

mod commands;
mod settings;
mod shortcut;
mod tray;
mod window;

use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be registered first so a second launch is routed to the running
        // instance instead of adding a second tray icon.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show(app);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            tray::build(app.handle())?;

            // A hotkey the OS refuses (already taken by something else) must not
            // stop the app from starting; the window is still reachable from the
            // tray, and the settings panel will show the failure.
            let saved = settings::load(app.handle());
            if let Err(err) = shortcut::apply(app.handle(), &saved.shortcut) {
                eprintln!("portify: {err}");
            }

            // Belt and braces for the first paint. The window is created hidden
            // and revealed by the `ready` command, but the background colour
            // still matters for the instant between the two, and for any resize
            // the compositor has to fill.
            if let Some(window) = app.get_webview_window(window::MAIN) {
                let dark = matches!(window.theme(), Ok(tauri::Theme::Dark));
                let base = if dark {
                    tauri::window::Color(0x16, 0x19, 0x1d, 0xff)
                } else {
                    tauri::window::Color(0xff, 0xff, 0xff, 0xff)
                };
                let _ = window.set_background_color(Some(base));
            }

            // If the UI never reports in — a broken bundle, a webview that fails
            // to start — show the window anyway. An ugly window beats a process
            // with no visible sign of life.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(3));
                if let Some(window) = handle.get_webview_window(window::MAIN) {
                    if !window.is_visible().unwrap_or(false) {
                        eprintln!("portify: UI did not report ready; showing anyway");
                        window::show(&handle);
                    }
                }
            });

            // A tray utility has no business in the macOS Dock or app switcher.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            Ok(())
        })
        .on_window_event(|window, event| match event {
            // Closing the window means "get out of my way", not "quit": the tray
            // icon is the app's real home.
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                window::hide(window.app_handle());
            }
            // Remembering when focus was lost is what lets a tray click tell
            // "the window is buried, raise it" apart from "the window is right
            // here, put it away" — the click itself removes focus either way.
            WindowEvent::Focused(false) => window::note_blur(),
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_ports,
            commands::kill_port,
            commands::kill_pid,
            commands::get_settings,
            commands::save_settings,
            commands::get_system_info,
            commands::ready,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Portify");
}
