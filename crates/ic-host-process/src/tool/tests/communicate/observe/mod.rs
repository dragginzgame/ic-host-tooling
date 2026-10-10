use super::*;

#[test]
fn output_is_observed_before_child_exit_and_descendant_pipe_cleanup() {
    for timeout in [LIMITS.timeout, None] {
        let limits = OutputLimits { timeout, ..LIMITS };
        let fixture = Fixture::new();
        let go = fixture.root.join("go");
        let mut command = command(
            "printf '\\377'; while ! test -f \"$GO\"; do sleep 0.01; done; sleep 30 & exit 0",
        );
        command.env("GO", &go);
        let mut child = OwnedChild::spawn(&mut command).unwrap();
        let mut observed = Vec::new();
        let started = Instant::now();
        let evidence = communicate_child_with_observer(
            &mut child,
            None,
            limits,
            SuccessfulExit::Cleanup,
            || started.elapsed() > LIMITS.timeout.unwrap(),
            |stream, bytes| {
                assert_eq!(stream, OutputStream::Stdout);
                observed.extend_from_slice(bytes);
                fs::write(&go, b"go").unwrap();
            },
        )
        .unwrap();
        assert_eq!(observed, [255]);
        assert_eq!(evidence.stdout, observed);
        assert!(evidence.status.unwrap().success());
        assert_reaped(&child);
    }
}

#[test]
fn observer_receives_only_retained_prefix_on_each_stream_overflow() {
    for timeout in [LIMITS.timeout, None] {
        let limits = OutputLimits { timeout, ..LIMITS };
        let _fixture = Fixture::new();
        for (script, expected) in [
            ("printf 12345; sleep 30", OutputStream::Stdout),
            ("printf 12345 >&2; sleep 30", OutputStream::Stderr),
        ] {
            for maximum in [0, 4] {
                let mut child = OwnedChild::spawn(&mut command(script)).unwrap();
                let mut observed = Vec::new();
                let error = communicate_child_with_observer(
                    &mut child,
                    None,
                    OutputLimits {
                        stdout: crate::tool::OutputLimit::Terminate(maximum),
                        stderr: crate::tool::OutputLimit::Terminate(maximum),
                        ..limits
                    },
                    SuccessfulExit::Cleanup,
                    || false,
                    |stream, bytes| {
                        assert_eq!(stream, expected);
                        assert_ne!(bytes, []);
                        observed.extend_from_slice(bytes);
                    },
                )
                .unwrap_err();
                let execution = error.execution_error().unwrap();
                assert!(
                    matches!(execution.failure, ExecutionFailure::OutputLimit { stream } if stream == expected)
                );
                assert_eq!(observed, &b"12345"[..maximum]);
                let retained = match expected {
                    OutputStream::Stdout => &execution.evidence.stdout,
                    OutputStream::Stderr => &execution.evidence.stderr,
                };
                assert_eq!(&observed, retained);
                assert!(execution.cleanup.is_none());
                assert_reaped(&child);
            }
        }
    }
}

#[test]
fn silent_child_allows_caller_progress_and_cancellation_without_output() {
    for timeout in [LIMITS.timeout, None] {
        let limits = OutputLimits { timeout, ..LIMITS };
        let _fixture = Fixture::new();
        let mut child = OwnedChild::spawn(&mut command("sleep 30")).unwrap();
        let mut polls = 0;
        let error = communicate_child_with_observer(
            &mut child,
            None,
            limits,
            SuccessfulExit::Cleanup,
            || {
                polls += 1;
                polls == 3
            },
            |_, _| panic!("silent child emitted output"),
        )
        .unwrap_err();
        assert!(matches!(
            error.execution_error().unwrap().failure,
            ExecutionFailure::Cancelled
        ));
        assert_eq!(polls, 3);
        assert_reaped(&child);
    }
}

#[test]
fn callback_unwind_cleans_group_even_when_borrowed_owner_survives_catch() {
    use std::{
        io::Read,
        panic::{AssertUnwindSafe, catch_unwind},
    };
    for timeout in [LIMITS.timeout, None] {
        let limits = OutputLimits { timeout, ..LIMITS };
        let _fixture = Fixture::new();
        for panic_in_output in [false, true] {
            let mut child =
                OwnedChild::spawn(&mut command("sleep 30 & printf ready; wait")).unwrap();
            // Retain a separate pipe as evidence that descendants release writers.
            let mut stderr = child.take_stderr().unwrap();
            let flags = rustix::fs::fcntl_getfl(&stderr).unwrap();
            rustix::fs::fcntl_setfl(&stderr, flags | rustix::fs::OFlags::NONBLOCK).unwrap();
            let panic = catch_unwind(AssertUnwindSafe(|| {
                let _ = communicate_child_with_observer(
                    &mut child,
                    None,
                    limits,
                    SuccessfulExit::Cleanup,
                    || {
                        assert!(panic_in_output, "cancel callback panic");
                        false
                    },
                    |_, _| panic!("output callback panic"),
                );
            }))
            .unwrap_err();
            assert_eq!(
                panic.downcast_ref::<&str>(),
                Some(&if panic_in_output {
                    "output callback panic"
                } else {
                    "cancel callback panic"
                })
            );
            assert_reaped(&child);
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                match stderr.read(&mut [0]) {
                    Ok(0) => break,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline);
                        std::thread::sleep(Duration::from_millis(2));
                    }
                    other => panic!("unexpected pipe result: {other:?}"),
                }
            }
        }
    }
}

#[test]
fn truncation_forwards_all_bytes_with_bounded_or_zero_storage() {
    let _fixture = Fixture::new();
    let input = vec![b'x'; 1024 * 1024];
    for retained in [0, 17] {
        let mut child = OwnedChild::spawn(&mut command("cat; printf diagnostic >&2")).unwrap();
        let mut counts = [0, 0];
        let evidence = communicate_child_with_observer(
            &mut child,
            Some(&input),
            OutputLimits {
                stdout: OutputLimit::Truncate(retained),
                stderr: OutputLimit::Truncate(0),
                timeout: LIMITS.timeout,
            },
            SuccessfulExit::Cleanup,
            || false,
            |stream, bytes| match stream {
                OutputStream::Stdout => {
                    assert!(bytes.iter().all(|&byte| byte == b'x'));
                    counts[0] += bytes.len();
                }
                OutputStream::Stderr => counts[1] += bytes.len(),
            },
        )
        .unwrap();
        assert_eq!(counts, [input.len(), b"diagnostic".len()]);
        assert_eq!(evidence.stdout, input[..retained]);
        assert!(evidence.stdout.capacity() <= retained);
        assert_eq!(evidence.stderr.capacity(), 0);
        assert!(evidence.stdout_truncated && evidence.stderr_truncated);
        assert!(evidence.status.unwrap().success());
        assert_reaped(&child);
    }
}

#[test]
fn truncated_output_preserves_failure_and_cancellation() {
    let _fixture = Fixture::new();
    for cancel in [false, true] {
        let mut child = OwnedChild::spawn(&mut command(if cancel {
            "printf ready; sleep 30"
        } else {
            "printf ready; exit 7"
        }))
        .unwrap();
        let observed = std::cell::Cell::new(0);
        let error = communicate_child_with_observer(
            &mut child,
            None,
            OutputLimits {
                stdout: OutputLimit::Truncate(2),
                ..LIMITS
            },
            SuccessfulExit::Cleanup,
            || cancel && observed.get() > 0,
            |_, bytes| observed.set(observed.get() + bytes.len()),
        )
        .unwrap_err();
        let execution = error.execution_error().unwrap();
        assert_eq!(execution.evidence.stdout, b"re");
        assert!(execution.evidence.stdout_truncated);
        if cancel {
            assert!(matches!(execution.failure, ExecutionFailure::Cancelled));
        } else {
            assert!(matches!(execution.failure, ExecutionFailure::ExitStatus));
            assert_eq!(execution.evidence.status.unwrap().code(), Some(7));
        }
        assert!(execution.cleanup.is_none());
        assert_reaped(&child);
    }
}
