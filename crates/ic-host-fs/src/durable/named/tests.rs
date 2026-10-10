use super::*;
use crate::{
    durable::{
        REPLACE_OPTIONS,
        supported::{FileCommitStep, commit_path_with_options},
    },
    test_support::Fixture,
};
use std::{
    fs,
    os::unix::fs::{MetadataExt as _, symlink},
};

#[test]
fn absolute_spaced_stage_publishes_validated_output_and_returns_value() {
    let fixture = Fixture::new();
    let output = fixture.root.join("parent with spaces/output file");
    let value = write_named_with(&output, |stage| -> io::Result<_> {
        assert!(stage.is_absolute());
        assert_eq!(stage.parent(), output.parent());
        let initial = fs::metadata(stage)?;
        assert_eq!(initial.len(), 0);
        fs::write(stage, b"validated output")?;
        assert_eq!(initial.ino(), fs::metadata(stage)?.ino());
        assert!(!output.exists());
        Ok(42)
    })
    .unwrap();
    assert_eq!(value, 42);
    assert_eq!(fs::read(&output).unwrap(), b"validated output");
    assert_eq!(fs::read_dir(output.parent().unwrap()).unwrap().count(), 1);
}

#[test]
fn producer_and_validation_errors_preserve_original_and_remove_owned_stage() {
    let fixture = Fixture::new();
    let output = fixture.root.join("output");
    fs::write(&output, b"old").unwrap();
    for bytes in [b"".as_slice(), b"invalid", b"partial"] {
        let error = write_named_with(&output, |stage| {
            fs::write(stage, bytes).unwrap();
            Err::<(), _>(vec![1, 2, 3])
        })
        .unwrap_err();
        assert!(
            matches!(error, NamedWriteError::Producer { source, cleanup_error: None } if source == [1, 2, 3])
        );
        assert_eq!(fs::read(&output).unwrap(), b"old");
        assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);
    }
}

#[test]
fn replaced_missing_or_linked_staging_is_never_published_or_blindly_removed() {
    let fixture = Fixture::new();
    for substitution in [
        "missing",
        "file",
        "symlink",
        "directory",
        "fifo",
        "hardlink",
    ] {
        let parent = fixture.root.join(substitution);
        fs::create_dir(&parent).unwrap();
        let output = parent.join("output");
        let other = parent.join("other");
        fs::write(&output, b"old").unwrap();
        fs::write(&other, b"unrelated").unwrap();
        let mut staged_path = None;
        let result = write_named_with(&output, |stage| -> io::Result<()> {
            staged_path = Some(stage.to_path_buf());
            fs::write(stage, b"produced")?;
            if substitution == "hardlink" {
                fs::hard_link(stage, parent.join("extra-link"))?;
            } else {
                fs::remove_file(stage)?;
                match substitution {
                    "missing" => {}
                    "file" => fs::write(stage, b"foreign")?,
                    "symlink" => symlink(&other, stage)?,
                    "directory" => fs::create_dir(stage)?,
                    "fifo" => assert!(
                        std::process::Command::new("/usr/bin/mkfifo")
                            .arg(stage)
                            .status()?
                            .success()
                    ),
                    _ => unreachable!(),
                }
            }
            Ok(())
        });
        let NamedWriteError::BeforePublication {
            source,
            cleanup_error,
        } = result.unwrap_err()
        else {
            panic!("identity drift must stop publication");
        };
        assert!(matches!(
            source.kind(),
            io::ErrorKind::InvalidData | io::ErrorKind::NotFound
        ));
        let stage = staged_path.unwrap();
        if matches!(substitution, "missing" | "hardlink") {
            assert!(cleanup_error.is_none());
            assert!(!stage.exists());
        } else {
            assert_eq!(cleanup_error.unwrap().kind(), io::ErrorKind::InvalidData);
            assert!(fs::symlink_metadata(&stage).is_ok());
        }
        assert_eq!(fs::read(&output).unwrap(), b"old");
        assert_eq!(fs::read(&other).unwrap(), b"unrelated");
    }
}

#[test]
fn parent_replacement_is_rejected_and_cleanup_uses_original_directory() {
    let fixture = Fixture::new();
    let parent = fixture.root.join("parent");
    let moved = fixture.root.join("moved");
    fs::create_dir(&parent).unwrap();
    let output = parent.join("output");
    fs::write(&output, b"old").unwrap();
    let result = write_named_with(&output, |stage| -> io::Result<()> {
        fs::write(stage, b"produced")?;
        fs::rename(&parent, &moved)?;
        fs::create_dir(&parent)?;
        fs::write(&output, b"foreign output")?;
        Ok(())
    });
    assert!(matches!(
        result,
        Err(NamedWriteError::BeforePublication {
            cleanup_error: None,
            ..
        })
    ));
    assert_eq!(fs::read(&output).unwrap(), b"foreign output");
    assert_eq!(fs::read(moved.join("output")).unwrap(), b"old");
    assert_eq!(fs::read_dir(moved).unwrap().count(), 1);
}

#[test]
fn sync_errors_have_correct_publication_state() {
    let fixture = Fixture::new();
    let output = fixture.root.join("output");
    fs::write(&output, b"old").unwrap();
    let result = commit_path_with_options(
        &output,
        REPLACE_OPTIONS,
        |_, file| {
            use std::io::Write as _;
            file.write_all(b"new")
        },
        None::<fn(&Path, &()) -> io::Result<()>>,
        |step, _| {
            if step == FileCommitStep::TemporaryFileSync {
                Err(io::Error::from_raw_os_error(5))
            } else {
                Ok(())
            }
        },
    );
    assert!(
        matches!(result, Err(NamedWriteError::BeforePublication { source, cleanup_error: None }) if source.raw_os_error() == Some(5))
    );
    assert_eq!(fs::read(&output).unwrap(), b"old");
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);

    let result = commit_path_with_options(
        &output,
        REPLACE_OPTIONS,
        |_, file| {
            use std::io::Write as _;
            file.write_all(b"new")
        },
        None::<fn(&Path, &()) -> io::Result<()>>,
        |step, _| {
            if step == FileCommitStep::FinalParentSync {
                Err(io::Error::from_raw_os_error(5))
            } else {
                Ok(())
            }
        },
    );
    assert!(
        matches!(result, Err(NamedWriteError::AfterPublication { source }) if source.raw_os_error() == Some(5))
    );
    assert_eq!(fs::read(&output).unwrap(), b"new");
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);
}

#[test]
fn replacement_after_sync_is_rejected_before_rename() {
    use std::cell::RefCell;
    let fixture = Fixture::new();
    let output = fixture.root.join("output");
    fs::write(&output, b"old").unwrap();
    let staged_path = RefCell::new(None);
    let result = commit_path_with_options(
        &output,
        REPLACE_OPTIONS,
        |stage, file| {
            use std::io::Write as _;
            staged_path.replace(Some(stage.to_path_buf()));
            file.write_all(b"produced")
        },
        None::<fn(&Path, &()) -> io::Result<()>>,
        |step, _| {
            if step == FileCommitStep::Publication {
                let borrowed = staged_path.borrow();
                let stage = borrowed.as_ref().unwrap();
                fs::remove_file(stage)?;
                fs::write(stage, b"foreign")?;
            }
            Ok(())
        },
    );
    assert!(matches!(
        result,
        Err(NamedWriteError::BeforePublication {
            cleanup_error: Some(_),
            ..
        })
    ));
    assert_eq!(fs::read(&output).unwrap(), b"old");
    assert_eq!(
        fs::read(staged_path.into_inner().unwrap()).unwrap(),
        b"foreign"
    );
}

#[test]
fn producer_error_retains_cleanup_rejection_and_error_source() {
    use std::error::Error as _;
    let fixture = Fixture::new();
    let output = fixture.root.join("output");
    let error = write_named_with(&output, |stage| {
        fs::remove_file(stage).unwrap();
        fs::write(stage, b"foreign").unwrap();
        Err::<(), _>(io::Error::from_raw_os_error(13))
    })
    .unwrap_err();
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .unwrap()
            .raw_os_error(),
        Some(13)
    );
    assert!(
        matches!(error, NamedWriteError::Producer { source, cleanup_error: Some(cleanup) }
        if source.raw_os_error() == Some(13) && cleanup.kind() == io::ErrorKind::InvalidData)
    );
    assert!(!output.exists());
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 1);
}

#[test]
fn display_keeps_phase_primary_cause_and_secondary_cleanup() {
    use std::error::Error as _;
    for error in [
        NamedWriteError::Producer {
            source: io::Error::other("producer failure"),
            cleanup_error: Some(io::Error::other("cleanup failure")),
        },
        NamedWriteError::BeforePublication {
            source: io::Error::other("filesystem failure"),
            cleanup_error: Some(io::Error::other("cleanup failure")),
        },
    ] {
        let text = error.to_string();
        assert!(text.contains(&error.source().unwrap().to_string()));
        assert!(text.contains("staging cleanup failed: cleanup failure"));
        match error {
            NamedWriteError::Producer { .. } => {
                assert!(text.starts_with("staged producer failed:"));
            }
            NamedWriteError::BeforePublication { .. } => {
                assert!(text.starts_with("publication failed before rename:"));
            }
            NamedWriteError::AfterPublication { .. } => unreachable!(),
        }
    }
}
