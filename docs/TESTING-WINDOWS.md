# Testing Portify on Windows

Windows is Portify's primary platform, and this is the checklist for verifying a build there by hand.

## Do not test from WSL

This matters more than it sounds. Inside WSL:

- The socket table Portify reads is **WSL's Linux socket table**, not Windows'. Your Windows dev servers will not appear, and the ports you do see belong to the Linux VM.
- A tray icon created inside WSL does not appear in the Windows notification area.
- The Windows-specific code paths — elevation detection, `TerminateProcess`, console VT handling — are never exercised.

Run everything below in **PowerShell on Windows**, not in a WSL shell.

## Prerequisites

```powershell
winget install Rustlang.Rustup
winget install OpenJS.NodeJS.LTS
```

### The MSVC linker

Rust's default Windows target (`x86_64-pc-windows-msvc`) links with Microsoft's
`link.exe`, so the C++ build tools are required even though Portify is pure
Rust. Without them the build dies on the very first crate:

```
error: linker `link.exe` not found
```

Installing `Microsoft.VisualStudio.2022.BuildTools` on its own is **not enough**
— that package installs the Visual Studio installer shell with no workloads, so
no compiler and no linker. The C++ workload has to be requested explicitly:

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools --force `
  --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

That pulls the MSVC toolchain and the Windows SDK — roughly 2–3 GB, several
minutes. If Build Tools is already present and winget refuses, do the same thing
from the GUI: **Visual Studio Installer → Build Tools 2022 → Modify → Desktop
development with C++ → Modify**.

Then open a **new** PowerShell window and confirm:

```powershell
cargo --version
rustc --print target-libdir      # should print an ...msvc... path
```

You do not need the "Developer PowerShell": cargo locates the toolchain through
the registry, so an ordinary shell works once the workload is installed.

WebView2 is already present on Windows 10 and 11. Restart the terminal after
installing so `cargo` and `npm` are on `PATH`.

## Build

From a checkout of the repository on a Windows drive. If you are working out of
WSL, copy the source across rather than building over `\\wsl$\…` — cargo and npm
on a 9p network path are slow and occasionally flaky:

```powershell
# From WSL, once:
#   rsync -a --exclude target --exclude node_modules --exclude .git \
#       ~/personal/portify/ /mnt/c/Users/$USER/dev/portify/

cd C:\Users\$env:USERNAME\dev\portify

# CLI
cargo build --release -p portify-cli
.\target\release\portify.exe --version

# Desktop app
cd app
npm install
npm run tauri build
```

Installers land in `target\release\bundle\` (`msi\` and `nsis\`). For an iteration loop, use `npm run tauri dev` instead.

## CLI checklist

Start a throwaway listener in a second PowerShell window so nothing real is at risk:

```powershell
# Window 2 — holds port 39999 until you close it
$listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, 39999)
$listener.Start()
Write-Host "listening on 39999 — Ctrl+C to stop"
while ($true) { Start-Sleep 1 }
```

Then, in window 1:

| # | Command | Expected |
|---|---|---|
| 1 | `portify` | Table of ports. Real Windows process names (`node.exe`, `svchost.exe`), no `MainThread`, no duplicate rows per port |
| 2 | `portify 39999` | One row, `python`/`powershell`, correct PID |
| 3 | `portify 39998` | "No ports in use (port 39998) found." |
| 4 | `echo $LASTEXITCODE` | `2` |
| 5 | `portify list --filter powershell --proto tcp` | Both filters applied, not just one |
| 6 | `portify list --json \| ConvertFrom-Json` | Parses cleanly |
| 7 | `portify info` | Correct hostname and user; `privileges` says *standard user* |
| 8 | `portify kill 39999` | Shows the plan, prompts `Continue? [y/N]` |
| 9 | Answer `n` | "Cancelled." and the listener survives |
| 10 | `portify kill 39999` → `y` | `✔ port 39999 freed: …` and window 2 dies |
| 11 | `portify kill 39999` | "nothing is holding port 39999", exit code 2 |
| 12 | `portify watch` | Redraws on an interval; Ctrl+C exits cleanly |
| 13 | `portify completions powershell` | Emits a completion script |

Things to look at rather than assert:

- **Colour and alignment.** `anstream` should give colour in Windows Terminal and degrade cleanly in the legacy console host (`conhost`). Try both if you can.
- **Piping.** `portify | Select-Object -First 5` must contain no escape-code garbage.
- **Narrow window.** Resize to ~60 columns; the table should truncate, not wrap.

### Elevated pass

Open PowerShell as Administrator and repeat 1 and 7:

- `portify info` should now say **elevated**.
- Rows that read `(hidden)` unelevated should now show real names and PIDs, and the "owned by another user" hint should be gone.

## Desktop app checklist

| Area | What to check |
|---|---|
| Tray | Icon appears in the notification area (may be under the "^" overflow — drag it out) |
| Left-click | Shows and hides the window |
| `Ctrl+Alt+P` | Shows and hides the window from any application |
| Custom shortcut | Settings → type e.g. `Ctrl+Alt+K`, Enter; the old one stops working, the new one works |
| Bad shortcut | Type `nonsense`; the field turns red, the message explains, and the previous shortcut keeps working |
| Taskbar | Portify has **no** taskbar button, visible or not — only the tray icon |
| Right-click | Menu with Open / Refresh now / Quit |
| Window chrome | The window has no OS title bar; dragging the header moves it |
| List | Same data as the CLI; refreshes on the interval |
| Search | Typing `3000`, `node` or a PID filters live |
| Kill | Hovering a row reveals **Kill**; first click says **Confirm?**; second click frees the port |
| Shift-click | Force-kills immediately, skipping the confirmation step |
| Notification | A Windows toast appears with the result |
| Settings | Changes persist across a restart (`%APPDATA%\com.dfagundez.portify\settings.json`) |
| Close button | Hides the window; app stays in the tray |
| Quit | Only the tray menu quits it |
| Second launch | Running the exe again focuses the existing window instead of adding a second tray icon |
| Theme | Switch Windows to dark mode; the app follows |

## Known rough edges

- Installers are **unsigned**. SmartScreen will warn: *More info → Run anyway*. Signing needs a certificate and is not set up.
- Elevated processes stay hidden unless Portify itself is elevated. There is no UAC prompt to escalate on demand yet.
- Killing a WSL-forwarded port from Windows kills `wslrelay.exe`, the bridge — not the process inside WSL. One relay usually fronts every forwarded port, so Portify warns before you take them all down at once. To manage the real processes, run Portify inside WSL.

## Reporting what you find

Useful bug reports include `portify info` output, the exact command, what you expected, and what happened. For the app, `npm run tauri dev` prints Rust panics and JS console errors to the terminal.
