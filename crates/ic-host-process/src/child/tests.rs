#![allow(
    clippy::significant_drop_tightening,
    reason = "fixture lock covers all child operations"
)]

use super::*;
use crate::test_support::Fixture;
use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
use std::{
    io::{Read, Write},
    os::fd::AsFd,
    process::Stdio,
    time::{Duration, Instant},
};

fn command(script: &str) -> Command {
    let mut command = Command::new("/bin/sh");
    command.args(["-c", script]);
    command
}

fn nonblocking(pipe: &impl AsFd) {
    let flags = fcntl_getfl(pipe).unwrap();
    fcntl_setfl(pipe, flags | OFlags::NONBLOCK).unwrap();
}

fn until(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ready() {
        assert!(Instant::now() < deadline, "child fixture did not complete");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn read_until(pipe: &mut impl Read, eof: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    until(|| {
        let mut buffer = [0; 256];
        match pipe.read(&mut buffer) {
            Ok(0) => true,
            Ok(count) => {
                bytes.extend_from_slice(&buffer[..count]);
                !eof
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => false,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => false,
            Err(error) => panic!("fixture pipe read failed: {error}"),
        }
    });
    bytes
}

fn assert_reaped(pid: u32) {
    let pid = Pid::from_raw(i32::try_from(pid).unwrap()).unwrap();
    assert_eq!(
        waitid(
            WaitId::Pid(pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG
        )
        .unwrap_err(),
        Errno::CHILD
    );
}

#[test]
fn command_context_and_caller_owned_pipes_are_preserved() {
    let fixture = Fixture::new();
    let mut command = command(
        "read value; printf '%s|%s|%s' \"$value\" \"$SELECTED\" \"$PWD\"; printf error >&2; exit 23",
    );
    command
        .process_group(rustix::process::getpgrp().as_raw_nonzero().get())
        .current_dir(&fixture.root)
        .env_clear()
        .env("SELECTED", "explicit")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = OwnedChild::spawn(&mut command).unwrap();
    let pid = child.id();
    assert_eq!(
        rustix::process::getpgid(Pid::from_raw(i32::try_from(pid).unwrap()))
            .unwrap()
            .as_raw_nonzero()
            .get(),
        i32::try_from(pid).unwrap()
    );
    let mut stdin = child.take_stdin().unwrap();
    let mut stdout = child.take_stdout().unwrap();
    let mut stderr = child.take_stderr().unwrap();
    nonblocking(&stdout);
    nonblocking(&stderr);
    assert!(child.take_stdin().is_none());
    assert!(child.try_wait().unwrap().is_none());
    writeln!(stdin, "caller input").unwrap();
    drop(stdin);
    let output = read_until(&mut stdout, true);
    let errors = read_until(&mut stderr, true);
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(23));
    assert_eq!(child.try_wait().unwrap(), Some(status));
    assert_eq!(child.terminate().unwrap(), status);
    assert_eq!(
        output,
        format!("caller input|explicit|{}", fixture.root.display()).as_bytes()
    );
    assert_eq!(errors, b"error");
    assert_reaped(pid);
}

#[test]
fn natural_exit_cleans_descendants_before_reaping_and_caches_status() {
    let _fixture = Fixture::new();
    let mut command = command("sleep 30 & printf ready; exit 17");
    command.stdout(Stdio::piped());
    let mut child = OwnedChild::spawn(&mut command).unwrap();
    let pid = child.id();
    let mut stdout = child.take_stdout().unwrap();
    nonblocking(&stdout);
    assert_eq!(read_until(&mut stdout, false), b"ready");
    let mut status = None;
    until(|| {
        status = child.try_wait().unwrap();
        status.is_some()
    });
    assert_eq!(status.unwrap().code(), Some(17));
    // The descendant inherited stdout. EOF proves it no longer holds that pipe.
    assert_eq!(read_until(&mut stdout, true), b"");
    assert_eq!(child.wait().unwrap(), status.unwrap());
    assert_eq!(child.terminate().unwrap(), status.unwrap());
    assert_reaped(pid);
}

#[test]
fn explicit_termination_drop_and_unwind_close_descendant_pipes() {
    let _fixture = Fixture::new();
    for mode in ["terminate", "drop", "panic"] {
        let mut command = command("sleep 30 & printf ready; wait");
        command.stdout(Stdio::piped());
        let mut child = OwnedChild::spawn(&mut command).unwrap();
        let pid = child.id();
        let mut stdout = child.take_stdout().unwrap();
        nonblocking(&stdout);
        assert_eq!(read_until(&mut stdout, false), b"ready");
        assert!(child.try_wait().unwrap().is_none());
        match mode {
            "terminate" => {
                assert!(!child.terminate().unwrap().success());
                drop(child);
            }
            "drop" => drop(child),
            _ => {
                let result = std::panic::catch_unwind(move || {
                    let _owned = child;
                    panic!("caller cancellation callback failed");
                });
                assert!(result.is_err());
            }
        }
        assert_eq!(read_until(&mut stdout, true), b"");
        assert_reaped(pid);
    }
}

#[test]
fn failed_spawn_preserves_native_error() {
    let fixture = Fixture::new();
    let path = fixture.root.join("missing");
    let expected = Command::new(&path).spawn().err().unwrap();
    let actual = OwnedChild::spawn(&mut Command::new(&path)).err().unwrap();
    assert_eq!(actual.raw_os_error(), expected.raw_os_error());
}

#[test]
fn interrupted_inspection_retries_but_other_errors_remain_native() {
    let mut calls = 0;
    let result = retry_interrupted(|| {
        calls += 1;
        if calls < 3 {
            Err(io::ErrorKind::Interrupted.into())
        } else {
            Err::<(), _>(io::Error::from_raw_os_error(13))
        }
    });
    assert_eq!(calls, 3);
    assert_eq!(result.unwrap_err().raw_os_error(), Some(13));
}

#[test]
fn external_reaping_loses_ownership_and_retains_cleanup_failures() {
    let _fixture = Fixture::new();
    let mut child = OwnedChild::spawn(&mut command("exit 0")).unwrap();
    let pid = Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap();
    // Deliberately violate exclusive reaping to exercise the fail-closed boundary.
    waitid(WaitId::Pid(pid), WaitIdOptions::EXITED).unwrap();
    assert_eq!(
        child.try_wait().unwrap_err().raw_os_error(),
        Some(Errno::CHILD.raw_os_error())
    );
    let error = child.terminate().unwrap_err();
    assert!(error.status.is_none());
    assert!(error.kill_error.is_none());
    assert_eq!(
        error.group_error.as_ref().unwrap().raw_os_error(),
        Some(Errno::CHILD.raw_os_error())
    );
    assert_eq!(
        error.wait_error.as_ref().unwrap().raw_os_error(),
        Some(Errno::CHILD.raw_os_error())
    );
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<io::Error>()
            .is_some()
    );
    drop(child);
}
