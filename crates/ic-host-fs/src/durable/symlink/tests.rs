use super::*;
use crate::test_support::fixture;

#[test]
fn admitted_bundle_runs_before_and_after_selection_and_rejection_preserves_it() {
    use std::os::unix::fs::PermissionsExt as _;
    let fixture = fixture();
    let bundle = fixture.path().join("bundle");
    fs::create_dir_all(bundle.join("bin")).unwrap();
    fs::create_dir(bundle.join("lib")).unwrap();
    let target = bundle.join("bin/tool");
    fs::write(&target, b"#!/bin/sh\ncat \"${0%/*}/../lib/runtime\"\n").unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(bundle.join("lib/runtime"), b"admitted").unwrap();
    let destination = fixture.path().join("selected");
    let stage = bundle.join("staged-link");
    let admit = |path: &Path| {
        let output = std::process::Command::new(path).output()?;
        assert!(output.status.success());
        assert_eq!(output.stdout, b"admitted");
        Ok::<_, io::Error>(())
    };
    publish_symlink_with(&target, &stage, &destination, admit).unwrap();
    admit(&fs::canonicalize(&destination).unwrap()).unwrap();
    let before = fs::read_link(&destination).unwrap();
    let result = publish_symlink_with(&target, &stage, &destination, |_| Err(42));
    assert!(matches!(
        result,
        Err(NamedWriteError::Producer {
            source: 42,
            cleanup_error: None
        })
    ));
    assert_eq!(fs::read_link(&destination).unwrap(), before);
    assert!(!stage.exists());
    assert!(target.exists());
}

#[test]
fn occupied_stage_and_failed_rename_preserve_existing_entries() {
    let fixture = fixture();
    let target = fixture.path().join("target");
    let stage = fixture.path().join("stage");
    let destination = fixture.path().join("directory");
    fs::write(&target, b"target").unwrap();
    fs::write(&stage, b"occupied").unwrap();
    fs::create_dir(&destination).unwrap();
    let result = publish_symlink_with(&target, &stage, &destination, |_| Ok::<_, ()>(()));
    assert!(matches!(
        result,
        Err(NamedWriteError::BeforePublication { .. })
    ));
    assert_eq!(fs::read(&stage).unwrap(), b"occupied");
    fs::remove_file(&stage).unwrap();
    let result = publish_symlink_with(&target, &stage, &destination, |_| Ok::<_, ()>(()));
    assert!(matches!(
        result,
        Err(NamedWriteError::BeforePublication {
            cleanup_error: None,
            ..
        })
    ));
    assert!(destination.is_dir());
    assert!(!stage.exists());
    assert_eq!(fs::read(&target).unwrap(), b"target");
}

#[test]
fn sync_failures_preserve_selection_according_to_publication_phase() {
    for fail_at in 1..=3 {
        let fixture = fixture();
        let target = fixture.path().join("new-target");
        let previous = fixture.path().join("old-target");
        let staging_directory = fixture.path().join("staging");
        fs::create_dir(&staging_directory).unwrap();
        let stage = staging_directory.join("link");
        let destination = fixture.path().join("selected");
        fs::write(&target, b"new").unwrap();
        fs::write(&previous, b"old").unwrap();
        std::os::unix::fs::symlink(&previous, &destination).unwrap();
        let mut syncs = 0;
        let result = publish_with_sync(
            &target,
            &stage,
            &destination,
            |_| Ok::<_, ()>(()),
            |directory| {
                syncs += 1;
                if syncs == fail_at {
                    Err(io::Error::from_raw_os_error(5))
                } else {
                    directory.sync_all()
                }
            },
        );
        if fail_at == 1 {
            assert!(matches!(result, Err(NamedWriteError::BeforePublication {
                source, cleanup_error: None,
            }) if source.raw_os_error() == Some(5)));
            assert_eq!(fs::read_link(&destination).unwrap(), previous);
        } else {
            assert!(matches!(result, Err(NamedWriteError::AfterPublication {
                source,
            }) if source.raw_os_error() == Some(5)));
            assert_eq!(fs::read_link(&destination).unwrap(), target);
        }
        assert_eq!(syncs, fail_at);
        assert!(fs::symlink_metadata(&stage).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"new");
        assert_eq!(fs::read(&previous).unwrap(), b"old");
    }
}

#[test]
fn observed_staging_replacement_is_never_published_or_removed() {
    for reject in [false, true] {
        let fixture = fixture();
        let target = fixture.path().join("target");
        let stage = fixture.path().join("stage");
        let moved = fixture.path().join("moved");
        let destination = fixture.path().join("selected");
        fs::write(&target, b"target").unwrap();
        fs::write(&destination, b"old").unwrap();
        let result = publish_symlink_with(&target, &stage, &destination, |_| {
            // Deliberately violate custody to exercise observed replacement refusal.
            fs::rename(&stage, &moved).unwrap();
            fs::write(&stage, b"foreign").unwrap();
            if reject { Err(42) } else { Ok(()) }
        });
        if reject {
            assert!(matches!(result, Err(NamedWriteError::Producer {
                source: 42, cleanup_error: Some(source),
            }) if source.kind() == io::ErrorKind::InvalidData));
        } else {
            assert!(matches!(result, Err(NamedWriteError::BeforePublication {
                source, cleanup_error: Some(cleanup),
            }) if source.kind() == io::ErrorKind::InvalidData
                && cleanup.kind() == io::ErrorKind::InvalidData));
        }
        assert_eq!(fs::read(&stage).unwrap(), b"foreign");
        assert_eq!(fs::read(&destination).unwrap(), b"old");
        assert_eq!(fs::read_link(&moved).unwrap(), target);
        assert_eq!(fs::read(&target).unwrap(), b"target");
    }
}

#[test]
fn invalid_paths_refuse_before_admission_without_creating_parents() {
    let fixture = fixture();
    let target = fixture.path().join("target");
    let stage = fixture.path().join("stage");
    let destination = fixture.path().join("selected");
    let missing_parent = fixture.path().join("missing");
    let missing_stage = missing_parent.join("stage");
    fs::write(&target, b"target").unwrap();
    fs::write(&destination, b"old").unwrap();
    for (target, stage) in [
        (Path::new("relative"), stage.as_path()),
        (target.as_path(), destination.as_path()),
        (target.as_path(), missing_stage.as_path()),
    ] {
        let result = publish_symlink_with(target, stage, &destination, |_| -> Result<(), ()> {
            panic!("admission must not run for invalid paths")
        });
        assert!(matches!(
            result,
            Err(NamedWriteError::BeforePublication { .. })
        ));
        assert_eq!(fs::read(&destination).unwrap(), b"old");
    }
    assert!(fs::symlink_metadata(&stage).is_err());
    assert!(!missing_parent.exists());
}
