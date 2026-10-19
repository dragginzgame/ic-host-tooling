use super::*;
use crate::{
    durable::{PublicationMode, WriteOptions, write_at_validated_with},
    testing::FixtureDirectory,
};
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStringExt;
use std::{
    fs,
    io::Write,
    os::unix::fs::{PermissionsExt as _, symlink},
};

const OBSERVE: DirectoryOptions = DirectoryOptions {
    create_permissions: None,
    forbidden_permissions: 0o022,
};
const CREATE: DirectoryOptions = DirectoryOptions {
    create_permissions: Some(0o700),
    ..OBSERVE
};

fn write(entry: &FileEntry, bytes: &[u8]) {
    let parent = entry.parent().open_syncable().unwrap();
    write_at_validated_with(
        parent.as_fd(),
        entry.file_name(),
        WriteOptions {
            mode: PublicationMode::Replace,
            permissions: 0o600,
        },
        |file| file.write_all(bytes),
        |()| Ok(()),
    )
    .unwrap();
}

#[test]
fn managed_round_trip_uses_owner_only_modes() {
    let fixture = FixtureDirectory::new(&std::env::temp_dir(), "capability-round-trip").unwrap();
    let root_path = fixture.path().join("root");
    let root = Directory::open_root(&root_path, CREATE).unwrap().unwrap();
    let entry = root
        .resolve_parent(Path::new("nns/ic/report"), CREATE)
        .unwrap()
        .unwrap();
    write(&entry, b"evidence");
    let file = entry.open_regular_file().unwrap().unwrap();
    assert_eq!(root.permissions().unwrap(), 0o700);
    assert_eq!(file.metadata().unwrap().permissions().mode() & 0o777, 0o600);
    assert_eq!(crate::read::read_opened_file(file, 8).unwrap(), b"evidence");
}

#[test]
fn missing_root_and_file_are_observed_without_creation() {
    let fixture = FixtureDirectory::new(&std::env::temp_dir(), "capability-missing").unwrap();
    let missing = fixture.path().join("missing");
    assert!(Directory::open_root(&missing, OBSERVE).unwrap().is_none());
    assert!(!missing.exists());
    let root = Directory::open_root(fixture.path(), OBSERVE)
        .unwrap()
        .unwrap();
    assert!(
        root.resolve_parent(Path::new("missing/file"), OBSERVE)
            .unwrap()
            .is_none()
    );
    let entry = root
        .resolve_parent(Path::new("file"), OBSERVE)
        .unwrap()
        .unwrap();
    assert!(entry.open_regular_file().unwrap().is_none());
    assert!(!entry.remove_file_and_sync().unwrap());
}

#[test]
fn managed_path_rejects_escape() {
    let fixture = FixtureDirectory::new(&std::env::temp_dir(), "capability-escape").unwrap();
    let root = Directory::open_root(fixture.path(), OBSERVE)
        .unwrap()
        .unwrap();
    for path in ["", ".", "../file", "child/../file", "/outside"] {
        assert!(matches!(
            root.resolve_parent(Path::new(path), CREATE),
            Err(CapabilityError::Refused {
                reason: Refusal::InvalidPath,
                ..
            })
        ));
    }
    assert!(matches!(
        Directory::open_root(&fixture.path().join("../outside"), CREATE),
        Err(CapabilityError::Refused {
            reason: Refusal::InvalidPath,
            ..
        })
    ));
    assert!(!fixture.path().join("child").exists());
}

#[test]
fn managed_path_rejects_symlinked_parent_and_file() {
    let fixture = FixtureDirectory::new(&std::env::temp_dir(), "capability-links").unwrap();
    let root_path = fixture.path().join("root");
    let root = Directory::open_root(&root_path, CREATE).unwrap().unwrap();
    let outside = fixture.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("target"), b"outside").unwrap();
    symlink(&outside, root_path.join("linked-dir")).unwrap();
    symlink(outside.join("target"), root_path.join("linked-file")).unwrap();
    symlink("absent", root_path.join("dangling")).unwrap();
    assert!(matches!(
        root.open_directory(OsStr::new("dangling"), OBSERVE),
        Err(CapabilityError::Refused {
            reason: Refusal::SymbolicLink,
            ..
        })
    ));
    assert!(matches!(
        root.resolve_parent(Path::new("linked-dir/file"), CREATE),
        Err(CapabilityError::Refused {
            reason: Refusal::SymbolicLink,
            ..
        })
    ));
    let entry = root
        .resolve_parent(Path::new("linked-file"), OBSERVE)
        .unwrap()
        .unwrap();
    assert!(matches!(
        entry.open_regular_file(),
        Err(CapabilityError::Refused {
            reason: Refusal::SymbolicLink,
            ..
        })
    ));
    assert!(
        matches!(entry.create_new_file(0o600), Err(CapabilityError::Io { operation: Operation::CreateFile, source, .. }) if source.kind() == io::ErrorKind::AlreadyExists)
    );
    let alias = fixture.path().join("root-alias");
    symlink(&root_path, &alias).unwrap();
    assert!(matches!(
        Directory::open_root(&alias, CREATE),
        Err(CapabilityError::Refused {
            reason: Refusal::SymbolicLink,
            ..
        })
    ));
    assert!(!outside.join("file").exists());
}

#[test]
fn managed_publication_stays_with_the_admitted_parent_after_path_replacement() {
    let fixture = FixtureDirectory::new(&std::env::temp_dir(), "capability-parent-move").unwrap();
    let root = Directory::open_root(fixture.path(), OBSERVE)
        .unwrap()
        .unwrap();
    let entry = root
        .resolve_parent(Path::new("parent/report"), CREATE)
        .unwrap()
        .unwrap();
    write(&entry, b"complete");
    fs::rename(fixture.path().join("parent"), fixture.path().join("moved")).unwrap();
    fs::create_dir(fixture.path().join("parent")).unwrap();
    fs::write(
        fixture.path().join("parent/report"),
        b"replacement evidence",
    )
    .unwrap();
    write(&entry, b"new complete");
    assert_eq!(
        fs::read(fixture.path().join("moved/report")).unwrap(),
        b"new complete"
    );
    assert_eq!(
        fs::read(fixture.path().join("parent/report")).unwrap(),
        b"replacement evidence"
    );
    assert!(entry.remove_file_and_sync().unwrap());
    assert!(!fixture.path().join("moved/report").exists());
    assert_eq!(
        fs::read(fixture.path().join("parent/report")).unwrap(),
        b"replacement evidence"
    );
}

#[test]
fn readable_cache_tree_supports_load_and_publication_without_permission_repair() {
    let fixture = FixtureDirectory::new(&std::env::temp_dir(), "capability-readable").unwrap();
    fs::set_permissions(fixture.path(), fs::Permissions::from_mode(0o755)).unwrap();
    let root = Directory::open_root(fixture.path(), OBSERVE)
        .unwrap()
        .unwrap();
    let entry = root
        .resolve_parent(Path::new("report"), OBSERVE)
        .unwrap()
        .unwrap();
    fs::write(fixture.path().join("report"), b"evidence").unwrap();
    fs::set_permissions(
        fixture.path().join("report"),
        fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    write(&entry, b"replacement");
    assert_eq!(root.permissions().unwrap(), 0o755);
    assert_eq!(
        fs::metadata(fixture.path().join("report"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

#[test]
fn directory_permission_settings_remain_explicit() {
    let fixture =
        FixtureDirectory::new(&std::env::temp_dir(), "capability-permission-policy").unwrap();
    fs::create_dir(fixture.path().join("shared")).unwrap();
    fs::set_permissions(
        fixture.path().join("shared"),
        fs::Permissions::from_mode(0o775),
    )
    .unwrap();
    let root = Directory::open_root(fixture.path(), OBSERVE)
        .unwrap()
        .unwrap();
    assert!(matches!(
        root.resolve_parent(Path::new("shared/file"), OBSERVE),
        Err(CapabilityError::Permissions {
            actual: 0o775,
            forbidden: 0o022,
            ..
        })
    ));
    let allow_shared = DirectoryOptions {
        forbidden_permissions: 0,
        ..OBSERVE
    };
    assert!(
        root.resolve_parent(Path::new("shared/file"), allow_shared)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        fs::metadata(fixture.path().join("shared"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o775
    );
}

#[test]
fn exclusive_creation_retains_existing_bytes_and_native_cause() {
    let fixture = FixtureDirectory::new(&std::env::temp_dir(), "capability-exclusive").unwrap();
    let root = Directory::open_root(fixture.path(), OBSERVE)
        .unwrap()
        .unwrap();
    let entry = root
        .resolve_parent(Path::new("lock"), OBSERVE)
        .unwrap()
        .unwrap();
    entry
        .create_new_file(0o600)
        .unwrap()
        .write_all(b"intent")
        .unwrap();
    assert!(
        matches!(entry.create_new_file(0o600), Err(CapabilityError::Io { operation: Operation::CreateFile, source, .. }) if source.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(fixture.path().join("lock")).unwrap(), b"intent");
}

#[test]
fn directory_creation_rechecks_already_existing_entries() {
    let fixture = FixtureDirectory::new(&std::env::temp_dir(), "capability-create-race").unwrap();
    let root = Directory::open_root(fixture.path(), OBSERVE)
        .unwrap()
        .unwrap();
    fs::create_dir(fixture.path().join("directory")).unwrap();
    fs::set_permissions(
        fixture.path().join("directory"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    assert_eq!(
        root.create_component(OsStr::new("directory"), 0o700, 0o022)
            .unwrap()
            .permissions()
            .unwrap(),
        0o700
    );
    fs::write(fixture.path().join("file"), b"existing").unwrap();
    symlink("directory", fixture.path().join("link")).unwrap();
    for (name, expected) in [
        ("file", Refusal::NotDirectory),
        ("link", Refusal::SymbolicLink),
    ] {
        assert!(
            matches!(root.create_component(OsStr::new(name), 0o700, 0o022), Err(CapabilityError::Refused { reason, .. }) if reason == expected)
        );
    }
    fs::set_permissions(
        fixture.path().join("directory"),
        fs::Permissions::from_mode(0o777),
    )
    .unwrap();
    assert!(matches!(
        root.create_component(OsStr::new("directory"), 0o700, 0o022),
        Err(CapabilityError::Permissions { .. })
    ));
    assert_eq!(fs::read(fixture.path().join("file")).unwrap(), b"existing");
}

#[test]
fn removal_preserves_completed_effect_and_original_sync_cause() {
    let fixture = FixtureDirectory::new(&std::env::temp_dir(), "capability-remove-sync").unwrap();
    let root = Directory::open_root(fixture.path(), OBSERVE)
        .unwrap()
        .unwrap();
    let entry = root
        .resolve_parent(Path::new("file"), OBSERVE)
        .unwrap()
        .unwrap();
    write(&entry, b"evidence");
    let failure = entry
        .remove_file_and_sync_with(|parent| {
            Err(super::failure(
                Operation::SyncDirectory,
                &parent.display,
                io::Error::from_raw_os_error(rustix::io::Errno::IO.raw_os_error()),
            ))
        })
        .unwrap_err();
    assert!(failure.removed);
    assert!(!fixture.path().join("file").exists());
    assert!(
        matches!(failure.source, CapabilityError::Io { operation: Operation::SyncDirectory, source, .. } if source.raw_os_error() == Some(rustix::io::Errno::IO.raw_os_error()))
    );
    fs::create_dir(fixture.path().join("file")).unwrap();
    let failure = entry
        .remove_file_and_sync_with(|_| panic!("unlink failed; sync must not run"))
        .unwrap_err();
    assert!(!failure.removed);
    assert!(matches!(
        failure.source,
        CapabilityError::Io {
            operation: Operation::RemoveFile,
            ..
        }
    ));
    assert!(fixture.path().join("file").is_dir());
}

#[test]
fn entries_are_bounded_without_candidate_or_unicode_policy() {
    let fixture = FixtureDirectory::new(&std::env::temp_dir(), "capability-entry-budget").unwrap();
    let root = Directory::open_root(fixture.path(), OBSERVE)
        .unwrap()
        .unwrap();
    assert!(root.entries(0).unwrap().next().is_none());
    #[cfg(target_os = "linux")]
    let expected = OsString::from_vec(b"non-utf8-\xff".to_vec());
    #[cfg(not(target_os = "linux"))]
    let expected = OsString::from("data");
    fs::write(fixture.path().join(&expected), b"data").unwrap();
    assert!(matches!(
        root.entries(0).unwrap().next(),
        Some(Err(CapabilityError::EntryLimit { maximum: 0, .. }))
    ));
    let exact = root
        .entries(1)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        exact,
        vec![DirectoryEntry {
            name: expected,
            kind: EntryKind::File
        }]
    );
    fs::create_dir(fixture.path().join("directory")).unwrap();
    let mut bounded = root.entries(1).unwrap();
    assert!(bounded.next().unwrap().is_ok());
    assert!(matches!(
        bounded.next(),
        Some(Err(CapabilityError::EntryLimit { maximum: 1, .. }))
    ));
    assert!(bounded.next().is_none());
    assert!(bounded.next().is_none());
}

#[test]
fn special_files_are_refused_without_waiting_for_a_fifo_writer() {
    let fixture = FixtureDirectory::new(&std::env::temp_dir(), "capability-fifo").unwrap();
    let fifo = fixture.path().join("fifo");
    assert!(
        std::process::Command::new("/usr/bin/mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let root = Directory::open_root(fixture.path(), OBSERVE)
        .unwrap()
        .unwrap();
    let entry = root
        .resolve_parent(Path::new("fifo"), OBSERVE)
        .unwrap()
        .unwrap();
    assert!(matches!(
        entry.open_regular_file(),
        Err(CapabilityError::Refused {
            reason: Refusal::NotRegularFile,
            ..
        })
    ));
    let entries = root
        .entries(1)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries[0].kind, EntryKind::Other);
}
