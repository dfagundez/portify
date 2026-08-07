//! Command-line surface.

use clap::{Args, Parser, Subcommand, ValueEnum};
use portify_core::Protocol;

const AFTER_HELP: &str = "\x1b[1mExamples:\x1b[0m
  portify                     List every port currently in use
  portify 3000                Show what is holding port 3000
  portify kill 3000           Free port 3000
  portify kill 3000 8080 -y   Free several ports without confirmation
  portify kill --pid 12841    Kill a specific process
  portify watch               Live view, refreshed every 2s
  portify list --json         Machine-readable output for scripts

\x1b[1mExit codes:\x1b[0m
  0 success   2 nothing found   3 permission denied   4 bad input   5 internal error";

#[derive(Debug, Parser)]
#[command(
    name = "portify",
    version,
    about = "Find out what is holding a port and take it back",
    after_help = AFTER_HELP,
    subcommand_negates_reqs = true,
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Ports to look up when no subcommand is given (e.g. `portify 3000`).
    #[arg(value_name = "PORT")]
    pub ports: Vec<u16>,

    /// Output JSON instead of a table.
    #[arg(long, global = true)]
    pub json: bool,

    /// Never use colour or styling.
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List ports currently in use (default)
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// Free a port, or kill a process by PID
    #[command(visible_alias = "k")]
    Kill(KillArgs),

    /// Live view of ports, refreshed on an interval
    #[command(visible_alias = "w")]
    Watch(WatchArgs),

    /// Show host and permission information
    Info,

    /// Print a shell completion script
    Completions {
        /// Shell to generate for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Debug, Args, Clone)]
pub struct ListArgs {
    /// Only these ports (repeatable, or given as bare arguments)
    #[arg(value_name = "PORT")]
    pub ports: Vec<u16>,

    /// Filter by process name, executable path or command line (case-insensitive)
    #[arg(short, long, value_name = "TEXT")]
    pub filter: Option<String>,

    /// Show everything: established connections and outbound UDP sockets too
    #[arg(short, long)]
    pub all: bool,

    /// Only these protocols (default: both)
    #[arg(long, value_enum, value_name = "PROTO")]
    pub proto: Option<ProtoArg>,

    /// One row per socket instead of one row per port
    #[arg(long)]
    pub raw: bool,

    /// Show the full command line of each process
    #[arg(long)]
    pub wide: bool,
}

#[derive(Debug, Args, Clone)]
pub struct KillArgs {
    /// Ports to free — whatever is listening on them gets terminated
    #[arg(value_name = "PORT")]
    pub ports: Vec<u16>,

    /// Kill by process ID instead of by port (repeatable)
    #[arg(long, value_name = "PID")]
    pub pid: Vec<u32>,

    /// Skip the confirmation prompt
    #[arg(short, long)]
    pub yes: bool,

    /// Go straight to SIGKILL instead of asking politely first
    #[arg(short, long)]
    pub force: bool,

    /// Restrict port lookups to one protocol
    #[arg(long, value_enum, value_name = "PROTO")]
    pub proto: Option<ProtoArg>,
}

#[derive(Debug, Args, Clone)]
pub struct WatchArgs {
    /// Refresh interval in seconds
    #[arg(short, long, default_value_t = 2, value_parser = clap::value_parser!(u64).range(1..=3600))]
    pub interval: u64,

    /// Only these ports
    #[arg(value_name = "PORT")]
    pub ports: Vec<u16>,

    /// Filter by process name, executable path or command line
    #[arg(short, long, value_name = "TEXT")]
    pub filter: Option<String>,

    /// Show everything: established connections and outbound UDP sockets too
    #[arg(short, long)]
    pub all: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProtoArg {
    Tcp,
    Udp,
}

impl From<ProtoArg> for Protocol {
    fn from(arg: ProtoArg) -> Self {
        match arg {
            ProtoArg::Tcp => Protocol::Tcp,
            ProtoArg::Udp => Protocol::Udp,
        }
    }
}

impl ListArgs {
    /// Bare `portify 3000` is shorthand for `portify list 3000`.
    pub fn with_ports(ports: Vec<u16>) -> Self {
        Self {
            ports,
            filter: None,
            all: false,
            proto: None,
            raw: false,
            wide: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn bare_ports_parse_without_a_subcommand() {
        let cli = Cli::try_parse_from(["portify", "3000", "8080"]).unwrap();
        assert!(cli.command.is_none());
        assert_eq!(cli.ports, vec![3000, 8080]);
    }

    #[test]
    fn kill_takes_ports_positionally() {
        let cli = Cli::try_parse_from(["portify", "kill", "3000", "-y"]).unwrap();
        match cli.command {
            Some(Command::Kill(args)) => {
                assert_eq!(args.ports, vec![3000]);
                assert!(args.yes);
                assert!(!args.force);
                assert!(args.pid.is_empty());
            }
            other => panic!("expected kill, got {other:?}"),
        }
    }

    #[test]
    fn kill_by_pid_is_explicit() {
        let cli = Cli::try_parse_from(["portify", "kill", "--pid", "12841"]).unwrap();
        match cli.command {
            Some(Command::Kill(args)) => {
                assert_eq!(args.pid, vec![12841]);
                assert!(args.ports.is_empty());
            }
            other => panic!("expected kill, got {other:?}"),
        }
    }

    #[test]
    fn ports_above_u16_are_rejected() {
        assert!(Cli::try_parse_from(["portify", "70000"]).is_err());
    }

    #[test]
    fn watch_interval_must_be_positive() {
        assert!(Cli::try_parse_from(["portify", "watch", "-i", "0"]).is_err());
        assert!(Cli::try_parse_from(["portify", "watch", "-i", "5"]).is_ok());
    }

    #[test]
    fn json_is_available_on_subcommands() {
        let cli = Cli::try_parse_from(["portify", "list", "--json"]).unwrap();
        assert!(cli.json);
    }
}
