use super::*;
use std::io::{BufRead as _, BufReader};

#[test]
fn buffered_final_frame_and_cached_nonzero_status_are_retained() {
    let mut child = OwnedChild::spawn(&mut command(
        "printf 'control\\nfinal\\n'; printf diagnostic >&2; exit 7",
    ))
    .unwrap();
    let mut reader = BufReader::new(child.take_stdout().unwrap());
    child.wait().unwrap();
    assert_eq!(reader.fill_buf().unwrap(), b"control\nfinal\n");
    let mut frame = String::new();
    reader.read_line(&mut frame).unwrap();
    assert_eq!(frame, "control\n");
    let output = drain_child_output(&mut child, Some(reader), LIMITS, || false).unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"final\n");
    assert_eq!(output.stderr, b"diagnostic");
    assert_reaped(&child);
}

#[test]
fn prefetched_bytes_share_hard_and_truncating_limits() {
    for stdout in [OutputLimit::Terminate(3), OutputLimit::Truncate(3)] {
        let mut child = OwnedChild::spawn(&mut command("printf abcdef")).unwrap();
        let mut reader = BufReader::new(child.take_stdout().unwrap());
        child.wait().unwrap();
        assert_eq!(reader.fill_buf().unwrap(), b"abcdef");
        let result = drain_child_output(
            &mut child,
            Some(reader),
            OutputLimits { stdout, ..LIMITS },
            || false,
        );
        if matches!(stdout, OutputLimit::Terminate(_)) {
            assert!(
                matches!(result, Err(ToolError::Execution(error)) if matches!(error.failure, ExecutionFailure::OutputLimit { .. }) && error.evidence.stdout == b"abc")
            );
        } else {
            let output = result.unwrap();
            assert_eq!(output.stdout, b"abc");
            assert!(output.stdout_truncated);
        }
        assert_reaped(&child);
    }
}

#[test]
fn retained_reader_drains_both_full_pipes_and_cancellation_cleans_child() {
    let mut child = OwnedChild::spawn(&mut command("n=0; while [ $n -lt 10000 ]; do printf abcdefghijklmnop; printf ponmlkjihgfedcba >&2; n=$((n+1)); done")).unwrap();
    let reader = BufReader::new(child.take_stdout().unwrap());
    let output = drain_child_output(
        &mut child,
        Some(reader),
        OutputLimits {
            stdout: OutputLimit::Terminate(160_000),
            stderr: OutputLimit::Terminate(160_000),
            ..LIMITS
        },
        || false,
    )
    .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout.len(), 160_000);
    assert_eq!(output.stderr.len(), 160_000);
    let mut child = OwnedChild::spawn(&mut command("sleep 30")).unwrap();
    let reader = BufReader::new(child.take_stdout().unwrap());
    assert!(
        matches!(drain_child_output(&mut child, Some(reader), LIMITS, || true), Err(ToolError::Execution(error)) if matches!(error.failure, ExecutionFailure::Cancelled))
    );
    assert_reaped(&child);
}
