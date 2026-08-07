//! Portify CLI.

mod cli;
mod filter;
mod render;

use std::io::{IsTerminal, Write};
use std::process::ExitCode as ProcessExitCode;
use std::time::{Duration, Instant};

use clap::{CommandFactory, Parser};
use portify_core::error::ExitCode;
use portify_core::{
    group_by_port, is_elevated, kill_pid, kill_port, other_ports_held_by, scan, system_summary,
    KillMode, KillOutcome, KillStatus, PortEntry, PortGroup, ScanOptions,
};

use cli::{Cli, Command, KillArgs, ListArgs, WatchArgs};
use filter::Filters;

fn main() -> ProcessExitCode {
    // Not `Cli::parse()`: that exits with clap's own code, which is 2 for a
    // usage error — the same 2 this CLI uses for "nothing found". A script
    // doing `portify 3000 || ...` could not tell "the port is free" from
    // "you misspelled the flag". Usage errors are bad input, so they get 4.
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            // clap prints --help and --version to stdout, real errors to
            // stderr, and `use_stderr()` is how it tells the two apart.
            let failed = error.use_stderr();
            let _ = error.print();
            return ProcessExitCode::from(if failed {
                ExitCode::InvalidInput as u8
            } else {
                ExitCode::Success as u8
            });
        }
    };

    if cli.no_color {
        // Honoured by anstream for every stream it wraps.
        // SAFETY: single-threaded, before any output has been produced.
        unsafe { std::env::set_var("NO_COLOR", "1") };
    }

    let code = run(cli);
    ProcessExitCode::from(code as u8)
}

fn run(cli: Cli) -> ExitCode {
    let json = cli.json;
    match cli.command {
        None => list(ListArgs::with_ports(cli.ports), json),
        Some(Command::List(args)) => list(args, json),
        Some(Command::Kill(args)) => kill(args, json),
        Some(Command::Watch(args)) => watch(args),
        Some(Command::Info) => info(json),
        Some(Command::Completions { shell }) => {
            let mut command = Cli::command();
            let name = command.get_name().to_string();
            clap_complete::generate(shell, &mut command, name, &mut std::io::stdout());
            ExitCode::Success
        }
    }
}

/// Build scan options from the flags shared by `list` and `watch`.
///
/// `explicit_ports` is true when the user named ports on the command line. That
/// turns a browse into a lookup, and a lookup must be able to find sockets the
/// browse view hides — otherwise `portify 54995` would claim a port is free
/// while something is plainly sitting on it.
fn scan_options(all: bool, explicit_ports: bool) -> ScanOptions {
    if all {
        return ScanOptions::everything();
    }
    ScanOptions {
        include_ephemeral_udp: explicit_ports,
        ..ScanOptions::default()
    }
}

fn list(args: ListArgs, json: bool) -> ExitCode {
    let filters = Filters {
        ports: args.ports.clone(),
        text: args.filter.clone(),
        proto: args.proto.map(Into::into),
    };

    let started = Instant::now();
    let entries = match scan(&scan_options(args.all, !args.ports.is_empty())) {
        Ok(entries) => entries,
        Err(err) => return fail(&err.to_string(), json),
    };
    let entries = filters.apply(entries);
    let elapsed = started.elapsed();

    if json {
        return print_list_json(&entries, args.raw, elapsed);
    }

    if entries.is_empty() {
        render::print_empty(&filters.describe());
        // Asking about a specific port and finding nothing is a meaningful
        // answer for scripts; an unfiltered empty list is not an error.
        return if filters.is_empty() {
            ExitCode::Success
        } else {
            ExitCode::NotFound
        };
    }

    let hidden = entries.iter().filter(|e| e.owner_hidden).count();

    if args.raw {
        render::print_entries(&entries, args.wide);
        render::print_summary(entries.len(), hidden, elapsed.as_millis(), is_elevated());
    } else {
        let groups = group_by_port(&entries);
        render::print_groups(&groups, args.wide);
        render::print_summary(groups.len(), hidden, elapsed.as_millis(), is_elevated());
    }

    ExitCode::Success
}

fn print_list_json(entries: &[PortEntry], raw: bool, elapsed: Duration) -> ExitCode {
    let payload = if raw {
        serde_json::json!({
            "portify": portify_core::VERSION,
            "elapsed_ms": elapsed.as_millis(),
            "elevated": is_elevated(),
            "sockets": entries,
        })
    } else {
        serde_json::json!({
            "portify": portify_core::VERSION,
            "elapsed_ms": elapsed.as_millis(),
            "elevated": is_elevated(),
            "ports": group_by_port(entries),
        })
    };

    match serde_json::to_string_pretty(&payload) {
        Ok(text) => {
            println!("{text}");
            if entries.is_empty() {
                ExitCode::NotFound
            } else {
                ExitCode::Success
            }
        }
        Err(err) => fail(&format!("could not serialise output: {err}"), false),
    }
}

fn kill(args: KillArgs, json: bool) -> ExitCode {
    if args.ports.is_empty() && args.pid.is_empty() {
        return fail_with(
            ExitCode::InvalidInput,
            "nothing to kill: pass a port (portify kill 3000) or a PID (portify kill --pid 12841)",
            json,
        );
    }

    let mode = if args.force {
        KillMode::Force
    } else {
        KillMode::Graceful
    };
    let proto = args.proto.map(Into::into);

    // Show what is about to die before doing it. Skipped for --json, where the
    // caller is a script and the prompt would be a hang.
    if !args.yes && !json {
        let (plan, collateral) = build_kill_plan(&args, proto);
        if plan.is_empty() && args.pid.is_empty() {
            let ports: Vec<String> = args.ports.iter().map(|p| p.to_string()).collect();
            render::print_empty(&format!("process on port {}", ports.join(", ")));
            return ExitCode::NotFound;
        }
        if !plan.is_empty() {
            render::print_kill_plan(&plan, &collateral);
        }
        if !args.pid.is_empty() {
            anstream::println!("  PID {:?}", args.pid);
        }
        match confirm() {
            Confirmation::Yes => {}
            Confirmation::No => {
                anstream::println!("Cancelled.");
                return ExitCode::Success;
            }
            Confirmation::NotATerminal => return fail_with(
                ExitCode::InvalidInput,
                "refusing to kill without confirmation while stdin is not a terminal; pass --yes",
                json,
            ),
        }
    }

    let mut outcomes: Vec<KillOutcome> = Vec::new();
    for port in &args.ports {
        outcomes.extend(kill_port(*port, proto, mode));
    }
    for pid in &args.pid {
        outcomes.push(kill_pid(*pid, mode));
    }

    if json {
        let payload = serde_json::json!({
            "portify": portify_core::VERSION,
            "outcomes": outcomes,
        });
        match serde_json::to_string_pretty(&payload) {
            Ok(text) => println!("{text}"),
            Err(err) => return fail(&format!("could not serialise output: {err}"), false),
        }
    } else {
        for outcome in &outcomes {
            render::print_kill_outcome(outcome);
        }
    }

    worst_exit_code(&outcomes)
}

/// Look up what currently holds the requested ports, for the confirmation
/// prompt. Best-effort: a failed scan just means a less informative prompt.
#[allow(clippy::type_complexity)]
fn build_kill_plan(
    args: &KillArgs,
    proto: Option<portify_core::Protocol>,
) -> (Vec<(u16, PortGroup)>, Vec<(u16, Vec<u16>)>) {
    if args.ports.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let Ok(entries) = scan(&ScanOptions {
        include_ephemeral_udp: true,
        ..ScanOptions::default()
    }) else {
        return (Vec::new(), Vec::new());
    };
    let filters = Filters {
        ports: args.ports.clone(),
        text: None,
        proto,
    };
    let groups = group_by_port(&filters.apply(entries.clone()));

    // Collateral is computed against the unfiltered scan: the whole point is
    // the ports the user did *not* ask about.
    let collateral = groups
        .iter()
        .map(|group| {
            let others = group
                .primary_pid()
                .map(|pid| other_ports_held_by(&entries, pid, group.port))
                .unwrap_or_default();
            (group.port, others)
        })
        .collect();

    let plan = groups
        .into_iter()
        .map(|group| (group.port, group))
        .collect();

    (plan, collateral)
}

fn worst_exit_code(outcomes: &[KillOutcome]) -> ExitCode {
    let mut code = ExitCode::Success;
    for outcome in outcomes {
        let candidate = match outcome.status {
            KillStatus::Killed | KillStatus::Escalated => ExitCode::Success,
            KillStatus::NotFound => ExitCode::NotFound,
            KillStatus::PermissionDenied => ExitCode::PermissionDenied,
            // Refused is a rejected request, not a malfunction.
            KillStatus::Refused => ExitCode::InvalidInput,
            KillStatus::Survived => ExitCode::Internal,
        };
        // Ordering is by severity, which matches the numeric order.
        if (candidate as i32) > (code as i32) {
            code = candidate;
        }
    }
    code
}

enum Confirmation {
    Yes,
    No,
    NotATerminal,
}

fn confirm() -> Confirmation {
    let stdin = std::io::stdin();
    if !stdin.is_terminal() {
        return Confirmation::NotATerminal;
    }

    anstream::print!("Continue? [y/N] ");
    let _ = std::io::stdout().flush();

    let mut answer = String::new();
    if stdin.read_line(&mut answer).is_err() {
        return Confirmation::No;
    }
    match answer.trim().to_lowercase().as_str() {
        "y" | "yes" => Confirmation::Yes,
        _ => Confirmation::No,
    }
}

fn watch(args: WatchArgs) -> ExitCode {
    let filters = Filters {
        ports: args.ports.clone(),
        text: args.filter.clone(),
        proto: None,
    };
    let options = scan_options(args.all, !args.ports.is_empty());
    let elevated = is_elevated();

    loop {
        let started = Instant::now();
        let entries = match scan(&options) {
            Ok(entries) => filters.apply(entries),
            Err(err) => return fail(&err.to_string(), false),
        };
        let elapsed = started.elapsed();

        render::clear_screen();
        render::print_watch_header(args.interval, elevated);

        if entries.is_empty() {
            render::print_empty(&filters.describe());
        } else {
            let hidden = entries.iter().filter(|e| e.owner_hidden).count();
            let groups = group_by_port(&entries);
            render::print_groups(&groups, false);
            render::print_summary(groups.len(), hidden, elapsed.as_millis(), elevated);
        }

        let _ = std::io::stdout().flush();
        std::thread::sleep(Duration::from_secs(args.interval));
    }
}

fn info(json: bool) -> ExitCode {
    let summary = system_summary();
    if json {
        match serde_json::to_string_pretty(&summary) {
            Ok(text) => println!("{text}"),
            Err(err) => return fail(&format!("could not serialise output: {err}"), false),
        }
    } else {
        render::print_info(&summary);
    }
    ExitCode::Success
}

/// Report an internal failure (something broke that should not have).
fn fail(message: &str, json: bool) -> ExitCode {
    fail_with(ExitCode::Internal, message, json)
}

/// Report a failure the caller can act on, with an explicit exit code so
/// scripts can distinguish "you asked wrong" from "I broke".
fn fail_with(code: ExitCode, message: &str, json: bool) -> ExitCode {
    if json {
        let payload = serde_json::json!({ "error": message, "exit_code": code as i32 });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else {
        anstream::eprintln!("portify: {message}");
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_options_track_the_all_flag() {
        assert!(scan_options(false, false).only_serving);
        assert!(!scan_options(true, false).only_serving);
    }

    #[test]
    fn naming_a_port_reveals_sockets_the_browse_view_hides() {
        assert!(!scan_options(false, false).include_ephemeral_udp);
        assert!(scan_options(false, true).include_ephemeral_udp);
        assert!(scan_options(true, false).include_ephemeral_udp);
    }

    fn outcome(status: KillStatus) -> KillOutcome {
        KillOutcome {
            pid: 1,
            process_name: "x".into(),
            port: None,
            status,
            detail: String::new(),
            collateral_ports: Vec::new(),
        }
    }

    #[test]
    fn exit_code_is_success_when_everything_died() {
        let outcomes = vec![outcome(KillStatus::Killed), outcome(KillStatus::Escalated)];
        assert_eq!(worst_exit_code(&outcomes), ExitCode::Success);
    }

    #[test]
    fn exit_code_reports_the_worst_outcome() {
        let outcomes = vec![
            outcome(KillStatus::Killed),
            outcome(KillStatus::NotFound),
            outcome(KillStatus::PermissionDenied),
        ];
        assert_eq!(worst_exit_code(&outcomes), ExitCode::PermissionDenied);
    }

    #[test]
    fn not_found_alone_is_exit_two() {
        assert_eq!(
            worst_exit_code(&[outcome(KillStatus::NotFound)]),
            ExitCode::NotFound
        );
    }

    #[test]
    fn empty_outcomes_are_success() {
        assert_eq!(worst_exit_code(&[]), ExitCode::Success);
    }

    #[test]
    fn a_refused_kill_is_bad_input_not_a_malfunction() {
        assert_eq!(
            worst_exit_code(&[outcome(KillStatus::Refused)]),
            ExitCode::InvalidInput
        );
    }

    #[test]
    fn a_survivor_is_an_internal_failure() {
        assert_eq!(
            worst_exit_code(&[outcome(KillStatus::Survived)]),
            ExitCode::Internal
        );
    }
}
