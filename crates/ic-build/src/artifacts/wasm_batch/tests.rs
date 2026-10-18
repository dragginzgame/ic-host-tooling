use crate::batch::BatchLabelError;
use crate::cargo::selection::CargoSelection;
use crate::cargo_fixture::spawn_cargo;
#[cfg(unix)]
use ic_host_process::testing::write_executable_script;

use super::{
    BatchMaintenanceTracker, LabeledWasmBuildSpec, WasmBuildBatchConfig,
    WasmBuildBatchContractError, WasmBuildBatchProgressEvent, build_wasm_canisters_cached_batch,
};
use crate::artifacts::{
    CacheRetentionLimits, SharedIncrementalTargetMaintenanceConfig,
    SharedIncrementalTargetMaintenanceFailureMode, WasmBuildBatchEntry, WasmBuildError,
    WasmBuildPhase, WasmBuildProgressConfig, WasmBuildSpec,
};
use std::{path::Path, time::Duration};

#[cfg(unix)]
use crate::artifacts::{
    SharedIncrementalTargetMaintenanceOutcome, WasmBuildInputSnapshot,
    test_support::fake_wasm_build_spec,
};
use crate::fs_fixture::unique_temp_directory;
#[cfg(unix)]
use std::{fs, os::unix::fs::symlink, sync::Mutex};

#[test]
fn empty_independent_batch_succeeds_without_work() {
    let report =
        build_wasm_canisters_cached_batch(&[], WasmBuildBatchConfig::new(), None, spawn_cargo)
            .expect("empty labeled batch");
    assert!(report.is_success());
    assert_eq!(report.outcomes().count(), 0);
    let metrics = report.metrics();
    assert_eq!(metrics.specifications(), 0);
    assert_eq!(metrics.succeeded(), 0);
    assert_eq!(metrics.failed(), 0);
    assert_eq!(metrics.built(), 0);
    assert_eq!(metrics.reused(), 0);
}

#[cfg(unix)]
#[test]
fn library_output_validation_survives_batch_and_prepared_resolution() {
    let (root, valid) = fake_wasm_build_spec("batch-library-output-validation");
    fs::create_dir_all(root.join("ordinary/src")).unwrap();
    fs::write(
        root.join("ordinary/Cargo.toml"),
        "[package]\nname = \"ordinary\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::write(root.join("ordinary/src/lib.rs"), "pub fn ordinary() {}\n").unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\", \"ordinary\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    let invalid = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["ordinary"],
        crate::cargo_fixture::environment(),
    )
    .with_cargo_program(root.join("cargo.sh"));
    let specs = [
        LabeledWasmBuildSpec::new("ordinary", invalid.clone()),
        LabeledWasmBuildSpec::new("canister", valid.clone()),
    ];
    let check = |report: super::WasmBuildBatchReport| {
        assert_eq!(report.outcomes().count(), 1);
        assert_eq!(report.failures().count(), 1);
        let failure = report.failures().next().unwrap();
        assert!(std::ptr::eq(
            failure,
            &raw const report.entries()[failure.index()]
        ));
        let success = report.outcomes().next().unwrap();
        assert!(std::ptr::eq(
            success,
            &raw const report.entries()[success.index()]
        ));
        assert_eq!(failure.label(), "ordinary");
        assert!(matches!(
            failure.error().unwrap(),
            WasmBuildError::InvalidSpec { .. }
        ));
        assert_eq!(
            failure.failure_details().unwrap().phase(),
            WasmBuildPhase::InputDiscovery
        );
        assert_eq!(
            failure.failure_details().unwrap().timings().cargo_build(),
            None
        );
    };
    check(
        build_wasm_canisters_cached_batch(&specs, WasmBuildBatchConfig::new(), None, spawn_cargo)
            .unwrap(),
    );

    let source_lock = Mutex::new(());
    let source_guard = source_lock.lock().unwrap();
    let snapshot = WasmBuildInputSnapshot::prepare_assuming_sources_immutable(
        &source_guard,
        &[invalid, valid],
    )
    .unwrap();
    check(
        snapshot
            .build_batch(&specs, WasmBuildBatchConfig::new(), None, spawn_cargo)
            .unwrap(),
    );
    drop(snapshot);
    drop(source_guard);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn batch_retains_every_indexed_failure() {
    let root = unique_temp_directory("wasm-batch-indexed-failures");
    let target = root.join("target");
    let cargo = root.join("must-not-run-cargo");
    let specs = [
        LabeledWasmBuildSpec::new(
            "root",
            WasmBuildSpec::new(&root, &target, &[], crate::cargo_fixture::environment())
                .with_cargo_program(&cargo),
        ),
        LabeledWasmBuildSpec::new(
            "worker",
            WasmBuildSpec::new(
                &root,
                &target,
                &["fixture"],
                crate::cargo_fixture::environment(),
            )
            .with_cargo_selection(CargoSelection {
                profile: Some(String::new()),
                ..Default::default()
            })
            .with_cargo_program(&cargo),
        ),
    ];

    let report =
        build_wasm_canisters_cached_batch(&specs, WasmBuildBatchConfig::new(), None, spawn_cargo)
            .expect("valid labeled batch");

    assert!(!report.is_success());
    assert_eq!(report.entries().len(), 2);
    assert_eq!(report.entries()[0].label(), "root");
    assert_eq!(report.entries()[1].label(), "worker");
    assert_eq!(
        report
            .failures()
            .map(WasmBuildBatchEntry::label)
            .collect::<Vec<_>>(),
        ["root", "worker"]
    );
    assert!(
        report
            .failures()
            .all(|failure| failure.entry_elapsed() <= report.total())
    );
    assert!(report.failures().all(|failure| {
        failure.failure_details().unwrap().phase() == WasmBuildPhase::Specification
            && failure.failure_details().unwrap().timings().total() <= failure.entry_elapsed()
    }));
    assert_eq!(report.outcomes().count(), 0);
    let metrics = report.metrics();
    assert_eq!(metrics.specifications(), 2);
    assert_eq!(metrics.succeeded(), 0);
    assert_eq!(metrics.failed(), 2);
    assert_eq!(metrics.built(), 0);
    assert_eq!(metrics.reused(), 0);
    assert!(!target.exists());
    std::fs::remove_dir_all(root).expect("remove indexed failure fixture");
}

#[cfg(unix)]
#[test]
fn batch_failure_retains_partial_metadata_timing() {
    let root = unique_temp_directory("wasm-batch-failure-timings");
    let cargo = root.join("cargo.sh");
    write_executable_script(
        &cargo,
        b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'cargo 1.0.0'; exit 0; fi\nsleep 0.02\necho 'synthetic metadata failure' >&2\nexit 23\n",
    )
    .expect("write executable test script");
    let specs = [LabeledWasmBuildSpec::new(
        "metadata-failure",
        WasmBuildSpec::new(
            &root,
            &root.join("target"),
            &["fixture"],
            crate::cargo_fixture::environment(),
        )
        .with_cargo_program(&cargo),
    )];

    let report =
        build_wasm_canisters_cached_batch(&specs, WasmBuildBatchConfig::new(), None, spawn_cargo)
            .expect("valid labeled batch");
    let failure = report.failures().next().expect("metadata failure");

    assert_eq!(
        failure.failure_details().unwrap().phase(),
        WasmBuildPhase::CargoMetadata
    );
    assert!(
        failure
            .failure_details()
            .unwrap()
            .timings()
            .input_resolution()
            .tool_identity()
            > Duration::ZERO
    );
    assert!(
        failure
            .failure_details()
            .unwrap()
            .timings()
            .input_resolution()
            .cargo_metadata()
            > Duration::ZERO
    );
    assert_eq!(
        failure.failure_details().unwrap().timings().cargo_build(),
        None
    );
    assert!(failure.failure_details().unwrap().timings().total() <= failure.entry_elapsed());

    std::fs::remove_dir_all(root).expect("remove failure timing fixture");
}

#[cfg(unix)]
#[test]
fn malformed_shared_metadata_is_reported_for_every_entry() {
    let (root, spec) = fake_wasm_build_spec("batch-malformed-metadata");
    let cargo = root.join("cargo.sh");
    write_executable_script(
        &cargo,
        b"#!/bin/sh\ncase \"$1\" in\n--version) echo 'cargo 1.0.0' ;;\nmetadata) echo '{}' ;;\n*) exit 97 ;;\nesac\n",
    )
    .expect("write executable test script");
    let specs = [
        LabeledWasmBuildSpec::new("first", spec.clone()),
        LabeledWasmBuildSpec::new("second", spec),
    ];
    let report = build_wasm_canisters_cached_batch(
        &specs,
        WasmBuildBatchConfig::new(),
        Some((WasmBuildProgressConfig::new(), &mut |_| {})),
        spawn_cargo,
    )
    .expect("valid labeled batch");

    assert_eq!(report.outcomes().count(), 0);
    let failures = report.failures().collect::<Vec<_>>();
    assert_eq!(failures.len(), 2);
    for (index, failure) in failures.iter().enumerate() {
        assert_eq!(failure.index(), index);
        assert_eq!(failure.label(), specs[index].label());
        assert_eq!(
            failure.failure_details().unwrap().phase(),
            WasmBuildPhase::CargoMetadata
        );
        assert!(matches!(
            failure.error().unwrap(),
            WasmBuildError::InvalidMetadata { .. }
        ));
        assert_eq!(
            failure.failure_details().unwrap().timings().cargo_build(),
            None
        );
    }
    std::fs::remove_dir_all(root).expect("remove malformed metadata fixture");
}

#[test]
fn batch_rejects_invalid_labels_before_progress_or_build_work() {
    let invalid = WasmBuildSpec::new(
        Path::new("."),
        Path::new("target"),
        &[],
        crate::cargo_fixture::environment(),
    );
    let empty = [LabeledWasmBuildSpec::new("", invalid.clone())];
    let mut progress_events = 0;
    let empty_error = build_wasm_canisters_cached_batch(
        &empty,
        WasmBuildBatchConfig::new(),
        Some((WasmBuildProgressConfig::new(), &mut |_| {
            progress_events += 1;
        })),
        spawn_cargo,
    )
    .expect_err("empty label must reject the batch");
    assert_eq!(
        empty_error,
        WasmBuildBatchContractError::Labels(BatchLabelError::Empty { index: 0 })
    );
    assert_eq!(progress_events, 0);
    assert_eq!(
        std::error::Error::source(&empty_error)
            .unwrap()
            .downcast_ref::<BatchLabelError>(),
        Some(&BatchLabelError::Empty { index: 0 })
    );

    let duplicate = [
        LabeledWasmBuildSpec::new("same", invalid.clone()),
        LabeledWasmBuildSpec::new("same", invalid),
    ];
    assert_eq!(
        build_wasm_canisters_cached_batch(
            &duplicate,
            WasmBuildBatchConfig::new(),
            None,
            spawn_cargo,
        )
        .expect_err("duplicate labels must reject the batch"),
        WasmBuildBatchContractError::Labels(BatchLabelError::Duplicate {
            label: "same".to_owned(),
            first_index: 0,
            duplicate_index: 1,
        })
    );
}

#[test]
fn batch_progress_retains_the_caller_label() {
    let specs = [LabeledWasmBuildSpec::new(
        "root",
        WasmBuildSpec::new(
            Path::new("."),
            Path::new("target"),
            &[],
            crate::cargo_fixture::environment(),
        ),
    )];
    let mut labels = Vec::new();

    let report = build_wasm_canisters_cached_batch(
        &specs,
        WasmBuildBatchConfig::new(),
        Some((WasmBuildProgressConfig::new(), &mut |event| match event {
            WasmBuildBatchProgressEvent::BuildStarted { label, .. }
            | WasmBuildBatchProgressEvent::BuildProgress { label, .. }
            | WasmBuildBatchProgressEvent::BuildFinished { label, .. }
            | WasmBuildBatchProgressEvent::BuildFailed { label, .. } => labels.push(label),
        })),
        spawn_cargo,
    )
    .expect("valid progress batch");

    assert!(!report.is_success());
    assert_eq!(labels, ["root", "root"]);
}

#[test]
fn batch_maintenance_configures_each_shared_target_once() {
    let maintenance = SharedIncrementalTargetMaintenanceConfig::new(
        CacheRetentionLimits::new().with_max_size_bytes(1024),
        Duration::from_secs(60),
    )
    .with_failure_mode(SharedIncrementalTargetMaintenanceFailureMode::BestEffort);
    let batch = WasmBuildBatchConfig::new().with_shared_incremental_target_maintenance(maintenance);
    let mut tracker = BatchMaintenanceTracker::new(batch.shared_incremental_target_maintenance());
    let first = WasmBuildSpec::new(
        Path::new("."),
        Path::new("exact-a"),
        &["a"],
        crate::cargo_fixture::environment(),
    )
    .with_shared_incremental_target("shared-a");
    let second = WasmBuildSpec::new(
        Path::new("."),
        Path::new("exact-b"),
        &["b"],
        crate::cargo_fixture::environment(),
    )
    .with_shared_incremental_target("shared-a");
    let other = WasmBuildSpec::new(
        Path::new("."),
        Path::new("exact-c"),
        &["c"],
        crate::cargo_fixture::environment(),
    )
    .with_shared_incremental_target("shared-b");
    let isolated = WasmBuildSpec::new(
        Path::new("."),
        Path::new("exact-d"),
        &["d"],
        crate::cargo_fixture::environment(),
    )
    .with_isolated_builds();

    let prepared_first = tracker
        .prepare_spec(&first)
        .expect("resolve shared target")
        .expect("first shared target must own maintenance");
    assert_eq!(
        prepared_first.shared_incremental_target_maintenance(),
        Some(maintenance)
    );
    assert!(tracker.prepare_spec(&second).unwrap().is_none());
    assert!(tracker.prepare_spec(&other).unwrap().is_some());
    assert!(tracker.prepare_spec(&isolated).unwrap().is_none());
}

#[cfg(unix)]
#[test]
fn batch_maintenance_distinguishes_workspaces_and_deduplicates_aliases() {
    let (first_root, first) = fake_wasm_build_spec("batch-maintenance-first");
    let (second_root, second) = fake_wasm_build_spec("batch-maintenance-second");
    let alias_root = unique_temp_directory("batch-maintenance-alias");
    symlink(&first_root, alias_root.join("alias")).expect("create workspace alias");
    let first = first.with_shared_incremental_target("shared");
    let second = second.with_shared_incremental_target("shared");
    let alias = first
        .clone()
        .with_shared_incremental_target(alias_root.join("alias/shared"));
    let specs = [
        LabeledWasmBuildSpec::new("first", first),
        LabeledWasmBuildSpec::new("second", second),
        LabeledWasmBuildSpec::new("alias", alias),
    ];
    let config = WasmBuildBatchConfig::new()
        .with_shared_incremental_target_maintenance_at_most_every(
            CacheRetentionLimits::new(),
            Duration::ZERO,
        );
    let report = super::build_wasm_canisters_cached_batch(&specs, config, None, spawn_cargo)
        .expect("valid maintenance batch");
    assert!(report.is_success(), "{report:?}");
    assert_eq!(report.shared_incremental_maintenance_outcomes().count(), 2);
    assert!(
        report
            .shared_incremental_maintenance_outcomes()
            .all(|(_, outcome)| {
                matches!(
                    outcome,
                    SharedIncrementalTargetMaintenanceOutcome::Performed { .. }
                )
            })
    );
    for root in [&first_root, &second_root] {
        assert!(
            root.join("shared/.ic-testkit/.ic-testkit-last-maintenance")
                .is_file()
        );
    }
    assert!(
        report
            .outcomes()
            .last()
            .is_some_and(|entry| entry.outcome().unwrap().is_reused())
    );
    drop(report);
    fs::remove_dir_all(first_root).expect("remove first maintenance workspace");
    fs::remove_dir_all(second_root).expect("remove second maintenance workspace");
    fs::remove_dir_all(alias_root).expect("remove maintenance alias");
}

#[test]
fn batch_maintenance_rejects_per_spec_policy_ownership() {
    let maintenance = SharedIncrementalTargetMaintenanceConfig::new(
        CacheRetentionLimits::new(),
        Duration::from_secs(60),
    );
    let spec = LabeledWasmBuildSpec::new(
        "fixture",
        WasmBuildSpec::new(
            Path::new("."),
            Path::new("exact"),
            &["fixture"],
            crate::cargo_fixture::environment(),
        )
        .with_shared_incremental_target("shared")
        .with_shared_incremental_target_maintenance(maintenance),
    );
    let batch = WasmBuildBatchConfig::new().with_shared_incremental_target_maintenance(maintenance);

    let report = build_wasm_canisters_cached_batch(&[spec], batch, None, spawn_cargo)
        .expect("valid labeled batch");
    let failures = report.failures().collect::<Vec<_>>();
    assert_eq!(report.entries().len(), 1);
    assert_eq!(failures.len(), 1);
    let failure = failures[0];
    assert_eq!(failure.index(), 0);
    assert_eq!(failure.label(), "fixture");
    assert_eq!(failure.entry_elapsed(), report.entries()[0].entry_elapsed());
    assert_eq!(
        failure.failure_details().unwrap().phase(),
        WasmBuildPhase::Specification
    );
    assert!(
        matches!(failure.error().unwrap(), WasmBuildError::InvalidSpec { message } if message.contains("cannot be combined"))
    );
}

#[cfg(unix)]
#[test]
fn unsafe_shared_target_does_not_fail_other_compatible_batch_entries() {
    let (root, spec) = fake_wasm_build_spec("batch-source-boundary");
    let unsafe_target = root.join("fixture/src/generated-target");
    fs::create_dir_all(&unsafe_target).expect("create unsafe shared target");
    let sentinel = unsafe_target.join("source-sentinel");
    fs::write(&sentinel, b"preserve").expect("write source sentinel");
    let specs = [
        LabeledWasmBuildSpec::new(
            "unsafe",
            spec.clone().with_shared_incremental_target(&unsafe_target),
        ),
        LabeledWasmBuildSpec::new("safe", spec),
    ];
    let config = WasmBuildBatchConfig::new()
        .with_shared_incremental_target_maintenance_at_most_every(
            CacheRetentionLimits::new().with_max_size_bytes(0),
            Duration::ZERO,
        );

    let report = build_wasm_canisters_cached_batch(&specs, config, None, spawn_cargo)
        .expect("valid labeled batch");
    assert_eq!(report.outcomes().count(), 1);
    let failures = report.failures().collect::<Vec<_>>();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].label(), "unsafe");
    assert_eq!(
        failures[0].failure_details().unwrap().phase(),
        WasmBuildPhase::InputDiscovery
    );
    assert!(matches!(
        failures[0].error().unwrap(),
        WasmBuildError::InvalidSpec { .. }
    ));
    assert_eq!(
        failures[0]
            .failure_details()
            .unwrap()
            .timings()
            .cargo_build(),
        None
    );
    assert_eq!(
        fs::read(&sentinel).expect("read preserved source"),
        b"preserve"
    );
    drop(report);
    fs::remove_dir_all(root).expect("remove batch source boundary fixture");
}

#[cfg(unix)]
#[test]
fn a_missing_input_does_not_fail_other_compatible_batch_entries() {
    for invalid_first in [true, false] {
        for observed in [true, false] {
            let (root, spec) = fake_wasm_build_spec("batch-hash-isolation");
            let good = LabeledWasmBuildSpec::new("good", spec.clone());
            let bad =
                LabeledWasmBuildSpec::new("bad", spec.with_additional_inputs(["missing-input"]));
            let specs = if invalid_first {
                [bad, good]
            } else {
                [good, bad]
            };
            let report = if observed {
                build_wasm_canisters_cached_batch(
                    &specs,
                    WasmBuildBatchConfig::new(),
                    Some((WasmBuildProgressConfig::new(), &mut |_| {})),
                    spawn_cargo,
                )
            } else {
                build_wasm_canisters_cached_batch(
                    &specs,
                    WasmBuildBatchConfig::new(),
                    None,
                    spawn_cargo,
                )
            }
            .expect("valid batch labels");
            assert_eq!(report.outcomes().count(), 1);
            let failures = report.failures().collect::<Vec<_>>();
            assert_eq!(failures.len(), 1);
            assert_eq!(failures[0].label(), "bad");
            assert_eq!(
                failures[0].failure_details().unwrap().phase(),
                WasmBuildPhase::ContentHashing
            );
            for entry in report.into_entries() {
                let failed = entry.label() == "bad";
                assert_eq!(entry.result().is_err(), failed);
                assert_eq!(entry.failure_details().is_some(), failed);
                let (_, label, result, details, _) = entry.into_parts();
                assert_eq!(result.is_err(), label == "bad");
                if let Err(error) = result {
                    let details = details.unwrap();
                    assert!(matches!(error, WasmBuildError::Io { .. }));
                    assert_eq!(details.phase(), WasmBuildPhase::ContentHashing);
                }
            }
            std::fs::remove_dir_all(root).expect("remove batch hashing fixture");
        }
    }
}

#[cfg(unix)]
#[test]
fn missing_tools_keep_their_phase_in_silent_and_observed_batches() {
    let (root, spec) = fake_wasm_build_spec("canonical-tool-failure-phases");
    let missing = root.join("missing-tool");
    for (spec, expected) in [
        (
            spec.clone().with_cargo_program(&missing),
            WasmBuildPhase::CargoIdentity,
        ),
        (
            spec.with_environment(crate::cargo_fixture::environment_with([(
                "RUSTC",
                missing.as_os_str(),
            )])),
            WasmBuildPhase::RustcIdentity,
        ),
    ] {
        for observed in [false, true] {
            let specs = [LabeledWasmBuildSpec::new("tool-failure", spec.clone())];
            let mut events = Vec::new();
            let mut observe = |event| events.push(event);
            let progress = observed.then_some((
                WasmBuildProgressConfig::new().without_heartbeats(),
                &mut observe as &mut dyn FnMut(_),
            ));
            let report = build_wasm_canisters_cached_batch(
                &specs,
                WasmBuildBatchConfig::new(),
                progress,
                spawn_cargo,
            )
            .unwrap();
            let entry = &report.entries()[0];
            assert_eq!(entry.failure_details().unwrap().phase(), expected);
            assert!(
                matches!(entry.error(), Some(WasmBuildError::CommandSpawn { phase, .. }) if *phase == expected)
            );
            assert_eq!(events.is_empty(), !observed);
            assert_eq!(report.outcomes().count(), 0);
        }
    }
    fs::remove_dir_all(root).unwrap();
}
