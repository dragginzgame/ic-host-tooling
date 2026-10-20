use super::{
    hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier, wait_for_child_path,
    wait_for_path,
};
use crate::test_support::Fixture;
use std::{fs, io, process::Command, time::Duration};

#[test]
fn acknowledged_barrier_kills_only_after_the_child_is_armed() {
    const ROOT_ENV: &str = "IC_HOST_PROCESS_CRASH_BARRIER_ROOT";
    if let Some(root) = std::env::var_os(ROOT_ENV) {
        hold_at_acknowledged_barrier(std::path::Path::new(&root), Duration::from_secs(5));
    }
    let fixture = Fixture::new();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "testing::crash_barrier::tests::acknowledged_barrier_kills_only_after_the_child_is_armed",
            "--nocapture",
        ])
        .env(ROOT_ENV, fixture.path())
        .spawn()
        .unwrap();
    kill_child_at_acknowledged_barrier(&mut child, fixture.path(), Duration::from_secs(5)).unwrap();
    assert_eq!(
        fs::read(fixture.path().join("barrier-ready")).unwrap(),
        b"ready\n"
    );
    assert_eq!(
        fs::read(fixture.path().join("barrier-acknowledged")).unwrap(),
        b"acknowledged\n"
    );
    assert_eq!(
        fs::read(fixture.path().join("barrier-armed")).unwrap(),
        b"armed\n"
    );
}

#[test]
fn missing_marker_times_out_without_creating_it() {
    let fixture = Fixture::new();
    let missing = fixture.path().join("absent");
    assert_eq!(
        wait_for_path(&missing, "missing", Duration::ZERO)
            .unwrap_err()
            .kind(),
        io::ErrorKind::TimedOut
    );
    assert!(!missing.exists());
    drop(fixture);
}

#[test]
fn marker_precedes_child_exit_and_child_exit_precedes_timeout() {
    let fixture = Fixture::new();
    let marker = fixture.path().join("ready");
    fs::write(&marker, b"ready\n").unwrap();
    let mut child = Command::new("/bin/sh")
        .args(["-c", "exit 17"])
        .spawn()
        .unwrap();
    assert_eq!(child.wait().unwrap().code(), Some(17));

    wait_for_path(&marker, "ready", Duration::ZERO).unwrap();
    wait_for_child_path(&mut child, &marker, "ready", Duration::ZERO).unwrap();
    let missing = fixture.path().join("absent");
    assert_eq!(
        wait_for_child_path(&mut child, &missing, "missing", Duration::ZERO)
            .unwrap_err()
            .kind(),
        io::ErrorKind::UnexpectedEof
    );
    assert!(!missing.exists());
}

#[test]
fn handshake_requires_complete_marker_bytes_but_path_waits_keep_their_contract() {
    let fixture = Fixture::new();
    let marker = fixture.path().join("barrier-armed");
    for bytes in [b"".as_slice(), b"arm", b"armed", b"wrong\n"] {
        fs::write(&marker, bytes).unwrap();
        wait_for_path(&marker, "existing file", Duration::ZERO).unwrap();
        assert_eq!(
            super::wait_for_marker(
                &marker,
                "armed child barrier",
                Duration::ZERO,
                Some(b"armed\n"),
                || Ok(())
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::TimedOut,
        );
    }
    fs::write(&marker, b"armed\n").unwrap();
    super::wait_for_marker(
        &marker,
        "armed child barrier",
        Duration::ZERO,
        Some(b"armed\n"),
        || panic!("complete marker precedes child observation"),
    )
    .unwrap();
}
