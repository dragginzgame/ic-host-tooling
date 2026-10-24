use super::resolution::{
    append_cargo_configuration_inputs, locked_package_identities, semantic_package_identity,
};
use crate::cargo::selection::{CargoFeatureSelection, CargoSelection};
use crate::cargo_fixture::spawn_cargo;
use ic_host_fs::owned_directory::OwnedDirectory;
use ic_host_fs::read::directory_logical_size;
use ic_host_process::child::OwnedChild;
#[cfg(unix)]
use ic_host_process::testing::write_executable_script;
use ic_host_process::tool::OutputStream;

use super::{
    ProgressReporter, SharedIncrementalTargetMaintenanceConfig,
    SharedIncrementalTargetMaintenanceFailureMode, SharedIncrementalTargetMaintenanceOutcome,
    WasmBuildBatchInputResolver, WasmBuildError, WasmBuildOutcome, WasmBuildPhase,
    WasmBuildProgressConfig, WasmBuildProgressEvent, WasmBuildSpec, artifact_stamp_path,
    build_wasm_canisters_cached, ensure_cache_directory_tag, finish_fingerprint_build,
    inspect_shared_incremental_target, integrated_shared_maintenance_result, lock_wasm_build_cache,
    lock_wasm_build_cache_with_progress, maintain_shared_incremental_target,
    maintain_shared_incremental_target_at_most_every, materialize_artifacts,
    perform_configured_shared_incremental_target_maintenance, prune_wasm_build_cache,
    prune_wasm_build_cache_locked, publish_artifact_stamps, resolve_cargo_build_inputs,
    run_cargo_build, validate_spec, validated_artifact_set,
};
use crate::artifacts::cache_fs::{CacheRetentionLimits, write_last_used};
use crate::artifacts::{WasmBuildInputSnapshot, digest::digest_bytes};
use crate::fs_fixture::unique_temp_directory;
use std::{
    collections::BTreeSet,
    ffi::OsStr,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use crate::artifacts::test_support::fake_wasm_build_spec;
#[cfg(unix)]
use std::{
    fs::FileTimes,
    os::unix::fs::{MetadataExt as _, PermissionsExt as _, symlink},
};

fn artifact_pairs(paths: &[PathBuf]) -> Vec<(PathBuf, PathBuf)> {
    paths
        .iter()
        .map(|path| (path.clone(), path.clone()))
        .collect()
}

fn canonical_fixture(path: &Path) -> PathBuf {
    path.canonicalize().expect("canonicalize test fixture path")
}

#[cfg(unix)]
#[test]
fn outcome_facts_are_retained_without_additional_identity_probes() {
    let (root, spec) = fake_wasm_build_spec("outcome-facts");
    let cargo = root.join("cargo.sh");
    let log = root.join("cargo-invocations");
    let script = fs::read_to_string(&cargo).unwrap().replace(
        "set -eu\n",
        &format!("set -eu\nprintf '%s\\n' \"$1\" >> '{}'\n", log.display()),
    );
    write_executable_script(&cargo, script).unwrap();
    let cold = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
    let warm = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
    assert!(!cold.is_reused());
    assert!(warm.is_reused());
    assert_eq!(cold.record().facts(), warm.record().facts());
    let before = fs::read(&log).unwrap();
    fs::remove_file(cargo).unwrap();
    let cloned = warm.record().clone();
    assert_eq!(cloned.facts(), cold.record().facts());
    assert_eq!(fs::read(&log).unwrap(), before);
    let calls = String::from_utf8(before).unwrap();
    assert_eq!(calls.lines().filter(|line| *line == "--version").count(), 4);
    assert_eq!(calls.lines().filter(|line| *line == "metadata").count(), 4);
    assert_eq!(calls.lines().filter(|line| *line == "build").count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn caller_selected_environment_reaches_build_and_preserves_cache_identity() {
    let (root, spec) = fake_wasm_build_spec("merged-environment");
    let wrapper = root.join("cargo.sh");
    let producer = root.join("producer.sh");
    fs::rename(&wrapper, &producer).unwrap();
    let quoted_producer = format!("'{}'", producer.to_string_lossy().replace('\'', "'\"'\"'"));
    write_executable_script(
        &wrapper,
        format!(
            r#"#!/bin/sh
set -eu
if [ "$1" = build ]; then
    [ "$LEFT" = 'left value' ]
    [ "$RIGHT" = 'right value' ]
    [ "$DUP" = 'last value' ]
    printf '%s\n' "$LEFT" "$RIGHT" "$DUP" > merged-environment
fi
exec {quoted_producer} "$@"
"#
        ),
    )
    .expect("write environment-checking Cargo wrapper");
    let mut selected =
        crate::cargo_fixture::environment_with([("LEFT", "left value"), ("DUP", "first value")]);
    selected.extend([
        ("RIGHT".into(), "right value".into()),
        ("DUP".into(), "last value".into()),
    ]);
    let chained = spec.clone().with_environment(selected);
    let first = build_wasm_canisters_cached(&chained, OwnedChild::spawn_direct).unwrap();
    assert!(matches!(first, WasmBuildOutcome::Built { .. }));
    assert_eq!(
        fs::read_to_string(root.join("merged-environment")).unwrap(),
        "left value\nright value\nlast value\n"
    );
    let combined = spec.with_environment(crate::cargo_fixture::environment_with([
        ("LEFT", "left value"),
        ("RIGHT", "right value"),
        ("DUP", "last value"),
    ]));
    let reused = build_wasm_canisters_cached(&combined, OwnedChild::spawn_direct).unwrap();
    assert!(matches!(reused, WasmBuildOutcome::Reused { .. }));
    assert_eq!(first.record().fingerprint(), reused.record().fingerprint());
    assert_eq!(first.record().facts(), reused.record().facts());
    assert_eq!(first.record().facts().environment, combined.environment);
    let changed = resolve_cargo_build_inputs(&chained.with_environment(
        crate::cargo_fixture::environment_with([("LEFT", "changed")]),
    ))
    .expect("resolve changed environment identity");
    assert_ne!(first.record().fingerprint(), changed.fingerprint());
    assert_eq!(first.record().facts().selection, combined.cargo_selection);
    assert_eq!(first.record().facts().target, combined.target);
    drop((first, reused));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn shim_compiler_identity_invalidates_exact_cache_and_explicit_rustc_takes_precedence() {
    let (root, spec) = fake_wasm_build_spec("shim-compiler-identity");
    let compiler = root.join("selected-rustc");
    write_executable_script(&compiler, "#!/bin/sh\nprintf 'selected-compiler-A\\n'\n")
        .expect("write executable test script");
    // The validation adapter may select RUSTC. This case exercises the identity
    // shim; the explicit-RUSTC precedence case below selects it separately.
    let mut environment = spec.environment().clone();
    environment.remove(OsStr::new("RUSTC"));
    let spec = spec
        .with_environment(environment)
        .with_rustc_identity_program(compiler.as_os_str());
    let first = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
    assert!(matches!(first, WasmBuildOutcome::Built { .. }));
    assert!(matches!(
        build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap(),
        WasmBuildOutcome::Reused { .. }
    ));
    write_executable_script(&compiler, "#!/bin/sh\nprintf 'selected-compiler-B\\n'\n")
        .expect("write executable test script");
    let second = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
    assert!(matches!(second, WasmBuildOutcome::Built { .. }));
    assert_ne!(first.record().fingerprint(), second.record().fingerprint());
    let selected_rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let mut selected = crate::cargo_fixture::environment_with([(
        "UNRELATED_OVERRIDE",
        "retained compiler selection",
    )]);
    selected.insert("RUSTC".into(), selected_rustc);
    let override_spec = spec
        .with_rustc_identity_program("/missing/identity-probe")
        .with_environment(selected);
    build_wasm_canisters_cached(&override_spec, OwnedChild::spawn_direct)
        .expect("explicit RUSTC selects metadata/build environment and identity probe");
    drop((first, second));
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn default_builds_share_cargo_state_across_fingerprints() {
    let (root, fixture) = fake_wasm_build_spec("default-shared-cargo");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_cargo_program(fixture.cargo_program);
    let first = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
    assert!(
        first
            .record()
            .timings()
            .shared_incremental_lock_wait()
            .is_some()
    );
    let target = root.join("target/ic-testkit-incremental");
    fs::write(target.join("retained-cargo-state"), b"retained").unwrap();
    let second = build_wasm_canisters_cached(
        &spec.with_environment(crate::cargo_fixture::environment_with([(
            "BUILD_VARIANT",
            "second",
        )])),
        OwnedChild::spawn_direct,
    )
    .unwrap();
    assert_ne!(first.record().fingerprint(), second.record().fingerprint());
    assert!(
        second
            .record()
            .timings()
            .shared_incremental_lock_wait()
            .is_some()
    );
    assert_eq!(
        fs::read(target.join("retained-cargo-state")).unwrap(),
        b"retained"
    );
    drop((first, second));
    fs::remove_dir_all(root).unwrap();
}

fn write_projection_package(root: &Path, package: &str, manifest_suffix: &str) {
    let directory = root.join(package);
    fs::create_dir_all(directory.join("src")).expect("create projection fixture package");
    fs::write(
        directory.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{package}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n{manifest_suffix}"
        ),
    )
    .expect("write projection fixture package manifest");
    fs::write(
        directory.join("src/lib.rs"),
        format!("pub fn {package}() {{}}\n"),
    )
    .expect("write projection fixture source");
}

#[test]
fn workspace_selection_preserves_membership_ambiguity_and_requested_order() {
    let packages = [("a", "alpha"), ("z", "zeta"), ("other", "alpha")].map(|(id, name)| {
        serde_json::json!({
            "id": id, "name": name, "version": "1.0.0",
            "manifest_path": format!("/workspace/{id}/Cargo.toml"),
            "dependencies": [], "targets": [], "features": {},
        })
    });
    let original = serde_json::json!({
        "packages": packages,
        "workspace_members": ["a", "z", "a", "unknown-id"],
        "workspace_root": "/workspace", "target_directory": "/workspace/target",
        "resolve": {"nodes": [], "root": null}, "version": 1,
    });
    let mut spec = WasmBuildSpec::new(
        Path::new("/workspace"),
        Path::new("/target"),
        &[],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );
    // Exercise the selector independently of the builder's canonical ordering.
    spec.packages = vec!["zeta".into(), "alpha".into()];
    let canonical = serde_json::from_value(original.clone()).unwrap();
    let parsed = super::ParsedCargoMetadata::parse(&canonical).unwrap();
    assert_eq!(
        super::resolution::selected_package_ids(&spec, &parsed).unwrap(),
        ["z", "a"]
    );
    spec.packages = vec!["missing".into(), "alpha".into()];
    assert!(matches!(
        super::resolution::selected_package_ids(&spec, &parsed),
        Err(WasmBuildError::InvalidSpec { message })
            if message == "Cargo workspace contains no package named `missing`"
    ));

    let mut ambiguous = original;
    ambiguous["workspace_members"] = serde_json::json!(["a", "z", "other"]);
    let canonical = serde_json::from_value(ambiguous).unwrap();
    let parsed = super::ParsedCargoMetadata::parse(&canonical).unwrap();
    spec.packages = vec!["alpha".into()];
    assert!(matches!(
        super::resolution::selected_package_ids(&spec, &parsed),
        Err(WasmBuildError::InvalidSpec { message })
            if message == "Cargo workspace package name `alpha` is ambiguous"
    ));
    // An unused ambiguous name still permits selecting another workspace member.
    spec.packages = vec!["zeta".into()];
    assert_eq!(
        super::resolution::selected_package_ids(&spec, &parsed).unwrap(),
        ["z"]
    );
}

#[test]
#[cfg(unix)]
fn package_selection_agrees_across_identity_artifacts_and_cargo_invocation() {
    let root = unique_temp_directory("canonical-wasm-packages");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"alpha\", \"zeta\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    write_projection_package(&root, "alpha", "");
    write_projection_package(&root, "zeta", "");
    let target = root.join("exact");
    let ordered = WasmBuildSpec::new(
        &root,
        &target,
        &["alpha", "zeta"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );
    let redundant = WasmBuildSpec::new(
        &root,
        &target,
        &["zeta", "alpha", "zeta", "alpha"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );
    assert_eq!(redundant.packages(), ordered.packages());
    assert_eq!(redundant, ordered);

    let ordered_inputs = resolve_cargo_build_inputs(&ordered).unwrap();
    let redundant_inputs = resolve_cargo_build_inputs(&redundant).unwrap();
    assert_eq!(redundant_inputs.fingerprint(), ordered_inputs.fingerprint());
    assert_eq!(redundant_inputs.inputs(), ordered_inputs.inputs());
    assert_eq!(
        expected_artifacts(&redundant, &target),
        ["alpha.wasm", "zeta.wasm"]
            .map(|name| target.join("wasm32-unknown-unknown/debug").join(name)),
    );

    // Read a script through the shell, avoiding execution of a newly written
    // fixture. Capture the actual arguments in both command-output modes.
    fs::write(
        root.join("build"),
        "printf '%s\\n' \"$@\" > \"$IC_TESTKIT_PACKAGE_ARGUMENTS\"\n",
    )
    .unwrap();
    let arguments = root.join("arguments");
    let spec = redundant.with_cargo_program("/bin/sh").with_environment(
        crate::cargo_fixture::environment_with([(
            "IC_TESTKIT_PACKAGE_ARGUMENTS",
            arguments.as_os_str(),
        )]),
    );
    for observed in [false, true] {
        let mut observer = |_| {};
        let mut progress = if observed {
            ProgressReporter::new(Some((
                WasmBuildProgressConfig::new().without_heartbeats(),
                &mut observer,
            )))
            .unwrap()
        } else {
            ProgressReporter::silent()
        };
        run_cargo_build(
            &spec,
            &root.join("cargo-target"),
            &[],
            &mut progress,
            &mut (spawn_cargo),
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(&arguments).unwrap(),
            "--message-format=json\n--lib\n--target\nwasm32-unknown-unknown\n-p\nalpha\n-p\nzeta\n",
        );
        fs::remove_file(&arguments).unwrap();
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn wasm_stamps_require_exact_identity_and_unmodified_contents() {
    let root = unique_temp_directory("wasm-stamp-validation");
    let artifact = root.join("fixture.wasm");
    let artifacts = [artifact.clone()];
    let fingerprint = digest_bytes("stamp-fixture-v1", b"first");
    let other_fingerprint = digest_bytes("stamp-fixture-v1", b"second");
    let original = b"\0asm\x01\0\0\0";
    fs::write(&artifact, original).expect("write artifact");
    assert!(validated_artifact_set(artifact_pairs(&artifacts), fingerprint).is_none());
    publish_artifact_stamps(artifact_pairs(&artifacts), fingerprint).expect("publish stamp");
    let stamp_path = artifact_stamp_path(&artifact);
    let stamp = fs::read_to_string(&stamp_path).expect("read stamp");
    let artifact_digest = digest_bytes("wasm-artifact-v1", original);
    assert_eq!(
        stamp,
        format!(
            "ic-testkit-wasm-build-v1\nbuild-sha256:{fingerprint}\nartifact-sha256:{artifact_digest}\n"
        ),
    );
    assert!(validated_artifact_set(artifact_pairs(&artifacts), fingerprint).is_some());
    assert!(validated_artifact_set(artifact_pairs(&artifacts), other_fingerprint).is_none());

    // A correct build header still requires checking the artifact's bytes.
    fs::write(&artifact, b"\0asm\x02\0\0\0").expect("modify same-size artifact");
    assert!(validated_artifact_set(artifact_pairs(&artifacts), fingerprint).is_none());
    fs::write(&artifact, original).expect("restore artifact bytes");
    assert!(validated_artifact_set(artifact_pairs(&artifacts), fingerprint).is_some());

    for invalid in [
        stamp.replace("ic-testkit-wasm-build-v1", "invalid-format"),
        stamp.replace(&fingerprint.to_hex(), &other_fingerprint.to_hex()),
        stamp.replace("artifact-sha256:", "unknown-digest:"),
        format!("{stamp}extra\n"),
        stamp.trim_end().to_owned(),
    ] {
        fs::write(&stamp_path, invalid).expect("write invalid stamp");
        assert!(validated_artifact_set(artifact_pairs(&artifacts), fingerprint).is_none());
    }
    fs::write(&stamp_path, [0xff]).expect("write invalid UTF-8 stamp");
    assert!(validated_artifact_set(artifact_pairs(&artifacts), fingerprint).is_none());

    // A matching header must not turn an oversized sidecar into an unbounded read.
    fs::write(&stamp_path, &stamp).expect("write matching stamp prefix");
    fs::OpenOptions::new()
        .write(true)
        .open(&stamp_path)
        .expect("open oversized stamp")
        .set_len(1024 * 1024 * 1024)
        .expect("extend oversized stamp");
    assert!(validated_artifact_set(artifact_pairs(&artifacts), fingerprint).is_none());
    publish_artifact_stamps(artifact_pairs(&artifacts), fingerprint)
        .expect("replace oversized stamp");
    assert!(validated_artifact_set(artifact_pairs(&artifacts), fingerprint).is_some());
    for bytes in [[0; 32], [0xff; 32]] {
        let fingerprint = super::InputDigest::from_bytes(bytes);
        publish_artifact_stamps(artifact_pairs(&artifacts), fingerprint).unwrap();
        let stamp = fs::read(&stamp_path).unwrap();
        assert_eq!(stamp.len(), 184);
        assert!(validated_artifact_set(artifact_pairs(&artifacts), fingerprint).is_some());
        let mut oversized = stamp.clone();
        oversized.push(b'\n');
        fs::write(&stamp_path, oversized).unwrap();
        assert!(validated_artifact_set(artifact_pairs(&artifacts), fingerprint).is_none());
        fs::write(&stamp_path, &stamp[..stamp.len() - 1]).unwrap();
        assert!(validated_artifact_set(artifact_pairs(&artifacts), fingerprint).is_none());
    }
    fs::remove_dir_all(root).expect("remove stamp fixture");
}

#[test]
fn typed_features_watch_enabled_optional_dependencies() {
    let root = unique_temp_directory("compact-feature-inputs");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\", \"optional_dep\"]\nresolver = \"2\"\n",
    )
    .expect("write optional dependency workspace");
    write_projection_package(
        &root,
        "fixture",
        "[features]\nextra = [\"dep:optional_dep\"]\nmap = [\"dep:optional_dep\"]\n\
         [dependencies]\noptional_dep = { path = \"../optional_dep\", optional = true }\n",
    );
    write_projection_package(&root, "optional_dep", "");
    let dependency = root.join("optional_dep");
    let dependency_source = dependency.join("src/lib.rs");
    let base = WasmBuildSpec::new(
        &root,
        &root.join("target"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );

    for feature in ["extra", "map"] {
        fs::write(&dependency_source, "pub fn optional_dep() -> u8 { 1 }\n")
            .expect("reset optional dependency source");
        let spec = base.clone().with_cargo_selection(CargoSelection {
            features: CargoFeatureSelection {
                features: [feature.to_owned()].into(),
                ..Default::default()
            },
            ..Default::default()
        });
        validate_spec(&spec).expect("accept feature selection without input overrides");
        let snapshot = resolve_cargo_build_inputs(&spec).expect("resolve enabled dependency");
        assert!(
            snapshot
                .inputs()
                .iter()
                .any(|input| input.path() == canonical_fixture(&dependency)),
            "{feature:?} must include the enabled optional dependency",
        );
        assert!(
            snapshot
                .is_content_current()
                .expect("check unchanged inputs")
        );
        {
            let source_exclusion = std::sync::Mutex::new(());
            let guard = source_exclusion.lock().expect("lock source fixture");
            let prepared = WasmBuildInputSnapshot::prepare_assuming_sources_immutable(
                &guard,
                &[base.clone(), spec],
            )
            .expect("prepare distinct feature graphs");
            assert_eq!(prepared.metrics().input_resolution_runs(), 2);
            assert_eq!(prepared.metrics().input_resolution_reuses(), 0);
            drop(prepared);
            drop(guard);
        }
        fs::write(&dependency_source, "pub fn optional_dep() -> u8 { 2 }\n")
            .expect("change optional dependency source");
        assert!(
            !snapshot
                .is_content_current()
                .expect("detect dependency change"),
            "{feature:?} must watch the enabled optional dependency",
        );
    }
    fs::remove_dir_all(root).expect("remove compact feature fixture");
}

#[test]
fn builders_preserve_os_native_values() {
    let selected = [
        (OsString::from("MODE"), OsString::from("exact")),
        ("RUSTFLAGS".into(), "".into()),
    ];
    let spec = WasmBuildSpec::new(
        Path::new("."),
        Path::new("target"),
        &["fixture"],
        selected,
        CONFIGURATION_LIMITS,
    )
    .with_cargo_selection(CargoSelection {
        locked: true,
        ..Default::default()
    })
    .with_additional_inputs([PathBuf::from("schema")]);
    assert_eq!(
        spec.cargo_selection.build_arguments(),
        [OsString::from("--locked")]
    );
    assert_eq!(spec.environment()[OsStr::new("MODE")], "exact");
    assert_eq!(spec.environment()[OsStr::new("RUSTFLAGS")], "");
    assert!(!spec.environment().contains_key(OsStr::new("PATH")));
    assert_eq!(spec.additional_inputs, [PathBuf::from("schema")]);
    let replaced = spec.clone().with_environment([("ONLY", "selected")]);
    assert_eq!(replaced.environment().len(), 1);
    assert!(!replaced.environment().contains_key(OsStr::new("MODE")));
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt as _;
        let name = OsString::from_vec(b"NATIVE_\xff".to_vec());
        let value = OsString::from_vec(b"value_\xff".to_vec());
        let spec = spec.with_environment([(name.clone(), value.clone())]);
        assert_eq!(spec.environment()[&name], value);
    }
}

#[test]
fn cargo_target_overrides_are_rejected_before_acquisition() {
    let root = unique_temp_directory("cargo-target-overrides");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );
    let override_spec = spec.with_environment(crate::cargo_fixture::environment_with([(
        "CARGO_TARGET_DIR",
        "other",
    )]));
    assert!(matches!(
        build_wasm_canisters_cached(&override_spec, OwnedChild::spawn_direct),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    assert!(!root.join("exact").exists());
    fs::remove_dir_all(root).expect("remove Cargo target override fixture");
}

#[cfg(unix)]
#[test]
fn shared_target_boundaries_preserve_retained_exact_entries() {
    let (root, base) = fake_wasm_build_spec("shared-exact-boundary");
    let missing = root.join("missing-input");
    let generated = base
        .clone()
        .with_shared_incremental_target(base.target_dir.join("incremental"));
    assert!(matches!(
        super::resolution::validate_shared_incremental_target_boundary(
            &generated,
            &[("root".into(), root.clone()), ("missing".into(), missing.clone())],
        ),
        Err(WasmBuildError::Io { path, source, .. })
            if path == missing && source.kind() == std::io::ErrorKind::NotFound
    ));
    let unsafe_shared = root.join("shared");
    let mut nested = base.clone();
    nested.target_dir = unsafe_shared.join("exact");
    // A retained isolated build already exists before shared maintenance is enabled.
    let retained = build_wasm_canisters_cached(&nested, OwnedChild::spawn_direct)
        .expect("build retained exact artifact");
    let artifact = retained.record().artifacts()[0].clone();
    nested = nested.with_shared_incremental_target(&unsafe_shared);
    let policy = CacheRetentionLimits::new().with_max_size_bytes(0);
    assert!(matches!(
        build_wasm_canisters_cached(&nested, OwnedChild::spawn_direct),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    assert!(matches!(
        maintain_shared_incremental_target(&nested, policy),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    assert!(matches!(
        maintain_shared_incremental_target_at_most_every(&nested, policy, Duration::ZERO),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    assert!(artifact.is_file());

    let inside_entries = base.clone().with_shared_incremental_target(
        base.target_dir.join(".ic-testkit/wasm-targets/incremental"),
    );
    assert!(matches!(
        resolve_cargo_build_inputs(&inside_entries),
        Err(WasmBuildError::InvalidSpec { .. })
    ));

    // Sharing the exact target container remains safe: its metadata is preserved.
    let same_root = base
        .clone()
        .with_shared_incremental_target(&base.target_dir);
    let safe = build_wasm_canisters_cached(&same_root, OwnedChild::spawn_direct)
        .expect("build with common target container");
    maintain_shared_incremental_target(&same_root, policy)
        .expect("maintain common target container");
    assert!(safe.record().artifacts()[0].is_file());
    let sibling = base.with_shared_incremental_target(root.join("incremental"));
    resolve_cargo_build_inputs(&sibling).expect("separate target roots remain valid");
    drop(safe);
    drop(retained);
    fs::remove_dir_all(root).expect("remove shared boundary fixture");
}

#[cfg(unix)]
#[test]
fn relative_exact_target_is_shared_by_cargo_and_cache_operations() {
    let (root, mut spec) = fake_wasm_build_spec("relative-exact-target");
    let caller_dir = std::env::current_dir().expect("read caller working directory");
    spec.workspace_root = root.join("nested/workspace/one/two/three/four");
    fs::create_dir_all(&spec.workspace_root).expect("create separate Cargo working directory");
    fs::rename(
        root.join("Cargo.toml"),
        spec.workspace_root.join("Cargo.toml"),
    )
    .expect("move workspace manifest");
    fs::rename(root.join("fixture"), spec.workspace_root.join("fixture"))
        .expect("move workspace package");
    let wrapper = root.join("cargo.sh");
    let script = fs::read_to_string(&wrapper).unwrap();
    fs::write(
        &wrapper,
        script.replace(
            root.to_str().unwrap(),
            spec.workspace_root.to_str().unwrap(),
        ),
    )
    .unwrap();
    // Reach an owned /tmp target using a relative path without changing the
    // process-wide working directory or creating test output in the checkout.
    spec.target_dir = caller_dir
        .components()
        .filter(|part| matches!(part, std::path::Component::Normal(_)))
        .fold(PathBuf::new(), |path, _| path.join(".."))
        .join(root.strip_prefix("/").unwrap())
        .join("exact");
    let cold = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("build relative exact target");
    assert!(cold.record().artifacts()[0].is_file());
    assert!(root.join("exact/.ic-testkit/wasm-targets").is_dir());
    let warm = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("reuse relative exact target");
    assert!(warm.is_reused());
    drop(warm);
    drop(cold);
    fs::remove_dir_all(root).expect("remove relative exact target fixture");
}

#[test]
fn nested_workspace_targets_do_not_change_cargo_snapshots() {
    let root = unique_temp_directory("nested-workspace-targets");
    crate::cargo_fixture::write_nested_workspace_fixture(&root).unwrap();
    fs::write(
        root.join("adapter/embedded-consumer/Cargo.toml"),
        b"[workspace]\nmembers = []\n",
    )
    .unwrap();
    let spec = WasmBuildSpec::new(
        &root.join("driver"),
        &root.join("build-target"),
        &["driver"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );
    let before = resolve_cargo_build_inputs(&spec).unwrap();
    for workspace in ["consumer", "embedded-consumer"] {
        let target = root.join("adapter").join(workspace).join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("generated"), b"unrelated compiler output").unwrap();
        #[cfg(unix)]
        symlink("missing", target.join("dangling-artifact")).unwrap();
        assert!(before.is_content_current().unwrap());
        let after = resolve_cargo_build_inputs(&spec).unwrap();
        assert_eq!(before.fingerprint(), after.fingerprint());
        assert_eq!(before.validation_digest(), after.validation_digest());
        fs::write(target.join("generated"), b"changed compiler output").unwrap();
        assert!(before.is_content_current().unwrap());
        assert_eq!(
            before.fingerprint(),
            resolve_cargo_build_inputs(&spec).unwrap().fingerprint()
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn nested_workspace_source_and_explicit_input_changes_are_not_excluded() {
    let root = unique_temp_directory("nested-workspace-source");
    crate::cargo_fixture::write_nested_workspace_fixture(&root).unwrap();
    for (path, bytes) in [
        ("adapter/assets/target/source", "ordinary source"),
        ("adapter/consumer/Cargo.lock", "nested lock"),
        ("adapter/consumer/.cargo/config.toml", "[build]\n"),
        (
            "adapter/.canic/release-builds/artifacts/root.wasm",
            "consumer-owned state",
        ),
        (
            "adapter/decoy/Cargo.toml",
            "workspace = 'not a Cargo workspace'\n",
        ),
        ("adapter/decoy/target/source", "still source"),
        ("adapter/malformed/Cargo.toml", "not valid [workspace"),
        ("adapter/malformed/target/source", "still watched"),
    ] {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    let spec = WasmBuildSpec::new(
        &root.join("driver"),
        &root.join("build-target"),
        &["driver"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );
    let before = resolve_cargo_build_inputs(&spec).unwrap();
    for path in [
        "adapter/src/lib.rs",
        "adapter/Cargo.toml",
        "driver/Cargo.lock",
        "adapter/consumer/Cargo.toml",
        "adapter/consumer/Cargo.lock",
        "adapter/consumer/.cargo/config.toml",
        "adapter/assets/target/source",
        "adapter/.canic/release-builds/artifacts/root.wasm",
        "adapter/decoy/target/source",
        "adapter/malformed/target/source",
    ] {
        let path = root.join(path);
        let original = fs::read(&path).unwrap();
        let mut changed = original.clone();
        changed.extend_from_slice(b"\n# changed\n");
        fs::write(&path, changed).unwrap();
        assert!(
            !before.is_content_current().unwrap(),
            "must watch {}",
            path.display()
        );
        fs::write(&path, original).unwrap();
        assert!(before.is_content_current().unwrap());
    }
    // A file with this name cannot be a Cargo generated directory. Admission
    // must be re-derived during content guards, not retained after it changes.
    let target = root.join("adapter/consumer/target");
    fs::write(&target, b"source file named target").unwrap();
    assert!(!before.is_content_current().unwrap());
    let file_snapshot = resolve_cargo_build_inputs(&spec).unwrap();
    fs::write(&target, b"changed source file").unwrap();
    assert!(!file_snapshot.is_content_current().unwrap());
    fs::remove_file(&target).unwrap();
    fs::create_dir_all(&target).unwrap();
    let generated = target.join("schema");
    fs::write(&generated, b"explicit schema").unwrap();
    assert!(before.is_content_current().unwrap());
    let explicit =
        resolve_cargo_build_inputs(&spec.with_additional_inputs([generated.clone()])).unwrap();
    fs::write(generated, b"changed explicit schema").unwrap();
    assert!(
        !explicit.is_content_current().unwrap(),
        "declared generated inputs must remain watched"
    );
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn nested_workspace_target_aliases_remain_watched_with_source_aliases() {
    let root = unique_temp_directory("nested-workspace-aliases");
    crate::cargo_fixture::write_nested_workspace_fixture(&root).unwrap();
    let external = root.join("external");
    fs::create_dir_all(&external).unwrap();
    fs::write(external.join("source"), b"external source").unwrap();
    symlink(&external, root.join("adapter/source-alias")).unwrap();
    symlink(root.join("adapter"), root.join("adapter/cycle")).unwrap();
    let spec = WasmBuildSpec::new(
        &root.join("driver"),
        &root.join("build-target"),
        &["driver"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );
    let before = resolve_cargo_build_inputs(&spec).unwrap();
    fs::write(external.join("source"), b"changed external source").unwrap();
    assert!(!before.is_content_current().unwrap());
    fs::write(external.join("source"), b"external source").unwrap();
    assert!(before.is_content_current().unwrap());
    let target = root.join("adapter/consumer/target");
    for redirected in [root.join("adapter/src"), external] {
        symlink(redirected, &target).unwrap();
        assert!(!before.is_content_current().unwrap());
        let aliased = resolve_cargo_build_inputs(&spec).unwrap();
        fs::write(target.join("linked-change"), b"changed linked source").unwrap();
        assert!(!aliased.is_content_current().unwrap());
        fs::remove_file(target.join("linked-change")).unwrap();
        fs::remove_file(&target).unwrap();
        assert!(before.is_content_current().unwrap());
    }
    symlink(root.join("missing"), &target).unwrap();
    assert!(before.is_content_current().is_err());
    assert!(resolve_cargo_build_inputs(&spec).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn selected_cargo_sources_under_nested_targets_are_watched() {
    let root = unique_temp_directory("nested-workspace-selected-source");
    crate::cargo_fixture::write_nested_workspace_fixture(&root).unwrap();
    let manifest = root.join("adapter/Cargo.toml");
    let original = fs::read_to_string(&manifest).unwrap();
    fs::write(
        &manifest,
        format!("{original}\n[lib]\npath = 'consumer/target/selected.rs'\n"),
    )
    .unwrap();
    let source = root.join("adapter/consumer/target/selected.rs");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, b"pub fn selected() {}\n").unwrap();
    let spec = WasmBuildSpec::new(
        &root.join("driver"),
        &root.join("build-target"),
        &["driver"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );
    let snapshot = resolve_cargo_build_inputs(&spec).unwrap();
    fs::write(&source, b"pub fn changed() {}\n").unwrap();
    assert!(!snapshot.is_content_current().unwrap());
    assert_ne!(
        snapshot.fingerprint(),
        resolve_cargo_build_inputs(&spec).unwrap().fingerprint()
    );
    #[cfg(unix)]
    {
        // Cargo may spell a target source through a link outside the generated
        // tree. Admission must protect its physical source, not just its name.
        fs::write(&manifest, original).unwrap();
        fs::remove_file(root.join("adapter/src/lib.rs")).unwrap();
        symlink(&source, root.join("adapter/src/lib.rs")).unwrap();
        let snapshot = resolve_cargo_build_inputs(&spec).unwrap();
        fs::write(&source, b"pub fn changed_again() {}\n").unwrap();
        assert!(!snapshot.is_content_current().unwrap());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn public_cargo_input_snapshot_detects_local_source_changes() {
    let root = unique_temp_directory("resolved-cargo-inputs");
    let package = root.join("fixture");
    crate::cargo_fixture::write_workspace(&root, &["fixture"])
        .expect("write fixture workspace manifest");
    crate::cargo_fixture::write_fixture_package(&package, "", "pub fn value() -> u8 { 1 }\n")
        .expect("write fixture package");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("target"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );

    let snapshot = resolve_cargo_build_inputs(&spec).expect("resolve Cargo input snapshot");
    let facts = snapshot.facts().clone();
    assert_eq!(facts.selection, spec.cargo_selection);
    assert_eq!(facts.target, spec.target);
    assert_eq!(facts.environment, spec.environment);
    assert_ne!(facts.cargo_identity, [] as [u8; 0]);
    assert_ne!(facts.rustc_identity, [] as [u8; 0]);
    assert!(
        snapshot.artifact_paths().is_err(),
        "generic non-Wasm input evidence remains usable"
    );
    assert!(
        snapshot
            .is_current(&spec)
            .expect("revalidate unchanged inputs")
    );
    assert!(
        snapshot
            .inputs()
            .iter()
            .any(|input| input.path() == canonical_fixture(&package))
    );
    assert!(
        snapshot
            .is_content_current()
            .expect("rehash unchanged resolved inputs")
    );

    fs::write(package.join("src/lib.rs"), "pub fn value() -> u8 { 2 }\n")
        .expect("change fixture source");
    assert!(!snapshot.is_current(&spec).expect("detect changed input"));
    assert_eq!(snapshot.facts(), &facts);
    assert!(
        !snapshot
            .is_content_current()
            .expect("rehash changed resolved inputs")
    );

    let unsafe_path = package.join("src/generated-target");
    let unsafe_target = spec.clone().with_shared_incremental_target(&unsafe_path);
    assert!(matches!(
        resolve_cargo_build_inputs(&unsafe_target),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    fs::create_dir_all(&unsafe_path).expect("create unsafe maintenance target");
    let sentinel = unsafe_path.join("source-sentinel");
    fs::write(&sentinel, b"preserve").expect("write unsafe maintenance sentinel");
    let maintenance = maintain_shared_incremental_target(
        &unsafe_target,
        CacheRetentionLimits::new().with_max_size_bytes(0),
    );
    assert!(
        matches!(maintenance, Err(WasmBuildError::InvalidSpec { .. })),
        "unsafe maintenance returned {maintenance:?}",
    );
    assert!(sentinel.is_file());
    let scheduled = maintain_shared_incremental_target_at_most_every(
        &unsafe_target,
        CacheRetentionLimits::new().with_max_size_bytes(0),
        Duration::from_secs(60),
    );
    assert!(
        matches!(scheduled, Err(WasmBuildError::InvalidSpec { .. })),
        "unsafe scheduled maintenance returned {scheduled:?}",
    );
    assert!(sentinel.is_file());

    let broad_target = spec.with_shared_incremental_target(&root);
    assert!(matches!(
        resolve_cargo_build_inputs(&broad_target),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    fs::remove_dir_all(root).expect("remove Cargo input fixture");
}

#[cfg(unix)]
#[test]
fn observed_facts_retain_probe_bytes_without_io_and_redact_debug() {
    let (root, spec) = fake_wasm_build_spec("observed-build-facts");
    let cargo = PathBuf::from(&spec.cargo_program);
    let script = fs::read_to_string(&cargo).unwrap().replace(
        "set -eu\n",
        "set -eu\nif [ \"$1\" = --version ]; then printf 'cargo-observed\\n'; printf 'cargo-stderr\\n' >&2; exit 0; fi\n",
    );
    write_executable_script(&cargo, script).unwrap();
    let compiler = root.join("compiler-identity");
    write_executable_script(
        &compiler,
        "#!/bin/sh\nprintf 'compiler-observed\\n'; printf 'compiler-stderr\\n' >&2\n",
    )
    .unwrap();
    let mut environment = spec.environment().clone();
    environment.remove(OsStr::new("RUSTC"));
    environment.insert("PROVENANCE_TOKEN".into(), "private-facts-value".into());
    let spec = spec
        .with_environment(environment)
        .with_rustc_identity_program(compiler.as_os_str());
    let snapshot = resolve_cargo_build_inputs(&spec).unwrap();
    fs::remove_file(cargo).unwrap();
    fs::remove_file(compiler).unwrap();

    let facts = snapshot.facts();
    assert_eq!(facts.cargo_identity, b"cargo-observed\ncargo-stderr\n");
    assert_eq!(
        facts.rustc_identity,
        b"compiler-observed\ncompiler-stderr\n"
    );
    assert_eq!(facts.environment, spec.environment);
    let debug = format!("{snapshot:?}");
    for private in ["private-facts-value", "cargo-observed", "compiler-observed"] {
        assert!(!debug.contains(private), "Debug exposed retained facts");
    }
    for bytes in [&facts.cargo_identity, &facts.rustc_identity] {
        assert!(
            !debug.contains(&format!("{bytes:?}")),
            "Debug exposed probe bytes"
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn semantic_workspace_projection_ignores_unrelated_host_dependency_changes() {
    let root = unique_temp_directory("semantic-workspace-projection-host");
    write_projection_package(&root, "canister", "");
    write_projection_package(
        &root,
        "host",
        "\n[dependencies]\nhost_dep = { workspace = true }\n",
    );
    write_projection_package(&root, "host_dep_a", "");
    write_projection_package(&root, "host_dep_b", "");
    let workspace_manifest = root.join("Cargo.toml");
    let write_workspace = |dependency: &str| {
        fs::write(
            &workspace_manifest,
            format!(
                "[workspace]\nmembers = [\"canister\", \"host\"]\nexclude = [\"host_dep_a\", \"host_dep_b\"]\nresolver = \"2\"\n\n[workspace.dependencies]\nhost_dep = {{ package = \"{dependency}\", path = \"{dependency}\" }}\n"
            ),
        )
        .expect("write host projection workspace manifest");
    };
    write_workspace("host_dep_a");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("target"),
        &["canister"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );

    let before = resolve_cargo_build_inputs(&spec).expect("resolve initial semantic projection");
    write_workspace("host_dep_b");
    let after = resolve_cargo_build_inputs(&spec).expect("resolve changed host projection");

    assert_eq!(before.input_digest(), after.input_digest());
    assert_eq!(before.fingerprint(), after.fingerprint());
    assert_ne!(before.validation_digest(), after.validation_digest());
    assert!(before.is_current(&spec).expect("recheck semantic identity"));
    assert!(
        !before
            .is_content_current()
            .expect("validate conservative snapshot")
    );
    fs::remove_dir_all(root).expect("remove host projection fixture");
}

#[test]
fn semantic_workspace_projection_tracks_selected_dependencies_and_profiles() {
    let root = unique_temp_directory("semantic-workspace-projection-selected");
    write_projection_package(
        &root,
        "canister",
        "\n[dependencies]\nselected_dep = { workspace = true }\n",
    );
    write_projection_package(&root, "selected_dep_a", "");
    write_projection_package(&root, "selected_dep_b", "");
    let workspace_manifest = root.join("Cargo.toml");
    let write_workspace = |dependency: &str, optimize: &str| {
        fs::write(
            &workspace_manifest,
            format!(
                "[workspace]\nmembers = [\"canister\"]\nexclude = [\"selected_dep_a\", \"selected_dep_b\"]\nresolver = \"2\"\n\n[workspace.dependencies]\nselected_dep = {{ package = \"{dependency}\", path = \"{dependency}\" }}\n\n[profile.release]\nopt-level = \"{optimize}\"\n"
            ),
        )
        .expect("write selected projection workspace manifest");
    };
    write_workspace("selected_dep_a", "s");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("target"),
        &["canister"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_cargo_selection(CargoSelection {
        profile: Some("release".into()),
        ..Default::default()
    });

    let initial = resolve_cargo_build_inputs(&spec).expect("resolve selected projection");
    write_workspace("selected_dep_b", "s");
    let dependency_changed =
        resolve_cargo_build_inputs(&spec).expect("resolve changed selected dependency");
    assert_ne!(initial.input_digest(), dependency_changed.input_digest());
    assert_ne!(initial.fingerprint(), dependency_changed.fingerprint());

    write_workspace("selected_dep_b", "z");
    let profile_changed = resolve_cargo_build_inputs(&spec).expect("resolve changed profile");
    assert_ne!(
        dependency_changed.input_digest(),
        profile_changed.input_digest()
    );
    assert_ne!(
        dependency_changed.fingerprint(),
        profile_changed.fingerprint()
    );
    fs::remove_dir_all(root).expect("remove selected projection fixture");
}

#[test]
fn explicit_environment_key_separates_retained_keys_and_keeps_stamp_format() {
    let spec = WasmBuildSpec::new(
        Path::new("/workspace"),
        Path::new("/target"),
        &["fixture"],
        [
            "CARGO_BUILD_RUSTC",
            "CARGO_ENCODED_RUSTFLAGS",
            "RUSTC",
            "RUSTC_WRAPPER",
            "RUSTC_WORKSPACE_WRAPPER",
            "RUSTFLAGS",
            "RUSTUP_TOOLCHAIN",
        ]
        .map(|name| (name, "selected")),
        CONFIGURATION_LIMITS,
    );
    let fingerprint = super::finish_build_fingerprint(
        &spec,
        &spec
            .cargo_invocation()
            .facts(b"cargo".to_vec(), b"rustc".to_vec()),
        super::InputDigest::from_bytes([7; 32]),
    );
    // Independent byte-framing fixtures: every recipe fact is the same, only
    // the domain differs. This also isolates ambient environment/tool choices.
    assert_eq!(
        fingerprint.to_hex(),
        "a3db61f964cd022bebd7068eba254d93916c18ee8f046d7708083388f7e65f82"
    );
    assert_ne!(
        fingerprint.to_hex(),
        "18ec3a22f7bf556ef4858afa79aea7df5fd3c18939f7056cc4c481f4d3fefa46"
    );
    assert_eq!(
        super::artifact_stamp_header(fingerprint),
        format!("ic-testkit-wasm-build-v1\nbuild-sha256:{fingerprint}\n")
    );
}

#[test]
fn semantic_projection_uses_typed_v2_metadata_fields() {
    let id = "path+file:///workspace/member#member@1.2.3";
    let original = serde_json::json!({
        "id": id, "name": "member", "version": "1.2.3",
        "manifest_path": "/workspace/member/Cargo.toml", "source": null,
        "dependencies": [], "targets": [], "features": {},
        "metadata": {"z": 7, "a": true}, "description": null,
        "authors": [], "edition": "2024", "rust_version": "1.88",
    });
    let package = serde_json::from_value(original.clone()).unwrap();
    let identity = semantic_package_identity(
        &package,
        Path::new("/workspace"),
        &super::resolution::LockedPackageIndex::new(),
    )
    .unwrap();
    let value = serde_json::json!({
        "id": id, "features": ["z", "a"], "dependencies": [],
        "deps": [{"name": "self", "pkg": id,
            "dep_kinds": [{"kind": null, "target": null, "future_field": "preserve"}]}],
    });
    let node = serde_json::from_value(value.clone()).unwrap();
    let projection = super::resolution::semantic_package_projection(
        &package,
        &node,
        &std::collections::HashMap::from([(id, identity)]),
    )
    .unwrap();

    // Package identity framing is unchanged; the typed projection starts a new
    // key domain rather than pretending to preserve the former raw JSON bytes.
    assert_eq!(
        identity.to_hex(),
        "ba511839396518effb69ed5d2aaf730822eb6c49bba88c264799b97b76059adb"
    );
    assert_ne!(
        projection.to_hex(),
        "aa1361a94b31abb1e0c59b6b7c90e6e9946d137e357612f80cff3189e0d48b3a"
    );
    let identities = std::collections::HashMap::from([(id, identity)]);
    let mut normalized = original.clone();
    normalized.as_object_mut().unwrap().remove("description");
    normalized["rust_version"] = serde_json::json!("1.88.0");
    let normalized = serde_json::from_value(normalized).unwrap();
    assert_eq!(
        projection,
        super::resolution::semantic_package_projection(&normalized, &node, &identities).unwrap()
    );
    let mut reordered = value.clone();
    reordered["features"] = serde_json::json!(["a", "z"]);
    reordered["deps"][0]["dep_kinds"][0]
        .as_object_mut()
        .unwrap()
        .remove("future_field");
    let reordered = serde_json::from_value(reordered).unwrap();
    assert_eq!(
        projection,
        super::resolution::semantic_package_projection(&package, &reordered, &identities).unwrap()
    );
    for (field, changed) in [
        ("description", serde_json::json!("changed")),
        ("metadata", serde_json::json!({"z": 8, "a": true})),
        ("edition", serde_json::json!("2021")),
        ("rust_version", serde_json::json!("1.91")),
    ] {
        let mut changed_package = original.clone();
        changed_package[field] = changed;
        let changed_package = serde_json::from_value(changed_package).unwrap();
        assert_ne!(
            projection,
            super::resolution::semantic_package_projection(&changed_package, &node, &identities)
                .unwrap(),
            "{field} remains hash-significant"
        );
    }
    for changed in [
        serde_json::json!({"name":"renamed", "pkg":id, "dep_kinds":[{"kind":null,"target":null}]}),
        serde_json::json!({"name":"self", "pkg":id, "dep_kinds":[{"kind":"build","target":null}]}),
        serde_json::json!({"name":"self", "pkg":id, "dep_kinds":[{"kind":null,"target":"cfg(unix)"}]}),
    ] {
        let mut changed_node = value.clone();
        changed_node["deps"][0] = changed;
        let changed_node = serde_json::from_value(changed_node).unwrap();
        assert_ne!(
            projection,
            super::resolution::semantic_package_projection(&package, &changed_node, &identities)
                .unwrap()
        );
    }
    let mut unsupported = value;
    unsupported["deps"][0]["dep_kinds"][0]["kind"] = serde_json::json!("future-kind");
    let unsupported = serde_json::from_value(unsupported).unwrap();
    assert!(matches!(
        super::resolution::semantic_package_projection(&package, &unsupported, &identities),
        Err(WasmBuildError::InvalidMetadata { message }) if message.contains("unsupported")
    ));
}

#[test]
fn semantic_projection_survives_checkout_relocation() {
    let projected = ["/original", "/relocated"].map(|root| {
        let id = format!("path+file://{root}/member#member@1.2.3");
        let package = serde_json::from_value(serde_json::json!({
            "id": id, "name": "member", "version": "1.2.3",
            "manifest_path": format!("{root}/member/Cargo.toml"), "source": null,
            "dependencies": [], "targets": [], "features": {}, "edition": "2024",
            "readme": "README.md", "license_file": "LICENSE",
        }))
        .unwrap();
        let node = serde_json::from_value(serde_json::json!({
            "id": id, "features": ["a"], "dependencies": [],
            "deps": [{"name": "self", "pkg": id,
                "dep_kinds": [{"kind": null, "target": null}]}],
        }))
        .unwrap();
        let identity = semantic_package_identity(
            &package,
            Path::new(root),
            &super::resolution::LockedPackageIndex::new(),
        )
        .unwrap();
        let projection = super::resolution::semantic_package_projection(
            &package,
            &node,
            &std::collections::HashMap::from([(id.as_str(), identity)]),
        )
        .unwrap();
        (identity, projection)
    });
    assert_eq!(projected[0], projected[1]);
}

#[test]
fn semantic_package_identity_uses_selected_registry_lock_checksum() {
    let root = unique_temp_directory("semantic-registry-lock-identity");
    let source = "registry+https://example.invalid/index";
    let package_id = format!("{source}#dependency@1.2.3");
    let original = serde_json::json!({
        "id": package_id, "name": "dependency", "version": "1.2.3",
        "manifest_path": root.join("registry/dependency/Cargo.toml"),
        "source": source, "dependencies": [], "targets": [], "features": {}, "metadata": null,
    });
    let package = serde_json::from_value(original).unwrap();
    let write_lock = |checksum: Option<&str>| {
        let checksum = checksum.map_or_else(String::new, |checksum| {
            format!("checksum = \"{checksum}\"\n")
        });
        fs::write(
            root.join("Cargo.lock"),
            format!(
                "version = 4\n\n[[package]]\nname = \"dependency\"\nversion = \"1.2.3\"\nsource = \"{source}\"\n{checksum}"
            ),
        )
        .expect("write registry identity lockfile");
    };

    write_lock(Some("aaa"));
    let first_lock = locked_package_identities(&root).expect("project first lockfile");
    let first = semantic_package_identity(
        &package,
        &root,
        &super::resolution::index_locked_packages(&first_lock),
    )
    .expect("project checksummed registry package");
    write_lock(Some("bbb"));
    let second_lock = locked_package_identities(&root).expect("project second lockfile");
    let second = semantic_package_identity(
        &package,
        &root,
        &super::resolution::index_locked_packages(&second_lock),
    )
    .expect("project changed registry checksum");
    assert_ne!(first, second);

    write_lock(None);
    let missing_checksum = locked_package_identities(&root).expect("project incomplete lockfile");
    assert!(
        semantic_package_identity(
            &package,
            &root,
            &super::resolution::index_locked_packages(&missing_checksum)
        )
        .is_none()
    );

    // Same names with different versions/sources must not shadow the exact row;
    // duplicate exact rows retain the first checksum, even when it is missing.
    for checksum in [Some("aaa"), None] {
        write_lock(checksum);
        let selected = fs::read_to_string(root.join("Cargo.lock")).unwrap();
        let row = |name: &str, version: &str, source: &str| {
            format!(
                "\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\nsource = \"{source}\"\nchecksum = \"decoy\"\n"
            )
        };
        let mut contents = String::from("version = 4\n");
        contents.push_str(&row("other", "1.2.3", source));
        contents.push_str(&row("dependency", "1.2.4", source));
        contents.push_str(&row(
            "dependency",
            "1.2.3",
            "registry+https://other.invalid",
        ));
        contents.push_str(selected.strip_prefix("version = 4\n").unwrap());
        contents.push_str(&row("dependency", "1.2.3", source));
        fs::write(root.join("Cargo.lock"), contents).unwrap();
        let rows = locked_package_identities(&root).unwrap();
        assert_eq!(
            semantic_package_identity(
                &package,
                &root,
                &super::resolution::index_locked_packages(&rows)
            ),
            checksum.map(|_| first)
        );
    }
    fs::remove_dir_all(root).expect("remove registry identity fixture");
}

#[test]
fn semantic_workspace_projection_falls_back_for_workspace_root_packages() {
    let root = unique_temp_directory("semantic-workspace-projection-fallback");
    fs::create_dir_all(root.join("src")).expect("create root package source directory");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"root_canister\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write root package manifest");
    fs::write(root.join("src/lib.rs"), "pub fn root_canister() {}\n")
        .expect("write root package source");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("target/exact"),
        &["root_canister"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );

    let resolved = resolve_cargo_build_inputs(&spec).expect("resolve fallback projection");
    assert_eq!(resolved.input_digest(), resolved.validation_digest());
    fs::remove_dir_all(root).expect("remove projection fallback fixture");
}

#[test]
#[cfg(unix)]
fn batch_input_snapshot_reuses_compatible_toolchain_and_metadata_resolution() {
    let root = unique_temp_directory("batch-input-snapshot");
    for package in ["fixture_a", "fixture_b"] {
        fs::create_dir_all(root.join(package).join("src")).expect("create batch fixture package");
        fs::write(
            root.join(package).join("Cargo.toml"),
            format!("[package]\nname = \"{package}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[features]\nalpha = []\n"),
        )
        .expect("write batch fixture manifest");
        fs::write(
            root.join(package).join("src/lib.rs"),
            "pub fn fixture() {}\n",
        )
        .expect("write batch fixture source");
    }
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture_a\", \"fixture_b\"]\nresolver = \"2\"\n",
    )
    .expect("write batch fixture workspace");
    let invocation_log = root.join("cargo-invocations");
    let cargo_wrapper = root.join("cargo-wrapper.sh");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    write_executable_script(
        &cargo_wrapper,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$1\" >> '{}'\nexec '{}' \"$@\"\n",
            invocation_log.display(),
            PathBuf::from(cargo).display(),
        ),
    )
    .expect("write executable test script");
    let target = root.join("exact");
    let selected = crate::cargo_fixture::environment();
    let recipe = |package| {
        WasmBuildSpec::new(
            &root,
            &target,
            &[package],
            selected.clone(),
            CONFIGURATION_LIMITS,
        )
        .with_cargo_program(&cargo_wrapper)
    };
    let alpha = CargoSelection {
        features: CargoFeatureSelection {
            features: ["alpha".into()].into(),
            ..Default::default()
        },
        ..Default::default()
    };
    let specs = [
        recipe("fixture_a"),
        recipe("fixture_b").with_cargo_selection(alpha.clone()),
        recipe("fixture_b"),
        recipe("fixture_a").with_cargo_selection(alpha),
    ];
    let mut resolver = WasmBuildBatchInputResolver::new(&specs, None).unwrap();
    let mut progress = ProgressReporter::silent();

    let snapshots = (0..specs.len())
        .map(|index| resolver.resolve(index, &mut progress).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(resolver.metrics().runs, 2);
    assert_eq!(resolver.metrics().reuses, 2);
    for (index, inputs) in snapshots.iter().enumerate() {
        assert_eq!(inputs.timings().total() > Duration::ZERO, index < 2);
        assert_eq!(inputs.facts().selection, specs[index].cargo_selection);
        assert_eq!(
            inputs.facts().cargo_identity,
            snapshots[0].facts().cargo_identity
        );
    }

    let invocations = fs::read_to_string(&invocation_log).expect("read Cargo invocation log");
    assert_eq!(
        invocations
            .lines()
            .filter(|line| *line == "--version")
            .count(),
        2
    );
    assert_eq!(
        invocations
            .lines()
            .filter(|line| *line == "metadata")
            .count(),
        2
    );
    for (spec, inputs) in specs.iter().zip(&snapshots) {
        let mut fresh =
            resolve_cargo_build_inputs(spec).expect("resolve equivalent standalone inputs");
        let mut batched = inputs.clone();
        fresh.timings = super::WasmInputResolutionTimings::default();
        batched.timings = super::WasmInputResolutionTimings::default();
        assert_eq!(
            batched, fresh,
            "all snapshot fields must agree independently of timing attribution"
        );
    }
    assert_ne!(snapshots[0].fingerprint(), snapshots[3].fingerprint());
    assert_ne!(snapshots[1].fingerprint(), snapshots[2].fingerprint());
    fs::remove_dir_all(root).expect("remove batch input snapshot fixture");
}

#[test]
fn profile_output_directory_is_confined_before_resolution_or_acquisition() {
    let root = unique_temp_directory("wasm-profile-boundary");
    let target = root.join("target");
    let absolute = root.join("outside");
    for profile in [
        "",
        ".",
        "..",
        "../outside",
        "release/../../outside",
        "release/nested",
        absolute.to_str().unwrap(),
    ] {
        let spec = WasmBuildSpec::new(
            &root,
            &target,
            &["fixture"],
            crate::cargo_fixture::environment(),
            CONFIGURATION_LIMITS,
        )
        .with_cargo_selection(CargoSelection {
            profile: Some(profile.into()),
            ..Default::default()
        })
        .with_cargo_program(root.join("missing-cargo"));
        assert!(
            matches!(
                resolve_cargo_build_inputs(&spec),
                Err(WasmBuildError::InvalidSpec { .. })
            ),
            "{profile:?} must fail before tool resolution",
        );
        assert!(
            matches!(
                build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct),
                Err(WasmBuildError::InvalidSpec { .. })
            ),
            "{profile:?} must fail before acquisition",
        );
        assert!(!target.exists());
        assert!(!absolute.exists());
    }
    for profile in ["debug", "release", "fast", "release-with-debug"] {
        let spec = WasmBuildSpec::new(
            &root,
            &target,
            &["fixture"],
            crate::cargo_fixture::environment(),
            CONFIGURATION_LIMITS,
        )
        .with_cargo_selection(CargoSelection {
            profile: Some(profile.into()),
            ..Default::default()
        });
        validate_spec(&spec).expect("accept a normal profile output directory");
        assert!(expected_artifacts(&spec, &target)[0].starts_with(&target));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cargo_profiles_match_output_directories() {
    for (directory, profile) in [
        ("debug", None),
        ("debug", Some("dev")),
        ("debug", Some("test")),
        ("release", Some("release")),
        ("release", Some("bench")),
        ("fast", Some("fast")),
    ] {
        let spec = WasmBuildSpec::new(
            Path::new("."),
            Path::new("target"),
            &["fixture"],
            crate::cargo_fixture::environment(),
            CONFIGURATION_LIMITS,
        )
        .with_cargo_selection(CargoSelection {
            profile: profile.map(str::to_owned),
            ..Default::default()
        });
        validate_spec(&spec).unwrap();
        assert_eq!(
            expected_artifacts(&spec, Path::new("target")),
            [Path::new("target")
                .join("wasm32-unknown-unknown")
                .join(directory)
                .join("fixture.wasm")],
        );
    }
}

#[cfg(unix)]
#[test]
fn wasm_output_validation_distinguishes_libraries_from_cdylib_examples() {
    let (root, spec) = fake_wasm_build_spec("wasm-library-versus-example");
    fs::create_dir_all(root.join("fixture/examples")).unwrap();
    fs::write(
        root.join("fixture/examples/fixture.rs"),
        "pub fn example() {}\n",
    )
    .unwrap();
    let manifest = root.join("fixture/Cargo.toml");
    let original = fs::read_to_string(&manifest).unwrap();
    let example = "\n[[example]]\nname = \"fixture\"\ncrate-type = [\"cdylib\"]\n";
    fs::write(&manifest, original.replace("cdylib", "rlib") + example).unwrap();
    resolve_cargo_build_inputs(&spec).expect("generic snapshot with a cdylib example");
    assert!(matches!(
        super::build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    assert!(
        !spec
            .target_dir
            .join("wasm32-unknown-unknown/debug/fixture.wasm")
            .exists()
    );

    fs::write(
        &manifest,
        original.replace("[\"cdylib\"]", "[\"rlib\", \"cdylib\"]") + example,
    )
    .unwrap();
    let built = super::build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("library with both crate types");
    assert!(matches!(built, WasmBuildOutcome::Built(_)));
    drop(built);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn build_spec_requires_at_least_one_package() {
    let zero_messages = WasmBuildSpec::new(
        Path::new("."),
        Path::new("target"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_cargo_message_bytes(0);
    assert!(matches!(
        validate_spec(&zero_messages),
        Err(WasmBuildError::InvalidSpec { .. })
    ));
    let spec = WasmBuildSpec::new(
        Path::new("."),
        Path::new("target"),
        &[],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );
    assert!(matches!(
        validate_spec(&spec),
        Err(WasmBuildError::InvalidSpec { .. })
    ));

    let isolated_maintenance = WasmBuildSpec::new(
        Path::new("."),
        Path::new("target"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_isolated_builds()
    .with_shared_incremental_target_maintenance_at_most_every(
        CacheRetentionLimits::new(),
        Duration::from_secs(60),
    );
    assert!(matches!(
        validate_spec(&isolated_maintenance),
        Err(WasmBuildError::InvalidSpec { message }) if message.contains("requires a shared")
    ));
}

#[test]
fn shared_target_inspection_is_explicit_and_does_not_create_a_missing_target() {
    let root = unique_temp_directory("shared-target-inspection");
    let target = root.join("missing-shared-target");
    let isolated = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_isolated_builds();
    assert!(matches!(
        inspect_shared_incremental_target(&isolated),
        Err(WasmBuildError::InvalidSpec { .. })
    ));

    let shared = isolated.with_shared_incremental_target(&target);
    assert!(
        inspect_shared_incremental_target(&shared)
            .expect("inspect missing shared target")
            .is_none()
    );
    assert!(!target.exists());
    let scheduled = maintain_shared_incremental_target_at_most_every(
        &shared,
        CacheRetentionLimits::new(),
        Duration::from_secs(60),
    )
    .expect("schedule missing shared target maintenance");
    assert!(matches!(
        &scheduled,
        SharedIncrementalTargetMaintenanceOutcome::Missing { .. }
    ));
    assert_eq!(scheduled.target_dir(), target);
    assert!(!scheduled.was_performed());
    assert_eq!(scheduled.lock_wait(), None);
    assert_eq!(scheduled.schedule_check(), None);
    assert!(!target.exists());
    fs::remove_dir_all(root).expect("remove shared-target inspection fixture");
}

#[test]
fn shared_target_maintenance_clears_cargo_state_but_preserves_coordination() {
    let root = unique_temp_directory("shared-target-maintenance");
    let package = root.join("fixture");
    fs::create_dir_all(package.join("src")).expect("create maintenance fixture package");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .expect("write maintenance fixture workspace");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write maintenance fixture manifest");
    fs::write(package.join("src/lib.rs"), "pub fn fixture() {}\n")
        .expect("write maintenance fixture source");
    let target = root.join("shared-target");
    fs::create_dir_all(target.join("debug/deps")).expect("create shared Cargo state");
    fs::write(target.join("debug/deps/object.o"), vec![7_u8; 128])
        .expect("write shared Cargo state");
    fs::write(target.join(".rustc_info.json"), b"rustc state")
        .expect("write shared Cargo metadata");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_shared_incremental_target(&target);

    let report = maintain_shared_incremental_target(
        &spec,
        CacheRetentionLimits::new().with_max_size_bytes(0),
    )
    .expect("maintain shared target")
    .expect("existing shared target report");

    assert!(report.was_cleared());
    assert!(report.logical_size_bytes_before() > report.logical_size_bytes_after());
    assert!(!target.join("debug").exists());
    assert!(!target.join(".rustc_info.json").exists());
    assert!(target.join("CACHEDIR.TAG").is_file());
    assert!(target.join(".ic-testkit/wasm-incremental.lock").is_file());

    let retained = maintain_shared_incremental_target(&spec, CacheRetentionLimits::new())
        .expect("reinspect shared target")
        .expect("existing shared target report");
    assert!(!retained.was_cleared());
    assert_eq!(
        retained.logical_size_bytes_before(),
        retained.logical_size_bytes_after()
    );
    fs::remove_dir_all(root).expect("remove shared-target maintenance fixture");
}

#[test]
fn scheduled_shared_target_maintenance_skips_expensive_work_inside_interval() {
    let root = unique_temp_directory("scheduled-shared-target-maintenance");
    let package = root.join("fixture");
    fs::create_dir_all(package.join("src")).expect("create scheduled fixture package");
    let workspace_manifest = root.join("Cargo.toml");
    fs::write(
        &workspace_manifest,
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .expect("write scheduled fixture workspace");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("write scheduled fixture manifest");
    fs::write(package.join("src/lib.rs"), "pub fn fixture() {}\n")
        .expect("write scheduled fixture source");
    let target = root.join("shared-target");
    fs::create_dir_all(target.join("debug/deps")).expect("create scheduled Cargo state");
    fs::write(target.join("debug/deps/object.o"), vec![7_u8; 128])
        .expect("write scheduled Cargo state");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_shared_incremental_target(&target);
    let policy = CacheRetentionLimits::new().with_max_size_bytes(0);
    let interval = Duration::from_hours(1);

    let first = maintain_shared_incremental_target_at_most_every(&spec, policy, interval)
        .expect("perform scheduled shared-target maintenance");
    let report = first
        .maintenance()
        .expect("first scheduled maintenance must be performed");
    assert!(first.was_performed());
    assert!(report.was_cleared());
    assert!(first.lock_wait().is_some());
    assert!(first.schedule_check().is_some());

    let sentinel = target.join("debug/deps/preserve-on-skip");
    fs::create_dir_all(sentinel.parent().expect("scheduled sentinel parent"))
        .expect("recreate shared Cargo state");
    fs::write(&sentinel, b"preserve").expect("write scheduled skip sentinel");
    fs::remove_file(&workspace_manifest).expect("hide Cargo metadata from skipped call");

    let skipped = maintain_shared_incremental_target_at_most_every(&spec, policy, interval)
        .expect("skip recently completed shared-target maintenance");
    assert!(matches!(
        &skipped,
        SharedIncrementalTargetMaintenanceOutcome::Skipped { .. }
    ));
    assert!(!skipped.was_performed());
    assert!(skipped.lock_wait().is_some());
    assert!(skipped.schedule_check().is_some());
    assert!(sentinel.is_file());

    fs::write(
        &workspace_manifest,
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .expect("restore scheduled fixture workspace");
    let changed_policy = CacheRetentionLimits::new().with_max_size_bytes(u64::MAX);
    let changed = maintain_shared_incremental_target_at_most_every(&spec, changed_policy, interval)
        .expect("changed policy must make scheduled maintenance due");
    assert!(changed.was_performed());
    assert!(sentinel.is_file());

    let zero_interval =
        maintain_shared_incremental_target_at_most_every(&spec, changed_policy, Duration::ZERO)
            .expect("zero interval must perform shared-target maintenance");
    assert!(zero_interval.was_performed());

    fs::remove_dir_all(root).expect("remove scheduled shared-target fixture");
}

#[test]
fn zero_progress_heartbeat_is_rejected_for_a_valid_build() {
    let root = unique_temp_directory("zero-progress-heartbeat");
    fs::create_dir_all(&root).expect("create heartbeat fixture");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .expect("write heartbeat workspace");
    write_projection_package(&root, "fixture", "[lib]\ncrate-type = [\"cdylib\"]\n");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    );
    validate_spec(&spec).expect("valid heartbeat build spec");
    let result = crate::artifacts::build_wasm_canisters_cached_with_progress(
        &spec,
        WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::ZERO),
        &mut |_| {},
        spawn_cargo,
    );
    assert!(matches!(result, Err(WasmBuildError::InvalidSpec { .. })));
    assert!(!root.join("exact").exists());
    let outcome = crate::artifacts::build_wasm_canisters_cached_with_progress(
        &spec,
        WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::from_secs(1)),
        &mut |_| {},
        spawn_cargo,
    )
    .expect("the same build succeeds with a positive heartbeat interval");
    assert!(outcome.record().artifacts()[0].is_file());
    drop(outcome);
    fs::remove_dir_all(root).expect("remove heartbeat fixture");
}

#[test]
fn maintenance_configuration_is_readable_and_strict_by_default() {
    let shared_policy = CacheRetentionLimits::new().with_max_size_bytes(1024 * 1024);
    let shared_interval = Duration::from_hours(1);
    let shared_config =
        SharedIncrementalTargetMaintenanceConfig::new(shared_policy, shared_interval);
    let exact_policy = CacheRetentionLimits::new().with_max_size_bytes(2 * 1024 * 1024);
    let exact_interval = Duration::from_mins(30);
    let spec = WasmBuildSpec::new(
        Path::new("."),
        Path::new("target"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_shared_incremental_target("target/shared")
    .with_shared_incremental_target_maintenance(shared_config)
    .with_prune_policy_at_most_every(exact_policy, exact_interval);

    assert_eq!(shared_config.policy(), shared_policy);
    assert_eq!(shared_config.minimum_interval(), shared_interval);
    assert_eq!(
        shared_config.failure_mode(),
        SharedIncrementalTargetMaintenanceFailureMode::Strict
    );
    assert_eq!(
        spec.shared_incremental_target_maintenance(),
        Some(shared_config)
    );
    assert_eq!(spec.prune_policy(), Some(exact_policy));
    assert_eq!(spec.prune_interval(), Some(exact_interval));
    for always in [
        spec.clone().with_prune_policy(exact_policy),
        spec.with_prune_policy_at_most_every(exact_policy, Duration::ZERO),
    ] {
        assert_eq!(always.prune_policy(), Some(exact_policy));
        assert_eq!(always.prune_interval(), None);
    }
}

#[cfg(unix)]
#[test]
fn zero_interval_maintenance_ignores_bad_markers_and_keeps_exact_hits() {
    let (root, spec) = fake_wasm_build_spec("zero-interval-maintenance");
    let first = build_wasm_canisters_cached(&spec, spawn_cargo).unwrap();
    assert_eq!(first.record().maintenance(), None);
    let marker = spec
        .target_dir()
        .join(".ic-testkit/wasm-targets")
        .join(".ic-testkit-last-maintenance");
    for explicit_zero in [false, true] {
        let always = if explicit_zero {
            spec.clone()
                .with_prune_policy_at_most_every(CacheRetentionLimits::new(), Duration::ZERO)
        } else {
            spec.clone().with_prune_policy(CacheRetentionLimits::new())
        };
        fs::write(&marker, [0xff]).unwrap();
        let outcome = build_wasm_canisters_cached(&always, spawn_cargo).unwrap();
        assert!(outcome.is_reused());
        assert!(matches!(
            outcome.record().maintenance(),
            Some(crate::artifacts::ArtifactCacheMaintenance::Pruned(_))
        ));
        assert_eq!(outcome.record().fingerprint(), first.record().fingerprint());
        assert!(fs::read_to_string(&marker).is_ok());
    }
    drop(first);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn best_effort_integrated_maintenance_retains_a_failure_message() {
    let root = unique_temp_directory("best-effort-shared-maintenance-failure");
    let target = root.join("shared");
    let schedule_root = target.join(".ic-testkit");
    fs::create_dir_all(&schedule_root).expect("create corrupt schedule fixture");
    fs::write(schedule_root.join(".ic-testkit-last-maintenance"), [0xff])
        .expect("write invalid UTF-8 schedule marker");
    let config = SharedIncrementalTargetMaintenanceConfig::new(
        CacheRetentionLimits::new(),
        Duration::from_secs(60),
    )
    .with_failure_mode(SharedIncrementalTargetMaintenanceFailureMode::BestEffort);
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_shared_incremental_target(&target)
    .with_shared_incremental_target_maintenance(config);
    let lock_wait = Duration::from_millis(7);
    let outcome = perform_configured_shared_incremental_target_maintenance(
        &spec,
        &target,
        lock_wait,
        &mut ProgressReporter::silent(),
    )
    .expect("best-effort maintenance must preserve acquisition");

    assert_eq!(outcome.target_dir(), target);
    assert_eq!(outcome.lock_wait(), Some(lock_wait));
    assert!(!outcome.was_performed());
    assert!(outcome.schedule_check().is_none());
    assert!(matches!(
        &outcome,
        SharedIncrementalTargetMaintenanceOutcome::Failed { .. }
    ));
    assert!(
        outcome
            .failure_message()
            .is_some_and(|message| !message.is_empty())
    );
    assert_eq!(
        fs::read(schedule_root.join(".ic-testkit-last-maintenance")).unwrap(),
        [0xff]
    );
    fs::remove_dir_all(root).expect("remove best-effort maintenance fixture");
}

#[test]
fn strict_integrated_maintenance_propagates_failure() {
    let config = SharedIncrementalTargetMaintenanceConfig::new(
        CacheRetentionLimits::new(),
        Duration::from_secs(60),
    );
    let result = integrated_shared_maintenance_result(
        config,
        Path::new("target/shared"),
        Duration::ZERO,
        Err(WasmBuildError::InvalidSpec {
            message: "synthetic strict failure".to_owned(),
        }),
    );

    assert!(matches!(
        result,
        Err(WasmBuildError::InvalidSpec { message }) if message == "synthetic strict failure"
    ));
}

#[test]
fn observed_phase_emits_phase_aware_heartbeats() {
    let (release, released) = mpsc::channel();
    let mut events = Vec::new();
    let value = {
        let mut observer = |event| {
            if matches!(
                event,
                WasmBuildProgressEvent::Heartbeat {
                    phase: WasmBuildPhase::ContentHashing,
                    ..
                }
            ) {
                let _ = release.send(());
            }
            events.push(event);
        };
        let mut progress = ProgressReporter::new(Some((
            WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::from_millis(5)),
            &mut observer,
        )))
        .unwrap();
        progress.run_phase(WasmBuildPhase::ContentHashing, move || {
            released
                .recv_timeout(Duration::from_secs(10))
                .expect("observer must release the operation on its heartbeat");
            42
        })
    };

    assert_eq!(value, 42);
    assert!(events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::Heartbeat {
            phase: WasmBuildPhase::ContentHashing,
            ..
        }
    )));
}

#[test]
fn observer_panic_joins_active_phase_worker() {
    let (release, released) = mpsc::channel::<()>();
    let mut release = Some(release);
    let completed = Arc::new(AtomicBool::new(false));
    let worker_completed = Arc::clone(&completed);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut observer = |event| {
            if matches!(
                event,
                WasmBuildProgressEvent::Heartbeat {
                    phase: WasmBuildPhase::InputDiscovery,
                    ..
                }
            ) {
                // Unwinding this callback releases the worker before scope cleanup
                // joins it; it never needs a callback after the observer panics.
                let _release_on_unwind = release.take().expect("one observer panic");
                panic!("synthetic observer panic");
            }
        };
        let mut progress = ProgressReporter::new(Some((
            WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::from_millis(5)),
            &mut observer,
        )))
        .unwrap();
        progress.run_phase(WasmBuildPhase::InputDiscovery, move || {
            assert_eq!(
                released.recv_timeout(Duration::from_secs(10)),
                Err(mpsc::RecvTimeoutError::Disconnected),
                "observer unwinding must release the active worker"
            );
            worker_completed.store(true, Ordering::SeqCst);
        });
    }));

    let panic = result.expect_err("observer must panic");
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"synthetic observer panic")
    );
    assert!(completed.load(Ordering::SeqCst));
}

#[test]
fn exact_cache_lock_wait_emits_phase_aware_heartbeats() {
    let target_dir = unique_temp_directory("observed-exact-lock-wait");
    let (held_lock, _) = lock_wasm_build_cache(&target_dir).expect("acquire fixture cache lock");
    let (heartbeat_sender, heartbeat_receiver) = mpsc::channel();
    let release = thread::spawn(move || {
        heartbeat_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("wait for exact-cache lock heartbeat");
        drop(held_lock);
    });
    let mut events = Vec::new();
    {
        let mut observer = |event| {
            if matches!(
                &event,
                WasmBuildProgressEvent::Heartbeat {
                    phase: WasmBuildPhase::ExactCacheCoordination,
                    ..
                }
            ) {
                let _ = heartbeat_sender.send(());
            }
            events.push(event);
        };
        let mut progress = ProgressReporter::new(Some((
            WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::from_millis(5)),
            &mut observer,
        )))
        .unwrap();
        let (_lock, wait) = lock_wasm_build_cache_with_progress(&target_dir, &mut progress)
            .expect("acquire observed fixture cache lock");
        assert!(wait >= Duration::from_millis(5));
    }
    release.join().expect("join fixture lock holder");

    assert!(events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::Heartbeat {
            phase: WasmBuildPhase::ExactCacheCoordination,
            ..
        }
    )));
    fs::remove_dir_all(target_dir).expect("remove observed lock fixture");
}

#[test]
#[cfg(unix)]
fn observed_cargo_build_forwards_raw_output_and_quiet_heartbeats() {
    let root = unique_temp_directory("observed-cargo-progress");
    // Cargo's first argument is `build`; let an existing shell read that fixture
    // instead of executing a freshly written file, which can fail with ETXTBSY.
    let script = root.join("build");
    let release = root.join("release-cargo");
    // Keep the producer alive until the observer actually sees a quiet interval.
    // A fixed sleep can finish before output readers are scheduled under load.
    fs::write(
        &script,
        r#"set -eu
printf 'observed-stdout'
printf 'observed-stderr' >&2
attempts=0
while [ ! -f "$IC_TESTKIT_OBSERVED_CARGO_RELEASE" ]; do
    attempts=$((attempts + 1))
    if [ "$attempts" -ge 1000 ]; then
        printf 'heartbeat observer did not release Cargo fixture\n' >&2
        exit 1
    fi
    sleep 0.01
done
"#,
    )
    .expect("write observed Cargo fixture");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_cargo_program("/bin/sh")
    .with_environment(crate::cargo_fixture::environment_with([(
        "IC_TESTKIT_OBSERVED_CARGO_RELEASE",
        release.as_os_str(),
    )]));
    let mut events = Vec::new();
    {
        let mut observer = |event| {
            if matches!(
                &event,
                WasmBuildProgressEvent::Heartbeat {
                    phase: WasmBuildPhase::CargoBuild,
                    ..
                }
            ) {
                fs::write(&release, b"released").expect("release Cargo on its observed heartbeat");
            }
            events.push(event);
        };
        let mut progress = ProgressReporter::new(Some((
            WasmBuildProgressConfig::new().with_heartbeat_interval(Duration::from_millis(10)),
            &mut observer,
        )))
        .unwrap();

        run_cargo_build(
            &spec,
            &root.join("cargo-target"),
            &[],
            &mut progress,
            &mut (spawn_cargo),
        )
        .expect("run observed Cargo fixture");
    }

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    for event in &events {
        if let WasmBuildProgressEvent::CargoOutput { stream, bytes } = event {
            match stream {
                OutputStream::Stdout => stdout.extend_from_slice(bytes),
                OutputStream::Stderr => stderr.extend_from_slice(bytes),
            }
        }
    }
    assert_eq!(stdout, b"observed-stdout");
    assert_eq!(stderr, b"observed-stderr");
    assert!(events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::Heartbeat {
            phase: WasmBuildPhase::CargoBuild,
            ..
        }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        WasmBuildProgressEvent::CargoFinished { success: true, .. }
    )));
    fs::remove_dir_all(root).expect("remove observed Cargo fixture");
}

#[test]
#[cfg(unix)]
fn observed_cargo_parallel_streams_preserve_raw_bytes() {
    let root = unique_temp_directory("cargo-parallel-streams");
    let stdout = (0_u8..=255)
        .cycle()
        .take(super::CARGO_DIAGNOSTIC_BYTES + 37)
        .collect::<Vec<_>>();
    let stderr = b"diagnostic\0\xff\n".repeat(128 * 1024);
    fs::write(root.join("stdout"), &stdout).unwrap();
    fs::write(root.join("stderr"), &stderr).unwrap();
    fs::write(
        root.join("build"),
        "set -eu\ncat stdout &\ncat stderr >&2 &\nwait\n",
    )
    .unwrap();
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_cargo_program("/bin/sh");
    let mut forwarded_stdout = Vec::new();
    let mut forwarded_stderr = Vec::new();
    let mut observer = |event| {
        if let WasmBuildProgressEvent::CargoOutput { stream, bytes } = event {
            match stream {
                OutputStream::Stdout => forwarded_stdout.extend(bytes),
                OutputStream::Stderr => forwarded_stderr.extend(bytes),
            }
        }
    };
    let mut progress = ProgressReporter::new(Some((
        WasmBuildProgressConfig::new()
            .without_heartbeats()
            .with_cargo_output(true),
        &mut observer,
    )))
    .unwrap();
    run_cargo_build(
        &spec,
        &root.join("cargo-target"),
        &[],
        &mut progress,
        &mut (spawn_cargo),
    )
    .unwrap();
    assert_eq!(forwarded_stdout, stdout);
    assert_eq!(forwarded_stderr, stderr);
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn observed_cargo_leader_exit_cleans_descendant_held_pipes_without_eof() {
    let root = unique_temp_directory("cargo-descendant-pipes");
    let release = root.join("release");
    fs::write(root.join("build"),
        "set -eu\n/bin/sh -c 'while [ ! -f release ]; do sleep 0.01; done' &\nprintf leader-exited\nexit 0\n").unwrap();
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_cargo_program("/bin/sh");
    let (done, completion) = mpsc::channel();
    thread::scope(|scope| {
        scope.spawn(|| {
            let mut output = Vec::new();
            let mut observer = |event| {
                if let WasmBuildProgressEvent::CargoOutput {
                    stream: OutputStream::Stdout,
                    bytes,
                } = event
                {
                    output.extend(bytes);
                }
            };
            let mut progress = ProgressReporter::new(Some((
                WasmBuildProgressConfig::new()
                    .without_heartbeats()
                    .with_cargo_output(true),
                &mut observer,
            )))
            .unwrap();
            run_cargo_build(
                &spec,
                &root.join("cargo-target"),
                &[],
                &mut progress,
                &mut (spawn_cargo),
            )
            .unwrap();
            assert_eq!(output, b"leader-exited");
            done.send(()).unwrap();
        });
        let completed = completion.recv_timeout(Duration::from_secs(5));
        // Release inherited pipe writers even if the old EOF-first behavior
        // regresses, allowing the scoped worker to finish before refusal.
        fs::write(&release, b"release").unwrap();
        completed.expect("leader cleanup must complete before descendant pipe EOF");
    });
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn observed_cargo_failure_retains_captured_diagnostics_and_exit_event() {
    let root = unique_temp_directory("observed-cargo-failure");
    // Each stream exceeds the pipe buffer and Host communication chunks.
    let expected_stdout = "failure-stdout".repeat(16 * 1024);
    let expected_stderr = "failure-stderr".repeat(16 * 1024);
    fs::write(root.join("stdout"), &expected_stdout).unwrap();
    fs::write(root.join("stderr"), &expected_stderr).unwrap();
    // `sh build` reads the fixture without executing the freshly written file.
    fs::write(
        root.join("build"),
        "set -eu\ncat stdout\ncat stderr >&2\nexit 23\n",
    )
    .expect("write failing observed Cargo fixture");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_cargo_program("/bin/sh");
    for emit in [Some(true), Some(false), None] {
        let mut events = Vec::new();
        let error = {
            let mut observer = |event| events.push(event);
            let mut progress = emit.map_or_else(ProgressReporter::silent, |emit| {
                ProgressReporter::new(Some((
                    WasmBuildProgressConfig::new()
                        .without_heartbeats()
                        .with_cargo_output(emit),
                    &mut observer,
                )))
                .unwrap()
            });

            run_cargo_build(
                &spec,
                &root.join("cargo-target"),
                &[],
                &mut progress,
                &mut (spawn_cargo),
            )
            .expect_err("Cargo fixture must fail")
        };

        assert!(
            matches!(
                &error,
                WasmBuildError::CommandFailed {
                    status,
                    stdout,
                    stderr,
                    ..
                } if status.code() == Some(23)
                    && stdout == &expected_stdout
                    && stderr == &expected_stderr
            ),
            "unexpected Cargo fixture failure: {error:?}"
        );
        assert_eq!(
            events.iter().any(|event| matches!(
                event,
                WasmBuildProgressEvent::CargoFinished {
                    success: false,
                    code: Some(23),
                    ..
                }
            )),
            emit.is_some()
        );
        assert_eq!(
            events
                .iter()
                .any(|event| matches!(event, WasmBuildProgressEvent::CargoOutput { .. })),
            emit == Some(true),
        );
    }
    fs::remove_dir_all(root).expect("remove failing observed Cargo fixture");
}

#[test]
#[cfg(unix)]
fn overflowing_cargo_diagnostics_are_bounded_without_losing_forwarded_output() {
    let root = unique_temp_directory("cargo-truncated-failure");
    let bytes = vec![b'x'; super::CARGO_DIAGNOSTIC_BYTES + 37];
    fs::write(root.join("payload"), &bytes).unwrap();
    fs::write(
        root.join("build"),
        "cat payload; cat payload >&2; exit 23\n",
    )
    .unwrap();
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_cargo_program("/bin/sh");
    for observed in [false, true] {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut observer = |event| {
            if let WasmBuildProgressEvent::CargoOutput { stream, bytes } = event {
                match stream {
                    OutputStream::Stdout => stdout.extend(bytes),
                    OutputStream::Stderr => stderr.extend(bytes),
                }
            }
        };
        let mut progress = if observed {
            ProgressReporter::new(Some((
                WasmBuildProgressConfig::new()
                    .without_heartbeats()
                    .with_cargo_output(true),
                &mut observer,
            )))
            .unwrap()
        } else {
            ProgressReporter::silent()
        };
        let error = run_cargo_build(
            &spec,
            &root.join("target"),
            &[],
            &mut progress,
            &mut (spawn_cargo),
        )
        .unwrap_err();
        let WasmBuildError::CommandFailed {
            status,
            stdout: diagnostic,
            stderr: errors,
            ..
        } = error
        else {
            panic!("expected bounded command diagnostics");
        };
        assert_eq!(status.code(), Some(23));
        for text in [&diagnostic, &errors] {
            assert_eq!(
                &text.as_bytes()[..super::CARGO_DIAGNOSTIC_BYTES],
                &bytes[..super::CARGO_DIAGNOSTIC_BYTES]
            );
            assert!(text[super::CARGO_DIAGNOSTIC_BYTES..].contains("diagnostic truncated"));
            assert!(text.len() < super::CARGO_DIAGNOSTIC_BYTES + 128);
        }
        if observed {
            assert_eq!(stdout, bytes);
            assert_eq!(stderr, bytes);
        } else {
            assert!(stdout.is_empty() && stderr.is_empty());
        }
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn caller_probe_policy_bounds_every_phase_and_revalidation() {
    use super::{CargoInputProbePolicy, resolve_cargo_build_inputs_with_probe_policy};
    use ic_host_process::{
        child::CleanupPolicy,
        tool::{ExecutionFailure, ToolError},
    };
    use std::time::{Duration, Instant};

    for phase in ["--version", "-vV", "metadata"] {
        for cancel in [false, true] {
            let (root, original) = fake_wasm_build_spec("bounded-probes");
            let before = resolve_cargo_build_inputs(&original).unwrap();
            let program = root.join("stall");
            let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
            let cargo = format!("'{}'", cargo.to_string_lossy().replace('\'', "'\"'\"'"));
            write_executable_script(
                &program,
                format!(
                    r#"#!/bin/sh
if [ "$1" = '{phase}' ]; then
    trap 'printf term > terminated; exit 0' TERM
    printf original-stdout
    printf original-stderr >&2
    printf ready > ready
    sh -c 'trap "" TERM; exec sleep 30' & wait
fi
exec {cargo} "$@"
"#
                ),
            )
            .unwrap();
            let spec = if phase == "-vV" {
                let mut environment = original.environment().clone();
                environment.remove(OsStr::new("RUSTC"));
                original
                    .clone()
                    .with_environment(environment)
                    .with_rustc_identity_program(&program)
            } else {
                original.clone().with_cargo_program(&program)
            };
            let mut ready_since = None;
            let mut cancelled = || {
                cancel
                    && root.join("ready").exists()
                    && ready_since.get_or_insert_with(Instant::now).elapsed()
                        >= Duration::from_millis(100)
            };
            let mut policy = CargoInputProbePolicy {
                deadline: Some(Instant::now() + Duration::from_secs(1)),
                cleanup: CleanupPolicy::TermThenKill {
                    grace: Duration::from_millis(100),
                    reap_timeout: Duration::from_secs(2),
                },
                cancelled: &mut cancelled,
            };
            // Revalidation must use the same bounded path, including a stall
            // introduced after the original successful acquisition.
            let error = if cancel {
                before
                    .is_current_with_probe_policy(&spec, &mut policy)
                    .unwrap_err()
            } else {
                resolve_cargo_build_inputs_with_probe_policy(&spec, &mut policy).unwrap_err()
            };
            let WasmBuildError::Io { path, source, .. } = error else {
                panic!("expected retained Host execution evidence");
            };
            assert_eq!(path, program);
            let host = source
                .get_ref()
                .unwrap()
                .downcast_ref::<ToolError>()
                .unwrap();
            let execution = host.execution_error().unwrap();
            assert!(matches!(
                (&execution.failure, cancel),
                (ExecutionFailure::Cancelled, true) | (ExecutionFailure::TimedOut, false)
            ));
            assert_eq!(execution.evidence.stdout, b"original-stdout");
            assert_eq!(execution.evidence.stderr, b"original-stderr");
            assert!(execution.evidence.status.is_some());
            assert!(execution.cleanup.is_none());
            // The leader may be escalated to KILL before its TERM trap finishes
            // under load; a reaped status and original evidence are required.
            // The elapsed policy stays expired; it must not restart a timeout
            // or invoke the next command during revalidation.
            fs::remove_file(root.join("ready")).unwrap();
            policy.deadline = Some(Instant::now());
            assert!(
                before
                    .is_current_with_probe_policy(&spec, &mut policy)
                    .is_err()
            );
            assert!(!root.join("ready").exists());
            fs::remove_dir_all(root).unwrap();
        }
    }
}

#[test]
#[cfg(unix)]
fn controlled_probes_keep_complete_output_bounds_and_spawn_causes() {
    use super::CargoInputProbePolicy;
    use ic_host_process::{
        child::CleanupPolicy,
        tool::{ExecutionFailure, ToolError},
    };
    use std::{io, process::Command};

    let root = unique_temp_directory("controlled-probe-output");
    let payload = root.join("payload");
    fs::write(&payload, vec![b'x'; super::TOOL_IDENTITY_BYTES + 1]).unwrap();
    let mut cancelled = || false;
    let mut policy = CargoInputProbePolicy {
        deadline: None,
        cleanup: CleanupPolicy::KillAndWait,
        cancelled: &mut cancelled,
    };
    let mut command = Command::new("/bin/cat");
    command.arg(&payload);
    let phase = super::WasmBuildPhase::CargoIdentity;
    let error = super::capture_probe_output_with_policy(
        &mut command,
        phase,
        super::TOOL_IDENTITY_BYTES,
        &mut Some(&mut policy),
    )
    .unwrap_err();
    let WasmBuildError::Io { source, .. } = error else {
        panic!("expected Host evidence")
    };
    let host = source
        .get_ref()
        .unwrap()
        .downcast_ref::<ToolError>()
        .unwrap();
    assert!(matches!(
        host.execution_error().unwrap().failure,
        ExecutionFailure::OutputLimit { .. }
    ));
    assert_eq!(
        host.evidence().unwrap().stdout.len(),
        super::TOOL_IDENTITY_BYTES
    );
    let mut missing = Command::new(root.join("missing"));
    assert!(
        matches!(super::capture_probe_output_with_policy(&mut missing, phase, super::TOOL_IDENTITY_BYTES, &mut Some(&mut policy)), Err(WasmBuildError::CommandSpawn { source, .. }) if source.kind() == io::ErrorKind::NotFound)
    );
    for cancelled in [true, false] {
        let marker = root.join("unexpected-start");
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", "printf started > \"$1\"", "probe"])
            .arg(&marker);
        let mut cancellation = || cancelled;
        let mut policy = CargoInputProbePolicy {
            deadline: Some(std::time::Instant::now()),
            cleanup: CleanupPolicy::KillAndWait,
            cancelled: &mut cancellation,
        };
        let error = super::capture_probe_output_with_policy(
            &mut command,
            phase,
            super::TOOL_IDENTITY_BYTES,
            &mut Some(&mut policy),
        )
        .unwrap_err();
        let WasmBuildError::Io { source, .. } = error else {
            panic!("expected Host pre-spawn refusal")
        };
        let host = source
            .get_ref()
            .unwrap()
            .downcast_ref::<ToolError>()
            .unwrap();
        assert!(matches!(
            (&host.execution_error().unwrap().failure, cancelled),
            (ExecutionFailure::Cancelled, true) | (ExecutionFailure::TimedOut, false)
        ));
        assert!(!marker.exists(), "refused probe must not execute");
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn complete_probe_limits_preserve_status_spawn_causes_and_refuse_partial_data() {
    use ic_host_process::tool::{ExecutionFailure, ToolError};
    use std::{io, process::Command};

    let root = unique_temp_directory("complete-probe-limits");
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_cargo_program("/bin/sh");
    let identity = vec![b'x'; super::TOOL_IDENTITY_BYTES];
    fs::write(root.join("payload"), &identity).unwrap();
    let phase = super::WasmBuildPhase::CargoIdentity;
    assert_eq!(
        super::command_identity(
            &spec,
            phase,
            std::ffi::OsStr::new("/bin/sh"),
            &["-c", "cat payload"]
        )
        .unwrap(),
        identity
    );
    for script in ["cat payload; printf x", "cat payload >&2; printf x >&2"] {
        let error = super::command_identity(
            &spec,
            phase,
            std::ffi::OsStr::new("/bin/sh"),
            &["-c", script],
        )
        .unwrap_err();
        let WasmBuildError::Io { source, .. } = error else {
            panic!("overflow must retain Host evidence");
        };
        let host = source
            .get_ref()
            .unwrap()
            .downcast_ref::<ToolError>()
            .unwrap();
        assert!(matches!(
            host.execution_error().unwrap().failure,
            ExecutionFailure::OutputLimit { .. }
        ));
    }
    let mut failed = Command::new("/bin/sh");
    failed.args(["-c", "printf diagnostic >&2; exit 23"]);
    let output =
        super::capture_probe_output(&mut failed, phase, super::TOOL_IDENTITY_BYTES).unwrap();
    assert!(
        matches!(super::ensure_command_success(phase, output), Err(WasmBuildError::CommandFailed { status, stderr, .. }) if status.code() == Some(23) && stderr == "diagnostic")
    );
    let mut missing = Command::new(root.join("absent"));
    assert!(
        matches!(super::capture_probe_output(&mut missing, phase, super::TOOL_IDENTITY_BYTES), Err(WasmBuildError::CommandSpawn { source, .. }) if source.kind() == io::ErrorKind::NotFound)
    );

    // A valid complete JSON value followed by excess whitespace must not be
    // parsed successfully from a captured prefix.
    fs::write(root.join("metadata"), "cat payload\n").unwrap();
    let mut metadata = br#"{"packages":[],"workspace_members":[],"workspace_root":"/workspace","target_directory":"/target","version":1}"#.to_vec();
    metadata.resize(super::CARGO_METADATA_BYTES, b' ');
    fs::write(root.join("payload"), &metadata).unwrap();
    assert_eq!(super::cargo_metadata(&spec).unwrap().packages, []);
    metadata.push(b' ');
    fs::write(root.join("payload"), &metadata).unwrap();
    assert!(matches!(
        super::cargo_metadata(&spec),
        Err(WasmBuildError::Io { .. })
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cache_directory_tag_is_created_at_target_root() {
    let target_dir = unique_temp_directory("cache-directory-tag");
    fs::write(target_dir.join("CACHEDIR.TAG"), "not a cache tag").expect("write invalid cache tag");

    ensure_cache_directory_tag(&target_dir).expect("write valid cache tag");

    let contents =
        fs::read_to_string(target_dir.join("CACHEDIR.TAG")).expect("read cache directory tag");
    assert!(contents.starts_with(ic_host_fs::cache_tag::SIGNATURE));
    fs::remove_dir_all(target_dir).expect("remove tag test directory");
}

#[test]
fn failed_build_removes_its_incomplete_fingerprint_directory() {
    let target_dir = unique_temp_directory("failed-build-cleanup");
    let fingerprint_dir = target_dir.join("a".repeat(64));
    fs::create_dir_all(&fingerprint_dir).expect("create incomplete target directory");
    fs::write(fingerprint_dir.join("partial-output"), b"partial").expect("write incomplete output");
    let failure: Result<WasmBuildOutcome, WasmBuildError> = Err(WasmBuildError::InvalidSpec {
        message: "synthetic build failure".to_owned(),
    });

    let mut progress = ProgressReporter::silent();
    let result = finish_fingerprint_build(
        failure,
        OwnedDirectory::new(fingerprint_dir.clone()),
        &mut progress,
    );

    assert!(matches!(result, Err(WasmBuildError::InvalidSpec { .. })));
    assert!(progress.failure_timings.cleanup().is_some());
    assert!(!fingerprint_dir.exists());
    fs::remove_dir_all(target_dir).expect("remove cleanup test directory");
}

#[test]
#[cfg(unix)]
fn oversized_wasm_cache_stamp_requires_a_rebuild() {
    let (root, spec) = fake_wasm_build_spec("oversized-wasm-cache-stamp");
    let cold =
        build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).expect("build cold artifact");
    let fingerprint = cold.record().fingerprint();
    let cached = cold.record().artifacts()[0].clone();
    let original = fs::read(&cached).expect("read original artifact");
    let stamp_path = artifact_stamp_path(&cached);
    let stamp = fs::read(&stamp_path).expect("read original stamp");
    // Release the retained entry before simulating external cache damage.
    drop(cold);
    fs::remove_file(&stamp_path).expect("remove read-only stamp");
    fs::write(&stamp_path, stamp).expect("write matching stamp prefix");
    fs::OpenOptions::new()
        .write(true)
        .open(&stamp_path)
        .expect("open oversized stamp")
        .set_len(1024 * 1024 * 1024)
        .expect("extend oversized stamp");

    let recovered = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
        .expect("rebuild invalid exact entry");
    assert!(!recovered.is_reused());
    assert!(recovered.record().timings().cargo_build().is_some());
    assert_eq!(
        fs::read(&recovered.record().artifacts()[0]).unwrap(),
        original
    );
    assert!(
        validated_artifact_set(artifact_pairs(recovered.record().artifacts()), fingerprint)
            .is_some()
    );
    drop(recovered);
    fs::remove_dir_all(root).expect("remove recovery fixture");
}

#[test]
#[cfg(unix)]
fn warm_wasm_outputs_follow_retained_entry_and_preserve_matching_files() {
    for mode in ["isolated", "shared", "scheduled"] {
        let (root, mut spec) = fake_wasm_build_spec("warm-wasm-publication");
        if mode != "isolated" {
            spec = spec.with_shared_incremental_target(root.join("incremental"));
        }
        if mode == "scheduled" {
            spec = spec.with_shared_incremental_target_maintenance_at_most_every(
                CacheRetentionLimits::new(),
                Duration::from_secs(60),
            );
        }
        let public = expected_artifacts(&spec, &spec.target_dir);
        fs::create_dir_all(public[0].parent().unwrap()).unwrap();
        let original = b"\0asm\x01\0\0\0";
        fs::write(&public[0], original).unwrap();
        let before_cold = fs::metadata(&public[0]).unwrap().ino();
        let cold = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
        assert!(!cold.is_reused());
        assert_ne!(before_cold, fs::metadata(&public[0]).unwrap().ino());

        // Both stamps are valid for this fingerprint, but their bytes differ.
        fs::write(&public[0], b"stamped!").unwrap();
        publish_artifact_stamps(artifact_pairs(&public), cold.record().fingerprint()).unwrap();
        let reused = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
        assert!(reused.is_reused());
        assert!(reused.record().timings().cargo_build().is_none());
        assert_eq!(fs::read(&public[0]).unwrap(), original, "{mode}");
        assert_eq!(fs::read(&reused.record().artifacts()[0]).unwrap(), original);

        let stamp = artifact_stamp_path(&public[0]);
        let expected_stamp = fs::read(&stamp).unwrap();
        let file_identity = fs::metadata(&public[0]).unwrap().ino();
        for damaged_stamp in [None, Some(b"invalid".as_slice())] {
            if let Some(bytes) = damaged_stamp {
                fs::write(&stamp, bytes).unwrap();
            } else {
                fs::remove_file(&stamp).unwrap();
            }
            assert!(
                build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
                    .unwrap()
                    .is_reused()
            );
            assert_eq!(fs::metadata(&public[0]).unwrap().ino(), file_identity);
            assert_eq!(fs::read(&stamp).unwrap(), expected_stamp);
        }

        let time = UNIX_EPOCH + Duration::from_secs(3600);
        for path in [&public[0], &stamp] {
            fs::File::open(path)
                .unwrap()
                .set_times(FileTimes::new().set_modified(time))
                .unwrap();
        }
        let stamp_identity = fs::metadata(&stamp).unwrap().ino();
        assert!(
            build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
                .unwrap()
                .is_reused()
        );
        assert_eq!(fs::metadata(&public[0]).unwrap().ino(), file_identity);
        assert_eq!(fs::metadata(&public[0]).unwrap().modified().unwrap(), time);
        assert_eq!(fs::metadata(&stamp).unwrap().ino(), stamp_identity);
        assert_eq!(fs::metadata(&stamp).unwrap().modified().unwrap(), time);

        fs::remove_file(&public[0]).unwrap();
        assert!(
            build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
                .unwrap()
                .is_reused()
        );
        assert_eq!(fs::read(&public[0]).unwrap(), original);
        assert_eq!(fs::metadata(&stamp).unwrap().ino(), stamp_identity);
        drop(reused);
        drop(cold);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
#[cfg(unix)]
fn warm_wasm_publication_detaches_linked_or_restricted_outputs_and_stamps() {
    let (root, spec) = fake_wasm_build_spec("warm-wasm-ownership");
    let cold = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
    let public = expected_artifacts(&spec, &spec.target_dir);
    let cached = &cold.record().artifacts()[0];
    let public_stamp = artifact_stamp_path(&public[0]);
    let cached_stamp = artifact_stamp_path(cached);
    let expected = fs::read(cached).unwrap();
    let expected_stamp = fs::read(&cached_stamp).unwrap();
    for case in ["symlink", "hardlink", "readonly", "executable"] {
        for (source, destination) in [(cached, &public[0]), (&cached_stamp, &public_stamp)] {
            fs::remove_file(destination).unwrap();
            match case {
                "symlink" => symlink(source, destination).unwrap(),
                "hardlink" => fs::hard_link(source, destination).unwrap(),
                _ => {
                    fs::copy(source, destination).unwrap();
                    let mode = if case == "readonly" { 0o444 } else { 0o755 };
                    fs::set_permissions(destination, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }
        assert!(
            build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
                .unwrap()
                .is_reused()
        );
        for destination in [&public[0], &public_stamp] {
            let metadata = fs::symlink_metadata(destination).unwrap();
            assert!(metadata.is_file(), "{case}");
            assert_eq!(metadata.nlink(), 1, "{case}");
            assert_eq!(metadata.mode() & 0o600, 0o600, "{case}");
            assert_eq!(metadata.mode() & 0o7111, 0, "{case}");
        }
        assert_eq!(fs::read(&public[0]).unwrap(), expected);
        assert_eq!(fs::read(&public_stamp).unwrap(), expected_stamp);
        fs::write(&public[0], b"changed").unwrap();
        fs::write(&public_stamp, b"changed").unwrap();
        assert_eq!(fs::read(cached).unwrap(), expected);
        assert_eq!(fs::read(&cached_stamp).unwrap(), expected_stamp);
    }
    drop(cold);
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn warm_wasm_materialization_repairs_only_changed_members() {
    let root = unique_temp_directory("selective-wasm-materialization");
    let cached = [root.join("first.wasm"), root.join("second.wasm")];
    fs::write(&cached[0], b"first").unwrap();
    fs::write(&cached[1], b"second").unwrap();
    let fingerprint = digest_bytes("wasm-materialization-test-v1", b"fixture");
    let public = [
        root.join("public-first.wasm"),
        root.join("public-second.wasm"),
    ];
    let paths = cached
        .iter()
        .cloned()
        .zip(public.iter().cloned())
        .collect::<Vec<_>>();
    let info = publish_artifact_stamps(paths.clone(), fingerprint).unwrap();
    fs::write(&public[0], b"first").unwrap();
    fs::write(&public[1], b"wrong!").unwrap();
    publish_artifact_stamps(artifact_pairs(&public), fingerprint).unwrap();
    let stamp = artifact_stamp_path(&public[0]);
    let time = UNIX_EPOCH + Duration::from_secs(3600);
    let identities = [&public[0], &stamp].map(|path| {
        fs::File::open(path)
            .unwrap()
            .set_times(FileTimes::new().set_modified(time))
            .unwrap();
        fs::metadata(path).unwrap().ino()
    });

    materialize_artifacts(&info, fingerprint, true).unwrap();

    assert_eq!(fs::read(&public[0]).unwrap(), b"first");
    assert_eq!(fs::read(&public[1]).unwrap(), b"second");
    assert_eq!(validated_artifact_set(paths, fingerprint).unwrap(), info);
    let published = validated_artifact_set(artifact_pairs(&public), fingerprint).unwrap();
    assert_eq!(
        published
            .iter()
            .map(|artifact| artifact.info)
            .collect::<Vec<_>>(),
        info.iter()
            .map(|artifact| artifact.info)
            .collect::<Vec<_>>()
    );
    for (path, identity) in [&public[0], &stamp].into_iter().zip(identities) {
        assert_eq!(fs::metadata(path).unwrap().ino(), identity);
        assert_eq!(fs::metadata(path).unwrap().modified().unwrap(), time);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_wasm_materialization_does_not_stamp_a_partial_public_set() {
    let root = unique_temp_directory("partial-wasm-materialization");
    let cached = [root.join("first.wasm"), root.join("second.wasm")];
    for path in &cached {
        fs::write(path, b"wasm").unwrap();
    }
    let fingerprint = digest_bytes("wasm-materialization-test-v1", b"fixture");
    fs::write(root.join("blocked"), b"file").unwrap();
    let public = [
        root.join("public/first.wasm"),
        root.join("blocked/second.wasm"),
    ];
    let paths = cached.into_iter().zip(public.iter().cloned()).collect();
    let info = publish_artifact_stamps(paths, fingerprint).unwrap();
    for preserve_matching in [false, true] {
        let error = materialize_artifacts(&info, fingerprint, preserve_matching)
            .expect_err("second destination must fail before any stamp is published");
        assert!(matches!(
            error,
            WasmBuildError::Io {
                operation: "publish Wasm artifact",
                ..
            }
        ));
        assert_eq!(fs::read(&public[0]).unwrap(), b"wasm");
        assert!(!artifact_stamp_path(&public[0]).exists());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn warm_caller_artifacts_do_not_replace_missing_or_retained_invalid_exact_entries() {
    for mode in ["isolated", "shared", "scheduled"] {
        let (root, mut spec) = fake_wasm_build_spec("missing-exact-entry");
        if mode != "isolated" {
            spec = spec.with_shared_incremental_target(root.join("incremental"));
        }
        if mode == "scheduled" {
            spec = spec.with_shared_incremental_target_maintenance_at_most_every(
                CacheRetentionLimits::new(),
                Duration::from_secs(60),
            );
        }
        let built = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
            .expect("build initial exact entry");
        let entry = built.record().exact_cache_path().to_owned();
        let expected = fs::read(&built.record().artifacts()[0]).expect("read initial Wasm");
        let public = expected_artifacts(&spec, &spec.target_dir);
        // Even a completely stamped public set cannot supply a replacement entry.
        fs::write(&public[0], b"public replacement").unwrap();
        publish_artifact_stamps(artifact_pairs(&public), built.record().fingerprint()).unwrap();
        drop(built);
        fs::remove_dir_all(&entry).expect("remove exact entry while keeping caller artifacts");

        let rebuilt = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
            .expect("rebuild missing exact entry");
        assert!(!rebuilt.is_reused());
        assert!(rebuilt.record().timings().cargo_build().is_some());
        assert_eq!(rebuilt.record().exact_cache_path(), entry);
        assert_eq!(
            fs::read(&rebuilt.record().artifacts()[0]).unwrap(),
            expected
        );
        fs::write(&rebuilt.record().artifacts()[0], b"corrupt").unwrap();
        let error = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
            .expect_err("an invalid retained entry must not be replaced");
        assert!(matches!(
            error,
            WasmBuildError::Io {
                operation: "replace retained cache entry",
                ..
            }
        ));
        assert_eq!(fs::read(&public[0]).unwrap(), expected);
        drop(rebuilt);
        let recovered = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
            .expect("rebuild the invalid entry once its consumer releases it");
        assert!(!recovered.is_reused());
        assert!(recovered.record().timings().cargo_build().is_some());
        assert_eq!(
            fs::read(&recovered.record().artifacts()[0]).unwrap(),
            expected
        );
        drop(recovered);
        assert!(
            entry.is_dir(),
            "rebuilt exact entry must survive record drop"
        );
        fs::remove_dir_all(root).expect("remove rebuild fixture");
    }
}

#[test]
fn age_pruning_removes_only_stale_fingerprint_directories() {
    let target_dir = unique_temp_directory("age-pruning");
    let cache_root = target_dir.join(".ic-testkit/wasm-targets");
    let old = create_cache_entry(&cache_root, 'a', 10, UNIX_EPOCH + Duration::from_secs(1));
    let current = create_cache_entry(&cache_root, 'b', 10, SystemTime::now());
    let unrelated = cache_root.join("not-a-fingerprint");
    fs::create_dir_all(&unrelated).expect("create unrelated directory");

    let report = prune_wasm_build_cache(
        &target_dir,
        CacheRetentionLimits::new()
            .with_max_age(Duration::from_secs(60))
            .with_max_size_bytes(u64::MAX),
    )
    .expect("prune old cache entry");

    assert_eq!(report.entries_scanned(), 2);
    assert_eq!(report.entries_removed(), 1);
    assert_eq!(report.entries_retained(), 1);
    assert!(!old.exists());
    assert!(current.exists());
    assert!(unrelated.exists());
    assert!(target_dir.join("CACHEDIR.TAG").is_file());
    fs::remove_dir_all(target_dir).expect("remove age-pruning test directory");
}

#[test]
fn size_pruning_removes_least_recently_used_entries_first() {
    let target_dir = unique_temp_directory("size-pruning");
    let cache_root = target_dir.join(".ic-testkit/wasm-targets");
    let oldest = create_cache_entry(&cache_root, 'a', 10, UNIX_EPOCH + Duration::from_secs(1));
    let middle = create_cache_entry(&cache_root, 'b', 10, UNIX_EPOCH + Duration::from_secs(2));
    let newest = create_cache_entry(&cache_root, 'c', 10, UNIX_EPOCH + Duration::from_secs(3));
    let newest_bytes = directory_logical_size(&newest).expect("measure newest entry");
    let all_bytes = [&oldest, &middle, &newest]
        .into_iter()
        .map(|entry| directory_logical_size(entry).unwrap())
        .sum::<u64>();
    for limit in [all_bytes, all_bytes + 1] {
        let report = prune_wasm_build_cache(
            &target_dir,
            CacheRetentionLimits::new().with_max_size_bytes(limit),
        )
        .expect("retain entries within the size budget");
        assert_eq!(report.entries_scanned(), 3);
        assert_eq!(report.entries_removed(), 0);
        assert_eq!(report.bytes_before(), all_bytes);
        assert_eq!(report.bytes_retained(), all_bytes);
        assert!(oldest.exists() && middle.exists() && newest.exists());
    }

    let report = prune_wasm_build_cache(
        &target_dir,
        CacheRetentionLimits::new().with_max_size_bytes(newest_bytes),
    )
    .expect("prune cache to size");

    assert_eq!(report.entries_scanned(), 3);
    assert_eq!(report.entries_removed(), 2);
    assert_eq!(report.entries_retained(), 1);
    assert!(report.bytes_retained() <= newest_bytes);
    assert!(!oldest.exists());
    assert!(!middle.exists());
    assert!(newest.exists());
    fs::remove_dir_all(target_dir).expect("remove size-pruning test directory");
}

#[test]
fn in_build_pruning_protects_the_active_fingerprint() {
    let target_dir = unique_temp_directory("protected-pruning");
    let cache_root = target_dir.join(".ic-testkit/wasm-targets");
    let stale = create_cache_entry(&cache_root, 'a', 10, UNIX_EPOCH + Duration::from_secs(1));
    let active = create_cache_entry(&cache_root, 'b', 10, UNIX_EPOCH + Duration::from_secs(2));

    let report = prune_wasm_build_cache_locked(
        &target_dir,
        CacheRetentionLimits::new()
            .with_max_age(Duration::ZERO)
            .with_max_size_bytes(0),
        Some(&active),
    )
    .expect("prune while protecting active cache entry");

    assert_eq!(report.entries_scanned(), 2);
    assert_eq!(report.entries_removed(), 1);
    assert!(!stale.exists());
    assert!(active.exists());
    assert!(report.bytes_retained() > 0);
    fs::remove_dir_all(target_dir).expect("remove protected-pruning test directory");
}

#[test]
fn cargo_configuration_discovery_matches_cargo_search_and_include_rules() {
    let root = unique_temp_directory("cargo-configuration-discovery");
    let workspace = root.join("workspace");
    let workspace_cargo = workspace.join(".cargo");
    let ancestor_cargo = root.join(".cargo");
    let cargo_home = root.join("cargo-home");
    fs::create_dir_all(&workspace_cargo).expect("create workspace Cargo directory");
    fs::create_dir_all(&ancestor_cargo).expect("create ancestor Cargo directory");
    fs::create_dir_all(&cargo_home).expect("create Cargo home");

    fs::write(
        workspace_cargo.join("config"),
        "include = [\"included.toml\", { path = \"missing.toml\", optional = true }]\n",
    )
    .expect("write effective workspace Cargo config");
    fs::write(
        workspace_cargo.join("config.toml"),
        "[build]\ntarget-dir = \"ignored-by-cargo\"\n",
    )
    .expect("write shadowed workspace Cargo config");
    fs::write(
        workspace_cargo.join("included.toml"),
        "include = \"nested.toml\"\n",
    )
    .expect("write included Cargo config");
    fs::write(
        workspace_cargo.join("nested.toml"),
        "[build]\nincremental = false\n",
    )
    .expect("write nested Cargo config");
    fs::write(
        ancestor_cargo.join("config.toml"),
        "[net]\noffline = true\n",
    )
    .expect("write ancestor Cargo config");
    fs::write(cargo_home.join("config"), "[term]\nquiet = true\n")
        .expect("write Cargo-home config");

    let cargo_home_text = cargo_home.to_str().expect("temporary path is UTF-8");
    let spec = WasmBuildSpec::new(
        &workspace,
        &root.join("target"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_environment(crate::cargo_fixture::environment_with([(
        "CARGO_HOME",
        cargo_home_text,
    )]));
    let mut inputs = Vec::new();
    append_cargo_configuration_inputs(&mut inputs, &spec, &workspace)
        .expect("discover effective Cargo configuration");
    let paths = inputs
        .into_iter()
        .map(|(_, path)| path)
        .collect::<BTreeSet<_>>();

    assert!(paths.contains(&canonical_fixture(&workspace_cargo.join("config"))));
    assert!(paths.contains(&canonical_fixture(&workspace_cargo.join("included.toml"))));
    assert!(paths.contains(&canonical_fixture(&workspace_cargo.join("nested.toml"))));
    assert!(paths.contains(&canonical_fixture(&ancestor_cargo.join("config.toml"))));
    // Explicit Cargo home lookups retain the caller's path, including directory
    // aliases, so later replacement of an alias remains observable.
    assert!(paths.contains(&cargo_home.join("config")));
    assert!(!paths.contains(&canonical_fixture(&workspace_cargo.join("config.toml"))));
    assert_eq!(paths.len(), 5);
    fs::remove_dir_all(root).expect("remove Cargo-configuration test directory");
}

#[test]
#[cfg(unix)]
fn cargo_configuration_symlinks_preserve_include_locations_and_mutation_guards() {
    let root = unique_temp_directory("cargo-configuration-symlinks");
    let workspace = root.join("workspace");
    let workspace_cargo = workspace.join(".cargo");
    let ancestor_cargo = root.join(".cargo");
    let external = root.join("external");
    for directory in [&workspace_cargo, &ancestor_cargo, &external] {
        fs::create_dir_all(directory).unwrap();
    }
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    write_projection_package(&workspace, "fixture", "");
    let shared_config = external.join("config.toml");
    fs::write(&shared_config, "include = [\"fragment.toml\"]\n").unwrap();
    let workspace_config = workspace_cargo.join("config.toml");
    let ancestor_config = ancestor_cargo.join("config.toml");
    symlink(&shared_config, &workspace_config).unwrap();
    symlink(&shared_config, &ancestor_config).unwrap();
    let workspace_fragment = workspace_cargo.join("fragment.toml");
    let ancestor_fragment = ancestor_cargo.join("fragment.toml");
    let fragments = [&workspace_fragment, &ancestor_fragment];
    for fragment in fragments {
        fs::write(fragment, "[build]\nrustflags = []\n").unwrap();
    }
    // This decoy exists so resolving beside the referent can silently watch
    // the wrong input instead of failing on a missing include.
    let decoy = external.join("fragment.toml");
    fs::write(&decoy, "[build]\nrustflags = []\n").unwrap();
    let spec = WasmBuildSpec::new(
        &workspace,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_environment(crate::cargo_fixture::environment_with([(
        "CARGO_HOME",
        &ancestor_cargo,
    )]));
    let snapshot = resolve_cargo_build_inputs(&spec).expect("resolve symlinked Cargo configs");
    assert_eq!(
        snapshot
            .inputs()
            .iter()
            .map(super::CargoBuildInput::path)
            .collect::<BTreeSet<_>>()
            .len(),
        snapshot.inputs().len(),
        "Cargo home and ancestor discovery must not duplicate the same lookup path",
    );
    assert!(snapshot.is_content_current().unwrap());
    for fragment in fragments {
        fs::write(fragment, "[build]\nrustflags = ['--cfg=changed']\n").unwrap();
        assert!(
            !snapshot.is_content_current().unwrap(),
            "Cargo's include at {} must be watched",
            fragment.display(),
        );
        fs::write(fragment, "[build]\nrustflags = []\n").unwrap();
        assert!(snapshot.is_content_current().unwrap());
    }
    fs::write(&decoy, "[build]\nrustflags = ['--cfg=unused']\n").unwrap();
    assert!(snapshot.is_content_current().unwrap());

    // Retargeting the configured entry must be observed, even while the
    // previous referent remains unchanged.
    let replacement = external.join("replacement.toml");
    fs::write(&replacement, "[build]\nrustflags = ['--cfg=replaced']\n").unwrap();
    fs::remove_file(&workspace_config).unwrap();
    symlink(&replacement, &workspace_config).unwrap();
    assert!(!snapshot.is_content_current().unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn cargo_configuration_guards_follow_each_directory_alias() {
    let root = unique_temp_directory("cargo-configuration-directory-aliases");
    let workspace = root.join("workspace");
    let cargo_dir = workspace.join(".cargo");
    fs::create_dir_all(&cargo_dir).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"fixture\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    write_projection_package(&workspace, "fixture", "");
    fs::write(
        cargo_dir.join("config.toml"),
        "include = [\"first/config.toml\", \"second/config.toml\"]\n",
    )
    .unwrap();
    let original = root.join("original");
    let replacement = root.join("replacement");
    for directory in [&original, &replacement] {
        fs::create_dir_all(directory).unwrap();
        fs::write(
            directory.join("config.toml"),
            "include = [\"fragment.toml\"]\n",
        )
        .unwrap();
        fs::write(directory.join("fragment.toml"), "[build]\nrustflags = []\n").unwrap();
    }
    symlink(&original, cargo_dir.join("first")).unwrap();
    let second = cargo_dir.join("second");
    symlink(&original, &second).unwrap();
    let spec = WasmBuildSpec::new(
        &workspace,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_environment(crate::cargo_fixture::environment_with([(
        "CARGO_HOME",
        root.join("cargo-home"),
    )]));
    let snapshot = resolve_cargo_build_inputs(&spec).unwrap();
    assert!(snapshot.is_content_current().unwrap());
    fs::write(
        replacement.join("fragment.toml"),
        "[build]\nrustflags = ['--cfg=replaced']\n",
    )
    .unwrap();
    assert!(snapshot.is_content_current().unwrap());
    fs::remove_file(&second).unwrap();
    symlink(&replacement, &second).unwrap();
    assert!(
        !snapshot.is_content_current().unwrap(),
        "the second alias's nested include must be guarded even when config bytes match",
    );

    // Directory-alias cycles must terminate discovery even for callers using
    // a custom Cargo program that does not reject them in metadata.
    fs::write(
        cargo_dir.join("config.toml"),
        "include = [\"loop/config.toml\"]\n",
    )
    .unwrap();
    symlink(&cargo_dir, cargo_dir.join("loop")).unwrap();
    append_cargo_configuration_inputs(&mut Vec::new(), &spec, &workspace).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn required_cargo_configuration_include_is_an_exact_input() {
    let root = unique_temp_directory("required-cargo-configuration-include");
    let workspace = root.join("workspace");
    let cargo_dir = workspace.join(".cargo");
    fs::create_dir_all(&cargo_dir).expect("create workspace Cargo directory");
    fs::write(
        cargo_dir.join("config.toml"),
        "include = \"missing.toml\"\n",
    )
    .expect("write Cargo config");
    let isolated_home = root.join("isolated-cargo-home");
    let isolated_home_text = isolated_home.to_str().expect("temporary path is UTF-8");
    let spec = WasmBuildSpec::new(
        &workspace,
        &root.join("target"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_environment(crate::cargo_fixture::environment_with([(
        "CARGO_HOME",
        isolated_home_text,
    )]));

    let error = append_cargo_configuration_inputs(&mut Vec::new(), &spec, &workspace)
        .expect_err("required missing include must fail input discovery");

    assert!(matches!(error, WasmBuildError::Io { .. }));
    fs::remove_dir_all(root).expect("remove required-include test directory");
}

fn create_cache_entry(
    cache_root: &Path,
    fingerprint_digit: char,
    payload_bytes: usize,
    last_used: SystemTime,
) -> PathBuf {
    let path = cache_root.join(fingerprint_digit.to_string().repeat(64));
    fs::create_dir_all(&path).expect("create cache entry");
    fs::write(path.join("payload"), vec![0; payload_bytes]).expect("write cache payload");
    write_last_used(&path, last_used).expect("write cache use time");
    path
}

#[cfg(unix)]
#[test]
fn warm_hits_reject_source_mutation_for_active_and_materialized_artifacts() {
    for mode in ["isolated", "shared", "scheduled"] {
        for active_artifact in [true, false] {
            let (root, mut spec) = fake_wasm_build_spec("warm-hit-source-race");
            if mode != "isolated" {
                spec = spec.with_shared_incremental_target(root.join("incremental"));
            }
            if mode == "scheduled" {
                spec = spec.with_shared_incremental_target_maintenance_at_most_every(
                    CacheRetentionLimits::new(),
                    Duration::from_secs(60),
                );
            }
            let cold = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
                .expect("build initial artifacts");
            assert!(
                build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct)
                    .expect("unchanged hit")
                    .is_reused()
            );
            if !active_artifact {
                fs::remove_file(&expected_artifacts(&spec, &spec.target_dir)[0])
                    .expect("remove caller artifact");
            }
            let mut mutated = false;
            let result = crate::artifacts::build_wasm_canisters_cached_with_progress(
                &spec,
                WasmBuildProgressConfig::new(),
                &mut |event| {
                    if !mutated && matches!(event, WasmBuildProgressEvent::InputsResolved { .. }) {
                        fs::write(
                            root.join("fixture/src/lib.rs"),
                            "pub fn value() -> u8 { 2 }\n",
                        )
                        .expect("mutate source during acquisition");
                        mutated = true;
                    }
                },
                spawn_cargo,
            );
            assert!(mutated);
            assert!(
                matches!(
                    result,
                    Err(WasmBuildError::InputsChangedDuringAcquisition { .. })
                ),
                "{mode}, active={active_artifact}: {result:?}"
            );
            drop(cold);
            fs::remove_dir_all(root).expect("remove warm-hit race fixture");
        }
    }
}

#[test]
#[cfg(unix)]
fn observed_cargo_panic_stops_compiler_descendants() {
    let root = unique_temp_directory("observed-cargo-panic");
    let pids = root.join("pids");
    fs::write(
        root.join("build"),
        "set -eu\nsleep 30 &\nprintf '%s %s' \"$$\" \"$!\" > pids\nprintf ready\nwait\n",
    )
    .unwrap();
    let spec = WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_cargo_program("/bin/sh");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut observer = |event| {
            if matches!(event, WasmBuildProgressEvent::CargoOutput { .. }) {
                panic!("synthetic Cargo observer panic");
            }
        };
        let mut progress = ProgressReporter::new(Some((
            WasmBuildProgressConfig::new().with_cargo_output(true),
            &mut observer,
        )))
        .unwrap();
        run_cargo_build(
            &spec,
            &root.join("cargo-target"),
            &[],
            &mut progress,
            &mut (spawn_cargo),
        )
        .unwrap();
    }));
    assert!(result.is_err(), "observer must panic");
    for pid in fs::read_to_string(pids).unwrap().split_whitespace() {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let output = std::process::Command::new("/bin/ps")
                .args(["-p", pid, "-o", "stat="])
                .output()
                .unwrap();
            assert!(output.status.success() || output.status.code() == Some(1));
            let state = String::from_utf8(output.stdout).unwrap();
            if state.trim().is_empty() || state.trim().starts_with('Z') {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "Cargo process {pid} remains running"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn optional_duration_addition_preserves_absence_and_saturates() {
    for (left, right, expected) in [
        (None, None, None),
        (Some(Duration::ZERO), None, Some(Duration::ZERO)),
        (
            None,
            Some(Duration::from_secs(2)),
            Some(Duration::from_secs(2)),
        ),
        (
            Some(Duration::from_secs(2)),
            None,
            Some(Duration::from_secs(2)),
        ),
        (
            Some(Duration::MAX),
            Some(Duration::from_nanos(1)),
            Some(Duration::MAX),
        ),
    ] {
        let left = super::WasmBuildTimings {
            shared_incremental_lock_wait: left,
            cargo_build: left,
            cache_maintenance: left,
            ..Default::default()
        };
        let right = super::WasmBuildTimings {
            shared_incremental_lock_wait: right,
            cargo_build: right,
            cache_maintenance: right,
            ..Default::default()
        };
        let total = left.saturating_add(right);
        assert_eq!(
            [
                total.shared_incremental_lock_wait(),
                total.cargo_build(),
                total.cache_maintenance()
            ],
            [expected; 3]
        );
    }
}

#[cfg(unix)]
#[test]
fn cargo_artifact_protocol_refuses_bad_evidence_after_child_completion() {
    for case in [
        "missing",
        "package",
        "kind",
        "relative",
        "duplicate",
        "ambiguous",
        "no-wasm",
        "crate-type",
        "malformed",
        "oversized",
        "failed",
    ] {
        let (root, mut spec) = fake_wasm_build_spec(&format!("cargo-message-{case}"));
        let resolved = resolve_cargo_build_inputs(&spec).unwrap();
        let wrapper = root.join("cargo.sh");
        let original = fs::read_to_string(&wrapper).unwrap();
        let record = original
            .lines()
            .find(|line| line.contains("compiler-artifact"))
            .unwrap();
        let changed = match case {
            "missing" => original.replace(record, ""),
            "package" => original.replace(
                &resolved.wasm_outputs().unwrap().targets[0].package_id,
                "unrelated-package",
            ),
            "kind" => original.replace("\"kind\":[\"cdylib\"]", "\"kind\":[\"lib\"]"),
            "relative" => original.replace("'\"$target\"'", "relative"),
            "duplicate" => original.replace(record, &format!("{record}\n{record}")),
            "ambiguous" => original.replace("fixture.wasm\"]", "fixture.wasm\",\"/extra.wasm\"]"),
            "no-wasm" => original.replace("fixture.wasm\"]", "fixture.rlib\"]"),
            "crate-type" => {
                original.replace("\"crate_types\":[\"cdylib\"]", "\"crate_types\":[\"lib\"]")
            }
            "malformed" => original.replace(record, "printf '%s\\n' '{invalid}'"),
            _ => original.clone(),
        };
        let exit = if case == "failed" { 23 } else { 0 };
        let changed = changed.replace(
            "exit 0",
            &format!("printf completed > completed\n    exit {exit}"),
        );
        fs::write(&wrapper, changed).unwrap();
        if matches!(case, "oversized" | "failed") {
            spec = spec.with_cargo_message_bytes(8);
        }
        let error = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap_err();
        match case {
            "missing" | "package" | "kind" => assert!(
                matches!(error, WasmBuildError::MissingArtifacts { .. }),
                "{case}: {error}"
            ),
            "failed" => assert!(
                matches!(error, WasmBuildError::CommandFailed { status, .. } if status.code() == Some(23))
            ),
            _ => assert!(
                matches!(
                    error,
                    WasmBuildError::Io {
                        operation: "decode Cargo build artifacts",
                        ..
                    }
                ),
                "{case}: {error}"
            ),
        }
        assert!(
            root.join("completed").is_file(),
            "{case}: Cargo must finish even after protocol refusal"
        );
        let entry = super::cache_entry_directory(
            &spec,
            resolve_cargo_build_inputs(&spec).unwrap().fingerprint(),
        );
        assert!(
            !entry.exists(),
            "{case}: incomplete exact entry must be cleaned up"
        );
        assert!(!expected_artifacts(&spec, &spec.target_dir)[0].exists());
        fs::remove_dir_all(root).unwrap();
    }
}

// Path-only cases declare the fixture's default targets without executing Cargo.
fn fixture_targets(spec: &WasmBuildSpec) -> Vec<super::WasmArtifactTarget> {
    spec.packages
        .iter()
        .map(|package| super::WasmArtifactTarget {
            package_id: String::new(),
            name: spec
                .example
                .clone()
                .unwrap_or_else(|| package.replace('-', "_")),
            kind: if spec.example.is_some() {
                crate::cargo::observation::schema::TargetKind::Example
            } else {
                crate::cargo::observation::schema::TargetKind::CDyLib
            },
        })
        .collect()
}
fn expected_artifacts(spec: &WasmBuildSpec, target_dir: &Path) -> Vec<PathBuf> {
    super::expected_artifacts(spec, target_dir, &fixture_targets(spec))
}

#[cfg(unix)]
#[test]
fn prepared_reader_counts_are_concurrent_saturating_and_only_for_matches() {
    let (root, spec) = fake_wasm_build_spec("reader-counter");
    let state = super::WasmBuildInputSnapshotState::prepare(std::slice::from_ref(&spec)).unwrap();
    let unknown = spec
        .clone()
        .with_environment(crate::cargo_fixture::environment_with([(
            "OTHER_RECIPE",
            "yes",
        )]));
    assert!(state.admit(&[&unknown]).is_err());
    assert_eq!(state.admit(&[&spec, &unknown]).unwrap_err(), 1);
    assert_eq!(state.reader_reuses(), 0);
    thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                for _ in 0..32 {
                    assert!(state.admit(&[&spec]).is_ok());
                }
            });
        }
    });
    assert_eq!(state.reader_reuses(), 256);
    state.reader_reuses.store(usize::MAX - 1, Ordering::Relaxed);
    thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| assert!(state.admit(&[&spec]).is_ok()));
        }
    });
    assert_eq!(state.reader_reuses(), usize::MAX);
    assert!(state.admit(&[&unknown]).is_err());
    assert_eq!(state.reader_reuses(), usize::MAX);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn separate_tool_phases_keep_aggregate_identity_timings() {
    let mut progress = ProgressReporter::silent();
    progress.record_phase(WasmBuildPhase::CargoIdentity, Duration::from_secs(2));
    progress.record_phase(WasmBuildPhase::RustcIdentity, Duration::from_secs(3));
    assert_eq!(
        progress.failure_timings.input_resolution().tool_identity(),
        Duration::from_secs(5)
    );
    assert_eq!(
        progress.failure_timings.input_resolution().total(),
        Duration::from_secs(5)
    );
    let error = WasmBuildError::CommandSpawn {
        phase: WasmBuildPhase::RustcIdentity,
        program: "missing-rustc".into(),
        source: std::io::ErrorKind::NotFound.into(),
    };
    assert_eq!(
        progress
            .failure_details(&error, Duration::from_secs(7))
            .phase(),
        WasmBuildPhase::RustcIdentity
    );
    assert_eq!(
        ProgressReporter::silent()
            .failure_details(&error, Duration::ZERO)
            .phase(),
        WasmBuildPhase::RustcIdentity
    );
}

#[test]
#[cfg(unix)]
fn cargo_spawn_custody_and_stdin_do_not_depend_on_progress() {
    use std::process::{Command, Stdio};
    for grouped in [false, true] {
        for observed in [false, true] {
            let root = unique_temp_directory("cargo-explicit-spawn");
            fs::write(root.join("stdin"), b"caller input\n").unwrap();
            fs::write(root.join("build"), "set -eu\nps -o pgid= -p $$ > group\nIFS= read -r line\nprintf '%s' \"$line\" > input\n").unwrap();
            let spec = WasmBuildSpec::new(
                &root,
                &root.join("exact"),
                &["fixture"],
                crate::cargo_fixture::environment(),
                CONFIGURATION_LIMITS,
            )
            .with_cargo_program("/bin/sh");
            let mut observer = |_| {};
            let mut progress = if observed {
                ProgressReporter::new(Some((
                    WasmBuildProgressConfig::new().without_heartbeats(),
                    &mut observer,
                )))
                .unwrap()
            } else {
                ProgressReporter::silent()
            };
            let mut child_pid = 0;
            let mut spawn = |command: &mut Command| {
                command.stdin(Stdio::from(fs::File::open(root.join("stdin"))?));
                let child = if grouped {
                    OwnedChild::spawn(command, ic_host_process::child::CleanupPolicy::KillAndWait)?
                } else {
                    OwnedChild::spawn_direct(command)?
                };
                child_pid = child.id();
                Ok(child)
            };
            run_cargo_build(
                &spec,
                &root.join("cargo-target"),
                &[],
                &mut progress,
                &mut spawn,
            )
            .unwrap();
            let group = fs::read_to_string(root.join("group"))
                .unwrap()
                .trim()
                .parse::<u32>()
                .unwrap();
            let parent = Command::new("ps")
                .args(["-o", "pgid=", "-p", &std::process::id().to_string()])
                .output()
                .unwrap();
            assert!(parent.status.success());
            let parent_group = String::from_utf8(parent.stdout)
                .unwrap()
                .trim()
                .parse::<u32>()
                .unwrap();
            assert_eq!(group, if grouped { child_pid } else { parent_group });
            assert_eq!(fs::read(root.join("input")).unwrap(), b"caller input");
            fs::remove_dir_all(root).unwrap();
        }
    }
}

#[test]
#[cfg(unix)]
fn explicit_environment_is_shared_by_probes_metadata_and_build_without_inheritance() {
    let (root, spec) = fake_wasm_build_spec("explicit-environment");
    assert!(std::env::var_os("HOME").is_some());
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let metadata = std::process::Command::new(cargo)
        .current_dir(&root)
        .args(["metadata", "--offline", "--format-version", "1"])
        .output()
        .unwrap();
    assert!(metadata.status.success());
    fs::write(root.join("metadata.json"), metadata.stdout).unwrap();
    let wrapper = root.join("cargo.sh");
    let record = crate::cargo_fixture::wasm_artifact_message_script(&root, "target");
    write_executable_script(
        &wrapper,
        format!(
            r#"#!/bin/sh
set -eu
test -z "${{HOME+x}}"
test "$ONLY" = 'selected recipe value'
printf '%s\n' "$1" >> calls
case "$1" in
  --version|-vV) printf 'fixture tool\n' ;;
  metadata) /bin/cat metadata.json ;;
  build)
    target="$CARGO_TARGET_DIR"
    mkdir -p "$target/wasm32-unknown-unknown/debug"
    printf '\000asm\001\000\000\000' > "$target/wasm32-unknown-unknown/debug/fixture.wasm"
    {record}
    ;;
  *) exit 19 ;;
esac
"#
        ),
    )
    .unwrap();
    let selected = [("PATH", "/usr/bin:/bin"), ("ONLY", "selected recipe value")];
    let spec = spec
        .with_rustc_identity_program(&wrapper)
        .with_environment(selected);
    assert!(!format!("{spec:?}").contains("selected recipe value"));
    let first = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
    let warm = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
    assert!(warm.is_reused());
    assert_eq!(first.record().fingerprint(), warm.record().fingerprint());
    let calls = fs::read_to_string(root.join("calls")).unwrap();
    for expected in ["--version", "-vV", "metadata", "build"] {
        assert!(calls.lines().any(|line| line == expected), "{calls}");
    }
    assert_eq!(calls.lines().filter(|line| *line == "build").count(), 1);
    drop((first, warm));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_environment_is_rejected_before_commands_or_cache_creation() {
    let root = unique_temp_directory("invalid-child-environment");
    for (name, value) in [
        ("", "value"),
        ("BAD=NAME", "value"),
        ("BAD\0NAME", "value"),
        ("NAME", "bad\0value"),
    ] {
        let spec = WasmBuildSpec::new(
            &root,
            &root.join("exact"),
            &["fixture"],
            [(name, value)],
            CONFIGURATION_LIMITS,
        )
        .with_cargo_program("/missing/must-not-run");
        assert!(matches!(
            build_wasm_canisters_cached(&spec, |_| panic!("must not spawn")),
            Err(WasmBuildError::InvalidSpec { .. })
        ));
        assert!(!root.join("exact").exists());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn mismatched_cargo_artifact_count_refuses_before_copy_and_can_recover() {
    let root = unique_temp_directory("wasm-copy-inventory");
    let source = root.join("source.wasm");
    let cached = root.join("cache.wasm");
    let public = root.join("public.wasm");
    fs::write(&source, b"new wasm").unwrap();
    fs::write(&cached, b"retained wasm").unwrap();
    let paths = vec![(cached.clone(), public.clone())];
    for sources in [vec![], vec![source.clone(), source.clone()]] {
        assert!(matches!(
            super::copy_wasm_artifacts(&sources, &paths),
            Err(WasmBuildError::InvalidMetadata { .. })
        ));
        assert_eq!(fs::read(&cached).unwrap(), b"retained wasm");
        assert!(!artifact_stamp_path(&cached).exists());
        assert!(!public.exists());
    }
    super::copy_wasm_artifacts(&[source], &paths).unwrap();
    let fingerprint = digest_bytes("wasm-copy-inventory-test", b"fixture");
    let verified = publish_artifact_stamps(paths.clone(), fingerprint).unwrap();
    materialize_artifacts(&verified, fingerprint, false).unwrap();
    assert_eq!(fs::read(public).unwrap(), b"new wasm");
    assert_eq!(
        validated_artifact_set(paths, fingerprint).unwrap(),
        verified
    );
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn recipe_identity_separates_source_features_and_unrelated_environment() {
    let (root, spec) = fake_wasm_build_spec("comparison-recipe");
    let manifest = root.join("fixture/Cargo.toml");
    let original = fs::read_to_string(&manifest).unwrap();
    fs::write(&manifest, format!("{original}\n[features]\ncohort=[]\n")).unwrap();
    let before = resolve_cargo_build_inputs(&spec).unwrap();
    fs::write(
        root.join("fixture/src/lib.rs"),
        "pub fn value() -> u8 { 2 }\n",
    )
    .unwrap();
    let after = resolve_cargo_build_inputs(&spec).unwrap();
    assert_eq!(before.recipe_identity(), after.recipe_identity());
    assert_ne!(before.input_digest(), after.input_digest());
    let mut selection = spec.cargo_selection.clone();
    selection.features.features.insert("cohort".to_owned());
    let feature =
        resolve_cargo_build_inputs(&spec.clone().with_cargo_selection(selection)).unwrap();
    assert_eq!(before.recipe_identity(), feature.recipe_identity());
    assert_ne!(
        before.facts().selection.features,
        feature.facts().selection.features
    );
    let mut environment = spec.environment().clone();
    environment.insert("PRIVATE_CREDENTIAL".into(), "do-not-report-me".into());
    let unrelated =
        resolve_cargo_build_inputs(&spec.clone().with_environment(environment.clone())).unwrap();
    assert_eq!(before.recipe_identity(), unrelated.recipe_identity());
    assert!(!format!("{unrelated:?}").contains("do-not-report-me"));
    for variable in [
        "CARGO_INCREMENTAL",
        "RUSTC_BOOTSTRAP",
        "CARGO_BUILD_TARGET_DIR",
        "CARGO_BUILD_BUILD_DIR",
    ] {
        let mut selected = spec.environment().clone();
        selected.insert(variable.into(), "1".into());
        let observed =
            resolve_cargo_build_inputs(&spec.clone().with_environment(selected)).unwrap();
        assert_eq!(
            before.recipe_identity() == observed.recipe_identity(),
            variable.starts_with("CARGO_BUILD_")
        );
    }
    environment.insert("RUSTFLAGS".into(), "-C opt-level=2".into());
    let flags = resolve_cargo_build_inputs(&spec.clone().with_environment(environment)).unwrap();
    assert_ne!(before.recipe_identity(), flags.recipe_identity());
    fs::create_dir(root.join(".cargo")).unwrap();
    fs::write(
        root.join(".cargo/config.toml"),
        "[build]\nrustflags=['-C','opt-level=1']\n",
    )
    .unwrap();
    let configured = resolve_cargo_build_inputs(&spec).unwrap();
    assert_ne!(before.recipe_identity(), configured.recipe_identity());
    let workspace = root.join("Cargo.toml");
    let original = fs::read_to_string(&workspace).unwrap();
    fs::write(
        workspace,
        format!("{original}\n[profile.dev]\nopt-level=2\n"),
    )
    .unwrap();
    let profile = resolve_cargo_build_inputs(&spec).unwrap();
    assert_ne!(configured.recipe_identity(), profile.recipe_identity());
    let built = build_wasm_canisters_cached(&spec, OwnedChild::spawn_direct).unwrap();
    assert_eq!(profile.recipe_identity(), built.record().recipe_identity());
    let config = root.join(".cargo");
    fs::write(config.join("config.toml"), "include=['a.toml']\n").unwrap();
    fs::write(
        config.join("a.toml"),
        "include=['b.toml']\n[profile.dev]\nopt-level=1\n",
    )
    .unwrap();
    fs::write(config.join("b.toml"), "[profile.dev]\nopt-level=2\n").unwrap();
    let nested = resolve_cargo_build_inputs(&spec).unwrap();
    fs::write(config.join("config.toml"), "include=['a.toml','b.toml']\n").unwrap();
    fs::write(config.join("a.toml"), "[profile.dev]\nopt-level=1\n").unwrap();
    let flat = resolve_cargo_build_inputs(&spec).unwrap();
    assert_ne!(nested.recipe_identity(), flat.recipe_identity());
}

// This caller admits small configuration trees; libraries do not select this policy.
const CONFIGURATION_LIMITS: crate::cargo::environment::ConfigurationLimits =
    crate::cargo::environment::ConfigurationLimits {
        file_bytes: 1024 * 1024,
        total_bytes: 16 * 1024 * 1024,
        files: 128,
        depth: 32,
    };
