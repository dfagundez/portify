//! Host facts that change what Portify can do — chiefly whether we are
//! elevated, since that decides which processes we are allowed to see and kill.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SystemSummary {
    pub os: String,
    pub os_version: String,
    pub kernel_version: String,
    pub arch: String,
    pub hostname: String,
    /// True when running as Administrator (Windows) or root (Unix).
    pub elevated: bool,
    pub portify_version: String,
}

/// True when the current process can see and signal other users' processes.
///
/// On Windows this inspects the process token's elevation flag, which is what
/// "Run as Administrator" actually sets — checking the username is not enough,
/// because an admin account still runs unelevated by default under UAC.
#[cfg(windows)]
pub fn is_elevated() -> bool {
    use std::mem::size_of;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::TOKEN_QUERY;
    use windows_sys::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut returned: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        );
        CloseHandle(token);

        ok != 0 && elevation.TokenIsElevated != 0
    }
}

#[cfg(unix)]
pub fn is_elevated() -> bool {
    // Safe: geteuid takes no arguments and cannot fail.
    unsafe { libc::geteuid() == 0 }
}

#[cfg(not(any(unix, windows)))]
pub fn is_elevated() -> bool {
    false
}

/// The command a user should run to get elevated access on this platform.
pub fn elevation_hint() -> &'static str {
    if cfg!(windows) {
        "re-open your terminal with \"Run as Administrator\""
    } else {
        "re-run with sudo"
    }
}

pub fn system_summary() -> SystemSummary {
    SystemSummary {
        os: sysinfo::System::name().unwrap_or_else(|| std::env::consts::OS.to_string()),
        os_version: sysinfo::System::os_version().unwrap_or_else(|| "unknown".to_string()),
        kernel_version: sysinfo::System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
        arch: std::env::consts::ARCH.to_string(),
        hostname: sysinfo::System::host_name().unwrap_or_else(|| "unknown".to_string()),
        elevated: is_elevated(),
        portify_version: crate::VERSION.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_is_populated() {
        let summary = system_summary();
        assert!(!summary.arch.is_empty());
        assert!(!summary.portify_version.is_empty());
    }

    #[test]
    fn elevation_check_does_not_panic() {
        let _ = is_elevated();
        assert!(!elevation_hint().is_empty());
    }
}
