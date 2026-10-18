use super::*;
use crate::cargo::packages::ExternalLockError;
use crate::fs_fixture::unique_temp_directory;
use ic_host_process::tool::{ExecutionFailure, OutputLimit};
use std::{process::Stdio, time::Duration};

const fn limits() -> PackageQualificationLimits {
    PackageQualificationLimits {
        process: OutputLimits {
            stdout: OutputLimit::Terminate(4096),
            stderr: OutputLimit::Terminate(4096),
            timeout: Some(Duration::from_secs(5)),
        },
        file_bytes: 4096,
        source_entries: 10,
        source_depth: 10,
    }
}

#[test]
fn verification_manifest_preserves_members_and_only_selected_patches() {
    let specs = [
        PackageArchiveSpec {
            name: "first-package",
            version: "1.0.0-rc.1+build",
            patch_registry: true,
        },
        PackageArchiveSpec {
            name: "second_package",
            version: "2.0.0",
            patch_registry: false,
        },
    ];
    let manifest: toml::Table = toml::from_str(&verification_manifest(&specs).unwrap()).unwrap();
    let expected: toml::Table = toml::from_str(
        r#"
        [workspace]
        resolver = "3"
        members = ["first-package-1.0.0-rc.1+build", "second_package-2.0.0"]
        [patch.crates-io]
        first-package = { path = "first-package-1.0.0-rc.1+build" }
        "#,
    )
    .unwrap();
    assert_eq!(manifest, expected);
    let unpatched = [PackageArchiveSpec {
        patch_registry: false,
        ..specs[0]
    }];
    let manifest: toml::Table =
        toml::from_str(&verification_manifest(&unpatched).unwrap()).unwrap();
    assert_eq!(manifest["patch"]["crates-io"].as_table().unwrap().len(), 0);
}

#[test]
fn invalid_selection_and_occupied_evidence_do_not_invoke_tools() {
    let root = unique_temp_directory("package-admission");
    let evidence = root.join("evidence");
    fs::create_dir(&evidence).unwrap();
    let specs = [PackageArchiveSpec {
        name: "../outside",
        version: "1.0.0",
        patch_registry: false,
    }];
    assert!(
        qualify_package_archives(
            &root,
            &root,
            &evidence,
            &specs,
            limits(),
            || panic!("no Cargo effect"),
            |_, _| panic!("no extraction effect")
        )
        .is_err()
    );
    fs::write(evidence.join("prior"), "retained").unwrap();
    let specs = [PackageArchiveSpec {
        name: "sample",
        version: "1.0.0",
        patch_registry: false,
    }];
    assert!(
        qualify_package_archives(
            &root,
            &root,
            &evidence,
            &specs,
            limits(),
            || panic!("no Cargo effect"),
            |_, _| panic!("no extraction effect")
        )
        .is_err()
    );
    assert_eq!(fs::read(evidence.join("prior")).unwrap(), b"retained");
}

#[test]
fn graph_requires_exact_extracted_identity_and_unique_members() {
    let root = unique_temp_directory("package-graph")
        .canonicalize()
        .unwrap();
    let directory = root.join("sample-1.0.0");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("Cargo.toml"), "manifest").unwrap();
    let specs = [PackageArchiveSpec {
        name: "sample",
        version: "1.0.0",
        patch_registry: false,
    }];
    let metadata = json!({"packages": [{"name": "sample", "version": "1.0.0", "source": null, "manifest_path": directory.join("Cargo.toml"), "id": "selected", "dependencies": [], "targets": [], "features": {}, "metadata": null}], "workspace_members": ["selected"], "workspace_root": root, "target_directory": root.join("target"), "version": 1});
    let admit = |metadata: &Value| -> Result<(), Cause> {
        admit_graph(&serde_json::from_value(metadata.clone())?, &root, &specs)
    };
    admit(&metadata).unwrap();
    let mut missing = metadata.clone();
    missing["packages"] = json!([]);
    let mut ambiguous = metadata.clone();
    ambiguous["packages"]
        .as_array_mut()
        .unwrap()
        .push(metadata["packages"][0].clone());
    for rejected in [&missing, &ambiguous] {
        assert_eq!(
            admit(rejected).unwrap_err().to_string(),
            "expected one extracted package: sample"
        );
    }
    for (field, value) in [
        ("source", json!("registry+wrong")),
        ("version", json!("0.9.0")),
        ("manifest_path", json!(root.join("Cargo.toml"))),
        ("id", json!("foreign")),
    ] {
        let mut altered = metadata.clone();
        altered["packages"][0][field] = value;
        assert!(admit(&altered).is_err());
    }
    let mut altered = metadata;
    altered["workspace_members"] = json!(["selected", "selected"]);
    assert!(admit(&altered).is_err());
}

#[test]
fn lock_allows_only_original_external_selections_and_checksums() {
    let row = format!(
        "[[package]]\nname = \"external\"\nversion = \"1.0.0\"\nsource = \"registry+test\"\nchecksum = \"{}\"\n",
        "a".repeat(64)
    );
    admit_lock(row.as_bytes(), row.as_bytes()).unwrap();
    for changed in [
        row.replace("1.0.0", "1.0.1"),
        row.replace(&"a".repeat(64), &"b".repeat(64)),
        format!("{row}{row}"),
        row.replace("checksum", "missing"),
    ] {
        assert!(admit_lock(row.as_bytes(), changed.as_bytes()).is_err());
    }
    for checksum in ["0".repeat(64), "0123456789abcdef".repeat(4)] {
        let valid = row.replace(&"a".repeat(64), &checksum);
        admit_lock(valid.as_bytes(), valid.as_bytes()).unwrap();
    }
    for checksum in [
        "a".repeat(63),
        "a".repeat(65),
        "A".repeat(64),
        "g".repeat(64),
        "é".repeat(32),
    ] {
        let invalid = row.replace(&"a".repeat(64), &checksum);
        let error = admit_lock(invalid.as_bytes(), invalid.as_bytes()).unwrap_err();
        assert!(matches!(
            error.downcast_ref::<ExternalLockError>(),
            Some(ExternalLockError::InvalidPackage {
                input: "original",
                index: 0,
                reason: "invalid package checksum",
            })
        ));
    }
    assert!(admit_lock(row.as_bytes(), b"package = []").is_err());
    assert!(admit_lock(row.as_bytes(), b"version = 4").is_err());
}

#[test]
fn source_inventory_detects_bytes_modes_links_and_resource_overflow() {
    use std::os::unix::fs::PermissionsExt as _;
    let root = unique_temp_directory("package-sources");
    let path = root.join("sample-1.0.0");
    fs::create_dir(&path).unwrap();
    let file = path.join("source");
    fs::write(&file, b"original").unwrap();
    let specs = [PackageArchiveSpec {
        name: "sample",
        version: "1.0.0",
        patch_registry: false,
    }];
    let before = sources(&root, &specs, limits()).unwrap();
    let expected = json!({"sample-1.0.0/source": {
        "sha256": Sha256Digest::compute(b"original").to_string(),
        "mode": fs::metadata(&file).unwrap().permissions().mode() & 0o777,
    }});
    assert_eq!(
        serde_json::to_vec(&before).unwrap(),
        serde_json::to_vec(&expected).unwrap()
    );
    fs::write(&file, b"changed").unwrap();
    assert_ne!(before, sources(&root, &specs, limits()).unwrap());
    fs::write(&file, b"original").unwrap();
    let mode = fs::metadata(&file).unwrap().permissions().mode();
    fs::set_permissions(&file, fs::Permissions::from_mode(mode ^ 0o100)).unwrap();
    assert_ne!(before, sources(&root, &specs, limits()).unwrap());
    assert!(
        sources(
            &root,
            &specs,
            PackageQualificationLimits {
                file_bytes: 1,
                ..limits()
            }
        )
        .is_err()
    );
    std::os::unix::fs::symlink(&file, path.join("link")).unwrap();
    assert!(sources(&root, &specs, limits()).is_err());
}

#[test]
fn failed_command_retains_original_status_and_diagnostics() {
    let root = unique_temp_directory("package-failure");
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf partial; printf diagnostic >&2; exit 23"])
        .stdin(Stdio::null());
    let error = execute(&mut command, &root, "metadata", limits().process).unwrap_err();
    assert!(
        matches!(error.downcast_ref::<PackageCommandError>().and_then(|error| error.execution.as_ref()), Some(ToolError::Execution(error)) if matches!(error.failure, ExecutionFailure::ExitStatus) && error.evidence.status.unwrap().code() == Some(23))
    );
    assert_eq!(fs::read(root.join("metadata.json")).unwrap(), b"partial");
    assert_eq!(
        fs::read(root.join("metadata.stderr")).unwrap(),
        b"diagnostic"
    );
}

#[test]
fn logging_failure_does_not_replace_nonzero_process_evidence() {
    let root = unique_temp_directory("package-retention-failure");
    let mut command = Command::new("/bin/sh");
    command
        .args([
            "-c",
            "rmdir \"$1\"; printf retained-in-error; exit 23",
            "sh",
        ])
        .arg(&root);
    let error = execute(&mut command, &root, "metadata", limits().process).unwrap_err();
    let error = error.downcast_ref::<PackageCommandError>().unwrap();
    assert_eq!(error.retention.len(), 2);
    assert!(
        matches!(&error.execution, Some(ToolError::Execution(error)) if error.evidence.status.unwrap().code() == Some(23) && error.evidence.stdout == b"retained-in-error")
    );
}

#[test]
fn directory_depth_and_entry_counts_bound_empty_tree_traversal() {
    let root = unique_temp_directory("package-traversal");
    fs::create_dir_all(root.join("sample-1.0.0/one/two")).unwrap();
    let specs = [PackageArchiveSpec {
        name: "sample",
        version: "1.0.0",
        patch_registry: false,
    }];
    assert!(
        sources(
            &root,
            &specs,
            PackageQualificationLimits {
                source_depth: 1,
                ..limits()
            }
        )
        .is_err()
    );
    assert!(
        sources(
            &root,
            &specs,
            PackageQualificationLimits {
                source_entries: 2,
                ..limits()
            }
        )
        .is_err()
    );
    fs::create_dir(root.join("other-1.0.0")).unwrap();
    let all_specs = [
        PackageArchiveSpec {
            name: "sample",
            version: "1.0.0",
            patch_registry: false,
        },
        PackageArchiveSpec {
            name: "other",
            version: "1.0.0",
            patch_registry: false,
        },
    ];
    for (source_entries, succeeds) in [(3, false), (4, true)] {
        assert_eq!(
            sources(
                &root,
                &all_specs,
                PackageQualificationLimits {
                    source_entries,
                    ..limits()
                }
            )
            .is_ok(),
            succeeds
        );
    }
}
