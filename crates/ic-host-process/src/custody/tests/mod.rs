//! Native regressions adapted from Backup's command-custody boundary.

use super::*;
use crate::test_support::Fixture;
use rustix::io::{FdFlags, fcntl_getfd};
use std::{
    os::unix::fs::{PermissionsExt, symlink},
    process::Stdio,
};

const DESCRIPTOR_ENV: &str = "HOST_TEST_CUSTODY_FD";
const ROOT_ENV: &str = "HOST_TEST_CUSTODY_ROOT";
const ROLE_ENV: &str = "HOST_TEST_CUSTODY_ROLE";
const TREE_CASE: &str = "custody::tests::owner_death_retains_custody_until_last_descendant_exits";

fn until(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !ready() {
        assert!(Instant::now() < deadline, "custody fixture did not settle");
        thread::sleep(Duration::from_millis(5));
    }
}

fn wait_file(path: &Path) {
    until(|| path.is_file());
}

fn command_for(root: &Path, role: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", TREE_CASE, "--nocapture"])
        .env(ROOT_ENV, root)
        .env(ROLE_ENV, role)
        .stdin(Stdio::null());
    command
}

#[test]
fn one_spawn_preserves_identity_permissions_and_owner_cloexec() {
    let fixture = Fixture::new();
    let path = fixture.path().join("sidecar");
    let mut custody = CommandCustody::acquire(&path).unwrap();
    let identity = custody.identity();
    assert_eq!(custody.path(), path);
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(
        fcntl_getfd(&custody.file)
            .unwrap()
            .contains(FdFlags::CLOEXEC)
    );
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "test -f /dev/fd/$HOST_TEST_CUSTODY_FD"]);
    assert!(
        custody
            .spawn(command, DESCRIPTOR_ENV)
            .unwrap()
            .wait()
            .unwrap()
            .success()
    );
    assert!(
        fcntl_getfd(&custody.file)
            .unwrap()
            .contains(FdFlags::CLOEXEC)
    );
    assert!(matches!(
        custody.spawn(Command::new("unused"), DESCRIPTOR_ENV),
        Err(CustodyError::AlreadyDispatched { .. })
    ));
    let guard = custody
        .finish(Duration::from_secs(5), Duration::from_millis(5))
        .unwrap();
    assert_eq!(guard.identity(), identity);
    assert!(matches!(
        CommandCustody::acquire(&path),
        Err(CustodyError::InFlight { .. })
    ));
    drop(guard);
    assert_eq!(CommandCustody::acquire(&path).unwrap().identity(), identity);
}

#[test]
fn failed_spawn_consumes_allowance_and_closes_command_copy() {
    let fixture = Fixture::new();
    let mut custody = CommandCustody::acquire(&fixture.path().join("sidecar")).unwrap();
    assert!(
        matches!(custody.spawn(Command::new(fixture.path().join("absent")), DESCRIPTOR_ENV), Err(CustodyError::Io(e)) if e.kind() == io::ErrorKind::NotFound)
    );
    assert!(matches!(
        custody.spawn(Command::new("unused"), DESCRIPTOR_ENV),
        Err(CustodyError::AlreadyDispatched { .. })
    ));
    custody
        .finish(Duration::from_secs(5), Duration::from_millis(1))
        .unwrap();
}

#[test]
fn quiescence_drop_unlocks_even_with_a_transient_descriptor_copy() {
    let fixture = Fixture::new();
    let path = fixture.path().join("sidecar");
    let custody = CommandCustody::acquire(&path).unwrap();
    let identity = custody.identity();
    let guard = custody
        .finish(Duration::from_secs(5), Duration::from_millis(1))
        .unwrap();
    let retained = guard.file.try_clone().unwrap();
    assert!(matches!(
        QuiescenceGuard::acquire(&path, identity),
        Err(CustodyError::InFlight { .. })
    ));
    drop(guard);
    let fresh = QuiescenceGuard::acquire(&path, identity).unwrap();
    assert_eq!(FileIdentity::from_file(&retained).unwrap(), identity);
    drop(retained);
    assert!(matches!(
        QuiescenceGuard::acquire(&path, identity),
        Err(CustodyError::InFlight { .. })
    ));
    drop(fresh);
}

#[test]
fn replaced_missing_and_unsafe_sidecars_never_prove_quiescence() {
    let fixture = Fixture::new();
    let path = fixture.path().join("sidecar");
    let retained = fixture.path().join("original");
    let mut custody = CommandCustody::acquire(&path).unwrap();
    let identity = custody.identity();
    fs::rename(&path, &retained).unwrap();
    fs::write(&path, b"replacement evidence").unwrap();
    let replacement = CommandCustody::acquire(&path).unwrap();
    assert!(matches!(
        custody.spawn(Command::new("unused"), DESCRIPTOR_ENV),
        Err(CustodyError::IdentityChanged { .. })
    ));
    assert!(matches!(
        QuiescenceGuard::acquire(&path, identity),
        Err(CustodyError::IdentityChanged { .. })
    ));
    assert!(matches!(
        custody.finish(Duration::ZERO, Duration::from_millis(1)),
        Err(CustodyError::IdentityChanged { .. })
    ));
    assert_eq!(fs::read(&path).unwrap(), b"replacement evidence");
    drop(replacement);
    fs::remove_file(&path).unwrap();
    assert!(
        matches!(QuiescenceGuard::acquire(&path, identity), Err(CustodyError::Io(e)) if e.kind() == io::ErrorKind::NotFound)
    );
    assert!(!path.exists());
    symlink(&retained, &path).unwrap();
    assert!(matches!(
        QuiescenceGuard::acquire(&path, identity),
        Err(CustodyError::Lock(RegularFileLockError::NotRegular))
    ));
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(matches!(
        CommandCustody::acquire(&path),
        Err(CustodyError::Lock(RegularFileLockError::NotRegular))
    ));
}

#[test]
fn existing_bytes_and_permissions_survive_and_missing_parents_are_not_created() {
    let fixture = Fixture::new();
    let path = fixture.path().join("sidecar");
    fs::write(&path, b"retained bytes").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    let original = FileIdentity::from_file(&File::open(&path).unwrap()).unwrap();
    let custody = CommandCustody::acquire(&path).unwrap();
    assert_eq!(custody.identity(), original);
    assert_eq!(fs::read(&path).unwrap(), b"retained bytes");
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o640
    );
    assert!(CommandCustody::acquire(&fixture.path().join("missing/sidecar")).is_err());
    assert!(!fixture.path().join("missing").exists());
    assert!(
        matches!(custody.finish(Duration::ZERO, Duration::ZERO), Err(CustodyError::Io(e)) if e.kind() == io::ErrorKind::InvalidInput)
    );
}

#[test]
fn owner_death_retains_custody_until_last_descendant_exits() {
    if let Some(root) = std::env::var_os(ROOT_ENV) {
        let root = PathBuf::from(root);
        match std::env::var(ROLE_ENV).unwrap().as_str() {
            "owner" => {
                let mut custody = CommandCustody::acquire(&root.join("sidecar")).unwrap();
                let mut child = custody
                    .spawn(command_for(&root, "direct"), DESCRIPTOR_ENV)
                    .unwrap();
                wait_file(&root.join("direct-ready"));
                fs::write(root.join("owner-ready"), b"ready").unwrap();
                wait_file(&root.join("never-release-owner"));
                child.wait().unwrap();
            }
            "direct" => {
                let descriptor = std::env::var(DESCRIPTOR_ENV).unwrap();
                assert!(
                    fs::metadata(format!("/dev/fd/{descriptor}"))
                        .unwrap()
                        .is_file()
                );
                let _descendant = command_for(&root, "descendant").spawn().unwrap();
                wait_file(&root.join("descendant-ready"));
                fs::write(root.join("direct-ready"), b"ready").unwrap();
                wait_file(&root.join("allow-direct-exit"));
                std::process::exit(0);
            }
            "descendant" => {
                let direct = rustix::process::getppid();
                fs::write(root.join("descendant-ready"), b"ready").unwrap();
                until(|| rustix::process::getppid() != direct);
                fs::write(root.join("direct-exited"), b"reparented").unwrap();
                wait_file(&root.join("allow-descendant-exit"));
            }
            "holder" => {
                fs::write(root.join("holder-ready"), b"ready").unwrap();
                wait_file(&root.join("allow-holder-exit"));
            }
            role => panic!("unknown custody role: {role}"),
        }
        return;
    }
    let fixture = Fixture::new();
    let root = fixture.path();
    let path = root.join("sidecar");
    let mut owner = command_for(root, "owner").spawn().unwrap();
    wait_file(&root.join("owner-ready"));
    let identity = FileIdentity::from_file(&File::open(&path).unwrap()).unwrap();
    assert!(matches!(
        QuiescenceGuard::acquire(&path, identity),
        Err(CustodyError::InFlight { .. })
    ));
    owner.kill().unwrap();
    owner.wait().unwrap();
    assert!(matches!(
        QuiescenceGuard::acquire(&path, identity),
        Err(CustodyError::InFlight { .. })
    ));
    fs::write(root.join("allow-direct-exit"), b"exit").unwrap();
    wait_file(&root.join("direct-exited"));
    assert!(matches!(
        QuiescenceGuard::acquire(&path, identity),
        Err(CustodyError::InFlight { .. })
    ));
    fs::write(root.join("allow-descendant-exit"), b"exit").unwrap();
    let mut quiescent = None;
    until(|| match QuiescenceGuard::acquire(&path, identity) {
        Ok(guard) => {
            quiescent = Some(guard);
            true
        }
        Err(CustodyError::InFlight { .. }) => false,
        other => panic!("unexpected final custody: {other:?}"),
    });
    assert_eq!(quiescent.unwrap().identity(), identity);
}

#[test]
fn finish_stops_after_grace_while_an_inherited_holder_remains() {
    let fixture = Fixture::new();
    let path = fixture.path().join("sidecar");
    let mut custody = CommandCustody::acquire(&path).unwrap();
    let identity = custody.identity();
    let mut child = custody
        .spawn(command_for(fixture.path(), "holder"), DESCRIPTOR_ENV)
        .unwrap();
    wait_file(&fixture.path().join("holder-ready"));
    let started = Instant::now();
    assert!(matches!(
        custody.finish(Duration::from_millis(25), Duration::from_millis(5)),
        Err(CustodyError::InFlight { .. })
    ));
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(matches!(
        QuiescenceGuard::acquire(&path, identity),
        Err(CustodyError::InFlight { .. })
    ));
    fs::write(fixture.path().join("allow-holder-exit"), b"exit").unwrap();
    assert!(child.wait().unwrap().success());
    until(|| match QuiescenceGuard::acquire(&path, identity) {
        Ok(_) => true,
        Err(CustodyError::InFlight { .. }) => false,
        other => panic!("unexpected custody after exit: {other:?}"),
    });
}
