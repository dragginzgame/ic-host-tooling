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
    write_with(&path, REPLACE, |file| -> io::Result<_> {
        file.write_all(b"old")?;
        Ok(42)
    })
    .unwrap();
    let error = write_with(&path, REPLACE, |file| {
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
    let error = write_with(&path, REPLACE, |file| {
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
    write_with(&path, REPLACE, |file| {
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
    let error = write_with(&path, CREATE, |file| -> io::Result<()> {
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
    write_with(&fresh, CREATE, |file| file.write_all(b"new")).unwrap();
    assert_eq!(fs::read(fresh).unwrap(), b"new");
    let link = fixture.root.join("link");
    symlink(&path, &link).unwrap();
    assert!(
        matches!(write_with(&link, CREATE, |file| file.write_all(b"blocked")),
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
    assert!(write_with(&dir, CREATE, |file| file.write_all(b"blocked")).is_err());
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
    for permissions in [0o1000, 0o4600, 0x1_0000 | 0o600, u32::MAX] {
        let options = WriteOptions {
            permissions,
            ..CREATE
        };
        let error = write_with(&path, options, |_| -> io::Result<()> {
            panic!("invalid mode dispatched")
        })
        .unwrap_err();
        assert!(
            matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::InvalidInput)
        );
        let error = write_at_with(
            directory.as_fd(),
            OsStr::new("output"),
            options,
            |_| -> io::Result<()> { panic!("invalid descriptor mode dispatched") },
        )
        .unwrap_err();
        assert!(
            matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::InvalidInput)
        );
    }
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
}

#[test]
fn non_utf8_publication_matches_native_filesystem_admission() {
    for options in [REPLACE, CREATE] {
        let fixture = Fixture::new();
        let directory = fs::File::open(&fixture.root).unwrap();
        let name = OsString::from_vec(vec![0xff, b'x']);
        let destination = fixture.root.join(&name);
        let native_stage = fixture.root.join("native-stage");
        fs::write(&native_stage, b"native").unwrap();
        // Observe this filesystem's destination admission independently. Darwin
        // filesystems can reject these bytes; a Unix OsString alone admits them.
        let native = match options.mode {
            PublicationMode::Replace => fs::rename(&native_stage, &destination),
            PublicationMode::CreateNew => fs::hard_link(&native_stage, &destination),
        };
        if native.is_ok() {
            assert_eq!(fs::read(&destination).unwrap(), b"native");
            fs::remove_file(&destination).unwrap();
        }
        if native.is_err() || options.mode == PublicationMode::CreateNew {
            assert_eq!(fs::read(&native_stage).unwrap(), b"native");
            fs::remove_file(&native_stage).unwrap();
        }
        let mut produced = false;
        let result = write_at_with(directory.as_fd(), &name, options, |file| {
            produced = true;
            file.write_all(b"non-UTF-8")
        });
        // Lexical name admission precedes the callback; native final-entry
        // admission occurs at publication after the producer has completed.
        assert!(produced);
        match native {
            Ok(()) => {
                result.unwrap();
                assert_eq!(fs::read(&destination).unwrap(), b"non-UTF-8");
                let names: Vec<_> = fs::read_dir(&fixture.root)
                    .unwrap()
                    .map(|entry| entry.unwrap().file_name())
                    .collect();
                assert_eq!(names, [name]);
            }
            Err(native_error) => {
                assert_eq!(
                    native_error.raw_os_error(),
                    Some(rustix::io::Errno::ILSEQ.raw_os_error())
                );
                assert!(matches!(
                    result,
                    Err(NamedWriteError::BeforePublication { source, cleanup_error: None })
                        if source.raw_os_error() == native_error.raw_os_error()
                ));
                // Enumerate instead of looking up a name rejected by this FS.
                assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 0);
            }
        }
    }
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
            None::<fn(&Path, &()) -> io::Result<()>>,
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
        None::<fn(&Path, &()) -> Result<(), &'static str>>,
        |_, _| Ok(()),
    )
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::Producer { source: "producer evidence", cleanup_error: Some(source) } if source.kind() == io::ErrorKind::InvalidData)
    );
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 2);
}

#[test]
fn closed_writer_admission_executes_before_replacing_the_destination() {
    const DRIVER: &str = "IC_HOST_FS_EXECUTABLE_ADMISSION_DRIVER";
    if std::env::var_os(DRIVER).is_none() {
        // Concurrent process creation can inherit our writable staging fd until
        // exec, transiently preventing execution even after our writer closes.
        // Keep the real execution proof, isolated from other test threads.
        let thread = std::thread::current();
        let test_name = thread.name().expect("libtest names each test thread");
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test_name, "--test-threads=1"])
            .env(DRIVER, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "isolated executable admission failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let fixture = Fixture::new();
    let output = fixture.root.join("executable with spaces");
    fs::write(&output, b"old").unwrap();
    let value = write_validated_with(
        &output,
        WriteOptions {
            permissions: 0o700,
            ..REPLACE
        },
        |file| -> io::Result<_> {
            let mut source = fs::File::open("/bin/sh")?;
            io::copy(&mut source, file)?;
            Ok(42)
        },
        |stage, value| -> io::Result<()> {
            assert!(stage.is_absolute());
            assert_eq!(*value, 42);
            assert_eq!(fs::read(&output)?, b"old");
            assert!(
                std::process::Command::new(stage)
                    .args(["-c", "exit 0"])
                    .status()?
                    .success()
            );
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(value, 42);
    assert!(
        std::process::Command::new(&output)
            .args(["-c", "exit 0"])
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);
}

#[test]
fn admission_rejection_and_identity_replacement_preserve_previous_output() {
    let fixture = Fixture::new();
    let output = fixture.root.join("output");
    for replace in [false, true] {
        fs::write(&output, b"old").unwrap();
        let mut stage_path = None;
        let result = write_validated_with(
            &output,
            REPLACE,
            |file| file.write_all(b"new"),
            |stage, ()| -> io::Result<()> {
                stage_path = Some(stage.to_owned());
                if replace {
                    fs::remove_file(stage)?;
                    fs::write(stage, b"foreign")?;
                    Ok(())
                } else {
                    Err(io::Error::from_raw_os_error(13))
                }
            },
        );
        assert_eq!(fs::read(&output).unwrap(), b"old");
        if replace {
            assert!(
                matches!(result, Err(NamedWriteError::BeforePublication { source, cleanup_error: Some(_) }) if source.kind() == io::ErrorKind::InvalidData)
            );
            assert_eq!(fs::read(stage_path.unwrap()).unwrap(), b"foreign");
        } else {
            assert!(
                matches!(result, Err(NamedWriteError::Producer { source, cleanup_error: None }) if source.raw_os_error() == Some(13))
            );
            assert!(!stage_path.unwrap().exists());
        }
    }
}

#[test]
fn validated_publication_retains_sync_and_publication_failure_phases() {
    let fixture = Fixture::new();
    let output = fixture.root.join("output");
    for step in [
        FileCommitStep::TemporaryFileSync,
        FileCommitStep::Publication,
        FileCommitStep::FinalParentSync,
    ] {
        fs::write(&output, b"old").unwrap();
        let mut admitted = false;
        let error = super::super::supported::commit_path_with_options(
            &output,
            REPLACE,
            |_, file| file.write_all(b"new"),
            Some(|_: &Path, (): &()| -> io::Result<()> {
                admitted = true;
                Ok(())
            }),
            |current, _| {
                if current == step {
                    Err(io::Error::from_raw_os_error(5))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();
        assert_eq!(admitted, step != FileCommitStep::TemporaryFileSync);
        if step == FileCommitStep::FinalParentSync {
            assert!(
                matches!(error, NamedWriteError::AfterPublication { source } if source.raw_os_error() == Some(5))
            );
            assert_eq!(fs::read(&output).unwrap(), b"new");
        } else {
            assert!(
                matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.raw_os_error() == Some(5))
            );
            assert_eq!(fs::read(&output).unwrap(), b"old");
        }
        assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);
    }
}

#[test]
fn validated_create_new_preserves_an_admission_race_winner() {
    let fixture = Fixture::new();
    let output = fixture.root.join("output");
    let result = write_validated_with(
        &output,
        CREATE,
        |file| file.write_all(b"ours"),
        |_, ()| fs::write(&output, b"winner"),
    );
    assert!(
        matches!(result, Err(NamedWriteError::BeforePublication { source, cleanup_error: None }) if source.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(output).unwrap(), b"winner");
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);
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

#[test]
fn pathname_writers_reject_nul_before_creating_parents() {
    use crate::durable::{
        create_new_bytes_with_parents, create_private_bytes_with_parents, write_bytes,
        write_named_with,
    };

    let fixture = Fixture::new();
    for name in ["invalid\0name", "invalid\0parent/output", "invalid/child\0"] {
        let path = fixture.root.join("missing/nested").join(name);
        for writer in 0..7 {
            let error = match writer {
                0 => write_bytes(&path, b"contents"),
                1 => create_new_bytes_with_parents(&path, b"contents"),
                2 => create_private_bytes_with_parents(&path, b"contents"),
                3 => write_with(&path, REPLACE, |_| -> io::Result<()> {
                    panic!("producer ran")
                }),
                4 => write_with(&path, CREATE, |_| -> io::Result<()> {
                    panic!("producer ran")
                }),
                5 => write_named_with(&path, |_| -> io::Result<()> { panic!("producer ran") }),
                _ => write_validated_with(
                    &path,
                    REPLACE,
                    |_| -> io::Result<()> { panic!("producer ran") },
                    |_, ()| panic!("admission ran"),
                ),
            }
            .expect_err("NUL-containing pathname must be refused");
            assert!(
                matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None }
                if source.kind() == io::ErrorKind::InvalidInput)
            );
            assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 0);
        }
    }
}

#[test]
fn pathname_writers_reject_directory_suffixes_before_any_effect() {
    use crate::durable::{
        create_new_bytes_with_parents, create_private_bytes_with_parents, write_bytes,
        write_named_with,
    };
    use std::path::PathBuf;

    let fixture = Fixture::new();
    let existing = fixture.root.join("existing");
    fs::write(&existing, b"preserved").unwrap();
    // Exercise relative spellings without changing the process-wide working directory.
    let cwd = std::env::current_dir().unwrap();
    let mut relative_root = PathBuf::new();
    for _ in cwd.components().skip(1) {
        relative_root.push("..");
    }
    relative_root.push(fixture.root.strip_prefix("/").unwrap());
    for root in [&fixture.root, &relative_root] {
        for name in ["existing", "absent", "missing/child"] {
            for suffix in ["/", "//", "/.", "/./.", "/.//"] {
                let mut raw = root.join(name).into_os_string();
                raw.push(suffix);
                let path = PathBuf::from(raw);
                for writer in 0..7 {
                    let error = match writer {
                        0 => write_bytes(&path, b"replacement"),
                        1 => create_new_bytes_with_parents(&path, b"replacement"),
                        2 => create_private_bytes_with_parents(&path, b"replacement"),
                        3 => write_with(&path, REPLACE, |_| -> io::Result<()> {
                            panic!("producer ran")
                        }),
                        4 => write_with(&path, CREATE, |_| -> io::Result<()> {
                            panic!("producer ran")
                        }),
                        5 => write_named_with(&path, |_| -> io::Result<()> {
                            panic!("producer ran")
                        }),
                        _ => write_validated_with(
                            &path,
                            REPLACE,
                            |_| -> io::Result<()> { panic!("producer ran") },
                            |_, ()| panic!("admission ran"),
                        ),
                    }
                    .expect_err("directory-required pathname must be refused");
                    assert!(
                        matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None }
                        if source.kind() == io::ErrorKind::InvalidInput)
                    );
                    assert_eq!(fs::read(&existing).unwrap(), b"preserved");
                    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);
                }
            }
        }
    }
    // Ordinary current-directory components in the parent remain valid.
    write_bytes(&fixture.root.join("./existing"), b"updated").unwrap();
    assert_eq!(fs::read(&existing).unwrap(), b"updated");
}
