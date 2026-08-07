//! Terminal rendering.
//!
//! Deliberately hand-rolled and flat: aligned columns, one line per port, no
//! box drawing. The output has to stay readable when it is piped, grepped or
//! pasted into an issue, so styling is applied on top of a fixed-width layout
//! and `anstream` removes it when the destination is not a terminal.

use owo_colors::OwoColorize;
use portify_core::scan::format_bytes;
use portify_core::{KillOutcome, KillStatus, PortEntry, PortGroup, SystemSummary};

/// Column alignment.
#[derive(Clone, Copy, PartialEq)]
enum Align {
    Left,
    Right,
}

/// How a column should be styled once it has been padded.
#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Strong,
    Normal,
    Muted,
    Good,
    Warn,
    Bad,
}

fn tone(text: &str, tone: Tone) -> String {
    match tone {
        Tone::Strong => text.bold().to_string(),
        Tone::Normal => text.to_string(),
        Tone::Muted => text.dimmed().to_string(),
        Tone::Good => text.green().to_string(),
        Tone::Warn => text.yellow().to_string(),
        Tone::Bad => text.red().to_string(),
    }
}

/// Placeholder written into a cell that has no value.
const EMPTY: &str = "—";

struct Column {
    header: &'static str,
    align: Align,
    tone: Tone,
    /// Columns marked flexible are the first to be truncated when the terminal
    /// is narrow.
    flexible: bool,
    /// Drop the column entirely when it carries no information for any row.
    /// Keeps the default view from filling up with columns of dashes.
    elidable: bool,
}

/// Shorthand so the column definitions below stay readable.
fn column(
    header: &'static str,
    align: Align,
    tone: Tone,
    flexible: bool,
    elidable: bool,
) -> Column {
    Column {
        header,
        align,
        tone,
        flexible,
        elidable,
    }
}

struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
}

impl Table {
    fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
        }
    }

    fn push(&mut self, row: Vec<String>) {
        debug_assert_eq!(row.len(), self.columns.len(), "row/column count mismatch");
        self.rows.push(row);
    }

    /// Indices of the columns worth printing.
    fn visible_columns(&self) -> Vec<usize> {
        (0..self.columns.len())
            .filter(|idx| {
                !self.columns[*idx].elidable
                    || self.rows.iter().any(|row| is_informative(&row[*idx]))
            })
            .collect()
    }

    fn print(&self) {
        if self.rows.is_empty() {
            return;
        }

        let visible = self.visible_columns();
        let columns: Vec<&Column> = visible.iter().map(|idx| &self.columns[*idx]).collect();
        let rows: Vec<Vec<&String>> = self
            .rows
            .iter()
            .map(|row| visible.iter().map(|idx| &row[*idx]).collect())
            .collect();

        let mut widths: Vec<usize> = columns
            .iter()
            .map(|column| display_width(column.header))
            .collect();
        for row in &rows {
            for (idx, cell) in row.iter().enumerate() {
                widths[idx] = widths[idx].max(display_width(cell));
            }
        }

        // Shrink flexible columns if the whole table would not fit.
        let gap = 2usize;
        let terminal = terminal_width();
        let total: usize = widths.iter().sum::<usize>() + gap * (widths.len() - 1);
        if total > terminal {
            let mut overflow = total - terminal;
            for (idx, column) in columns.iter().enumerate() {
                if overflow == 0 || !column.flexible {
                    continue;
                }
                let floor = display_width(column.header).max(8);
                let can_give = widths[idx].saturating_sub(floor);
                let give = can_give.min(overflow);
                widths[idx] -= give;
                overflow -= give;
            }
        }

        let header: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(idx, column)| pad(&column.header.to_uppercase(), widths[idx], column.align))
            .collect();
        anstream::println!("{}", tone(&header.join("  "), Tone::Muted));

        for row in &rows {
            let line: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(idx, cell)| {
                    let text = truncate(cell, widths[idx]);
                    let padded = pad(&text, widths[idx], columns[idx].align);
                    tone(&padded, columns[idx].tone)
                })
                .collect();
            anstream::println!("{}", line.join("  ").trim_end());
        }
    }
}

/// Collapse a value onto one line.
///
/// Command lines routinely contain newlines and tabs (`python -c "…"`, shell
/// wrappers). Printing them raw destroys the column layout, so every
/// control character becomes a single space before it reaches the terminal.
/// The unmodified value is still what `--json` emits.
fn single_line(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_was_space = false;
    for ch in text.chars() {
        let is_blank = ch.is_control() || ch.is_whitespace();
        if is_blank {
            if !last_was_space && !out.is_empty() {
                out.push(' ');
            }
            last_was_space = true;
        } else {
            out.push(ch);
            last_was_space = false;
        }
    }
    out.trim_end().to_string()
}

/// A cell carries information unless it is blank or the empty placeholder.
fn is_informative(cell: &str) -> bool {
    let trimmed = cell.trim();
    !trimmed.is_empty() && trimmed != EMPTY
}

fn display_width(text: &str) -> usize {
    text.chars().count()
}

fn pad(text: &str, width: usize, align: Align) -> String {
    let len = display_width(text);
    if len >= width {
        return text.to_string();
    }
    let padding = " ".repeat(width - len);
    match align {
        Align::Left => format!("{text}{padding}"),
        Align::Right => format!("{padding}{text}"),
    }
}

fn truncate(text: &str, width: usize) -> String {
    if display_width(text) <= width {
        return text.to_string();
    }
    if width <= 1 {
        return "…".to_string();
    }
    let kept: String = text.chars().take(width - 1).collect();
    format!("{kept}…")
}

/// Usable width of the terminal.
///
/// Asks the OS first, since `COLUMNS` is set but *not exported* by most shells
/// and is therefore absent from a child process's environment. An explicit
/// `COLUMNS` still wins when present, which keeps the output scriptable in
/// tests. Falls back to a width that does not wrap in a default window.
fn terminal_width() -> usize {
    if let Some(width) = std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|width| *width >= MIN_WIDTH)
    {
        return width;
    }

    terminal_size::terminal_size()
        .map(|(terminal_size::Width(width), _)| width as usize)
        .filter(|width| *width >= MIN_WIDTH)
        .unwrap_or(DEFAULT_WIDTH)
}

/// Below this the table stops being a table, so narrower reports are ignored.
const MIN_WIDTH: usize = 40;
/// Used when the output is not a terminal at all (pipes, CI logs).
const DEFAULT_WIDTH: usize = 120;

/// One row per port — the default view.
pub fn print_groups(groups: &[PortGroup], wide: bool) {
    let mut columns = vec![
        column("port", Align::Right, Tone::Strong, false, false),
        column("proto", Align::Left, Tone::Muted, false, false),
        column("process", Align::Left, Tone::Normal, true, false),
        column("pid", Align::Right, Tone::Muted, false, true),
        column("memory", Align::Right, Tone::Muted, false, true),
        column("conns", Align::Right, Tone::Muted, false, true),
        column("service", Align::Left, Tone::Muted, true, true),
    ];
    if wide {
        columns.push(column("command", Align::Left, Tone::Muted, true, true));
    }

    let mut table = Table::new(columns);
    for group in groups {
        let process = group.processes.first();
        let mut row = vec![
            group.port.to_string(),
            group.protocol.as_str().to_string(),
            group.process_label(),
            group
                .primary_pid()
                .map(|pid| pid.to_string())
                .unwrap_or_else(|| EMPTY.to_string()),
            process
                .and_then(|p| p.memory_bytes)
                .map(format_bytes)
                .unwrap_or_else(|| EMPTY.to_string()),
            if group.connections > 0 {
                group.connections.to_string()
            } else {
                EMPTY.to_string()
            },
            // The port hint is a convention; the process description is an
            // observation. Show the convention when there is one, and fall back
            // to what the process actually is — which is what turns twenty rows
            // of "svchost.exe" into something readable.
            group
                .service
                .or_else(|| process.and_then(|p| p.description))
                .unwrap_or("")
                .to_string(),
        ];
        if wide {
            row.push(
                process
                    .and_then(|p| p.command.as_deref().or(p.exe.as_deref()))
                    .map(single_line)
                    .unwrap_or_default(),
            );
        }
        table.push(row);
    }
    table.print();
}

/// One row per socket — `--raw`.
pub fn print_entries(entries: &[PortEntry], wide: bool) {
    let mut columns = vec![
        column("port", Align::Right, Tone::Strong, false, false),
        column("proto", Align::Left, Tone::Muted, false, false),
        column("state", Align::Left, Tone::Normal, false, false),
        column("process", Align::Left, Tone::Normal, true, false),
        column("pid", Align::Right, Tone::Muted, false, true),
        column("local address", Align::Left, Tone::Muted, true, false),
    ];
    if wide {
        columns.push(column("remote", Align::Left, Tone::Muted, true, true));
    }

    let mut table = Table::new(columns);
    for entry in entries {
        let mut row = vec![
            entry.port.to_string(),
            entry.protocol.as_str().to_string(),
            entry.state.as_str().to_string(),
            entry.process_name().to_string(),
            entry
                .pid
                .map(|pid| pid.to_string())
                .unwrap_or_else(|| EMPTY.to_string()),
            entry.local_address.clone(),
        ];
        if wide {
            row.push(entry.remote_address.clone().unwrap_or_default());
        }
        table.push(row);
    }
    table.print();
}

/// Footer with the honest numbers: how much was found and how long it took.
pub fn print_summary(port_count: usize, hidden: usize, elapsed_ms: u128, elevated: bool) {
    let mut parts = vec![format!(
        "{} {}",
        port_count,
        if port_count == 1 { "port" } else { "ports" }
    )];
    parts.push(format!("{elapsed_ms} ms"));

    anstream::println!();
    anstream::println!("{}", tone(&parts.join("  ·  "), Tone::Muted));

    if hidden > 0 && !elevated {
        anstream::println!(
            "{}",
            tone(
                &format!(
                    "{hidden} socket(s) owned by another user — {} to see them",
                    portify_core::system::elevation_hint()
                ),
                Tone::Warn
            )
        );
    }
}

pub fn print_empty(scope: &str) {
    anstream::println!("{}", tone(&format!("No {scope} found."), Tone::Muted));
}

/// What is about to be killed, shown before the confirmation prompt.
///
/// `collateral` maps each port to the other ports its owner also holds. A
/// process fronting dozens of ports (a WSL relay, a reverse proxy) turns
/// "free one port" into "drop them all", and that has to be visible *before*
/// the prompt, not in the result.
pub fn print_kill_plan(targets: &[(u16, PortGroup)], collateral: &[(u16, Vec<u16>)]) {
    anstream::println!("{}", tone("About to terminate:", Tone::Strong));
    for (port, group) in targets {
        let process = group.process_label();
        let pid = group
            .primary_pid()
            .map(|pid| pid.to_string())
            .unwrap_or_else(|| "?".into());
        anstream::println!(
            "  {} {}  {}  {}",
            tone(&format!("port {port}"), Tone::Strong),
            tone(group.protocol.as_str(), Tone::Muted),
            process,
            tone(&format!("(PID {pid})"), Tone::Muted)
        );
        if let Some(command) = group.processes.first().and_then(|p| p.command.as_deref()) {
            anstream::println!(
                "      {}",
                tone(&truncate(&single_line(command), 96), Tone::Muted)
            );
        }

        if let Some((_, others)) = collateral.iter().find(|(p, _)| p == port) {
            if !others.is_empty() {
                anstream::println!("      {}", tone(&collateral_line(others), Tone::Warn));
            }
        }
    }
}

/// Human-readable summary of the ports that go down as collateral.
fn collateral_line(ports: &[u16]) -> String {
    let shown: Vec<String> = ports.iter().take(6).map(|p| p.to_string()).collect();
    let listed = shown.join(", ");
    if ports.len() > shown.len() {
        format!(
            "⚠ this process also holds {} other ports ({listed}, …) — killing it drops all of them",
            ports.len()
        )
    } else if ports.len() == 1 {
        format!("⚠ this process also holds port {listed}, which goes down with it")
    } else {
        format!("⚠ this process also holds ports {listed} — all of them go down with it")
    }
}

pub fn print_kill_outcome(outcome: &KillOutcome) {
    let (mark, mark_tone) = match outcome.status {
        KillStatus::Killed | KillStatus::Escalated => ("✔", Tone::Good),
        KillStatus::NotFound => ("·", Tone::Muted),
        KillStatus::PermissionDenied => ("✖", Tone::Bad),
        KillStatus::Survived => ("✖", Tone::Bad),
        KillStatus::Refused => ("!", Tone::Warn),
    };
    anstream::println!("{} {}", tone(mark, mark_tone), outcome.detail);

    // Reported after the fact too, for `--yes` runs that skipped the plan.
    if outcome.status.is_success() {
        if let Some(warning) = outcome.collateral_warning() {
            anstream::println!("  {}", tone(&warning, Tone::Warn));
        }
    }
}

pub fn print_info(summary: &SystemSummary) {
    let rows = [
        ("portify", summary.portify_version.clone()),
        (
            "os",
            format!("{} {}", summary.os, summary.os_version)
                .trim()
                .to_string(),
        ),
        ("kernel", summary.kernel_version.clone()),
        ("arch", summary.arch.clone()),
        ("host", summary.hostname.clone()),
        (
            "privileges",
            if summary.elevated {
                "elevated — every process is visible and killable".to_string()
            } else {
                format!(
                    "standard user — {} to reach other users' processes",
                    portify_core::system::elevation_hint()
                )
            },
        ),
    ];

    let label_width = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, value) in rows {
        anstream::println!(
            "{}  {}",
            tone(&pad(key, label_width, Align::Left), Tone::Muted),
            value
        );
    }
}

/// Clear the screen for the live view. `anstream` converts these sequences for
/// legacy Windows consoles.
pub fn clear_screen() {
    anstream::print!("\x1b[2J\x1b[H");
}

pub fn print_watch_header(interval: u64, elevated: bool) {
    let mut header = format!("portify watch  ·  every {interval}s  ·  Ctrl+C to stop");
    if !elevated {
        header.push_str("  ·  not elevated");
    }
    anstream::println!("{}", tone(&header, Tone::Muted));
    anstream::println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_both_directions() {
        assert_eq!(pad("3000", 6, Align::Right), "  3000");
        assert_eq!(pad("node", 6, Align::Left), "node  ");
        assert_eq!(pad("exactly", 7, Align::Left), "exactly");
    }

    #[test]
    fn never_pads_below_content_width() {
        assert_eq!(pad("verylong", 3, Align::Left), "verylong");
    }

    #[test]
    fn truncation_adds_an_ellipsis_and_respects_the_budget() {
        assert_eq!(truncate("node", 10), "node");
        assert_eq!(truncate("chromium-browser", 8), "chromiu…");
        assert_eq!(display_width(&truncate("chromium-browser", 8)), 8);
        assert_eq!(truncate("abc", 1), "…");
    }

    #[test]
    fn truncation_is_char_safe_for_multibyte_names() {
        // Byte slicing here would panic; char slicing must not.
        let truncated = truncate("aplicación-de-red", 6);
        assert_eq!(display_width(&truncated), 6);
    }

    #[test]
    fn columns_with_nothing_in_them_are_dropped() {
        let mut table = Table::new(vec![
            column("port", Align::Right, Tone::Strong, false, false),
            column("memory", Align::Right, Tone::Muted, false, true),
            column("service", Align::Left, Tone::Muted, true, true),
        ]);
        table.push(vec!["3000".into(), EMPTY.into(), "Dev server".into()]);
        table.push(vec!["5432".into(), EMPTY.into(), "".into()]);

        // memory is all dashes so it goes; service has one real value so it stays.
        assert_eq!(table.visible_columns(), vec![0, 2]);
    }

    #[test]
    fn non_elidable_columns_survive_even_when_empty() {
        let mut table = Table::new(vec![
            column("port", Align::Right, Tone::Strong, false, false),
            column("process", Align::Left, Tone::Normal, true, false),
        ]);
        table.push(vec!["3000".into(), EMPTY.into()]);
        assert_eq!(table.visible_columns(), vec![0, 1]);
    }

    #[test]
    fn commands_are_flattened_onto_one_line() {
        let command = "python3 -c import socket\ns.bind(('0.0.0.0', 3000))\ts.listen()";
        let flattened = single_line(command);
        assert!(!flattened.contains('\n'));
        assert!(!flattened.contains('\t'));
        assert!(!flattened.contains("  "), "runs of blanks collapse to one");
        assert!(flattened.starts_with("python3 -c import socket s.bind"));
    }

    #[test]
    fn flattening_leaves_ordinary_commands_alone() {
        assert_eq!(single_line("node server.js"), "node server.js");
        assert_eq!(single_line(""), "");
        assert_eq!(single_line("  padded  "), "padded");
    }

    #[test]
    fn placeholder_is_not_information() {
        assert!(!is_informative(EMPTY));
        assert!(!is_informative("  "));
        assert!(is_informative("node"));
    }

    #[test]
    fn terminal_width_falls_back_when_columns_is_nonsense() {
        // Only asserts the guard rails; the real value depends on the env.
        assert!(terminal_width() >= 40);
    }
}
