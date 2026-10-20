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
fn marker_publication_failure_reaps_the_child() {
    let fixture = Fixture::new();
    super::publish_marker(&fixture.path().join("barrier-ready"), b"ready\n").unwrap();
    fs::create_dir(fixture.path().join("barrier-acknowledged")).unwrap();
    let mut child = Command::new("/bin/sh")
        .args(["-c", "exec sleep 30"])
        .spawn()
        .unwrap();

    assert!(
        kill_child_at_acknowledged_barrier(&mut child, fixture.path(), Duration::from_secs(5))
            .is_err()
    );
    assert!(child.try_wait().unwrap().is_some());
    assert!(!fixture.path().join("barrier-armed").exists());
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
