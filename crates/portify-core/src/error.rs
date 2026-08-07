use std::fmt;

/// Errors that can come out of a scan.
///
/// Killing a process never returns an `Err`: a failed kill is a *result* the
/// user needs to see (permission denied, already gone, …), not an exception, so
/// it is modelled as [`crate::KillStatus`] instead.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The OS socket table could not be read at all.
    #[error("could not read the system socket table: {0}")]
    SocketTable(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Process-exit contract, shared by the CLI so scripts can branch on it.
///
/// Mirrors the convention used by other port tools: 0 clean, 2 nothing found,
/// 3 blocked by permissions, 5 something broke internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    NotFound = 2,
    PermissionDenied = 3,
    InvalidInput = 4,
    Internal = 5,
}

impl fmt::Display for ExitCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as i32)
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code as i32
    }
}
