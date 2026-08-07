//! Filtering.
//!
//! Every filter is a conjunction applied in a single pass over the scan result.
//! This is not incidental: filters that each re-derive their own subset from the
//! full list silently drop each other, which is exactly the bug this rewrite
//! replaces.

use portify_core::{PortEntry, Protocol};

#[derive(Debug, Default, Clone)]
pub struct Filters {
    /// Keep only these ports. Empty means "any port".
    pub ports: Vec<u16>,
    /// Case-insensitive substring matched against process name, executable path
    /// and full command line.
    pub text: Option<String>,
    /// Keep only this protocol. `None` means "any protocol".
    pub proto: Option<Protocol>,
}

impl Filters {
    pub fn is_empty(&self) -> bool {
        self.ports.is_empty() && self.text.is_none() && self.proto.is_none()
    }

    /// True when `entry` satisfies *all* active filters.
    pub fn matches(&self, entry: &PortEntry) -> bool {
        if !self.ports.is_empty() && !self.ports.contains(&entry.port) {
            return false;
        }
        if let Some(proto) = self.proto {
            if entry.protocol != proto {
                return false;
            }
        }
        if let Some(needle) = &self.text {
            let needle = needle.to_lowercase();
            let haystacks = [
                Some(entry.process_name().to_string()),
                entry.process.as_ref().and_then(|p| p.exe.clone()),
                entry.process.as_ref().and_then(|p| p.command.clone()),
            ];
            let hit = haystacks
                .iter()
                .flatten()
                .any(|value| value.to_lowercase().contains(&needle));
            if !hit {
                return false;
            }
        }
        true
    }

    pub fn apply(&self, entries: Vec<PortEntry>) -> Vec<PortEntry> {
        if self.is_empty() {
            return entries;
        }
        entries.into_iter().filter(|e| self.matches(e)).collect()
    }

    /// Description of what was searched for, used in "nothing found" messages.
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if !self.ports.is_empty() {
            let ports: Vec<String> = self.ports.iter().map(|p| p.to_string()).collect();
            parts.push(format!("port {}", ports.join(", ")));
        }
        if let Some(proto) = self.proto {
            parts.push(proto.as_str().to_string());
        }
        if let Some(text) = &self.text {
            parts.push(format!("matching \"{text}\""));
        }
        if parts.is_empty() {
            "ports in use".to_string()
        } else {
            format!("ports in use ({})", parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portify_core::{ProcessInfo, SocketState};

    fn entry(port: u16, proto: Protocol, name: &str, command: Option<&str>) -> PortEntry {
        PortEntry {
            port,
            protocol: proto,
            state: SocketState::Listen,
            local_address: format!("0.0.0.0:{port}"),
            remote_address: None,
            pid: Some(1000 + port as u32),
            process: Some(ProcessInfo {
                pid: 1000 + port as u32,
                name: name.to_string(),
                description: None,
                exe: Some(format!("/usr/bin/{name}")),
                command: command.map(str::to_string),
                memory_bytes: None,
                started_at: None,
            }),
            service: None,
            owner_hidden: false,
        }
    }

    fn fixture() -> Vec<PortEntry> {
        vec![
            entry(3000, Protocol::Tcp, "node", Some("node server.js")),
            entry(3000, Protocol::Udp, "node", Some("node server.js")),
            entry(5432, Protocol::Tcp, "postgres", Some("postgres -D /data")),
            entry(8080, Protocol::Tcp, "java", Some("java -jar app.jar")),
        ]
    }

    #[test]
    fn no_filters_keeps_everything() {
        let filters = Filters::default();
        assert!(filters.is_empty());
        assert_eq!(filters.apply(fixture()).len(), 4);
    }

    #[test]
    fn port_and_text_filters_compose_instead_of_overriding() {
        // The regression this rewrite exists to prevent: with both filters set,
        // the result must satisfy both, not just the last one applied.
        let filters = Filters {
            ports: vec![3000],
            text: Some("postgres".into()),
            proto: None,
        };
        assert!(
            filters.apply(fixture()).is_empty(),
            "port 3000 is not postgres, so the intersection is empty"
        );
    }

    #[test]
    fn port_and_proto_compose() {
        let filters = Filters {
            ports: vec![3000],
            text: None,
            proto: Some(Protocol::Udp),
        };
        let result = filters.apply(fixture());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].protocol, Protocol::Udp);
    }

    #[test]
    fn all_three_filters_compose() {
        let filters = Filters {
            ports: vec![3000],
            text: Some("server.js".into()),
            proto: Some(Protocol::Tcp),
        };
        let result = filters.apply(fixture());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].port, 3000);
        assert_eq!(result[0].protocol, Protocol::Tcp);
    }

    #[test]
    fn text_matches_name_exe_and_command_case_insensitively() {
        let by_name = Filters {
            text: Some("POSTGRES".into()),
            ..Default::default()
        };
        assert_eq!(by_name.apply(fixture()).len(), 1);

        let by_command = Filters {
            text: Some("app.jar".into()),
            ..Default::default()
        };
        assert_eq!(by_command.apply(fixture()).len(), 1);

        let by_exe = Filters {
            text: Some("/usr/bin/java".into()),
            ..Default::default()
        };
        assert_eq!(by_exe.apply(fixture()).len(), 1);
    }

    #[test]
    fn multiple_ports_are_a_union_within_the_port_filter() {
        let filters = Filters {
            ports: vec![3000, 8080],
            ..Default::default()
        };
        assert_eq!(filters.apply(fixture()).len(), 3);
    }

    #[test]
    fn describes_the_active_filters() {
        let filters = Filters {
            ports: vec![3000],
            text: Some("node".into()),
            proto: Some(Protocol::Tcp),
        };
        let described = filters.describe();
        assert!(described.contains("3000"));
        assert!(described.contains("TCP"));
        assert!(described.contains("node"));
    }
}
