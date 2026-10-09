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
    process::{Pid, Signal, WaitId, WaitIdOptions, WaitIdStatus, kill_process_group, waitid},
};
use std::{
    fmt, io,
    os::unix::process::{CommandExt, ExitStatusExt},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus},
    time::{Duration, Instant},
};

/// Caller-selected termination behavior for an owned process group.
#[derive(Clone, Copy, Debug)]
pub enum CleanupPolicy {
    /// Signal KILL immediately, then wait synchronously for the leader.
    KillAndWait,
    /// Signal TERM, reserve the leader through the grace period, then signal
    /// KILL and poll reaping for at most the selected duration.
    TermThenKill {
        /// Time allowed after successful TERM signalling, even if the leader exits.
        grace: Duration,
        /// Reap allowance after KILL; zero permits one nonblocking observation.
        reap_timeout: Duration,
    },
}

/// One exclusively owned child, normally spawned as a new process-group leader.
///
/// [`Self::spawn`] preserves the command's IO, environment and other settings,
/// replacing its process-group selection with a new group. It performs no
/// executable admission. The child must not change groups, and callers must
/// not independently reap it (including through a global SIGCHLD handler).
///
/// Ordinary waiting signals remaining group members before reaping the leader.
/// For a deliberate background handoff, [`Self::poll_exit`] observes without
/// releasing cleanup ownership, then [`Self::handoff`] reaps a successful leader
/// without signalling its group. The caller then owns the background lifetime.
/// Drop makes a best-effort kill/reap attempt, including during unwinding. Use
/// [`Self::terminate`] to observe cleanup failures. The default cleanup waits
/// synchronously; [`Self::spawn_with_cleanup`] can select bounded reaping.
/// Successful signalling is not proof that descendants
/// have exited or completed external effects. Only the direct child is reaped.
/// Group signalling can succeed for only some members when credentials differ.
/// [`Self::spawn_direct`] instead preserves the command's process-group selection
/// and owns only the direct child. Its wait, termination and Drop never signal
/// other group members; descendant lifetime remains with the caller.
pub struct OwnedChild {
    child: Child,
    group: bool,
    status: Option<ExitStatus>,
    owned: bool,
    cleanup: CleanupPolicy,
    reap_started: Option<Instant>,
}

/// Failures observed during one explicit termination attempt.
///
/// Keep this separately from the caller's original cancellation/operation error.
/// If group signalling fails, direct-child kill and reaping are still attempted.
#[derive(Debug)]
pub struct CleanupError {
    /// Status retained if the direct child was reaped despite another failure.
    pub status: Option<ExitStatus>,
    /// Failure signalling TERM to the owned process group.
    pub term_error: Option<io::Error>,
    /// Failure signalling KILL to the owned process group.
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
            ("group TERM", &self.term_error),
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
        self.term_error
            .as_ref()
            .or(self.group_error.as_ref())
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
        Self::spawn_with_cleanup(command, CleanupPolicy::KillAndWait)
    }

    /// Spawn a new owned group with caller-selected termination timing.
    ///
    /// The policy applies whenever termination is needed, including communication
    /// failures and Drop during unwinding. It does not change natural waiting or
    /// successful background handoff. No signal handler or reaper thread is added.
    ///
    /// ```no_run
    /// use ic_host_process::child::{CleanupPolicy, OwnedChild};
    /// use std::{process::Command, time::Duration};
    /// let mut child = OwnedChild::spawn_with_cleanup(
    ///     &mut Command::new("caller-selected-tool"),
    ///     CleanupPolicy::TermThenKill {
    ///         grace: Duration::from_secs(5),
    ///         reap_timeout: Duration::from_secs(5),
    ///     },
    /// )?;
    /// // Configure command IO before spawn; communicate/admit before handoff.
    /// child.terminate()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    /// # Errors
    /// Returns the native spawn/setup failure. No retries are performed.
    pub fn spawn_with_cleanup(command: &mut Command, cleanup: CleanupPolicy) -> io::Result<Self> {
        command.process_group(0);
        Self::spawn_inner(command, true, cleanup)
    }

    /// Spawn once owning only the direct child, without changing command settings.
    ///
    /// Preserves inherited or explicitly configured process-group selection and
    /// IO. This permits foreground callers to retain terminal-group membership;
    /// it does not create a session, transfer terminal control or forward signals.
    /// No executable admission, retry or signal handler is installed.
    ///
    /// Waiting reaps only this child. Termination and Drop kill only this child
    /// and wait synchronously using [`CleanupPolicy::KillAndWait`]. Descendants
    /// are never signalled, even if the command explicitly selects a new group.
    /// Their lifecycle and inherited pipe writers remain caller-owned; no-deadline
    /// communication may wait indefinitely for their EOF after the child exits.
    /// Use [`Self::spawn`] when ownership of a new group is required.
    ///
    /// # Errors
    /// Returns the native spawn/setup failure. No retries are performed.
    pub fn spawn_direct(command: &mut Command) -> io::Result<Self> {
        Self::spawn_inner(command, false, CleanupPolicy::KillAndWait)
    }

    pub(crate) const fn is_owned(&self) -> bool {
        self.owned && self.reap_started.is_none()
    }

    fn spawn_inner(command: &mut Command, group: bool, cleanup: CleanupPolicy) -> io::Result<Self> {
        command.spawn().map(|child| Self {
            child,
            group,
            status: None,
            owned: true,
            cleanup,
            reap_started: None,
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

    /// Observe leader exit without signalling or reaping it.
    ///
    /// An exited leader stays reserved with `WNOWAIT`, so cancellation, failed IO
    /// admission and unwinding still clean its group. Repeated observations do
    /// not release ownership. After a completed wait/termination/handoff, returns
    /// the cached status. This status alone does not establish a handoff.
    ///
    /// Call [`Self::handoff`] only after admitting a successful background start.
    /// Otherwise use [`Self::wait`] or [`Self::terminate`] to clean and reap.
    /// # Errors
    /// Returns native inspection errors; external reaping invalidates ownership.
    pub fn poll_exit(&mut self) -> io::Result<Option<ExitStatus>> {
        if let Some(status) = self.status {
            return Ok(Some(status));
        }
        self.observe_exit(true)?
            .map(|status| {
                // Unix wait status encoding used by Linux and Darwin. waitid's
                // siginfo status is an exit code/signal, not an encoded wait status.
                let raw = if let Some(code) = status.exit_status() {
                    code << 8
                } else if let Some(signal) = status.terminating_signal() {
                    signal | if status.dumped() { 0x80 } else { 0 }
                } else {
                    return Err(io::Error::other("waitid returned a non-exit observation"));
                };
                Ok(ExitStatus::from_raw(raw))
            })
            .transpose()
    }

    /// Reap an already successful leader without signalling its remaining group.
    ///
    /// This is the explicit transfer point for a background lifetime. The caller
    /// must first admit its IO/result and arrange application-owned readiness,
    /// cancellation and stop/recovery. Zero exit does not prove those obligations.
    /// No PID/group handle is transferred: it could be reused after reaping.
    /// Subsequent wait/termination/Drop never signal the handed-off group.
    ///
    /// Use [`Self::poll_exit`] while draining IO and checking cancellation. A
    /// running or unsuccessful leader is refused without releasing ownership;
    /// use ordinary wait/termination for failed startup, keeping its original
    /// failure separate from any cleanup error. Drop still attempts cleanup.
    /// # Errors
    /// Returns `InvalidInput` for a running, unsuccessful, terminating or already-reaped
    /// leader, or native inspection/reap errors. Reap failures retain cleanup
    /// ownership unless it was lost externally.
    pub fn handoff(&mut self) -> io::Result<ExitStatus> {
        if !self.is_owned() || !self.poll_exit()?.is_some_and(|status| status.success()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "handoff requires an owned, successfully exited leader",
            ));
        }
        self.reap()
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
            if self.observe_exit(true)?.is_none() {
                return Ok(None);
            }
            self.signal_group(Signal::KILL)?;
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
            self.signal_group(Signal::KILL)?;
        }
        self.reap()
    }

    /// Terminate the owned group or direct child, then reap the child.
    ///
    /// Repeated calls after reaping return the cached status and never signal a
    /// reused PID. A prior group failure still matters even if reaping succeeded;
    /// later calls cannot recover group ownership and do not erase that evidence.
    /// With [`CleanupPolicy::TermThenKill`], successful TERM signalling is followed
    /// by the full grace period without reaping, then KILL even if the leader has
    /// exited. A TERM error does not prevent KILL or reaping. The reap allowance
    /// begins after signalling; repeated termination and Drop never restart it or
    /// repeat escalation. Reap timeout returns `TimedOut` in `wait_error` while
    /// keeping the unreaped child owned. Callers may explicitly recover with
    /// `try_wait`/`wait`; dropping after timeout can leave an unreaped child until
    /// parent exit. No background reaper is installed. Scheduling and native
    /// syscall latency are outside these polling bounds.
    /// # Errors
    /// Retains each failed cleanup step separately. Group failure triggers a
    /// direct-child kill fallback. Drop cannot report errors; call this explicitly
    /// when cleanup evidence matters.
    pub fn terminate(&mut self) -> Result<ExitStatus, CleanupError> {
        if let Some(status) = self.status {
            return Ok(status);
        }
        let first_attempt = self.reap_started.is_none();
        let term_error =
            if first_attempt && let CleanupPolicy::TermThenKill { grace, .. } = self.cleanup {
                let error = self.signal_group(Signal::TERM).err();
                if error.is_none() {
                    let started = Instant::now();
                    while started.elapsed() < grace {
                        std::thread::sleep(
                            grace
                                .saturating_sub(started.elapsed())
                                .min(Duration::from_secs(1)),
                        );
                    }
                }
                error
            } else {
                None
            };
        let group_error = if first_attempt && self.group {
            self.signal_group(Signal::KILL).err()
        } else {
            None
        };
        let kill_error = if first_attempt && self.owned && (!self.group || group_error.is_some()) {
            retry_interrupted(|| self.child.kill()).err()
        } else {
            None
        };
        let waited = match self.cleanup {
            CleanupPolicy::KillAndWait => self.reap(),
            CleanupPolicy::TermThenKill { reap_timeout, .. } => {
                let started = *self.reap_started.get_or_insert_with(Instant::now);
                self.reap_bounded(started, reap_timeout)
            }
        };
        match waited {
            Ok(status) if term_error.is_none() && group_error.is_none() && kill_error.is_none() => {
                Ok(status)
            }
            other => Err(CleanupError {
                status: self.status,
                term_error,
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

    fn observe_exit(&mut self, nonblocking: bool) -> io::Result<Option<WaitIdStatus>> {
        let pid = self.pid()?;
        // NOWAIT reserves the leader PID until cleanup or explicit handoff,
        // avoiding signals to an unrelated group after an early leader exit.
        let mut options = WaitIdOptions::EXITED | WaitIdOptions::NOWAIT;
        if nonblocking {
            options |= WaitIdOptions::NOHANG;
        }
        let result = retry_interrupted(|| waitid(WaitId::Pid(pid), options).map_err(Into::into));
        self.check_wait_ownership(&result);
        result
    }

    #[cfg_attr(
        not(target_os = "macos"),
        allow(
            clippy::needless_pass_by_ref_mut,
            reason = "Darwin inspects and may invalidate child ownership"
        )
    )]
    fn signal_group(&mut self, signal: Signal) -> io::Result<()> {
        let pid = self.pid()?;
        match retry_interrupted(|| kill_process_group(pid, signal).map_err(Into::into)) {
            Ok(()) => Ok(()),
            Err(error) if error.raw_os_error() == Some(Errno::SRCH.raw_os_error()) => Ok(()),
            #[cfg(target_os = "macos")]
            Err(error)
                if error.raw_os_error() == Some(Errno::PERM.raw_os_error())
                    && self.observe_exit(true)?.is_some()
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

    fn reap_bounded(&mut self, started: Instant, timeout: Duration) -> io::Result<ExitStatus> {
        if !self.owned {
            return Err(Errno::CHILD.into());
        }
        loop {
            // Never call blocking wait, including after an interrupted poll.
            let result = self.child.try_wait();
            self.check_wait_ownership(&result);
            match result {
                Ok(Some(status)) => {
                    self.status = Some(status);
                    self.owned = false;
                    return Ok(status);
                }
                Ok(None) => {}
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(error),
            }
            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "child reap timed out",
                ));
            }
            std::thread::sleep(remaining.min(Duration::from_millis(2)));
        }
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
