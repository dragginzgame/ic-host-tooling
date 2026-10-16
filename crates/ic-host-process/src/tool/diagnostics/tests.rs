use crate::{
    child::CleanupError,
    tool::{
        ExecutionError, ExecutionEvidence, ExecutionFailure, ToolError, format_failure_evidence,
    },
};
use std::{io, os::unix::process::ExitStatusExt as _, process::ExitStatus};

#[test]
fn failure_projection_retains_each_cleanup_error_and_original_evidence() {
    for field in 0..4 {
        let mut failure = ExecutionError {
            failure: ExecutionFailure::Cancelled,
            evidence: ExecutionEvidence {
                status: None,
                stdout: b"retained stdout\n".to_vec(),
                stderr: b"retained stderr\n".to_vec(),
                ..ExecutionEvidence::default()
            },
            cleanup: None,
        };
        let mut cleanup = CleanupError {
            status: None,
            term_error: None,
            group_error: None,
            kill_error: None,
            wait_error: None,
        };
        let cause = io::Error::from(io::ErrorKind::PermissionDenied);
        match field {
            0 => cleanup.term_error = Some(cause),
            1 => cleanup.group_error = Some(cause),
            2 => cleanup.kill_error = Some(cause),
            _ => cleanup.wait_error = Some(cause),
        }
        failure.cleanup = Some(Box::new(cleanup));
        // Check the diagnostic projection, deriving error display from its
        // typed owner rather than freezing platform-dependent error prose.
        let expected = format!(
            "fixture: {failure}\nstatus: {:?}\nstdout:\nretained stdout\nstderr:\nretained stderr\ncleanup: {}",
            failure.evidence.status,
            failure.cleanup.as_ref().unwrap(),
        );
        assert_eq!(
            format_failure_evidence("fixture", &ToolError::Execution(Box::new(failure))),
            expected,
        );
    }
}

#[test]
fn errors_without_capture_keep_their_original_display() {
    let error = ToolError::Io(io::Error::from(io::ErrorKind::PermissionDenied));
    let expected = format!("fixture: {error}");
    assert_eq!(format_failure_evidence("fixture", &error), expected);
    assert!(error.evidence().is_none());
}

#[test]
fn version_evidence_keeps_status_and_lossy_trimmed_streams() {
    let evidence = ExecutionEvidence {
        status: Some(ExitStatus::from_raw(7 << 8)),
        stdout: b"\xff stdout  \n".to_vec(),
        stderr: b"stderr\t\n".to_vec(),
        ..ExecutionEvidence::default()
    };
    let status = evidence.status;
    let error = ToolError::VersionMismatch {
        evidence: Box::new(evidence),
    };
    let expected =
        format!("version: {error}\nstatus: {status:?}\nstdout:\n� stdout\nstderr:\nstderr");
    assert_eq!(format_failure_evidence("version", &error), expected);
    assert_eq!(error.evidence().unwrap().stdout, b"\xff stdout  \n");
}
