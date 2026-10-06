use super::{
    ExecutionContext, ExecutionError, ExecutionEvidence, ExecutionFailure, ExecutionOperation,
    OutputLimits, OutputStream,
};
use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
use std::{
    ffi::OsString,
    io::{self, Read},
    os::fd::AsFd,
    path::Path,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

const POLL_INTERVAL: Duration = Duration::from_millis(2);

pub(super) fn capture(
    path: &Path,
    arguments: &[OsString],
    context: &ExecutionContext<'_>,
    limits: OutputLimits,
) -> Result<ExecutionEvidence, ExecutionError> {
    let started = Instant::now();
    let mut child = Command::new(path)
        .args(arguments)
        .current_dir(context.current_dir)
        .env_clear()
        .envs(context.environment.iter().map(|(key, value)| (key, value)))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| ExecutionError {
            failure: io_failure(ExecutionOperation::Spawn, source),
            evidence: ExecutionEvidence::default(),
            kill_error: None,
            wait_error: None,
        })?;
    let mut evidence = ExecutionEvidence::default();
    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();
    let result = (|| {
        let stdout = stdout.as_mut().ok_or_else(|| {
            io_failure(
                ExecutionOperation::StdoutPipe,
                io::ErrorKind::BrokenPipe.into(),
            )
        })?;
        let stderr = stderr.as_mut().ok_or_else(|| {
            io_failure(
                ExecutionOperation::StderrPipe,
                io::ErrorKind::BrokenPipe.into(),
            )
        })?;
        nonblocking(stdout)?;
        nonblocking(stderr)?;
        let mut stdout_eof = false;
        let mut stderr_eof = false;
        loop {
            if started.elapsed() >= limits.timeout {
                return Err(ExecutionFailure::TimedOut);
            }
            let progress_out = read_chunk(
                stdout,
                &mut evidence.stdout,
                limits.stdout_bytes,
                OutputStream::Stdout,
                &mut stdout_eof,
                &mut evidence.stdout_truncated,
            )?;
            let progress_err = read_chunk(
                stderr,
                &mut evidence.stderr,
                limits.stderr_bytes,
                OutputStream::Stderr,
                &mut stderr_eof,
                &mut evidence.stderr_truncated,
            )?;
            evidence.status = child
                .try_wait()
                .map_err(|source| io_failure(ExecutionOperation::Wait, source))?;
            if let Some(status) = evidence.status
                && stdout_eof
                && stderr_eof
            {
                return if status.success() {
                    Ok(())
                } else {
                    Err(ExecutionFailure::ExitStatus)
                };
            }
            if !progress_out && !progress_err {
                std::thread::sleep(
                    POLL_INTERVAL.min(limits.timeout.saturating_sub(started.elapsed())),
                );
            }
        }
    })();
    // Close both pipes before cleanup. A descendant retaining a writer must not
    // cause a blocking drain or leave background reader threads after a timeout.
    drop(stdout);
    drop(stderr);
    match result {
        Ok(()) => Ok(evidence),
        Err(failure) => {
            let (kill_error, wait_error) = reap(&mut child, &mut evidence);
            Err(ExecutionError {
                failure,
                evidence,
                kill_error,
                wait_error,
            })
        }
    }
}

fn nonblocking(pipe: &impl AsFd) -> Result<(), ExecutionFailure> {
    let flags = fcntl_getfl(pipe)
        .map_err(|source| io_failure(ExecutionOperation::ReadPipeFlags, source.into()))?;
    fcntl_setfl(pipe, flags | OFlags::NONBLOCK)
        .map_err(|source| io_failure(ExecutionOperation::SetPipeFlags, source.into()))
}

fn read_chunk(
    reader: &mut impl Read,
    bytes: &mut Vec<u8>,
    limit: usize,
    stream: OutputStream,
    eof: &mut bool,
    truncated: &mut bool,
) -> Result<bool, ExecutionFailure> {
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

fn reap(
    child: &mut Child,
    evidence: &mut ExecutionEvidence,
) -> (Option<io::Error>, Option<io::Error>) {
    if evidence.status.is_some() {
        return (None, None);
    }
    let kill_error = child.kill().err();
    let wait_error = match child.wait() {
        Ok(status) => {
            evidence.status = Some(status);
            None
        }
        Err(source) => Some(source),
    };
    (kill_error, wait_error)
}

const fn io_failure(operation: ExecutionOperation, source: io::Error) -> ExecutionFailure {
    ExecutionFailure::Io { operation, source }
}
