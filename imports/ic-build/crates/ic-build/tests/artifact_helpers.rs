//! Native artifact discovery and Cargo workspace membership qualification.

mod support;

use std::{fs, path::PathBuf};
use support::discover_workspace;
use support::fs_fixture::unique_temp_directory as unique_temp_dir;

// Verify Cargo locates the workspace owning this package.
#[test]
fn workspace_helpers_resolve_expected_paths() {
    let workspace_root = discover_workspace(env!("CARGO_MANIFEST_DIR")).unwrap();
    assert_eq!(
        workspace_root.join("Cargo.toml"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("Cargo.toml")
    );
}

#[test]
fn workspace_discovery_obeys_cargo_membership_and_independent_roots() {
    let root = unique_temp_dir("cargo-workspace-discovery");
    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = [\"apps/demo/worker\", \"external\"]\nexclude = [\"excluded\", \"testing\"]\nresolver = \"2\"\n").unwrap();
    for (name, relative, extra) in [
        ("worker", "apps/demo/worker", ""),
        ("external", "external", "workspace = \"..\"\n"),
        ("excluded", "excluded", ""),
        ("independent", "testing", "\n[workspace]\n"),
        ("unlisted", "unlisted", ""),
    ] {
        let package = root.join(relative);
        fs::create_dir_all(package.join("src")).unwrap();
        fs::write(package.join("src/lib.rs"), "").unwrap();
        fs::write(
            package.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\n{extra}"),
        )
        .unwrap();
    }
    let canonical = root.canonicalize().unwrap();
    assert_eq!(
        discover_workspace(root.join("apps/demo/worker")).unwrap(),
        canonical
    );
    assert_eq!(
        discover_workspace(root.join("external")).unwrap(),
        canonical
    );
    for independent in ["excluded", "testing"] {
        assert_eq!(
            discover_workspace(root.join(independent)).unwrap(),
            canonical.join(independent)
        );
    }
    assert!(discover_workspace(root.join("unlisted")).is_err());
    assert!(discover_workspace(root.join("missing")).is_err());
    assert!(!root.join("Cargo.lock").exists());
    assert!(!root.join("target").exists());
    fs::remove_dir_all(root).unwrap();
}
