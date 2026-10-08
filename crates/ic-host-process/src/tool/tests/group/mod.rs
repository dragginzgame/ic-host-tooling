use super::*;
use std::{process::Command, time::Instant};

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
                stdout_bytes: 32,
                stderr_bytes: 32,
                timeout: Duration::from_secs(1),
            },
        );
        // Release any surviving fixture before asserting, including on regression.
        fs::write(&go, b"go").unwrap();
        assert!(
            ready.exists(),
            "descendant must have started before capture ended"
        );
        if mode == "success" {
            let evidence = output.unwrap();
            assert_eq!(evidence.stdout, b"output");
            assert_eq!(evidence.stderr, b"diagnostic");
            assert!(evidence.status.unwrap().success());
        } else {
            let error = output.unwrap_err();
            let execution = error.execution_error().unwrap();
            match mode {
                "failure" => {
                    assert!(matches!(execution.failure, ExecutionFailure::ExitStatus));
                    assert_eq!(execution.evidence.status.unwrap().code(), Some(17));
                    assert_eq!(execution.evidence.stderr, b"diagnostic");
                }
                "timeout" => assert!(matches!(execution.failure, ExecutionFailure::TimedOut)),
                _ => {
                    let expected = if mode == "stdout" {
                        OutputStream::Stdout
                    } else {
                        OutputStream::Stderr
                    };
                    assert!(
                        matches!(execution.failure, ExecutionFailure::OutputLimit { stream } if stream == expected)
                    );
                }
            }
            assert!(execution.group_error.is_none());
            assert!(execution.kill_error.is_none());
            assert!(execution.wait_error.is_none());
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
        let error =
            capture_group_command(&mut command, OutputLimits { timeout, ..LIMITS }).unwrap_err();
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
    assert!(execution.group_error.is_none());
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
            .unwrap()
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
