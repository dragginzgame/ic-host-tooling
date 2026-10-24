//! Named examples share Cargo identity/output selection while callers own execution.

use ic_build::cargo::selection::{CargoFeatureSelection, CargoSelection};
use ic_host_process::child::OwnedChild;
use support::cargo::spawn_cargo;

mod support;

#[cfg(unix)]
use ic_host_process::testing::write_executable_script;

use ic_build::artifacts::{
    CapturedArtifact, CargoInputProbePolicy, InputEvidence, InputEvidenceError, InputEvidenceSpec,
    ResolvedCargoBuildInputs, WasmBuildError, WasmBuildSpec, build_wasm_canisters_cached,
    read_cargo_depfile_inputs, resolve_cargo_build_inputs,
    resolve_cargo_build_inputs_with_probe_policy,
};
use ic_host_process::tool::{OutputLimit, OutputLimits, capture_command};
use std::{
    fs,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

fn workspace() -> std::path::PathBuf {
    let root = support::fs_fixture::unique_temp_directory("cargo-example");
    let workspace = root.join("workspace");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir(workspace.join(".cargo")).unwrap();
    fs::write(
        workspace.join(".cargo/config.toml"),
        "[build]\ntarget-dir=\"configured\"\n",
    )
    .unwrap();
    fs::create_dir_all(workspace.join("examples")).unwrap();
    fs::write(workspace.join("Cargo.toml"), concat!(
        "[package]\nname=\"example-fixture\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[workspace]\n",
        "[lib]\ncrate-type=[\"cdylib\"]\n[features]\nprobe=[]\n",
        "[[example]]\nname=\"probe\"\ncrate-type=[\"cdylib\"]\nrequired-features=[\"probe\"]\n",
        "[[example]]\nname=\"other-probe\"\ncrate-type=[\"cdylib\"]\n",
    )).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "#[unsafe(no_mangle)] pub extern \"C\" fn library() {}\n",
    )
    .unwrap();
    fs::write(
        workspace.join("examples/probe.rs"),
        "#[unsafe(no_mangle)] pub extern \"C\" fn probe() {}\n",
    )
    .unwrap();
    fs::write(
        workspace.join("examples/other-probe.rs"),
        "#[unsafe(no_mangle)] pub extern \"C\" fn other() {}\n",
    )
    .unwrap();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    assert!(
        Command::new(cargo)
            .current_dir(&workspace)
            .args(["generate-lockfile", "--offline"])
            .status()
            .unwrap()
            .success()
    );
    root
}

fn recipe(root: &Path) -> WasmBuildSpec {
    let workspace = root.join("workspace");
    WasmBuildSpec::new(
        &workspace,
        &workspace.join("target/cargo"),
        &["example-fixture"],
        support::cargo::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_example("probe")
    .with_cargo_selection(CargoSelection {
        features: CargoFeatureSelection {
            features: ["probe".into()].into(),
            ..Default::default()
        },
        locked: true,
        offline: true,
        ..Default::default()
    })
}

fn evidence(inputs: &ResolvedCargoBuildInputs) -> InputEvidenceSpec {
    let spec = inputs.inputs().iter().fold(
        InputEvidenceSpec::new(b"caller-owned-recipe".as_slice()),
        |spec, input| spec.with_input(input.path()),
    );
    inputs
        .exclusions()
        .iter()
        .fold(spec, InputEvidenceSpec::with_excluded_root)
}

fn execute(spec: &WasmBuildSpec) {
    let mut command = spec.cargo_build_command().unwrap();
    // An external caller keeps its deadline, output budgets and success admission.
    capture_command(
        &mut command,
        OutputLimits {
            stdout: OutputLimit::Terminate(1024 * 1024),
            stderr: OutputLimit::Terminate(1024 * 1024),
            timeout: Some(Duration::from_secs(600)),
        },
    )
    .unwrap()
    .require_success()
    .unwrap();
}

#[test]
fn named_examples_build_and_reuse_without_aliasing_library_or_other_example() {
    for shared in [false, true] {
        let root = workspace();
        let mut spec = recipe(&root).with_isolated_builds();
        if shared {
            spec = spec.with_shared_incremental_target(root.join("workspace/target/shared"));
        }
        assert_eq!(spec.example(), Some("probe"));
        let inputs = resolve_cargo_build_inputs(&spec).unwrap();
        assert_eq!(
            inputs.artifact_paths().unwrap(),
            &[
                root.join(
                    "workspace/target/cargo/wasm32-unknown-unknown/debug/examples/probe.wasm"
                )
            ]
        );
        let first = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
        let bytes = fs::read(&first.record().artifacts()[0]).unwrap();
        assert!(bytes.starts_with(b"\0asm"));
        assert!(first.record().artifacts()[0].ends_with("debug/examples/probe.wasm"));
        assert!(
            build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
                .unwrap()
                .is_reused()
        );
        let other = build_wasm_canisters_cached(
            &spec.clone().with_example("other-probe"),
            OwnedChild::spawn_direct,
        )
        .unwrap();
        assert_ne!(first.record().fingerprint(), other.record().fingerprint());
        assert!(other.record().artifacts()[0].ends_with("debug/examples/other_probe.wasm"));
        let library = WasmBuildSpec::new(
            &root.join("workspace"),
            &root.join("workspace/target/cargo"),
            &["example-fixture"],
            support::cargo::environment(),
            CONFIGURATION_LIMITS,
        )
        .with_cargo_selection(CargoSelection {
            features: CargoFeatureSelection {
                features: ["probe".into()].into(),
                ..Default::default()
            },
            locked: true,
            offline: true,
            ..Default::default()
        })
        .with_isolated_builds();
        let library = build_wasm_canisters_cached(&library, OwnedChild::spawn_direct).unwrap();
        assert_ne!(first.record().fingerprint(), library.record().fingerprint());
        let manifest = root.join("workspace/Cargo.toml");
        let original = fs::read_to_string(&manifest).unwrap();
        fs::write(
            &manifest,
            original.replace(
                "crate-type=[\"cdylib\"]\nrequired-features",
                "crate-type=[\"rlib\"]\nrequired-features",
            ),
        )
        .unwrap();
        assert!(
            resolve_cargo_build_inputs(&spec)
                .unwrap()
                .artifact_paths()
                .is_err()
        );
        assert!(matches!(
            build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct),
            Err(WasmBuildError::InvalidSpec { .. })
        ));
        assert_eq!(fs::read(&first.record().artifacts()[0]).unwrap(), bytes);
        drop((first, other, library));
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn external_example_execution_rechecks_inputs_before_capturing_exact_bytes() {
    let root = workspace();
    let spec = recipe(&root);
    let mut cancelled = || false;
    let mut policy = CargoInputProbePolicy {
        deadline: Some(Instant::now() + Duration::from_secs(600)),
        cleanup: ic_host_process::child::CleanupPolicy::TermThenKill {
            grace: Duration::from_millis(100),
            reap_timeout: Duration::from_secs(2),
        },
        cancelled: &mut cancelled,
    };
    let inputs = resolve_cargo_build_inputs_with_probe_policy(&spec, &mut policy).unwrap();
    assert_eq!(
        inputs.fingerprint(),
        resolve_cargo_build_inputs(&spec).unwrap().fingerprint()
    );
    let before = InputEvidence::capture(&evidence(&inputs)).unwrap();
    execute(&spec);
    assert!(
        inputs
            .is_current_with_probe_policy(&spec, &mut policy)
            .unwrap()
    );
    let after =
        evidence(&resolve_cargo_build_inputs_with_probe_policy(&spec, &mut policy).unwrap());
    before.validate_after(&after).unwrap();
    let output = &inputs.artifact_paths().unwrap()[0];
    let bytes = fs::read(output).unwrap();
    fs::create_dir(root.join("retained")).unwrap();
    let capture = CapturedArtifact::capture(output, &root.join("retained"), bytes.len()).unwrap();
    fs::write(output, b"overwritten mutable Cargo output").unwrap();
    assert_eq!(fs::read(capture.path()).unwrap(), bytes);
    assert!(!root.join("workspace/target/cargo/.ic-testkit").exists());
    assert!(!root.join("workspace/configured").exists());
    fs::write(root.join("workspace/examples/probe.rs"), "changed source").unwrap();
    assert!(!inputs.is_content_current().unwrap());
    let after = evidence(&inputs);
    assert!(matches!(
        before.validate_after(&after),
        Err(InputEvidenceError::ChangedInput { .. })
    ));
    drop(capture);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn late_example_depfile_inputs_require_prebuild_authority() {
    for predeclared in [false, true] {
        let root = workspace();
        let external = root.join("external.rs");
        fs::write(
            &external,
            "#[unsafe(no_mangle)] pub extern \"C\" fn external() {}\n",
        )
        .unwrap();
        fs::write(
            root.join("workspace/examples/probe.rs"),
            "include!(\"../../external.rs\");\n",
        )
        .unwrap();
        let mut spec = recipe(&root);
        if predeclared {
            spec = spec.with_additional_inputs([&external]);
        }
        let inputs = resolve_cargo_build_inputs(&spec).unwrap();
        let before = InputEvidence::capture(&evidence(&inputs)).unwrap();
        execute(&spec);
        let target = root.join("workspace/target/cargo");
        let paths = read_cargo_depfile_inputs(
            &root.join("workspace"),
            &target.join("wasm32-unknown-unknown/debug/examples/probe.d"),
            std::slice::from_ref(&target),
            64 * 1024,
        )
        .unwrap();
        assert!(paths.contains(&external));
        let after_spec = paths
            .iter()
            .fold(evidence(&inputs), InputEvidenceSpec::with_input);
        let after = after_spec;
        if predeclared {
            before.validate_after(&after).unwrap();
            fs::remove_file(&external).unwrap();
            assert!(before.validate_after(&after).is_err());
        } else {
            assert!(
                matches!(before.validate_after(&after), Err(InputEvidenceError::UnobservedInput { path }) if path == external)
            );
        }
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn unsafe_or_conflicting_example_selectors_fail_before_dispatch() {
    let root = workspace();
    let spec = recipe(&root);
    for name in ["", ".", "..", "../probe", "/probe"] {
        assert!(
            spec.clone()
                .with_example(name)
                .cargo_build_command()
                .is_err()
        );
    }
    let multi = WasmBuildSpec::new(
        &root.join("workspace"),
        &root.join("target"),
        &["one", "two"],
        support::cargo::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_example("probe");
    assert!(multi.cargo_build_command().is_err());
    assert!(
        resolve_cargo_build_inputs(&spec.with_example("missing"))
            .unwrap()
            .artifact_paths()
            .is_err()
    );
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn external_command_keeps_caller_failure_deadline_and_cancellation_policy() {
    use ic_host_process::{
        child::{CleanupPolicy, OwnedChild},
        tool::{ExecutionFailure, SuccessfulExit, ToolError, communicate_child},
    };
    use std::process::Stdio;
    let root = workspace();
    let program = root.join("cargo-script");
    write_executable_script(
        &program,
        "#!/bin/sh\nprintf diagnostic >&2\nprintf evidence\nexit 37\n",
    )
    .expect("write executable test script");
    let spec = recipe(&root).with_cargo_program(&program);
    let limits = OutputLimits {
        stdout: OutputLimit::Terminate(1024),
        stderr: OutputLimit::Terminate(1024),
        timeout: Some(Duration::from_secs(10)),
    };
    let error = capture_command(&mut spec.cargo_build_command().unwrap(), limits)
        .unwrap()
        .require_success()
        .unwrap_err();
    let ToolError::Execution(error) = error else {
        panic!("expected process evidence")
    };
    assert!(matches!(error.failure, ExecutionFailure::ExitStatus));
    assert_eq!(error.evidence.status.unwrap().code(), Some(37));
    assert_eq!(error.evidence.stdout, b"evidence");
    assert_eq!(error.evidence.stderr, b"diagnostic");
    write_executable_script(
        &program,
        "#!/bin/sh\nprintf diagnostic >&2\nexec sleep 30\n",
    )
    .expect("write executable test script");
    let error = capture_command(
        &mut spec.cargo_build_command().unwrap(),
        OutputLimits {
            timeout: Some(Duration::from_millis(100)),
            ..limits
        },
    )
    .unwrap_err();
    let ToolError::Execution(error) = error else {
        panic!("expected process evidence")
    };
    assert!(matches!(error.failure, ExecutionFailure::TimedOut));
    assert_eq!(error.evidence.stderr, b"diagnostic");
    let mut command = spec.cargo_build_command().unwrap();
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = OwnedChild::spawn(&mut command, CleanupPolicy::KillAndWait).unwrap();
    let error =
        communicate_child(&mut child, None, limits, SuccessfulExit::Cleanup, || true).unwrap_err();
    let ToolError::Execution(error) = error else {
        panic!("expected cancellation evidence")
    };
    assert!(matches!(error.failure, ExecutionFailure::Cancelled));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn prepared_metadata_sharing_preserves_distinct_example_selections() {
    use ic_build::artifacts::{LabeledWasmBuildSpec, WasmBuildBatchConfig, WasmBuildInputSnapshot};
    let root = workspace();
    let first = recipe(&root).with_isolated_builds();
    let second = first.clone().with_example("other-probe");
    let guard = std::sync::Mutex::new(());
    let guard = guard.lock().unwrap();
    let snapshot = WasmBuildInputSnapshot::prepare_assuming_sources_immutable(
        &guard,
        &[first.clone(), second.clone()],
    )
    .unwrap();
    assert_eq!(snapshot.metrics().input_resolution_runs(), 1);
    let recipes = [
        LabeledWasmBuildSpec::new("probe", first),
        LabeledWasmBuildSpec::new("other", second),
    ];
    let cold = snapshot
        .build_batch(&recipes, WasmBuildBatchConfig::new(), None, spawn_cargo)
        .unwrap();
    assert!(cold.is_success());
    let identities: Vec<_> = cold
        .outcomes()
        .map(|entry| entry.outcome().unwrap().record().fingerprint())
        .collect();
    assert_ne!(identities[0], identities[1]);
    let warm = snapshot
        .build_batch(&recipes, WasmBuildBatchConfig::new(), None, spawn_cargo)
        .unwrap();
    assert!(
        warm.outcomes()
            .all(|entry| entry.outcome().unwrap().is_reused())
    );
    let unprepared = [LabeledWasmBuildSpec::new(
        "unprepared",
        recipes[0].spec().clone().with_example("missing"),
    )];
    assert!(
        snapshot
            .build_batch(&unprepared, WasmBuildBatchConfig::new(), None, spawn_cargo,)
            .is_err()
    );
    drop((cold, warm, snapshot));
    drop(guard);
    fs::remove_dir_all(root).unwrap();
}

// This caller admits small configuration trees; libraries do not select this policy.
const CONFIGURATION_LIMITS: ic_build::cargo::environment::ConfigurationLimits =
    ic_build::cargo::environment::ConfigurationLimits {
        file_bytes: 1024 * 1024,
        total_bytes: 16 * 1024 * 1024,
        files: 128,
        depth: 32,
    };
