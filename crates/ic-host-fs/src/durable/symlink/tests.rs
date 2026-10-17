use super::*;
use crate::test_support::Fixture;

#[test]
fn admitted_bundle_runs_before_and_after_selection_and_rejection_preserves_it() {
    use std::os::unix::fs::PermissionsExt as _;
    let fixture = Fixture::new();
    let bundle = fixture.root.join("bundle");
    fs::create_dir_all(bundle.join("bin")).unwrap();
    fs::create_dir(bundle.join("lib")).unwrap();
    let target = bundle.join("bin/tool");
    fs::write(&target, b"#!/bin/sh\ncat \"${0%/*}/../lib/runtime\"\n").unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(bundle.join("lib/runtime"), b"admitted").unwrap();
    let destination = fixture.root.join("selected");
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
    let fixture = Fixture::new();
    let target = fixture.root.join("target");
    let stage = fixture.root.join("stage");
    let destination = fixture.root.join("directory");
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
