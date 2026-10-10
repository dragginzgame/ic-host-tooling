use super::*;
use ic_host_fs::{
    durable::{NamedWriteError, write_named_with},
    read::read_file_no_follow,
};

#[test]
fn named_output_keeps_bounded_capture_and_direct_child_cleanup_evidence() {
    let fixture = Fixture::new();
    let tool = fixture.tool();
    let output = fixture.root.join("output");
    fs::write(&output, b"old").unwrap();
    for argument in ["--flood-stdout", "--wait"] {
        let limits = OutputLimits {
            stdout: crate::tool::OutputLimit::Terminate(32),
            stderr: crate::tool::OutputLimit::Terminate(32),
            timeout: Some(if argument == "--wait" {
                Duration::from_millis(100)
            } else {
                Duration::from_secs(5)
            }),
        };
        let result = write_named_with(&output, |_| {
            tool.run(&[argument.into()], &fixture.context(&[]), limits)
        });
        let NamedWriteError::Producer {
            source,
            cleanup_error: None,
        } = result.unwrap_err()
        else {
            panic!("expected original process failure");
        };
        let failure = source.execution_error().unwrap();
        if argument == "--wait" {
            assert!(matches!(failure.failure, ExecutionFailure::TimedOut));
        } else {
            assert!(matches!(
                failure.failure,
                ExecutionFailure::OutputLimit {
                    stream: OutputStream::Stdout
                }
            ));
            assert_eq!(failure.evidence.stdout.len(), 32);
            assert!(failure.evidence.stdout_truncated);
        }
        assert!(failure.evidence.status.is_some());
        assert!(failure.cleanup.is_none());
        assert_eq!(fs::read(&output).unwrap(), b"old");
        assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);
    }
}

#[test]
fn admitted_named_output_preserves_process_evidence_and_validates_before_publication() {
    let fixture = Fixture::new();
    let tool = fixture.tool();
    let output = fixture.root.join("output with spaces");
    fs::write(&output, b"old").unwrap();
    for (payload, exit, valid) in [
        ("partial", "23", false),
        ("", "0", false),
        ("invalid", "0", false),
        ("complete", "0", true),
    ] {
        let result = write_named_with(&output, |stage| {
            let evidence = tool.run(
                &[
                    OsString::from("--named-output"),
                    stage.as_os_str().to_owned(),
                    payload.into(),
                    exit.into(),
                ],
                &fixture.context(&[]),
                LIMITS,
            )?;
            let bytes = read_file_no_follow(stage, 8).map_err(ToolError::Artifact)?;
            if bytes != b"complete" {
                return Err(ToolError::Io(io::Error::from(io::ErrorKind::InvalidData)));
            }
            Ok(evidence)
        });
        if valid {
            let evidence = result.unwrap();
            assert_eq!(evidence.stdout, b"producer stdout");
            assert_eq!(fs::read(&output).unwrap(), b"complete");
        } else {
            let NamedWriteError::Producer {
                source,
                cleanup_error: None,
            } = result.unwrap_err()
            else {
                panic!("expected original producer error");
            };
            if exit == "23" {
                let failure = source.execution_error().unwrap();
                assert_eq!(failure.evidence.status.unwrap().code(), Some(23));
                assert_eq!(failure.evidence.stdout, b"producer stdout");
                assert_eq!(failure.evidence.stderr, b"producer stderr");
                assert!(failure.cleanup.is_none());
            }
            assert_eq!(fs::read(&output).unwrap(), b"old");
        }
        assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);
    }
}
