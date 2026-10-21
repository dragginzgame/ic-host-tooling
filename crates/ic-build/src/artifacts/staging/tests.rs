use super::{CapturedArtifact, staging_directory};
use crate::fs_fixture::unique_temp_directory;
use ic_host_fs::owned_directory::OwnedDirectory;
use std::{fs, io};

#[test]
fn staging_unwind_cleans_up_only_owned_children_and_preserves_retained_capture() {
    let root = unique_temp_directory("staging-unwind");
    let source = root.join("source");
    fs::write(&source, b"retained").unwrap();
    let mut owned = None;
    let mut retained = None;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let staging = staging_directory(&root).unwrap();
        owned = Some(staging.path().to_owned());
        retained = Some(
            CapturedArtifact::capture(&source, &root, 8)
                .unwrap()
                .retain(),
        );
        panic!("synthetic producer panic");
    }));
    assert!(result.is_err());
    assert!(!owned.unwrap().exists());
    assert_eq!(fs::read(retained.unwrap()).unwrap(), b"retained");
    assert_eq!(fs::read(&source).unwrap(), b"retained");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn cleanup_failure_keeps_build_retry_but_explicit_close_transfers_recovery() {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

    let root = unique_temp_directory("owned-directory-cleanup-failure");
    // Root bypasses permission refusal; release evidence runs as an ordinary user.
    if fs::metadata(&root).unwrap().uid() == 0 {
        fs::remove_dir_all(root).unwrap();
        return;
    }
    let owned = root.join("build");
    fs::create_dir(&owned).unwrap();
    let mut build = OwnedDirectory::new(owned.clone());
    let staging = staging_directory(&root).unwrap();
    let stage_path = staging.path().to_owned();
    let original_permissions = fs::metadata(&root).unwrap().permissions();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o500)).unwrap();
    let build_error = build.try_cleanup().unwrap_err();
    let close_error = staging.cleanup().unwrap_err();
    fs::set_permissions(&root, original_permissions).unwrap();

    assert_eq!(build_error.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(close_error.kind(), io::ErrorKind::PermissionDenied);
    assert!(build.is_armed());
    drop(build);
    assert!(!owned.exists(), "failed build cleanup retries on Drop");
    assert!(
        stage_path.exists(),
        "explicit close leaves recovery to the caller"
    );
    fs::remove_dir_all(root).unwrap();
}

// Moved from Canic's capture tests; producer paths and public publication remain distinct.
#[test]
fn captured_cargo_input_survives_replacement_and_cleans_up() {
    let root = unique_temp_directory("captured-cargo-output");
    let source = root.join("cargo.wasm");
    let destination = root.join("published.wasm");
    fs::write(&source, b"compiled first role").unwrap();
    fs::write(&destination, b"previous artifact").unwrap();
    let captured = CapturedArtifact::capture(&source, &root, 1024).unwrap();
    let staging = captured.path().parent().unwrap().to_owned();

    fs::write(&source, b"later compilation").unwrap();
    assert_eq!(fs::read(captured.path()).unwrap(), b"compiled first role");
    assert_eq!(fs::read(&destination).unwrap(), b"previous artifact");
    fs::remove_file(&source).unwrap();
    assert_eq!(fs::read(captured.path()).unwrap(), b"compiled first role");

    drop(captured);
    assert!(!staging.exists());
    assert_eq!(fs::read(&destination).unwrap(), b"previous artifact");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_cargo_input_capture_leaves_no_stage_or_published_changes() {
    let root = unique_temp_directory("failed-cargo-capture");
    let destination = root.join("published.wasm");
    fs::write(&destination, b"previous artifact").unwrap();
    let error = CapturedArtifact::capture(&root.join("missing.wasm"), &root, 1024).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    assert_eq!(fs::read(&destination).unwrap(), b"previous artifact");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn capture_honors_the_selected_byte_budget_and_preserves_its_typed_error() {
    let root = unique_temp_directory("bounded-capture");
    let source = root.join("input");
    fs::write(&source, b"four").unwrap();
    let error = CapturedArtifact::capture(&source, &root, 3).unwrap_err();
    assert!(matches!(
        error
            .get_ref()
            .unwrap()
            .downcast_ref::<ic_host_artifacts::artifact::ArtifactError>(),
        Some(ic_host_artifacts::artifact::ArtifactError::LimitExceeded { limit: 3 })
    ));
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    let (captured, identity) = CapturedArtifact::capture_with_identity(&source, &root, 4).unwrap();
    assert_eq!(fs::read(captured.path()).unwrap(), b"four");
    assert_eq!(identity.bytes, 4);
    assert_eq!(
        identity.sha256,
        ic_host_artifacts::artifact::Sha256Digest::compute(b"four")
    );
    fs::write(&source, b"replacement").unwrap();
    assert_eq!(fs::read(captured.path()).unwrap(), b"four");
    let staging = captured.path().parent().unwrap().to_owned();
    captured.close().unwrap();
    assert!(!staging.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn simultaneous_captures_retain_distinct_versions_without_blocking() {
    let root = unique_temp_directory("distinct-captures");
    let source = root.join("input");
    fs::write(&source, b"first").unwrap();
    let first = CapturedArtifact::capture(&source, &root, 1024).unwrap();
    let duplicate = CapturedArtifact::capture(&source, &root, 1024).unwrap();
    fs::write(&source, b"second").unwrap();
    let second = CapturedArtifact::capture(&source, &root, 1024).unwrap();
    assert_ne!(first.path(), duplicate.path());
    assert_eq!(fs::read(first.path()).unwrap(), b"first");
    assert_eq!(fs::read(second.path()).unwrap(), b"second");
    first.close().unwrap();
    assert_eq!(fs::read(duplicate.path()).unwrap(), b"first");
    duplicate.close().unwrap();
    second.close().unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn capture_streams_empty_and_multichunk_sources_and_admits_before_staging() {
    use ic_host_fs::read::hash_file;

    let root = unique_temp_directory("streamed-capture");
    let source = root.join("input");
    for length in [0, 1, 16 * 1024, 3 * 64 * 1024 + 7] {
        fs::write(&source, vec![42; length]).unwrap();
        let captured = CapturedArtifact::capture(&source, &root, length).unwrap();
        assert_eq!(
            hash_file(captured.path(), length as u64).unwrap(),
            hash_file(&source, length as u64).unwrap()
        );
        captured.close().unwrap();
    }
    let absent_parent = root.join("absent");
    let error = CapturedArtifact::capture(&source, &absent_parent, 1).unwrap_err();
    assert!(matches!(
        error.get_ref().and_then(|cause| cause.downcast_ref()),
        Some(ic_host_artifacts::artifact::ArtifactError::LimitExceeded { limit: 1 })
    ));
    let error = CapturedArtifact::capture(&root, &absent_parent, usize::MAX).unwrap_err();
    assert!(matches!(
        error.get_ref().and_then(|cause| cause.downcast_ref()),
        Some(ic_host_artifacts::artifact::ArtifactError::NotRegularFile)
    ));
    assert!(!absent_parent.exists());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn staging_preserves_unowned_children_and_requires_an_existing_parent() {
    let root = unique_temp_directory("staging-custody");
    let foreign = root.join("foreign");
    fs::create_dir(&foreign).unwrap();
    fs::write(foreign.join("evidence"), b"preserve").unwrap();
    let missing = root.join("missing");
    assert_eq!(
        staging_directory(&missing).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    assert!(!missing.exists());
    let staging = staging_directory(&root).unwrap();
    fs::write(staging.path().join("candidate"), b"owned").unwrap();
    let staged_path = staging.path().to_owned();
    staging.cleanup().unwrap();
    assert!(!staged_path.exists());
    assert_eq!(fs::read(foreign.join("evidence")).unwrap(), b"preserve");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retained_capture_survives_owner_and_source_without_retaining_other_captures() {
    let root = unique_temp_directory("retained-capture");
    let source = root.join("producer.wasm");
    fs::write(&source, b"exact evidence").unwrap();
    let retained = CapturedArtifact::capture(&source, &root, 32).unwrap();
    let temporary = CapturedArtifact::capture(&source, &root, 32).unwrap();
    let temporary_path = temporary.path().to_owned();
    let path = retained.retain();
    fs::remove_file(&source).unwrap();
    drop(temporary);
    assert!(!temporary_path.exists());
    assert_eq!(fs::read(&path).unwrap(), b"exact evidence");
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
    assert!(!path.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retain_to_finalizes_exact_bytes_and_preserves_replacement_semantics() {
    let root = unique_temp_directory("retain-final");
    let source = root.join("producer");
    let destination = root.join("final");
    fs::write(&source, b"exact capture").unwrap();
    fs::write(&destination, b"previous").unwrap();
    let capture = CapturedArtifact::capture(&source, &root, 100).unwrap();
    let stage = capture.path().parent().unwrap().to_owned();
    fs::remove_file(source).unwrap();
    assert_eq!(capture.retain_to(&destination).unwrap(), destination);
    assert_eq!(fs::read(destination).unwrap(), b"exact capture");
    assert!(!stage.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retain_to_rename_failure_reports_original_capture_custody() {
    let root = unique_temp_directory("retain-rename-failure");
    let source = root.join("producer");
    let destination = root.join("occupied");
    fs::write(&source, b"retained failure").unwrap();
    fs::create_dir(&destination).unwrap();
    let capture = CapturedArtifact::capture(&source, &root, 100).unwrap();
    let retained = capture.path().to_owned();
    let error = capture.retain_to(&destination).unwrap_err();
    fs::remove_file(source).unwrap();
    assert_eq!(error.retained_path, retained);
    assert!(error.staging_directory.is_dir());
    assert_eq!(fs::read(&error.retained_path).unwrap(), b"retained failure");
    assert_eq!(fs::read_dir(destination).unwrap().count(), 0);
    let kind = error.source.kind();
    let mapped = io::Error::new(kind, error);
    assert_eq!(mapped.kind(), kind);
    let retention = mapped
        .get_ref()
        .unwrap()
        .downcast_ref::<super::CaptureRetentionError>()
        .unwrap();
    assert_eq!(
        std::error::Error::source(retention)
            .unwrap()
            .downcast_ref::<io::Error>()
            .unwrap()
            .kind(),
        kind
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retain_to_cleanup_failure_reports_final_bytes_and_leaves_extra_evidence() {
    let root = unique_temp_directory("retain-cleanup-failure");
    let source = root.join("producer");
    let destination = root.join("final");
    fs::write(&source, b"final exact bytes").unwrap();
    let capture = CapturedArtifact::capture(&source, &root, 100).unwrap();
    let stage = capture.path().parent().unwrap().to_owned();
    fs::write(stage.join("extra"), b"other evidence").unwrap();
    let error = capture.retain_to(&destination).unwrap_err();
    fs::remove_file(source).unwrap();
    assert_eq!(error.retained_path, destination);
    assert_eq!(error.staging_directory, stage);
    assert_eq!(
        fs::read(&error.retained_path).unwrap(),
        b"final exact bytes"
    );
    assert_eq!(fs::read(stage.join("extra")).unwrap(), b"other evidence");
    assert!(error.to_string().contains("final"));
    fs::remove_dir_all(root).unwrap();
}
