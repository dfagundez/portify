//! Reading the socket table and attaching process identity to it.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::net::IpAddr;

use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::error::{Error, Result};
use crate::services::{describe_process, service_for_port};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        }
    }
}

// `as_str` stays the single source of truth; Display just means callers can
// write `{protocol}` and `.to_string()` without reaching for it. Worth having
// now that this crate is published for other people to use.
impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Socket state, normalised across platforms.
///
/// UDP has no state machine, so a bound UDP socket is reported as
/// [`SocketState::Bound`] rather than pretending it is listening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SocketState {
    Listen,
    Bound,
    Established,
    SynSent,
    SynReceived,
    FinWait,
    CloseWait,
    Closing,
    TimeWait,
    Closed,
    Unknown,
}

impl SocketState {
    /// True when the socket is holding the port open for new work — the only
    /// thing that matters when you are trying to free a port.
    pub fn is_serving(self) -> bool {
        matches!(self, SocketState::Listen | SocketState::Bound)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            SocketState::Listen => "LISTEN",
            SocketState::Bound => "BOUND",
            SocketState::Established => "ESTABLISHED",
            SocketState::SynSent => "SYN_SENT",
            SocketState::SynReceived => "SYN_RECV",
            SocketState::FinWait => "FIN_WAIT",
            SocketState::CloseWait => "CLOSE_WAIT",
            SocketState::Closing => "CLOSING",
            SocketState::TimeWait => "TIME_WAIT",
            SocketState::Closed => "CLOSED",
            SocketState::Unknown => "UNKNOWN",
        }
    }
}

impl std::fmt::Display for SocketState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<TcpState> for SocketState {
    fn from(state: TcpState) -> Self {
        match state {
            TcpState::Listen => SocketState::Listen,
            TcpState::Established => SocketState::Established,
            TcpState::SynSent => SocketState::SynSent,
            TcpState::SynReceived => SocketState::SynReceived,
            TcpState::FinWait1 | TcpState::FinWait2 => SocketState::FinWait,
            TcpState::CloseWait => SocketState::CloseWait,
            TcpState::Closing | TcpState::LastAck => SocketState::Closing,
            TcpState::TimeWait => SocketState::TimeWait,
            TcpState::Closed | TcpState::DeleteTcb => SocketState::Closed,
            _ => SocketState::Unknown,
        }
    }
}

/// Identity of the process holding a socket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    /// Full path to the executable, when the OS lets us read it.
    pub exe: Option<String>,
    /// Full command line, useful to tell three `node` processes apart.
    pub command: Option<String>,
    /// What this executable is, when it is one we recognise ("WSL port
    /// forward", "Windows service host"). Unlike the port hint, this is
    /// observed rather than assumed.
    pub description: Option<&'static str>,
    /// Resident memory in bytes.
    pub memory_bytes: Option<u64>,
    /// Unix timestamp (seconds) when the process started.
    pub started_at: Option<u64>,
}

/// One socket, with whatever we could learn about its owner.
#[derive(Debug, Clone, Serialize)]
pub struct PortEntry {
    pub port: u16,
    pub protocol: Protocol,
    pub state: SocketState,
    pub local_address: String,
    pub remote_address: Option<String>,
    pub pid: Option<u32>,
    pub process: Option<ProcessInfo>,
    pub service: Option<&'static str>,
    /// True when the socket exists but the OS would not tell us who owns it.
    /// Almost always means "another user's process, re-run elevated".
    pub owner_hidden: bool,
}

impl PortEntry {
    /// Best available label for the owning process.
    pub fn process_name(&self) -> &str {
        match &self.process {
            Some(p) => &p.name,
            None if self.owner_hidden => "(hidden)",
            None => "(system)",
        }
    }
}

/// A port collapsed across all of its sockets.
///
/// A single dev server routinely produces one socket per address family per
/// interface; listing them raw is what makes `netstat` output unreadable.
#[derive(Debug, Clone, Serialize)]
pub struct PortGroup {
    pub port: u16,
    pub protocol: Protocol,
    pub service: Option<&'static str>,
    /// Distinct processes holding the port (normally exactly one).
    pub processes: Vec<ProcessInfo>,
    /// Distinct local addresses the port is bound to.
    pub addresses: Vec<String>,
    /// How many serving sockets were collapsed into this row.
    pub sockets: usize,
    /// How many established connections currently sit on this port.
    pub connections: usize,
    pub owner_hidden: bool,
}

impl PortGroup {
    pub fn primary_pid(&self) -> Option<u32> {
        self.processes.first().map(|p| p.pid)
    }

    pub fn process_label(&self) -> String {
        match self.processes.len() {
            0 if self.owner_hidden => "(hidden)".to_string(),
            0 => "(system)".to_string(),
            1 => self.processes[0].name.clone(),
            n => format!("{} (+{} more)", self.processes[0].name, n - 1),
        }
    }
}

/// What to include in a scan.
#[derive(Debug, Clone, Copy)]
pub struct ScanOptions {
    pub include_tcp: bool,
    pub include_udp: bool,
    pub include_ipv4: bool,
    pub include_ipv6: bool,
    /// Keep only sockets that are holding a port open (LISTEN / bound UDP).
    pub only_serving: bool,
    /// Include UDP sockets bound to an OS-assigned ephemeral port.
    ///
    /// These are outbound client sockets — a browser talking QUIC, a VPN agent
    /// phoning home — not services anyone is trying to free. On Windows they
    /// outnumber real listeners roughly two to one, so they are hidden unless
    /// asked for.
    pub include_ephemeral_udp: bool,
    /// Resolve executable path and command line. Slightly more expensive, and
    /// not needed for the tray menu.
    pub with_details: bool,
}

impl Default for ScanOptions {
    /// The default answers the question the tool exists for: which ports are
    /// currently taken, over any protocol or address family.
    fn default() -> Self {
        Self {
            include_tcp: true,
            include_udp: true,
            include_ipv4: true,
            include_ipv6: true,
            only_serving: true,
            include_ephemeral_udp: false,
            with_details: true,
        }
    }
}

/// Start of the IANA dynamic/private port range, which is also the default
/// Windows ephemeral range and the usual Linux one.
pub const EPHEMERAL_PORT_START: u16 = 49152;

impl ScanOptions {
    /// Everything: transient connection states and ephemeral client sockets.
    pub fn everything() -> Self {
        Self {
            only_serving: false,
            include_ephemeral_udp: true,
            ..Self::default()
        }
    }

    fn address_flags(&self) -> AddressFamilyFlags {
        let mut flags = AddressFamilyFlags::empty();
        if self.include_ipv4 {
            flags |= AddressFamilyFlags::IPV4;
        }
        if self.include_ipv6 {
            flags |= AddressFamilyFlags::IPV6;
        }
        flags
    }

    fn protocol_flags(&self) -> ProtocolFlags {
        let mut flags = ProtocolFlags::empty();
        if self.include_tcp {
            flags |= ProtocolFlags::TCP;
        }
        if self.include_udp {
            flags |= ProtocolFlags::UDP;
        }
        flags
    }
}

/// Read the socket table and attach process identity.
pub fn scan(options: &ScanOptions) -> Result<Vec<PortEntry>> {
    let address_flags = options.address_flags();
    let protocol_flags = options.protocol_flags();
    if address_flags.is_empty() || protocol_flags.is_empty() {
        return Ok(Vec::new());
    }

    let sockets = get_sockets_info(address_flags, protocol_flags)
        .map_err(|err| Error::SocketTable(err.to_string()))?;

    // Collect the PIDs first so process metadata is resolved in one pass
    // instead of once per socket.
    let mut pids: HashSet<u32> = HashSet::new();
    for socket in &sockets {
        pids.extend(socket.associated_pids.iter().copied());
    }
    let processes = resolve_processes(&pids, options.with_details);

    let mut entries = Vec::with_capacity(sockets.len());
    for socket in sockets {
        let (port, state, local_addr, remote) = match &socket.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => (
                tcp.local_port,
                SocketState::from(tcp.state),
                tcp.local_addr,
                Some(format_addr(tcp.remote_addr, tcp.remote_port)),
            ),
            ProtocolSocketInfo::Udp(udp) => {
                (udp.local_port, SocketState::Bound, udp.local_addr, None)
            }
        };

        if options.only_serving && !state.is_serving() {
            continue;
        }

        let protocol = match &socket.protocol_socket_info {
            ProtocolSocketInfo::Tcp(_) => Protocol::Tcp,
            ProtocolSocketInfo::Udp(_) => Protocol::Udp,
        };

        // TCP listeners on high ports are real services (Windows RPC lives up
        // there). UDP sockets on high ports are almost always the client half of
        // an outbound conversation, so they are dropped by default.
        if is_ephemeral_udp(protocol, port) && !options.include_ephemeral_udp {
            continue;
        }

        // A socket with no associated PID means the OS refused to name the
        // owner, not that nobody owns it.
        let pid = socket.associated_pids.first().copied();
        let process = pid.and_then(|pid| processes.get(&pid).cloned());
        let owner_hidden = pid.is_none() || (pid.is_some() && process.is_none());

        entries.push(PortEntry {
            port,
            protocol,
            state,
            local_address: format_addr(local_addr, port),
            remote_address: remote.filter(|_| !state.is_serving()),
            pid,
            process,
            service: service_for_port(port),
            owner_hidden,
        });
    }

    entries.sort_by(|a, b| {
        a.port
            .cmp(&b.port)
            .then(a.protocol.cmp(&b.protocol))
            .then(a.local_address.cmp(&b.local_address))
    });

    Ok(entries)
}

/// Collapse raw sockets into one row per (port, protocol).
pub fn group_by_port(entries: &[PortEntry]) -> Vec<PortGroup> {
    let mut grouped: BTreeMap<(u16, Protocol), PortGroup> = BTreeMap::new();

    for entry in entries {
        let group = grouped
            .entry((entry.port, entry.protocol))
            .or_insert_with(|| PortGroup {
                port: entry.port,
                protocol: entry.protocol,
                service: entry.service,
                processes: Vec::new(),
                addresses: Vec::new(),
                sockets: 0,
                connections: 0,
                owner_hidden: false,
            });

        if entry.state.is_serving() {
            group.sockets += 1;
            if !group.addresses.contains(&entry.local_address) {
                group.addresses.push(entry.local_address.clone());
            }
        } else {
            group.connections += 1;
        }

        if let Some(process) = &entry.process {
            if !group.processes.iter().any(|p| p.pid == process.pid) {
                group.processes.push(process.clone());
            }
        } else if entry.owner_hidden {
            group.owner_hidden = true;
        }
    }

    grouped.into_values().collect()
}

fn resolve_processes(pids: &HashSet<u32>, with_details: bool) -> HashMap<u32, ProcessInfo> {
    if pids.is_empty() {
        return HashMap::new();
    }

    // The executable path is always refreshed: it is cheap (one symlink read)
    // and it is what makes process names trustworthy, which matters just as much
    // in the tray menu as in the CLI. Only the full command line is optional.
    let cmd_update = if with_details {
        UpdateKind::OnlyIfNotSet
    } else {
        UpdateKind::Never
    };
    let refresh_kind = ProcessRefreshKind::nothing()
        .with_exe(UpdateKind::OnlyIfNotSet)
        .with_cmd(cmd_update)
        .with_memory();

    let wanted: Vec<Pid> = pids.iter().map(|pid| Pid::from_u32(*pid)).collect();
    let mut system = System::new();
    system.refresh_processes_specifics(ProcessesToUpdate::Some(&wanted), true, refresh_kind);

    let mut resolved = HashMap::with_capacity(pids.len());
    for pid in pids {
        if let Some(process) = system.process(Pid::from_u32(*pid)) {
            let command = if with_details {
                let parts: Vec<String> = process
                    .cmd()
                    .iter()
                    .map(|arg| arg.to_string_lossy().into_owned())
                    .collect();
                Some(parts.join(" ")).filter(|c| !c.is_empty())
            } else {
                None
            };

            let name = process_name(process);
            resolved.insert(
                *pid,
                ProcessInfo {
                    pid: *pid,
                    description: describe_process(&name),
                    name,
                    exe: if with_details {
                        process
                            .exe()
                            .map(|path| path.to_string_lossy().into_owned())
                    } else {
                        None
                    },
                    command,
                    // Windows reports 0 for a process it will not let us open
                    // for query — that is "unknown", not "uses no memory", and
                    // printing "0 B" for half of `svchost.exe` is a lie the UI
                    // would repeat. No live process has zero resident memory.
                    memory_bytes: Some(process.memory()).filter(|bytes| *bytes > 0),
                    started_at: Some(process.start_time()).filter(|secs| *secs > 0),
                },
            );
        }
    }

    resolved
}

/// True for a UDP socket the OS handed out from the dynamic range.
fn is_ephemeral_udp(protocol: Protocol, port: u16) -> bool {
    protocol == Protocol::Udp && port >= EPHEMERAL_PORT_START
}

/// Best name for a process.
///
/// On Linux `Process::name` comes from `/proc/<pid>/stat`, which holds the
/// *thread* name capped at 15 bytes — so a Python service that names its main
/// thread shows up as "MainThread", and a long binary name gets silently
/// truncated. The executable's file name is both accurate and stable, so it
/// wins whenever the OS gives it to us.
pub(crate) fn process_name(process: &sysinfo::Process) -> String {
    if let Some(file_name) = process
        .exe()
        .and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
    {
        if !file_name.is_empty() {
            return file_name;
        }
    }
    process.name().to_string_lossy().into_owned()
}

fn format_addr(addr: IpAddr, port: u16) -> String {
    match addr {
        IpAddr::V4(v4) => format!("{v4}:{port}"),
        IpAddr::V6(v6) => format!("[{v6}]:{port}"),
    }
}

/// Human-readable size, used by both front ends.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(port: u16, protocol: Protocol, state: SocketState, addr: &str) -> PortEntry {
        PortEntry {
            port,
            protocol,
            state,
            local_address: addr.to_string(),
            remote_address: None,
            pid: Some(42),
            process: Some(ProcessInfo {
                pid: 42,
                name: "node".into(),
                description: None,
                exe: None,
                command: None,
                memory_bytes: None,
                started_at: None,
            }),
            service: service_for_port(port),
            owner_hidden: false,
        }
    }

    #[test]
    fn ephemeral_udp_is_recognised_but_ephemeral_tcp_is_not() {
        // Windows RPC services genuinely listen on high TCP ports, so the rule
        // must apply to UDP only.
        assert!(is_ephemeral_udp(Protocol::Udp, 54995));
        assert!(is_ephemeral_udp(Protocol::Udp, EPHEMERAL_PORT_START));
        assert!(!is_ephemeral_udp(Protocol::Udp, EPHEMERAL_PORT_START - 1));
        assert!(!is_ephemeral_udp(Protocol::Udp, 5353), "mDNS is a service");
        assert!(!is_ephemeral_udp(Protocol::Tcp, 49664), "RPC listener");
    }

    #[test]
    fn everything_stops_hiding_ephemeral_sockets() {
        assert!(!ScanOptions::default().include_ephemeral_udp);
        assert!(ScanOptions::everything().include_ephemeral_udp);
        assert!(!ScanOptions::everything().only_serving);
    }

    #[test]
    fn the_default_scan_hides_ephemeral_udp() {
        let default = scan(&ScanOptions::default()).expect("scan should succeed");
        assert!(
            !default.iter().any(|e| is_ephemeral_udp(e.protocol, e.port)),
            "default view must not contain outbound UDP client sockets"
        );
    }

    #[test]
    fn memory_is_absent_rather_than_zero_when_unknown() {
        // Zero resident memory is not a thing a running process has; if the OS
        // refuses to tell us, the field must be None so the UI can say so.
        let entries = scan(&ScanOptions::default()).expect("scan should succeed");
        for entry in &entries {
            if let Some(process) = &entry.process {
                assert_ne!(
                    process.memory_bytes,
                    Some(0),
                    "{} reported 0 bytes instead of unknown",
                    process.name
                );
            }
        }
    }

    #[test]
    fn scan_reads_the_socket_table() {
        // The machine running tests always has at least one listening socket in
        // practice, but an empty result is still a pass: what must not happen is
        // an error.
        let entries = scan(&ScanOptions::default()).expect("scan should succeed");
        for entry in &entries {
            assert!(entry.state.is_serving(), "default scan is serving-only");
        }
    }

    #[test]
    fn grouping_collapses_one_port_bound_to_many_addresses() {
        let entries = vec![
            entry(3000, Protocol::Tcp, SocketState::Listen, "0.0.0.0:3000"),
            entry(3000, Protocol::Tcp, SocketState::Listen, "[::]:3000"),
            entry(
                3000,
                Protocol::Tcp,
                SocketState::Established,
                "127.0.0.1:3000",
            ),
        ];

        let groups = group_by_port(&entries);
        assert_eq!(groups.len(), 1, "one port, one row");

        let group = &groups[0];
        assert_eq!(group.sockets, 2);
        assert_eq!(group.connections, 1);
        assert_eq!(group.addresses.len(), 2);
        assert_eq!(group.processes.len(), 1, "same PID must not repeat");
        assert_eq!(group.primary_pid(), Some(42));
    }

    #[test]
    fn tcp_and_udp_on_the_same_port_stay_separate() {
        let entries = vec![
            entry(53, Protocol::Tcp, SocketState::Listen, "127.0.0.1:53"),
            entry(53, Protocol::Udp, SocketState::Bound, "127.0.0.1:53"),
        ];
        assert_eq!(group_by_port(&entries).len(), 2);
    }

    #[test]
    fn udp_is_reported_as_bound_and_counts_as_serving() {
        assert!(SocketState::Bound.is_serving());
        assert!(SocketState::Listen.is_serving());
        assert!(!SocketState::Established.is_serving());
        assert!(!SocketState::TimeWait.is_serving());
    }

    #[test]
    fn formats_sizes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MB");
    }
}
