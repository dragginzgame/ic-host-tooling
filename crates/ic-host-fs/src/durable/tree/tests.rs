use super::*;
use crate::{read::tree::checksum::checksum_directory, test_support::Fixture};
use ic_host_artifacts::artifact::ArtifactError;
use std::{fs, os::unix::fs::symlink};

#[test]
fn syncs_files_then_directories_and_preserves_exact_component_framing() {
    let fixture = Fixture::new();
    let root = &fixture.root;
    fs::create_dir(root.join("a")).unwrap();
    fs::write(root.join("a/leaf"), b"a").unwrap();
    fs::write(root.join("a-file"), b"b").unwrap();
    let mut steps = Vec::new();
    let actual = sync_checksum_directory_at(&File::open(root).unwrap(), root, 1, |step, path| {
        steps.push((step, path.strip_prefix(root).unwrap().to_path_buf()));
        Ok(())
    })
    .unwrap();
    assert_eq!(
        actual.to_string(),
        "8ebe1da155356d34952f1515a80ee0c223ee530c1f87f6d10efa48f875b7c9b0"
    );
    assert_eq!(actual, checksum_directory(root, 1).unwrap());
    assert_eq!(
        steps.last(),
        Some(&(TreeSyncStep::RootDirectory, PathBuf::new()))
    );
    let position = |step, name: &str| {
        steps
            .iter()
            .position(|entry| entry == &(step, name.into()))
            .unwrap()
    };
    assert!(
        position(TreeSyncStep::RegularFile, "a/leaf")
            < position(TreeSyncStep::NestedDirectory, "a")
    );
    assert!(
        position(TreeSyncStep::NestedDirectory, "a") < position(TreeSyncStep::RootDirectory, "")
    );
    assert!(
        position(TreeSyncStep::RegularFile, "a-file") < position(TreeSyncStep::RootDirectory, "")
    );
    assert_eq!(steps.len(), 4);
    assert_eq!(fs::read(root.join("a/leaf")).unwrap(), b"a");
}

#[test]
fn each_sync_hook_can_refuse_without_later_steps_or_byte_replacement() {
    for denied in [
        TreeSyncStep::RegularFile,
        TreeSyncStep::NestedDirectory,
        TreeSyncStep::RootDirectory,
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.root.join("nested")).unwrap();
        let path = fixture.root.join("nested/bytes");
        fs::write(&path, b"original").unwrap();
        let mut steps = Vec::new();
        let result = sync_checksum_directory_at(
            &File::open(&fixture.root).unwrap(),
            &fixture.root,
            8,
            |step, _| {
                steps.push(step);
                if step == denied {
                    Err(io::Error::from_raw_os_error(13))
                } else {
                    Ok(())
                }
            },
        );
        assert!(
            matches!(result, Err(TreeChecksumError::Io(error)) if error.raw_os_error() == Some(13))
        );
        assert_eq!(steps.last(), Some(&denied));
        assert_eq!(fs::read(path).unwrap(), b"original");
    }
}

#[test]
fn bounds_and_unsafe_entries_reject_without_removing_evidence() {
    let fixture = Fixture::new();
    let root = &fixture.root;
    let file = root.join("bytes");
    fs::write(&file, b"abc").unwrap();
    let directory = File::open(root).unwrap();
    assert!(matches!(
        sync_checksum_directory_at(&directory, root, 2, |_, _| Ok(())),
        Err(TreeChecksumError::Input(ArtifactError::LimitExceeded {
            limit: 2
        }))
    ));
    fs::rename(&file, root.join("retained")).unwrap();
    symlink(root.join("retained"), &file).unwrap();
    assert!(
        matches!(sync_checksum_directory_at(&directory, root, 3, |_, _| Ok(())),
        Err(TreeChecksumError::UnsupportedEntry { kind: FileType::Symlink, path }) if path == file)
    );
    fs::remove_file(&file).unwrap();
    crate::test_support::create_fifo(&file, None).unwrap();
    assert!(
        matches!(sync_checksum_directory_at(&directory, root, 3, |_, _| Ok(())),
        Err(TreeChecksumError::UnsupportedEntry { kind: FileType::Fifo, path }) if path == file)
    );
    assert_eq!(fs::read(root.join("retained")).unwrap(), b"abc");
}

#[test]
fn empty_and_renamed_held_roots_keep_exact_identity_without_reopening_display_path() {
    let fixture = Fixture::new();
    let original = fixture.root.join("original");
    fs::create_dir(&original).unwrap();
    let directory = File::open(&original).unwrap();
    assert_eq!(
        sync_checksum_directory_at(&directory, &original, 0, |_, _| Ok(())).unwrap(),
        Sha256Digest::compute(b"")
    );
    fs::write(original.join("bytes"), b"abc").unwrap();
    let expected = checksum_directory(&original, 3).unwrap();
    let retained = fixture.root.join("retained");
    fs::rename(&original, &retained).unwrap();
    fs::create_dir(&original).unwrap();
    fs::write(original.join("bytes"), b"replacement").unwrap();
    assert_eq!(
        sync_checksum_directory_at(&directory, &original, 3, |_, _| Ok(())).unwrap(),
        expected
    );
    assert_eq!(fs::read(retained.join("bytes")).unwrap(), b"abc");
    assert_eq!(fs::read(original.join("bytes")).unwrap(), b"replacement");
}

#[test]
fn raw_non_utf8_names_preserve_exact_rejected_path() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let fixture = Fixture::new();
    let path = fixture.root.join(OsStr::from_bytes(b"\xff"));
    if let Err(error) = fs::write(&path, b"bytes") {
        assert!(
            cfg!(target_os = "macos")
                && error.raw_os_error() == Some(rustix::io::Errno::ILSEQ.raw_os_error())
        );
        return;
    }
    assert!(
        matches!(sync_checksum_directory_at(&File::open(&fixture.root).unwrap(), &fixture.root, 5, |_, _| Ok(())),
        Err(TreeChecksumError::NonUtf8Path { path: actual }) if actual == path)
    );
    assert_eq!(fs::read(path).unwrap(), b"bytes");
}
