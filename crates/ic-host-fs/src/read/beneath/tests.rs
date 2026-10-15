use super::*;
use crate::test_support::Fixture;
use std::{fs, io::Read, os::unix::fs::symlink};

#[test]
fn descriptor_selection_survives_path_replacement_and_bounds_reads() {
    let fixture = Fixture::new();
    let dir = &fixture.root;
    fs::create_dir(dir.join("input")).unwrap();
    fs::write(dir.join("input/body"), b"original").unwrap();
    let root = File::open(dir).unwrap();
    let mut body = open_regular_file_beneath(root.as_fd(), Path::new("input/body"), 8).unwrap();
    fs::rename(dir.join("input"), dir.join("retained")).unwrap();
    fs::create_dir(dir.join("input")).unwrap();
    fs::write(dir.join("input/body"), b"replacement").unwrap();
    let mut bytes = Vec::new();
    body.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"original");
    assert!(open_regular_file_beneath(root.as_fd(), Path::new("input/body"), 8).is_err());
    drop(body);
    drop(root);
    drop(fixture);
}

#[test]
fn traversal_aliases_special_files_and_missing_inputs_fail_closed() {
    let fixture = Fixture::new();
    let dir = &fixture.root;
    fs::create_dir(dir.join("input")).unwrap();
    fs::write(dir.join("input/body"), b"data").unwrap();
    symlink("input", dir.join("alias")).unwrap();
    symlink("body", dir.join("input/link")).unwrap();
    assert!(
        std::process::Command::new("/usr/bin/mkfifo")
            .arg(dir.join("input/fifo"))
            .status()
            .unwrap()
            .success()
    );
    let root = File::open(dir).unwrap();
    for path in [
        "",
        "/input/body",
        "../body",
        "input/../input/body",
        "alias/body",
        "input/link",
    ] {
        assert!(
            matches!(open_regular_file_beneath(root.as_fd(), Path::new(path), 100),
            Err(ArtifactError::Io(ref e)) if e.kind() == io::ErrorKind::InvalidInput)
        );
    }
    for path in ["input", "input/fifo"] {
        assert!(matches!(
            open_regular_file_beneath(root.as_fd(), Path::new(path), 100),
            Err(ArtifactError::NotRegularFile)
        ));
    }
    assert!(
        matches!(open_regular_file_beneath(root.as_fd(), Path::new("missing/body"), 100),
        Err(ArtifactError::Io(ref e)) if e.kind() == io::ErrorKind::NotFound)
    );
    assert_eq!(fs::read(dir.join("input/body")).unwrap(), b"data");
    drop(root);
    drop(fixture);
}

#[test]
fn root_opening_refuses_aliases_files_and_fifos_without_waiting() {
    let fixture = Fixture::new();
    let dir = &fixture.root;
    fs::create_dir(dir.join("root")).unwrap();
    fs::write(dir.join("file"), b"data").unwrap();
    symlink("root", dir.join("alias")).unwrap();
    assert!(
        std::process::Command::new("/usr/bin/mkfifo")
            .arg(dir.join("fifo"))
            .status()
            .unwrap()
            .success()
    );
    assert!(open_directory_no_follow(&dir.join("root")).is_ok());
    for name in ["alias", "file", "fifo"] {
        assert!(open_directory_no_follow(&dir.join(name)).is_err());
    }
}
