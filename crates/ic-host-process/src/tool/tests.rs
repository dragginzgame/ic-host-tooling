#![expect(
    clippy::significant_drop_tightening,
    reason = "fixture serialization and directory ownership must last through output/filesystem assertions"
)]

use super::*;
use crate::test_support::{Fixture, LIMITS, VERSION, admit, tool_path};
use std::os::unix::ffi::OsStringExt as _;

mod capture;
mod evidence;
mod group;
mod named;
mod version;

#[test]
fn digest_rejection_precedes_any_version_execution() {
    let fixture = Fixture::new();
    let marker = fixture.root.join("invocations");
    let environment = [(
        OsString::from("FIXTURE_MARKER"),
        marker.as_os_str().to_owned(),
    )];
    let spec = ToolSpec {
        executable: &tool_path(),
        sha256: Sha256Digest::compute(b"wrong"),
        executable_bytes: 1024 * 1024,
        version_arguments: &[OsString::from("--version")],
        version_identity: VERSION,
    };
    assert!(matches!(
        AdmittedTool::admit(&spec, &fixture.context(&environment), LIMITS),
        Err(ToolError::Artifact(ArtifactError::DigestMismatch { .. }))
    ));
    assert!(!marker.exists());
}

#[test]
fn admission_observes_exact_version_and_preserves_invalid_version_output() {
    let fixture = Fixture::new();
    let tool = fixture.tool();
    assert_eq!(tool.path(), tool_path());
    assert_eq!(
        tool.identity(),
        hash_file(&tool_path(), 1024 * 1024).unwrap()
    );
    assert_eq!(tool.version_identity(), VERSION);
    let wrong = admit(&tool_path(), &fixture.context(&[]), "different");
    assert!(
        matches!(wrong, Err(ToolError::VersionMismatch { evidence }) if evidence.stdout == format!("{VERSION}\n").as_bytes() && evidence.status.is_some_and(|status| status.success()))
    );
    let environment = [(
        OsString::from("FIXTURE_VERSION_MODE"),
        OsString::from("invalid-utf8"),
    )];
    assert!(
        matches!(admit(&tool_path(), &fixture.context(&environment), VERSION), Err(ToolError::VersionUtf8 { evidence, .. }) if evidence.stdout == [0xff])
    );
    let environment = [(
        OsString::from("FIXTURE_VERSION"),
        OsString::from(format!("  {VERSION}\t")),
    )];
    assert!(admit(&tool_path(), &fixture.context(&environment), VERSION).is_ok());
    let environment = [(
        OsString::from("FIXTURE_VERSION_MODE"),
        OsString::from("fail"),
    )];
    assert!(
        matches!(admit(&tool_path(), &fixture.context(&environment), VERSION), Err(ToolError::Execution(error)) if matches!(error.failure, ExecutionFailure::ExitStatus) && error.evidence.stderr == b"version failed")
    );
}

#[test]
fn arguments_environment_directory_and_stdin_are_explicit() {
    let fixture = Fixture::new();
    let tool = fixture.tool();
    let marker = fixture.root.join("must-not-be-created");
    let suspicious = format!("$(touch {}) ; newline\nnon-ascii:é", marker.display());
    let output = tool
        .run(
            &[
                OsString::from("--arguments"),
                OsString::from(&suspicious),
                OsString::from("a b"),
            ],
            &fixture.context(&[]),
            LIMITS,
        )
        .unwrap();
    assert_eq!(output.stdout, format!("[{suspicious}][a b]").as_bytes());
    assert!(!marker.exists());
    let output = tool
        .run(
            &[
                OsString::from("--arguments"),
                OsString::from_vec(vec![0xff]),
            ],
            &fixture.context(&[]),
            LIMITS,
        )
        .unwrap();
    assert_eq!(output.stdout, [b'[', 0xff, b']']);
    let environment = [(
        OsString::from("FIXTURE_VALUE"),
        OsString::from("selected value"),
    )];
    let output = tool
        .run(
            &[OsString::from("--environment")],
            &fixture.context(&environment),
            LIMITS,
        )
        .unwrap();
    assert_eq!(output.stdout, b"selected value|absent");
    let output = tool
        .run(&[OsString::from("--cwd")], &fixture.context(&[]), LIMITS)
        .unwrap();
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap().trim_end(),
        fixture.root.to_str().unwrap()
    );
    let output = tool
        .run(&[OsString::from("--stdin")], &fixture.context(&[]), LIMITS)
        .unwrap();
    assert_eq!(output.stdout, b"stdin closed");
}

#[test]
fn nonzero_exit_retains_bytes_without_automatic_logging_or_retry() {
    let fixture = Fixture::new();
    let tool = fixture.tool();
    let marker = fixture.root.join("invocations");
    let secret = "private credential value";
    let environment = [
        (
            OsString::from("FIXTURE_MARKER"),
            marker.as_os_str().to_owned(),
        ),
        (OsString::from("FIXTURE_SECRET"), OsString::from(secret)),
    ];
    let Err(ToolError::Execution(error)) = tool.run(
        &[OsString::from("--fail")],
        &fixture.context(&environment),
        LIMITS,
    ) else {
        panic!("expected execution failure");
    };
    assert!(matches!(error.failure, ExecutionFailure::ExitStatus));
    assert_eq!(
        error.evidence.status.and_then(|status| status.code()),
        Some(23)
    );
    assert_eq!(error.evidence.stdout, secret.as_bytes());
    assert_eq!(error.evidence.stderr, b"failure stderr");
    assert!(!format!("{error:?}").contains(secret));
    assert!(!error.to_string().contains(secret));
    assert_eq!(fs::read(marker).unwrap(), b"invoked\n");
}

#[test]
fn drains_both_streams_beyond_pipe_capacity_without_deadlock() {
    let fixture = Fixture::new();
    let output = fixture
        .tool()
        .run(&[OsString::from("--both")], &fixture.context(&[]), LIMITS)
        .unwrap();
    assert_eq!(output.stdout, b"0123456789abcdef".repeat(10_000));
    assert_eq!(output.stderr, b"fedcba9876543210".repeat(10_000));
    assert!(!output.stdout_truncated && !output.stderr_truncated);
}

#[test]
fn output_overflow_kills_and_reaps_with_bounded_prefix_evidence() {
    let fixture = Fixture::new();
    let tool = fixture.tool();
    for (argument, stream) in [
        ("--flood-stdout", OutputStream::Stdout),
        ("--flood-stderr", OutputStream::Stderr),
    ] {
        for limit in [0, 17] {
            let bounds = OutputLimits {
                stdout_bytes: limit,
                stderr_bytes: limit,
                ..LIMITS
            };
            let Err(ToolError::Execution(error)) =
                tool.run(&[OsString::from(argument)], &fixture.context(&[]), bounds)
            else {
                panic!("expected overflow");
            };
            assert!(
                matches!(error.failure, ExecutionFailure::OutputLimit { stream: actual } if actual == stream)
            );
            match stream {
                OutputStream::Stdout => {
                    assert_eq!(error.evidence.stdout.len(), limit);
                    assert!(error.evidence.stdout_truncated);
                }
                OutputStream::Stderr => {
                    assert_eq!(error.evidence.stderr.len(), limit);
                    assert!(error.evidence.stderr_truncated);
                }
            }
            assert!(error.evidence.status.is_some());
            assert!(error.kill_error.is_none() && error.wait_error.is_none());
        }
    }
}

#[test]
fn exact_output_limits_allow_complete_output() {
    let fixture = Fixture::new();
    let limits = OutputLimits {
        stdout_bytes: 3,
        stderr_bytes: 0,
        ..LIMITS
    };
    let output = fixture
        .tool()
        .run(
            &[OsString::from("--arguments"), OsString::from("x")],
            &fixture.context(&[]),
            limits,
        )
        .unwrap();
    assert_eq!(output.stdout, b"[x]");
}

#[test]
fn deadlines_cover_running_children_closed_streams_and_descendant_pipe_copies() {
    let fixture = Fixture::new();
    let tool = fixture.tool();
    let limits = OutputLimits {
        timeout: Duration::from_millis(100),
        ..LIMITS
    };
    for argument in ["--wait", "--closed-wait", "--descendant"] {
        let Err(ToolError::Execution(error)) =
            tool.run(&[OsString::from(argument)], &fixture.context(&[]), limits)
        else {
            panic!("expected deadline failure");
        };
        assert!(matches!(error.failure, ExecutionFailure::TimedOut));
        assert!(error.evidence.status.is_some());
        assert!(error.kill_error.is_none() && error.wait_error.is_none());
        if argument == "--descendant" {
            assert!(error.evidence.status.is_some_and(|status| status.success()));
        }
    }
}

#[test]
fn changed_or_nonexecutable_candidates_are_rejected_before_execution() {
    let fixture = Fixture::new();
    let candidate = fixture.root.join("tool");
    fs::copy(tool_path(), &candidate).unwrap();
    let tool = admit(&candidate, &fixture.context(&[]), VERSION).unwrap();
    fs::write(&candidate, b"#!/bin/sh\nexit 0\n").unwrap();
    assert!(matches!(
        tool.run(&[], &fixture.context(&[]), LIMITS),
        Err(ToolError::Artifact(ArtifactError::DigestMismatch { .. }))
    ));
    fs::copy(tool_path(), &candidate).unwrap();
    fs::set_permissions(&candidate, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(matches!(
        admit(&candidate, &fixture.context(&[]), VERSION),
        Err(ToolError::NotExecutable)
    ));
    assert!(matches!(
        tool.run(&[], &fixture.context(&[]), LIMITS),
        Err(ToolError::NotExecutable)
    ));
}

#[test]
fn invalid_invocations_are_typed_and_never_dispatched() {
    let fixture = Fixture::new();
    let tool = fixture.tool();
    assert!(matches!(
        tool.run(
            &[],
            &fixture.context(&[]),
            OutputLimits {
                timeout: Duration::ZERO,
                ..LIMITS
            }
        ),
        Err(ToolError::InvalidInvocation(InvalidInvocation::Deadline))
    ));
    assert!(matches!(
        tool.run(
            &[],
            &ExecutionContext {
                current_dir: Path::new("relative"),
                environment: &[]
            },
            LIMITS
        ),
        Err(ToolError::InvalidInvocation(
            InvalidInvocation::WorkingDirectory
        ))
    ));
    let arguments = [OsString::from_vec(b"has\0nul".to_vec())];
    assert!(matches!(
        tool.run(&arguments, &fixture.context(&[]), LIMITS),
        Err(ToolError::InvalidInvocation(InvalidInvocation::Argument {
            index: 0
        }))
    ));
    for environment in [
        vec![(OsString::from("a=b"), OsString::from("v"))],
        vec![
            (OsString::from("k"), OsString::from("v")),
            (OsString::from("k"), OsString::from("other")),
        ],
    ] {
        assert!(matches!(
            tool.run(&[], &fixture.context(&environment), LIMITS),
            Err(ToolError::InvalidInvocation(
                InvalidInvocation::EnvironmentName { .. }
            ))
        ));
    }
    let environment = [(OsString::from("k"), OsString::from_vec(b"a\0b".to_vec()))];
    assert!(matches!(
        tool.run(&[], &fixture.context(&environment), LIMITS),
        Err(ToolError::InvalidInvocation(
            InvalidInvocation::EnvironmentValue { index: 0 }
        ))
    ));
}

#[test]
fn unavailable_working_directory_returns_spawn_evidence_without_retry() {
    let fixture = Fixture::new();
    let tool = fixture.tool();
    let missing = fixture.root.join("missing");
    let context = ExecutionContext {
        current_dir: &missing,
        environment: &[],
    };
    let Err(ToolError::Execution(error)) = tool.run(&[], &context, LIMITS) else {
        panic!("expected spawn failure");
    };
    assert!(
        matches!(&error.failure, ExecutionFailure::Io { operation: ExecutionOperation::Spawn, source } if source.kind() == io::ErrorKind::NotFound)
    );
    assert!(error.evidence.status.is_none() && error.evidence.stdout.is_empty());
}
