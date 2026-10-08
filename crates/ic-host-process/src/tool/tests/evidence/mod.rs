use super::*;

#[test]
fn failures_before_capture_have_no_invented_evidence() {
    for error in [
        ToolError::InvalidInvocation(InvalidInvocation::ExecutablePath),
        ToolError::Io(io::Error::from(io::ErrorKind::PermissionDenied)),
        ToolError::NotExecutable,
        ToolError::Artifact(ArtifactError::NotRegularFile),
    ] {
        assert!(error.evidence().is_none());
        assert!(error.execution_error().is_none());
    }
}

#[test]
fn successful_version_captures_are_borrowed_after_admission_rejects_them() {
    let fixture = Fixture::new();
    for environment in [
        vec![(
            OsString::from("FIXTURE_VERSION"),
            OsString::from("different"),
        )],
        vec![(
            OsString::from("FIXTURE_VERSION_MODE"),
            OsString::from("invalid-utf8"),
        )],
    ] {
        let Err(error) = admit(&tool_path(), &fixture.context(&environment), VERSION) else {
            panic!("version admission must reject");
        };
        let retained = match &error {
            ToolError::VersionMismatch { evidence } | ToolError::VersionUtf8 { evidence, .. } => {
                evidence.as_ref()
            }
            _ => panic!("expected successful version capture"),
        };
        assert!(std::ptr::eq(error.evidence().unwrap(), retained));
        assert!(retained.status.unwrap().success());
        assert_ne!(retained.stdout, []);
        assert!(error.execution_error().is_none());
    }
}

#[test]
fn failed_version_execution_retains_the_same_error_and_capture() {
    let fixture = Fixture::new();
    let environment = [(
        OsString::from("FIXTURE_VERSION_MODE"),
        OsString::from("fail"),
    )];
    let Err(error) = admit(&tool_path(), &fixture.context(&environment), VERSION) else {
        panic!("version execution must fail");
    };
    let ToolError::Execution(retained) = &error else {
        panic!("expected execution error")
    };
    assert!(std::ptr::eq(
        error.execution_error().unwrap(),
        retained.as_ref()
    ));
    assert!(std::ptr::eq(
        error.evidence().unwrap(),
        &raw const retained.evidence
    ));
    assert_eq!(error.evidence().unwrap().stderr, b"version failed");
}

#[test]
fn borrowed_execution_keeps_original_failure_and_cleanup_errors_together() {
    // Real execution tests exercise cleanup success. Synthetic failed cleanup
    // qualifies borrowing without inducing an uncontrolled process or OS failure.
    let error = ToolError::Execution(Box::new(ExecutionError {
        failure: ExecutionFailure::TimedOut,
        evidence: ExecutionEvidence {
            stdout: b"retained prefix".to_vec(),
            stdout_truncated: true,
            ..ExecutionEvidence::default()
        },
        term_error: Some(io::ErrorKind::PermissionDenied.into()),
        group_error: Some(io::ErrorKind::PermissionDenied.into()),
        kill_error: Some(io::ErrorKind::PermissionDenied.into()),
        wait_error: Some(io::ErrorKind::Interrupted.into()),
    }));
    let ToolError::Execution(retained) = &error else {
        unreachable!()
    };
    let borrowed = error.execution_error().unwrap();
    assert!(std::ptr::eq(borrowed, retained.as_ref()));
    assert!(std::ptr::eq(
        error.evidence().unwrap(),
        &raw const borrowed.evidence
    ));
    assert!(matches!(borrowed.failure, ExecutionFailure::TimedOut));
    assert_eq!(
        borrowed.term_error.as_ref().unwrap().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(
        borrowed.group_error.as_ref().unwrap().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(
        borrowed.kill_error.as_ref().unwrap().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(
        borrowed.wait_error.as_ref().unwrap().kind(),
        io::ErrorKind::Interrupted
    );
    assert!(borrowed.evidence.stdout_truncated);
    assert_eq!(borrowed.evidence.stdout, b"retained prefix");
    assert!(!format!("{error:?}").contains("retained prefix"));
}
