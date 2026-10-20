use super::test_support::{Fixture, LIMITS};
use super::*;
use ic_host_process::tool::{ExecutionFailure, OutputStream};
use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
use std::{ffi::OsString, fs};

#[test]
fn extraction_retains_source_tool_and_original_output_evidence() {
    let fixture = Fixture::new();
    let source = fixture.source();
    let original = fs::read(&source).unwrap();
    let tool = fixture.tool();
    let candid = extract(&tool, &source, &fixture.context(&[]), 8, LIMITS).unwrap();
    assert_eq!(
        candid.evidence.stdout,
        b"//  \r\nservice : {  \r\n  query : () -> () query;\t\r\n}"
    );
    assert_eq!(candid.source_identity, hash_file(&source, 8).unwrap());
    assert_eq!(candid.tool_identity, tool.identity());
    assert!(candid.evidence.stdout.contains(&b'\r'));
    assert!(candid.evidence.status.success());
    assert_eq!(fs::read(source).unwrap(), original);
}

#[test]
fn extraction_rejects_missing_relative_and_oversized_sources_before_invocation() {
    let fixture = Fixture::new();
    let source = fixture.source();
    let tool = fixture.tool();
    let marker = fixture.root.join("invocations");
    let environment = [(
        OsString::from("FIXTURE_MARKER"),
        marker.as_os_str().to_owned(),
    )];
    let context = fixture.context(&environment);
    assert!(matches!(
        extract(&tool, Path::new("source.wasm"), &context, 8, LIMITS),
        Err(ExtractionError::SourcePath)
    ));
    assert!(matches!(
        extract(&tool, &source, &context, 7, LIMITS),
        Err(ExtractionError::Input(ArtifactError::LimitExceeded {
            limit: 7
        }))
    ));
    assert!(matches!(
        extract(&tool, &fixture.root.join("missing"), &context, 8, LIMITS),
        Err(ExtractionError::Input(ArtifactError::Io(_)))
    ));
    assert!(!marker.exists());
}

#[test]
fn extractor_failure_and_output_overflow_keep_evidence_without_decoding() {
    let fixture = Fixture::new();
    let source = fixture.source();
    let tool = fixture.tool();
    let environment = [(
        OsString::from("FIXTURE_CANDID_MODE"),
        OsString::from("fail"),
    )];
    let error = extract(&tool, &source, &fixture.context(&environment), 8, LIMITS);
    assert!(
        matches!(error, Err(ExtractionError::Tool(ToolError::Execution(error))) if matches!(error.failure, ExecutionFailure::ExitStatus) && error.evidence.stderr == b"extract failed")
    );
    let environment = [(
        OsString::from("FIXTURE_CANDID_MODE"),
        OsString::from("invalid-utf8"),
    )];
    let candid = extract(&tool, &source, &fixture.context(&environment), 8, LIMITS).unwrap();
    assert_eq!(candid.evidence.stdout, [0xff]);
    assert!(candid.evidence.status.success());
    let error = extract(
        &tool,
        &source,
        &fixture.context(&[]),
        8,
        OutputLimits {
            stdout: ic_host_process::tool::OutputLimit::Terminate(10),
            ..LIMITS
        },
    );
    assert!(
        matches!(error, Err(ExtractionError::Tool(ToolError::Execution(error))) if matches!(error.failure, ExecutionFailure::OutputLimit { stream: OutputStream::Stdout }) && error.evidence.stdout.len() == 10)
    );
    assert_eq!(fs::read(source).unwrap(), b"\0asm\x01\0\0\0");
}

#[test]
fn exact_raw_output_bound_accepts_unterminated_bytes() {
    let fixture = Fixture::new();
    let source = fixture.source();
    let tool = fixture.tool();
    let environment = [(
        OsString::from("FIXTURE_CANDID_MODE"),
        OsString::from("unterminated"),
    )];
    let candid = extract(
        &tool,
        &source,
        &fixture.context(&environment),
        8,
        OutputLimits {
            stdout: ic_host_process::tool::OutputLimit::Terminate(b"service : {}".len()),
            ..LIMITS
        },
    )
    .unwrap();
    assert_eq!(candid.evidence.stdout, b"service : {}");
    assert!(candid.evidence.status.success());
}

#[test]
fn source_changes_or_failed_reinspection_reject_successful_output() {
    let fixture = Fixture::new();
    let tool = fixture.tool();
    let source = fixture.source();
    let environment = [(
        OsString::from("FIXTURE_CANDID_MODE"),
        OsString::from("mutate"),
    )];
    let error = extract(&tool, &source, &fixture.context(&environment), 100, LIMITS);
    assert!(
        matches!(error, Err(ExtractionError::SourceChanged { before, after, evidence }) if before != after && evidence.stdout == b"service : {}\n")
    );
    let source = fixture.source();
    let environment = [(
        OsString::from("FIXTURE_CANDID_MODE"),
        OsString::from("grow"),
    )];
    let error = extract(&tool, &source, &fixture.context(&environment), 8, LIMITS);
    assert!(
        matches!(error, Err(ExtractionError::SourceInspection { source: ArtifactError::LimitExceeded { limit: 8 }, evidence }) if evidence.stdout == b"service : {}\n")
    );
    let source = fixture.source();
    let environment = [(
        OsString::from("FIXTURE_CANDID_MODE"),
        OsString::from("remove"),
    )];
    let error = extract(&tool, &source, &fixture.context(&environment), 8, LIMITS);
    assert!(
        matches!(error, Err(ExtractionError::SourceInspection { source: ArtifactError::Io(_), evidence }) if evidence.stdout == b"service : {}\n")
    );
}

#[test]
fn grammar_and_expected_service_policy_remain_with_the_consumer() {
    let fixture = Fixture::new();
    let source = fixture.source();
    let environment = [(
        OsString::from("FIXTURE_CANDID_MODE"),
        OsString::from("empty"),
    )];
    let candid = extract(
        &fixture.tool(),
        &source,
        &fixture.context(&environment),
        8,
        LIMITS,
    )
    .unwrap();
    assert_eq!(candid.evidence.stdout, [] as [u8; 0]);
}

#[test]
fn source_paths_with_spaces_shell_characters_and_non_utf8_are_literal_arguments() {
    let mut name = b"source $(must-not-execute) space ".to_vec();
    name.push(0xff);
    name.extend_from_slice(b".wasm");
    for name in [
        OsString::from("source $(must-not-execute) space ☃.wasm"),
        OsString::from_vec(name),
    ] {
        let fixture = Fixture::new();
        let tool = fixture.tool();
        let non_utf8 = name.to_str().is_none();
        let source = fixture.root.join(name);
        let marker = fixture.root.join("invocations");
        let environment = [
            (
                OsString::from("FIXTURE_MARKER"),
                marker.as_os_str().to_owned(),
            ),
            (
                OsString::from("FIXTURE_CANDID_MODE"),
                OsString::from("source-path"),
            ),
        ];
        let context = fixture.context(&environment);
        if let Err(error) = fs::write(&source, b"\0asm\x01\0\0\0") {
            // macOS rejects creation of malformed filenames. Lookup can report
            // a different errno; compare extraction with the actual inspection
            // boundary rather than assuming lookup and creation fail alike.
            assert!(non_utf8);
            let illegal_sequence = rustix::io::Errno::ILSEQ.raw_os_error();
            assert_eq!(error.raw_os_error(), Some(illegal_sequence));
            let lookup = fs::metadata(&source).expect_err("rejected source must not exist");
            assert!(
                lookup.kind() == std::io::ErrorKind::NotFound
                    || lookup.raw_os_error() == Some(illegal_sequence)
            );
            let extraction = extract(&tool, &source, &context, 8, LIMITS)
                .expect_err("rejected source must fail before dispatch");
            assert!(
                matches!(
                    &extraction,
                    ExtractionError::Input(ArtifactError::Io(error))
                        if error.kind() == lookup.kind()
                            && error.raw_os_error() == lookup.raw_os_error()
                ),
                "{extraction:?}"
            );
            assert!(!marker.exists());
            continue;
        }
        let candid = extract(&tool, &source, &context, 8, LIMITS).unwrap();
        assert_eq!(candid.evidence.stderr, source.as_os_str().as_bytes());
        assert_eq!(candid.source_identity.bytes, 8);
        assert_eq!(fs::read(&source).unwrap(), b"\0asm\x01\0\0\0");
        assert_eq!(fs::read(marker).unwrap(), b"invoked\n");
    }
}

#[test]
fn truncated_extractor_output_cannot_be_published_as_candid() {
    let fixture = Fixture::new();
    let error = extract(
        &fixture.tool(),
        &fixture.source(),
        &fixture.context(&[]),
        1024,
        OutputLimits {
            stdout: ic_host_process::tool::OutputLimit::Truncate(0),
            ..LIMITS
        },
    )
    .unwrap_err();
    let ExtractionError::Tool(error) = error else {
        panic!("incomplete capture must fail")
    };
    assert!(error.evidence().unwrap().stdout_truncated);
}
