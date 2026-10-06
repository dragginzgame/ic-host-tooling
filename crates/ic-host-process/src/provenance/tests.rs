#![expect(
    clippy::significant_drop_tightening,
    reason = "fixture ownership and serialization must cover evidence assertions"
)]

use super::*;
use crate::{
    test_support::{Fixture, LIMITS, VERSION, admit},
    tool::{ExecutionFailure, OutputStream},
};
use std::{fs, path::Path};

const OPTIONS: StatusOptions = StatusOptions {
    untracked: UntrackedFiles::All,
    ignore_submodules: IgnoreSubmodules::None,
};

fn fixture_tool() -> std::path::PathBuf {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/git.sh"
    ))
    .canonicalize()
    .unwrap()
}

#[test]
fn captures_raw_evidence_and_explicit_status_scope() {
    let fixture = Fixture::new();
    let marker = fixture.root.join("queries");
    let environment = [("FIXTURE_MARKER".into(), marker.as_os_str().to_owned())];
    let context = fixture.context(&environment);
    let git = admit(&fixture_tool(), &context, VERSION).unwrap();
    let result = capture_git(&git, &context, OPTIONS, LIMITS).unwrap();
    assert_eq!(result.revision, "0".repeat(40));
    assert_eq!(result.tree, "0".repeat(40));
    assert!(result.is_dirty());
    assert_eq!(result.options, OPTIONS);
    assert_eq!(result.tool_identity, git.identity());
    assert_eq!(result.status_evidence.stdout, b"?? space \xffpath\0");
    assert_eq!(
        result.status_identity.sha256,
        Sha256Digest::compute(&result.status_evidence.stdout)
    );
    assert_eq!(
        result.status_identity.bytes,
        result.status_evidence.stdout.len() as u64
    );
    let queries = fs::read_to_string(marker).unwrap();
    assert_eq!(queries.lines().count(), 3);
    assert!(queries.contains("rev-parse --verify HEAD^{tree}"));
    assert!(
        queries.contains("status --porcelain=v1 -z --untracked-files=all --ignore-submodules=none")
    );
    assert!(!format!("{result:?}").contains("space"));
}

#[test]
fn accepts_empty_status_and_both_git_object_formats() {
    let fixture = Fixture::new();
    for (mode, width) in [("clean", 40), ("sha256", 64)] {
        let environment = [("FIXTURE_MODE".into(), mode.into())];
        let context = fixture.context(&environment);
        let git = admit(&fixture_tool(), &context, VERSION).unwrap();
        for (untracked, ignore_submodules) in [
            (UntrackedFiles::No, IgnoreSubmodules::All),
            (UntrackedFiles::Normal, IgnoreSubmodules::Dirty),
            (UntrackedFiles::All, IgnoreSubmodules::Untracked),
        ] {
            let options = StatusOptions {
                untracked,
                ignore_submodules,
            };
            let result = capture_git(&git, &context, options, LIMITS).unwrap();
            assert_eq!(result.revision.len(), width);
            assert_eq!(result.tree.len(), width);
            assert_eq!(result.options, options);
            assert!(!result.is_dirty());
            assert_eq!(result.status_identity.sha256, Sha256Digest::compute(b""));
        }
    }
}

#[test]
fn rejects_malformed_successful_output_without_losing_evidence() {
    let fixture = Fixture::new();
    for mode in ["invalid-id", "uppercase-id", "bad-status"] {
        let environment = [("FIXTURE_MODE".into(), mode.into())];
        let context = fixture.context(&environment);
        let git = admit(&fixture_tool(), &context, VERSION).unwrap();
        let error = capture_git(&git, &context, OPTIONS, LIMITS).unwrap_err();
        if mode == "bad-status" {
            assert_eq!(error.query, GitQuery::Status);
            assert!(matches!(error.failure, GitFailure::UnterminatedStatus));
            assert!(error.completed.iter().all(Option::is_some));
            assert_eq!(
                error.completed[2].as_ref().unwrap().stdout,
                b"?? unterminated"
            );
        } else {
            assert_eq!(error.query, GitQuery::Revision);
            assert!(matches!(error.failure, GitFailure::InvalidObjectId));
            assert!(error.completed[0].is_some());
            assert!(error.completed[1..].iter().all(Option::is_none));
        }
    }
}

#[test]
fn retains_prior_captures_on_process_failure_and_bounds_output() {
    let fixture = Fixture::new();
    let marker = fixture.root.join("queries");
    let environment = [
        ("FIXTURE_MODE".into(), "fail-tree".into()),
        ("FIXTURE_MARKER".into(), marker.as_os_str().to_owned()),
    ];
    let context = fixture.context(&environment);
    let git = admit(&fixture_tool(), &context, VERSION).unwrap();
    let error = capture_git(&git, &context, OPTIONS, LIMITS).unwrap_err();
    assert_eq!(error.query, GitQuery::Tree);
    assert_eq!(fs::read_to_string(marker).unwrap().lines().count(), 2);
    assert_eq!(error.completed[0].as_ref().unwrap().stdout.len(), 41);
    assert!(error.completed[1..].iter().all(Option::is_none));
    let GitFailure::Tool(source) = error.failure else {
        panic!("expected tool failure")
    };
    let ToolError::Execution(source) = *source else {
        panic!("expected process failure")
    };
    assert!(matches!(source.failure, ExecutionFailure::ExitStatus));
    assert_eq!(source.evidence.status.unwrap().code(), Some(23));
    assert_eq!(source.evidence.stderr, b"retained tree diagnostic\n");

    let error = capture_git(
        &git,
        &context,
        OPTIONS,
        OutputLimits {
            stdout_bytes: 10,
            ..LIMITS
        },
    )
    .unwrap_err();
    assert_eq!(error.query, GitQuery::Revision);
    let GitFailure::Tool(source) = error.failure else {
        panic!("expected tool failure")
    };
    let ToolError::Execution(source) = *source else {
        panic!("expected process failure")
    };
    assert!(matches!(
        source.failure,
        ExecutionFailure::OutputLimit {
            stream: OutputStream::Stdout
        }
    ));
    assert_eq!(source.evidence.stdout.len(), 10);
    assert!(source.evidence.stdout_truncated);
}
