use super::*;
use crate::fs_fixture::unique_temp_directory;

#[test]
fn nearest_declaration_accepts_file_start_and_standalone_package() {
    let root = unique_temp_directory("declared-workspace");
    let nested = root.join("nested");
    fs::create_dir_all(&nested).unwrap();
    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers=[]").unwrap();
    let manifest = nested.join("Cargo.toml");
    fs::write(&manifest, "[package]\nname='standalone'\nversion='0.1.0'").unwrap();
    assert_eq!(
        declared_workspace_root_from(&manifest).unwrap(),
        Some(root.canonicalize().unwrap())
    );
    fs::write(&manifest, "[ workspace ]\nmembers=[]").unwrap();
    assert_eq!(
        declared_workspace_root_from(&manifest).unwrap(),
        Some(nested.canonicalize().unwrap())
    );
    fs::remove_file(root.join("Cargo.toml")).unwrap();
    fs::write(&manifest, "description='literal [workspace]'").unwrap();
    assert_eq!(declared_workspace_root_from(&manifest).unwrap(), None);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_and_non_file_manifests_are_not_skipped() {
    let root = unique_temp_directory("declared-workspace-invalid");
    let manifest = root.join("Cargo.toml");
    fs::write(&manifest, "[workspace").unwrap();
    assert!(
        matches!(declared_workspace_root_from(&root), Err(DiscoveryError::ParseManifest { path, .. }) if path == manifest)
    );
    fs::remove_file(&manifest).unwrap();
    fs::create_dir(&manifest).unwrap();
    assert!(
        matches!(declared_workspace_root_from(&root), Err(DiscoveryError::ExpectedFile { path }) if path == manifest)
    );
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn file_symlinks_resolve_but_dangling_markers_fail() {
    use std::os::unix::fs::symlink;
    let root = unique_temp_directory("declared-workspace-links");
    let target = root.join("manifest");
    let manifest = root.join("Cargo.toml");
    fs::write(&target, "[workspace]").unwrap();
    symlink(&target, &manifest).unwrap();
    assert_eq!(
        declared_workspace_root_from(&root).unwrap(),
        Some(root.canonicalize().unwrap())
    );
    fs::remove_file(&target).unwrap();
    assert!(
        matches!(declared_workspace_root_from(&root), Err(DiscoveryError::Inspect { path, source }) if path == manifest && source.kind() == io::ErrorKind::NotFound)
    );
    fs::remove_dir_all(root).unwrap();
}
