use super::*;
use crate::tool::{
    CommunicationLimits, ExecutionFailure, SuccessfulExit, communicate_child_with_observer,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn direct_spawn_preserves_inherited_group_context_and_io() {
    let fixture = Fixture::new();
    let mut command = command("read value; printf '%s|%s|%s' \"$value\" \"$SELECTED\" \"$PWD\"");
    command
        .current_dir(&fixture.root)
        .env_clear()
        .env("SELECTED", "direct")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    let mut child = OwnedChild::spawn_direct(&mut command).unwrap();
    let pid = child.id();
    assert_eq!(
        rustix::process::getpgid(Pid::from_raw(i32::try_from(pid).unwrap())).unwrap(),
        rustix::process::getpgrp()
    );
    let mut observed = Vec::new();
    let evidence = communicate_child_with_observer(
        &mut child,
        Some(b"input\n"),
        crate::test_support::LIMITS,
        SuccessfulExit::Cleanup,
        || false,
        |_, bytes| observed.extend_from_slice(bytes),
    )
    .unwrap();
    assert_eq!(
        observed,
        format!("input|direct|{}", fixture.root.display()).as_bytes()
    );
    assert_eq!(evidence.stdout, observed);
    assert!(evidence.status.unwrap().success());
    assert_reaped(pid);
}

#[test]
fn direct_cleanup_preserves_other_members_of_the_selected_group() {
    let _fixture = Fixture::new();
    let mut keeper_command = command("cat");
    keeper_command.stdin(Stdio::piped()).stdout(Stdio::null());
    let mut keeper = OwnedChild::spawn(&mut keeper_command).unwrap();
    let group = i32::try_from(keeper.id()).unwrap();
    for case in ["success", "cancel", "overflow", "panic", "drop"] {
        let script = if case == "success" {
            "read value; printf ready"
        } else {
            "read value; printf ready; exec sleep 30"
        };
        let mut command = command(script);
        command
            .process_group(group)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        let mut child = OwnedChild::spawn_direct(&mut command).unwrap();
        let pid = child.id();
        assert_eq!(
            rustix::process::getpgid(Pid::from_raw(i32::try_from(pid).unwrap())).unwrap(),
            Pid::from_raw(group).unwrap()
        );
        if case != "drop" {
            let result = catch_unwind(AssertUnwindSafe(|| {
                communicate_child_with_observer(
                    &mut child,
                    Some(b"start\n"),
                    CommunicationLimits {
                        stdout_bytes: if case == "overflow" { 1 } else { 5 },
                        stderr_bytes: 0,
                        timeout: None,
                    },
                    SuccessfulExit::Cleanup,
                    || case == "cancel",
                    |_, _| assert_ne!(case, "panic", "observer panic"),
                )
            }));
            match case {
                "success" => assert_eq!(result.unwrap().unwrap().stdout, b"ready"),
                "cancel" | "overflow" => {
                    let error = result.unwrap().unwrap_err();
                    let execution = error.execution_error().unwrap();
                    assert!(
                        matches!(execution.failure,
                        ExecutionFailure::Cancelled if case == "cancel")
                            || matches!(execution.failure,
                        ExecutionFailure::OutputLimit { .. } if case == "overflow")
                    );
                    assert!(execution.group_error.is_none() && execution.wait_error.is_none());
                }
                "panic" => assert!(result.is_err()),
                _ => unreachable!(),
            }
            assert_reaped(pid);
        }
        drop(child);
        assert_reaped(pid);
        assert!(
            keeper.poll_exit().unwrap().is_none(),
            "{case} signalled another member"
        );
    }
    keeper.terminate().unwrap();
}
