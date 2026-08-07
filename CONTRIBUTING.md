# Contributing to Portify

## Setup

Rust 1.95+ ([rustup](https://rustup.rs)) and Node 20+ for the desktop app.

```bash
git clone https://github.com/dfagundez/portify.git
cd portify
cargo test --workspace --exclude portify-app   # core + CLI, no desktop deps needed
```

Working on the desktop app also needs the platform WebView toolchain:

- **Windows** — MSVC build tools; WebView2 ships with Windows 10/11.
- **macOS** — `xcode-select --install`.
- **Linux** — `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf`.

```bash
cd app
npm install
npm run tauri dev
```

> The frontend must be built before any cargo command that touches `portify-app`:
> `tauri::generate_context!` embeds `app/dist` at compile time and fails if it is
> missing. `npm run tauri dev|build` handles this; a bare `cargo check -p
> portify-app` does not.

## Before opening a PR

```bash
cargo fmt --all
cargo clippy --workspace --all-targets   # CI treats warnings as errors
cargo test --workspace
```

CI runs this on Windows, macOS and Linux.

## Where code goes

| Layer | Rule |
|---|---|
| `crates/portify-core` | All behaviour. If the CLI and the app could ever disagree about it, it belongs here. |
| `crates/portify-cli` | Argument parsing and terminal rendering only. |
| `app/src-tauri` | Thin IPC adapters over the core, plus window and tray plumbing. |
| `app/src` | Presentation. No logic that decides what a port *is*. |

Anything platform-specific goes behind `cfg` inside the core, never duplicated in a front end.

## Testing

Prefer tests that need no live system state — that is why grouping, filtering,
rendering, exit-code mapping and settings parsing are all pure functions with
their own tests.

Tests that do touch the machine must stay safe on any developer's box and on CI:

- Never kill a process the test did not create.
- Use port 1 (never bound, needs root anyway) for "not found" paths.
- Assert on *shape*, not on the specific ports a machine happens to have open.

Kill-path tests cover the refusal rules (self, PID 0, init) rather than real
kills, for obvious reasons.

## Cross-checking Windows from another OS

Windows is the primary target, so verify it even when you are not on it:

```bash
rustup target add x86_64-pc-windows-msvc
cargo check --target x86_64-pc-windows-msvc -p portify-core -p portify-cli
```

This type-checks the Windows code paths without a Windows machine. It does not
cover `portify-app`, whose build script needs a Windows resource compiler
(`llvm-rc`); CI covers that on a real Windows runner.

## Icons

`assets/icon.png` and `assets/tray-mono.png` are generated, not drawn:

```bash
node scripts/generate-icon.mjs      # regenerate the sources
cd app && npx tauri icon ../../assets/icon.png   # regenerate platform icons
```

Edit the geometry constants in `scripts/generate-icon.mjs` rather than the PNGs.

## Style

- Comments explain *why*, not *what*. If a line needs a comment to say what it
  does, rename something instead.
- Errors the user can act on get a message that says what to do next
  ("run as Administrator"), not just what failed.
- A failed kill is a result, not an exception.
