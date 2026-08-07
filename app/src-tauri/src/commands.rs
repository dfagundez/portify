//! The IPC surface exposed to the UI.
//!
//! Every command is a thin adapter over `portify-core`: no business logic lives
//! here, so the CLI and the app can never disagree about what a port is or what
//! killing one means.

use std::time::Instant;

use portify_core::{
    group_by_port, is_elevated, kill_pid as core_kill_pid, kill_port as core_kill_port,
    system_summary, KillMode, KillOutcome, PortGroup, Protocol, ScanOptions, SystemSummary,
};
use serde::Serialize;
use tauri::AppHandle;

use crate::settings::{self, Settings};

#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub ports: Vec<PortGroup>,
    pub elapsed_ms: u128,
    pub elevated: bool,
}

/// Parse the protocol string the UI sends. An unrecognised value means
/// "any protocol" rather than an error: the worst case is a wider match.
fn parse_protocol(value: Option<String>) -> Option<Protocol> {
    match value.as_deref() {
        Some("tcp") => Some(Protocol::Tcp),
        Some("udp") => Some(Protocol::Udp),
        _ => None,
    }
}

#[tauri::command]
pub fn list_ports(include_all: bool) -> Result<ScanResult, String> {
    let options = if include_all {
        ScanOptions::everything()
    } else {
        ScanOptions::default()
    };

    let started = Instant::now();
    let entries = portify_core::scan(&options).map_err(|err| err.to_string())?;
    let ports = group_by_port(&entries);

    Ok(ScanResult {
        ports,
        elapsed_ms: started.elapsed().as_millis(),
        elevated: is_elevated(),
    })
}

#[tauri::command]
pub fn kill_port(port: u16, protocol: Option<String>, force: bool) -> Vec<KillOutcome> {
    core_kill_port(port, parse_protocol(protocol), kill_mode(force))
}

#[tauri::command]
pub fn kill_pid(pid: u32, force: bool) -> KillOutcome {
    core_kill_pid(pid, kill_mode(force))
}

fn kill_mode(force: bool) -> KillMode {
    if force {
        KillMode::Force
    } else {
        KillMode::Graceful
    }
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    settings::load(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    // Register first: an accelerator that cannot be bound is never written to
    // disk, so a typo cannot leave the app starting with a dead hotkey.
    crate::shortcut::apply(&app, &settings.shortcut)?;
    settings::save(&app, &settings)
}

#[tauri::command]
pub fn get_system_info() -> SystemSummary {
    system_summary()
}

/// Called by the UI once it has painted its first frame.
///
/// The window is created hidden so nobody ever sees the webview's blank default
/// background; this is what puts it on screen.
#[tauri::command]
pub fn ready(app: AppHandle) {
    crate::window::show(&app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_strings_map_to_the_core_enum() {
        assert_eq!(parse_protocol(Some("tcp".into())), Some(Protocol::Tcp));
        assert_eq!(parse_protocol(Some("udp".into())), Some(Protocol::Udp));
    }

    #[test]
    fn an_unknown_protocol_widens_rather_than_fails() {
        assert_eq!(parse_protocol(None), None);
        assert_eq!(parse_protocol(Some("sctp".into())), None);
        assert_eq!(parse_protocol(Some("TCP".into())), None);
    }

    #[test]
    fn force_selects_the_hard_kill() {
        assert_eq!(kill_mode(true), KillMode::Force);
        assert_eq!(kill_mode(false), KillMode::Graceful);
    }

    #[test]
    fn listing_ports_returns_a_usable_result() {
        let result = list_ports(false).expect("scan should succeed");
        assert!(result.elapsed_ms < 10_000, "a scan must not take seconds");
        for group in &result.ports {
            assert!(group.sockets > 0 || group.connections > 0);
        }
    }
}
