use super::{lock_exclusive_with_wait, wait_for_lock};
use crate::test_support::Fixture;
use std::{fs::File, io, time::Duration};

#[test]
fn descriptor_contention_reports_and_preserves_returned_ownership() {
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
