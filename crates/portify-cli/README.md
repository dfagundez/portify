# portify-cli

**Find out what is holding a port, and take it back.**

```console
$ portify kill 3000
✔ port 3000 freed: node (PID 12841) terminated
```

Instead of `netstat -ano | findstr 3000`, reading a PID off the screen, and
`taskkill /PID 12841 /F`.

```bash
cargo install portify-cli     # installs a binary named `portify`
```

Prebuilt binaries for Windows, macOS and Linux are attached to
[each release](https://github.com/dfagundez/portify/releases/latest), and on
Windows there is a Scoop bucket. See the
[project README](https://github.com/dfagundez/portify#readme) for both.

## What it does

```console
$ portify
 PORT  PROTO  PROCESS       PID    MEMORY  SERVICE
 3000  TCP    node       753572  236.7 MB  Dev server (Node/Rails/Grafana)
 5432  TCP    postgres     4410   28.1 MB  PostgreSQL
 6379  TCP    redis-server 4711   12.3 MB  Redis

 3 ports  ·  7 ms
```

One row per port, not one per socket: a dev server bound to IPv4 and IPv6 across
three interfaces is one line, the way you think about it.

| Command | What it does |
|---|---|
| `portify` | List every port in use |
| `portify 3000` | Show what is holding port 3000 |
| `portify kill 3000` | Free port 3000 |
| `portify kill 3000 8080 -y` | Free several, no prompt |
| `portify kill --pid 12841` | Kill a specific process |
| `portify watch` | Live view on an interval |
| `portify info` | Host details and whether you are elevated |
| `portify completions <shell>` | bash, zsh, fish, PowerShell, Elvish |

Bare numbers are **ports**, never PIDs — killing by PID is always explicit via
`--pid`, so a typo cannot take down an unrelated process.

## Killing is deliberate

Before terminating anything, Portify names the other ports the same process is
holding. One process fronting several ports is common — a WSL relay, a reverse
proxy, a dev server with a debugger attached — so "free port 7003" can quietly
mean "drop three of them".

A graceful kill sends SIGTERM, waits, and escalates to SIGKILL only if the
process ignores it, because "the kill succeeded but the port is still taken" is
the one outcome this tool must never produce. Success is confirmed by re-reading
the process table, not assumed because the signal returned.

It refuses to kill itself, PID 0, PID 1 (init) on Unix and PID 4 (System) on
Windows.

## Scripting

`--json` on every command, with a stable shape, plus meaningful exit codes:

| Code | Meaning |
|---|---|
| 0 | Success |
| 2 | Nothing found |
| 3 | Permission denied |
| 4 | Bad input |
| 5 | Internal error |

```bash
portify 3000 >/dev/null || echo "port 3000 is free"
```

## Permissions

Portify sees and kills only what your user account can. Ports owned by services,
other users or elevated processes appear as `(hidden)` with no PID; `portify
info` tells you which side of that line you are on. It never asks for elevation
on your behalf and never escalates silently.

The engine lives in [`portify-core`](https://crates.io/crates/portify-core), and
[the desktop app](https://github.com/dfagundez/portify) is built on the same
crate — so the two can never disagree about what a port is or what killing one
means.

## License

MIT
