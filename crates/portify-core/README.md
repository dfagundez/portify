# portify-core

The engine behind [Portify](https://github.com/dfagundez/portify): find which
process holds a port, and terminate it without terminating something else by
accident.

Used by the [`portify`](https://crates.io/crates/portify-cli) CLI and the Portify
desktop app, which is the point — two front ends over one engine can never
disagree about what a port is or what killing one means.

```toml
[dependencies]
portify-core = "0.1"
```

```rust
use portify_core::{group_by_port, scan, ScanOptions};

let entries = scan(&ScanOptions::default())?;
for group in group_by_port(&entries) {
    println!("{} {} {}", group.port, group.protocol, group.process_label());
}
# Ok::<(), portify_core::error::Error>(())
```

Sockets come from [`netstat2`](https://crates.io/crates/netstat2), process
identity from [`sysinfo`](https://crates.io/crates/sysinfo). Windows, macOS and
Linux.

## What it gives you that the raw socket table does not

**One row per port.** A dev server binds IPv4 and IPv6 across several
interfaces; the OS reports that as many sockets. `group_by_port` collapses them
into the single thing a human means by "port 3000".

**Ephemeral UDP filtered out.** Outbound client sockets — a browser on QUIC, a
VPN agent phoning home — sit on OS-assigned high ports and outnumber real
listeners about two to one on a Windows desktop. Hidden unless you ask, and
asking for a specific port always finds it.

**Honest process names.** Read from the executable path rather than the OS comm
field, which on Linux is a 15-character thread name and reports `MainThread`
where `python3` belongs.

**Honest absences.** Memory reads as `None`, not `0`, when the OS refuses to open
a process for query — which on Windows is most of `svchost.exe`. Sockets whose
owner is hidden say so instead of reporting a blank.

**Collateral awareness.** `other_ports_held_by` answers the question that makes
killing safe: what *else* does this process hold? One `wslrelay.exe` can front
every forwarded WSL port at once, so "free port 6379" is otherwise a way to drop
every WSL service simultaneously without being told.

## Killing

```rust,no_run
use portify_core::{kill_port, KillMode};

// A Vec, because a port can be held over both TCP and UDP, and because
// `None` here means "either protocol".
for outcome in kill_port(3000, None, KillMode::Graceful) {
    if let Some(warning) = outcome.collateral_warning() {
        eprintln!("{warning}");
    }
    println!("{:?}", outcome.status);
}
```

`KillMode::Graceful` sends SIGTERM and escalates to SIGKILL only if the process
ignores it, because "the kill succeeded but the port is still taken" is the one
outcome this must never produce. Success is confirmed by re-reading the process
table rather than inferred from the signal returning. On Windows, where there is
no SIGTERM, both modes are a single `TerminateProcess`.

It refuses to kill the calling process, PID 0, PID 1 (init) on Unix and PID 4
(System) on Windows.

## Design

No UI, no async, no background threads. A full scan is milliseconds, so callers
just call it on a timer — which is what both front ends do.

## License

MIT
