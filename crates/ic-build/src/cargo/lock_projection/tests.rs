use super::*;

const BEFORE: &str = r#"version = 4
[[package]]
name = "app"
version = "0.1.0"
dependencies = ["dependency"]
[[package]]
name = "app"
version = "9.0.0"
source = "registry+https://example.invalid"
checksum = "fixed"
[[package]]
name = "dependency"
version = "1.0.0"
source = "registry+https://example.invalid"
checksum = "dependency-checksum"
"#;
fn projection() -> WorkspaceVersionProjection<'static> {
    WorkspaceVersionProjection {
        package: "app",
        previous: "0.1.0",
        next: "0.1.1",
    }
}
fn after() -> String {
    BEFORE.replacen("0.1.0", "0.1.1", 1)
}

#[test]
fn projects_only_workspace_versions_and_accepts_comments() {
    let after = format!("# retained source comment\n{}", after());
    verify_workspace_version_projection(BEFORE, &after, &[projection()]).unwrap();
    assert!(matches!(
        verify_workspace_version_projection(BEFORE, BEFORE, &[projection()]),
        Err(LockProjectionError::ChangedInputs)
    ));
}

#[test]
fn dependency_source_checksum_and_other_fields_cannot_change() {
    for after in [
        after().replace("dependency-checksum", "changed"),
        after().replace("example.invalid", "different.invalid"),
        after().replace("9.0.0", "9.0.1"),
        after().replace("[\"dependency\"]", "[]"),
        after().replace("version = 4", "version = 3"),
    ] {
        assert!(matches!(
            verify_workspace_version_projection(BEFORE, &after, &[projection()]),
            Err(LockProjectionError::ChangedInputs)
        ));
    }
}

#[test]
fn missing_duplicate_and_wrong_prior_workspace_records_refuse() {
    assert!(matches!(
        verify_workspace_version_projection(
            BEFORE,
            &after(),
            &[WorkspaceVersionProjection {
                package: "missing",
                ..projection()
            }]
        ),
        Err(LockProjectionError::PackageSelection { matches: 0, .. })
    ));
    let duplicate = format!("{BEFORE}\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\n");
    assert!(matches!(
        verify_workspace_version_projection(&duplicate, &after(), &[projection()]),
        Err(LockProjectionError::PackageSelection { matches: 2, .. })
    ));
    assert!(matches!(
        verify_workspace_version_projection(
            BEFORE,
            &after(),
            &[WorkspaceVersionProjection {
                previous: "0.0.9",
                ..projection()
            }]
        ),
        Err(LockProjectionError::PreviousVersion { .. })
    ));
    assert!(matches!(
        verify_workspace_version_projection(BEFORE, &after(), &[]),
        Err(LockProjectionError::InvalidSelection)
    ));
    assert!(matches!(
        verify_workspace_version_projection(BEFORE, &after(), &[projection(), projection()]),
        Err(LockProjectionError::InvalidSelection)
    ));
    assert!(matches!(
        verify_workspace_version_projection("not valid [", &after(), &[projection()]),
        Err(LockProjectionError::Parse {
            input: "before",
            ..
        })
    ));
    assert!(matches!(
        verify_workspace_version_projection(BEFORE, "not valid [", &[projection()]),
        Err(LockProjectionError::Parse { input: "after", .. })
    ));
}

#[test]
fn projects_a_coherent_multi_package_family() {
    let before = "version=4\n[[package]]\nname=\"one\"\nversion=\"0.1.0\"\n[[package]]\nname=\"one\"\nversion=\"9.0.0\"\nsource=\"registry+test\"\n[[package]]\nname=\"two\"\nversion=\"0.1.0\"\n";
    let changes = [
        WorkspaceVersionProjection {
            package: "two",
            ..projection()
        },
        WorkspaceVersionProjection {
            package: "one",
            ..projection()
        },
    ];
    verify_workspace_version_projection(before, &before.replace("0.1.0", "0.1.1"), &changes)
        .unwrap();
}

#[test]
fn projection_refusals_preserve_caller_order_and_prior_version_precedence() {
    let app = projection();
    let missing = WorkspaceVersionProjection {
        package: "missing",
        ..app
    };
    let wrong_prior = WorkspaceVersionProjection {
        previous: "wrong",
        ..app
    };
    assert!(matches!(
        verify_workspace_version_projection(BEFORE, "invalid [", &[missing, wrong_prior]),
        Err(LockProjectionError::PackageSelection { package, matches: 0 }) if package == "missing"
    ));
    assert!(matches!(
        verify_workspace_version_projection(BEFORE, "invalid [", &[wrong_prior, missing]),
        Err(LockProjectionError::PreviousVersion { package }) if package == "app"
    ));
    for versions in [["0.1.0", "wrong"], ["wrong", "0.1.0"]] {
        let before = versions
            .map(|version| format!("[[package]]\nname='app'\nversion='{version}'\n"))
            .concat();
        assert!(matches!(
            verify_workspace_version_projection(&before, "invalid [", &[app]),
            Err(LockProjectionError::PreviousVersion { .. })
        ));
    }
    for invalid in [
        vec![app, app],
        vec![WorkspaceVersionProjection { next: "", ..app }],
    ] {
        assert!(matches!(
            verify_workspace_version_projection("invalid [", "invalid [", &invalid),
            Err(LockProjectionError::InvalidSelection)
        ));
    }
}

#[test]
fn cargo_generated_workspace_update_preserves_external_selections() {
    use crate::fs_fixture::unique_temp_directory;
    use std::{fs, process::Command};

    let root = unique_temp_directory("cargo-lock-projection");
    fs::create_dir_all(root.join("external/src")).unwrap();
    fs::write(root.join("external/src/lib.rs"), "").unwrap();
    fs::write(
        root.join("external/Cargo.toml"),
        "[package]\nname = 'projection-external'\nversion = '9.0.0'\nedition = '2024'\n[workspace]\n",
    )
    .unwrap();
    for name in ["a", "b"] {
        fs::create_dir_all(root.join(name).join("src")).unwrap();
        fs::write(root.join(name).join("src/lib.rs"), "").unwrap();
        let dependency = if name == "a" {
            "projection-b = { path = '../b' }"
        } else {
            "projection-external = { path = '../external' }"
        };
        fs::write(
            root.join(name).join("Cargo.toml"),
            format!(
                "[package]\nname = 'projection-{name}'\nversion.workspace = true\n\
                 edition = '2024'\n[dependencies]\n{dependency}\n"
            ),
        )
        .unwrap();
    }
    let manifest = "[workspace]\nmembers = ['a', 'b']\nexclude = ['external']\nresolver = '3'\n\
                    [workspace.package]\nversion = '0.1.0'\n";
    fs::write(root.join("Cargo.toml"), manifest).unwrap();
    let generate = || {
        let output = Command::new(env!("CARGO"))
            .current_dir(&root)
            .args(["generate-lockfile", "--offline"])
            .env("RUSTUP_AUTO_INSTALL", "0")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::read_to_string(root.join("Cargo.lock")).unwrap()
    };
    let before = generate();
    fs::write(root.join("Cargo.toml"), manifest.replace("0.1.0", "0.1.1")).unwrap();
    let after = generate();
    let changes = ["projection-a", "projection-b"].map(|package| WorkspaceVersionProjection {
        package,
        previous: "0.1.0",
        next: "0.1.1",
    });
    verify_workspace_version_projection(&before, &after, &changes).unwrap();

    let mut drift: toml::Value = toml::from_str(&after).unwrap();
    let external = drift["package"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|package| package["name"].as_str() == Some("projection-external"))
        .unwrap();
    external["version"] = "9.0.1".into();
    assert!(matches!(
        verify_workspace_version_projection(&before, &toml::to_string(&drift).unwrap(), &changes),
        Err(LockProjectionError::ChangedInputs)
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn parse_error_preserves_role_message_and_original_parser_source() {
    use std::error::Error as _;
    let source = toml::from_str::<toml::Value>("[broken").unwrap_err();
    let expected = format!("invalid before lockfile: {source}");
    let error = LockProjectionError::Parse {
        input: "before",
        source,
    };
    assert_eq!(error.to_string(), expected);
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<toml::de::Error>()
            .is_some()
    );
    for (error, expected) in [
        (
            LockProjectionError::InvalidSelection,
            "invalid workspace version selection",
        ),
        (
            LockProjectionError::MissingPackages,
            "lockfile has no package array",
        ),
        (
            LockProjectionError::PackageSelection {
                package: "fixture".into(),
                matches: 2,
            },
            "workspace package fixture has 2 unsourced records; expected one",
        ),
        (
            LockProjectionError::PreviousVersion {
                package: "fixture".into(),
            },
            "unexpected previous workspace version for fixture",
        ),
        (
            LockProjectionError::ChangedInputs,
            "lockfile differs from the selected workspace version projection",
        ),
    ] {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
    }
}
