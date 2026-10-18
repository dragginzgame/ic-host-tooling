use super::*;
use crate::fs_fixture::unique_temp_directory;
use ic_host_process::tool::{ExecutionFailure, OutputLimit};
use std::{
    cell::Cell,
    fs,
    process::{Command, Stdio},
};

fn artifact(manifest: &Path, paths: &[PathBuf]) -> schema::Message {
    serde_json::from_value(serde_json::json!({
        "reason":"compiler-artifact", "package_id":"path+file:///fixture#0.1.0",
        "manifest_path":manifest,
        "target":{"name":"custom_library","kind":["cdylib"],"crate_types":["cdylib"],"src_path":"/fixture/src/lib.rs","edition":"2024","doc":true,"doctest":false,"test":true},
        "profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},
        "features":[],"filenames":paths,"executable":null,"fresh":false
    })).unwrap()
}

#[test]
fn exact_manifest_selection_refuses_stale_foreign_and_ambiguous_outputs() {
    let root = unique_temp_directory("cargo-execution-selection");
    let manifest = root.join("Cargo.toml");
    let foreign = root.join("foreign.toml");
    fs::write(&manifest, "").unwrap();
    fs::write(&foreign, "").unwrap();
    let wasm = root.join("custom_library.wasm");
    fs::write(&wasm, b"stale").unwrap();
    for messages in [
        vec![],
        vec![artifact(&foreign, std::slice::from_ref(&wasm))],
    ] {
        assert!(matches!(
            select_cdylib_wasm(&messages, &manifest),
            Err(CargoArtifactError::Selection { count: 0, .. })
        ));
    }
    let selected = artifact(&manifest, std::slice::from_ref(&wasm));
    assert_eq!(
        select_cdylib_wasm(&[selected.clone(), selected], &manifest).unwrap(),
        wasm
    );
    let ambiguous = artifact(&manifest, &[wasm, root.join("second.wasm")]);
    assert!(matches!(
        select_cdylib_wasm(&[ambiguous], &manifest),
        Err(CargoArtifactError::Selection { count: 2, .. })
    ));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn protocol_and_message_sink_refusals_drain_before_returning() {
    for bad_json in [false, true] {
        let mut command = Command::new("/bin/sh");
        command.args([
            "-c",
            if bad_json {
                "printf '{broken}\\n'; printf drained >&2"
            } else {
                "printf '{\"reason\":\"build-finished\",\"success\":true}\\n'; printf drained >&2"
            },
        ]);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = OwnedChild::spawn_direct(&mut command).unwrap();
        let mut output = Vec::new();
        let observed = communicate_cargo(
            &mut child,
            OutputLimits {
                stdout: OutputLimit::Truncate(8),
                stderr: OutputLimit::Terminate(64),
                timeout: None,
            },
            1024,
            || false,
            |stream, bytes| {
                output.push((stream, bytes.to_vec()));
                Ok(())
            },
            |_| {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "original message sink",
                ))
            },
        );
        let evidence = observed.execution.unwrap();
        assert!(evidence.status.success());
        assert!(evidence.stdout_truncated);
        assert_eq!(evidence.stderr, b"drained");
        assert!(
            output
                .iter()
                .any(|(stream, bytes)| *stream == OutputStream::Stderr && bytes == b"drained")
        );
        let error = observed.messages.unwrap_err();
        if bad_json {
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        } else {
            assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
            assert_eq!(error.to_string(), "original message sink");
        }
    }
}

#[cfg(unix)]
#[test]
fn build_script_messages_and_raw_streams_survive_cancellation_and_failed_exit() {
    for cancel in [false, true] {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", if cancel {
            "printf '{\"reason\":\"build-script-executed\",\"package_id\":\"fixture\",\"linked_libs\":[],\"linked_paths\":[],\"cfgs\":[],\"env\":[[\"CONSUMER_INPUT\",\"selected\"]],\"out_dir\":\"/fixture/out\"}\\n'; exec sleep 30"
        } else { "printf '{broken}\\n'; printf original-diagnostic >&2; exit 23" }]);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = OwnedChild::spawn_direct(&mut command).unwrap();
        let seen = Cell::new(false);
        let observed = communicate_cargo(
            &mut child,
            OutputLimits {
                stdout: OutputLimit::Truncate(4096),
                stderr: OutputLimit::Truncate(4096),
                timeout: Some(std::time::Duration::from_secs(5)),
            },
            4096,
            || cancel && seen.get(),
            |_, _| Ok(()),
            |message| {
                if let schema::Message::BuildScriptExecuted(script) = message {
                    assert_eq!(script.env, [("CONSUMER_INPUT".into(), "selected".into())]);
                    assert_eq!(script.out_dir, "/fixture/out");
                    seen.set(true);
                }
                Ok(())
            },
        );
        if cancel {
            assert!(seen.get());
            assert!(matches!(
                observed
                    .execution
                    .unwrap_err()
                    .execution_error()
                    .unwrap()
                    .failure,
                ExecutionFailure::Cancelled
            ));
        } else {
            let evidence = observed.execution.unwrap();
            assert_eq!(evidence.status.code(), Some(23));
            assert_eq!(evidence.stderr, b"original-diagnostic");
            assert!(observed.messages.is_err());
        }
    }
}
