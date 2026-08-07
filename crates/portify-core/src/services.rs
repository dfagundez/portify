//! Human-readable hints for well-known ports.
//!
//! Biased towards what a developer actually meets on localhost, not towards the
//! full IANA registry: the point is to make a port list scannable at a glance.

/// Well-known ports, sorted by port so lookups can binary-search.
const SERVICES: &[(u16, &str)] = &[
    (21, "FTP"),
    (22, "SSH"),
    (23, "Telnet"),
    (25, "SMTP"),
    (53, "DNS"),
    (80, "HTTP"),
    (110, "POP3"),
    (143, "IMAP"),
    (389, "LDAP"),
    (443, "HTTPS"),
    (445, "SMB"),
    (587, "SMTP (submission)"),
    (631, "IPP / CUPS"),
    (993, "IMAPS"),
    (995, "POP3S"),
    (1080, "SOCKS proxy"),
    (1433, "SQL Server"),
    (1521, "Oracle DB"),
    (1883, "MQTT"),
    (2049, "NFS"),
    (2375, "Docker (plain)"),
    (2376, "Docker (TLS)"),
    (3000, "Dev server (Node/Rails/Grafana)"),
    (3001, "Dev server"),
    (3306, "MySQL / MariaDB"),
    (3389, "RDP"),
    (4000, "Dev server (Phoenix/Jekyll)"),
    (4200, "Angular dev server"),
    (4318, "OpenTelemetry (HTTP)"),
    (4369, "Erlang port mapper"),
    (5000, "Dev server (Flask/ASP.NET)"),
    (5001, "Dev server"),
    (5173, "Vite dev server"),
    (5432, "PostgreSQL"),
    (5672, "RabbitMQ"),
    (5900, "VNC"),
    (6006, "Storybook / TensorBoard"),
    (6379, "Redis"),
    (7000, "Dev server / AirPlay"),
    (7687, "Neo4j (Bolt)"),
    (8000, "Dev server (Django/FastAPI)"),
    (8080, "HTTP alternate (Tomcat/proxy)"),
    (8081, "HTTP alternate"),
    (8086, "InfluxDB"),
    (8088, "HTTP alternate"),
    (8443, "HTTPS alternate"),
    (8888, "Jupyter / HTTP alternate"),
    (9000, "Dev server (PHP-FPM/MinIO/SonarQube)"),
    (9090, "Prometheus"),
    (9092, "Kafka"),
    (9200, "Elasticsearch"),
    (9229, "Node.js debugger"),
    (9300, "Elasticsearch (transport)"),
    (11211, "Memcached"),
    (15672, "RabbitMQ (management)"),
    (16686, "Jaeger UI"),
    (27017, "MongoDB"),
    (50051, "gRPC"),
];

/// Returns a short description for a well-known port, if there is one.
///
/// This is a *convention*, not detection: it reports what a port is normally
/// used for, and says "Redis" for anything sitting on 6379. Where the owning
/// process is known, [`describe_process`] is the more trustworthy answer,
/// because it describes what is actually running.
pub fn service_for_port(port: u16) -> Option<&'static str> {
    SERVICES
        .binary_search_by_key(&port, |(p, _)| *p)
        .ok()
        .map(|idx| SERVICES[idx].1)
}

/// What a well-known executable actually is.
///
/// Matched on the executable's file name, so unlike [`service_for_port`] this
/// is a fact about the running process rather than a guess from a number. The
/// list is deliberately short: system processes that turn a listing into a wall
/// of identical names, plus the plumbing developers meet every day.
const PROCESSES: &[(&str, &str)] = &[
    // Windows plumbing — the repeated names that tell you nothing on their own.
    ("system", "Windows kernel"),
    ("svchost.exe", "Windows service host"),
    ("lsass.exe", "Windows security (LSA)"),
    ("services.exe", "Windows service manager"),
    ("wininit.exe", "Windows startup"),
    ("spoolsv.exe", "Print spooler"),
    ("dashost.exe", "Device association"),
    ("wudfhost.exe", "User-mode driver host"),
    ("searchindexer.exe", "Windows Search"),
    ("msedgewebview2.exe", "Edge WebView2"),
    ("crossdeviceservice.exe", "Windows Phone Link"),
    ("shellexperiencehost.exe", "Windows shell UI"),
    // Developer plumbing that fronts other people's ports.
    ("wslrelay.exe", "WSL port forward"),
    ("wslhost.exe", "WSL host"),
    ("wslservice.exe", "WSL service"),
    ("vpnkit.exe", "Docker network proxy"),
    ("com.docker.backend.exe", "Docker Desktop"),
    ("com.docker.backend", "Docker Desktop"),
    ("docker-proxy", "Docker port mapping"),
    ("dockerd", "Docker daemon"),
    ("containerd", "containerd"),
    // Common runtimes, so an unknown high port at least says what kind of thing
    // is on it.
    ("node.exe", "Node.js"),
    ("node", "Node.js"),
    ("deno", "Deno"),
    ("bun", "Bun"),
    ("python.exe", "Python"),
    ("python3", "Python"),
    ("java.exe", "Java"),
    ("java", "Java"),
    ("ruby", "Ruby"),
    ("dotnet.exe", ".NET"),
    ("dotnet", ".NET"),
    ("postgres", "PostgreSQL"),
    ("mysqld", "MySQL"),
    ("redis-server", "Redis"),
    ("mongod", "MongoDB"),
    ("nginx", "nginx"),
];

/// What a process is, matched on its executable name (case-insensitive).
///
/// Note on `svchost.exe`: Windows runs dozens of unrelated services inside
/// copies of it, so "Windows service host" is as specific as a name lookup can
/// honestly get. Naming the individual service means asking the service control
/// manager which services live in that PID — a real feature, but a much larger
/// one than a table.
pub fn describe_process(executable_name: &str) -> Option<&'static str> {
    let needle = executable_name.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return None;
    }
    PROCESSES
        .iter()
        .find(|(name, _)| *name == needle)
        .map(|(_, description)| *description)
}

/// Ports below 1024 need elevation to *bind* on Unix. Useful as a UI hint when
/// a kill fails.
pub fn is_privileged_port(port: u16) -> bool {
    port < 1024
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_sorted_so_binary_search_is_valid() {
        for pair in SERVICES.windows(2) {
            assert!(
                pair[0].0 < pair[1].0,
                "SERVICES must be strictly sorted by port; {} >= {}",
                pair[0].0,
                pair[1].0
            );
        }
    }

    #[test]
    fn looks_up_known_and_unknown_ports() {
        assert_eq!(service_for_port(5432), Some("PostgreSQL"));
        assert_eq!(service_for_port(5173), Some("Vite dev server"));
        assert_eq!(service_for_port(54321), None);
    }

    #[test]
    fn describes_the_processes_that_flood_a_windows_listing() {
        assert_eq!(
            describe_process("svchost.exe"),
            Some("Windows service host")
        );
        assert_eq!(describe_process("wslrelay.exe"), Some("WSL port forward"));
        assert_eq!(describe_process("System"), Some("Windows kernel"));
    }

    #[test]
    fn process_lookup_ignores_case() {
        assert_eq!(
            describe_process("SVCHOST.EXE"),
            describe_process("svchost.exe")
        );
        assert_eq!(describe_process("  node  "), Some("Node.js"));
    }

    #[test]
    fn unknown_and_empty_names_describe_nothing() {
        assert_eq!(describe_process("my-own-server.exe"), None);
        assert_eq!(describe_process(""), None);
        assert_eq!(describe_process("   "), None);
    }

    #[test]
    fn process_identity_and_port_convention_are_separate_answers() {
        // Port 6379 says "Redis" by convention even when the process fronting it
        // is a WSL relay. Both facts are true and the UI shows the process one
        // first, because it is the one that is actually observed.
        assert_eq!(service_for_port(6379), Some("Redis"));
        assert_eq!(describe_process("wslrelay.exe"), Some("WSL port forward"));
    }

    #[test]
    fn privileged_ports() {
        assert!(is_privileged_port(80));
        assert!(!is_privileged_port(3000));
    }
}
