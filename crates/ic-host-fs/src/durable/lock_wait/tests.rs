use super::{
    lock_exclusive, lock_exclusive_with_wait, lock_shared, try_lock_exclusive, unlock,
    wait_for_lock,
};
use crate::test_support::Fixture;
use std::{fs::File, io, time::Duration};

#[test]
fn shared_holders_and_explicit_unlock_preserve_descriptor_custody() {
    use crate::durable::{RegularFileLockError, try_lock_regular_file_with_parents};

    let fixture = Fixture::new();
    let path = fixture.root.join("shared-lock");
    std::fs::write(&path, b"caller-owned bytes").unwrap();
    let first = File::open(&path).unwrap();
    let duplicate = first.try_clone().unwrap();
    let second = File::open(&path).unwrap();
    lock_shared(&first).unwrap();
    lock_shared(&second).unwrap();
    let contended = || {
        assert!(matches!(
            try_lock_regular_file_with_parents(&path),
            Err(RegularFileLockError::Io(error)) if error.kind() == io::ErrorKind::WouldBlock
        ));
    };
    contended();
    unlock(&first).unwrap();
    contended();
    unlock(&second).unwrap();
    let exclusive = try_lock_regular_file_with_parents(&path).unwrap();
    unlock(&exclusive).unwrap();
    lock_exclusive(&first).unwrap();
    contended();
    unlock(&first).unwrap();
    let acquired = try_lock_regular_file_with_parents(&path).unwrap();
    unlock(&acquired).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"caller-owned bytes");
    // Every descriptor, including the duplicate, remains open after unlock.
    for file in [&first, &duplicate, &second, &exclusive, &acquired] {
        assert_eq!(file.metadata().unwrap().len(), 18);
    }
}

#[test]
fn direct_descriptor_operations_refuse_nonregular_files() {
    let fixture = Fixture::new();
    let directory = File::open(&fixture.root).unwrap();
    let operations: [fn(&File) -> io::Result<()>; 4] =
        [lock_shared, lock_exclusive, try_lock_exclusive, unlock];
    for operation in operations {
        assert_eq!(
            operation(&directory).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }
}

#[test]
fn descriptor_contention_reports_and_preserves_returned_ownership() {
    const DRIVER: &str = "IC_HOST_FS_DESCRIPTOR_LOCK_DRIVER";
    if std::env::var_os(DRIVER).is_none() {
        // A parallel subprocess test can inherit our locked descriptor until
        // exec, keeping the lock alive after Drop. Run the close/reacquisition
        // assertion alone, without weakening its immediate-release contract.
        let thread = std::thread::current();
        let test_name = thread.name().expect("libtest names each test thread");
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test_name, "--test-threads=1"])
            .env(DRIVER, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "isolated descriptor lock test failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let fixture = Fixture::new();
    let path = fixture.root.join("lock");
    std::fs::write(&path, b"").unwrap();
    let held = File::open(&path).unwrap();
    rustix::fs::flock(&held, rustix::fs::FlockOperation::LockExclusive).unwrap();
    let acquired = File::open(&path).unwrap();
    let mut reports = 0;
    lock_exclusive_with_wait(&acquired, Duration::from_millis(1), |_| {
        reports += 1;
        rustix::fs::flock(&held, rustix::fs::FlockOperation::Unlock).unwrap();
        Ok(())
    })
    .unwrap();
    assert_eq!(reports, 1);
    assert_eq!(
        rustix::fs::flock(&held, rustix::fs::FlockOperation::NonBlockingLockExclusive),
        Err(rustix::io::Errno::WOULDBLOCK)
    );
    drop(acquired);
    lock_exclusive_with_wait(&held, Duration::from_millis(1), |_| {
        panic!("uncontended acquisition must not report")
    })
    .unwrap();
}

#[test]
fn callback_failure_and_invalid_intervals_do_not_acquire() {
    let fixture = Fixture::new();
    let path = fixture.root.join("lock");
    std::fs::write(&path, b"").unwrap();
    let held = File::open(&path).unwrap();
    rustix::fs::flock(&held, rustix::fs::FlockOperation::LockExclusive).unwrap();
    let pending = File::open(&path).unwrap();
    assert_eq!(
        lock_exclusive_with_wait(&pending, Duration::ZERO, |_| panic!("invalid interval"))
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(
        lock_exclusive_with_wait(&pending, Duration::from_millis(1), |_| Err(
            io::Error::from(io::ErrorKind::PermissionDenied)
        ))
        .unwrap_err()
        .kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(
        rustix::fs::flock(
            &pending,
            rustix::fs::FlockOperation::NonBlockingLockExclusive
        ),
        Err(rustix::io::Errno::WOULDBLOCK)
    );
}

#[test]
fn interrupted_acquisition_retries_without_false_contention() {
    let mut attempts = 0;
    wait_for_lock(
        Duration::from_millis(1),
        |_| panic!("interruption is not contention"),
        || {
            attempts += 1;
            if attempts < 3 {
                Err(io::Error::from(io::ErrorKind::Interrupted))
            } else {
                Ok(())
            }
        },
    )
    .unwrap();
    assert_eq!(attempts, 3);
}

#[test]
fn nonregular_descriptors_are_rejected_before_observation() {
    let fixture = Fixture::new();
    let directory = File::open(&fixture.root).unwrap();
    assert_eq!(
        lock_exclusive_with_wait(&directory, Duration::from_millis(1), |_| panic!(
            "nonregular descriptor"
        ))
        .unwrap_err()
        .kind(),
        io::ErrorKind::InvalidInput
    );
}
