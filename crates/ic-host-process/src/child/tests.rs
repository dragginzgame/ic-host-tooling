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

#[test]
fn successful_handoff_preserves_background_io_until_consumer_stop() {
    let _fixture = Fixture::new();
    // A separate inherited control pipe is the fixture's application stop
    // mechanism; it never signals a potentially reused PID after handoff.
    let mut command = command(
        "exec 3<&0; read gate; (while IFS= read -r line; do printf '%s\\n' \"$line\"; done) <&3 & exit 0",
    );
    command.stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child = OwnedChild::spawn(&mut command).unwrap();
    let pid = child.id();
    let mut stdin = child.take_stdin().unwrap();
    let mut stdout = child.take_stdout().unwrap();
    nonblocking(&stdout);
    assert!(child.poll_exit().unwrap().is_none());
    assert_eq!(
        child.handoff().unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    writeln!(stdin, "start").unwrap();
    until(|| child.poll_exit().unwrap().is_some());
    let observed = child.poll_exit().unwrap().unwrap();
    assert!(observed.success());
    // A second native observation proves polling reserved the leader for the
    // eventual disposition instead of losing group identity by reaping early.
    assert!(
        waitid(
            WaitId::Pid(Pid::from_raw(i32::try_from(pid).unwrap()).unwrap()),
            WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG,
        )
        .unwrap()
        .is_some()
    );
    assert_eq!(child.handoff().unwrap(), observed);
    assert_reaped(pid);
    assert_eq!(
        child.handoff().unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(child.poll_exit().unwrap(), Some(observed));
    assert_eq!(child.try_wait().unwrap(), Some(observed));
    assert_eq!(child.wait().unwrap(), observed);
    assert_eq!(child.terminate().unwrap(), observed);
    drop(child);
    writeln!(stdin, "still running after handoff and drop").unwrap();
    assert_eq!(
        read_until(&mut stdout, false),
        b"still running after handoff and drop\n"
    );
    drop(stdin); // Consumer-owned stop, after background work has been observed.
    assert_eq!(read_until(&mut stdout, true), b"");
}

#[test]
fn unsuccessful_handoff_retains_status_and_group_cleanup() {
    let _fixture = Fixture::new();
    for ending in [
        "exit 23",
        "exit 255",
        "kill -TERM $$",
        "ulimit -c 0; kill -ABRT $$",
    ] {
        let mut command = command(&format!("sleep 30 & {ending}"));
        command.stdout(Stdio::piped());
        let mut child = OwnedChild::spawn(&mut command).unwrap();
        let pid = child.id();
        let mut stdout = child.take_stdout().unwrap();
        nonblocking(&stdout);
        until(|| child.poll_exit().unwrap().is_some());
        let original = child.poll_exit().unwrap().unwrap();
        assert!(!original.success());
        assert_eq!(
            child.handoff().unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        // Original exit/signal survives cleanup. Failed handoff is not detach.
        assert_eq!(child.terminate().unwrap(), original);
        assert_eq!(read_until(&mut stdout, true), b"");
        assert_reaped(pid);
    }
}

#[test]
fn observed_success_still_cleans_on_cancellation_wait_and_unwind() {
    let _fixture = Fixture::new();
    for mode in ["terminate", "wait", "drop", "panic"] {
        let mut command = command("sleep 30 & exit 0");
        command.stdout(Stdio::piped());
        let mut child = OwnedChild::spawn(&mut command).unwrap();
        let pid = child.id();
        let mut stdout = child.take_stdout().unwrap();
        nonblocking(&stdout);
        until(|| child.poll_exit().unwrap().is_some());
        match mode {
            "terminate" => assert!(child.terminate().unwrap().success()),
            "wait" => assert!(child.wait().unwrap().success()),
            "drop" => drop(child),
            _ => assert!(
                std::panic::catch_unwind(move || {
                    let _owned = child;
                    panic!("caller rejected background IO before handoff");
                })
                .is_err()
            ),
        }
        assert_eq!(read_until(&mut stdout, true), b"");
        assert_reaped(pid);
    }
}

#[test]
fn handoff_refuses_external_reaping_without_signalling_reused_identity() {
    let _fixture = Fixture::new();
    let mut child = OwnedChild::spawn(&mut command("exit 0")).unwrap();
    until(|| child.poll_exit().unwrap().is_some());
    let pid = Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap();
    waitid(WaitId::Pid(pid), WaitIdOptions::EXITED).unwrap();
    assert_eq!(
        child.handoff().unwrap_err().raw_os_error(),
        Some(Errno::CHILD.raw_os_error())
    );
    let error = child.terminate().unwrap_err();
    assert!(error.status.is_none());
    assert!(error.kill_error.is_none());
    assert_eq!(
        error.group_error.unwrap().raw_os_error(),
        Some(Errno::CHILD.raw_os_error())
    );
    drop(child);
}

#[test]
fn term_grace_reserves_exited_leader_until_descendant_kill() {
    let fixture = Fixture::new();
    let marker = fixture.root.join("term");
    let mut command = command(
        "trap 'printf term > \"$MARKER\"; exit 0' TERM; \
         sh -c 'trap \"\" TERM; printf ready; exec sleep 30' & wait",
    );
    command.env("MARKER", &marker).stdout(Stdio::piped());
    let grace = Duration::from_millis(100);
    let mut child = OwnedChild::spawn_with_cleanup(
        &mut command,
        CleanupPolicy::TermThenKill {
            grace,
            reap_timeout: Duration::from_secs(2),
        },
    )
    .unwrap();
    let pid = child.id();
    let mut stdout = child.take_stdout().unwrap();
    let mut ready = [0; 5];
    stdout.read_exact(&mut ready).unwrap();
    assert_eq!(&ready, b"ready");
    nonblocking(&stdout);
    let started = Instant::now();
    assert!(child.terminate().unwrap().success());
    assert!(started.elapsed() >= grace);
    assert_eq!(std::fs::read(marker).unwrap(), b"term");
    // The leader handled TERM, but its TERM-ignoring descendant held this pipe
    // until the subsequent group KILL. Early leader reaping would lose authority.
    assert_eq!(read_until(&mut stdout, true), b"");
    assert_reaped(pid);
}

#[test]
fn bounded_policy_applies_to_termination_drop_and_unwinding() {
    let _fixture = Fixture::new();
    for mode in ["terminate", "drop", "panic"] {
        let mut command = command("trap '' TERM; printf ready; exec sleep 30");
        command.stdout(Stdio::piped());
        let grace = Duration::from_millis(25);
        let mut child = OwnedChild::spawn_with_cleanup(
            &mut command,
            CleanupPolicy::TermThenKill {
                grace,
                reap_timeout: Duration::from_secs(2),
            },
        )
        .unwrap();
        let pid = child.id();
        child
            .take_stdout()
            .unwrap()
            .read_exact(&mut [0; 5])
            .unwrap();
        let started = Instant::now();
        match mode {
            "terminate" => assert_eq!(child.terminate().unwrap().signal(), Some(9)),
            "drop" => drop(child),
            _ => assert!(
                std::panic::catch_unwind(move || {
                    let _owned = child;
                    panic!("caller failed during admission");
                })
                .is_err()
            ),
        }
        assert!(started.elapsed() >= grace);
        assert_reaped(pid);
    }
}

#[test]
fn exhausted_reap_allowance_is_not_restarted_by_return_or_drop() {
    let _fixture = Fixture::new();
    let mut child = OwnedChild::spawn_with_cleanup(
        &mut command("exec sleep 30"),
        CleanupPolicy::TermThenKill {
            grace: Duration::from_secs(30),
            reap_timeout: Duration::from_millis(30),
        },
    )
    .unwrap();
    let pid = Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap();
    // Substitute already-dispatched escalation while the real child is still
    // running. This tests pending reaping without inducing an unkillable OS task.
    child.reap_started = Some(Instant::now());
    let started = Instant::now();
    let error = child.terminate().unwrap_err();
    assert_eq!(error.wait_error.unwrap().kind(), io::ErrorKind::TimedOut);
    assert!(started.elapsed() >= Duration::from_millis(30));
    assert!(child.owned);
    assert!(child.handoff().is_err());
    assert_eq!(
        child.terminate().unwrap_err().wait_error.unwrap().kind(),
        io::ErrorKind::TimedOut
    );
    drop(child);
    assert!(started.elapsed() < Duration::from_secs(2));
    // The caller still owes recovery after a reap timeout; no detached thread or
    // silently blocking destructor took that obligation. Clean this test child.
    kill_process_group(pid, Signal::KILL).unwrap();
    waitid(WaitId::Pid(pid), WaitIdOptions::EXITED).unwrap();
}

#[test]
fn bounded_policy_keeps_successful_handoff_explicit() {
    let fixture = Fixture::new();
    let marker = fixture.root.join("survived");
    let mut command = command("(sleep 0.1; printf alive > \"$MARKER\") & exit 0");
    command.env("MARKER", &marker);
    let mut child = OwnedChild::spawn_with_cleanup(
        &mut command,
        CleanupPolicy::TermThenKill {
            grace: Duration::ZERO,
            reap_timeout: Duration::ZERO,
        },
    )
    .unwrap();
    until(|| child.poll_exit().unwrap().is_some());
    assert!(child.handoff().unwrap().success());
    drop(child);
    until(|| std::fs::read(&marker).is_ok_and(|bytes| bytes == b"alive"));
}

#[test]
fn bounded_termination_accepts_a_reserved_sole_zombie() {
    let _fixture = Fixture::new();
    let mut child = OwnedChild::spawn_with_cleanup(
        &mut command("exit 0"),
        CleanupPolicy::TermThenKill {
            grace: Duration::from_millis(25),
            reap_timeout: Duration::ZERO,
        },
    )
    .unwrap();
    until(|| child.poll_exit().unwrap().is_some());
    // Darwin may refuse signalling a group containing only its zombie leader.
    // Both TERM and KILL must use the existing conservative native inspection.
    assert!(child.terminate().unwrap().success());
    assert_reaped(child.id());
}
