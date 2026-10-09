use super::{
    CommunicationLimits, ExecutionContext, ExecutionError, ExecutionEvidence, ExecutionFailure,
    ExecutionOperation, OutputLimits, OutputStream, SuccessfulExit,
};
use crate::child::OwnedChild;
use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
use std::{
    ffi::OsString,
    io::{self, Read, Write},
    os::fd::AsFd,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
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
        term_error: None,
        group_error: None,
        kill_error: None,
        wait_error: None,
    })?;
    exchange(
        &mut child,
        None,
        limits.into(),
        started,
        SuccessfulExit::Cleanup,
        || false,
        |_, _| {},
    )
}

pub(super) fn communicate(
    child: &mut OwnedChild,
    input: Option<&[u8]>,
    limits: CommunicationLimits,
    successful_exit: SuccessfulExit,
    cancelled: impl FnMut() -> bool,
    output: impl FnMut(OutputStream, &[u8]),
) -> Result<ExecutionEvidence, ExecutionError> {
    exchange(
        child,
        input,
        limits,
        Instant::now(),
        successful_exit,
        cancelled,
        output,
    )
}

fn exchange(
    child: &mut OwnedChild,
    input: Option<&[u8]>,
    limits: CommunicationLimits,
    started: Instant,
    successful_exit: SuccessfulExit,
    mut cancelled: impl FnMut() -> bool,
    mut output: impl FnMut(OutputStream, &[u8]),
) -> Result<ExecutionEvidence, ExecutionError> {
    let timeout = limits.timeout;
    let mut evidence = ExecutionEvidence::default();
    let mut stdin = child.take_stdin();
    let mut stdout = child.take_stdout();
    let mut stderr = child.take_stderr();
    // Resume the original panic after closing pipes and attempting cleanup.
    // The borrowed owner can outlive a caller's catch_unwind, so its Drop alone
    // cannot guarantee cleanup here. No callback is reused after unwinding.
    let result = catch_unwind(AssertUnwindSafe(|| {
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
        stdin.as_ref().map(nonblocking).transpose()?;
        stdout.as_ref().map(nonblocking).transpose()?;
        stderr.as_ref().map(nonblocking).transpose()?;
        loop {
            if cancelled() {
                return Err(ExecutionFailure::Cancelled);
            }
            if timeout.is_some_and(|timeout| started.elapsed() >= timeout) {
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
                &mut evidence.stdout_truncated,
                &mut output,
            )?;
            let progress_err = read_chunk(
                &mut stderr,
                &mut evidence.stderr,
                limits.stderr_bytes,
                OutputStream::Stderr,
                &mut evidence.stderr_truncated,
                &mut output,
            )?;
            evidence.status = poll_completion(child, successful_exit)
                .map_err(|source| io_failure(ExecutionOperation::Wait, source))?;
            if let Some(status) = evidence.status
                && stdout.is_none()
                && stderr.is_none()
                && stdin.is_none()
            {
                if cancelled() {
                    return Err(ExecutionFailure::Cancelled);
                }
                // Retained success still needs timely caller admission. Ordinary
                // capture keeps its existing synchronous-cleanup exception.
                if successful_exit == SuccessfulExit::Retain
                    && timeout.is_some_and(|timeout| started.elapsed() >= timeout)
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
                std::thread::sleep(timeout.map_or(POLL_INTERVAL, |timeout| {
                    POLL_INTERVAL.min(timeout.saturating_sub(started.elapsed()))
                }));
            }
        }
    }));
    // Close all pipes before cleanup. A descendant retaining a writer must not
    // cause a blocking drain or leave background reader threads after a timeout.
    drop(stdin);
    drop(stdout);
    drop(stderr);
    match result {
        Ok(result) => finish_capture(child, evidence, result),
        Err(panic) => {
            let _ = child.terminate();
            resume_unwind(panic)
        }
    }
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
            let (term_error, group_error, kill_error, wait_error) = match child.terminate() {
                Ok(status) => {
                    evidence.status = Some(status);
                    (None, None, None, None)
                }
                Err(error) => {
                    evidence.status = error.status;
                    (
                        error.term_error,
                        error.group_error,
                        error.kill_error,
                        error.wait_error,
                    )
                }
            };
            Err(ExecutionError {
                failure,
                evidence,
                term_error,
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
    truncated: &mut bool,
    output: &mut impl FnMut(OutputStream, &[u8]),
) -> Result<bool, ExecutionFailure> {
    let Some(pipe) = reader.as_mut() else {
        return Ok(false);
    };
    let mut buffer = [0; 16 * 1024];
    let allowance = (limit - bytes.len()).saturating_add(1).min(buffer.len());
    let count = match pipe.read(&mut buffer[..allowance]) {
        Ok(0) => {
            // A closed pipe is also the completion state; never read it again.
            drop(reader.take());
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
    let required = bytes.len() + retained;
    if required > bytes.capacity() {
        // Keep requested capacity within the stream budget while amortizing
        // small pipe reads. This crate owns process capture storage.
        let capacity = bytes.capacity().saturating_mul(2).max(required).min(limit);
        bytes
            .try_reserve_exact(capacity - bytes.len())
            .map_err(|source| ExecutionFailure::Allocation { stream, source })?;
    }
    bytes.extend_from_slice(&buffer[..retained]);
    if retained != 0 {
        output(stream, &buffer[..retained]);
    }
    if retained != count {
        *truncated = true;
        return Err(ExecutionFailure::OutputLimit { stream });
    }
    Ok(true)
}

const fn io_failure(operation: ExecutionOperation, source: io::Error) -> ExecutionFailure {
    ExecutionFailure::Io { operation, source }
}
