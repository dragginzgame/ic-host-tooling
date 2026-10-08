use super::*;
use crate::test_support::Fixture;
use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};

#[test]
fn fragmented_capture_keeps_capacity_and_overflow_evidence_within_budget() {
    struct Fragmented<'a>(&'a [u8]);
    impl Read for Fragmented<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let count = buffer.len().min(1);
            self.0.read(&mut buffer[..count])
        }
    }
    for limit in [0, 1, 3, 37, 257] {
        let input = vec![7; limit + 1];
        let mut reader = Some(Fragmented(&input));
        let mut bytes = Vec::new();
        let (mut eof, mut truncated) = (false, false);
        for _ in 0..limit {
            assert!(
                read_chunk(
                    &mut reader,
                    &mut bytes,
                    limit,
                    OutputStream::Stdout,
                    &mut eof,
                    &mut truncated
                )
                .unwrap()
            );
            assert!(bytes.capacity() <= limit);
        }
        assert!(matches!(
            read_chunk(
                &mut reader,
                &mut bytes,
                limit,
                OutputStream::Stdout,
                &mut eof,
                &mut truncated
            ),
            Err(ExecutionFailure::OutputLimit {
                stream: OutputStream::Stdout
            })
        ));
        assert_eq!(bytes, input[..limit]);
        assert!(truncated);
        assert!(!eof);
        assert!(bytes.capacity() <= limit);
    }
}

#[test]
fn io_failure_cleans_the_group_and_keeps_the_original_error() {
    let _fixture = Fixture::new();
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "sleep 30 & printf ready; wait"])
        .stdout(Stdio::piped());
    let mut child = OwnedChild::spawn(&mut command).unwrap();
    let mut stdout = child.take_stdout().unwrap();
    let mut ready = [0; 5];
    stdout.read_exact(&mut ready).unwrap();
    assert_eq!(&ready, b"ready");
    nonblocking(&stdout).unwrap();
    // Substitute an OS read failure at the capture finalization boundary; the
    // child/group and cleanup are real, with no production fault hook.
    let error = finish_capture(
        &mut child,
        ExecutionEvidence::default(),
        Err(io_failure(
            ExecutionOperation::ReadOutput,
            io::Error::from_raw_os_error(5),
        )),
    )
    .unwrap_err();
    assert!(
        matches!(&error.failure, ExecutionFailure::Io { operation: ExecutionOperation::ReadOutput, source } if source.raw_os_error() == Some(5))
    );
    assert!(
        error.group_error.is_none() && error.kill_error.is_none() && error.wait_error.is_none()
    );
    assert!(error.evidence.status.is_some());
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match stdout.read(&mut [0; 1]) {
            Ok(0) => break, // Descendant no longer holds the inherited writer.
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline);
                std::thread::sleep(POLL_INTERVAL);
            }
            result => panic!("unexpected fixture read: {result:?}"),
        }
    }
}

#[test]
fn cleanup_ownership_failure_does_not_replace_original_timeout() {
    let _fixture = Fixture::new();
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "exit 0"]);
    let mut child = OwnedChild::spawn(&mut command).unwrap();
    let pid = Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap();
    // Deliberately break exclusive reaping to provoke real typed cleanup errors;
    // there are no descendants and no signalling through a reused identity.
    waitid(WaitId::Pid(pid), WaitIdOptions::EXITED).unwrap();
    assert!(child.poll_exit().is_err());
    let error = finish_capture(
        &mut child,
        ExecutionEvidence::default(),
        Err(ExecutionFailure::TimedOut),
    )
    .unwrap_err();
    assert!(matches!(error.failure, ExecutionFailure::TimedOut));
    assert_eq!(
        error.group_error.unwrap().raw_os_error(),
        Some(rustix::io::Errno::CHILD.raw_os_error())
    );
    assert!(error.kill_error.is_none());
    assert_eq!(
        error.wait_error.unwrap().raw_os_error(),
        Some(rustix::io::Errno::CHILD.raw_os_error())
    );
}

#[test]
fn bounded_cleanup_retains_term_kill_and_reap_failures_beside_cancellation() {
    let _fixture = Fixture::new();
    let mut child = OwnedChild::spawn_with_cleanup(
        &mut Command::new("/bin/true"),
        crate::child::CleanupPolicy::TermThenKill {
            grace: Duration::from_secs(30),
            reap_timeout: Duration::ZERO,
        },
    )
    .unwrap();
    let pid = Pid::from_raw(i32::try_from(child.id()).unwrap()).unwrap();
    waitid(WaitId::Pid(pid), WaitIdOptions::EXITED).unwrap();
    assert!(child.poll_exit().is_err());
    let error = finish_capture(
        &mut child,
        ExecutionEvidence {
            stdout: b"prefix".to_vec(),
            ..ExecutionEvidence::default()
        },
        Err(ExecutionFailure::Cancelled),
    )
    .unwrap_err();
    assert!(matches!(error.failure, ExecutionFailure::Cancelled));
    assert_eq!(error.evidence.stdout, b"prefix");
    for failure in [error.term_error, error.group_error, error.wait_error] {
        assert_eq!(
            failure.unwrap().raw_os_error(),
            Some(rustix::io::Errno::CHILD.raw_os_error())
        );
    }
    assert!(error.kill_error.is_none());
}
