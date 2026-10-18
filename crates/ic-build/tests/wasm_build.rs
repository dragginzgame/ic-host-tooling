//! Actual Cargo/Wasm compilation, exact inputs and publication qualification.

use ic_build::cargo::selection::CargoSelection;
use ic_host_process::child::OwnedChild;
use support::cargo::spawn_cargo;

mod support;

#[cfg(unix)]
use ic_host_process::testing::write_executable_script;

use ic_host_fs::read::read_file;

use ic_build::artifacts::{
    ArtifactCacheError, ArtifactCachePreparation, ArtifactCacheSpec, CacheRetentionLimits,
    LabeledWasmBuildSpec, SharedIncrementalTargetMaintenanceOutcome, WasmBuildBatchConfig,
    WasmBuildBatchContractError, WasmBuildBatchEntry, WasmBuildBatchMetrics,
    WasmBuildBatchProgressEvent, WasmBuildBatchReport, WasmBuildError, WasmBuildInputSnapshot,
    WasmBuildOutcome, WasmBuildPhase, WasmBuildProgressConfig, WasmBuildProgressEvent,
    build_wasm_canisters_cached, inspect_shared_incremental_target, prepare_artifact_cache,
    prune_wasm_build_cache, resolve_cargo_build_inputs,
};
use std::{
    fs,
    path::Path,
    sync::{Arc, Barrier, Mutex},
    thread,
    time::Duration,
};

use support::fs_fixture::unique_temp_directory as unique_temp_dir;
use support::wasm_path;

#[cfg(unix)]
use ic_host_process::testing::crash_barrier::wait_for_path;
#[cfg(unix)]
use std::ffi::OsString;

const PERF_PROBE_PACKAGE: &str = "fixture";

#[test]
fn hyphenated_packages_use_the_default_library_artifact_in_every_cache_mode() {
    for shared in [false, true] {
        let root = real_wasm_workspace();
        let manifest = root.join("Cargo.toml");
        let original = fs::read_to_string(&manifest)
            .unwrap()
            .replace("name = \"fixture\"", "name = \"fixture-canister\"");
        fs::write(&manifest, &original).unwrap();
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        assert!(
            std::process::Command::new(cargo)
                .current_dir(&root)
                .args(["generate-lockfile", "--offline"])
                .status()
                .unwrap()
                .success()
        );
        let target = root.join("exact");
        let mut spec =
            support::wasm::spec(&root, &target, &["fixture-canister"]).with_isolated_builds();
        if shared {
            spec = spec.with_shared_incremental_target(root.join("target/shared"));
        }
        let first = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
        assert_eq!(
            first.record().artifacts()[0].file_name().unwrap(),
            "fixture_canister.wasm"
        );
        let bytes = fs::read(&first.record().artifacts()[0]).unwrap();
        assert!(bytes.starts_with(b"\0asm"));
        assert!(
            target
                .join("wasm32-unknown-unknown/debug/fixture_canister.wasm")
                .is_file()
        );
        assert!(
            !target
                .join("wasm32-unknown-unknown/debug/fixture-canister.wasm")
                .exists()
        );
        assert!(
            build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
                .unwrap()
                .is_reused()
        );
        fs::write(
            &manifest,
            original.replace("[lib]", "[lib]\nname=\"renamed\""),
        )
        .unwrap();
        let renamed_inputs = resolve_cargo_build_inputs(&spec).unwrap();
        assert_eq!(
            renamed_inputs.artifact_paths().unwrap()[0]
                .file_name()
                .unwrap(),
            "renamed.wasm"
        );
        let renamed = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
            .expect("capture Cargo-reported custom library");
        assert!(!renamed.is_reused());
        assert_ne!(first.record().fingerprint(), renamed.record().fingerprint());
        assert_eq!(
            renamed.record().artifacts()[0].file_name().unwrap(),
            "renamed.wasm"
        );
        assert!(
            fs::read(&renamed.record().artifacts()[0])
                .unwrap()
                .starts_with(b"\0asm")
        );
        assert!(
            build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
                .unwrap()
                .is_reused()
        );
        assert_eq!(fs::read(&first.record().artifacts()[0]).unwrap(), bytes);
        fs::write(&manifest, original).unwrap();
        assert!(
            build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
                .unwrap()
                .is_reused()
        );
        drop((first, renamed));
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn packages_with_colliding_default_library_names_cannot_publish_one_artifact() {
    let root = unique_temp_dir("normalized-library-collision");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers=[\"a\",\"b\"]\nresolver=\"3\"\n",
    )
    .unwrap();
    for (directory, name) in [("a", "same-name"), ("b", "same_name")] {
        fs::create_dir_all(root.join(directory).join("src")).unwrap();
        fs::write(root.join(directory).join("Cargo.toml"),
            format!("[package]\nname=\"{name}\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[lib]\ncrate-type=[\"cdylib\"]\n")).unwrap();
        fs::write(root.join(directory).join("src/lib.rs"), "").unwrap();
    }
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    assert!(
        std::process::Command::new(cargo)
            .current_dir(&root)
            .args(["generate-lockfile", "--offline"])
            .status()
            .unwrap()
            .success()
    );
    let spec = support::wasm::spec(&root, &root.join("exact"), &["same-name", "same_name"]);
    resolve_cargo_build_inputs(&spec).unwrap();
    assert!(matches!(
        build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn shared_wasm_target_rejects_library_that_no_longer_produces_wasm() {
    let root = unique_temp_dir("ic-testkit-shared-library-kind");
    let workspace = root.join("workspace");
    write_independent_feature_workspace(&workspace);
    let shared = root.join("shared");
    let spec = support::wasm::spec(&workspace, &root.join("exact"), &["feature_a"])
        .with_shared_incremental_target(&shared);
    let first = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("build initial canister library");
    let before = read_file(&wasm_path(&shared, "feature_a", "debug"), 128 * 1024 * 1024)
        .expect("read bounded Wasm");
    let manifest = workspace.join("feature_a/Cargo.toml");
    let contents = fs::read_to_string(&manifest).expect("read canister manifest");
    for changed in [contents.replace("cdylib", "rlib")] {
        fs::write(&manifest, changed).expect("change the declared library output");
        // Cargo input snapshots remain useful for generic transactional builds.
        resolve_cargo_build_inputs(&spec).expect("resolve changed library inputs");
        let scheduled = spec.clone().with_shared_incremental_target_maintenance(
            ic_build::artifacts::SharedIncrementalTargetMaintenanceConfig::new(
                CacheRetentionLimits::new().with_max_size_bytes(0),
                Duration::ZERO,
            ),
        );
        for candidate in [&spec, &scheduled] {
            let result = build_wasm_canisters_cached(candidate, OwnedChild::spawn_direct);
            assert!(
                matches!(
                    result,
                    Err(ic_build::artifacts::WasmBuildError::InvalidSpec { .. })
                ),
                "a changed library output must not certify old Wasm: {result:?}",
            );
            assert_eq!(
                read_file(&wasm_path(&shared, "feature_a", "debug"), 128 * 1024 * 1024)
                    .expect("read bounded Wasm"),
                before
            );
        }
    }
    assert!(
        !shared
            .join("wasm32-unknown-unknown/debug/libfeature_a.rlib")
            .exists()
    );
    assert!(
        !shared
            .join("wasm32-unknown-unknown/debug/renamed.wasm")
            .exists()
    );
    fs::write(&manifest, contents).expect("restore cdylib library");
    assert!(matches!(
        build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
            .expect("reuse restored canister library"),
        WasmBuildOutcome::Reused(_)
    ));
    drop(first);
    fs::remove_dir_all(root).expect("remove library-kind fixture");
}

#[test]
fn wasm_acquisition_builds_library_when_binary_has_the_same_output_name() {
    for shared_target in [false, true] {
        for observed in [false, true] {
            let root = unique_temp_dir("ic-testkit-library-output-collision");
            let workspace = root.join("workspace");
            write_independent_feature_workspace(&workspace);
            fs::write(workspace.join("feature_a/src/main.rs"), "fn main() {}\n")
                .expect("write same-name binary target");
            let mut spec = support::wasm::spec(&workspace, &root.join("exact"), &["feature_a"]);
            if shared_target {
                spec = spec.with_shared_incremental_target(root.join("shared"));
            }
            let baseline_spec =
                support::wasm::spec(&workspace, &root.join("baseline"), &["feature_a"])
                    .with_isolated_builds();
            let baseline = build_wasm_canisters_cached(&baseline_spec, OwnedChild::spawn_direct)
                .expect("build library-only baseline");
            let library =
                fs::read(&baseline.record().artifacts()[0]).expect("read library baseline");
            let outcome = if observed {
                ic_build::artifacts::build_wasm_canisters_cached_with_progress(
                    &spec,
                    WasmBuildProgressConfig::new(),
                    &mut |_| {},
                    spawn_cargo,
                )
            } else {
                build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
            }
            .expect("acquire default library build");
            assert!(matches!(outcome, WasmBuildOutcome::Built(_)));
            assert!(
                fs::read(&outcome.record().artifacts()[0]).expect("read acquired Wasm") == library,
                "binary must not replace library: shared={shared_target}, observed={observed}",
            );
            assert!(matches!(
                build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
                    .expect("reuse canister library"),
                WasmBuildOutcome::Reused(_)
            ));
            drop(baseline);
            drop(outcome);
            fs::remove_dir_all(root).expect("remove library-output fixture");
        }
    }
}

#[test]
fn shared_wasm_target_refreshes_library_without_building_unrelated_binary() {
    let root = unique_temp_dir("ic-testkit-shared-binary-only");
    let workspace = root.join("workspace");
    write_independent_feature_workspace(&workspace);
    fs::create_dir_all(workspace.join("feature_a/src/bin")).expect("create binary directory");
    fs::write(
        workspace.join("feature_a/src/bin/helper.rs"),
        "fn main() {}\n",
    )
    .expect("write unrelated binary target");
    let target = root.join("exact-target");
    let shared = root.join("shared-target");
    let spec = support::wasm::spec(&workspace, &target, &["feature_a"])
        .with_shared_incremental_target(&shared);
    let first = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("build initial canister Wasm");
    let before = read_file(&wasm_path(&shared, "feature_a", "debug"), 128 * 1024 * 1024)
        .expect("read bounded Wasm");
    fs::write(
        workspace.join("shared/src/lib.rs"),
        "pub fn value() -> u8 { 2 }\n",
    )
    .expect("change source while retaining old canister output");

    let rebuilt = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("build changed canister Wasm");
    assert!(matches!(rebuilt, WasmBuildOutcome::Built(_)));
    assert_ne!(
        read_file(&wasm_path(&shared, "feature_a", "debug"), 128 * 1024 * 1024)
            .expect("read bounded Wasm"),
        before
    );
    assert!(matches!(
        build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
            .expect("reuse matching canister Wasm"),
        WasmBuildOutcome::Reused(_)
    ));
    drop(first);
    drop(rebuilt);
    fs::remove_dir_all(root).expect("remove binary-only fixture");
}

#[test]
fn shared_wasm_target_uses_updated_cargo_profile_before_publication() {
    let root = unique_temp_dir("ic-testkit-shared-profile-mismatch");
    let workspace = root.join("workspace");
    write_independent_feature_workspace(&workspace);
    let target = root.join("exact-target");
    let shared = root.join("shared-target");
    let spec = support::wasm::spec(&workspace, &target, &["feature_a"])
        .with_shared_incremental_target(&shared)
        .with_cargo_selection(CargoSelection {
            profile: Some("release".into()),
            ..Default::default()
        });
    let first = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("build initial release Wasm");
    let before = read_file(
        &wasm_path(&shared, "feature_a", "release"),
        128 * 1024 * 1024,
    )
    .expect("read bounded Wasm");
    fs::write(
        workspace.join("shared/src/lib.rs"),
        "pub fn value() -> u8 { 2 }\n",
    )
    .expect("change source while retaining old release output");

    let debug_spec = spec.clone().with_cargo_selection(CargoSelection {
        offline: true,
        ..Default::default()
    });
    let debug = build_wasm_canisters_cached(&debug_spec, OwnedChild::spawn_direct)
        .expect("build changed debug Wasm");
    assert!(matches!(debug, WasmBuildOutcome::Built(_)));
    assert!(wasm_path(&shared, "feature_a", "debug").is_file());
    assert!(wasm_path(&target, "feature_a", "debug").is_file());
    assert_eq!(
        fs::read(&debug.record().artifacts()[0]).unwrap(),
        fs::read(wasm_path(&target, "feature_a", "debug")).unwrap(),
    );
    assert_eq!(
        read_file(
            &wasm_path(&shared, "feature_a", "release"),
            128 * 1024 * 1024
        )
        .expect("read bounded Wasm"),
        before
    );

    let rebuilt = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("build changed release Wasm");
    assert!(matches!(rebuilt, WasmBuildOutcome::Built(_)));
    assert_ne!(
        read_file(
            &wasm_path(&shared, "feature_a", "release"),
            128 * 1024 * 1024
        )
        .expect("read bounded Wasm"),
        before
    );
    assert!(matches!(
        build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
            .expect("reuse matching release Wasm"),
        WasmBuildOutcome::Reused(_)
    ));
    assert!(
        build_wasm_canisters_cached(&debug_spec, OwnedChild::spawn_direct)
            .unwrap()
            .is_reused()
    );
    drop(debug);
    drop(first);
    drop(rebuilt);
    fs::remove_dir_all(root).expect("remove shared profile fixture");
}

#[test]
fn independent_wasm_batch_preserves_standalone_feature_resolution() {
    let root = unique_temp_dir("ic-testkit-independent-feature-batch");
    let workspace = root.join("workspace");
    write_independent_feature_workspace(&workspace);
    let target = root.join("exact-target");
    let specs = [
        LabeledWasmBuildSpec::new(
            "feature-a",
            support::wasm::spec(&workspace, &target, &["feature_a"]),
        ),
        LabeledWasmBuildSpec::new(
            "feature-b",
            support::wasm::spec(&workspace, &target, &["feature_b"]),
        ),
    ];

    let batch = ic_build::artifacts::build_wasm_canisters_cached_batch(
        &specs,
        WasmBuildBatchConfig::new(),
        None,
        spawn_cargo,
    )
    .unwrap();

    assert!(batch.is_success());
    assert_cold_batch_metrics(batch.metrics());
    assert_eq!(batch.outcomes().count(), 2);
    assert_eq!(
        batch
            .outcomes()
            .map(WasmBuildBatchEntry::label)
            .collect::<Vec<_>>(),
        ["feature-a", "feature-b"]
    );
    assert!(
        batch
            .outcomes()
            .all(|entry| matches!(entry.outcome().unwrap(), WasmBuildOutcome::Built(_)))
    );
    assert!(wasm_path(&target, "feature_a", "debug").is_file());
    assert!(wasm_path(&target, "feature_b", "debug").is_file());
    assert!(batch.outcomes().all(|entry| {
        entry
            .outcome()
            .unwrap()
            .record()
            .exact_cache_path()
            .is_dir()
    }));
    let first_exact_cache = batch
        .outcomes()
        .next()
        .expect("first batch outcome")
        .outcome()
        .unwrap()
        .record()
        .exact_cache_path()
        .to_owned();
    // Release the records before simulating external cache loss.
    drop(batch);
    fs::remove_dir_all(&first_exact_cache)
        .expect("remove exact entry while keeping public outputs");

    let with_invalid_middle = [
        specs[0].clone(),
        LabeledWasmBuildSpec::new("invalid", support::wasm::spec(&workspace, &target, &[])),
        specs[1].clone(),
    ];
    let mut completed = Vec::new();
    let with_failure = ic_build::artifacts::build_wasm_canisters_cached_batch(
        &with_invalid_middle,
        ic_build::artifacts::WasmBuildBatchConfig::new(),
        Some((WasmBuildProgressConfig::new(), &mut |event| match event {
            WasmBuildBatchProgressEvent::BuildFinished { index, label } => {
                completed.push((index, label, true));
            }
            WasmBuildBatchProgressEvent::BuildFailed { index, label } => {
                completed.push((index, label, false));
            }
            _ => {}
        })),
        spawn_cargo,
    )
    .expect("valid leased failure batch");
    assert_mixed_batch_with_failure(&with_failure, &completed);
    assert!(
        first_exact_cache.is_dir(),
        "a cache miss must rebuild its immutable cache directory"
    );
    fs::remove_dir_all(root).expect("remove independent feature fixture");
}

#[test]
fn prepared_wasm_input_snapshots_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<WasmBuildInputSnapshot<'static>>();
}

#[test]
fn prepared_wasm_inputs_serve_concurrent_warm_readers() {
    let root = unique_temp_dir("ic-testkit-prepared-wasm-inputs");
    let workspace = root.join("workspace");
    write_independent_feature_workspace(&workspace);
    let first_spec = support::wasm::spec(&workspace, &root.join("exact-a"), &["feature_a"])
        .with_shared_incremental_target(root.join("shared-a"));
    let second_spec = support::wasm::spec(&workspace, &root.join("exact-b"), &["feature_b"])
        .with_shared_incremental_target(root.join("shared-b"));
    for spec in [&first_spec, &second_spec] {
        let built = build_wasm_canisters_cached(spec, OwnedChild::spawn_direct)
            .expect("warm exact Wasm entry");
        assert!(!built.is_reused());
    }

    let source_write_exclusion = Mutex::new(());
    let source_guard = source_write_exclusion
        .lock()
        .expect("lock immutable prepared-source fixture");
    let snapshot = WasmBuildInputSnapshot::prepare_assuming_sources_immutable(
        &source_guard,
        &[first_spec.clone(), second_spec.clone()],
    )
    .expect("prepare exact Wasm inputs");
    let preparation = snapshot.metrics();
    assert_eq!(preparation.specifications(), 2);
    assert_eq!(preparation.input_resolution_runs(), 1);
    assert_eq!(preparation.input_resolution_reuses(), 1);
    assert!(preparation.input_resolution_timings().total() > Duration::ZERO);
    assert_eq!(preparation.reader_reuses(), 0);
    assert!(!preparation.is_invalidated());

    let first = [LabeledWasmBuildSpec::new("local-test", first_spec)];
    let second = [LabeledWasmBuildSpec::new("production", second_spec)];
    let barrier = Arc::new(Barrier::new(3));
    let (first_report, second_report) = thread::scope(|scope| {
        let snapshot = &snapshot;
        let first_barrier = Arc::clone(&barrier);
        let first_reader = scope.spawn(move || {
            first_barrier.wait();
            let mut finished = Vec::new();
            let report = snapshot
                .build_batch(
                    &first,
                    WasmBuildBatchConfig::new(),
                    Some((WasmBuildProgressConfig::new(), &mut |event| {
                        if let WasmBuildBatchProgressEvent::BuildFinished { index, label } = event {
                            finished.push((index, label));
                        }
                    })),
                    spawn_cargo,
                )
                .expect("read first prepared input");
            assert_eq!(finished, [(0, "local-test".to_owned())]);
            report
        });
        let second_barrier = Arc::clone(&barrier);
        let second_reader = scope.spawn(move || {
            second_barrier.wait();
            snapshot
                .build_batch(&second, WasmBuildBatchConfig::new(), None, spawn_cargo)
                .expect("read second prepared input")
        });
        barrier.wait();
        (
            first_reader.join().expect("join first prepared reader"),
            second_reader.join().expect("join second prepared reader"),
        )
    });
    for report in [&first_report, &second_report] {
        assert!(
            report
                .outcomes()
                .all(|entry| entry.outcome().unwrap().is_reused())
        );
        assert_prepared_batch_metrics(report.metrics());
    }
    assert_eq!(snapshot.metrics().reader_reuses(), 2);

    let undeclared = [LabeledWasmBuildSpec::new(
        "undeclared",
        support::wasm::spec(&workspace, &root.join("exact-undeclared"), &["feature_a"]),
    )];
    let mut progress_events = 0;
    let error = snapshot
        .build_batch(
            &undeclared,
            WasmBuildBatchConfig::new(),
            Some((WasmBuildProgressConfig::new(), &mut |_| {
                progress_events += 1;
            })),
            spawn_cargo,
        )
        .expect_err("undeclared prepared reader must fail preflight");
    assert!(matches!(
        error,
        WasmBuildBatchContractError::SpecificationNotPrepared {
            index: 0,
            label,
        } if label == "undeclared"
    ));
    assert_eq!(progress_events, 0);

    drop(snapshot);
    drop(source_guard);
    fs::remove_dir_all(root).expect("remove prepared-input fixture");
}

fn write_independent_feature_workspace(workspace: &Path) {
    for package in ["shared", "feature_a", "feature_b"] {
        fs::create_dir_all(workspace.join(package).join("src"))
            .expect("create feature fixture package");
    }
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"shared\", \"feature_a\", \"feature_b\"]\nresolver = \"2\"\n",
    )
    .expect("write feature fixture workspace");
    fs::write(
        workspace.join("shared/Cargo.toml"),
        "[package]\nname = \"shared\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
         [features]\na = []\nb = []\n",
    )
    .expect("write shared feature manifest");
    fs::write(
        workspace.join("shared/src/lib.rs"),
        "#[cfg(all(feature = \"a\", feature = \"b\"))]\n\
         compile_error!(\"standalone package features were unified\");\n\
         pub fn value() -> u8 { 1 }\n",
    )
    .expect("write shared feature source");
    for (package, feature) in [("feature_a", "a"), ("feature_b", "b")] {
        fs::write(
            workspace.join(package).join("Cargo.toml"),
            format!(
                "[package]\nname = \"{package}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
                 [lib]\ncrate-type = [\"cdylib\"]\n\
                 [dependencies]\nshared = {{ path = \"../shared\", features = [\"{feature}\"] }}\n"
            ),
        )
        .expect("write feature canister manifest");
        fs::write(
            workspace.join(package).join("src/lib.rs"),
            "#[unsafe(no_mangle)]\npub extern \"C\" fn probe() -> u8 { shared::value() }\n",
        )
        .expect("write feature canister source");
    }
}

fn assert_cold_batch_metrics(metrics: WasmBuildBatchMetrics) {
    assert_eq!(metrics.specifications(), 2);
    assert_eq!(metrics.succeeded(), 2);
    assert_eq!(metrics.failed(), 0);
    assert_eq!(metrics.built(), 2);
    assert_eq!(metrics.reused(), 0);
    assert_eq!(metrics.input_resolution_runs(), 1);
    assert_eq!(metrics.input_resolution_reuses(), 1);
    assert!(metrics.successful_timings().input_resolution().total() > Duration::ZERO);
    assert!(metrics.successful_timings().cargo_build().is_some());
}

fn assert_prepared_batch_metrics(metrics: WasmBuildBatchMetrics) {
    assert_eq!(metrics.succeeded(), 1);
    assert_eq!(metrics.failed(), 0);
    assert_eq!(metrics.input_resolution_runs(), 0);
    assert_eq!(metrics.input_resolution_reuses(), 0);
    assert_eq!(metrics.input_resolution_prepared_reuses(), 1);
    assert_eq!(
        metrics.successful_timings().input_resolution().total(),
        Duration::ZERO
    );
}

fn assert_mixed_batch_with_failure(
    report: &WasmBuildBatchReport,
    completed: &[(usize, String, bool)],
) {
    assert!(!report.is_success());
    assert_eq!(
        completed
            .iter()
            .map(|(index, label, success)| (*index, label.as_str(), *success))
            .collect::<Vec<_>>(),
        [
            (0, "feature-a", true),
            (1, "invalid", false),
            (2, "feature-b", true)
        ],
    );
    assert_eq!(report.outcomes().count(), 2);
    assert_eq!(
        report
            .outcomes()
            .map(|entry| entry.outcome().unwrap().is_reused())
            .collect::<Vec<_>>(),
        [false, true]
    );
    assert_eq!(
        report
            .failures()
            .map(|failure| (failure.index(), failure.label()))
            .collect::<Vec<_>>(),
        [(1, "invalid")]
    );
    let metrics = report.metrics();
    assert_eq!(metrics.succeeded(), 2);
    assert_eq!(metrics.failed(), 1);
    assert_eq!(metrics.built(), 1);
    assert_eq!(metrics.reused(), 1);
    assert_eq!(metrics.input_resolution_runs(), 1);
    assert_eq!(metrics.input_resolution_reuses(), 1);
}

#[test]
fn exact_wasm_cache_coordinates_overlapping_builds() {
    let workspace = real_wasm_workspace();
    let target_dir = unique_temp_dir("ic-testkit-overlapping-wasm-cache-target");
    let spec = support::wasm::spec(&workspace, &target_dir, &[PERF_PROBE_PACKAGE]);
    let start = Arc::new(Barrier::new(3));

    let workers = std::array::from_fn::<_, 2, _>(|_| {
        let worker_spec = spec.clone();
        let worker_start = Arc::clone(&start);
        thread::spawn(move || {
            worker_start.wait();
            build_wasm_canisters_cached(&worker_spec, OwnedChild::spawn_direct)
                .expect("coordinated exact Wasm build should succeed")
        })
    });
    start.wait();

    let outcomes = workers
        .into_iter()
        .map(|worker| worker.join().expect("Wasm build worker should not panic"))
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, WasmBuildOutcome::Built(_)))
            .count(),
        1,
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, WasmBuildOutcome::Reused(_)))
            .count(),
        1,
    );
    assert_eq!(
        outcomes[0].record().fingerprint(),
        outcomes[1].record().fingerprint(),
    );

    fs::remove_dir_all(target_dir).expect("clean overlapping cache target dir");
}

#[test]
fn shared_incremental_wasm_cache_keeps_mutable_cargo_state_outside_exact_entries() {
    let workspace = real_wasm_workspace();
    let target_dir = unique_temp_dir("ic-testkit-shared-incremental-cache");
    let shared_target = unique_temp_dir("ic-testkit-shared-incremental-cargo-target");
    let marker = shared_target.join("caller-owned-marker");
    fs::write(&marker, b"preserve").expect("write caller-owned shared-target marker");
    let spec = support::wasm::spec(&workspace, &target_dir, &[PERF_PROBE_PACKAGE])
        .with_shared_incremental_target(&shared_target);

    let resolved = resolve_cargo_build_inputs(&spec).expect("resolve exact Cargo inputs");
    assert_ne!(resolved.inputs(), []);
    assert!(resolved.is_current(&spec).expect("revalidate Cargo inputs"));

    let first = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("build through shared Cargo target");
    assert_eq!(first.record().facts(), resolved.facts());
    assert!(matches!(first, WasmBuildOutcome::Built(_)));
    assert!(
        first
            .record()
            .timings()
            .shared_incremental_lock_wait()
            .is_some()
    );
    assert!(shared_target.join("wasm32-unknown-unknown").is_dir());
    assert!(marker.is_file());
    let inspection = inspect_shared_incremental_target(&spec)
        .expect("inspect shared Cargo target")
        .expect("built shared Cargo target should exist");
    assert_eq!(
        inspection.target_dir(),
        shared_target
            .canonicalize()
            .expect("canonicalize shared Cargo target")
    );
    assert!(inspection.logical_size_bytes() > 0);
    let last_used = inspection.last_used();

    let cache_entry = first.record().exact_cache_path();
    assert!(cache_entry.is_dir());
    assert!(
        !cache_entry
            .join("wasm32-unknown-unknown/debug/incremental")
            .exists()
    );

    let reused = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("reuse exact shared-mode output");
    assert!(matches!(reused, WasmBuildOutcome::Reused(_)));
    assert!(
        reused
            .record()
            .timings()
            .shared_incremental_lock_wait()
            .is_none(),
        "an immediate exact hit should not acquire the shared-target lock"
    );
    assert_eq!(
        inspect_shared_incremental_target(&spec)
            .expect("reinspect shared Cargo target")
            .expect("shared Cargo target must still exist")
            .last_used(),
        last_used,
        "an exact hit must not touch caller-owned shared Cargo state"
    );

    let changed = spec
        .clone()
        .with_environment(support::cargo::environment_with([(
            "IC_TESTKIT_SHARED_INCREMENTAL_PROBE",
            "changed",
        )]));
    let rebuilt = build_wasm_canisters_cached(&changed, OwnedChild::spawn_direct)
        .expect("build changed exact identity");
    assert!(matches!(rebuilt, WasmBuildOutcome::Built(_)));
    assert_ne!(first.record().fingerprint(), rebuilt.record().fingerprint());
    assert!(marker.is_file());

    let restored = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("restore original exact output");
    assert!(matches!(restored, WasmBuildOutcome::Reused(_)));
    assert!(restored.record().timings().cargo_build().is_none());

    prune_wasm_build_cache(
        &target_dir,
        CacheRetentionLimits::new().with_max_size_bytes(0),
    )
    .expect("prune exact output entries");
    assert!(marker.is_file(), "retention must not own the shared target");

    fs::remove_dir_all(target_dir).expect("clean shared-mode exact cache");
    fs::remove_dir_all(shared_target).expect("clean shared Cargo target");
}

#[test]
fn scheduled_shared_target_maintenance_participates_in_wasm_acquisition() {
    let workspace = real_wasm_workspace();
    let root = unique_temp_dir("ic-testkit-integrated-shared-maintenance");
    let target_dir = root.join("exact");
    let shared_target = root.join("shared");
    assert!(!shared_target.exists());
    let spec = support::wasm::spec(&workspace, &target_dir, &[PERF_PROBE_PACKAGE])
        .with_shared_incremental_target(&shared_target)
        .with_shared_incremental_target_maintenance_at_most_every(
            CacheRetentionLimits::new().with_max_size_bytes(u64::MAX),
            Duration::from_hours(1),
        );

    let mut first_events = Vec::new();
    let first = ic_build::artifacts::build_wasm_canisters_cached_with_progress(
        &spec,
        WasmBuildProgressConfig::new().without_heartbeats(),
        &mut |event| first_events.push(event),
        spawn_cargo,
    )
    .expect("build with integrated shared-target maintenance");
    assert!(matches!(first, WasmBuildOutcome::Built(_)));
    assert!(shared_target.is_dir());
    assert!(matches!(
        first.record().shared_incremental_maintenance(),
        Some(SharedIncrementalTargetMaintenanceOutcome::Performed { .. })
    ));
    assert!(first.to_string().contains("shared_maintenance=("));
    let maintenance_started = first_events
        .iter()
        .position(|event| {
            matches!(
                event,
                WasmBuildProgressEvent::SharedTargetMaintenanceStarted { .. }
            )
        })
        .expect("integrated maintenance start event");
    let cargo_started = first_events
        .iter()
        .position(|event| matches!(event, WasmBuildProgressEvent::CargoStarted { .. }))
        .expect("integrated Cargo start event");
    assert!(maintenance_started < cargo_started);
    assert_eq!(
        first_events[..cargo_started]
            .iter()
            .filter(|event| matches!(event, WasmBuildProgressEvent::InputsResolved { .. }))
            .count(),
        1,
        "due maintenance must reuse the acquisition's pre-build resolution",
    );
    assert!(first_events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::SharedTargetMaintenanceFinished {
            outcome: SharedIncrementalTargetMaintenanceOutcome::Performed { .. }
        }
    )));

    let mut reused_events = Vec::new();
    let reused = ic_build::artifacts::build_wasm_canisters_cached_with_progress(
        &spec,
        WasmBuildProgressConfig::new().without_heartbeats(),
        &mut |event| reused_events.push(event),
        spawn_cargo,
    )
    .expect("reuse with integrated shared-target maintenance");
    assert!(matches!(reused, WasmBuildOutcome::Reused(_)));
    let reused_timings = reused.record().timings();
    assert!(reused_timings.shared_incremental_lock_wait().is_some());
    assert!(matches!(
        reused.record().shared_incremental_maintenance(),
        Some(SharedIncrementalTargetMaintenanceOutcome::Skipped { .. })
    ));
    assert_warm_shared_maintenance_progress(&reused_events);

    fs::remove_dir_all(&shared_target).expect("remove shared target before exact hit");
    let recreated = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("exact hit must recreate configured shared maintenance target");
    assert!(matches!(recreated, WasmBuildOutcome::Reused(_)));
    assert!(shared_target.is_dir());
    assert!(matches!(
        recreated.record().shared_incremental_maintenance(),
        Some(SharedIncrementalTargetMaintenanceOutcome::Performed { .. })
    ));

    fs::remove_dir_all(root).expect("clean integrated shared maintenance fixture");
}

fn assert_warm_shared_maintenance_progress(events: &[WasmBuildProgressEvent]) {
    let warm_acquisition_phases = events
        .iter()
        .filter_map(|event| match event {
            WasmBuildProgressEvent::SharedTargetLockStarted { .. } => Some("shared lock started"),
            WasmBuildProgressEvent::SharedTargetLockAcquired { .. } => Some("shared lock acquired"),
            WasmBuildProgressEvent::InputsResolved { .. } => Some("inputs resolved"),
            WasmBuildProgressEvent::SharedTargetMaintenanceStarted { .. } => {
                Some("maintenance started")
            }
            WasmBuildProgressEvent::SharedTargetMaintenanceFinished {
                outcome: SharedIncrementalTargetMaintenanceOutcome::Skipped { .. },
            } => Some("maintenance skipped"),
            WasmBuildProgressEvent::CargoStarted { .. } => Some("Cargo build"),
            WasmBuildProgressEvent::CacheHit { .. } => Some("cache hit"),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        warm_acquisition_phases,
        [
            "shared lock started",
            "shared lock acquired",
            "inputs resolved",
            "maintenance started",
            "maintenance skipped",
            "inputs resolved",
            "cache hit",
        ],
        "maintenance reuses initial resolution; warm validation precedes the cache hit without building",
    );
}

#[test]
fn resolved_cargo_inputs_guard_transactional_artifacts_through_commit() {
    let root = unique_temp_dir("ic-testkit-cargo-input-artifact-bridge");
    let workspace = root.join("workspace");
    let package = workspace.join("package");
    fs::create_dir_all(package.join("src")).expect("create Cargo bridge fixture");
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"package\"]\nresolver = \"2\"\n",
    )
    .expect("write bridge workspace manifest");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"artifact_bridge_probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write bridge package manifest");
    let source = package.join("src/lib.rs");
    fs::write(&source, "pub fn value() -> u8 { 1 }\n").expect("write bridge source");
    let wasm_spec = support::wasm::spec(
        &workspace,
        &workspace.join("target/exact-wasm"),
        &["artifact_bridge_probe"],
    );
    let resolved = resolve_cargo_build_inputs(&wasm_spec).expect("resolve bridge Cargo inputs");
    let destination = workspace.join("target/public/transformed.wasm");
    let mismatched_build_spec =
        wasm_spec
            .clone()
            .with_environment(support::cargo::environment_with([(
                "IC_TESTKIT_BRIDGE_MODE",
                "changed",
            )]));
    let mismatched = ArtifactCacheSpec::new(
        &workspace.join("target/artifact-cache"),
        "cargo-bridge",
        "pipeline/v1",
    )
    .with_cargo_build_inputs("wasm-source", &mismatched_build_spec, &resolved)
    .with_output("transformed.wasm", &destination);
    assert!(matches!(
        prepare_artifact_cache(&mismatched),
        Err(ArtifactCacheError::CargoBuildInputsChanged { .. })
    ));

    let artifact_spec = ArtifactCacheSpec::new(
        &workspace.join("target/artifact-cache"),
        "cargo-bridge",
        "pipeline/v1",
    )
    .with_cargo_build_inputs("wasm-source", &wasm_spec, &resolved)
    .with_output("transformed.wasm", &destination);
    let transaction = match prepare_artifact_cache(&artifact_spec).expect("prepare bridge miss") {
        ArtifactCachePreparation::Build(transaction) => transaction,
        ArtifactCachePreparation::Reused(_) => panic!("new bridge fixture must miss"),
    };
    fs::write(
        transaction
            .output_path("transformed.wasm")
            .expect("resolve transformed output path"),
        b"transformed",
    )
    .expect("write transformed output");
    fs::write(&source, "pub fn value() -> u8 { 2 }\n").expect("change exact Cargo input");

    assert!(matches!(
        transaction.commit(),
        Err(ArtifactCacheError::CargoBuildInputsChanged { .. })
    ));
    assert!(!destination.exists());

    let refreshed = resolve_cargo_build_inputs(&wasm_spec).expect("refresh bridge Cargo inputs");
    let refreshed_spec = ArtifactCacheSpec::new(
        &workspace.join("target/artifact-cache"),
        "cargo-bridge",
        "pipeline/v1",
    )
    .with_cargo_build_inputs("wasm-source", &wasm_spec, &refreshed)
    .with_output("transformed.wasm", &destination);
    let transaction = match prepare_artifact_cache(&refreshed_spec).expect("prepare refreshed miss")
    {
        ArtifactCachePreparation::Build(transaction) => transaction,
        ArtifactCachePreparation::Reused(_) => panic!("changed Cargo inputs must select a miss"),
    };
    fs::write(
        transaction
            .output_path("transformed.wasm")
            .expect("resolve refreshed transformed output path"),
        b"transformed-updated",
    )
    .expect("write refreshed transformed output");
    transaction.commit().expect("commit refreshed artifact");
    assert!(matches!(
        prepare_artifact_cache(&refreshed_spec).expect("reuse refreshed artifact"),
        ArtifactCachePreparation::Reused(_)
    ));
    fs::remove_dir_all(root).expect("remove Cargo bridge fixture");
}

#[test]
fn failed_shared_incremental_build_preserves_cargo_state_without_publishing_an_entry() {
    let workspace = real_wasm_workspace();
    let target_dir = unique_temp_dir("ic-testkit-failed-shared-cache");
    let shared_target = unique_temp_dir("ic-testkit-failed-shared-cargo-target");
    let marker = shared_target.join("caller-owned-marker");
    fs::write(&marker, b"preserve").expect("write failed-build marker");
    let spec = support::wasm::spec(&workspace, &target_dir, &[PERF_PROBE_PACKAGE])
        .with_shared_incremental_target(&shared_target)
        .with_cargo_selection(CargoSelection {
            profile: Some("undefined-profile".into()),
            ..Default::default()
        });
    let fingerprint = resolve_cargo_build_inputs(&spec)
        .expect("resolve failing build identity")
        .fingerprint();

    let error = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect_err("undefined Cargo profile must fail");

    assert!(matches!(
        error,
        WasmBuildError::CommandFailed {
            phase: WasmBuildPhase::CargoBuild,
            ..
        }
    ));
    assert!(marker.is_file());
    assert!(shared_target.is_dir());
    assert!(
        !target_dir
            .join(".ic-testkit/wasm-targets")
            .join(fingerprint.to_hex())
            .exists(),
        "failed build must not publish an exact entry"
    );

    fs::remove_dir_all(target_dir).expect("clean failed shared-mode exact cache");
    fs::remove_dir_all(shared_target).expect("clean failed shared Cargo target");
}

#[test]
#[cfg(unix)]
fn source_changes_during_shared_incremental_build_reject_exact_publication() {
    let root = unique_temp_dir("ic-testkit-shared-input-race");
    let workspace = root.join("workspace");
    let package = workspace.join("race_probe");
    fs::create_dir_all(package.join("src")).expect("create race fixture source directory");
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"race_probe\"]\nresolver = \"2\"\n",
    )
    .expect("write race workspace manifest");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"race_probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
         [lib]\ncrate-type = [\"cdylib\"]\n",
    )
    .expect("write race package manifest");
    let source = package.join("src/lib.rs");
    fs::write(
        &source,
        "#[unsafe(no_mangle)]\npub extern \"C\" fn probe() -> u32 { 1 }\n",
    )
    .expect("write original race source");
    let wrapper = root.join("cargo-wrapper.sh");
    let started = root.join("build-started");
    let release = root.join("release-build");
    write_blocking_cargo_wrapper(&wrapper);
    let real_cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let spec = support::wasm::spec(&workspace, &root.join("exact-cache"), &["race_probe"])
        .with_shared_incremental_target(root.join("shared-target"))
        .with_cargo_program(&wrapper)
        .with_environment(support::cargo::environment_with([
            (OsString::from("REAL_CARGO"), real_cargo),
            (
                OsString::from("IC_TESTKIT_BUILD_STARTED"),
                started.clone().into_os_string(),
            ),
            (
                OsString::from("IC_TESTKIT_RELEASE_BUILD"),
                release.clone().into_os_string(),
            ),
        ]));
    let fingerprint = resolve_cargo_build_inputs(&spec)
        .expect("resolve original race fingerprint")
        .fingerprint();
    let worker = thread::spawn(move || {
        let source_write_exclusion = Mutex::new(());
        let source_guard = source_write_exclusion.lock().unwrap();
        let snapshot = WasmBuildInputSnapshot::prepare_assuming_sources_immutable(
            &source_guard,
            std::slice::from_ref(&spec),
        )
        .unwrap();
        let specs = [LabeledWasmBuildSpec::new("race", spec)];
        let report = snapshot
            .build_batch(&specs, WasmBuildBatchConfig::new(), None, spawn_cargo)
            .expect("valid race-test batch");
        let invalidated = snapshot.metrics().is_invalidated();
        let next = snapshot.build_batch(&specs, WasmBuildBatchConfig::new(), None, spawn_cargo);
        drop(snapshot);
        drop(source_guard);
        (report, invalidated, next)
    });
    wait_for_path(&started, "build start", Duration::from_secs(10)).expect("build start");
    fs::write(
        &source,
        "#[unsafe(no_mangle)]\npub extern \"C\" fn probe() -> u32 { 2 }\n",
    )
    .expect("change source during blocked Cargo build");
    fs::write(&release, b"go").expect("release blocked Cargo build");

    let (report, invalidated, next) = worker
        .join()
        .expect("shared input-race worker must not panic");
    let failure = report
        .failures()
        .next()
        .expect("changed source must fail the leased entry");

    assert!(matches!(
        failure.error().unwrap(),
        WasmBuildError::InputsChangedDuringAcquisition { .. }
    ));
    let details = failure.failure_details().unwrap();
    assert_eq!(details.phase(), WasmBuildPhase::ContentHashing);
    let timings = details.timings();
    assert!(timings.input_resolution().content_hashing() > Duration::ZERO);
    assert!(timings.cargo_build().is_some());
    assert!(timings.artifact_publication().is_some());
    assert!(timings.cleanup().is_some());
    assert!(invalidated);
    assert!(matches!(
        next,
        Err(WasmBuildBatchContractError::SourceLeaseInvalidated)
    ));
    assert!(root.join("shared-target").is_dir());
    assert!(
        !root
            .join("exact-cache/.ic-testkit/wasm-targets")
            .join(fingerprint.to_hex())
            .exists()
    );
    fs::remove_dir_all(root).expect("clean shared input-race fixture");
}

#[test]
#[cfg(unix)]
fn source_race_invalidates_every_prepared_snapshot_reader() {
    let root = unique_temp_dir("ic-testkit-prepared-input-race");
    let workspace = root.join("workspace");
    let package = workspace.join("race_probe");
    fs::create_dir_all(package.join("src")).expect("create prepared-race source directory");
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"race_probe\"]\nresolver = \"2\"\n",
    )
    .expect("write prepared-race workspace manifest");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"race_probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
         [lib]\ncrate-type = [\"cdylib\"]\n",
    )
    .expect("write prepared-race package manifest");
    let source = package.join("src/lib.rs");
    fs::write(
        &source,
        "#[unsafe(no_mangle)]\npub extern \"C\" fn probe() -> u32 { 1 }\n",
    )
    .expect("write original prepared-race source");
    let wrapper = root.join("cargo-wrapper.sh");
    let started = root.join("build-started");
    let release = root.join("release-build");
    write_blocking_cargo_wrapper(&wrapper);
    let real_cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let spec = support::wasm::spec(&workspace, &root.join("exact-cache"), &["race_probe"])
        .with_shared_incremental_target(root.join("shared-target"))
        .with_cargo_program(&wrapper)
        .with_environment(support::cargo::environment_with([
            (OsString::from("REAL_CARGO"), real_cargo),
            (
                OsString::from("IC_TESTKIT_BUILD_STARTED"),
                started.clone().into_os_string(),
            ),
            (
                OsString::from("IC_TESTKIT_RELEASE_BUILD"),
                release.clone().into_os_string(),
            ),
        ]));
    let fingerprint = resolve_cargo_build_inputs(&spec)
        .expect("resolve original prepared-race fingerprint")
        .fingerprint();
    let source_write_exclusion = Mutex::new(());
    let source_guard = source_write_exclusion
        .lock()
        .expect("lock prepared-race source lease");
    let snapshot = WasmBuildInputSnapshot::prepare_assuming_sources_immutable(
        &source_guard,
        std::slice::from_ref(&spec),
    )
    .expect("prepare race input snapshot");
    let specs = [LabeledWasmBuildSpec::new("race", spec)];

    let report = thread::scope(|scope| {
        let worker = scope.spawn(|| {
            snapshot
                .build_batch(
                    &specs,
                    WasmBuildBatchConfig::new(),
                    Some((WasmBuildProgressConfig::new(), &mut |_| {})),
                    spawn_cargo,
                )
                .expect("valid prepared race-test batch")
        });
        wait_for_path(&started, "build start", Duration::from_secs(10)).expect("build start");
        fs::write(
            &source,
            "#[unsafe(no_mangle)]\npub extern \"C\" fn probe() -> u32 { 2 }\n",
        )
        .expect("change source during prepared Cargo build");
        fs::write(&release, b"go").expect("release prepared Cargo build");
        worker.join().expect("prepared input-race worker")
    });
    let failure = report
        .failures()
        .next()
        .expect("changed source must fail the prepared reader");
    assert!(matches!(
        failure.error().unwrap(),
        WasmBuildError::InputsChangedDuringAcquisition { .. }
    ));
    assert_eq!(
        failure.failure_details().unwrap().phase(),
        WasmBuildPhase::ContentHashing
    );
    assert!(snapshot.metrics().is_invalidated());
    assert!(matches!(
        snapshot.build_batch(&specs, WasmBuildBatchConfig::new(), None, spawn_cargo,),
        Err(WasmBuildBatchContractError::SourceLeaseInvalidated)
    ));
    assert!(
        !root
            .join("exact-cache/.ic-testkit/wasm-targets")
            .join(fingerprint.to_hex())
            .exists()
    );

    drop(snapshot);
    drop(source_guard);
    fs::remove_dir_all(root).expect("clean prepared input-race fixture");
}

#[cfg(unix)]
fn write_blocking_cargo_wrapper(path: &std::path::Path) {
    write_executable_script(
        path,
        b"#!/bin/sh\n\
if [ \"$1\" = \"build\" ]; then\n\
  : > \"$IC_TESTKIT_BUILD_STARTED\"\n\
  while [ ! -f \"$IC_TESTKIT_RELEASE_BUILD\" ]; do sleep 0.05; done\n\
fi\n\
exec \"$REAL_CARGO\" \"$@\"\n",
    )
    .expect("write executable test script");
}

fn real_wasm_workspace() -> std::path::PathBuf {
    let root = unique_temp_dir("ic-build-real-wasm-source");
    support::wasm::write_workspace(&root).expect("prepare real Wasm workspace");
    root
}

#[test]
fn custom_library_names_do_not_overwrite_another_packages_outputs() {
    for shared in [false, true] {
        let root = unique_temp_dir("custom-library-layout");
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers=[\"left\",\"right\"]\nresolver=\"3\"\n[profile.custom]\ninherits=\"release\"\n").unwrap();
        for (directory, package, library, value) in
            [("left", "alpha", "beta", 7), ("right", "beta", "alpha", 9)]
        {
            let path = root.join(directory);
            fs::create_dir_all(path.join("src")).unwrap();
            fs::write(path.join("Cargo.toml"), format!("[package]\nname=\"{package}\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[lib]\nname=\"{library}\"\ncrate-type=[\"cdylib\"]\n")).unwrap();
            fs::write(
                path.join("src/lib.rs"),
                format!("#[unsafe(no_mangle)] pub extern \"C\" fn value() -> u32 {{ {value} }}\n"),
            )
            .unwrap();
        }
        let mut spec =
            support::wasm::spec(&root, &root.join("exact with spaces"), &["alpha", "beta"])
                .with_isolated_builds()
                .with_cargo_selection(CargoSelection {
                    profile: Some("custom".into()),
                    offline: true,
                    ..Default::default()
                });
        if shared {
            spec = spec.with_shared_incremental_target(root.join("shared with spaces"));
        }
        let first = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
        let paths = first.record().artifacts();
        assert_eq!(paths[0].file_name().unwrap(), "beta.wasm");
        assert_eq!(paths[1].file_name().unwrap(), "alpha.wasm");
        let bytes = paths
            .iter()
            .map(|path| fs::read(path).unwrap())
            .collect::<Vec<_>>();
        assert_ne!(bytes[0], bytes[1]);
        let reused = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
        assert!(reused.is_reused());
        for (path, expected) in reused.record().artifacts().iter().zip(bytes) {
            assert_eq!(fs::read(path).unwrap(), expected);
        }
        drop((first, reused));
        fs::remove_dir_all(root).unwrap();
    }
}
