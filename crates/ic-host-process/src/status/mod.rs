//! Explicit shell-style projection of observed child status; callers own CLI fallback policy.

use std::process::ExitStatus;

/// Return the numeric exit code, or `128 + signal` for Unix signal termination.
///
/// An unclassified status returns `None`. Callers choose its fallback and the
/// conversion to their CLI's supported exit-code range. This does not wait,
/// signal, reap or otherwise manage a process, and does not imply success.
#[must_use]
pub fn shell_status_code(status: ExitStatus) -> Option<i32> {
    if let Some(code) = status.code() {
        return Some(code);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt as _;
        status.signal().map(|signal| 128 + signal)
    }
    #[cfg(not(unix))]
    {
        None
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::{os::unix::process::ExitStatusExt as _, process::Command};

    #[test]
    fn observed_exits_and_signals_keep_their_numeric_identity() {
        for code in [0, 37, 255] {
            let status = Command::new("/bin/sh")
                .args(["-c", &format!("exit {code}")])
                .status()
                .unwrap();
            assert_eq!(shell_status_code(status), Some(code));
        }
        let status = Command::new("/bin/sh")
            .args(["-c", "kill -TERM $$"])
            .status()
            .unwrap();
        assert_eq!(shell_status_code(status), Some(143));
    }

    #[test]
    fn unclassified_status_leaves_the_fallback_to_each_caller() {
        let stopped = ExitStatus::from_raw((19 << 8) | 0x7f);
        assert_eq!(shell_status_code(stopped), None);
        assert_eq!(shell_status_code(stopped).unwrap_or(1), 1);
        assert_eq!(shell_status_code(stopped).unwrap_or(129), 129);
    }
}
