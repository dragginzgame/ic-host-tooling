use super::*;

#[test]
fn observed_identity_is_retained_and_drift_is_rejected_before_execution() {
    let fixture = Fixture::new();
    let candidate = fixture.root.join("installed tool");
    fs::copy(tool_path(), &candidate).unwrap();
    let marker = fixture.root.join("invocations");
    let environment = [(
        OsString::from("FIXTURE_MARKER"),
        marker.as_os_str().to_owned(),
    )];
    let tool = AdmittedTool::admit_version(
        &VersionSpec {
            executable: &candidate,
            executable_bytes: 1024 * 1024,
            version_arguments: &[OsString::from("--version")],
            version_identity: VERSION,
        },
        &fixture.context(&environment),
        LIMITS,
    )
    .unwrap();
    assert_eq!(tool.path(), candidate);
    assert_eq!(tool.version_identity(), VERSION);
    assert_eq!(tool.identity(), hash_file(&candidate, 1024 * 1024).unwrap());
    assert_eq!(fs::read(&marker).unwrap(), b"invoked\n");
    let output = tool
        .run(
            &[OsString::from("--arguments"), OsString::from("a b")],
            &fixture.context(&environment),
            LIMITS,
        )
        .unwrap();
    assert_eq!(output.stdout, b"[a b]");
    let original = fs::read(&candidate).unwrap();
    let mut changed = original.clone();
    changed.extend_from_slice(b"\n# changed bytes, same version\n");
    fs::write(&candidate, changed).unwrap();
    assert!(matches!(
        tool.run(&[], &fixture.context(&environment), LIMITS),
        Err(ToolError::Artifact(ArtifactError::DigestMismatch { .. }))
    ));
    fs::write(&candidate, original).unwrap();
    fs::set_permissions(&candidate, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(matches!(
        tool.run(&[], &fixture.context(&environment), LIMITS),
        Err(ToolError::NotExecutable)
    ));
    assert_eq!(fs::read(marker).unwrap(), b"invoked\ninvoked\n");
}

#[test]
fn version_admission_retains_exact_match_and_failure_evidence() {
    let fixture = Fixture::new();
    let path = tool_path();
    let spec = VersionSpec {
        executable: &path,
        executable_bytes: 1024 * 1024,
        version_arguments: &[OsString::from("--version")],
        version_identity: VERSION,
    };
    let environment = [(
        OsString::from("FIXTURE_VERSION"),
        OsString::from(format!("  {VERSION}\t")),
    )];
    assert!(AdmittedTool::admit_version(&spec, &fixture.context(&environment), LIMITS).is_ok());
    let environment = [(OsString::from("FIXTURE_VERSION"), OsString::from("wrong"))];
    let Err(ToolError::VersionMismatch { evidence }) =
        AdmittedTool::admit_version(&spec, &fixture.context(&environment), LIMITS)
    else {
        panic!("version must match exactly");
    };
    assert_eq!(evidence.stdout, b"wrong\n");
    assert!(evidence.status.unwrap().success());
    let environment = [(
        OsString::from("FIXTURE_VERSION_MODE"),
        OsString::from("invalid-utf8"),
    )];
    assert!(matches!(
        AdmittedTool::admit_version(&spec, &fixture.context(&environment), LIMITS),
        Err(ToolError::VersionUtf8 { evidence, .. }) if evidence.stdout == [0xff]
    ));
    let environment = [(
        OsString::from("FIXTURE_VERSION_MODE"),
        OsString::from("fail"),
    )];
    let Err(ToolError::Execution(error)) =
        AdmittedTool::admit_version(&spec, &fixture.context(&environment), LIMITS)
    else {
        panic!("version command must succeed");
    };
    assert!(matches!(error.failure, ExecutionFailure::ExitStatus));
    assert_eq!(error.evidence.status.unwrap().code(), Some(23));
    assert_eq!(error.evidence.stderr, b"version failed");
}

#[test]
fn installed_file_and_spec_are_admitted_before_version_dispatch() {
    let fixture = Fixture::new();
    let candidate = fixture.root.join("tool");
    fs::copy(tool_path(), &candidate).unwrap();
    let marker = fixture.root.join("invocations");
    let environment = [(
        OsString::from("FIXTURE_MARKER"),
        marker.as_os_str().to_owned(),
    )];
    let mut spec = VersionSpec {
        executable: &candidate,
        executable_bytes: 1,
        version_arguments: &[OsString::from("--version")],
        version_identity: VERSION,
    };
    assert!(matches!(
        AdmittedTool::admit_version(&spec, &fixture.context(&environment), LIMITS),
        Err(ToolError::Artifact(ArtifactError::LimitExceeded { .. }))
    ));
    spec.executable_bytes = 1024 * 1024;
    for invalid in ["", " leading", "trailing "] {
        spec.version_identity = invalid;
        assert!(matches!(
            AdmittedTool::admit_version(&spec, &fixture.context(&environment), LIMITS),
            Err(ToolError::InvalidInvocation(
                InvalidInvocation::VersionIdentity
            ))
        ));
    }
    spec.version_identity = VERSION;
    spec.executable = Path::new("relative");
    assert!(matches!(
        AdmittedTool::admit_version(&spec, &fixture.context(&environment), LIMITS),
        Err(ToolError::InvalidInvocation(
            InvalidInvocation::ExecutablePath
        ))
    ));
    spec.executable = &candidate;
    fs::set_permissions(&candidate, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(matches!(
        AdmittedTool::admit_version(&spec, &fixture.context(&environment), LIMITS),
        Err(ToolError::NotExecutable)
    ));
    assert!(!marker.exists());
}

#[test]
fn version_capture_keeps_output_and_deadline_limits() {
    let fixture = Fixture::new();
    let path = tool_path();
    for argument in ["--flood-stdout", "--wait"] {
        let Err(ToolError::Execution(error)) = AdmittedTool::admit_version(
            &VersionSpec {
                executable: &path,
                executable_bytes: 1024 * 1024,
                version_arguments: &[OsString::from(argument)],
                version_identity: VERSION,
            },
            &fixture.context(&[]),
            OutputLimits {
                stdout_bytes: 17,
                timeout: Duration::from_millis(100),
                ..LIMITS
            },
        ) else {
            panic!("version capture must remain bounded");
        };
        if argument == "--flood-stdout" {
            assert!(matches!(
                error.failure,
                ExecutionFailure::OutputLimit {
                    stream: OutputStream::Stdout
                }
            ));
            assert_eq!(error.evidence.stdout.len(), 17);
            assert!(error.evidence.stdout_truncated);
        } else {
            assert!(matches!(error.failure, ExecutionFailure::TimedOut));
        }
        assert!(error.evidence.status.is_some());
        assert!(error.kill_error.is_none() && error.wait_error.is_none());
    }
}
