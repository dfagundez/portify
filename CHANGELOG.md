# Changelog

All notable changes to Portify. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows [Semantic Versioning](https://semver.org/).

## [0.1.0] — unreleased

Ground-up rewrite in Rust. The Python version (tagged `v1.0.0`) is gone; nothing carries over but the name and the icon.

The version number went *down* on purpose. The old `1.0.0` was marked
"Production/Stable" while its README documented a Settings menu, batch kills and
change notifications that did not exist in the code. 0.1.0 is what this actually
is: a first release of a new codebase.

### Added

- **`portify kill 3000`** — free a port directly. Previously you had to list ports, read a PID off the screen, and kill that. This is the reason the project exists and it was the one thing missing.
- **One row per port**, collapsing the many sockets a single server opens across address families and interfaces.
- **`portify watch`** — live view on an interval.
- **`--json`** on every command, with a stable shape, for scripting.
- **Meaningful exit codes** — 0 success, 2 not found, 3 permission denied, 4 bad input, 5 internal.
- **`portify completions <shell>`** for bash, zsh, fish, PowerShell and Elvish.
- **Ephemeral UDP sockets hidden by default.** Outbound client sockets on OS-assigned high ports were 60 of 149 rows on a real Windows desktop. `--all` shows them, and asking for one by number still finds it.
- **Elevation awareness** — sockets whose owner the OS hides are labelled as such, with the exact command to see them. On Windows this reads the process token's elevation flag rather than guessing from the username.
- **Desktop app** (Tauri v2): tray icon, filterable list, one-click kill with confirm, light/dark, persisted settings.
- **Kill safety rules** — refuses to kill itself, PID 0, init/System.
- **Collateral warning.** Before killing, Portify says what *else* the target process is holding. This is not hypothetical: on a Windows machine with WSL, one `wslrelay.exe` fronts every forwarded port — 57 of them on the dev box this was found on — so "free port 6379" would silently drop every WSL service at once. The CLI shows it in the confirmation plan and in `--yes` output; the app shows it when a row is armed.
- **Kill verification** — a graceful kill escalates to SIGKILL when the process ignores SIGTERM, and success is confirmed by re-reading the process table rather than assumed from the signal returning.

### Changed

- Rust replaces Python: one binary of about 950 KB with no runtime to install, versus a Python package plus platform extras.
- Windows is now the primary platform. The project is no longer described as macOS-first.
- Process names come from the executable path rather than the OS comm field, which on Linux is a 15-character thread name — the source of `MainThread` appearing where `python3` belonged.
- Killing by a bare number now means **port**, not PID. PIDs require `--pid`.
- Memory reads as unknown rather than `0 B` when the OS refuses to open a process for query, which on Windows is most of `svchost.exe`.
- Notifications fire on kill results only, which is all they ever did; the previous claim of alerts when processes start or stop was never implemented.

### Fixed

Carried over from the Python version, all verified fixed by tests in the rewrite:

- Filters no longer override each other. `--filter node --port 3000` previously ignored `--filter` entirely, because each filter re-derived its own subset from the full scan.
- The 500 ms decorative spinner is gone. It ran *before* a scan that takes about 7 ms, so it was 98% of the perceived runtime.
- `--no-auto-refresh` is respected. The old menu-bar update loop never read the flag and rebuilt its menu every 5 seconds regardless.
- Toggling auto-refresh off and on again works. The old toggle set a shared `is_running` flag to false, stopping the whole manager, and could never restart because it checked a thread handle that was never cleared.
- "Open CLI" is gone rather than silently macOS-only; it ran `osascript` and did nothing on Windows or Linux.
- Notifications work on Windows. The old detection had branches for macOS and Linux only, so Windows fell through to printing on a console nobody could see.
- `info` reports the right user and elevation state on Windows, where `$USER` is unset and `os.geteuid` does not exist.
- Dependencies are current. The Python pins were from 2023 and would not install on Python 3.13.

### Removed

- The entire Python implementation, its three overlapping install scripts, and the two competing dependency lists that let the menu-bar extras drift apart.
- `updater.py` and `version.json` — dead code pointing at `https://tu-sitio-web.com`.
- `quick-install.sh`, which cloned a repository that does not exist and appended to `~/.zshrc` without asking.
- The placeholder hero image, pointing at a discontinued service.
