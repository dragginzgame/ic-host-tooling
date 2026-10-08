use super::*;
use crate::child::OwnedChild;
use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};
use std::{process::Stdio, time::Instant};

fn command(script: &str) -> Command {
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", script])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

fn assert_reaped(child: &OwnedChild) {
    let pid = Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap();
    assert_eq!(
        waitid(
            WaitId::Pid(pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG
        )
        .unwrap_err(),
        rustix::io::Errno::CHILD
    );
}

#[test]
fn full_duplex_io_is_bounded_fair_and_leaves_success_reserved() {
    let _fixture = Fixture::new();
    // Fill both output pipes before reading input larger than a pipe capacity.
    let mut command = command(
        "n=0; while [ \"$n\" -lt 10000 ]; do printf 0123456789abcdef; \
         printf fedcba9876543210 >&2; n=$((n + 1)); done; cat",
    );
    let mut child = OwnedChild::spawn(&mut command).unwrap();
    let input = vec![b'x'; 256 * 1024];
    let evidence = communicate_child(
        &mut child,
        Some(&input),
        OutputLimits {
            stdout_bytes: 160_000 + input.len(),
            ..LIMITS
        },
        SuccessfulExit::Retain,
        || false,
    )
    .unwrap();
    assert_eq!(
        &evidence.stdout[..160_000],
        b"0123456789abcdef".repeat(10_000)
    );
    assert_eq!(&evidence.stdout[160_000..], input);
    assert_eq!(evidence.stderr, b"fedcba9876543210".repeat(10_000));
    assert!(evidence.status.unwrap().success());
    let pid = Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap();
    assert!(
        waitid(
            WaitId::Pid(pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOWAIT
        )
        .unwrap()
        .is_some()
    );
    assert!(child.wait().unwrap().success());
    assert_reaped(&child);
}

#[test]
fn absent_input_closes_piped_stdin_and_caller_io_is_preserved() {
    let fixture = Fixture::new();
    for input in [None, Some(&b""[..])] {
        let mut child = OwnedChild::spawn(&mut command("cat; printf eof")).unwrap();
        assert_eq!(
            communicate_child(&mut child, input, LIMITS, SuccessfulExit::Retain, || false)
                .unwrap()
                .stdout,
            b"eof"
        );
        child.wait().unwrap();
    }
    let input_path = fixture.root.join("input");
    let output_path = fixture.root.join("output");
    fs::write(&input_path, b"caller-selected input").unwrap();
    let mut command = command("cat >&2");
    command
        .stdin(fs::File::open(input_path).unwrap())
        .stdout(Stdio::inherit())
        .stderr(fs::File::create(&output_path).unwrap());
    let mut child = OwnedChild::spawn(&mut command).unwrap();
    let evidence = communicate_child(
        &mut child,
        None,
        OutputLimits {
            stdout_bytes: 0,
            stderr_bytes: 0,
            ..LIMITS
        },
        SuccessfulExit::Retain,
        || false,
    )
    .unwrap();
    child.wait().unwrap();
    assert!(evidence.stdout.is_empty() && evidence.stderr.is_empty());
    assert_eq!(fs::read(output_path).unwrap(), b"caller-selected input");
}

#[test]
fn early_stdin_close_preserves_command_status_and_diagnostics() {
    let _fixture = Fixture::new();
    for code in [0, 23] {
        let mut child = OwnedChild::spawn(&mut command(&format!(
            "exec 0<&-; printf diagnostic >&2; exit {code}"
        )))
        .unwrap();
        let input = vec![b'x'; 1024 * 1024];
        let result = communicate_child(
            &mut child,
            Some(&input),
            LIMITS,
            SuccessfulExit::Retain,
            || false,
        );
        let evidence = if code == 0 {
            let evidence = result.unwrap();
            child.wait().unwrap();
            evidence
        } else {
            let ToolError::Execution(error) = result.unwrap_err() else {
                panic!("execution failure")
            };
            assert!(matches!(error.failure, ExecutionFailure::ExitStatus));
            error.evidence
        };
        assert_eq!(evidence.stderr, b"diagnostic");
        assert_eq!(evidence.status.unwrap().code(), Some(code));
        assert_reaped(&child);
    }
}

#[test]
fn cancellation_interrupts_blocked_input_and_reaps_the_group() {
    let fixture = Fixture::new();
    let ready = fixture.root.join("ready");
    let go = fixture.root.join("go");
    let late = fixture.root.join("late");
    let mut command = command(
        "(printf ready > \"$READY\"; while ! test -f \"$GO\"; do sleep 0.01; done; \
         printf late > \"$LATE\") & wait",
    );
    command
        .env("READY", &ready)
        .env("GO", &go)
        .env("LATE", &late);
    let mut child = OwnedChild::spawn(&mut command).unwrap();
    let input = vec![b'x'; 1024 * 1024];
    let result = communicate_child(
        &mut child,
        Some(&input),
        LIMITS,
        SuccessfulExit::Cleanup,
        || ready.exists(),
    );
    fs::write(go, b"go").unwrap();
    let error = result.unwrap_err();
    let execution = error.execution_error().unwrap();
    assert!(matches!(execution.failure, ExecutionFailure::Cancelled));
    assert!(execution.evidence.status.is_some());
    assert!(
        execution.group_error.is_none()
            && execution.kill_error.is_none()
            && execution.wait_error.is_none()
    );
    assert_reaped(&child);
    std::thread::sleep(Duration::from_millis(100));
    assert!(!late.exists());
}

#[test]
fn successful_background_handoff_requires_explicit_post_io_admission() {
    let fixture = Fixture::new();
    for handoff in [false, true] {
        let ready = fixture.root.join(format!("ready-{handoff}"));
        let go = fixture.root.join(format!("go-{handoff}"));
        let late = fixture.root.join(format!("late-{handoff}"));
        let mut command = command(
            "(printf ready > \"$READY\"; while ! test -f \"$GO\"; do sleep 0.01; done; \
             printf late > \"$LATE\") >/dev/null 2>&1 & \
             while ! test -f \"$READY\"; do sleep 0.01; done; printf '\\377'",
        );
        command
            .env("READY", ready)
            .env("GO", &go)
            .env("LATE", &late);
        let mut child = OwnedChild::spawn(&mut command).unwrap();
        let evidence =
            communicate_child(&mut child, None, LIMITS, SuccessfulExit::Retain, || false).unwrap();
        assert!(std::str::from_utf8(&evidence.stdout).is_err());
        if handoff {
            // A binary-output consumer can admit these bytes and transfer lifecycle.
            child.handoff().unwrap();
        } else {
            // A text consumer rejects the same output with cleanup still armed.
            child.terminate().unwrap();
        }
        assert_reaped(&child);
        let error = communicate_child(&mut child, None, LIMITS, SuccessfulExit::Retain, || false)
            .unwrap_err();
        assert!(matches!(&error.execution_error().unwrap().failure,
            ExecutionFailure::Io { operation: ExecutionOperation::Wait, source }
            if source.kind() == io::ErrorKind::InvalidInput));
        drop(child);
        fs::write(go, b"go").unwrap();
        if handoff {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !late.exists() {
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(2));
            }
        } else {
            std::thread::sleep(Duration::from_millis(100));
            assert!(!late.exists());
        }
    }
}

#[test]
fn held_pipe_deadline_and_overflow_keep_cleanup_armed_after_leader_exit() {
    let _fixture = Fixture::new();
    for (script, overflow) in [
        ("sleep 30 & exit 0", false),
        ("printf 12345; sleep 30 & exit 0", true),
    ] {
        let mut child = OwnedChild::spawn(&mut command(script)).unwrap();
        let error = communicate_child(
            &mut child,
            None,
            OutputLimits {
                stdout_bytes: 4,
                timeout: Duration::from_secs(1),
                ..LIMITS
            },
            SuccessfulExit::Retain,
            || false,
        )
        .unwrap_err();
        let execution = error.execution_error().unwrap();
        if overflow {
            assert!(matches!(
                execution.failure,
                ExecutionFailure::OutputLimit {
                    stream: OutputStream::Stdout
                }
            ));
            assert_eq!(execution.evidence.stdout, b"1234");
        } else {
            assert!(matches!(execution.failure, ExecutionFailure::TimedOut));
        }
        assert!(execution.group_error.is_none() && execution.wait_error.is_none());
        assert_reaped(&child);
    }
}

#[test]
fn invalid_deadline_retains_owner_and_missing_input_pipe_cleans_it() {
    let _fixture = Fixture::new();
    let mut child = OwnedChild::spawn(&mut command("cat")).unwrap();
    for timeout in [Duration::ZERO, Duration::MAX] {
        assert!(matches!(
            communicate_child(
                &mut child,
                None,
                OutputLimits { timeout, ..LIMITS },
                SuccessfulExit::Retain,
                || { false }
            ),
            Err(ToolError::InvalidInvocation(InvalidInvocation::Deadline))
        ));
    }
    assert_eq!(
        communicate_child(
            &mut child,
            Some(b"still available"),
            LIMITS,
            SuccessfulExit::Retain,
            || false
        )
        .unwrap()
        .stdout,
        b"still available"
    );
    child.wait().unwrap();

    let mut command = command("sleep 30");
    command.stdin(Stdio::null());
    let mut child = OwnedChild::spawn(&mut command).unwrap();
    let error = communicate_child(
        &mut child,
        Some(b""),
        LIMITS,
        SuccessfulExit::Retain,
        || false,
    )
    .unwrap_err();
    assert!(matches!(
        error.execution_error().unwrap().failure,
        ExecutionFailure::Io {
            operation: ExecutionOperation::StdinPipe,
            ..
        }
    ));
    assert_reaped(&child);
}

#[test]
fn cancellation_is_checked_again_before_returning_reserved_success() {
    let _fixture = Fixture::new();
    let mut child = OwnedChild::spawn(&mut command("exit 0")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while child.poll_exit().unwrap().is_none() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    let mut polls = 0;
    let error = communicate_child(&mut child, None, LIMITS, SuccessfulExit::Retain, || {
        polls += 1;
        polls > 1
    })
    .unwrap_err();
    assert!(matches!(
        error.execution_error().unwrap().failure,
        ExecutionFailure::Cancelled
    ));
    assert!(error.evidence().unwrap().status.unwrap().success());
    assert_reaped(&child);
    assert!(child.handoff().is_err());
}
