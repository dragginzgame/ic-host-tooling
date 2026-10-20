//! Actual Cargo packaging, extracted graph admission and retained source evidence.
#![cfg(unix)]

mod support;

use ic_build::cargo::packages::{
    PackageArchiveSpec, PackageQualificationLimits, qualify_package_archives,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_process::tool::{OutputLimit, OutputLimits};
use std::{fs, os::unix::fs::PermissionsExt as _, process::Command};

#[test]
fn real_package_qualification_retains_exact_graph_and_source_inventory() {
    let root = support::fs_fixture::unique_temp_directory("package-qualification");
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("Cargo.toml"), "[workspace]\n[package]\nname='qualification-fixture'\nversion='0.1.0'\nedition='2024'\n[dependencies]\nitoa='1.0'\n").unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> String { itoa::Buffer::new().format(42).to_owned() }\n#[test]\nfn formats_value() { assert_eq!(value(), \"42\"); }\n").unwrap();
    let cargo = || {
        let mut command = Command::new(env!("CARGO"));
        command
            .env("RUSTUP_AUTO_INSTALL", "0")
            .env("CARGO_NET_OFFLINE", "true")
            .env("CARGO_TARGET_DIR", root.join("target"));
        command
    };
    let prepared = cargo()
        .current_dir(&root)
        .args(["generate-lockfile", "--offline"])
        .output()
        .unwrap();
    assert!(
        prepared.status.success(),
        "{}",
        String::from_utf8_lossy(&prepared.stderr)
    );
    let entry_lock = fs::read(root.join("Cargo.lock")).unwrap();
    let evidence = support::fs_fixture::unique_temp_directory("package-qualification-evidence");
    fs::create_dir_all(root.join("target/package")).unwrap();
    fs::set_permissions(&evidence, fs::Permissions::from_mode(0o700)).unwrap();
    qualify_package_archives(
        &root,
        &root.join("target/package"),
        &evidence,
        &[PackageArchiveSpec {
            name: "qualification-fixture",
            version: "0.1.0",
            patch_registry: true,
        }],
        PackageQualificationLimits {
            process: OutputLimits {
                stdout: OutputLimit::Terminate(4 * 1024 * 1024),
                stderr: OutputLimit::Terminate(1024 * 1024),
                timeout: None,
            },
            file_bytes: 4 * 1024 * 1024,
            source_entries: 100,
            source_depth: 10,
        },
        cargo,
        |archive, destination| {
            let mut command = Command::new("tar");
            command.arg("-xzf").arg(archive).arg("-C").arg(destination);
            command
        },
    )
    .unwrap();
    assert_eq!(fs::read(root.join("Cargo.lock")).unwrap(), entry_lock);
    let archive = fs::read(root.join("target/package/qualification-fixture-0.1.0.crate")).unwrap();
    assert_eq!(
        fs::read(evidence.join("archive.sha256")).unwrap(),
        format!("{}\n", Sha256Digest::compute(&archive)).as_bytes()
    );
    let original = fs::read(evidence.join("entry-sources.json")).unwrap();
    assert_eq!(
        original,
        fs::read(evidence.join("final-sources.json")).unwrap()
    );
    let inventory: serde_json::Value = serde_json::from_slice(&original).unwrap();
    assert_eq!(
        inventory["qualification-fixture-0.1.0/src/lib.rs"]["sha256"],
        Sha256Digest::compute(&fs::read(root.join("src/lib.rs")).unwrap()).to_string()
    );
    assert!(inventory["qualification-fixture-0.1.0/src/lib.rs"]["mode"].is_number());
    for name in ["test-targets.json", "test-docs.json", "admitted-Cargo.lock"] {
        assert!(evidence.join(name).is_file());
    }
}
