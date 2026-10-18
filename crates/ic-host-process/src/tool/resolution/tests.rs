use super::{ResolutionError, resolve_executable};
use crate::{
    test_support::{Fixture, LIMITS, VERSION, tool_path},
    tool::{AdmittedTool, ToolError, VersionSpec},
};
use ic_host_artifacts::artifact::{ArtifactError, Sha256Digest};
use std::{
    ffi::OsString,
    fs, io,
    os::unix::{
        ffi::OsStringExt as _,
        fs::{PermissionsExt as _, symlink},
    },
    path::{Path, PathBuf},
};

fn executable(path: &Path) {
    // Deliberately not a valid program: resolution must inspect only metadata.
    fs::write(path, b"candidate bytes, not an admitted executable").unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn explicit_absolute_and_relative_paths_ignore_the_search_list() {
    let fixture = Fixture::new();
    let local = fixture.path().join("selected");
    executable(&local);
    let other = fixture.path().join("other");
    fs::create_dir(&other).unwrap();
    executable(&other.join("selected"));
    let directories = [other];
    let expected = local.canonicalize().unwrap();
    assert_eq!(
        resolve_executable(&local, fixture.path(), &directories).unwrap(),
        expected
    );
    assert_eq!(
        resolve_executable(Path::new("./selected"), fixture.path(), &directories).unwrap(),
        expected
    );
    assert_ne!(
        resolve_executable(Path::new("selected"), fixture.path(), &directories).unwrap(),
        expected
    );
    // An explicit missing path cannot fall back to the same name in the list.
    assert!(
        matches!(resolve_executable(Path::new("./absent/selected"), fixture.path(), &directories),
        Err(ResolutionError::Io { directory: None, source }) if source.kind() == io::ErrorKind::NotFound)
    );
}

#[test]
fn search_order_skips_missing_nonregular_and_nonexecutable_candidates() {
    let fixture = Fixture::new();
    let missing = fixture.path().join("missing");
    let directory = fixture.path().join("directory");
    fs::create_dir_all(directory.join("tool")).unwrap();
    let nonexecutable = fixture.path().join("nonexecutable");
    fs::create_dir(&nonexecutable).unwrap();
    fs::write(nonexecutable.join("tool"), b"not executable").unwrap();
    fs::set_permissions(
        nonexecutable.join("tool"),
        fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    let dangling = fixture.path().join("dangling");
    fs::create_dir(&dangling).unwrap();
    symlink("absent", dangling.join("tool")).unwrap();
    let first = fixture.path().join("first");
    let second = fixture.path().join("second");
    fs::create_dir(&first).unwrap();
    fs::create_dir(&second).unwrap();
    executable(&first.join("tool"));
    executable(&second.join("tool"));
    let directories = [
        missing,
        directory,
        nonexecutable,
        dangling,
        first.clone(),
        second,
    ];
    assert_eq!(
        resolve_executable(Path::new("tool"), fixture.path(), &directories).unwrap(),
        first.join("tool").canonicalize().unwrap()
    );
}

#[test]
fn relative_and_empty_search_entries_use_only_the_explicit_working_directory() {
    let fixture = Fixture::new();
    executable(&fixture.path().join("tool"));
    let nested = fixture.path().join("bin");
    fs::create_dir(&nested).unwrap();
    executable(&nested.join("tool"));
    assert!(matches!(
        resolve_executable(Path::new("tool"), fixture.path(), &[]),
        Err(ResolutionError::NotFound)
    ));
    assert_eq!(
        resolve_executable(Path::new("tool"), fixture.path(), &[PathBuf::new()]).unwrap(),
        fixture.path().join("tool")
    );
    assert_eq!(
        resolve_executable(Path::new("tool"), fixture.path(), &[PathBuf::from("bin")]).unwrap(),
        nested.join("tool")
    );
    assert!(matches!(
        resolve_executable(Path::new("sh"), fixture.path(), &[]),
        Err(ResolutionError::NotFound)
    ));
}

#[test]
fn symlinks_and_parent_components_follow_filesystem_semantics() {
    let fixture = Fixture::new();
    let real = fixture.path().join("real");
    let inner = real.join("inner");
    fs::create_dir_all(&inner).unwrap();
    executable(&real.join("tool"));
    executable(&fixture.path().join("tool"));
    symlink("real/inner", fixture.path().join("alias")).unwrap();
    assert_eq!(
        resolve_executable(Path::new("alias/../tool"), fixture.path(), &[]).unwrap(),
        real.join("tool")
    );
    symlink("real/tool", fixture.path().join("symlink-tool")).unwrap();
    assert_eq!(
        resolve_executable(Path::new("./symlink-tool"), fixture.path(), &[]).unwrap(),
        real.join("tool")
    );
}

#[test]
fn literal_unicode_spaces_and_shell_characters_are_not_expanded() {
    let fixture = Fixture::new();
    let name = Path::new("tool $(must-not-execute) space ☃");
    executable(&fixture.path().join(name));
    assert_eq!(
        resolve_executable(name, fixture.path(), &[PathBuf::new()]).unwrap(),
        fixture.path().join(name)
    );
    assert!(!fixture.path().join("must-not-execute").exists());
    assert!(
        matches!(resolve_executable(Path::new("~/tool"), fixture.path(), &[]),
        Err(ResolutionError::Io { source, .. }) if source.kind() == io::ErrorKind::NotFound)
    );
}

#[test]
fn invalid_inputs_are_typed_and_unused_directories_are_not_consulted() {
    let fixture = Fixture::new();
    for name in [
        Path::new(""),
        Path::new("."),
        Path::new(".."),
        Path::new("tool\0name"),
    ] {
        assert!(matches!(
            resolve_executable(name, fixture.path(), &[]),
            Err(ResolutionError::InvalidRequest)
        ));
    }
    for cwd in [Path::new("relative"), Path::new("/invalid\0directory")] {
        assert!(matches!(
            resolve_executable(Path::new("tool"), cwd, &[]),
            Err(ResolutionError::InvalidWorkingDirectory)
        ));
    }
    let path = fixture.path().join("tool");
    executable(&path);
    let directories = [PathBuf::new(), PathBuf::from("invalid\0directory")];
    assert!(matches!(
        resolve_executable(Path::new("tool"), fixture.path(), &directories),
        Err(ResolutionError::InvalidSearchDirectory { index: 1 })
    ));
    assert_eq!(
        resolve_executable(&path, fixture.path(), &directories).unwrap(),
        path
    );
    // Non-UTF-8 bytes are not converted to lossy text or confused with NUL.
    let name = PathBuf::from(OsString::from_vec(vec![b't', 0xff]));
    assert!(matches!(
        resolve_executable(&name, fixture.path(), &[]),
        Err(ResolutionError::NotFound)
    ));
}

#[test]
fn explicit_paths_preserve_nonregular_permission_and_missing_failures() {
    let fixture = Fixture::new();
    assert!(matches!(
        resolve_executable(fixture.path(), fixture.path(), &[]),
        Err(ResolutionError::NotRegularFile)
    ));
    let path = fixture.path().join("data");
    fs::write(&path, b"data").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(
        resolve_executable(&path, fixture.path(), &[]),
        Err(ResolutionError::NotExecutable)
    ));
    assert!(
        matches!(resolve_executable(&fixture.path().join("absent"), fixture.path(), &[]),
        Err(ResolutionError::Io { directory: None, source }) if source.kind() == io::ErrorKind::NotFound)
    );
}

#[test]
fn search_stops_on_filesystem_errors_instead_of_selecting_a_later_tool() {
    let fixture = Fixture::new();
    let first = fixture.path().join("first");
    let later = fixture.path().join("later");
    fs::create_dir(&first).unwrap();
    fs::create_dir(&later).unwrap();
    symlink("tool", first.join("tool")).unwrap();
    executable(&later.join("tool"));
    assert!(
        matches!(resolve_executable(Path::new("tool"), fixture.path(), &[first, later.clone()]),
        Err(ResolutionError::Io { directory: Some(0), source }) if source.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()))
    );
    let file = fixture.path().join("not-a-directory");
    executable(&file);
    assert!(
        matches!(resolve_executable(Path::new("tool"), fixture.path(), &[file, later]),
        Err(ResolutionError::Io { directory: Some(0), source }) if source.raw_os_error() == Some(rustix::io::Errno::NOTDIR.raw_os_error()))
    );
}

#[test]
fn selected_candidate_must_be_admitted_without_falling_back_after_digest_failure() {
    let fixture = Fixture::new();
    let first = fixture.path().join("first");
    let later = fixture.path().join("later");
    fs::create_dir(&first).unwrap();
    fs::create_dir(&later).unwrap();
    crate::testing::write_executable_script(&first.join("tool"), fs::read(tool_path()).unwrap())
        .unwrap();
    crate::testing::write_executable_script(&later.join("tool"), fs::read(tool_path()).unwrap())
        .unwrap();
    let marker = fixture.path().join("invocations");
    let environment = [(
        OsString::from("FIXTURE_MARKER"),
        marker.as_os_str().to_owned(),
    )];
    let selected =
        resolve_executable(Path::new("tool"), fixture.path(), &[first.clone(), later]).unwrap();
    assert_eq!(selected, first.join("tool"));
    let spec = VersionSpec {
        executable: &selected,
        executable_bytes: 1024 * 1024,
        version_arguments: &[OsString::from("--version")],
    };
    assert!(matches!(
        AdmittedTool::admit(
            &spec,
            Sha256Digest::compute(b"wrong authority"),
            VERSION,
            &fixture.context(&environment),
            LIMITS
        ),
        Err(ToolError::Artifact(ArtifactError::DigestMismatch { .. }))
    ));
    assert!(!marker.exists());
}
