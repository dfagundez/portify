//! Terminating whatever is holding a port.
//!
//! Killing is the destructive half of the tool, so the rules are explicit:
//!
//! * A failed kill is a reported outcome, never a panic or an `Err`.
//! * Portify refuses to kill itself, PID 0, and the OS init/System process.
//! * Graceful first (SIGTERM), escalating to SIGKILL only when asked or when
//!   the process ignores the polite request.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System, UpdateKind};

use crate::scan::{process_name, scan, Protocol, ScanOptions};

/// How hard to try.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KillMode {
    /// SIGTERM, escalating to SIGKILL if the process is still alive after the
    /// grace period. On Windows there is no SIGTERM, so this is a single
    /// `TerminateProcess`.
    #[default]
    Graceful,
    /// Straight to SIGKILL / `TerminateProcess`.
    Force,
}

/// What happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KillStatus {
    /// The process is gone, confirmed by re-reading the process table.
    Killed,
    /// Escalated to SIGKILL because the process ignored SIGTERM.
    Escalated,
    /// No such PID (it may have exited on its own between listing and killing).
    NotFound,
    /// The OS said no. Almost always "another user's process, run elevated".
    PermissionDenied,
    /// The signal was delivered but the process was still alive at timeout.
    Survived,
    /// Refused by Portify's own safety rules.
    Refused,
}

impl KillStatus {
    pub fn is_success(self) -> bool {
        matches!(self, KillStatus::Killed | KillStatus::Escalated)
    }
}

/// Result of one kill attempt.
#[derive(Debug, Clone, Serialize)]
pub struct KillOutcome {
    pub pid: u32,
    pub process_name: String,
    /// Set when the kill was requested by port rather than by PID.
    pub port: Option<u16>,
    pub status: KillStatus,
    /// Human-readable explanation, safe to show verbatim in a UI.
    pub detail: String,
    /// Other ports the same process was holding, which go down with it.
    ///
    /// This is not a detail: a single WSL relay process routinely fronts every
    /// forwarded port on the machine, so "free port 6379" can quietly mean
    /// "drop all 57 forwards". Callers are expected to surface this before
    /// killing, not after.
    pub collateral_ports: Vec<u16>,
}

impl KillOutcome {
    /// One-line note about what else went down with this process, if anything.
    ///
    /// Past tense on purpose: this hangs off an outcome, so by the time anyone
    /// reads it the other ports are already gone. The *pre*-kill warning is the
    /// caller's job, built from the listing it already has.
    pub fn collateral_warning(&self) -> Option<String> {
        let count = self.collateral_ports.len();
        if count == 0 {
            return None;
        }
        let preview: Vec<String> = self
            .collateral_ports
            .iter()
            .take(6)
            .map(|port| port.to_string())
            .collect();
        let listed = preview.join(", ");
        let name = if self.process_name.is_empty() {
            "this process".to_string()
        } else {
            self.process_name.clone()
        };
        Some(if count > preview.len() {
            format!("{name} also held {count} other ports ({listed}, …) — those went down with it")
        } else if count == 1 {
            format!("{name} also held port {listed}, which went down with it")
        } else {
            format!("{name} also held ports {listed} — those went down with it")
        })
    }
}

/// How long to wait for a graceful exit before escalating.
const GRACE_PERIOD: Duration = Duration::from_millis(1500);
/// How long to wait after SIGKILL before declaring the process a survivor.
const FORCE_TIMEOUT: Duration = Duration::from_millis(1000);
/// How often to re-check whether the process is gone.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Kill a single process by PID.
pub fn kill_pid(pid: u32, mode: KillMode) -> KillOutcome {
    kill_pid_inner(pid, mode, None)
}

/// Kill everything holding `port` open.
///
/// This is the operation the tool exists for: it does the list-then-find-the-PID
/// dance internally so the caller never has to read a PID off a screen.
/// `protocol` narrows the search; `None` means any protocol.
pub fn kill_port(port: u16, protocol: Option<Protocol>, mode: KillMode) -> Vec<KillOutcome> {
    let options = ScanOptions {
        with_details: false,
        // Naming a port explicitly means "whatever is on it": an ephemeral UDP
        // socket is hidden from the browse view, but it must still be findable
        // and killable when asked for by number.
        include_ephemeral_udp: true,
        ..ScanOptions::default()
    };

    let entries = match scan(&options) {
        Ok(entries) => entries,
        Err(err) => {
            return vec![KillOutcome {
                pid: 0,
                process_name: String::new(),
                port: Some(port),
                status: KillStatus::NotFound,
                detail: format!("could not read the socket table: {err}"),
                collateral_ports: Vec::new(),
            }]
        }
    };

    let mut targets: Vec<(u32, String)> = Vec::new();
    let mut hidden_owner = false;
    // Every port each PID holds, so the caller can be told what else dies.
    let mut ports_by_pid: BTreeMap<u32, BTreeSet<u16>> = BTreeMap::new();

    for entry in &entries {
        if let Some(pid) = entry.pid {
            ports_by_pid.entry(pid).or_default().insert(entry.port);
        }
    }

    for entry in entries {
        if entry.port != port {
            continue;
        }
        if let Some(wanted) = protocol {
            if entry.protocol != wanted {
                continue;
            }
        }
        match entry.pid {
            Some(pid) if !targets.iter().any(|(existing, _)| *existing == pid) => {
                targets.push((pid, entry.process_name().to_string()));
            }
            Some(_) => {}
            None => hidden_owner = true,
        }
    }

    if targets.is_empty() {
        let detail = if hidden_owner {
            format!(
                "port {port} is in use but the owning process is hidden; \
                 re-run elevated to see and kill it"
            )
        } else {
            format!("nothing is holding port {port}")
        };
        return vec![KillOutcome {
            pid: 0,
            process_name: String::new(),
            port: Some(port),
            status: if hidden_owner {
                KillStatus::PermissionDenied
            } else {
                KillStatus::NotFound
            },
            detail,
            collateral_ports: Vec::new(),
        }];
    }

    targets
        .into_iter()
        .map(|(pid, _)| {
            let collateral = ports_by_pid
                .get(&pid)
                .map(|ports| ports.iter().copied().filter(|p| *p != port).collect())
                .unwrap_or_default();
            let mut outcome = kill_pid_inner(pid, mode, Some(port));
            outcome.collateral_ports = collateral;
            outcome
        })
        .collect()
}

/// Ports held by `pid` other than `port`, read from an existing scan.
///
/// Exposed so a UI can warn *before* killing, using the listing it already has,
/// rather than discovering the damage from the result.
pub fn other_ports_held_by(entries: &[crate::PortEntry], pid: u32, port: u16) -> Vec<u16> {
    let mut ports: BTreeSet<u16> = BTreeSet::new();
    for entry in entries {
        if entry.pid == Some(pid) && entry.port != port {
            ports.insert(entry.port);
        }
    }
    ports.into_iter().collect()
}

fn kill_pid_inner(pid: u32, mode: KillMode, port: Option<u16>) -> KillOutcome {
    let outcome = |status: KillStatus, name: String, detail: String| KillOutcome {
        pid,
        process_name: name,
        port,
        status,
        detail,
        collateral_ports: Vec::new(),
    };

    if let Some(reason) = refuse_reason(pid) {
        return outcome(KillStatus::Refused, String::new(), reason);
    }

    let mut system = System::new();
    let target = Pid::from_u32(pid);
    // The executable is refreshed so the name reported here matches the name the
    // user saw in the listing they are acting on.
    let refresh = ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet);
    system.refresh_processes_specifics(ProcessesToUpdate::Some(&[target]), true, refresh);

    let Some(process) = system.process(target) else {
        return outcome(
            KillStatus::NotFound,
            String::new(),
            format!("no process with PID {pid} (it may have already exited)"),
        );
    };
    let name = process_name(process);

    // `kill_with` returns None when the platform has no such signal, which is
    // the normal case for SIGTERM on Windows.
    let delivered = match mode {
        KillMode::Force => process
            .kill_with(Signal::Kill)
            .unwrap_or_else(|| process.kill()),
        KillMode::Graceful => process
            .kill_with(Signal::Term)
            .unwrap_or_else(|| process.kill()),
    };

    if !delivered {
        return outcome(
            KillStatus::PermissionDenied,
            name,
            permission_hint(pid, port),
        );
    }

    let grace = match mode {
        KillMode::Graceful => GRACE_PERIOD,
        KillMode::Force => FORCE_TIMEOUT,
    };

    if wait_until_gone(pid, grace) {
        return outcome(
            KillStatus::Killed,
            name.clone(),
            match port {
                Some(port) => format!("port {port} freed: {name} (PID {pid}) terminated"),
                None => format!("{name} (PID {pid}) terminated"),
            },
        );
    }

    // Ignored SIGTERM: escalate, because "it said it worked but the port is
    // still taken" is the worst possible outcome for this tool.
    if mode == KillMode::Graceful {
        let mut system = System::new();
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[target]),
            true,
            ProcessRefreshKind::nothing(),
        );
        if let Some(process) = system.process(target) {
            let forced = process
                .kill_with(Signal::Kill)
                .unwrap_or_else(|| process.kill());
            if !forced {
                return outcome(
                    KillStatus::PermissionDenied,
                    name,
                    permission_hint(pid, port),
                );
            }
            if wait_until_gone(pid, FORCE_TIMEOUT) {
                return outcome(
                    KillStatus::Escalated,
                    name.clone(),
                    format!("{name} (PID {pid}) ignored SIGTERM and was force-killed"),
                );
            }
        } else {
            return outcome(
                KillStatus::Killed,
                name.clone(),
                format!("{name} (PID {pid}) terminated"),
            );
        }
    }

    outcome(
        KillStatus::Survived,
        name.clone(),
        format!("{name} (PID {pid}) is still running after a force kill"),
    )
}

/// Portify's own safety rules. Returns `Some(reason)` when the kill must not
/// even be attempted.
fn refuse_reason(pid: u32) -> Option<String> {
    if pid == 0 {
        return Some("PID 0 is not a killable process".to_string());
    }

    if let Ok(current) = sysinfo::get_current_pid() {
        if current.as_u32() == pid {
            return Some("refusing to kill Portify itself".to_string());
        }
    }

    #[cfg(unix)]
    if pid == 1 {
        return Some("refusing to kill PID 1 (init) — it would take the system down".to_string());
    }

    #[cfg(windows)]
    if pid == 4 {
        return Some("refusing to kill PID 4 (Windows System process)".to_string());
    }

    None
}

fn permission_hint(pid: u32, port: Option<u16>) -> String {
    let subject = match port {
        Some(port) => format!("the process on port {port} (PID {pid})"),
        None => format!("PID {pid}"),
    };
    if cfg!(windows) {
        format!("access denied killing {subject}; run Portify as Administrator")
    } else {
        format!("access denied killing {subject}; re-run with sudo")
    }
}

/// Poll the process table until the PID disappears, or the timeout elapses.
fn wait_until_gone(pid: u32, timeout: Duration) -> bool {
    let target = Pid::from_u32(pid);
    let deadline = Instant::now() + timeout;
    let mut system = System::new();

    loop {
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[target]),
            true,
            ProcessRefreshKind::nothing(),
        );
        match system.process(target) {
            None => return true,
            // A zombie has released its ports and is only waiting to be reaped,
            // which is as good as gone for our purposes.
            Some(process) if is_zombie(process) => return true,
            Some(_) => {}
        }

        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

fn is_zombie(process: &sysinfo::Process) -> bool {
    matches!(process.status(), sysinfo::ProcessStatus::Zombie)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_to_kill_itself() {
        let current = sysinfo::get_current_pid().unwrap().as_u32();
        let outcome = kill_pid(current, KillMode::Graceful);
        assert_eq!(outcome.status, KillStatus::Refused);
        assert!(outcome.detail.contains("itself"), "{}", outcome.detail);
    }

    #[test]
    fn refuses_pid_zero() {
        assert_eq!(kill_pid(0, KillMode::Force).status, KillStatus::Refused);
    }

    #[cfg(unix)]
    #[test]
    fn refuses_init() {
        let outcome = kill_pid(1, KillMode::Force);
        assert_eq!(outcome.status, KillStatus::Refused);
    }

    #[test]
    fn missing_pid_is_reported_not_panicked() {
        // A PID this high is above the default pid_max on Linux and is not a
        // valid Windows handle either.
        let outcome = kill_pid(4_294_000_000, KillMode::Graceful);
        assert_eq!(outcome.status, KillStatus::NotFound);
    }

    #[test]
    fn killing_a_free_port_reports_not_found() {
        // Port 1 is never bound on a normal machine, and binding it would need
        // root anyway.
        let outcomes = kill_port(1, None, KillMode::Graceful);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].status, KillStatus::NotFound);
        assert!(outcomes[0].detail.contains("nothing is holding"));
    }

    fn outcome_with(name: &str, ports: Vec<u16>) -> KillOutcome {
        KillOutcome {
            pid: 19064,
            process_name: name.to_string(),
            port: Some(6379),
            status: KillStatus::Killed,
            detail: String::new(),
            collateral_ports: ports,
        }
    }

    #[test]
    fn no_warning_when_the_process_only_holds_the_one_port() {
        assert!(outcome_with("node", Vec::new())
            .collateral_warning()
            .is_none());
    }

    #[test]
    fn warns_about_a_single_extra_port() {
        let warning = outcome_with("node", vec![3001])
            .collateral_warning()
            .unwrap();
        assert!(warning.contains("also held port 3001"), "{warning}");
    }

    #[test]
    fn warns_and_truncates_for_a_relay_holding_everything() {
        // The real case: one WSL relay fronting every forwarded port.
        let ports: Vec<u16> = (4510..=4566).collect();
        let warning = outcome_with("wslrelay.exe", ports.clone())
            .collateral_warning()
            .unwrap();
        assert!(warning.contains("wslrelay.exe"), "{warning}");
        assert!(warning.contains(&ports.len().to_string()), "{warning}");
        assert!(warning.contains("4510"), "{warning}");
        assert!(warning.contains('…'), "long lists are truncated: {warning}");
    }

    #[test]
    fn other_ports_excludes_the_target_and_other_pids() {
        use crate::scan::{ProcessInfo, Protocol, SocketState};

        let entry = |port: u16, pid: u32| crate::PortEntry {
            port,
            protocol: Protocol::Tcp,
            state: SocketState::Listen,
            local_address: format!("0.0.0.0:{port}"),
            remote_address: None,
            pid: Some(pid),
            process: Some(ProcessInfo {
                pid,
                name: "wslrelay.exe".into(),
                description: Some("WSL port forward"),
                exe: None,
                command: None,
                memory_bytes: None,
                started_at: None,
            }),
            service: None,
            owner_hidden: false,
        };

        let entries = vec![
            entry(6379, 19064),
            entry(4510, 19064),
            entry(4511, 19064),
            // Same port number, different process: must not be counted.
            entry(3000, 999),
        ];

        assert_eq!(other_ports_held_by(&entries, 19064, 6379), vec![4510, 4511]);
        assert!(other_ports_held_by(&entries, 999, 3000).is_empty());
    }

    #[test]
    fn success_statuses() {
        assert!(KillStatus::Killed.is_success());
        assert!(KillStatus::Escalated.is_success());
        assert!(!KillStatus::PermissionDenied.is_success());
        assert!(!KillStatus::Refused.is_success());
    }
}
