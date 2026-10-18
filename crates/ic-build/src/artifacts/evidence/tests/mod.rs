use super::*;
use crate::artifacts::test_support::fake_wasm_build_spec;

#[test]
fn export_preserves_exact_bytes_and_refuses_limits_and_changed_source() {
    let (root, spec) = fake_wasm_build_spec("wasm-evidence");
    let evidence = WasmBuildEvidence::acquire(
        spec.clone(),
        ic_host_process::child::OwnedChild::spawn_direct,
    )
    .unwrap();
    let destination = root.join("evidence.wasm");
    fs::write(&destination, b"previous").unwrap();
    assert!(evidence.export_artifact(0, &destination, 7).is_err());
    assert_eq!(fs::read(&destination).unwrap(), b"previous");
    assert!(evidence.export_artifact(1, &destination, 8).is_err());
    let receipt = evidence.export_artifact(0, &destination, 8).unwrap();
    assert_eq!(fs::read(&destination).unwrap(), b"\0asm\x01\0\0\0");
    assert_eq!(
        receipt.wasm_sha256,
        hash_reader(fs::File::open(&destination).unwrap(), 8)
            .unwrap()
            .sha256
            .to_string()
    );
    assert!(
        evidence
            .export_artifact(0, &evidence.build.record().artifacts()[0], 8)
            .is_err()
    );
    let reused =
        WasmBuildEvidence::acquire(spec, ic_host_process::child::OwnedChild::spawn_direct).unwrap();
    let second = reused.export_artifact(0, &destination, 8).unwrap();
    assert_eq!(receipt.build_fingerprint, second.build_fingerprint);
    let blocked = root.join("blocked");
    fs::create_dir(&blocked).unwrap();
    let error = reused.export_artifact(0, &blocked, 8).unwrap_err();
    let recovery = error
        .get_ref()
        .unwrap()
        .downcast_ref::<crate::artifacts::CaptureRetentionError>()
        .unwrap();
    assert_eq!(
        fs::read(&recovery.retained_path).unwrap(),
        b"\0asm\x01\0\0\0"
    );
    drop(reused);
    fs::write(root.join("fixture/src/lib.rs"), "pub fn changed() {}\n").unwrap();
    assert!(evidence.verify_current().is_err());
    assert!(evidence.export_artifact(0, &destination, 8).is_err());
    assert_eq!(fs::read(&destination).unwrap(), b"\0asm\x01\0\0\0");
    drop(evidence);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn raw_input_changes_and_mismatched_recipe_fingerprints_refuse_evidence() {
    let (root, spec) = fake_wasm_build_spec("evidence-raw-inputs");
    let mut evidence = WasmBuildEvidence::acquire(
        spec.clone(),
        ic_host_process::child::OwnedChild::spawn_direct,
    )
    .unwrap();
    let original = fs::read(root.join("Cargo.toml")).unwrap();
    let mut changed = original.clone();
    changed.extend_from_slice(b"\n# unrelated workspace edit\n");
    fs::write(root.join("Cargo.toml"), changed).unwrap();
    let current = resolve_cargo_build_inputs(&spec).unwrap();
    assert_eq!(current.fingerprint(), evidence.inputs.fingerprint());
    assert_ne!(
        current.validation_digest(),
        evidence.inputs.validation_digest()
    );
    assert!(evidence.verify_current().is_err());
    fs::write(root.join("Cargo.toml"), original).unwrap();
    let alternate = spec.with_environment(crate::cargo_fixture::environment_with([(
        "RECIPE_MARKER",
        "changed",
    )]));
    let other =
        build_wasm_canisters_cached(&alternate, ic_host_process::child::OwnedChild::spawn_direct)
            .unwrap();
    assert_eq!(
        other.record().input_digest(),
        evidence.build.record().input_digest()
    );
    assert_ne!(
        other.record().fingerprint(),
        evidence.build.record().fingerprint()
    );
    evidence.build = other;
    assert!(evidence.verify_current().is_err());
    drop(evidence);
    fs::remove_dir_all(root).unwrap();
}
