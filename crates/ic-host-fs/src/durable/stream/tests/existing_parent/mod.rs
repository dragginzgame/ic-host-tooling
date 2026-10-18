use super::*;
use std::os::unix::ffi::OsStrExt as _;

#[test]
fn existing_parent_selection_rejects_missing_and_directory_targets_before_production() {
    let fixture = fixture();
    for path in [
        fixture.path().join("missing/report"),
        fixture.path().join("directory/"),
        fixture.path().join("directory/."),
        fixture.path().join(OsStr::from_bytes(b"nul\0name")),
    ] {
        let mut produced = false;
        let result = write_existing_parent_validated_with(
            &path,
            CREATE,
            |_| {
                produced = true;
                Ok::<_, io::Error>(())
            },
            |()| Ok(()),
        );
        assert!(matches!(
            result,
            Err(NamedWriteError::BeforePublication { .. })
        ));
        assert!(!produced);
    }
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 0);
}

#[test]
fn existing_parent_modes_keep_permissions_symlinks_and_typed_admission() {
    let fixture = fixture();
    let output = fixture.path().join("report");
    write_existing_parent_validated_with(
        &output,
        CREATE,
        |file| file.write_all(b"first"),
        |()| Ok(()),
    )
    .unwrap();
    assert_eq!(
        fs::metadata(&output).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let rejected = write_existing_parent_validated_with(
        &output,
        REPLACE,
        |file| {
            file.write_all(b"rejected").map_err(|_| 7_u8)?;
            Ok(42_u8)
        },
        |value| {
            assert_eq!(*value, 42);
            Err(9_u8)
        },
    );
    assert!(matches!(
        rejected,
        Err(NamedWriteError::Producer {
            source: 9,
            cleanup_error: None
        })
    ));
    assert_eq!(fs::read(&output).unwrap(), b"first");
    assert!(
        write_existing_parent_validated_with(
            &output,
            CREATE,
            |file| file.write_all(b"duplicate"),
            |()| Ok(())
        )
        .is_err()
    );
    assert_eq!(fs::read(&output).unwrap(), b"first");
    let alias = fixture.path().join("parent alias");
    symlink(fixture.path(), &alias).unwrap();
    write_existing_parent_validated_with(
        &alias.join("report"),
        REPLACE,
        |file| {
            file.write_all(b"accepted")?;
            Ok::<_, io::Error>(42)
        },
        |value| {
            assert_eq!(*value, 42);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(fs::read(&output).unwrap(), b"accepted");
    let link = fixture.path().join("final-link");
    symlink(&output, &link).unwrap();
    assert!(
        write_existing_parent_validated_with(
            &link,
            CREATE,
            |file| file.write_all(b"duplicate"),
            |()| Ok(())
        )
        .is_err()
    );
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read(&output).unwrap(), b"accepted");
    write_existing_parent_validated_with(
        &link,
        REPLACE,
        |file| file.write_all(b"link replacement"),
        |()| Ok(()),
    )
    .unwrap();
    assert!(fs::symlink_metadata(&link).unwrap().is_file());
    assert_eq!(fs::read(&link).unwrap(), b"link replacement");
    assert_eq!(fs::read(&output).unwrap(), b"accepted");
}

#[test]
fn existing_parent_remains_selected_after_path_rename() {
    let fixture = fixture();
    let original = fixture.path().join("parent");
    let renamed = fixture.path().join("renamed");
    fs::create_dir(&original).unwrap();
    write_existing_parent_validated_with(
        &original.join("report"),
        CREATE,
        |file| {
            fs::rename(&original, &renamed)?;
            fs::create_dir(&original)?;
            file.write_all(b"retained directory")
        },
        |()| Ok(()),
    )
    .unwrap();
    assert_eq!(
        fs::read(renamed.join("report")).unwrap(),
        b"retained directory"
    );
    assert!(!original.join("report").exists());
}

#[test]
fn existing_parent_refuses_invalid_permissions_before_production_or_admission() {
    let fixture = fixture();
    for output in [
        fixture.path().join("report"),
        fixture.path().join("missing/report"),
    ] {
        let error = write_existing_parent_validated_with(
            &output,
            WriteOptions {
                permissions: 0o1000,
                ..CREATE
            },
            |_| -> io::Result<()> { panic!("invalid permissions ran producer") },
            |()| panic!("invalid permissions ran admission"),
        )
        .unwrap_err();
        assert!(
            matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None }
            if source.kind() == io::ErrorKind::InvalidInput)
        );
    }
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 0);
}

#[test]
fn existing_parent_refuses_non_directories_without_blocking() {
    use std::{
        process::Command,
        time::{Duration, Instant},
    };
    const CHILD: &str = "IC_HOST_FS_EXISTING_PARENT_CHILD";
    if let Some(parent) = std::env::var_os(CHILD) {
        let path = std::path::PathBuf::from(parent).join("report");
        let validated = write_existing_parent_validated_with(
            &path,
            CREATE,
            |_| -> io::Result<()> { panic!("non-directory parent ran producer") },
            |()| panic!("non-directory parent ran admission"),
        );
        assert!(
            matches!(validated, Err(NamedWriteError::BeforePublication { source, cleanup_error: None })
            if source.kind() == io::ErrorKind::NotADirectory)
        );
        return;
    }
    let fixture = fixture();
    let regular = fixture.path().join("regular");
    fs::write(&regular, b"preserved").unwrap();
    let fifo = fixture.path().join("fifo");
    crate::test_support::create_fifo(&fifo, None).unwrap();
    for parent in [&fifo, &regular] {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "durable::stream::tests::existing_parent::existing_parent_refuses_non_directories_without_blocking", "--test-threads=1"])
            .env(CHILD, parent)
            .spawn().unwrap();
        let started = Instant::now();
        let result = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Err(error) => break Err(error),
                Ok(None) => {}
            }
            if started.elapsed() >= Duration::from_secs(5) {
                break Err(io::Error::from(io::ErrorKind::TimedOut));
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        if result.is_err() {
            let _ = child.kill();
            let _ = child.wait();
        }
        assert!(result.expect("parent admission must not block").success());
    }
    assert_eq!(fs::read(&regular).unwrap(), b"preserved");
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 2);
}
