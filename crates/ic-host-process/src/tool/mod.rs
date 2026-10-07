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
    /// Positive deadline from immediately before spawning through output EOF.
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

/// An invalid invocation was rejected before a child was spawned.
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
    /// Observed direct-child status, if reaped. A killed process's status does
    /// not prove that any external effect did not happen.
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
    /// Obtain the child's stdout pipe.
    StdoutPipe,
    /// Obtain the child's stderr pipe.
    StderrPipe,
    /// Read a capture pipe's current descriptor flags.
    ReadPipeFlags,
    /// Enable nonblocking reads on a capture pipe.
    SetPipeFlags,
    /// Read bytes from a capture pipe.
    ReadOutput,
    /// Observe the direct child's exit status.
    Wait,
}

impl fmt::Display for ExecutionOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Spawn => "spawn",
            Self::StdoutPipe => "stdout pipe",
            Self::StderrPipe => "stderr pipe",
            Self::ReadPipeFlags => "read pipe flags",
            Self::SetPipeFlags => "set pipe flags",
            Self::ReadOutput => "read output",
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

/// A failed invocation with bounded output and direct-child cleanup evidence.
#[derive(Debug)]
pub struct ExecutionError {
    /// Original failure; never replaced by a cleanup failure.
    pub failure: ExecutionFailure,
    /// Observed status and bounded stdout/stderr prefixes.
    pub evidence: ExecutionEvidence,
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
    /// Invalid context, authority or arguments; no child was spawned.
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
    process::capture_command(command, limits)
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

/// An executable whose exact bytes and version were admitted by a consumer.
///
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
        let identity = verify_executable(&path, spec.executable_bytes, spec.sha256)?;
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

    /// Admitted raw executable identity.
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
        verify_executable(&self.path, self.executable_bytes, self.identity.sha256)?;
        process::capture(&self.path, arguments, context, limits)
            .map_err(|source| ToolError::Execution(Box::new(source)))
    }
}

fn verify_executable(
    path: &Path,
    limit: u64,
    expected: Sha256Digest,
) -> Result<ArtifactIdentity, ToolError> {
    let actual = hash_file(path, limit).map_err(ToolError::Artifact)?;
    if actual.sha256 != expected {
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
