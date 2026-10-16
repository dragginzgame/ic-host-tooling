use super::*;

#[test]
fn one_observation_retains_version_and_refuses_byte_drift() {
    let fixture = Fixture::new();
    let candidate = fixture.root.join("installed tool");
    crate::testing::write_executable_script(&candidate, fs::read(tool_path()).unwrap()).unwrap();
    let marker = fixture.root.join("invocations");
    let environment = [
        ("FIXTURE_MARKER".into(), marker.as_os_str().to_owned()),
        (
            "FIXTURE_VERSION".into(),
            "  caller-selected git build\t".into(),
        ),
    ];
    let tool = AdmittedTool::observe_version(
        &VersionObservationSpec {
            executable: &candidate,
            executable_bytes: 1024 * 1024,
            version_arguments: &["--version".into()],
        },
        &fixture.context(&environment),
        LIMITS,
    )
    .unwrap();
    assert_eq!(tool.path(), candidate);
    assert_eq!(tool.version_identity(), "caller-selected git build");
    assert_eq!(tool.identity(), hash_file(&candidate, 1024 * 1024).unwrap());
    assert_eq!(fs::read(&marker).unwrap(), b"invoked\n");
    let mut changed = fs::read(&candidate).unwrap();
    changed.extend_from_slice(b"\n# changed bytes, same version\n");
    crate::testing::write_executable_script(&candidate, changed).unwrap();
    assert!(matches!(
        tool.run(&[], &fixture.context(&environment), LIMITS),
        Err(ToolError::Artifact(ArtifactError::DigestMismatch { .. }))
    ));
    assert_eq!(fs::read(marker).unwrap(), b"invoked\n");
}

#[test]
fn observation_requires_complete_successful_nonempty_utf8() {
    let fixture = Fixture::new();
    let path = tool_path();
    let spec = VersionObservationSpec {
        executable: &path,
        executable_bytes: 1024 * 1024,
        version_arguments: &["--version".into()],
    };
    let environment = [("FIXTURE_VERSION".into(), " \t".into())];
    let Err(ToolError::VersionMismatch { evidence }) =
        AdmittedTool::observe_version(&spec, &fixture.context(&environment), LIMITS)
    else {
        panic!("empty version must be refused");
    };
    assert_eq!(evidence.stdout, b" \t\n");
    assert!(evidence.status.unwrap().success());
    let environment = [("FIXTURE_VERSION_MODE".into(), "invalid-utf8".into())];
    assert!(matches!(
        AdmittedTool::observe_version(&spec, &fixture.context(&environment), LIMITS),
        Err(ToolError::VersionUtf8 { evidence, .. }) if evidence.stdout == [0xff]
    ));
    let environment = [("FIXTURE_VERSION_MODE".into(), "fail".into())];
    let Err(ToolError::Execution(error)) =
        AdmittedTool::observe_version(&spec, &fixture.context(&environment), LIMITS)
    else {
        panic!("version command must succeed");
    };
    assert!(matches!(error.failure, ExecutionFailure::ExitStatus));
    assert_eq!(error.evidence.status.unwrap().code(), Some(23));
    assert_eq!(error.evidence.stderr, b"version failed");
    for limit in [OutputLimit::Truncate(3), OutputLimit::Terminate(3)] {
        let Err(ToolError::Execution(error)) = AdmittedTool::observe_version(
            &spec,
            &fixture.context(&[]),
            OutputLimits {
                stdout: limit,
                ..LIMITS
            },
        ) else {
            panic!("partial version must not be admitted");
        };
        assert!(matches!(
            error.failure,
            ExecutionFailure::OutputLimit {
                stream: OutputStream::Stdout
            }
        ));
        assert_eq!(error.evidence.stdout.len(), 3);
        assert!(error.evidence.stdout_truncated);
        assert!(error.evidence.status.is_some());
    }
}

#[test]
fn observation_admits_inputs_and_file_before_version_dispatch() {
    let fixture = Fixture::new();
    let path = tool_path();
    let marker = fixture.root.join("invocations");
    let environment = [("FIXTURE_MARKER".into(), marker.as_os_str().to_owned())];
    let mut spec = VersionObservationSpec {
        executable: &path,
        executable_bytes: 1,
        version_arguments: &["--version".into()],
    };
    assert!(matches!(
        AdmittedTool::observe_version(&spec, &fixture.context(&environment), LIMITS),
        Err(ToolError::Artifact(ArtifactError::LimitExceeded { .. }))
    ));
    spec.executable_bytes = 1024 * 1024;
    spec.executable = Path::new("relative");
    assert!(matches!(
        AdmittedTool::observe_version(&spec, &fixture.context(&environment), LIMITS),
        Err(ToolError::InvalidInvocation(
            InvalidInvocation::ExecutablePath
        ))
    ));
    spec.executable = &path;
    assert!(matches!(
        AdmittedTool::observe_version(
            &spec,
            &fixture.context(&environment),
            OutputLimits {
                timeout: Some(Duration::ZERO),
                ..LIMITS
            }
        ),
        Err(ToolError::InvalidInvocation(InvalidInvocation::Deadline))
    ));
    let environment = [(OsString::from(""), marker.as_os_str().to_owned())];
    assert!(matches!(
        AdmittedTool::observe_version(&spec, &fixture.context(&environment), LIMITS),
        Err(ToolError::InvalidInvocation(
            InvalidInvocation::EnvironmentName { index: 0 }
        ))
    ));
    assert!(!marker.exists());
}
