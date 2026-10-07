use super::*;
use crate::{
    durable::supported::{FileCommitStep, commit_at_with_hook},
    test_support::Fixture,
};
use std::{
    ffi::{OsStr, OsString},
    io::{self, Write as _},
    os::{
        fd::AsFd as _,
        unix::{
            ffi::OsStringExt as _,
            fs::{PermissionsExt as _, symlink},
        },
    },
};

const REPLACE: WriteOptions = WriteOptions {
    mode: PublicationMode::Replace,
    permissions: 0o600,
};
const CREATE: WriteOptions = WriteOptions {
    mode: PublicationMode::CreateNew,
    ..REPLACE
};

#[test]
fn typed_serialization_preserves_errors_limits_and_previous_output() {
    struct InvalidValue;
    impl serde::Serialize for InvalidValue {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("invalid fixture value"))
        }
    }
    let fixture = Fixture::new();
    let path = fixture.root.join("nested/output");
    write_typed_with(&path, REPLACE, |file| -> io::Result<_> {
        file.write_all(b"old")?;
        Ok(42)
    })
    .unwrap();
    let error = write_typed_with(&path, REPLACE, |file| {
        serde_json::to_writer(file, &("prefix", InvalidValue))
    })
    .unwrap_err();
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .is::<serde_json::Error>()
    );
    assert!(
        matches!(error, NamedWriteError::Producer { source, cleanup_error: None } if source.is_data())
    );
    let error = write_typed_with(&path, REPLACE, |file| {
        serde_json::to_writer(
            ic_host_artifacts::artifact::BoundedWriter::new(file, 3),
            &"too long",
        )
    })
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::Producer { source, cleanup_error: None } if source.is_io())
    );
    assert_eq!(fs::read(&path).unwrap(), b"old");
    assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    write_typed_with(&path, REPLACE, |file| {
        serde_json::to_writer(file, &vec![1, 2])
    })
    .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"[1,2]");
    assert_eq!(fs::metadata(path).unwrap().permissions().mode() & 0o077, 0);
}

#[test]
fn create_only_refuses_existing_entries_and_a_competing_publication() {
    let fixture = Fixture::new();
    let path = fixture.root.join("output");
    let error = write_typed_with(&path, CREATE, |file| -> io::Result<()> {
        file.write_all(b"ours")?;
        fs::write(&path, b"winner")
    })
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(&path).unwrap(), b"winner");
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);
    let fresh = fixture.root.join("fresh");
    write_typed_with(&fresh, CREATE, |file| file.write_all(b"new")).unwrap();
    assert_eq!(fs::read(fresh).unwrap(), b"new");
    let link = fixture.root.join("link");
    symlink(&path, &link).unwrap();
    assert!(
        matches!(write_typed_with(&link, CREATE, |file| file.write_all(b"blocked")),
        Err(NamedWriteError::BeforePublication { source, .. }) if source.kind() == io::ErrorKind::AlreadyExists)
    );
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let dir = fixture.root.join("directory");
    fs::create_dir(&dir).unwrap();
    assert!(write_typed_with(&dir, CREATE, |file| file.write_all(b"blocked")).is_err());
    assert!(dir.is_dir());
}

#[test]
fn held_directory_survives_parent_rename_and_replacement() {
    let fixture = Fixture::new();
    let original = fixture.root.join("original parent");
    let moved = fixture.root.join("moved parent");
    fs::create_dir(&original).unwrap();
    let directory = fs::File::open(&original).unwrap();
    let result = write_at_with(
        directory.as_fd(),
        OsStr::new("output"),
        REPLACE,
        |file| -> io::Result<_> {
            fs::rename(&original, &moved)?;
            fs::create_dir(&original)?;
            fs::write(original.join("output"), b"foreign")?;
            file.write_all(b"held directory")?;
            Ok(73)
        },
    )
    .unwrap();
    assert_eq!(result, 73);
    assert_eq!(fs::read(moved.join("output")).unwrap(), b"held directory");
    assert_eq!(fs::read(original.join("output")).unwrap(), b"foreign");
    let error = write_at_with(
        directory.as_fd(),
        OsStr::new("output"),
        REPLACE,
        |file| -> io::Result<()> {
            file.write_all(b"partial")?;
            Err(io::Error::from_raw_os_error(13))
        },
    )
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::Producer { source, cleanup_error: None } if source.raw_os_error() == Some(13))
    );
    assert_eq!(fs::read(moved.join("output")).unwrap(), b"held directory");
    assert_eq!(fs::read_dir(moved).unwrap().count(), 1);
}

#[test]
fn filename_and_descriptor_admission_precedes_any_producer_or_stage() {
    let fixture = Fixture::new();
    let directory = fs::File::open(&fixture.root).unwrap();
    for name in ["", ".", "..", "a/b", "a/", "/absolute", "a\0b"] {
        let error = write_at_with(
            directory.as_fd(),
            OsStr::new(name),
            REPLACE,
            |_| -> io::Result<()> { panic!("invalid name dispatched") },
        )
        .unwrap_err();
        assert!(
            matches!(error, NamedWriteError::BeforePublication { source, .. } if source.kind() == io::ErrorKind::InvalidInput)
        );
    }
    let path = fixture.root.join("must not create/output");
    let error = write_typed_with(
        &path,
        WriteOptions {
            permissions: 0o4600,
            ..CREATE
        },
        |_| -> io::Result<()> { panic!("invalid mode dispatched") },
    )
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::BeforePublication { source, .. } if source.kind() == io::ErrorKind::InvalidInput)
    );
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 0);
    let regular = fs::File::create(fixture.root.join("file")).unwrap();
    let error = write_at_with(
        regular.as_fd(),
        OsStr::new("output"),
        REPLACE,
        |_| -> io::Result<()> { panic!("non-directory dispatched") },
    )
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::BeforePublication { source, .. } if source.kind() == io::ErrorKind::NotADirectory)
    );
    let name = OsString::from_vec(vec![0xff, b'x']);
    write_at_with(directory.as_fd(), &name, CREATE, |file| {
        file.write_all(b"non-UTF-8")
    })
    .unwrap();
    assert_eq!(fs::read(fixture.root.join(name)).unwrap(), b"non-UTF-8");
}

#[test]
fn replace_does_not_follow_a_final_symlink_and_refuses_a_directory() {
    let fixture = Fixture::new();
    let directory = fs::File::open(&fixture.root).unwrap();
    let target = fixture.root.join("target");
    fs::write(&target, b"retained").unwrap();
    symlink(&target, fixture.root.join("link")).unwrap();
    write_at_with(directory.as_fd(), OsStr::new("link"), REPLACE, |file| {
        file.write_all(b"replacement")
    })
    .unwrap();
    assert_eq!(fs::read(target).unwrap(), b"retained");
    assert!(
        !fs::symlink_metadata(fixture.root.join("link"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    fs::create_dir(fixture.root.join("dir")).unwrap();
    assert!(matches!(
        write_at_with(directory.as_fd(), OsStr::new("dir"), REPLACE, |file| file
            .write_all(b"no")),
        Err(NamedWriteError::BeforePublication {
            cleanup_error: None,
            ..
        })
    ));
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 3);
}

#[test]
fn descriptor_publication_preserves_sync_state_and_foreign_stage_cleanup_evidence() {
    let fixture = Fixture::new();
    let directory = fs::File::open(&fixture.root).unwrap();
    let target = fixture.root.join("output");
    for step in [
        FileCommitStep::TemporaryFileSync,
        FileCommitStep::Publication,
        FileCommitStep::FinalParentSync,
    ] {
        fs::write(&target, b"old").unwrap();
        let error = commit_at_with_hook(
            directory.as_fd(),
            None,
            OsStr::new("output"),
            REPLACE,
            |_, file| file.write_all(b"new"),
            |current, _| {
                if current == step {
                    Err(io::Error::from_raw_os_error(5))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();
        if step == FileCommitStep::FinalParentSync {
            assert!(
                matches!(error, NamedWriteError::AfterPublication { source } if source.raw_os_error() == Some(5))
            );
            assert_eq!(fs::read(&target).unwrap(), b"new");
        } else {
            assert!(
                matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.raw_os_error() == Some(5))
            );
            assert_eq!(fs::read(&target).unwrap(), b"old");
        }
        assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);
    }
    let error = commit_at_with_hook(
        directory.as_fd(),
        None,
        OsStr::new("output"),
        REPLACE,
        |stage, _| -> Result<(), &'static str> {
            fs::remove_file(fixture.root.join(stage)).unwrap();
            fs::write(fixture.root.join(stage), b"foreign").unwrap();
            Err("producer evidence")
        },
        |_, _| Ok(()),
    )
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::Producer { source: "producer evidence", cleanup_error: Some(source) } if source.kind() == io::ErrorKind::InvalidData)
    );
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 2);
}

#[test]
fn link_fallback_cleanup_failure_reports_already_published_output() {
    let fixture = Fixture::new();
    let directory = fs::File::open(&fixture.root).unwrap();
    let stage = OsStr::new("stage");
    fs::write(fixture.root.join(stage), b"complete").unwrap();
    let error = super::super::supported::finish_create_new_publication::<io::Error>(
        &directory,
        stage,
        OsStr::new("output"),
        Err(rustix::io::Errno::OPNOTSUPP),
        || Err(io::Error::from_raw_os_error(13)),
    )
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::AfterPublication { source } if source.raw_os_error() == Some(13))
    );
    assert_eq!(fs::read(fixture.root.join("output")).unwrap(), b"complete");
    assert_eq!(fs::read(fixture.root.join(stage)).unwrap(), b"complete");
}
