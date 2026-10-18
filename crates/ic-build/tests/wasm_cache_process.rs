//! Cross-process Wasm cache and shared incremental target qualification.

#![cfg(unix)]

mod support;

use ic_host_process::child::OwnedChild;
use ic_host_process::testing::{
    crash_barrier::{wait_for_child_path, wait_for_path},
    write_executable_script,
};

use ic_build::artifacts::{
    CacheRetentionLimits, SharedIncrementalTargetMaintenanceOutcome, WasmBuildOutcome,
    WasmBuildSpec, build_wasm_canisters_cached, maintain_shared_incremental_target_at_most_every,
};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use support::fs_fixture::unique_temp_directory;

const WORKER_ROOT_ENV: &str = "IC_TESTKIT_WASM_PROCESS_ROOT";
const WORKER_ID_ENV: &str = "IC_TESTKIT_WASM_PROCESS_WORKER";

#[test]
fn different_cache_roots_coordinate_one_shared_incremental_target_across_processes() {
    let root = unique_temp_directory("wasm-shared-process-lock");
    support::wasm::write_workspace(&root.join("workspace")).expect("prepare process workspace");
    write_cargo_wrapper(&root.join("cargo-wrapper.sh"));
    let executable = std::env::current_exe().expect("resolve current process-test executable");
    let mut first = spawn_worker(&executable, &root, "first");
    let mut second = spawn_worker(&executable, &root, "second");

    wait_for_child_path(
        &mut first,
        &root.join("ready-first"),
        "first worker ready",
        Duration::from_secs(10),
    )
    .expect("wait for first worker");
    wait_for_child_path(
        &mut second,
        &root.join("ready-second"),
        "second worker ready",
        Duration::from_secs(10),
    )
    .expect("wait for second worker");
    fs::write(root.join("go"), b"go").expect("release shared-target workers");

    assert!(first.wait().expect("wait for first worker").success());
    assert!(second.wait().expect("wait for second worker").success());
    assert!(
        !root.join("overlap").exists(),
        "Cargo builds using one shared target must not overlap"
    );
    assert_eq!(
        fs::read_to_string(root.join("builds"))
            .expect("read shared-target build log")
            .lines()
            .count(),
        2,
        "different exact caches should each build once"
    );
    fs::remove_dir_all(root).expect("remove shared-target process fixture");
}

#[test]
fn scheduled_shared_target_maintenance_runs_once_across_processes() {
    let root = unique_temp_directory("wasm-shared-scheduled-maintenance");
    support::wasm::write_workspace(&root.join("workspace")).expect("prepare maintenance workspace");
    fs::create_dir_all(root.join("shared-target/debug/deps"))
        .expect("create shared target for scheduled process test");
    fs::write(root.join("shared-target/debug/deps/state"), b"state")
        .expect("write shared target state for scheduled process test");
    let executable = std::env::current_exe().expect("resolve current process-test executable");
    let mut first = spawn_scheduled_maintenance_worker(&executable, &root, "first");
    let mut second = spawn_scheduled_maintenance_worker(&executable, &root, "second");

    wait_for_child_path(
        &mut first,
        &root.join("ready-first"),
        "first worker ready",
        Duration::from_secs(10),
    )
    .expect("wait for first worker");
    wait_for_child_path(
        &mut second,
        &root.join("ready-second"),
        "second worker ready",
        Duration::from_secs(10),
    )
    .expect("wait for second worker");
    fs::write(root.join("go"), b"go").expect("release scheduled-maintenance workers");

    assert!(first.wait().expect("wait for first worker").success());
    assert!(second.wait().expect("wait for second worker").success());
    let mut outcomes = [
        fs::read_to_string(root.join("maintenance-first")).expect("read first maintenance outcome"),
        fs::read_to_string(root.join("maintenance-second"))
            .expect("read second maintenance outcome"),
    ];
    outcomes.sort();
    assert_eq!(outcomes, ["performed", "skipped"]);
    fs::remove_dir_all(root).expect("remove scheduled-maintenance process fixture");
}

#[test]
#[ignore = "subprocess worker selected explicitly by the parent shared-target test"]
fn shared_incremental_process_worker() {
    let root = PathBuf::from(
        std::env::var_os(WORKER_ROOT_ENV).expect("worker root environment must be set"),
    );
    let worker = std::env::var(WORKER_ID_ENV).expect("worker identity environment must be set");
    fs::write(root.join(format!("ready-{worker}")), b"ready").expect("mark worker ready");
    wait_for_path(
        &root.join("go"),
        "parent start marker",
        Duration::from_secs(10),
    )
    .expect("wait for parent start");

    let workspace = root.join("workspace");
    let real_cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    let spec = WasmBuildSpec::new(
        &workspace,
        &root.join(format!("cache-{worker}")),
        &["fixture"],
        support::cargo::environment(),
    )
    .with_shared_incremental_target(root.join("shared-target"))
    .with_cargo_program(root.join("cargo-wrapper.sh"))
    .with_environment(support::cargo::environment_with([
        (OsString::from("REAL_CARGO"), real_cargo),
        (
            OsString::from("IC_TESTKIT_ACTIVE_BUILD"),
            root.join("active-build").into_os_string(),
        ),
        (
            OsString::from("IC_TESTKIT_OVERLAP_FILE"),
            root.join("overlap").into_os_string(),
        ),
        (
            OsString::from("IC_TESTKIT_BUILD_LOG"),
            root.join("builds").into_os_string(),
        ),
        (
            OsString::from("IC_TESTKIT_WORKER_VARIANT"),
            OsString::from(worker),
        ),
    ]));

    let outcome = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("run shared-target worker build");
    assert!(matches!(outcome, WasmBuildOutcome::Built(_)));
}

#[test]
#[ignore = "subprocess worker selected explicitly by the parent maintenance test"]
fn scheduled_shared_target_maintenance_process_worker() {
    let root = PathBuf::from(
        std::env::var_os(WORKER_ROOT_ENV).expect("worker root environment must be set"),
    );
    let worker = std::env::var(WORKER_ID_ENV).expect("worker identity environment must be set");
    fs::write(root.join(format!("ready-{worker}")), b"ready").expect("mark worker ready");
    wait_for_path(
        &root.join("go"),
        "parent start marker",
        Duration::from_secs(10),
    )
    .expect("wait for parent start");

    let workspace = root.join("workspace");
    let spec = WasmBuildSpec::new(
        &workspace,
        &root.join(format!("cache-{worker}")),
        &["fixture"],
        support::cargo::environment(),
    )
    .with_shared_incremental_target(root.join("shared-target"));
    let outcome = maintain_shared_incremental_target_at_most_every(
        &spec,
        CacheRetentionLimits::new().with_max_size_bytes(u64::MAX),
        Duration::from_hours(1),
    )
    .expect("run scheduled shared-target maintenance worker");
    let label = match outcome {
        SharedIncrementalTargetMaintenanceOutcome::Performed { .. } => "performed",
        SharedIncrementalTargetMaintenanceOutcome::Skipped { .. } => "skipped",
        SharedIncrementalTargetMaintenanceOutcome::Missing { .. } => {
            panic!("scheduled worker target must exist")
        }
        _ => panic!("unknown scheduled maintenance outcome"),
    };
    fs::write(root.join(format!("maintenance-{worker}")), label)
        .expect("write scheduled maintenance outcome");
}

fn write_cargo_wrapper(path: &Path) {
    write_executable_script(
        path,
        b"#!/bin/sh\n\
if [ \"$1\" = \"build\" ]; then\n\
  if mkdir \"$IC_TESTKIT_ACTIVE_BUILD\" 2>/dev/null; then\n\
    printf '%s\\n' \"$IC_TESTKIT_WORKER_VARIANT\" >> \"$IC_TESTKIT_BUILD_LOG\"\n\
    sleep 0.25\n\
    \"$REAL_CARGO\" \"$@\"\n\
    status=$?\n\
    rmdir \"$IC_TESTKIT_ACTIVE_BUILD\"\n\
    exit $status\n\
  fi\n\
  : > \"$IC_TESTKIT_OVERLAP_FILE\"\n\
fi\n\
exec \"$REAL_CARGO\" \"$@\"\n",
    )
    .expect("write executable test script");
}

fn spawn_worker(executable: &Path, root: &Path, worker: &str) -> std::process::Child {
    spawn_test_worker(
        executable,
        root,
        worker,
        "shared_incremental_process_worker",
    )
}

fn spawn_scheduled_maintenance_worker(
    executable: &Path,
    root: &Path,
    worker: &str,
) -> std::process::Child {
    spawn_test_worker(
        executable,
        root,
        worker,
        "scheduled_shared_target_maintenance_process_worker",
    )
}

fn spawn_test_worker(
    executable: &Path,
    root: &Path,
    worker: &str,
    test_name: &str,
) -> std::process::Child {
    Command::new(executable)
        .args(["--ignored", "--exact", test_name])
        .env(WORKER_ROOT_ENV, root)
        .env(WORKER_ID_ENV, worker)
        .spawn()
        .expect("spawn shared-target process worker")
}
