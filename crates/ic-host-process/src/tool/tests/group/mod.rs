use super::*;

#[test]
fn cancelled_group_capture_refuses_before_spawn_and_reaps_running_group() {
    let fixture = Fixture::new();
    let ready = fixture.root.join("cancel-ready");
    let mut command = Command::new("/bin/sh");
    command
        .args([
            "-c",
            "printf output; printf diagnostic >&2; printf ready > \"$READY\"; exec sleep 30",
        ])
        .env("READY", &ready);
    let error = capture_group_command_with_cancellation(
        &mut command,
        LIMITS,
        crate::child::CleanupPolicy::KillAndWait,
        None,
        || true,
    )
    .unwrap_err();
    assert!(matches!(
        error.execution_error().unwrap().failure,
        ExecutionFailure::Cancelled
    ));
    assert!(error.evidence().unwrap().status.is_none());
    assert!(!ready.exists());
    let error = capture_group_command_with_cancellation(
        &mut command,
        LIMITS,
        crate::child::CleanupPolicy::KillAndWait,
        None,
        || ready.exists(),
    )
    .unwrap_err();
    let execution = error.execution_error().unwrap();
    assert!(matches!(execution.failure, ExecutionFailure::Cancelled));
    assert!(!execution.evidence.status.unwrap().success());
    assert!(execution.cleanup.is_none());
    assert!(b"output".starts_with(&execution.evidence.stdout));
    assert!(b"diagnostic".starts_with(&execution.evidence.stderr));
}
use std::{process::Command, time::Instant};

#[test]
fn controlled_group_capture_uses_earlier_bound_and_refuses_expired_startup() {
    use crate::child::CleanupPolicy;

    let fixture = Fixture::new();
    let marker = fixture.root.join("expired-command-ran");
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf ran > \"$MARKER\""])
        .env("MARKER", &marker);
    let error = capture_group_command_with_cancellation(
        &mut command,
        LIMITS,
        CleanupPolicy::KillAndWait,
        Some(Instant::now()),
        || false,
    )
    .unwrap_err();
    assert!(matches!(
        error.execution_error().unwrap().failure,
        ExecutionFailure::TimedOut
    ));
    assert!(error.evidence().unwrap().status.is_none());
    assert!(!marker.exists());

    for absolute_first in [false, true] {
        let short = Duration::from_millis(25);
        let long = Duration::from_secs(30);
        let started = Instant::now();
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "exec sleep 30"]);
        let error = capture_group_command_with_cancellation(
            &mut command,
            OutputLimits {
                timeout: Some(if absolute_first { long } else { short }),
                ..LIMITS
            },
            CleanupPolicy::KillAndWait,
            Some(started + if absolute_first { short } else { long }),
            || started.elapsed() > Duration::from_secs(5),
        )
        .unwrap_err();
        let execution = error.execution_error().unwrap();
        assert!(matches!(execution.failure, ExecutionFailure::TimedOut));
        assert!(execution.evidence.status.is_some());
        assert!(execution.cleanup.is_none());
    }
}

#[test]
fn controlled_group_capture_honors_caller_term_cleanup() {
    let fixture = Fixture::new();
    let ready = fixture.root.join("term-ready");
    let marker = fixture.root.join("term-observed");
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "trap 'printf term > \"$TERM_MARKER\"; exit 0' TERM; sh -c 'trap \"\" TERM; printf ready > \"$READY\"; exec sleep 30' & wait"])
        .env("READY", &ready).env("TERM_MARKER", &marker);
    let error = capture_group_command_with_cancellation(
        &mut command,
        LIMITS,
        crate::child::CleanupPolicy::TermThenKill {
            grace: Duration::from_secs(1),
            reap_timeout: Duration::from_secs(1),
        },
        None,
        || ready.exists(),
    )
    .unwrap_err();
    assert!(matches!(
        error.execution_error().unwrap().failure,
        ExecutionFailure::Cancelled
    ));
    assert!(error.execution_error().unwrap().cleanup.is_none());
    assert_eq!(fs::read(marker).unwrap(), b"term");
}

#[test]
fn group_capture_cleans_wrapper_descendants_on_exit_timeout_and_overflow() {
    let fixture = Fixture::new();
    for (mode, ending) in [
        ("success", "printf output; printf diagnostic >&2; exit 0"),
        ("failure", "printf output; printf diagnostic >&2; exit 17"),
        ("timeout", "printf output; printf diagnostic >&2; wait"),
        ("stdout", "while :; do printf 0123456789abcdef; done"),
        ("stderr", "while :; do printf 0123456789abcdef >&2; done"),
    ] {
        let ready = fixture.root.join(format!("{mode}-ready"));
        let go = fixture.root.join(format!("{mode}-go"));
        let late = fixture.root.join(format!("{mode}-late"));
        let script = format!(
            "(printf ready > \"$READY\"; while ! test -f \"$GO\"; do sleep 0.01; done; printf late > \"$LATE\") & \
             while ! test -f \"$READY\"; do sleep 0.01; done; {ending}"
        );
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", &script])
            .env("READY", &ready)
            .env("GO", &go)
            .env("LATE", &late);
        let output = capture_group_command(
            &mut command,
            OutputLimits {
                stdout: crate::tool::OutputLimit::Terminate(32),
                stderr: crate::tool::OutputLimit::Terminate(32),
                timeout: Some(Duration::from_secs(1)),
            },
        );
        // Release any surviving fixture before asserting, including on regression.
        fs::write(&go, b"go").unwrap();
        assert!(
            ready.exists(),
            "descendant must have started before capture ended"
        );
        if mode == "success" || mode == "failure" {
            let evidence = output.unwrap();
            assert_eq!(evidence.stdout, b"output");
            assert_eq!(evidence.stderr, b"diagnostic");
            assert_eq!(
                evidence.status.code(),
                Some(if mode == "success" { 0 } else { 17 })
            );
        } else {
            let error = output.unwrap_err();
            let execution = error.execution_error().unwrap();
            if mode == "timeout" {
                assert!(matches!(execution.failure, ExecutionFailure::TimedOut));
            } else {
                let expected = if mode == "stdout" {
                    OutputStream::Stdout
                } else {
                    OutputStream::Stderr
                };
                assert!(
                    matches!(execution.failure, ExecutionFailure::OutputLimit { stream } if stream == expected)
                );
            }
            assert!(execution.cleanup.is_none());
            assert!(execution.evidence.status.is_some());
            assert!(execution.evidence.stdout.len() <= 32);
            assert!(execution.evidence.stderr.len() <= 32);
        }
        std::thread::sleep(Duration::from_millis(100));
        assert!(
            !late.exists(),
            "same-group descendant outlived {mode} cleanup"
        );
    }
}

#[test]
fn group_capture_retains_fair_streams_and_refuses_invalid_deadlines_before_spawn() {
    let fixture = Fixture::new();
    let marker = fixture.root.join("calls");
    let mut command = Command::new(tool_path());
    command.arg("--both").env("FIXTURE_MARKER", &marker);
    let evidence = capture_group_command(&mut command, LIMITS).unwrap();
    assert_eq!(evidence.stdout.len(), 160_000);
    assert_eq!(evidence.stderr.len(), 160_000);
    assert_eq!(fs::read(&marker).unwrap(), b"invoked\n");
    for timeout in [Duration::ZERO, Duration::MAX] {
        let error = capture_group_command(
            &mut command,
            OutputLimits {
                timeout: Some(timeout),
                ..LIMITS
            },
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ToolError::InvalidInvocation(InvalidInvocation::Deadline)
        ));
    }
    assert_eq!(fs::read(&marker).unwrap(), b"invoked\n");
    let error =
        capture_group_command(&mut Command::new(fixture.root.join("missing")), LIMITS).unwrap_err();
    let execution = error.execution_error().unwrap();
    assert!(matches!(
        execution.failure,
        ExecutionFailure::Io {
            operation: ExecutionOperation::Spawn,
            ..
        }
    ));
    assert!(execution.evidence.status.is_none());
    assert!(execution.cleanup.is_none());
}

#[test]
fn direct_capture_still_leaves_descendant_lifetime_to_the_caller() {
    let fixture = Fixture::new();
    let marker = fixture.root.join("direct-descendant");
    let mut command = Command::new("/bin/sh");
    command
        .args([
            "-c",
            "(sleep 0.1; printf late > \"$MARKER\") >/dev/null 2>&1 & exit 0",
        ])
        .env("MARKER", &marker);
    assert!(
        capture_command(&mut command, LIMITS)
            .unwrap()
            .status
            .success()
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while !marker.exists() {
        assert!(
            Instant::now() < deadline,
            "direct capture changed descendant lifetime"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn configured_group_execution_preserves_file_output_and_pipe_evidence() {
    use crate::child::CleanupPolicy;
    use std::process::Stdio;

    let fixture = Fixture::new();
    let path = fixture.root.join("configured-stdout");
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf file-output; printf diagnostic >&2; exit 23"])
        .stdin(Stdio::piped())
        .stdout(fs::File::create(&path).unwrap())
        .stderr(Stdio::piped());
    let completed = run_group_command(
        &mut command,
        OutputLimits {
            stdout: OutputLimit::Terminate(0),
            stderr: OutputLimit::Terminate(10),
            timeout: None,
        },
        CleanupPolicy::KillAndWait,
        None,
        || false,
    )
    .unwrap();
    assert_eq!(completed.status.code(), Some(23));
    assert_eq!(completed.stdout, b"");
    assert_eq!(completed.stderr, b"diagnostic");
    assert_eq!(fs::read(path).unwrap(), b"file-output");
    assert!(matches!(
        completed
            .require_success()
            .unwrap_err()
            .execution_error()
            .unwrap()
            .failure,
        ExecutionFailure::ExitStatus
    ));
}

#[test]
fn configured_group_refusal_preserves_the_original_command_io() {
    use crate::child::CleanupPolicy;
    use std::process::Stdio;

    let fixture = Fixture::new();
    let path = fixture.root.join("original-stdout");
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "cat; printf original"])
        .stdin(Stdio::piped())
        .stdout(fs::File::create(&path).unwrap())
        .stderr(Stdio::null());
    for (timeout, deadline, cancelled) in [
        (Some(Duration::ZERO), None, false),
        (None, None, true),
        (None, Some(Instant::now()), false),
    ] {
        let error = run_group_command(
            &mut command,
            OutputLimits { timeout, ..LIMITS },
            CleanupPolicy::KillAndWait,
            deadline,
            || cancelled,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ToolError::InvalidInvocation(InvalidInvocation::Deadline) | ToolError::Execution(_)
        ));
        assert_eq!(fs::read(&path).unwrap(), b"");
    }
    let completed = run_group_command(
        &mut command,
        OutputLimits {
            timeout: None,
            ..LIMITS
        },
        CleanupPolicy::KillAndWait,
        None,
        || false,
    )
    .unwrap();
    assert!(completed.status.success());
    assert_eq!(completed.stdout, b"");
    assert_eq!(fs::read(path).unwrap(), b"original");
}
