use super::*;
use crate::test_support::Fixture;
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn writes_in_place_and_preserves_unprotected_symlinks_and_permissions() {
    let fixture = Fixture::new();
    let protected = fixture.root.join("protected");
    let output = fixture.root.join("exports/output");
    fs::write(&protected, b"retained").unwrap();
    write_non_aliasing(&output, b"long initial output", &[&protected], 64).unwrap();
    fs::set_permissions(&output, fs::Permissions::from_mode(0o640)).unwrap();
    let inode = fs::metadata(&output).unwrap().ino();
    let link = fixture.root.join("link");
    symlink("exports/output", &link).unwrap();
    write_non_aliasing(&link, b"short", &[&protected], 64).unwrap();
    assert_eq!(fs::read(&output).unwrap(), b"short");
    assert_eq!(fs::read(&protected).unwrap(), b"retained");
    assert!(fs::symlink_metadata(link).unwrap().is_symlink());
    let metadata = fs::metadata(output).unwrap();
    assert_eq!(metadata.ino(), inode);
    assert_eq!(metadata.mode() & 0o777, 0o640);
}

#[test]
fn rejects_existing_path_and_inode_aliases() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.root.join("cache")).unwrap();
    let protected = fixture.root.join("cache/catalog");
    fs::write(&protected, b"retained").unwrap();
    let link = fixture.root.join("link");
    symlink("cache/catalog", &link).unwrap();
    let parent_link = fixture.root.join("cache-link");
    symlink("cache", &parent_link).unwrap();
    let hard_link = fixture.root.join("hard-link");
    fs::hard_link(&protected, &hard_link).unwrap();
    let cwd = std::env::current_dir().unwrap().canonicalize().unwrap();
    let mut relative = PathBuf::new();
    for _ in cwd.components().skip(1) {
        relative.push("..");
    }
    relative.push(protected.strip_prefix("/").unwrap());
    for output in [
        protected.clone(),
        fixture.root.join("cache/../cache/catalog"),
        relative,
        link,
        parent_link.join("catalog"),
        hard_link,
    ] {
        let error = write_non_aliasing(&output, b"replacement", &[&protected], 64).unwrap_err();
        assert!(matches!(
            error,
            NonAliasingWriteError::Alias { protected: path, .. } if path == protected
        ));
        assert_eq!(fs::read(&protected).unwrap(), b"retained");
    }
}

#[test]
fn refuses_missing_aliases_before_parent_creation() {
    let fixture = Fixture::new();
    let protected = fixture.root.join("cache/catalog");
    let link = fixture.root.join("link");
    symlink("cache/catalog", &link).unwrap();
    let parent_link = fixture.root.join("cache-link");
    symlink("cache", &parent_link).unwrap();
    for output in [link, parent_link.join("catalog"), protected.clone()] {
        assert!(matches!(
            write_non_aliasing(&output, b"replacement", &[&protected], 64),
            Err(NonAliasingWriteError::Alias { .. })
        ));
        assert!(!protected.parent().unwrap().exists());
    }
}

#[test]
fn enforces_symlink_expansion_limits_before_mutation() {
    let fixture = Fixture::new();
    let protected = fixture.root.join("cache/catalog");
    for index in 0..64 {
        symlink(
            format!("missing/../link-{}", index + 1),
            fixture.root.join(format!("link-{index}")),
        )
        .unwrap();
    }
    symlink("cache/catalog", fixture.root.join("link-64")).unwrap();
    assert!(matches!(
        validate_non_aliasing(&fixture.root.join("link-1"), &[&protected], 64),
        Err(NonAliasingWriteError::Alias { .. })
    ));
    let error =
        write_non_aliasing(&fixture.root.join("link-0"), b"x", &[&protected], 64).unwrap_err();
    assert!(matches!(
        error,
        NonAliasingWriteError::Write { source, .. } if source.kind() == io::ErrorKind::InvalidInput
    ));
    assert!(!protected.parent().unwrap().exists());
}

#[test]
fn retains_traversal_errors_and_existing_bytes() {
    let fixture = Fixture::new();
    let file = fixture.root.join("file");
    let protected = fixture.root.join("cache/catalog");
    fs::write(&file, b"retained").unwrap();
    for path in ["file/child", "missing/../file/child"] {
        let error =
            write_non_aliasing(&fixture.root.join(path), b"x", &[&protected], 64).unwrap_err();
        assert!(matches!(
            error,
            NonAliasingWriteError::Write { source, .. }
                if source.kind() == io::ErrorKind::NotADirectory
        ));
        assert_eq!(fs::read(&file).unwrap(), b"retained");
        assert!(!protected.parent().unwrap().exists());
    }
}

#[test]
fn opened_descriptor_is_checked_after_pathname_replacement() {
    let fixture = Fixture::new();
    let output = fixture.root.join("output");
    let protected = fixture.root.join("protected");
    fs::write(&output, b"retained").unwrap();
    validate_non_aliasing(&output, &[&protected], 64).unwrap();
    let mut descriptor = fs::OpenOptions::new().write(true).open(&output).unwrap();
    fs::hard_link(&output, &protected).unwrap();
    fs::rename(&output, fixture.root.join("original")).unwrap();
    fs::write(&output, b"replacement pathname").unwrap();
    validate_non_aliasing(&output, &[&protected], 64).unwrap();
    assert!(matches!(
        write_opened(&mut descriptor, &output, b"destructive", &[&protected]),
        Err(NonAliasingWriteError::Alias { .. })
    ));
    assert_eq!(fs::read(protected).unwrap(), b"retained");
    assert_eq!(fs::read(output).unwrap(), b"replacement pathname");
}

#[test]
fn opened_write_failure_retains_its_stage_and_cause() {
    let fixture = Fixture::new();
    let output = fixture.root.join("output");
    fs::write(&output, b"retained").unwrap();
    let mut read_only = fs::File::open(&output).unwrap();
    let error = write_opened(&mut read_only, &output, b"x", &[]).unwrap_err();
    assert!(std::error::Error::source(&error).is_some());
    assert!(matches!(error, NonAliasingWriteError::Write { path, .. } if path == output));
    assert_eq!(fs::read(output).unwrap(), b"retained");
}
