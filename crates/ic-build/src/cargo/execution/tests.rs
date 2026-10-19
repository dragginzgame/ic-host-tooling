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

#[cfg(unix)]
fn invocation_fixture() -> (PathBuf, CargoInvocation) {
    use std::os::unix::fs::PermissionsExt;
    let root = unique_temp_directory("observed-cargo-invocation");
    let tool = root.join("tool");
    fs::write(&tool, "#!/bin/sh\nset -eu\ncase \"$1\" in --version|-vV) printf '%s:%s:' \"$1\" \"$SELECTED\"; /bin/cat identity; printf probe-stderr >&2;; *) printf '%s\\n' \"$@\" > arguments; . ./build-script;; esac\n").unwrap();
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(root.join("identity"), b"original").unwrap();
    fs::write(root.join("build-script"), "test -z \"${HOME+x}\"\nprintf '{\"reason\":\"build-finished\",\"success\":true}\\n'\nprintf drained >&2\n").unwrap();
    let invocation = CargoInvocation {
        cargo_program: tool.clone().into_os_string(),
        working_directory: root.clone(),
        environment: [
            ("SELECTED".into(), "selected".into()),
            ("RUSTC".into(), tool.into_os_string()),
        ]
        .into(),
        target: "wasm32-unknown-unknown".into(),
        selection: crate::cargo::selection::CargoSelection::default(),
        packages: vec!["first".into(), "second".into()],
        manifest_path: Some(root.join("Cargo.toml")),
        recipe: CargoRecipe::Library,
        configurations: vec!["profile.release.lto=false".into()],
        render_diagnostics: true,
        keep_going: true,
    };
    (root, invocation)
}

#[cfg(unix)]
fn execution_policy() -> CargoExecutionPolicy {
    CargoExecutionPolicy {
        output: OutputLimits {
            stdout: OutputLimit::Truncate(8),
            stderr: OutputLimit::Truncate(128),
            timeout: None,
        },
        message_bytes: 1024,
        probe_cleanup: ic_host_process::child::CleanupPolicy::KillAndWait,
    }
}

#[test]
#[cfg(unix)]
fn rendered_selection_and_exact_tool_observations_belong_to_one_execution() {
    use crate::cargo::selection::CargoFeatureSelection;
    let (root, mut invocation) = invocation_fixture();
    invocation.selection = crate::cargo::selection::CargoSelection {
        profile: Some("release".into()),
        features: CargoFeatureSelection {
            features: ["feature-b".into(), "feature-a".into()].into(),
            default_features: false,
            all_features: false,
        },
        locked: true,
        offline: true,
    };
    invocation.recipe = CargoRecipe::RustcLibrary(vec!["--cfg".into(), "export_candid".into()]);
    let mut messages = Vec::new();
    let observed = execute_cargo(
        &invocation,
        execution_policy(),
        OwnedChild::spawn_direct,
        || false,
        |_, _| Ok(()),
        |message| {
            messages.push(message);
            Ok(())
        },
    )
    .unwrap();
    let execution = observed.execution.unwrap();
    assert!(execution.status.success());
    assert!(execution.stdout_truncated);
    assert_eq!(execution.stderr, b"drained");
    observed.messages.unwrap();
    assert_eq!(messages.len(), 1);
    let facts = observed.facts.unwrap().unwrap();
    assert_eq!(facts.environment, invocation.environment);
    assert_eq!(facts.selection, invocation.selection);
    assert_eq!(facts.target, invocation.target);
    assert_eq!(
        facts.cargo_identity,
        b"--version:selected:originalprobe-stderr"
    );
    assert_eq!(facts.rustc_identity, b"-vV:selected:originalprobe-stderr");
    let args = fs::read_to_string(root.join("arguments")).unwrap();
    assert!(args.starts_with("rustc\n--message-format=json-render-diagnostics\n--lib\n"));
    for argument in [
        "--target\nwasm32-unknown-unknown",
        "--release",
        "--locked",
        "--offline",
        "--no-default-features",
        "--features=feature-a,feature-b",
        "-p\nfirst\n-p\nsecond",
        "--config\nprofile.release.lto=false",
        "--\n--cfg\nexport_candid",
    ] {
        assert!(args.contains(argument), "{args}");
    }
    invocation.recipe = CargoRecipe::Example("probe".into());
    let args: Vec<_> = invocation
        .command()
        .unwrap()
        .get_args()
        .map(std::borrow::ToOwned::to_owned)
        .collect();
    assert!(args.windows(2).any(|pair| pair == ["--example", "probe"]));
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn tool_and_spawner_mutations_refuse_facts_after_draining() {
    for mutation in ["tool", "environment", "arguments"] {
        let (root, invocation) = invocation_fixture();
        if mutation == "tool" {
            fs::write(
                root.join("build-script"),
                "printf changed > identity\nprintf drained >&2\n",
            )
            .unwrap();
        }
        let observed = execute_cargo(
            &invocation,
            execution_policy(),
            |command| {
                match mutation {
                    "environment" => {
                        command.env("REINTRODUCED", "value");
                    }
                    "arguments" => {
                        command.arg("--all-features");
                    }
                    _ => {}
                }
                OwnedChild::spawn_direct(command)
            },
            || false,
            |_, _| Ok(()),
            |_| Ok(()),
        )
        .unwrap();
        assert!(observed.execution.unwrap().status.success());
        observed.messages.unwrap();
        let refusal = observed.facts.unwrap_err();
        assert_eq!(refusal.kind(), io::ErrorKind::InvalidData);
        assert!(refusal.to_string().contains(if mutation == "tool" {
            "identity changed"
        } else {
            "spawner changed"
        }));
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
#[cfg(unix)]
fn unsuccessful_execution_never_earns_facts_and_retains_original_failures() {
    for (script, code, bad_message) in [
        ("printf '{broken}\\n'; printf drained >&2", 0, true),
        (
            "printf '{broken}\\n'; printf drained >&2; exit 23",
            23,
            true,
        ),
        (
            "printf '{\"reason\":\"build-finished\",\"success\":true}\\n'; printf drained >&2",
            0,
            false,
        ),
    ] {
        let (root, invocation) = invocation_fixture();
        fs::write(root.join("build-script"), script).unwrap();
        let observed = execute_cargo(
            &invocation,
            execution_policy(),
            OwnedChild::spawn_direct,
            || false,
            |_, _| Ok(()),
            |_| {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "original sink",
                ))
            },
        )
        .unwrap();
        let execution = observed.execution.unwrap();
        assert_eq!(execution.status.code(), Some(code));
        assert_eq!(execution.stderr, b"drained");
        let error = observed.messages.unwrap_err();
        assert_eq!(
            error.kind(),
            if bad_message {
                io::ErrorKind::InvalidData
            } else {
                io::ErrorKind::PermissionDenied
            }
        );
        assert!(observed.facts.unwrap().is_none());
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
#[cfg(unix)]
fn invocation_cancellation_and_probe_refusals_preserve_host_evidence() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    let (root, mut invocation) = invocation_fixture();
    let cancelled = Arc::new(AtomicBool::new(false));
    let observe_cancel = Arc::clone(&cancelled);
    fs::write(
        root.join("build-script"),
        "printf started >&2\nexec /bin/sleep 60\n",
    )
    .unwrap();
    let observed = execute_cargo(
        &invocation,
        execution_policy(),
        |command| OwnedChild::spawn(command, ic_host_process::child::CleanupPolicy::KillAndWait),
        move || cancelled.load(Ordering::Relaxed),
        move |_, _| {
            observe_cancel.store(true, Ordering::Relaxed);
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap();
    let error = observed.execution.unwrap_err();
    assert!(
        matches!(error, ic_host_process::tool::ToolError::Execution(ref error) if matches!(error.failure, ExecutionFailure::Cancelled))
    );
    assert!(observed.facts.unwrap().is_none());
    fs::write(
        root.join("tool"),
        "#!/bin/sh\nprintf original-probe-error >&2\nexit 29\n",
    )
    .unwrap();
    let error = execute_cargo(
        &invocation,
        execution_policy(),
        |_| panic!("probe failure must precede build spawn"),
        || false,
        |_, _| Ok(()),
        |_| Ok(()),
    )
    .err()
    .unwrap();
    let error = error
        .get_ref()
        .unwrap()
        .downcast_ref::<ic_host_process::tool::ToolError>()
        .unwrap();
    assert_eq!(error.evidence().unwrap().status.unwrap().code(), Some(29));
    assert_eq!(error.evidence().unwrap().stderr, b"original-probe-error");
    fs::write(
        root.join("tool"),
        "#!/bin/sh\nexec /bin/dd if=/dev/zero bs=131072 count=1 2>/dev/null\n",
    )
    .unwrap();
    let error = execute_cargo(
        &invocation,
        execution_policy(),
        |_| panic!("partial identity must precede build spawn"),
        || false,
        |_, _| Ok(()),
        |_| Ok(()),
    )
    .err()
    .unwrap();
    assert!(
        error
            .get_ref()
            .unwrap()
            .is::<ic_host_process::tool::ToolError>()
    );
    invocation.environment.remove(std::ffi::OsStr::new("RUSTC"));
    let error = execute_cargo(
        &invocation,
        execution_policy(),
        |_| panic!("compiler selection is required"),
        || false,
        |_, _| Ok(()),
        |_| Ok(()),
    )
    .err()
    .unwrap();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    fs::remove_dir_all(root).unwrap();
}
