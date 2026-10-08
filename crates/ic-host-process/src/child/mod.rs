//! Explicit child ownership and cleanup, without application lifecycle policy.
//!
//! Callers own executable admission, IO, readiness, cancellation and deadlines.
//! Group cleanup signals members of a newly created group; it cannot contain
//! processes that escape that group or prove completion of external effects.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(test)]
mod tests;

use rustix::{
    io::Errno,
    process::{Pid, Signal, WaitId, WaitIdOptions, kill_process_group, waitid},
};
use std::{
    fmt, io,
    os::unix::process::CommandExt,
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus},
};

/// One exclusively owned child, normally spawned as a new process-group leader.
///
/// [`Self::spawn`] preserves the command's IO, environment and other settings,
/// replacing its process-group selection with a new group. It performs no
/// executable admission. The child must not change groups, and callers must
/// not independently reap it (including through a global SIGCHLD handler).
///
/// Polling an exited leader signals remaining group members before reaping it.
/// Drop makes a best-effort kill/reap attempt, including during unwinding. Use
/// [`Self::terminate`] to observe cleanup failures. Cleanup is synchronous and
/// has no wall-clock bound; successful signalling is not proof that descendants
/// have exited or completed external effects. Only the direct child is reaped.
/// Group signalling can succeed for only some members when credentials differ.
pub struct OwnedChild {
    child: Child,
    group: bool,
    status: Option<ExitStatus>,
    owned: bool,
}

/// Failures observed during one explicit termination attempt.
///
/// Keep this separately from the caller's original cancellation/operation error.
/// If group signalling fails, direct-child kill and reaping are still attempted.
#[derive(Debug)]
pub struct CleanupError {
    /// Status retained if the direct child was reaped despite another failure.
    pub status: Option<ExitStatus>,
    /// Failure signalling the owned process group.
    pub group_error: Option<io::Error>,
    /// Failure killing the direct child (including fallback after group failure).
    pub kill_error: Option<io::Error>,
    /// Failure reaping the direct child.
    pub wait_error: Option<io::Error>,
}

impl fmt::Display for CleanupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("child cleanup failed")?;
        for (operation, error) in [
            ("group signal", &self.group_error),
            ("child kill", &self.kill_error),
            ("child wait", &self.wait_error),
        ] {
            if let Some(error) = error {
                write!(f, "; {operation}: {error}")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for CleanupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.group_error
            .as_ref()
            .or(self.kill_error.as_ref())
            .or(self.wait_error.as_ref())
            .map(|error| error as &dyn std::error::Error)
    }
}

impl OwnedChild {
    /// Spawn once in a new owned process group, preserving caller-configured IO.
    ///
    /// # Errors
    /// Returns the native spawn/setup failure. No retries are performed.
    pub fn spawn(command: &mut Command) -> io::Result<Self> {
        command.process_group(0);
        Self::spawn_inner(command, true)
    }

    pub(crate) fn spawn_direct(command: &mut Command) -> io::Result<Self> {
        Self::spawn_inner(command, false)
    }

    fn spawn_inner(command: &mut Command, group: bool) -> io::Result<Self> {
        command.spawn().map(|child| Self {
            child,
            group,
            status: None,
            owned: true,
        })
    }

    /// Direct-child PID, for observation only; it can be reused after reaping.
    #[must_use]
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    /// Take caller-configured piped stdin. Close it before waiting for EOF-driven children.
    pub const fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.child.stdin.take()
    }

    /// Take caller-configured piped stdout; the caller owns draining and bounds.
    pub const fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout.take()
    }

    /// Take caller-configured piped stderr; the caller owns draining and bounds.
    pub const fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.child.stderr.take()
    }

    /// Inspect exit without blocking on a running child; clean its group before reaping.
    ///
    /// Repeated successful calls return the cached status without signalling again.
    /// # Errors
    /// Returns native inspection, group-signal or reap errors. The leader remains
    /// reserved on a group-signal failure, so explicit cleanup can still be attempted.
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        if let Some(status) = self.status {
            return Ok(Some(status));
        }
        if !self.owned {
            return Err(Errno::CHILD.into());
        }
        if self.group {
            if !self.observe_exit(true)? {
                return Ok(None);
            }
            self.signal_group()?;
            self.reap().map(Some)
        } else {
            let result = retry_interrupted(|| self.child.try_wait());
            if let Ok(Some(status)) = result {
                self.status = Some(status);
                self.owned = false;
            }
            self.check_wait_ownership(&result);
            result
        }
    }

    /// Wait for natural leader exit, then clean its group and reap the leader.
    ///
    /// Close/drain caller-owned pipes as needed before waiting. No deadline or
    /// cancellation policy is installed; callers may use polling instead.
    /// # Errors
    /// Returns native inspection, group-signal or reap errors.
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        if let Some(status) = self.status {
            return Ok(status);
        }
        if self.group {
            self.observe_exit(false)?;
            self.signal_group()?;
        }
        self.reap()
    }

    /// Kill the owned group (or internal direct child), then reap the leader.
    ///
    /// Repeated calls after reaping return the cached status and never signal a
    /// reused PID. A prior group failure still matters even if reaping succeeded;
    /// later calls cannot recover group ownership and do not erase that evidence.
    /// # Errors
    /// Retains each failed cleanup step separately. Group failure triggers a
    /// direct-child kill fallback. Drop cannot report errors; call this explicitly
    /// when cleanup evidence matters.
    pub fn terminate(&mut self) -> Result<ExitStatus, CleanupError> {
        if let Some(status) = self.status {
            return Ok(status);
        }
        let group_error = if self.group {
            self.signal_group().err()
        } else {
            None
        };
        let kill_error = if self.owned && (!self.group || group_error.is_some()) {
            retry_interrupted(|| self.child.kill()).err()
        } else {
            None
        };
        let waited = self.reap();
        match waited {
            Ok(status) if group_error.is_none() && kill_error.is_none() => Ok(status),
            other => Err(CleanupError {
                status: self.status,
                group_error,
                kill_error,
                wait_error: other.err(),
            }),
        }
    }

    fn pid(&self) -> io::Result<Pid> {
        if !self.owned {
            return Err(Errno::CHILD.into());
        }
        Pid::from_raw(i32::try_from(self.id()).map_err(io::Error::other)?)
            .ok_or_else(|| io::Error::other("child PID is zero"))
    }

    fn observe_exit(&mut self, nonblocking: bool) -> io::Result<bool> {
        let pid = self.pid()?;
        // NOWAIT reserves the leader PID until the final group signal, avoiding
        // signalling an unrelated group if the leader exited before cleanup.
        let mut options = WaitIdOptions::EXITED | WaitIdOptions::NOWAIT;
        if nonblocking {
            options |= WaitIdOptions::NOHANG;
        }
        let result = retry_interrupted(|| waitid(WaitId::Pid(pid), options).map_err(Into::into));
        self.check_wait_ownership(&result);
        result.map(|status| status.is_some())
    }

    #[cfg_attr(
        not(target_os = "macos"),
        allow(
            clippy::needless_pass_by_ref_mut,
            reason = "Darwin inspects and may invalidate child ownership"
        )
    )]
    fn signal_group(&mut self) -> io::Result<()> {
        let pid = self.pid()?;
        match retry_interrupted(|| kill_process_group(pid, Signal::KILL).map_err(Into::into)) {
            Ok(()) => Ok(()),
            Err(error) if error.raw_os_error() == Some(Errno::SRCH.raw_os_error()) => Ok(()),
            #[cfg(target_os = "macos")]
            Err(error)
                if error.raw_os_error() == Some(Errno::PERM.raw_os_error())
                    && self.observe_exit(true)?
                    && macos::sole_group_member(pid) =>
            {
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn reap(&mut self) -> io::Result<ExitStatus> {
        if !self.owned {
            return Err(Errno::CHILD.into());
        }
        let result = retry_interrupted(|| self.child.wait());
        if let Ok(status) = result {
            self.status = Some(status);
            self.owned = false;
        }
        self.check_wait_ownership(&result);
        result
    }

    fn check_wait_ownership<T>(&mut self, result: &io::Result<T>) {
        if result
            .as_ref()
            .is_err_and(|error| error.raw_os_error() == Some(Errno::CHILD.raw_os_error()))
        {
            // An external reaper violates exclusive ownership; never signal a
            // potentially reused PID after observing that ownership was lost.
            self.owned = false;
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.owned {
            let _ = self.terminate();
        }
    }
}

fn retry_interrupted<T>(mut operation: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    loop {
        match operation() {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            result => return result,
        }
    }
}
