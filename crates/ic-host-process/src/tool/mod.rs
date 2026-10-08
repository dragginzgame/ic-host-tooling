//! Executable admission and bounded execution of caller-selected Unix commands.
//!
//! Consumers own pins, arguments, credentials, environment and trusted executable
//! directories. Execution is not a sandbox or process-tree supervisor. Calls
//! execute once, including on timeout or ambiguous completion; no retry occurs.

mod process;
mod resolution;
#[cfg(test)]
mod tests;

pub use resolution::{ResolutionError, resolve_executable};

use ic_host_artifacts::artifact::{ArtifactError, ArtifactIdentity, Sha256Digest};
use ic_host_fs::read::hash_file;
use std::{
    ffi::OsString,
    fmt, fs, io,
    os::unix::{ffi::OsStrExt as _, fs::PermissionsExt as _},
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    time::Duration,
};

/// Caller-selected stdout/stderr storage bounds and capture deadline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputLimits {
    /// Maximum retained stdout bytes; zero permits only empty stdout.
    pub stdout_bytes: usize,
    /// Maximum retained stderr bytes; zero permits only empty stderr.
    pub stderr_bytes: usize,
    /// Positive deadline from immediately before spawning through output EOF,
    /// or from entry to [`communicate_child`] for an already spawned child.
    /// Verification reads, spawning syscalls and kill/reap may take longer.
    pub timeout: Duration,
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

/// Exact executable and version authority selected by a consumer.
pub struct ToolSpec<'a> {
    /// Absolute path in a caller-controlled filesystem; no PATH search occurs.
    pub executable: &'a Path,
    /// Admitted SHA-256 of the executable bytes.
    pub sha256: Sha256Digest,
    /// Maximum executable bytes read during every verification.
    pub executable_bytes: u64,
    /// Exact argument vector used to observe version identity.
    pub version_arguments: &'a [OsString],
    /// Required UTF-8 stdout after Unicode whitespace is trimmed at both ends.
    pub version_identity: &'a str,
}

/// Exact version authority for a caller-trusted installed executable.
///
/// Unlike [`ToolSpec`], this supplies no trusted executable digest. Admission
/// records the installed bytes for later drift checks; a matching version is
/// not authentication of those bytes. Consumers own installation provenance.
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

/// Bounded evidence from one invocation, including interrupted invocations.
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
    /// The direct child completed unsuccessfully; its status is in evidence.
    ExitStatus,
    /// Child exit or pipe EOF was not observed before the caller's deadline.
    TimedOut,
    /// The caller's cancellation predicate requested cleanup.
    Cancelled,
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
    /// Failure signalling TERM under the caller-selected child cleanup policy.
    pub term_error: Option<io::Error>,
    /// Failure signalling KILL to an owned process group; absent in direct capture.
    pub group_error: Option<io::Error>,
    /// Failure to terminate the direct child, if termination was needed.
    pub kill_error: Option<io::Error>,
    /// Failure to reap the direct child, if reaping was needed.
    pub wait_error: Option<io::Error>,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.failure {
            ExecutionFailure::ExitStatus => {
                write!(f, "tool exited unsuccessfully: {:?}", self.evidence.status)
            }
            ExecutionFailure::TimedOut => f.write_str("tool capture exceeded its deadline"),
            ExecutionFailure::Cancelled => f.write_str("tool communication cancelled"),
            ExecutionFailure::OutputLimit { stream } => {
                write!(f, "tool {stream:?} exceeded its byte limit")
            }
            ExecutionFailure::Io { operation, .. } => write!(f, "tool {operation} failed"),
            ExecutionFailure::Allocation { stream, .. } => {
                write!(f, "tool {stream:?} allocation failed")
            }
        }
    }
}
impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.failure {
            ExecutionFailure::Io { source, .. } => Some(source),
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
/// capture, nonzero exit, overflow and deadline failures retain bounded evidence.
pub fn capture_command(
    command: &mut Command,
    limits: OutputLimits,
) -> Result<ExecutionEvidence, ToolError> {
    validate_limits(limits)?;
    process::capture_command(command, limits, process::CleanupScope::DirectChild)
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
) -> Result<ExecutionEvidence, ToolError> {
    validate_limits(limits)?;
    process::capture_command(command, limits, process::CleanupScope::ProcessGroup)
        .map_err(|source| ToolError::Execution(Box::new(source)))
}

/// Successful leader disposition during [`communicate_child`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuccessfulExit {
    /// Clean the owned group before reaping, then finish draining captured pipes.
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
/// Stdin closes after input is written. An early broken pipe is accepted, like
/// standard communicate semantics; command status and diagnostics remain primary.
/// This does not guarantee the command consumed all input or applied its effects.
///
/// The deadline starts on entry, excluding earlier spawn time. `cancelled` is
/// polled between bounded IO steps and before returning success; it must return
/// promptly. No signal handler, cancellation thread, retry or output decoding
/// is installed. IO failure, cancellation, timeout, overflow and unsuccessful
/// exit close pipes and terminate/reap through the child's existing owner,
/// retaining original failure and separate cleanup errors.
///
/// On success, all owned pipes are closed. With [`SuccessfulExit::Cleanup`],
/// group cleanup and reaping occur as soon as leader exit is observed. With
/// [`SuccessfulExit::Retain`], the successful leader remains reserved and group
/// cleanup remains armed. After admitting output and checking
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
) -> Result<ExecutionEvidence, ToolError> {
    validate_limits(limits)?;
    process::communicate(child, input, limits, successful_exit, cancelled)
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
}

impl AdmittedTool {
    /// Admit exact executable bytes before invoking the selected version command.
    ///
    /// # Errors
    /// Rejects invalid inputs, non-executable files, digest/size mismatch, process
    /// failures, non-UTF-8 stdout, and a different successful version identity.
    pub fn admit(
        spec: &ToolSpec<'_>,
        context: &ExecutionContext<'_>,
        limits: OutputLimits,
    ) -> Result<Self, ToolError> {
        Self::admit_with_digest(
            &VersionSpec {
                executable: spec.executable,
                executable_bytes: spec.executable_bytes,
                version_arguments: spec.version_arguments,
                version_identity: spec.version_identity,
            },
            Some(spec.sha256),
            context,
            limits,
        )
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
        Self::admit_with_digest(spec, None, context, limits)
    }

    fn admit_with_digest(
        spec: &VersionSpec<'_>,
        expected: Option<Sha256Digest>,
        context: &ExecutionContext<'_>,
        limits: OutputLimits,
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
        let identity = verify_executable(&path, spec.executable_bytes, expected)?;
        let evidence = process::capture(&path, spec.version_arguments, context, limits)
            .map_err(|source| ToolError::Execution(Box::new(source)))?;
        let version = match std::str::from_utf8(&evidence.stdout) {
            Ok(version) => version.trim(),
            Err(source) => {
                return Err(ToolError::VersionUtf8 {
                    source,
                    evidence: Box::new(evidence),
                });
            }
        };
        if version != spec.version_identity {
            return Err(ToolError::VersionMismatch {
                evidence: Box::new(evidence),
            });
        }
        Ok(Self {
            path,
            identity,
            executable_bytes: spec.executable_bytes,
            version_identity: spec.version_identity.to_owned(),
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
    ///
    /// # Errors
    /// Returns invalid-input or identity failures before execution, or an
    /// execution failure retaining bounded prefixes and cleanup outcomes.
    pub fn run(
        &self,
        arguments: &[OsString],
        context: &ExecutionContext<'_>,
        limits: OutputLimits,
    ) -> Result<ExecutionEvidence, ToolError> {
        validate_invocation(arguments, context, limits)?;
        verify_executable(
            &self.path,
            self.executable_bytes,
            Some(self.identity.sha256),
        )?;
        process::capture(&self.path, arguments, context, limits)
            .map_err(|source| ToolError::Execution(Box::new(source)))
    }
}

fn verify_executable(
    path: &Path,
    limit: u64,
    expected: Option<Sha256Digest>,
) -> Result<ArtifactIdentity, ToolError> {
    let actual = hash_file(path, limit).map_err(ToolError::Artifact)?;
    if let Some(expected) = expected
        && actual.sha256 != expected
    {
        return Err(ToolError::Artifact(ArtifactError::DigestMismatch {
            expected,
            actual,
        }));
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
    if limits.timeout.is_zero()
        || std::time::Instant::now()
            .checked_add(limits.timeout)
            .is_none()
    {
        return Err(ToolError::InvalidInvocation(InvalidInvocation::Deadline));
    }
    Ok(())
}
