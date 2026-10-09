use super::*;

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
use super::supported::{FileCommitStep, commit_with_hook, publish_create_new_after_error};

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn streamed_replacement_returns_producer_value_only_after_complete_publication() {
    use std::io::Write as _;
    let root = temp_root("streamed-replacement");
    let path = root.join("nested/output");
    let value = write_with(&path, REPLACE_OPTIONS, |file| {
        assert!(!path.exists());
        file.write_all(b"first ")?;
        file.write_all(b"second")?;
        Ok::<_, io::Error>(42)
    })
    .unwrap();
    assert_eq!(value, 42);
    assert_eq!(fs::read(&path).unwrap(), b"first second");
    assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn producer_failure_and_bounded_copy_preserve_previous_destination() {
    use std::io::Write as _;
    struct BrokenInput;
    impl io::Read for BrokenInput {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::from_raw_os_error(13))
        }
    }
    let root = temp_root("streamed-failure");
    let path = root.join("output");
    write_bytes(&path, b"original").unwrap();
    let error = write_with(&path, REPLACE_OPTIONS, |file| {
        file.write_all(b"partial")?;
        Err::<(), _>(io::Error::from(io::ErrorKind::InvalidData))
    })
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::Producer { source, cleanup_error: None } if source.kind() == io::ErrorKind::InvalidData)
    );
    assert_eq!(fs::read(&path).unwrap(), b"original");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    let copy = |maximum| {
        write_with(&path, REPLACE_OPTIONS, |file| {
            ic_host_artifacts::artifact::copy_reader(b"new bytes".as_slice(), file, maximum)
                .map_err(io::Error::from)
        })
    };
    let error = copy(8).unwrap_err();
    let NamedWriteError::Producer {
        source: error,
        cleanup_error: None,
    } = error
    else {
        panic!("bounded-copy error must retain its producer and successful cleanup");
    };
    assert!(matches!(
        error
            .get_ref()
            .unwrap()
            .downcast_ref::<ic_host_artifacts::artifact::CopyError>(),
        Some(ic_host_artifacts::artifact::CopyError::Input(
            ic_host_artifacts::artifact::ArtifactError::LimitExceeded { limit: 8 }
        ))
    ));
    let error = write_with(&path, REPLACE_OPTIONS, |file| {
        ic_host_artifacts::artifact::copy_reader(BrokenInput, file, 9).map_err(io::Error::from)
    })
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::Producer { source, cleanup_error: None } if source.raw_os_error() == Some(13))
    );
    assert_eq!(fs::read(&path).unwrap(), b"original");
    let identity = copy(9).unwrap();
    assert_eq!(identity.bytes, 9);
    assert_eq!(fs::read(&path).unwrap(), b"new bytes");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    let error = write_with(&path, REPLACE_OPTIONS, |_| -> io::Result<()> {
        let stage = fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|entry| entry != &path)
            .unwrap();
        fs::remove_file(&stage).unwrap();
        fs::write(stage, b"foreign entry").unwrap();
        Err(io::Error::from_raw_os_error(13))
    })
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::Producer { source, cleanup_error: Some(cleanup) }
        if source.raw_os_error() == Some(13) && cleanup.kind() == io::ErrorKind::InvalidData)
    );
    assert_eq!(fs::read(&path).unwrap(), b"new bytes");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn durable_lock_reports_wait_for_another_process_and_retains_exclusion() {
    use std::{
        io::{BufRead, Write},
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    const DRIVER: &str = "CANIC_TEST_PROGRESS_LOCK_DRIVER";
    const CHILD_ROOT: &str = "CANIC_TEST_PROGRESS_LOCK_ROOT";
    let thread = std::thread::current();
    let test_name = thread.name().expect("libtest names each test thread");
    if std::env::var_os(DRIVER).is_none() {
        // Parallel tests can fork while this case holds a descriptor. CLOEXEC
        // closes inherited copies only at exec, so isolate the immediate Drop
        // release assertion from unrelated child-process creation.
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test_name])
            .env(DRIVER, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "isolated lock test failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        hold_regular_lock_until_release(&PathBuf::from(root).join("complete-build-reuse.lock"));
        return;
    }
    let root = temp_root("progress-lock");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name, "--nocapture"])
        .env(CHILD_ROOT, &root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = io::BufReader::new(child.stdout.take().unwrap());
    loop {
        let mut line = String::new();
        assert_ne!(
            stdout.read_line(&mut line).unwrap(),
            0,
            "child must hold lock"
        );
        if line.contains("LOCK_HELD") {
            break;
        }
    }
    let path = root.join("complete-build-reuse.lock");
    assert!(matches!(
        try_lock_regular_file_with_parents(&path),
        Err(RegularFileLockError::Io(error)) if error.kind() == io::ErrorKind::WouldBlock
    ));
    // The owner is still waiting for our release message: acquisition did not
    // wait for it to exit, and the failed attempt did not truncate its file.
    assert!(child.try_wait().unwrap().is_none());
    assert_eq!(fs::read(&path).unwrap(), b"existing lock contents");
    assert!(matches!(
        lock_file_with_progress(&path, |_, _| Err(io::ErrorKind::PermissionDenied.into())),
        Err(RegularFileLockError::Io(error)) if error.kind() == io::ErrorKind::PermissionDenied
    ));
    let started = Instant::now();
    let mut progress = None;
    let lock = lock_file_with_progress(&path, |_, elapsed| {
        if progress.is_none() {
            progress = Some(elapsed);
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(b"release\n")
                .unwrap();
        }
        Ok(())
    })
    .unwrap();
    assert!(child.wait().unwrap().success());
    assert!(progress.unwrap() >= Duration::from_secs(1));
    assert!(started.elapsed() >= progress.unwrap());
    let contender = fs::File::open(&path).unwrap();
    assert_eq!(
        rustix::fs::flock(
            &contender,
            rustix::fs::FlockOperation::NonBlockingLockExclusive
        ),
        Err(rustix::io::Errno::WOULDBLOCK)
    );
    drop(lock);
    rustix::fs::flock(
        &contender,
        rustix::fs::FlockOperation::NonBlockingLockExclusive,
    )
    .unwrap();
    drop(contender);
    let acquired = try_lock_regular_file_with_parents(&path).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"existing lock contents");
    drop(acquired);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
fn hold_regular_lock_until_release(path: &Path) {
    use std::io::Write as _;
    let lock = try_lock_regular_file_with_parents(path).unwrap();
    assert!(
        rustix::io::fcntl_getfd(&lock)
            .unwrap()
            .contains(rustix::io::FdFlags::CLOEXEC)
    );
    fs::write(path, b"existing lock contents").unwrap();
    println!("LOCK_HELD");
    io::stdout().flush().unwrap();
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
}

#[test]
fn durable_write_creates_parents_and_replaces_complete_contents() {
    let root = temp_root("replace");
    let path = root.join("reports/nested/state.json");

    write_bytes(&path, b"old").expect("write initial contents");
    write_bytes(&path, b"new complete contents").expect("replace contents");

    assert_eq!(
        fs::read(&path).expect("read target"),
        b"new complete contents"
    );
    assert_no_temporary_files(&root);

    fs::remove_dir_all(root).expect("remove temp root");
}

#[cfg(unix)]
#[test]
fn lock_errors_preserve_admission_and_original_io_causes() {
    use std::error::Error as _;
    let fixture = crate::test_support::Fixture::new();
    let target = fixture.root.join("target");
    fs::write(&target, b"unchanged").unwrap();
    let link = fixture.root.join("link");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let fifo = fixture.root.join("fifo");
    // rustix's mknodat/mkfifoat are unavailable on Apple hosts.
    assert!(
        std::process::Command::new("/usr/bin/mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    // Existing special entries must be rejected without staging in their parent.
    for path in [&fixture.root, &link, &fifo, Path::new("/dev/null")] {
        let result = open_regular_lock_file_with_parents(path);
        assert!(
            matches!(result, Err(RegularFileLockError::NotRegular)),
            "{path:?}: {result:?}"
        );
        assert!(matches!(
            try_lock_regular_file_with_parents(path),
            Err(RegularFileLockError::NotRegular)
        ));
        assert!(matches!(
            lock_regular_file_with_parents(path),
            Err(RegularFileLockError::NotRegular)
        ));
        assert!(matches!(
            lock_file_with_progress(path, |_, _| panic!("admission failed")),
            Err(RegularFileLockError::NotRegular)
        ));
    }
    assert_eq!(fs::read(&target).unwrap(), b"unchanged");
    let error = lock_regular_file_with_parents(&fixture.root.join("invalid\0path")).unwrap_err();
    let RegularFileLockError::Io(original) = &error else {
        panic!("expected original I/O cause");
    };
    let retained = error.source().unwrap().downcast_ref::<io::Error>().unwrap();
    assert!(std::ptr::eq(original, retained));
    assert_eq!(retained.kind(), io::ErrorKind::InvalidInput);
    let projected = io::Error::from(error);
    assert_eq!(projected.kind(), io::ErrorKind::InvalidInput);
    let projected = io::Error::from(RegularFileLockError::NotRegular);
    assert!(matches!(
        projected
            .get_ref()
            .unwrap()
            .downcast_ref::<RegularFileLockError>(),
        Some(RegularFileLockError::NotRegular)
    ));
}

#[cfg(unix)]
#[test]
fn lock_creation_preserves_publication_error_through_io_boundary() {
    use std::error::Error as _;
    let fixture = crate::test_support::Fixture::new();
    let link = fixture.root.join("parent");
    std::os::unix::fs::symlink(fixture.root.join("missing"), &link).unwrap();
    let error = open_regular_lock_file_with_parents(&link.join("lock")).unwrap_err();
    let RegularFileLockError::Publication(original) = &error else {
        panic!("expected typed lock-file creation failure: {error:?}");
    };
    assert!(
        matches!(original, NamedWriteError::BeforePublication { source, cleanup_error: None }
        if source.kind() == io::ErrorKind::NotADirectory)
    );
    assert!(std::ptr::eq(
        original,
        error
            .source()
            .unwrap()
            .downcast_ref::<NamedWriteError<io::Error>>()
            .unwrap()
    ));
    let projected = io::Error::from(error);
    assert!(
        matches!(projected.get_ref().unwrap().downcast_ref::<RegularFileLockError>(),
        Some(RegularFileLockError::Publication(NamedWriteError::BeforePublication { source, cleanup_error: None }))
            if source.kind() == io::ErrorKind::NotADirectory)
    );
    assert!(!fixture.root.join("missing").exists());
}

#[cfg(unix)]
#[test]
fn existing_lock_admission_does_not_require_parent_write_permission() {
    use std::os::unix::fs::PermissionsExt as _;
    let fixture = crate::test_support::Fixture::new();
    let path = fixture.root.join("lock");
    fs::write(&path, b"existing lock bytes").unwrap();
    fs::set_permissions(&fixture.root, fs::Permissions::from_mode(0o555)).unwrap();
    // Restore permissions before asserting so even a regression leaves the
    // fixture removable. The syscall probe separately checks no staging/sync.
    let result = open_regular_lock_file_with_parents(&path);
    fs::set_permissions(&fixture.root, fs::Permissions::from_mode(0o755)).unwrap();
    let file = result.unwrap();
    assert!(file.metadata().unwrap().is_file());
    assert_eq!(fs::read(&path).unwrap(), b"existing lock bytes");
}

#[cfg(unix)]
#[test]
fn concurrent_missing_lock_openers_converge_on_one_regular_file() {
    use std::{os::unix::fs::MetadataExt as _, sync::Barrier};
    let fixture = crate::test_support::Fixture::new();
    let path = fixture.root.join("nested/lock");
    let start = Barrier::new(8);
    let files = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    start.wait();
                    open_regular_lock_file_with_parents(&path).unwrap()
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>()
    });
    let expected = fs::metadata(&path).unwrap();
    for file in files {
        let metadata = file.metadata().unwrap();
        assert_eq!(
            (metadata.dev(), metadata.ino()),
            (expected.dev(), expected.ino())
        );
    }
    assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    assert_eq!(expected.len(), 0);
}

#[cfg(unix)]
#[test]
fn admitted_lock_files_compose_with_shared_locks_and_explicit_clone_unlock() {
    use rustix::fs::{FlockOperation as Op, flock};
    let root = temp_root("admitted-lock");
    let path = root.join("nested/lock");
    let first = open_regular_lock_file_with_parents(&path).unwrap();
    fs::write(&path, b"retained contents").unwrap();
    let second = open_regular_lock_file_with_parents(&path).unwrap();
    assert!(
        rustix::io::fcntl_getfd(&first)
            .unwrap()
            .contains(rustix::io::FdFlags::CLOEXEC)
    );
    // Both opens were unlocked: an independent exclusive acquisition succeeds.
    flock(&second, Op::NonBlockingLockExclusive).unwrap();
    flock(&second, Op::Unlock).unwrap();
    flock(&first, Op::NonBlockingLockShared).unwrap();
    flock(&second, Op::NonBlockingLockShared).unwrap();
    let contender = open_regular_lock_file_with_parents(&path).unwrap();
    assert_eq!(
        flock(&contender, Op::NonBlockingLockExclusive),
        Err(rustix::io::Errno::WOULDBLOCK)
    );
    let duplicate = first.try_clone().unwrap();
    flock(&first, Op::Unlock).unwrap();
    flock(&second, Op::Unlock).unwrap();
    // Explicit final-owner policy can release retention despite a live clone.
    lock_exclusive_with_wait(&contender, std::time::Duration::from_millis(1), |_| {
        panic!("explicitly unlocked descriptors must not retain contention")
    })
    .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"retained contents");
    flock(&contender, Op::Unlock).unwrap();
    drop((first, second, duplicate, contender));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn durable_create_new_with_parents_never_replaces_an_existing_file() {
    let root = temp_root("create-new-with-parents");
    fs::create_dir_all(&root).expect("create temp root");
    let path = root.join("nested/report.json");

    create_new_bytes_with_parents(&path, b"first complete contents").expect("create output");
    let error = create_new_bytes_with_parents(&path, b"replacement")
        .expect_err("existing output must reject");

    assert!(
        matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(
        fs::read(&path).expect("read target"),
        b"first complete contents"
    );
    assert_no_temporary_files(&root);

    fs::remove_dir_all(root).expect("remove temp root");
}

#[cfg(unix)]
#[test]
fn durable_create_new_with_parents_rejects_a_symlinked_parent() {
    use std::os::unix::fs::symlink;

    let root = temp_root("create-new-symlink-parent");
    let outside = temp_root("create-new-symlink-outside");
    fs::create_dir_all(&root).expect("create temp root");
    fs::create_dir_all(&outside).expect("create outside root");
    symlink(&outside, root.join("linked")).expect("create parent symlink");

    let error = create_new_bytes_with_parents(&root.join("linked/report.json"), b"contents")
        .expect_err("symlinked parent must reject");

    assert!(
        matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::NotADirectory)
    );
    assert!(!outside.join("report.json").exists());
    fs::remove_dir_all(root).expect("remove temp root");
    fs::remove_dir_all(outside).expect("remove outside root");
}

#[test]
fn durable_write_rejects_a_target_without_a_file_name() {
    let error = write_bytes(Path::new("/"), b"value").expect_err("directory target must fail");

    assert!(
        matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::InvalidInput)
    );
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn prepublication_failures_preserve_old_complete_bytes_and_remove_staging() {
    let steps = [
        FileCommitStep::TemporaryFileCreate,
        FileCommitStep::TemporaryFileWrite,
        FileCommitStep::TemporaryFileSync,
        FileCommitStep::Publication,
    ];

    for step in steps {
        let root = temp_root(&format!("prepublication-{step:?}"));
        fs::create_dir_all(&root).expect("create temp root");
        let path = root.join("report.json");
        fs::write(&path, b"old complete contents").expect("write old contents");
        let mut failed = false;

        let error = commit_with_hook(
            &path,
            b"new complete contents",
            REPLACE_OPTIONS,
            |current, _| {
                if current == step && !failed {
                    failed = true;
                    return Err(io::Error::other("injected file commit failure"));
                }
                Ok(())
            },
        )
        .expect_err("injected step must fail");

        assert!(
            matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::Other)
        );
        assert_eq!(
            fs::read(&path).expect("read preserved target"),
            b"old complete contents"
        );
        assert_no_temporary_files(&root);
        fs::remove_dir_all(root).expect("remove temp root");
    }
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn postpublication_sync_failure_exposes_only_new_complete_bytes() {
    for options in [REPLACE_OPTIONS, CREATE_NEW_OPTIONS, CREATE_PRIVATE_OPTIONS] {
        let root = temp_root("postpublication");
        fs::create_dir_all(&root).expect("create temp root");
        let path = root.join("report.json");
        if options.mode == PublicationMode::Replace {
            fs::write(&path, b"old complete contents").expect("write old contents");
        }

        let error = commit_with_hook(&path, b"new complete contents", options, |step, _| {
            if step == FileCommitStep::FinalParentSync {
                return Err(io::Error::other("injected parent sync failure"));
            }
            Ok(())
        })
        .expect_err("postpublication sync must fail");

        assert!(
            matches!(error, NamedWriteError::AfterPublication { source } if source.kind() == io::ErrorKind::Other)
        );
        assert_eq!(
            fs::read(&path).expect("read published target"),
            b"new complete contents"
        );
        assert_no_temporary_files(&root);

        fs::remove_dir_all(root).expect("remove temp root");
    }
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn create_new_publication_race_cannot_replace_the_winner() {
    let root = temp_root("create-new-race");
    fs::create_dir_all(&root).expect("create temp root");
    let path = root.join("report.json");

    let error = commit_with_hook(
        &path,
        b"our complete contents",
        CREATE_NEW_OPTIONS,
        |step, _| {
            if step == FileCommitStep::Publication {
                fs::write(&path, b"raced complete contents")?;
            }
            Ok(())
        },
    )
    .expect_err("atomic create-new must reject a publication race");

    assert!(
        matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(
        fs::read(&path).expect("read winning target"),
        b"raced complete contents"
    );
    assert_no_temporary_files(&root);

    fs::remove_dir_all(root).expect("remove temp root");
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn unsupported_no_replace_rename_uses_atomic_link_publication() {
    let unsupported = [
        ("invalid", rustix::io::Errno::INVAL),
        ("no-syscall", rustix::io::Errno::NOSYS),
        ("not-supported", rustix::io::Errno::OPNOTSUPP),
    ];

    for (label, error) in unsupported {
        let root = temp_root(&format!("create-new-link-{label}"));
        fs::create_dir_all(&root).expect("create temp root");
        let temp_name = std::ffi::OsStr::new(".report.json.canic-tmp-test");
        let file_name = std::ffi::OsStr::new("report.json");
        let temp_path = root.join(temp_name);
        let path = root.join(file_name);
        fs::write(&temp_path, b"complete contents").expect("write staged contents");

        publish_create_new_after_error(&root, temp_name, file_name, error)
            .expect("publish through hard-link fallback");

        assert_eq!(fs::read(&path).expect("read target"), b"complete contents");
        assert!(!temp_path.exists());
        fs::remove_dir_all(root).expect("remove temp root");
    }
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn link_publication_fallback_cannot_replace_an_existing_file() {
    let root = temp_root("create-new-link-race");
    fs::create_dir_all(&root).expect("create temp root");
    let temp_name = std::ffi::OsStr::new(".report.json.canic-tmp-test");
    let file_name = std::ffi::OsStr::new("report.json");
    let temp_path = root.join(temp_name);
    let path = root.join(file_name);
    fs::write(&temp_path, b"our complete contents").expect("write staged contents");
    fs::write(&path, b"raced complete contents").expect("write winning target");

    let error =
        publish_create_new_after_error(&root, temp_name, file_name, rustix::io::Errno::INVAL)
            .expect_err("link fallback must reject an existing destination");

    assert!(
        matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(
        fs::read(&path).expect("read winning target"),
        b"raced complete contents"
    );
    assert_eq!(
        fs::read(&temp_path).expect("read preserved staging"),
        b"our complete contents"
    );
    fs::remove_dir_all(root).expect("remove temp root");
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn new_parent_failures_never_create_the_final_file_or_staging() {
    let steps = [
        FileCommitStep::ParentDirectoryCreate,
        FileCommitStep::CreatedDirectorySync,
        FileCommitStep::CreatedDirectoryParentSync,
    ];

    for step in steps {
        let root = temp_root(&format!("parent-{step:?}"));
        fs::create_dir_all(&root).expect("create temp root");
        let path = root.join("reports/nested/report.json");
        let mut failed = false;

        let error = commit_with_hook(
            &path,
            b"complete contents",
            REPLACE_OPTIONS,
            |current, _| {
                if current == step && !failed {
                    failed = true;
                    return Err(io::Error::other("injected parent persistence failure"));
                }
                Ok(())
            },
        )
        .expect_err("injected parent step must fail");

        assert!(
            matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::Other)
        );
        assert!(!path.exists());
        assert_no_temporary_files(&root);
        fs::remove_dir_all(root).expect("remove temp root");
    }
}

#[test]
fn failed_replacement_removes_its_staging_file() {
    let root = temp_root("replace-error");
    let path = root.join("occupied");
    fs::create_dir_all(&path).expect("create non-file destination");
    fs::write(path.join("child"), b"preserved").expect("write destination child");

    write_bytes(&path, b"new contents").expect_err("non-empty directory must reject replacement");

    assert_eq!(
        fs::read(path.join("child")).expect("read destination child"),
        b"preserved"
    );
    assert_no_temporary_files(&root);

    fs::remove_dir_all(root).expect("remove temp root");
}

fn assert_no_temporary_files(root: &Path) {
    if !root.exists() {
        return;
    }
    for entry in fs::read_dir(root).expect("read test directory") {
        let entry = entry.expect("read test entry");
        let path = entry.path();
        if path.is_dir() {
            assert_no_temporary_files(&path);
        } else {
            assert!(
                !entry.file_name().to_string_lossy().contains(".canic-tmp-"),
                "temporary file remains at {}",
                path.display()
            );
        }
    }
}

fn temp_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch")
        .as_nanos();
    std::env::temp_dir().canonicalize().unwrap().join(format!(
        "canic-host-durable-io-{label}-{}-{nanos}",
        std::process::id()
    ))
}

#[test]
#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
fn private_publication_is_owner_only_before_writing_and_never_replaces() {
    use std::os::unix::fs::PermissionsExt as _;
    let root = temp_root("private-publication");
    let path = root.join("private/key");
    commit_with_hook(&path, &[7; 32], CREATE_PRIVATE_OPTIONS, |step, observed| {
        if step == FileCommitStep::TemporaryFileWrite {
            assert_eq!(fs::metadata(observed)?.permissions().mode() & 0o777, 0o600);
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(
        crate::read::read_private_bytes::<32>(&path).unwrap(),
        Some([7; 32])
    );
    let error = create_private_bytes_with_parents(&path, &[8; 32]).unwrap_err();
    assert!(
        matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(
        crate::read::read_private_bytes::<32>(&path).unwrap(),
        Some([7; 32])
    );
    fs::remove_dir_all(root).unwrap();
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn replacement_supports_long_destination_names() {
    let root = temp_root("long-replacement");
    fs::create_dir(&root).unwrap();
    let path = root.join("a".repeat(255));
    fs::write(&path, b"previous").unwrap();
    write_bytes(&path, b"complete").unwrap();
    let copied = write_with(&path, REPLACE_OPTIONS, |file| {
        std::io::Write::write_all(file, b"streamed")?;
        Ok::<_, io::Error>(8)
    })
    .unwrap();
    assert_eq!(copied, 8);
    assert_eq!(fs::read(&path).unwrap(), b"streamed");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn staging_collisions_retry_without_touching_unowned_files() {
    let root = temp_root("collision-replacement");
    fs::create_dir(&root).unwrap();
    let path = root.join("output");
    fs::write(&path, b"previous").unwrap();
    let mut collision = None;
    commit_with_hook(&path, b"complete", REPLACE_OPTIONS, |step, candidate| {
        if step == FileCommitStep::TemporaryFileCreate && collision.is_none() {
            fs::write(candidate, b"unowned")?;
            collision = Some(candidate.to_owned());
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"complete");
    assert_eq!(fs::read(collision.as_ref().unwrap()).unwrap(), b"unowned");
    let mut blocked = Vec::new();
    let error = commit_with_hook(
        &path,
        b"never publish",
        REPLACE_OPTIONS,
        |step, candidate| {
            if step == FileCommitStep::TemporaryFileCreate {
                fs::write(candidate, b"unowned")?;
                blocked.push(candidate.to_owned());
            }
            Ok(())
        },
    )
    .unwrap_err();
    assert!(
        matches!(error, NamedWriteError::BeforePublication { source, cleanup_error: None } if source.kind() == io::ErrorKind::AlreadyExists)
    );
    assert!(!blocked.is_empty() && blocked.len() <= 64);
    assert_eq!(fs::read(&path).unwrap(), b"complete");
    for candidate in blocked {
        assert_eq!(fs::read(candidate).unwrap(), b"unowned");
    }
    fs::remove_dir_all(root).unwrap();
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
#[test]
fn selected_staging_namespace_destinations_remain_absent_until_publication() {
    const CHILD: &str = "IC_HOST_FS_STAGING_DESTINATION_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "durable::tests::selected_staging_namespace_destinations_remain_absent_until_publication", "--test-threads=1"])
            .env(CHILD, "1")
            .output().unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let root = temp_root("staging-destination");
    fs::create_dir(&root).unwrap();
    for (prefix, sequence) in [(".ic-host-tmp", 0), (".IC-HOST-TMP", 2)] {
        let path = root.join(format!("{prefix}-{}-{sequence}", std::process::id()));
        write_with(&path, REPLACE_OPTIONS, |file| {
            assert!(!path.exists());
            std::io::Write::write_all(file, b"complete")
        })
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"complete");
    }
    fs::remove_dir_all(root).unwrap();
}
