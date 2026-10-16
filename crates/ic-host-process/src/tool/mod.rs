//! Executable admission and bounded execution of caller-selected Unix commands.
//!
//! Consumers own pins, arguments, credentials, environment and trusted executable
//! directories. Execution is not a sandbox or process-tree supervisor. Calls
//! execute once, including on timeout or ambiguous completion; no retry occurs.

mod diagnostics;
mod process;
mod resolution;
#[cfg(test)]
mod tests;

pub use diagnostics::format_failure_evidence;
pub use resolution::{ResolutionError, resolve_executable};

use ic_host_artifacts::artifact::{ArtifactError, ArtifactIdentity, Sha256Digest};
use ic_host_fs::read::{hash_file, hash_file_no_follow};
use std::{
    ffi::OsString,
    fmt, fs, io,
    os::unix::{ffi::OsStrExt as _, fs::PermissionsExt as _},
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    time::{Duration, Instant},
};

/// Retention and overflow behavior for one output stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputLimit {
    /// Retain at most this many bytes; excess output fails and cleans up the child.
    Terminate(usize),
    /// Retain this prefix, drain the rest, and report every byte to an observer.
    /// Truncation is recorded in evidence, without failing the command.
    Truncate(usize),
}

impl OutputLimit {
    /// Maximum retained bytes, independently of overflow behavior.
    #[must_use]
    pub const fn retained_bytes(self) -> usize {
        match self {
            Self::Terminate(bytes) | Self::Truncate(bytes) => bytes,
        }
    }
}

/// Caller-selected output retention and optional elapsed-time deadline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputLimits {
    /// Standard output retention and overflow behavior.
    pub stdout: OutputLimit,
    /// Standard error retention and overflow behavior.
    pub stderr: OutputLimit,
    /// Positive deadline from immediately before spawning through output EOF,
    /// or from entry to [`communicate_child`] for an already spawned child.
    /// Verification reads, spawning syscalls and kill/reap may take longer.
    ///
    /// `None` still enforces output bounds, polls cancellation and cleans up on
    /// failure. It can wait indefinitely for child exit or pipe EOF, including
    /// descendant-held pipes with [`SuccessfulExit::Retain`]. Choose cancellation
    /// and successful-exit policy deliberately. Cleanup timing remains governed
    /// by the child's [`crate::child::CleanupPolicy`].
    pub timeout: Option<Duration>,
}

/// Explicit process context. The child's inherited environment is cleared.
///
/// No ambient PATH, HOME or credentials are added. Consumers must include any
/// environment needed by their tool. This type intentionally has no `Debug`
/// implementation because its values may contain credentials.
pub struct ExecutionContext<'a> {
    /// Absolute current directory for the child.
    pub current_dir: &'a Path,
    /// Complete environment; names must be nonempty, unique, and contain no
    /// `=` or NUL bytes. Values must contain no NUL bytes.
    pub environment: &'a [(OsString, OsString)],
}

/// Executable and exact version authority selected by a consumer.
///
/// [`AdmittedTool::admit`] and [`AdmittedTool::admit_no_follow`] require a
/// separate trusted digest. [`AdmittedTool::admit_version`] instead records
/// caller-trusted installed bytes; a matching version does not authenticate
/// those bytes. Consumers own installation provenance.
pub struct VersionSpec<'a> {
    /// Absolute path in a caller-controlled filesystem; no PATH search occurs.
    pub executable: &'a Path,
    /// Maximum executable bytes read during admission and every later run.
    pub executable_bytes: u64,
    /// Exact argument vector used to observe version identity.
    pub version_arguments: &'a [OsString],
    /// Required UTF-8 stdout after Unicode whitespace is trimmed at both ends.
    pub version_identity: &'a str,
}

/// Invocation parameters rejected before dispatch or communication.
/// An already spawned child remains owned and untouched by this validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidInvocation {
    /// Tool paths must be absolute.
    ExecutablePath,
    /// Working directories must be absolute.
    WorkingDirectory,
    /// Deadlines must be positive and representable by the host clock.
    Deadline,
    /// Version identity must be nonempty and already trimmed.
    VersionIdentity,
    /// An argument contains a NUL byte.
    Argument {
        /// Argument position.
        index: usize,
    },
    /// An environment name is empty, contains `=`/NUL, or duplicates an earlier name.
    EnvironmentName {
        /// Environment entry position.
        index: usize,
    },
    /// An environment value contains a NUL byte.
    EnvironmentValue {
        /// Environment entry position.
        index: usize,
    },
}

/// A captured output stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// Bounded evidence retained after an invocation or admission failure.
///
/// Bytes are available explicitly to the caller; formatting prints only lengths
/// and status. Error messages never include command arguments or environment.
#[derive(Default)]
pub struct ExecutionEvidence {
    /// Observed direct-child status. Communication can reserve a successful
    /// leader without reaping it. Status does not prove external effects absent.
    pub status: Option<ExitStatus>,
    /// Retained stdout prefix, bounded by the selected limit.
    pub stdout: Vec<u8>,
    /// Retained stderr prefix, bounded by the selected limit.
    pub stderr: Vec<u8>,
    /// More stdout bytes were observed than could be retained.
    pub stdout_truncated: bool,
    /// More stderr bytes were observed than could be retained.
    pub stderr_truncated: bool,
}

/// One completed invocation, with a required exit status and bounded output.
///
/// Completion includes nonzero exits and signals. It does not imply successful
/// application effects. Use [`Self::require_success`] before success-only parsing.
/// A retained successful child still requires caller-owned disposition.
pub struct CompletedExecution {
    /// Direct-child exit status, including nonzero exits and signals.
    pub status: ExitStatus,
    /// Retained stdout prefix.
    pub stdout: Vec<u8>,
    /// Retained stderr prefix.
    pub stderr: Vec<u8>,
    /// More stdout bytes were observed than retained.
    pub stdout_truncated: bool,
    /// More stderr bytes were observed than retained.
    pub stderr_truncated: bool,
}

impl CompletedExecution {
    /// Require a successful exit while preserving output on rejection.
    ///
    /// # Errors
    /// Returns [`ExecutionFailure::ExitStatus`] for a nonzero or signalled exit.
    /// This does not perform child cleanup or transfer retained ownership.
    pub fn require_success(self) -> Result<Self, ToolError> {
        if self.status.success() {
            Ok(self)
        } else {
            Err(self.failure(ExecutionFailure::ExitStatus))
        }
    }

    /// Require complete output before parsing or treating it as an identity.
    ///
    /// # Errors
    /// Returns an output-limit failure with the original evidence if either
    /// stream was truncated. This does not perform child cleanup; callers using
    /// retained ownership still own the child's disposition.
    pub fn require_complete(self) -> Result<Self, ToolError> {
        let stream = if self.stdout_truncated {
            OutputStream::Stdout
        } else if self.stderr_truncated {
            OutputStream::Stderr
        } else {
            return Ok(self);
        };
        Err(self.failure(ExecutionFailure::OutputLimit { stream }))
    }

    fn failure(self, failure: ExecutionFailure) -> ToolError {
        ToolError::Execution(Box::new(ExecutionError {
            failure,
            evidence: self.into(),
            cleanup: None,
        }))
    }
}

impl From<CompletedExecution> for ExecutionEvidence {
    fn from(completed: CompletedExecution) -> Self {
        Self {
            status: Some(completed.status),
            stdout: completed.stdout,
            stderr: completed.stderr,
            stdout_truncated: completed.stdout_truncated,
            stderr_truncated: completed.stderr_truncated,
        }
    }
}

impl fmt::Debug for CompletedExecution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompletedExecution")
            .field("status", &self.status)
            .field("stdout_bytes", &self.stdout.len())
            .field("stderr_bytes", &self.stderr.len())
            .field("stdout_truncated", &self.stdout_truncated)
            .field("stderr_truncated", &self.stderr_truncated)
            .finish()
    }
}

impl fmt::Debug for ExecutionEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionEvidence")
            .field("status", &self.status)
            .field("stdout_bytes", &self.stdout.len())
            .field("stderr_bytes", &self.stderr.len())
            .field("stdout_truncated", &self.stdout_truncated)
            .field("stderr_truncated", &self.stderr_truncated)
            .finish()
    }
}

/// Process or pipe operation that produced an IO failure.
///
/// These categories contain no arguments, environment or captured output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionOperation {
    /// Create the child process in its selected working directory.
    Spawn,
    /// Obtain the child's pipe for explicitly supplied input.
    StdinPipe,
    /// Read an IO pipe's current descriptor flags.
    ReadPipeFlags,
    /// Enable nonblocking IO on a pipe.
    SetPipeFlags,
    /// Read bytes from a capture pipe.
    ReadOutput,
    /// Write bytes to the child's stdin pipe.
    WriteInput,
    /// Observe child exit, including selected group cleanup before reaping.
    Wait,
}

impl fmt::Display for ExecutionOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Spawn => "spawn",
            Self::StdinPipe => "stdin pipe",
            Self::ReadPipeFlags => "read pipe flags",
            Self::SetPipeFlags => "set pipe flags",
            Self::ReadOutput => "read output",
            Self::WriteInput => "write input",
            Self::Wait => "wait",
        })
    }
}

/// Why an invocation failed, independently of its retained output.
#[derive(Debug)]
pub enum ExecutionFailure {
    /// An explicit success check rejected the completed status in evidence.
    ExitStatus,
    /// Child exit or pipe EOF was not observed before the caller's deadline.
    TimedOut,
    /// The caller's cancellation predicate requested cleanup.
    Cancelled,
    /// The output observer rejected a chunk; its original sink error is retained.
    Observer {
        /// Stream being observed.
        stream: OutputStream,
        /// Caller-provided output-sink failure.
        source: io::Error,
    },
    /// A stream emitted more bytes than allowed.
    OutputLimit {
        /// Stream whose bound was exceeded.
        stream: OutputStream,
    },
    /// A process or pipe operation failed.
    Io {
        /// Operation category; contains no command or credential values.
        operation: ExecutionOperation,
        /// Underlying typed failure.
        source: io::Error,
    },
    /// Retained output storage could not be allocated.
    Allocation {
        /// Stream being captured.
        stream: OutputStream,
        /// Underlying allocation failure.
        source: std::collections::TryReserveError,
    },
}

/// A failed invocation with bounded output and separately retained cleanup errors.
#[derive(Debug)]
pub struct ExecutionError {
    /// Original failure; never replaced by a cleanup failure.
    pub failure: ExecutionFailure,
    /// Observed status and bounded stdout/stderr prefixes.
    pub evidence: ExecutionEvidence,
    /// Cleanup failures retained by the child owner, if cleanup failed.
    pub cleanup: Option<Box<crate::child::CleanupError>>,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.failure {
            ExecutionFailure::ExitStatus => {
                write!(f, "tool exited unsuccessfully: {:?}", self.evidence.status)
            }
            ExecutionFailure::TimedOut => f.write_str("tool capture exceeded its deadline"),
            ExecutionFailure::Cancelled => f.write_str("tool communication cancelled"),
            ExecutionFailure::Observer { stream, .. } => {
                write!(f, "tool {stream:?} observer failed")
            }
            ExecutionFailure::OutputLimit { stream } => {
                write!(f, "tool {stream:?} exceeded its byte limit")
            }
            ExecutionFailure::Io { operation, .. } => write!(f, "tool {operation} failed"),
            ExecutionFailure::Allocation { stream, .. } => {
                write!(f, "tool {stream:?} allocation failed")
            }
        }?;
        if let Some(cleanup) = &self.cleanup {
            write!(f, "; cleanup failed: {cleanup}")?;
        }
        Ok(())
    }
}
impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.failure {
            ExecutionFailure::Io { source, .. } | ExecutionFailure::Observer { source, .. } => {
                Some(source)
            }
            ExecutionFailure::Allocation { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Executable admission or execution failed.
#[derive(Debug)]
pub enum ToolError {
    /// Invalid context, authority or arguments; no new child was spawned.
    /// Communication validation leaves an existing child and its pipes untouched.
    InvalidInvocation(InvalidInvocation),
    /// Filesystem path resolution or metadata failed before execution.
    Io(io::Error),
    /// The selected file has no Unix executable permission bits.
    NotExecutable,
    /// Executable identity could not be read or did not match authority.
    Artifact(ArtifactError),
    /// One invocation failed; includes bounded evidence.
    Execution(Box<ExecutionError>),
    /// Successful version stdout was not valid UTF-8.
    VersionUtf8 {
        /// UTF-8 validation failure.
        source: std::str::Utf8Error,
        /// Raw bounded version output.
        evidence: Box<ExecutionEvidence>,
    },
    /// Successful trimmed version stdout did not match the selected identity.
    VersionMismatch {
        /// Raw bounded version output; never formatted into the error.
        evidence: Box<ExecutionEvidence>,
    },
}

impl ToolError {
    /// Borrow the original bounded capture, including successful version output.
    ///
    /// Validation and filesystem/admission failures have no capture and return
    /// `None`. A failed spawn retains its existing empty execution evidence.
    /// No output is copied, decoded, logged or formatted by this accessor.
    #[must_use]
    pub fn evidence(&self) -> Option<&ExecutionEvidence> {
        match self {
            Self::Execution(error) => Some(&error.evidence),
            Self::VersionUtf8 { evidence, .. } | Self::VersionMismatch { evidence } => {
                Some(evidence)
            }
            Self::InvalidInvocation(_) | Self::Io(_) | Self::NotExecutable | Self::Artifact(_) => {
                None
            }
        }
    }

    /// Borrow the original execution failure and its kill/reap outcomes, if any.
    ///
    /// A successful version capture rejected by admission has evidence but no
    /// execution failure. Formatting, redaction and recovery remain caller-owned.
    #[must_use]
    pub fn execution_error(&self) -> Option<&ExecutionError> {
        match self {
            Self::Execution(error) => Some(error),
            _ => None,
        }
    }
}

/// Capture one caller-configured command without performing executable admission.
///
/// The caller owns the program, arguments, working directory, environment and
/// platform setup. Their [`Command`] settings are used unchanged except that
/// stdin is set to null and stdout/stderr to pipes. Ambient environment or PATH
/// search remains enabled if the caller's command enables it. No digest/version
/// check, credential selection, command reconstruction or retry is performed.
/// Use [`AdmittedTool`] when exact executable-byte/version admission is required.
///
/// Shares the admitted-tool execution engine: stdout/stderr are drained fairly
/// with bounded storage, and the deadline starts immediately before spawning
/// and extends through pipe EOF. On failure, pipes are closed and the direct
/// child is terminated/reaped, retaining the original failure and cleanup
/// evidence. Descendants, platform setup hooks and inherited descriptor lifetimes
/// remain caller-owned. Spawning, setup hooks and kill/reap are synchronous and
/// may exceed the deadline. This does not supervise a process group, roll back
/// an external effect or make an uncertain command safe to repeat.
///
/// # Errors
/// Rejects an invalid deadline before touching or spawning the command. Spawn,
/// capture, overflow and deadline failures retain bounded evidence. Ordinary
/// nonzero/signalled exits return [`CompletedExecution`]; select
/// [`CompletedExecution::require_success`] when the caller requires zero exit.
pub fn capture_command(
    command: &mut Command,
    limits: OutputLimits,
) -> Result<CompletedExecution, ToolError> {
    validate_limits(limits)?;
    process::capture_command(
        command,
        limits,
        process::CleanupScope::DirectChild,
        true,
        || false,
    )
    .map_err(|source| ToolError::Execution(Box::new(source)))
}

/// Capture one caller-configured command in a newly owned process group.
///
/// Uses the same bounded, fair stdout/stderr capture and deadline as
/// [`capture_command`], with null stdin and no executable admission or retries.
/// Preserves caller command settings except IO and process-group selection;
/// the latter is replaced with a new owned group. On natural leader exit,
/// timeout, overflow or IO failure, remaining group members are signalled before
/// reaping the leader. Original failures and group/direct-child cleanup errors
/// remain separate in [`ExecutionError`]. There is no implicit background handoff.
///
/// Callers must not reap the leader independently or change its group. Escaped
/// descendants are not contained, and signalling is not proof of descendant exit
/// or completed external effects. Synchronous spawning/setup/cleanup can exceed
/// the deadline. Admission, budgets, inherited descriptors and recovery remain
/// caller-owned. Use [`capture_command`] for the existing direct-child contract.
/// # Errors
/// Rejects invalid deadlines before modifying/spawning the command. Execution
/// failures retain bounded output, observed status and separate cleanup errors.
pub fn capture_group_command(
    command: &mut Command,
    limits: OutputLimits,
) -> Result<CompletedExecution, ToolError> {
    validate_limits(limits)?;
    process::capture_command(
        command,
        limits,
        process::CleanupScope::ProcessGroup {
            cleanup: crate::child::CleanupPolicy::KillAndWait,
            deadline: None,
        },
        true,
        || false,
    )
    .map_err(|source| ToolError::Execution(Box::new(source)))
}

/// Capture a command with caller-selected group cleanup, deadline and cancellation.
///
/// Shares [`capture_group_command`]'s IO, bounds, exit and cleanup contract.
/// The callback runs before command modification/spawn and during communication;
/// it must return promptly. A true value refuses startup or cleans up the owned
/// group, retaining output and cleanup evidence. An expired absolute deadline
/// refuses before command effects; a live deadline is shared across calls without
/// restarting its allowance. If `limits.timeout` is also set, the earlier bound
/// applies. Both bounds include callback/spawn time, but synchronous spawning and
/// cleanup may exceed them. `None` adds no deadline. No signal handler, retry or
/// admission policy is installed. Cleanup and limits remain caller-selected.
///
/// # Errors
/// Invalid limits refuse before the callback or command effects. Cancellation,
/// spawn and communication failures retain the existing typed execution evidence.
pub fn capture_group_command_with_cancellation(
    command: &mut Command,
    limits: OutputLimits,
    cleanup: crate::child::CleanupPolicy,
    deadline: Option<Instant>,
    cancelled: impl FnMut() -> bool,
) -> Result<CompletedExecution, ToolError> {
    validate_limits(limits)?;
    process::capture_command(
        command,
        limits,
        process::CleanupScope::ProcessGroup { cleanup, deadline },
        true,
        cancelled,
    )
    .map_err(|source| ToolError::Execution(Box::new(source)))
}

/// Run one caller-configured command in an owned process group.
///
/// Preserves the caller's stdin/stdout/stderr selection, including inherited
/// progress output and file-backed streams. Only piped output is captured and
/// bounded. Any piped stdin is closed; this entry point sends no input bytes.
/// Group selection is replaced with a new owned group. This shares
/// [`capture_group_command_with_cancellation`]'s pre-spawn validation,
/// cancellation, absolute/relative deadlines and typed cleanup evidence.
/// The caller selects budgets, cleanup and cancellation; `None` adds no deadline.
/// Ordinary nonzero/signalled exits return [`CompletedExecution`]; callers
/// choose [`CompletedExecution::require_success`]. There is no admission or retry.
///
/// # Errors
/// Invalid limits, initial cancellation and expired deadlines refuse before
/// modifying/spawning the command. Later failures clean up the owned group.
pub fn run_group_command(
    command: &mut Command,
    limits: OutputLimits,
    cleanup: crate::child::CleanupPolicy,
    deadline: Option<Instant>,
    cancelled: impl FnMut() -> bool,
) -> Result<CompletedExecution, ToolError> {
    validate_limits(limits)?;
    process::capture_command(
        command,
        limits,
        process::CleanupScope::ProcessGroup { cleanup, deadline },
        false,
        cancelled,
    )
    .map_err(|source| ToolError::Execution(Box::new(source)))
}

/// Successful leader disposition during [`communicate_child`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessfulExit {
    /// Clean according to child ownership before reaping, then drain captured pipes.
    /// Direct-child ownership never signals descendants or other group members.
    Cleanup,
    /// Reserve the successful leader through IO completion and caller admission.
    /// The caller must then explicitly wait, terminate or hand off the owner.
    Retain,
}

/// Communicate once with a caller-spawned child using its configured IO.
///
/// Uses the capture engine to fairly drain available stdout/stderr pipes and
/// write borrowed input without blocking reader/writer threads. The caller
/// configures IO before [`crate::child::OwnedChild::spawn`]: absent output pipes
/// are left alone (for example inherited progress output or files). Limits only
/// apply to captured pipes. Do not take the child's pipes before this call.
/// `Some(input)`, including empty input, requires piped stdin. `None` closes any
/// available stdin pipe immediately; inherited stdin remains caller-selected.
/// [`crate::child::OwnedChild::spawn_direct`] preserves process-group selection
/// and limits cleanup to the direct child. Descendant-held pipes can still delay
/// EOF; this function does not acquire descendant or terminal ownership.
/// Stdin closes after input is written. An early broken pipe is accepted, like
/// standard communicate semantics; command status and diagnostics remain primary.
/// This does not guarantee the command consumed all input or applied its effects.
///
/// A selected deadline starts on entry, excluding earlier spawn time;
/// `timeout: None` imposes no elapsed-time deadline. Output retention and
/// cleanup remain active in either case. `cancelled` is
/// polled between bounded IO steps and before returning success; it must return
/// promptly. No signal handler, cancellation thread, retry or output decoding
/// is installed. IO failure, cancellation, timeout and overflow
/// close pipes and terminate/reap through the child's existing owner,
/// retaining original failure and separate cleanup errors.
///
/// Completed zero/nonzero/signalled exits return a required status with all
/// owned pipes closed. Nonzero/signalled leaders always undergo ordinary cleanup
/// and reaping, including with [`SuccessfulExit::Retain`]; they cannot be handed off.
/// With [`SuccessfulExit::Cleanup`],
/// cleanup for the selected ownership scope and reaping occur as soon as child
/// exit is observed. With [`SuccessfulExit::Retain`], the successful child remains
/// reserved and its selected cleanup remains armed. After admitting output and checking
/// application cancellation/deadlines, the caller must choose ordinary
/// [`crate::child::OwnedChild::wait`] cleanup or explicit
/// [`crate::child::OwnedChild::handoff`]. Rejected output can use `terminate`
/// to retain cleanup evidence. Dropping the owner provides best-effort cleanup.
/// This is not process-tree confinement or a descendant-exit barrier.
///
/// # Errors
/// Invalid deadlines leave the already spawned child and its pipes untouched;
/// the caller retains cleanup responsibility. An already terminating/reaped/transferred
/// owner is rejected instead of reporting reserved success. Other failures return execution
/// evidence after cleanup. Cleanup uses the policy chosen at child spawn and
/// can exceed the communication deadline by its separate grace/reap allowance.
pub fn communicate_child(
    child: &mut crate::child::OwnedChild,
    input: Option<&[u8]>,
    limits: OutputLimits,
    successful_exit: SuccessfulExit,
    cancelled: impl FnMut() -> bool,
) -> Result<CompletedExecution, ToolError> {
    communicate_child_with_observer(child, input, limits, successful_exit, cancelled, |_, _| {
        Ok(())
    })
}

/// Communicate with an owned child while observing output bytes live.
///
/// Uses the same bounds, pipe handling and cleanup as [`communicate_child`].
/// `output` receives nonempty borrowed chunks after retention, in order within
/// each stream; chunk boundaries and ordering between streams are unspecified.
/// With [`OutputLimit::Terminate`], only the retained prefix is reported before
/// overflow fails. With [`OutputLimit::Truncate`], every drained byte is reported,
/// while evidence retains only the selected prefix. A zero retained-byte budget
/// can therefore forward output without storing it.
/// Output is raw bytes, not necessarily complete lines or UTF-8.
/// The observer returns `io::Result<()>`. Its first error stops observation,
/// closes pipes and attempts cleanup, preserving the original sink error as
/// [`ExecutionFailure::Observer`] alongside retained bytes and cleanup failures.
/// A chunk is retained and any truncation recorded before observation; rejection
/// takes precedence over a hard overflow detected in that same chunk.
///
/// Both callbacks run synchronously and must return promptly. `cancelled` also
/// runs while the child is silent and may project caller-owned heartbeat events;
/// no exact callback cadence is guaranteed. Callback time counts toward any
/// selected communication deadline. A callback panic closes the pipes, attempts cleanup
/// using the child's policy and resumes the original unwind, even if the caller
/// catches it while retaining the child. Cleanup errors cannot be returned during
/// unwind; any unreaped child remains owned for caller recovery.
///
/// Callers own event schemas, scheduling, output budgets and diagnostic rendering.
/// Each stream explicitly selects hard overflow or truncation-and-continue.
///
/// # Panics
/// Resumes callback panics after attempting cleanup with the selected policy.
///
/// # Errors
/// Returns the same validation and execution errors as [`communicate_child`].
pub fn communicate_child_with_observer(
    child: &mut crate::child::OwnedChild,
    input: Option<&[u8]>,
    limits: OutputLimits,
    successful_exit: SuccessfulExit,
    cancelled: impl FnMut() -> bool,
    output: impl FnMut(OutputStream, &[u8]) -> io::Result<()>,
) -> Result<CompletedExecution, ToolError> {
    validate_limits(limits)?;
    process::communicate(child, input, limits, successful_exit, cancelled, output)
        .map_err(|source| ToolError::Execution(Box::new(source)))
}

/// Complete a child after a caller has read control frames from its stdout.
///
/// Hand off the original buffered pipe without converting it to its raw inner
/// pipe: unread prefetched bytes count against the same output budget as later
/// bytes. `None` uses any stdout still held by the child, or captures no stdout
/// if the caller consumed it completely. Stdin is closed; stderr is taken from
/// the child and both streams use the existing nonblocking capture engine.
/// No child is respawned and no retry occurs.
///
/// Unlike [`communicate_child`], a previously reaped/terminated child may be
/// drained using its cached status; it is never signalled again. Transferred
/// ownership refuses. Successful children use ordinary cleanup and cannot be
/// handed off. Callers retain pipe provenance, protocol decoding, limits and
/// cancellation. A selected timeout starts on entry; no default deadline is added.
///
/// # Errors
/// Returns validation, capture and cleanup evidence as [`communicate_child`].
/// Invalid limits do not touch the child; the passed reader is consumed/dropped.
pub fn drain_child_output(
    child: &mut crate::child::OwnedChild,
    stdout: Option<std::io::BufReader<std::process::ChildStdout>>,
    limits: OutputLimits,
    cancelled: impl FnMut() -> bool,
) -> Result<CompletedExecution, ToolError> {
    validate_limits(limits)?;
    process::drain_output(child, stdout, limits, cancelled)
        .map_err(|source| ToolError::Execution(Box::new(source)))
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInvocation(input) => write!(f, "invalid tool invocation: {input:?}"),
            Self::Io(_) => f.write_str("tool filesystem inspection failed"),
            Self::NotExecutable => f.write_str("tool file is not executable"),
            Self::Artifact(source) => write!(f, "tool identity verification failed: {source}"),
            Self::Execution(source) => source.fmt(f),
            Self::VersionUtf8 { .. } => f.write_str("tool version output is not UTF-8"),
            Self::VersionMismatch { .. } => f.write_str("tool version does not match authority"),
        }
    }
}
impl std::error::Error for ToolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::Artifact(source) => Some(source),
            Self::Execution(source) => Some(source.as_ref()),
            Self::VersionUtf8 { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// An executable with a retained byte identity and an admitted exact version.
///
/// [`Self::admit`] requires a consumer-supplied digest. [`Self::admit_version`]
/// records the installed identity without authenticating it against a pin.
/// Every execution rechecks digest/permission before spawning. Consumers must
/// exclude concurrent writers to the executable and its parent directories:
/// filesystem checks and `exec` are separate operations. This is not a file
/// capability or verification of dynamic libraries, interpreters, or descendants.
pub struct AdmittedTool {
    path: PathBuf,
    identity: ArtifactIdentity,
    executable_bytes: u64,
    version_identity: String,
    no_follow: bool,
}

impl AdmittedTool {
    /// Admit exact executable bytes before invoking the selected version command.
    ///
    /// # Errors
    /// Rejects invalid inputs, non-executable files, digest/size mismatch, process
    /// failures, non-UTF-8 stdout, and a different successful version identity.
    pub fn admit(
        spec: &VersionSpec<'_>,
        expected: Sha256Digest,
        context: &ExecutionContext<'_>,
        limits: OutputLimits,
    ) -> Result<Self, ToolError> {
        Self::admit_with_digest(spec, Some(expected), context, limits, false)
    }

    /// Admit exact bytes without following the executable's final-component symlink.
    ///
    /// Performs one bounded no-follow hash before the version command. Parents
    /// and concurrent writers remain caller-owned. The retained canonical path
    /// also rejects final-component links during subsequent [`Self::run`] calls.
    ///
    /// # Errors
    /// Returns the same admission failures as [`Self::admit`], additionally
    /// refusing final-component symbolic links and special files before execution.
    pub fn admit_no_follow(
        spec: &VersionSpec<'_>,
        expected: Sha256Digest,
        context: &ExecutionContext<'_>,
        limits: OutputLimits,
    ) -> Result<Self, ToolError> {
        Self::admit_with_digest(spec, Some(expected), context, limits, true)
    }

    /// Admit an exact version and record the caller-trusted installed bytes.
    ///
    /// For tools built locally, no portable published digest may exist. This
    /// entry hashes the executable within the supplied budget before running
    /// the version command. The resulting [`Self::identity`] is an observation,
    /// not a trusted published pin. All later runs reject changed bytes using
    /// the same verification and capture engine as [`Self::admit`].
    ///
    /// The caller must trust the installation before admission: the version
    /// command executes those bytes. Exact version output does not establish
    /// authenticity. No version ranges, tool installation or PATH search occur.
    /// The caller must exclude concurrent executable/directory writers.
    ///
    /// # Errors
    /// Rejects invalid inputs, non-executable or oversized files, process
    /// failures, non-UTF-8 stdout and a different successful version identity.
    pub fn admit_version(
        spec: &VersionSpec<'_>,
        context: &ExecutionContext<'_>,
        limits: OutputLimits,
    ) -> Result<Self, ToolError> {
        Self::admit_with_digest(spec, None, context, limits, false)
    }

    fn admit_with_digest(
        spec: &VersionSpec<'_>,
        expected: Option<Sha256Digest>,
        context: &ExecutionContext<'_>,
        limits: OutputLimits,
        no_follow: bool,
    ) -> Result<Self, ToolError> {
        if !spec.executable.is_absolute() {
            return Err(ToolError::InvalidInvocation(
                InvalidInvocation::ExecutablePath,
            ));
        }
        if spec.version_identity.is_empty() || spec.version_identity.trim() != spec.version_identity
        {
            return Err(ToolError::InvalidInvocation(
                InvalidInvocation::VersionIdentity,
            ));
        }
        validate_invocation(spec.version_arguments, context, limits)?;
        let path = fs::canonicalize(spec.executable).map_err(ToolError::Io)?;
        let identity = verify_executable(
            if no_follow { spec.executable } else { &path },
            spec.executable_bytes,
            expected,
            no_follow,
        )?;
        let evidence = process::capture(&path, spec.version_arguments, context, limits)
            .map_err(|source| ToolError::Execution(Box::new(source)))?
            .require_success()?
            .require_complete()?;
        let version = match std::str::from_utf8(&evidence.stdout) {
            Ok(version) => version.trim(),
            Err(source) => {
                return Err(ToolError::VersionUtf8 {
                    source,
                    evidence: Box::new(evidence.into()),
                });
            }
        };
        if version != spec.version_identity {
            return Err(ToolError::VersionMismatch {
                evidence: Box::new(evidence.into()),
            });
        }
        Ok(Self {
            path,
            identity,
            executable_bytes: spec.executable_bytes,
            version_identity: spec.version_identity.to_owned(),
            no_follow,
        })
    }

    /// Canonical absolute path selected during admission.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Retained raw executable identity.
    ///
    /// With [`Self::admit_version`], this is an observed identity, not proof of
    /// a published binary pin or trusted installation provenance.
    #[must_use]
    pub const fn identity(&self) -> ArtifactIdentity {
        self.identity
    }

    /// Successfully observed, trimmed version identity.
    #[must_use]
    pub fn version_identity(&self) -> &str {
        &self.version_identity
    }

    /// Run once with a cleared, explicitly supplied environment and null stdin.
    ///
    /// Stdout/stderr are drained fairly through nonblocking pipes without reader
    /// threads. On overflow/deadline the direct child is killed and reaped; pipe
    /// handles are closed without waiting for descendants to close their copies.
    /// Descendant processes remain caller-owned. This must not be interpreted
    /// as a rollback or safe automatic retry of a command with external effects.
    /// Nonzero/signalled exits are completed results. Call
    /// [`CompletedExecution::require_success`] before success-only interpretation.
    ///
    /// # Errors
    /// Returns invalid-input or identity failures before execution, or an
    /// execution failure retaining bounded prefixes and cleanup outcomes.
    pub fn run(
        &self,
        arguments: &[OsString],
        context: &ExecutionContext<'_>,
        limits: OutputLimits,
    ) -> Result<CompletedExecution, ToolError> {
        validate_invocation(arguments, context, limits)?;
        verify_executable(
            &self.path,
            self.executable_bytes,
            Some(self.identity.sha256),
            self.no_follow,
        )?;
        process::capture(&self.path, arguments, context, limits)
            .map_err(|source| ToolError::Execution(Box::new(source)))
    }
}

fn verify_executable(
    path: &Path,
    limit: u64,
    expected: Option<Sha256Digest>,
    no_follow: bool,
) -> Result<ArtifactIdentity, ToolError> {
    let actual = if no_follow {
        hash_file_no_follow(path, limit)
    } else {
        hash_file(path, limit)
    }
    .map_err(ToolError::Artifact)?;
    if let Some(expected) = expected {
        actual.verify(expected).map_err(ToolError::Artifact)?;
    }
    if fs::metadata(path)
        .map_err(ToolError::Io)?
        .permissions()
        .mode()
        & 0o111
        == 0
    {
        return Err(ToolError::NotExecutable);
    }
    Ok(actual)
}

fn validate_invocation(
    arguments: &[OsString],
    context: &ExecutionContext<'_>,
    limits: OutputLimits,
) -> Result<(), ToolError> {
    let reject = |input| ToolError::InvalidInvocation(input);
    if !context.current_dir.is_absolute() {
        return Err(reject(InvalidInvocation::WorkingDirectory));
    }
    validate_limits(limits)?;
    for (index, argument) in arguments.iter().enumerate() {
        if argument.as_bytes().contains(&0) {
            return Err(reject(InvalidInvocation::Argument { index }));
        }
    }
    for (index, (key, value)) in context.environment.iter().enumerate() {
        if key.is_empty()
            || key.as_bytes().iter().any(|byte| matches!(byte, b'=' | 0))
            || context.environment[..index]
                .iter()
                .any(|(earlier, _)| earlier == key)
        {
            return Err(reject(InvalidInvocation::EnvironmentName { index }));
        }
        if value.as_bytes().contains(&0) {
            return Err(reject(InvalidInvocation::EnvironmentValue { index }));
        }
    }
    Ok(())
}

fn validate_limits(limits: OutputLimits) -> Result<(), ToolError> {
    if limits.timeout.is_some_and(|timeout| {
        timeout.is_zero() || std::time::Instant::now().checked_add(timeout).is_none()
    }) {
        return Err(ToolError::InvalidInvocation(InvalidInvocation::Deadline));
    }
    Ok(())
}
