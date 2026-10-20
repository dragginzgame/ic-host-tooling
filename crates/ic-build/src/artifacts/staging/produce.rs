use std::{fs, io, path::Path, process::ExitStatus};

use super::CapturedArtifact;

/// Produce and retain one artifact, removing its producer only after exact capture.
///
/// The caller owns physically distinct paths, their parents and custody against
/// other writers, including any parent aliases. Identical paths refuse early.
/// Both producer and log are created exclusively, in that order, before `run`.
/// The callback receives the producer path and owned log file; it owns the tool,
/// arguments, process execution, output routing and cancellation policy. This
/// function starts no process itself and creates no cache or default deadline.
///
/// An unsuccessful exit is returned unchanged, without capture or cleanup. I/O
/// errors and panics also leave existing producer/log files for caller recovery.
/// On success, capture admits `maximum_bytes`, retains the exact bytes at
/// `destination`, then removes only the producer. The log always remains.
///
/// # Errors
/// Returns creation, callback, capture, retention or producer-removal errors.
/// Capture/retention errors retain their existing typed causes, including
/// [`super::CaptureRetentionError`] when capture custody has transferred.
pub fn produce_and_retain_artifact(
    producer: &Path,
    log: &Path,
    destination: &Path,
    maximum_bytes: usize,
    run: impl FnOnce(&Path, fs::File) -> io::Result<ExitStatus>,
) -> io::Result<ExitStatus> {
    if producer == log || producer == destination || log == destination {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "producer, log and destination must be distinct",
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| io::Error::other("missing evidence parent"))?;
    fs::File::create_new(producer)?;
    let log = fs::File::create_new(log)?;
    let status = run(producer, log)?;
    if status.success() {
        CapturedArtifact::capture(producer, parent, maximum_bytes)?
            .retain_to(destination)
            .map_err(io::Error::other)?;
        fs::remove_file(producer)?;
    }
    Ok(status)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::fs_fixture::unique_temp_directory;
    use ic_host_artifacts::artifact::ArtifactError;
    use std::{io::Write, os::unix::process::ExitStatusExt};

    #[test]
    fn successful_production_retains_exact_output_and_log_before_removing_producer() {
        let root = unique_temp_directory("produced-artifact-success");
        let producer = root.join("producer");
        let log = root.join("log");
        let destination = root.join("retained");
        let status =
            produce_and_retain_artifact(&producer, &log, &destination, 4, |path, mut log| {
                fs::write(path, b"four")?;
                log.write_all(b"tool diagnostics")?;
                Ok(ExitStatus::from_raw(0))
            })
            .unwrap();
        assert!(status.success());
        assert_eq!(fs::read(&destination).unwrap(), b"four");
        assert_eq!(fs::read(&log).unwrap(), b"tool diagnostics");
        assert!(!producer.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unsuccessful_exit_and_callback_error_preserve_partial_output_and_diagnostics() {
        for callback_error in [false, true] {
            let root = unique_temp_directory("produced-artifact-failure");
            let producer = root.join("producer");
            let log = root.join("log");
            let destination = root.join("retained");
            let status = ExitStatus::from_raw(37 << 8);
            let result =
                produce_and_retain_artifact(&producer, &log, &destination, 32, |path, mut log| {
                    fs::write(path, b"partial")?;
                    log.write_all(b"failed")?;
                    if callback_error {
                        Err(io::Error::other("callback failure"))
                    } else {
                        Ok(status)
                    }
                });
            if callback_error {
                assert_eq!(result.unwrap_err().to_string(), "callback failure");
            } else {
                assert_eq!(result.unwrap(), status);
            }
            assert_eq!(fs::read(&producer).unwrap(), b"partial");
            assert_eq!(fs::read(&log).unwrap(), b"failed");
            assert!(!destination.exists());
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn capture_and_retention_refusals_preserve_producer_and_existing_destination() {
        for refusal in ["limit", "retention"] {
            let root = unique_temp_directory("produced-artifact-capture-refusal");
            let producer = root.join("producer");
            let log = root.join("log");
            let destination = root.join("retained");
            if refusal == "retention" {
                fs::create_dir(&destination).unwrap();
            }
            let maximum = if refusal == "limit" { 3 } else { 4 };
            let error =
                produce_and_retain_artifact(&producer, &log, &destination, maximum, |path, _| {
                    fs::write(path, b"four")?;
                    Ok(ExitStatus::from_raw(0))
                })
                .unwrap_err();
            assert_eq!(fs::read(&producer).unwrap(), b"four");
            assert!(log.is_file());
            if refusal == "limit" {
                assert!(matches!(
                    error.get_ref().and_then(|source| source.downcast_ref()),
                    Some(ArtifactError::LimitExceeded { limit: 3 })
                ));
                assert!(!destination.exists());
            } else {
                let retained = error
                    .get_ref()
                    .unwrap()
                    .downcast_ref::<super::super::CaptureRetentionError>()
                    .unwrap();
                assert_eq!(fs::read(&retained.retained_path).unwrap(), b"four");
                assert_eq!(fs::read_dir(&destination).unwrap().count(), 0);
            }
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn collisions_refuse_before_callback_and_preserve_existing_evidence() {
        for occupied in ["producer", "log"] {
            let root = unique_temp_directory("produced-artifact-collision");
            let producer = root.join("producer");
            let log = root.join("log");
            let destination = root.join("retained");
            fs::write(root.join(occupied), b"existing").unwrap();
            let error = produce_and_retain_artifact(&producer, &log, &destination, 32, |_, _| {
                panic!("must refuse before dispatch")
            })
            .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
            assert_eq!(fs::read(root.join(occupied)).unwrap(), b"existing");
            assert!(!destination.exists());
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn callback_panic_leaves_partial_output_and_log_for_explicit_recovery() {
        let root = unique_temp_directory("produced-artifact-panic");
        let producer = root.join("producer");
        let log = root.join("log");
        let destination = root.join("retained");
        let result = std::panic::catch_unwind(|| {
            produce_and_retain_artifact(&producer, &log, &destination, 32, |path, mut log| {
                fs::write(path, b"partial").unwrap();
                log.write_all(b"panic diagnostics").unwrap();
                panic!("synthetic transform panic");
            })
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&producer).unwrap(), b"partial");
        assert_eq!(fs::read(&log).unwrap(), b"panic diagnostics");
        assert!(!destination.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn overlapping_paths_refuse_before_creation_or_callback() {
        let root = unique_temp_directory("produced-artifact-overlap");
        let producer = root.join("producer");
        let log = root.join("log");
        let destination = root.join("retained");
        for (producer, log, destination) in [
            (&producer, &producer, &destination),
            (&producer, &log, &producer),
            (&producer, &destination, &destination),
        ] {
            let error = produce_and_retain_artifact(producer, log, destination, 32, |_, _| {
                panic!("overlapping evidence must refuse before dispatch")
            })
            .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        }
        fs::remove_dir_all(root).unwrap();
    }
}
