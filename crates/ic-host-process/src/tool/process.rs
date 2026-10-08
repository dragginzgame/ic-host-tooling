use super::{
    ExecutionContext, ExecutionError, ExecutionEvidence, ExecutionFailure, ExecutionOperation,
    OutputLimits, OutputStream, SuccessfulExit,
};
use crate::child::OwnedChild;
use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
use std::{
    ffi::OsString,
    io::{self, Read, Write},
    os::fd::AsFd,
    path::Path,
    process::{Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

#[cfg(test)]
mod tests;

const POLL_INTERVAL: Duration = Duration::from_millis(2);

#[derive(Clone, Copy)]
pub(super) enum CleanupScope {
    DirectChild,
    ProcessGroup,
}

pub(super) fn capture(
    path: &Path,
    arguments: &[OsString],
    context: &ExecutionContext<'_>,
    limits: OutputLimits,
) -> Result<ExecutionEvidence, ExecutionError> {
    let mut command = Command::new(path);
    command
        .args(arguments)
        .current_dir(context.current_dir)
        .env_clear()
        .envs(context.environment.iter().map(|(key, value)| (key, value)));
    capture_command(&mut command, limits, CleanupScope::DirectChild)
}

pub(super) fn capture_command(
    command: &mut Command,
    limits: OutputLimits,
    scope: CleanupScope,
) -> Result<ExecutionEvidence, ExecutionError> {
    let started = Instant::now();
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let spawned = match scope {
        CleanupScope::DirectChild => OwnedChild::spawn_direct(command),
        CleanupScope::ProcessGroup => OwnedChild::spawn(command),
    };
    let mut child = spawned.map_err(|source| ExecutionError {
        failure: io_failure(ExecutionOperation::Spawn, source),
        evidence: ExecutionEvidence::default(),
        group_error: None,
        kill_error: None,
        wait_error: None,
    })?;
    exchange(
        &mut child,
        None,
        limits,
        started,
        SuccessfulExit::Cleanup,
        || false,
    )
}

pub(super) fn communicate(
    child: &mut OwnedChild,
    input: Option<&[u8]>,
    limits: OutputLimits,
    successful_exit: SuccessfulExit,
    cancelled: impl FnMut() -> bool,
) -> Result<ExecutionEvidence, ExecutionError> {
    exchange(
        child,
        input,
        limits,
        Instant::now(),
        successful_exit,
        cancelled,
    )
}

fn exchange(
    child: &mut OwnedChild,
    input: Option<&[u8]>,
    limits: OutputLimits,
    started: Instant,
    successful_exit: SuccessfulExit,
    mut cancelled: impl FnMut() -> bool,
) -> Result<ExecutionEvidence, ExecutionError> {
    let mut evidence = ExecutionEvidence::default();
    let mut stdin = child.take_stdin();
    let mut stdout = child.take_stdout();
    let mut stderr = child.take_stderr();
    let result = (|| {
        if !child.is_owned() {
            return Err(io_failure(
                ExecutionOperation::Wait,
                io::ErrorKind::InvalidInput.into(),
            ));
        }
        if input.is_some() && stdin.is_none() {
            return Err(io_failure(
                ExecutionOperation::StdinPipe,
                io::ErrorKind::BrokenPipe.into(),
            ));
        }
        let mut remaining = input.unwrap_or_default();
        if remaining.is_empty() {
            drop(stdin.take());
        }
        if let Some(pipe) = &stdin {
            nonblocking(pipe)?;
        }
        if let Some(pipe) = &stdout {
            nonblocking(pipe)?;
        }
        if let Some(pipe) = &stderr {
            nonblocking(pipe)?;
        }
        let mut stdout_eof = stdout.is_none();
        let mut stderr_eof = stderr.is_none();
        loop {
            if cancelled() {
                return Err(ExecutionFailure::Cancelled);
            }
            if started.elapsed() >= limits.timeout {
                return Err(ExecutionFailure::TimedOut);
            }
            let progress_in = if let Some(pipe) = &mut stdin {
                let progress = write_chunk(pipe, &mut remaining)?;
                if remaining.is_empty() {
                    drop(stdin.take());
                }
                progress
            } else {
                false
            };
            let progress_out = read_chunk(
                &mut stdout,
                &mut evidence.stdout,
                limits.stdout_bytes,
                OutputStream::Stdout,
                &mut stdout_eof,
                &mut evidence.stdout_truncated,
            )?;
            let progress_err = read_chunk(
                &mut stderr,
                &mut evidence.stderr,
                limits.stderr_bytes,
                OutputStream::Stderr,
                &mut stderr_eof,
                &mut evidence.stderr_truncated,
            )?;
            evidence.status = poll_completion(child, successful_exit)
                .map_err(|source| io_failure(ExecutionOperation::Wait, source))?;
            if let Some(status) = evidence.status
                && stdout_eof
                && stderr_eof
                && stdin.is_none()
            {
                if cancelled() {
                    return Err(ExecutionFailure::Cancelled);
                }
                // Retained success still needs timely caller admission. Ordinary
                // capture keeps its existing synchronous-cleanup exception.
                if successful_exit == SuccessfulExit::Retain && started.elapsed() >= limits.timeout
                {
                    return Err(ExecutionFailure::TimedOut);
                }
                return if status.success() {
                    Ok(())
                } else {
                    Err(ExecutionFailure::ExitStatus)
                };
            }
            if !progress_in && !progress_out && !progress_err {
                std::thread::sleep(
                    POLL_INTERVAL.min(limits.timeout.saturating_sub(started.elapsed())),
                );
            }
        }
    })();
    // Close all pipes before cleanup. A descendant retaining a writer must not
    // cause a blocking drain or leave background reader threads after a timeout.
    drop(stdin);
    drop(stdout);
    drop(stderr);
    finish_capture(child, evidence, result)
}

fn poll_completion(
    child: &mut OwnedChild,
    successful_exit: SuccessfulExit,
) -> io::Result<Option<ExitStatus>> {
    if successful_exit == SuccessfulExit::Retain {
        child.poll_exit().and_then(|status| {
            if status.is_some_and(|status| !status.success()) {
                child.try_wait()
            } else {
                Ok(status)
            }
        })
    } else {
        child.try_wait()
    }
}

fn finish_capture(
    child: &mut OwnedChild,
    mut evidence: ExecutionEvidence,
    result: Result<(), ExecutionFailure>,
) -> Result<ExecutionEvidence, ExecutionError> {
    match result {
        Ok(()) => Ok(evidence),
        Err(failure) => {
            let (group_error, kill_error, wait_error) = match child.terminate() {
                Ok(status) => {
                    evidence.status = Some(status);
                    (None, None, None)
                }
                Err(error) => {
                    evidence.status = error.status;
                    (error.group_error, error.kill_error, error.wait_error)
                }
            };
            Err(ExecutionError {
                failure,
                evidence,
                group_error,
                kill_error,
                wait_error,
            })
        }
    }
}

fn write_chunk(writer: &mut impl Write, remaining: &mut &[u8]) -> Result<bool, ExecutionFailure> {
    let count = match writer.write(&remaining[..remaining.len().min(16 * 1024)]) {
        Ok(0) => {
            return Err(io_failure(
                ExecutionOperation::WriteInput,
                io::ErrorKind::WriteZero.into(),
            ));
        }
        Ok(count) => count,
        // Commands may deliberately stop reading and still supply useful status
        // and diagnostics. Closing early is not proof the input was consumed.
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {
            *remaining = &[];
            return Ok(true);
        }
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
            ) =>
        {
            return Ok(false);
        }
        Err(source) => return Err(io_failure(ExecutionOperation::WriteInput, source)),
    };
    *remaining = &remaining[count..];
    Ok(true)
}

fn nonblocking(pipe: &impl AsFd) -> Result<(), ExecutionFailure> {
    let flags = fcntl_getfl(pipe)
        .map_err(|source| io_failure(ExecutionOperation::ReadPipeFlags, source.into()))?;
    fcntl_setfl(pipe, flags | OFlags::NONBLOCK)
        .map_err(|source| io_failure(ExecutionOperation::SetPipeFlags, source.into()))
}

fn read_chunk(
    reader: &mut Option<impl Read>,
    bytes: &mut Vec<u8>,
    limit: usize,
    stream: OutputStream,
    eof: &mut bool,
    truncated: &mut bool,
) -> Result<bool, ExecutionFailure> {
    let Some(reader) = reader else {
        return Ok(false);
    };
    if *eof {
        return Ok(false);
    }
    let mut buffer = [0; 16 * 1024];
    let allowance = (limit - bytes.len()).saturating_add(1).min(buffer.len());
    let count = match reader.read(&mut buffer[..allowance]) {
        Ok(0) => {
            *eof = true;
            return Ok(false);
        }
        Ok(count) => count,
        Err(source)
            if matches!(
                source.kind(),
                io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
            ) =>
        {
            return Ok(false);
        }
        Err(source) => return Err(io_failure(ExecutionOperation::ReadOutput, source)),
    };
    let retained = count.min(limit - bytes.len());
    bytes
        .try_reserve_exact(retained)
        .map_err(|source| ExecutionFailure::Allocation { stream, source })?;
    bytes.extend_from_slice(&buffer[..retained]);
    if retained != count {
        *truncated = true;
        return Err(ExecutionFailure::OutputLimit { stream });
    }
    Ok(true)
}

const fn io_failure(operation: ExecutionOperation, source: io::Error) -> ExecutionFailure {
    ExecutionFailure::Io { operation, source }
}
