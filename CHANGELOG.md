# Changelog

All notable changes to Portify. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows [Semantic Versioning](https://semver.org/).

## Unreleased

### Added

- **Scoop.** This repository is the bucket, so `scoop bucket add portify <repo>`
  then `scoop install portify` is the whole flow — no second repository, nothing
  to submit, nobody to wait for.
- **winget manifests**, validated against Microsoft's 1.6.0 schemas and
  [submitted](https://github.com/microsoft/winget-pkgs/pull/413867).
- **crates.io**: `cargo install portify-cli` installs a binary named `portify`,
  and [`portify-core`](https://crates.io/crates/portify-core) is available as a
  library. The `-cli` suffix is not a preference — `portify` there belongs to an
  unrelated HTTPS/SVCB library.
- Manifests are generated and verified by script rather than by hand:
  `update-packaging.mjs` rewrites them from a published release,
  `check-packaging.mjs` re-downloads what they point at and fails on a checksum
  mismatch or a version bumped without its URL following. CI runs the check on
  every push; publishing a release runs the update and commits it.
- `Display` for `Protocol` and `SocketState`, so `{protocol}` and `.to_string()`
  work without reaching for `as_str`. Published-crate ergonomics.
- Per-crate READMEs, and the `portify-core` examples now compile as doctests —
  the first version of them named the wrong argument order for `kill_port` and
  a type that does not implement `Display`, both caught by wiring them in.

### Changed

- `portify-core` and `portify-cli` carry the metadata crates.io wants (readme,
  keywords, categories, homepage). `portify-app` is marked `publish = false`:
  `generate_context!` embeds `app/dist` at compile time, so a published copy
  would not build for anyone.

### Fixed

- **A tag push can no longer overwrite a published release.** Force-pushing
  `v0.1.0` during a history rewrite re-ran the release workflow, which rebuilt
  every binary and replaced all ten assets — same source, different bytes,
  different checksums. Scoop healed itself on the next automated sync; the
  pending winget submission could not, and had to be corrected by hand after it
  had already passed Microsoft's validation. `release.yml` now refuses to build
  a tag whose release is already published.
- **Usage errors now exit 4, not 2.** Argument parsing errors used clap's
  default exit code, which is the same 2 this CLI uses for "nothing found", so
  `portify 3000 || echo free` could not tell a free port from a misspelled
  flag. `--help` and `--version` still exit 0. Locked in by CI, which now
  asserts every documented exit code rather than just the free-port case.
- Documentation: the README advertised a `--port` flag that does not exist
  (ports are bare arguments), told Linux and macOS users to install a Rust
  toolchain when the release already ships prebuilt binaries for them, gave a
  zsh completion snippet that silently does nothing under oh-my-zsh — it
  appends to `.zshrc` after the framework has already run `compinit` — and
  quoted a binary size and frontend size that no longer matched the build.

## [0.1.0] — 2026-08-07

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
