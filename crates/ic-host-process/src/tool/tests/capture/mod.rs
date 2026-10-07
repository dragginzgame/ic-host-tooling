use super::*;
use std::{
    os::unix::process::CommandExt as _,
    process::{Command, Stdio},
};

#[test]
fn caller_command_keeps_arguments_environment_and_directory_without_a_version_probe() {
    let fixture = Fixture::new();
    let directory = fixture.root.join("working directory");
    fs::create_dir(&directory).unwrap();
    let marker = fixture.root.join("invocations");
    let mut command = Command::new(tool_path());
    command
        .current_dir(&directory)
        .env_clear()
        .env("FIXTURE_MARKER", &marker)
        .env("FIXTURE_VERSION_MODE", "fail")
        .arg("--cwd");
    let output = capture_command(&mut command, LIMITS).unwrap();
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap().trim_end(),
        directory.to_str().unwrap()
    );
    assert_eq!(fs::read(&marker).unwrap(), b"invoked\n");

    let mut command = Command::new(tool_path());
    command
        .env_clear()
        .env("FIXTURE_VALUE", "caller selected")
        .env("HOME", "removed")
        .env_remove("HOME")
        .arg("--environment");
    assert_eq!(
        capture_command(&mut command, LIMITS).unwrap().stdout,
        b"caller selected|absent"
    );

    // Ambient inheritance is an explicit property of this caller's Command.
    let mut command = Command::new(tool_path());
    command
        .arg("--environment")
        .env("FIXTURE_VALUE", "selected");
    let mut expected = b"selected|".to_vec();
    expected.extend_from_slice(
        std::env::var_os("HOME")
            .unwrap_or_else(|| "absent".into())
            .as_bytes(),
    );
    assert_eq!(
        capture_command(&mut command, LIMITS).unwrap().stdout,
        expected
    );

    let argument = OsString::from_vec(b"literal ; $(unused)\n\xff".to_vec());
    let mut command = Command::new(tool_path());
    command.arg("--arguments").arg(&argument);
    let mut expected = vec![b'['];
    expected.extend_from_slice(argument.as_bytes());
    expected.push(b']');
    assert_eq!(
        capture_command(&mut command, LIMITS).unwrap().stdout,
        expected
    );

    let mut command = Command::new(tool_path());
    command
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::null());
    assert_eq!(
        capture_command(&mut command, LIMITS).unwrap().stdout,
        b"stdin closed"
    );

    // Reconstructing a Command from its visible arguments would discard this
    // platform setting and any opaque child setup hooks.
    let mut command = Command::new("/bin/sh");
    command
        .arg0("caller-selected-argv0")
        .args(["-c", "printf '%s' \"$0\""]);
    assert_eq!(
        capture_command(&mut command, LIMITS).unwrap().stdout,
        b"caller-selected-argv0"
    );
}

#[test]
fn caller_capture_retains_bounded_failures_and_runs_only_once() {
    let fixture = Fixture::new();
    let marker = fixture.root.join("invocations");
    let mut command = Command::new(tool_path());
    command.arg("--fail").env("FIXTURE_MARKER", &marker);
    let error = capture_command(&mut command, LIMITS).unwrap_err();
    let execution = error.execution_error().unwrap();
    assert!(matches!(execution.failure, ExecutionFailure::ExitStatus));
    assert_eq!(execution.evidence.status.unwrap().code(), Some(23));
    assert_eq!(execution.evidence.stderr, b"failure stderr");
    assert_eq!(fs::read(marker).unwrap(), b"invoked\n");
    for (argument, stream) in [
        ("--flood-stdout", OutputStream::Stdout),
        ("--flood-stderr", OutputStream::Stderr),
    ] {
        let mut command = Command::new(tool_path());
        command.arg(argument);
        let error = capture_command(
            &mut command,
            OutputLimits {
                stdout_bytes: 17,
                stderr_bytes: 17,
                ..LIMITS
            },
        )
        .unwrap_err();
        let execution = error.execution_error().unwrap();
        assert!(
            matches!(execution.failure, ExecutionFailure::OutputLimit { stream: actual } if actual == stream)
        );
        assert!(execution.evidence.status.is_some());
        assert!(execution.kill_error.is_none() && execution.wait_error.is_none());
        let evidence = error.evidence().unwrap();
        assert!(evidence.stdout.len() <= 17 && evidence.stderr.len() <= 17);
        assert_eq!(evidence.stdout_truncated, stream == OutputStream::Stdout);
        assert_eq!(evidence.stderr_truncated, stream == OutputStream::Stderr);
    }
}

#[test]
fn caller_deadlines_reap_children_and_do_not_wait_for_descendant_pipe_eof() {
    let _fixture = Fixture::new();
    for argument in ["--wait", "--closed-wait", "--descendant"] {
        let mut command = Command::new(tool_path());
        command.arg(argument);
        let error = capture_command(
            &mut command,
            OutputLimits {
                timeout: Duration::from_millis(100),
                ..LIMITS
            },
        )
        .unwrap_err();
        let execution = error.execution_error().unwrap();
        assert!(matches!(execution.failure, ExecutionFailure::TimedOut));
        assert!(execution.evidence.status.is_some());
        assert!(execution.kill_error.is_none() && execution.wait_error.is_none());
        if argument == "--descendant" {
            assert!(execution.evidence.status.unwrap().success());
        }
    }
}

#[test]
fn caller_validation_and_failed_spawn_preserve_their_distinct_evidence() {
    let fixture = Fixture::new();
    let marker = fixture.root.join("invocations");
    let mut command = Command::new(tool_path());
    command.arg("--version").env("FIXTURE_MARKER", &marker);
    for timeout in [Duration::ZERO, Duration::MAX] {
        let error = capture_command(&mut command, OutputLimits { timeout, ..LIMITS }).unwrap_err();
        assert!(matches!(
            error,
            ToolError::InvalidInvocation(InvalidInvocation::Deadline)
        ));
        assert!(error.evidence().is_none());
        assert!(!marker.exists());
    }
    let mut command = Command::new(fixture.root.join("not installed"));
    let error = capture_command(&mut command, LIMITS).unwrap_err();
    assert!(matches!(&error.execution_error().unwrap().failure,
        ExecutionFailure::Io { operation: ExecutionOperation::Spawn, source }
            if source.kind() == io::ErrorKind::NotFound));
    let evidence = error.evidence().unwrap();
    assert!(evidence.status.is_none() && evidence.stdout.is_empty() && evidence.stderr.is_empty());
}
