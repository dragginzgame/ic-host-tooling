//! Explicit, release-bound PocketIC assets for Rust fixtures and the CLI.
//!
//! [`check`] is offline and never sets up a missing bundle. [`setup`] explicitly
//! provisions it. Callers retain namespace custody and select the bundle root;
//! managed startup and application safety remain separate responsibilities.

use super::POCKET_IC_SERVER_VERSION;
use ic_host_artifacts::artifact::{Sha256Digest, copy_gzip, hash_gzip, hash_reader};
use ic_host_fs::path::validate_physical_directory_allow_missing;
use ic_host_fs::{durable, read::read_file_no_follow};
use ic_host_process::{
    child::CleanupPolicy,
    tool::{
        AdmittedTool, ExecutionContext, OutputLimit, OutputLimits, VersionSpec,
        capture_group_command_with_cancellation,
    },
};
use std::{
    error::Error,
    ffi::OsString,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const ARCHIVE_LIMIT: usize = 80 * 1024 * 1024;
const EXECUTABLE_LIMIT: usize = 512 * 1024 * 1024;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
/// Default bundle root used by `ic-testkit-server`.
pub const DEFAULT_DIRECTORY: &str = ".tools/ic-testkit-server";

struct Asset {
    host: &'static str,
    digest: Sha256Digest,
}

fn selected_asset() -> Result<Asset> {
    // Official GitHub release asset digests, reviewed 2026-10-09.
    let (host, digest) = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => (
            "x86_64-linux",
            "131219d90dcf9bf6f3ed8ee02d8f55504ea24a6db382f5680675b7bd6b7ed3bf",
        ),
        ("macos", "x86_64") => (
            "x86_64-darwin",
            "af9ad2d781530a43556ef2d1c8f93db99a2425c6cabdc78520f61922399ed530",
        ),
        ("macos", "aarch64") => (
            "arm64-darwin",
            "9ae843fbb7ae6c6eb30137671c8629a80ef53b3a3652eb85b344847ac39f9fcd",
        ),
        _ => return Err("no reviewed PocketIC server asset for this host".into()),
    };
    Ok(Asset {
        host,
        digest: digest.parse()?,
    })
}

fn root_path(directory: &Path) -> Result<PathBuf> {
    if directory.as_os_str().is_empty() {
        return Err("server directory must not be empty".into());
    }
    let absolute = if directory.is_absolute() {
        directory.to_owned()
    } else {
        std::env::current_dir()?.join(directory)
    };
    if absolute
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .any(|byte| matches!(byte, b'\n' | b'\r'))
    {
        return Err("server directory and its ancestors must not contain line breaks".into());
    }
    validate_physical_directory_allow_missing(&absolute)?;
    Ok(absolute.components().collect())
}

fn bundle_path(root: &Path, asset: &Asset) -> PathBuf {
    root.join(format!("{}-{}", POCKET_IC_SERVER_VERSION, asset.host))
}

fn authenticated_archive(archive: &Path, digest: Sha256Digest) -> Result<Vec<u8>> {
    let bytes = read_file_no_follow(archive, ARCHIVE_LIMIT)?;
    hash_reader(bytes.as_slice(), ARCHIVE_LIMIT as u64)?.verify(digest)?;
    Ok(bytes)
}

fn admit(executable: &Path, digest: Sha256Digest) -> Result<()> {
    let directory = executable.parent().ok_or("missing executable parent")?;
    AdmittedTool::admit_no_follow(
        &VersionSpec {
            executable,
            executable_bytes: EXECUTABLE_LIMIT as u64,
            version_arguments: &[OsString::from("--version")],
        },
        digest,
        &format!("pocket-ic-server {POCKET_IC_SERVER_VERSION}"),
        &ExecutionContext {
            current_dir: directory,
            environment: &[],
        },
        OutputLimits {
            stdout: OutputLimit::Terminate(4096),
            stderr: OutputLimit::Terminate(4096),
            timeout: Some(Duration::from_secs(10)),
        },
    )?;
    Ok(())
}

fn check_bundle(bundle: &Path, asset: &Asset) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(bundle)?;
    if !metadata.is_dir() || metadata.is_symlink() {
        return Err("PocketIC bundle must be a physical directory".into());
    }
    let bytes = authenticated_archive(&bundle.join("archive.gz"), asset.digest)?;
    let digest = hash_gzip(&bytes, ARCHIVE_LIMIT, EXECUTABLE_LIMIT as u64)?.sha256;
    drop(bytes);
    let executable = bundle.join("pocket-ic");
    admit(&executable, digest)?;
    Ok(executable)
}

/// Admit an existing reviewed bundle without downloading or changing it.
///
/// Authenticates the bounded archive before decoding, compares the executable's
/// exact bytes, then runs bounded `--version` admission with an empty environment.
/// Returns the absolute executable path, not ongoing custody of its namespace.
/// Callers must prevent replacement between admission and managed startup.
///
/// # Errors
/// Rejects unsupported hosts, redirected paths, missing or modified bundles,
/// invalid archives, or failed executable/version admission.
pub fn check(directory: &Path) -> Result<PathBuf> {
    let asset = selected_asset()?;
    check_bundle(&bundle_path(&root_path(directory)?, &asset), &asset)
}

/// Explicitly prepare or admit the reviewed bundle, retaining failed attempts.
///
/// Downloads only when the selected bundle is absent. Cooperating setups share
/// one lock; existing bundles must pass the same offline [`check`] admission.
/// `interrupted` lets the caller request bounded downloader cancellation without
/// this library installing process-global signal handlers.
///
/// # Errors
/// Returns path, lock, download, authentication, decoding, publication or
/// executable/version failures. Retained evidence is never automatically removed.
pub fn setup(directory: &Path, mut interrupted: impl FnMut() -> bool) -> Result<PathBuf> {
    let asset = selected_asset()?;
    setup_with(directory, &asset, |archive| {
        let url = format!(
            "https://github.com/dfinity/pocketic/releases/download/{POCKET_IC_SERVER_VERSION}/pocket-ic-{}.gz",
            asset.host
        );
        let mut command = Command::new("curl");
        command
            .args([
                "--proto",
                "=https",
                "--proto-redir",
                "=https",
                "--tlsv1.2",
                "--fail",
                "--silent",
                "--show-error",
                "--location",
                "--connect-timeout",
                "15",
                "--max-time",
                "300",
                "--max-filesize",
                "83886080",
                "--output",
            ])
            .arg(archive)
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let outcome = capture_group_command_with_cancellation(
            &mut command,
            OutputLimits {
                stdout: OutputLimit::Terminate(4096),
                stderr: OutputLimit::Terminate(64 * 1024),
                timeout: Some(Duration::from_secs(310)),
            },
            CleanupPolicy::KillAndWait,
            None,
            &mut interrupted,
        )
        .and_then(ic_host_process::tool::CompletedExecution::require_success);
        if let Some(stderr) = outcome
            .as_ref()
            .ok()
            .map(|completed| &completed.stderr)
            .or_else(|| {
                outcome
                    .as_ref()
                    .err()
                    .and_then(|error| error.evidence())
                    .map(|evidence| &evidence.stderr)
            })
            && let Err(record_error) = durable::write_bytes(
                &archive
                    .parent()
                    .ok_or("missing archive parent")?
                    .join("download.stderr"),
                durable::WriteOptions {
                    mode: durable::PublicationMode::Replace,
                    permissions: 0o666,
                },
                stderr,
            )
        {
            if outcome.is_ok() {
                return Err(record_error.into());
            }
            // Recording evidence must not replace the failed download's cause.
            eprintln!("could not record download stderr: {record_error}");
        }
        outcome?;
        Ok(())
    })
}

fn setup_with(
    directory: &Path,
    asset: &Asset,
    download: impl FnOnce(&Path) -> Result<()>,
) -> Result<PathBuf> {
    let root = root_path(directory)?;
    // Namespace custody remains with the caller. All cooperating setups serialize
    // through Host's admitted regular lock; checks are read-only.
    let lock = durable::open_regular_lock_file_with_parents(&root.join(".setup-v1.lock"))?;
    durable::lock_exclusive(&lock)?;
    let bundle = bundle_path(&root, asset);
    let attempt = root.join(format!(
        ".setup-v1-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let staging_directory = match fs::symlink_metadata(&bundle) {
        Ok(_) => {
            check_bundle(&bundle, asset)?;
            None
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let held = durable::create_private_directory(&attempt)?;
            eprintln!("PocketIC setup evidence: {}", attempt.display());
            if let Err(error) = install(&attempt, asset, download) {
                if let Err(record_error) = durable::write_bytes(
                    &attempt.join("failure.txt"),
                    durable::WriteOptions {
                        mode: durable::PublicationMode::Replace,
                        permissions: 0o666,
                    },
                    error.to_string().as_bytes(),
                ) {
                    eprintln!("could not record setup failure: {record_error}");
                }
                return Err(error);
            }
            Some(held)
        }
        Err(error) => return Err(error.into()),
    };
    // Host owns no-replace publication and phase-aware durability failures.
    // Fresh staging must retain its original directory identity. Recovery admits
    // the installed bundle again before Host synchronizes its parent.
    durable::commit_existing_directory_with(
        &attempt,
        &bundle,
        |selected, path| -> io::Result<()> {
            if let Some(held) = &staging_directory {
                durable::admit_directory(path, held)?;
            }
            selected.sync_all()?;
            Ok(())
        },
        |_, _| Ok(()),
    )?;
    eprintln!("PocketIC setup evidence retained: {}", bundle.display());
    Ok(bundle.join("pocket-ic"))
}

fn install(
    attempt: &Path,
    asset: &Asset,
    download: impl FnOnce(&Path) -> Result<()>,
) -> Result<()> {
    let archive = attempt.join("archive.gz");
    download(&archive)?;
    let bytes = authenticated_archive(&archive, asset.digest)?;
    File::open(&archive)?.sync_all()?;
    let executable = attempt.join("pocket-ic");
    let identity = durable::write_with(
        &executable,
        durable::WriteOptions {
            mode: durable::PublicationMode::CreateNew,
            permissions: 0o755,
        },
        |file| copy_gzip(&bytes, file, ARCHIVE_LIMIT, EXECUTABLE_LIMIT as u64),
    )?;
    drop(bytes);
    // PocketIC requires its canonical basename. The whole directory is private
    // and unselected until this admission succeeds; no existing bundle is replaced.
    if let Err(error) = admit(&executable, identity.sha256) {
        if let Some(evidence) = error
            .downcast_ref::<ic_host_process::tool::ToolError>()
            .and_then(|error| error.evidence())
        {
            for (name, bytes) in [
                ("version.stdout", &evidence.stdout),
                ("version.stderr", &evidence.stderr),
            ] {
                if let Err(record_error) = durable::write_bytes(
                    &attempt.join(name),
                    durable::WriteOptions {
                        mode: durable::PublicationMode::Replace,
                        permissions: 0o666,
                    },
                    bytes,
                ) {
                    eprintln!("could not record version evidence: {record_error}");
                }
            }
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_host_artifacts::artifact::encode_gzip;
    use ic_host_process::{
        child::OwnedChild,
        tool::{SuccessfulExit, communicate_child},
    };
    use std::os::unix::fs::symlink;

    fn run_in_subprocess() -> bool {
        let current = std::thread::current();
        let name = current.name().expect("registered asset test name");
        if std::env::var("IC_TESTKIT_ASSET_CASE").as_deref() == Ok(name) {
            return false;
        }
        // A library sibling's fork can retain a writable executable descriptor.
        // Re-exec before writing fixtures so only this case owns their lifetime.
        let mut child = OwnedChild::spawn(
            Command::new(std::env::current_exe().expect("library test executable"))
                .args(["--exact", name, "--nocapture"])
                .env("IC_TESTKIT_ASSET_CASE", name)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped()),
            CleanupPolicy::KillAndWait,
        )
        .expect("spawn isolated asset test");
        let completed = communicate_child(
            &mut child,
            None,
            OutputLimits {
                stdout: OutputLimit::Terminate(64 * 1024),
                stderr: OutputLimit::Terminate(64 * 1024),
                timeout: Some(Duration::from_secs(120)),
            },
            SuccessfulExit::Cleanup,
            &mut || false,
        )
        .expect("capture and reap isolated asset test");
        assert!(
            completed.status.success(),
            "isolated asset test failed: stdout={} stderr={}",
            String::from_utf8_lossy(&completed.stdout),
            String::from_utf8_lossy(&completed.stderr),
        );
        true
    }

    struct Fixture {
        root: ic_host_fs::testing::FixtureDirectory,
    }
    impl Fixture {
        fn new() -> Self {
            let root = ic_host_fs::testing::FixtureDirectory::new(
                &std::env::temp_dir(),
                "testkit-provision",
            )
            .unwrap();
            Self { root }
        }
    }
    fn archive(version: &str) -> (Asset, Vec<u8>) {
        let script = format!("#!/bin/sh\nprintf 'pocket-ic-server {version}\\n'\n");
        let mut bytes = Vec::new();
        encode_gzip(script.as_bytes(), &mut bytes, 1, 65536).unwrap();
        (
            Asset {
                host: "fixture",
                digest: Sha256Digest::compute(&bytes),
            },
            bytes,
        )
    }

    #[test]
    fn authenticated_setup_reuses_offline_and_refuses_changed_bytes_before_execution() {
        if run_in_subprocess() {
            return;
        }
        let root = Fixture::new();
        let (asset, bytes) = archive(POCKET_IC_SERVER_VERSION);
        let executable = setup_with(root.root.path(), &asset, |path| {
            fs::write(path, &bytes)?;
            Ok(())
        })
        .unwrap();
        let original = fs::read(&executable).unwrap();
        assert_eq!(
            setup_with(root.root.path(), &asset, |_| panic!(
                "verified reuse must not download"
            ))
            .unwrap(),
            executable
        );
        let entries = fs::read_dir(root.root.path()).unwrap().count();
        assert_eq!(
            check_bundle(&bundle_path(root.root.path(), &asset), &asset).unwrap(),
            executable
        );
        assert_eq!(fs::read_dir(root.root.path()).unwrap().count(), entries);
        let marker = root.root.path().join("executed");
        fs::write(
            &executable,
            format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
        )
        .unwrap();
        assert!(check_bundle(&bundle_path(root.root.path(), &asset), &asset).is_err());
        assert!(!marker.exists());
        fs::write(&executable, original).unwrap();
        assert!(check_bundle(&bundle_path(root.root.path(), &asset), &asset).is_ok());
        let saved = root.root.path().join("saved");
        fs::rename(&executable, &saved).unwrap();
        symlink(&saved, &executable).unwrap();
        assert!(check_bundle(&bundle_path(root.root.path(), &asset), &asset).is_err());
    }

    #[test]
    fn offline_check_authenticates_before_decoding_and_rejects_invalid_gzip() {
        use ic_host_artifacts::artifact::{ArtifactError, GzipError};

        if run_in_subprocess() {
            return;
        }

        let root = Fixture::new();
        let (mut asset, original) = archive(POCKET_IC_SERVER_VERSION);
        let executable = setup_with(root.root.path(), &asset, |path| {
            fs::write(path, &original)?;
            Ok(())
        })
        .unwrap();
        let installed = fs::read(&executable).unwrap();
        let bundle = bundle_path(root.root.path(), &asset);
        let path = bundle.join("archive.gz");

        // Both authentication and decoding would fail; authentication wins.
        fs::write(&path, b"not gzip").unwrap();
        let error = check_bundle(&bundle, &asset).unwrap_err();
        assert!(matches!(
            error.downcast_ref::<ArtifactError>(),
            Some(ArtifactError::DigestMismatch { .. })
        ));

        let mut bad_crc = original.clone();
        let crc = bad_crc.len() - 8;
        bad_crc[crc] ^= 1;
        let mut bad_length = original.clone();
        let length = bad_length.len() - 4;
        bad_length[length] ^= 1;
        for bytes in [
            b"not gzip".to_vec(),
            original[..original.len() - 1].to_vec(),
            bad_crc,
            bad_length,
            [original.as_slice(), b"trailing"].concat(),
            [original.as_slice(), original.as_slice()].concat(),
        ] {
            fs::write(&path, &bytes).unwrap();
            asset.digest = Sha256Digest::compute(&bytes);
            let error = check_bundle(&bundle, &asset).unwrap_err();
            // Gzip refusal occurs before executable admission/version execution.
            assert!(error.downcast_ref::<GzipError>().is_some());
            assert_eq!(fs::read(&executable).unwrap(), installed);
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
        fs::write(&path, &original).unwrap();
        asset.digest = Sha256Digest::compute(&original);
        assert_eq!(check_bundle(&bundle, &asset).unwrap(), executable);
    }

    #[test]
    fn setup_rejects_corrupt_payloads_without_execution_or_publication() {
        use ic_host_artifacts::artifact::GzipError;

        if run_in_subprocess() {
            return;
        }
        let root = Fixture::new();
        let marker = root.root.path().join("executed");
        let script = format!("#!/bin/sh\nprintf executed > '{}'\n", marker.display());
        let mut original = Vec::new();
        encode_gzip(script.as_bytes(), &mut original, 1, 65536).unwrap();
        let mut bad_crc = original.clone();
        let crc = bad_crc.len() - 8;
        bad_crc[crc] ^= 1;
        for bytes in [
            bad_crc,
            original[..original.len() - 1].to_vec(),
            [original.as_slice(), b"trailing"].concat(),
        ] {
            let asset = Asset {
                host: "fixture",
                digest: Sha256Digest::compute(&bytes),
            };
            let error = setup_with(root.root.path(), &asset, |path| {
                fs::write(path, &bytes)?;
                Ok(())
            })
            .unwrap_err();
            assert!(matches!(
                error.downcast_ref::<durable::NamedWriteError<GzipError>>(),
                Some(durable::NamedWriteError::Producer { .. })
            ));
            assert!(!marker.exists());
            assert!(!bundle_path(root.root.path(), &asset).exists());
            assert!(fs::read_dir(root.root.path()).unwrap().any(|entry| {
                let path = entry.unwrap().path();
                path.join("failure.txt").is_file()
                    && fs::read(path.join("archive.gz")).ok().as_deref() == Some(bytes.as_slice())
            }));
        }
    }

    #[test]
    fn failed_setup_retains_evidence_and_previous_bundles_then_can_retry() {
        if run_in_subprocess() {
            return;
        }
        let root = Fixture::new();
        let previous = root.root.path().join("previous-version");
        fs::create_dir(&previous).unwrap();
        fs::write(previous.join("pocket-ic"), b"previous bytes").unwrap();
        let (asset, bytes) = archive(POCKET_IC_SERVER_VERSION);
        assert!(
            setup_with(root.root.path(), &asset, |path| {
                fs::write(path, b"partial download")?;
                Err("interrupted download".into())
            })
            .is_err()
        );
        assert!(!bundle_path(root.root.path(), &asset).exists());
        let failed = fs::read_dir(root.root.path())
            .unwrap()
            .find_map(|entry| {
                let path = entry.unwrap().path();
                path.join("failure.txt").is_file().then_some(path)
            })
            .unwrap();
        assert_eq!(
            fs::read(failed.join("archive.gz")).unwrap(),
            b"partial download"
        );
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = setup_with(root.root.path(), &asset, |path| {
                    fs::write(path, b"interrupted candidate")?;
                    panic!("injected interruption before admission");
                });
            }))
            .is_err()
        );
        assert!(!bundle_path(root.root.path(), &asset).exists());
        assert!(fs::read_dir(root.root.path()).unwrap().any(|entry| {
            fs::read(entry.unwrap().path().join("archive.gz"))
                .ok()
                .as_deref()
                == Some(b"interrupted candidate")
        }));
        assert!(
            setup_with(root.root.path(), &asset, |path| {
                fs::write(path, &bytes)?;
                Ok(())
            })
            .is_ok()
        );
        assert!(failed.join("failure.txt").is_file());
        assert_eq!(
            fs::read(previous.join("pocket-ic")).unwrap(),
            b"previous bytes"
        );
    }

    #[test]
    fn digest_decode_and_version_failures_never_publish() {
        if run_in_subprocess() {
            return;
        }
        for mode in [0, 1, 2] {
            let root = Fixture::new();
            let (mut asset, mut bytes) = archive("15.0.0");
            if mode == 0 {
                asset.digest = Sha256Digest::compute(b"wrong");
            }
            if mode == 1 {
                bytes = b"not gzip".to_vec();
                asset.digest = Sha256Digest::compute(&bytes);
            }
            assert!(
                setup_with(root.root.path(), &asset, |path| {
                    fs::write(path, &bytes)?;
                    Ok(())
                })
                .is_err()
            );
            assert!(!bundle_path(root.root.path(), &asset).exists());
        }
    }

    #[test]
    fn redirected_paths_are_refused_without_touching_the_target() {
        if run_in_subprocess() {
            return;
        }
        let root = Fixture::new();
        let (asset, _) = archive(POCKET_IC_SERVER_VERSION);
        let target = root.root.path().join("target");
        fs::create_dir(&target).unwrap();
        let redirected = root.root.path().join("redirected");
        symlink(&target, &redirected).unwrap();
        assert!(
            setup_with(&redirected.join("child"), &asset, |_| panic!(
                "redirected ancestor reached downloader"
            ))
            .is_err()
        );
        assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
        symlink(&target, root.root.path().join(".setup-v1.lock")).unwrap();
        assert!(
            setup_with(root.root.path(), &asset, |_| panic!(
                "redirected lock reached downloader"
            ))
            .is_err()
        );
    }

    #[test]
    fn replaced_staging_directory_is_not_published() {
        if run_in_subprocess() {
            return;
        }
        let root = Fixture::new();
        let (asset, bytes) = archive(POCKET_IC_SERVER_VERSION);
        let retained = root.root.path().join("retained-original-stage");
        let error = setup_with(root.root.path(), &asset, |archive| {
            fs::write(archive, &bytes)?;
            let stage = archive.parent().unwrap();
            fs::rename(stage, &retained)?;
            fs::create_dir(stage)?;
            fs::write(archive, &bytes)?;
            Ok(())
        })
        .expect_err("replacement must not inherit the held directory claim");
        assert!(matches!(
            error.downcast_ref::<durable::DirectoryCommitError<io::Error>>(),
            Some(durable::DirectoryCommitError::Prepare(source))
                if source.kind() == io::ErrorKind::InvalidInput
        ));
        assert!(!bundle_path(root.root.path(), &asset).exists());
        assert_eq!(fs::read(retained.join("archive.gz")).unwrap(), bytes);
    }

    #[test]
    fn occupied_destination_preserves_both_bundle_and_verified_staging() {
        if run_in_subprocess() {
            return;
        }
        let root = Fixture::new();
        let (asset, bytes) = archive(POCKET_IC_SERVER_VERSION);
        let bundle = bundle_path(root.root.path(), &asset);
        let mut attempt = None;
        let error = setup_with(root.root.path(), &asset, |archive| {
            fs::write(archive, &bytes)?;
            attempt = archive.parent().map(Path::to_owned);
            fs::create_dir(&bundle)?;
            fs::write(bundle.join("unrelated"), b"retained destination")?;
            Ok(())
        })
        .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<durable::DirectoryCommitError<io::Error>>(),
            Some(durable::DirectoryCommitError::Conflict)
        ));
        assert_eq!(
            fs::read(bundle.join("unrelated")).unwrap(),
            b"retained destination"
        );
        let attempt = attempt.unwrap();
        assert_eq!(fs::read(attempt.join("archive.gz")).unwrap(), bytes);
        assert!(attempt.join("pocket-ic").is_file());
    }

    #[test]
    fn concurrent_setups_download_once_and_recheck_under_the_lock() {
        if run_in_subprocess() {
            return;
        }
        let root = Fixture::new();
        let (asset, bytes) = archive(POCKET_IC_SERVER_VERSION);
        let calls = AtomicU64::new(0);
        std::thread::scope(|scope| {
            let handles = (0..2)
                .map(|_| {
                    scope.spawn(|| {
                        setup_with(root.root.path(), &asset, |path| {
                            calls.fetch_add(1, Ordering::Relaxed);
                            fs::write(path, &bytes)?;
                            Ok(())
                        })
                        .unwrap()
                    })
                })
                .collect::<Vec<_>>();
            for handle in handles {
                assert_eq!(
                    handle.join().unwrap(),
                    bundle_path(root.root.path(), &asset).join("pocket-ic")
                );
            }
        });
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
}
