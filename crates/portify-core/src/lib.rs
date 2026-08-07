//! Portify core: everything the CLI and the desktop app need to answer
//! "what is holding this port, and how do I get it back?".
//!
//! The crate is deliberately free of UI concerns and of async: a full scan of a
//! developer machine takes single-digit milliseconds, so callers can just call
//! [`scan`] on a timer.

pub mod error;
pub mod kill;
pub mod scan;
pub mod services;
pub mod system;

pub use error::{Error, Result};
pub use kill::{kill_pid, kill_port, other_ports_held_by, KillMode, KillOutcome, KillStatus};
pub use scan::{
    group_by_port, scan, PortEntry, PortGroup, ProcessInfo, Protocol, ScanOptions, SocketState,
};
pub use services::service_for_port;
pub use system::{is_elevated, system_summary, SystemSummary};

/// Version of the Portify core, reported by the CLI and the app.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
