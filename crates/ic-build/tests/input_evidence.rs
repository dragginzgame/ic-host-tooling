//! Actual Cargo depfile qualification for standalone input evidence.

mod support;

use ic_build::artifacts::{
    InputEvidence, InputEvidenceError, InputEvidenceSpec, read_cargo_depfile_inputs,
};
use std::{fs, process::Command};

#[test]
fn real_cargo_depfiles_retain_external_include_evidence() {
    let root = support::fs_fixture::unique_temp_directory("real-cargo-input-evidence");
    fs::create_dir(root.join("src")).unwrap();
    fs::create_dir(root.join("external")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname=\"evidence_fixture\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[workspace]\n",
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub const DATA: &[u8] = include_bytes!(\"../external/value.txt\");\n",
    )
    .unwrap();
    let external = root.join("external/value.txt");
    fs::write(&external, b"retained").unwrap();
    let target = root.join("outputs");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .current_dir(&root)
        .args(["build", "--offline", "--target-dir"])
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let paths = read_cargo_depfile_inputs(
        &root,
        &target.join("debug/libevidence_fixture.d"),
        std::slice::from_ref(&target),
        64 * 1024,
    )
    .unwrap();
    let external = external.canonicalize().unwrap();
    assert!(paths.contains(&external));
    let spec = paths.iter().fold(
        InputEvidenceSpec::new(b"caller-identity".as_slice()),
        InputEvidenceSpec::with_input,
    );
    let before = InputEvidence::capture(&spec).unwrap();
    let after = InputEvidenceSpec::new(b"caller-identity".as_slice());
    before.validate_after(&after).unwrap();
    fs::write(&external, b"changed").unwrap();
    assert!(matches!(before.validate_after(&after),
        Err(InputEvidenceError::ChangedInput { path }) if path == external));
    fs::remove_dir_all(root).unwrap();
}
