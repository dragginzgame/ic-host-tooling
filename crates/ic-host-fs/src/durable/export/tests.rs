use super::*;
use crate::test_support::fixture;
use std::os::unix::fs::{MetadataExt as _, PermissionsExt, symlink};

#[test]
fn writes_in_place_and_preserves_unprotected_symlinks_and_permissions() {
    let fixture = fixture();
    let protected = fixture.path().join("protected");
    let output = fixture.path().join("exports/output");
    fs::write(&protected, b"retained").unwrap();
    write_non_aliasing(&output, b"long initial output", &[&protected], 64).unwrap();
    fs::set_permissions(&output, fs::Permissions::from_mode(0o640)).unwrap();
    let inode = fs::metadata(&output).unwrap().ino();
    let link = fixture.path().join("link");
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
    let fixture = fixture();
    fs::create_dir(fixture.path().join("cache")).unwrap();
    let protected = fixture.path().join("cache/catalog");
    fs::write(&protected, b"retained").unwrap();
    let link = fixture.path().join("link");
    symlink("cache/catalog", &link).unwrap();
    let parent_link = fixture.path().join("cache-link");
    symlink("cache", &parent_link).unwrap();
    let hard_link = fixture.path().join("hard-link");
    fs::hard_link(&protected, &hard_link).unwrap();
    let cwd = std::env::current_dir().unwrap().canonicalize().unwrap();
    let mut relative = PathBuf::new();
    for _ in cwd.components().skip(1) {
        relative.push("..");
    }
    relative.push(protected.strip_prefix("/").unwrap());
    for output in [
        protected.clone(),
        fixture.path().join("cache/../cache/catalog"),
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
    let fixture = fixture();
    let protected = fixture.path().join("cache/catalog");
    let link = fixture.path().join("link");
    symlink("cache/catalog", &link).unwrap();
    let parent_link = fixture.path().join("cache-link");
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
    let fixture = fixture();
    let protected = fixture.path().join("cache/catalog");
    for index in 0..64 {
        symlink(
            format!("missing/../link-{}", index + 1),
            fixture.path().join(format!("link-{index}")),
        )
        .unwrap();
    }
    symlink("cache/catalog", fixture.path().join("link-64")).unwrap();
    assert!(matches!(
        validate_non_aliasing(&fixture.path().join("link-1"), &[&protected], 64),
        Err(NonAliasingWriteError::Alias { .. })
    ));
    let error =
        write_non_aliasing(&fixture.path().join("link-0"), b"x", &[&protected], 64).unwrap_err();
    assert!(matches!(
        error,
        NonAliasingWriteError::Write { source, .. } if source.kind() == io::ErrorKind::InvalidInput
    ));
    assert!(!protected.parent().unwrap().exists());
}

#[test]
fn retains_traversal_errors_and_existing_bytes() {
    let fixture = fixture();
    let file = fixture.path().join("file");
    let protected = fixture.path().join("cache/catalog");
    fs::write(&file, b"retained").unwrap();
    for path in ["file/child", "missing/../file/child"] {
        let error =
            write_non_aliasing(&fixture.path().join(path), b"x", &[&protected], 64).unwrap_err();
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
    let fixture = fixture();
    let output = fixture.path().join("output");
    let protected = fixture.path().join("protected");
    fs::write(&output, b"retained").unwrap();
    validate_non_aliasing(&output, &[&protected], 64).unwrap();
    let mut descriptor = fs::OpenOptions::new().write(true).open(&output).unwrap();
    fs::hard_link(&output, &protected).unwrap();
    fs::rename(&output, fixture.path().join("original")).unwrap();
    fs::write(&output, b"replacement pathname").unwrap();
    validate_non_aliasing(&output, &[&protected], 64).unwrap();
    assert!(matches!(
        write_opened(
            &mut descriptor,
            &output,
            b"destructive",
            &[&protected],
            fs::File::sync_all
        ),
        Err(NonAliasingWriteError::Alias { .. })
    ));
    assert_eq!(fs::read(protected).unwrap(), b"retained");
    assert_eq!(fs::read(output).unwrap(), b"replacement pathname");
}

#[test]
fn opened_write_failure_retains_its_stage_and_cause() {
    let fixture = fixture();
    let output = fixture.path().join("output");
    fs::write(&output, b"retained").unwrap();
    let mut read_only = fs::File::open(&output).unwrap();
    let error = write_opened(&mut read_only, &output, b"x", &[], fs::File::sync_all).unwrap_err();
    assert!(std::error::Error::source(&error).is_some());
    assert!(matches!(error, NonAliasingWriteError::Write { path, .. } if path == output));
    assert_eq!(fs::read(output).unwrap(), b"retained");
}

#[test]
fn synchronization_failure_retains_complete_output_and_original_cause() {
    let fixture = fixture();
    let output = fixture.path().join("output");
    fs::write(&output, b"original longer bytes").unwrap();
    let mut file = fs::OpenOptions::new().write(true).open(&output).unwrap();
    let error = write_opened(&mut file, &output, b"new", &[], |opened| {
        assert_eq!(opened.metadata().unwrap().len(), 3);
        assert_eq!(fs::read(&output).unwrap(), b"new");
        Err(io::Error::from_raw_os_error(5))
    })
    .unwrap_err();
    assert!(matches!(error, NonAliasingWriteError::Sync { ref path, .. } if path == &output));
    let source = std::error::Error::source(&error)
        .unwrap()
        .downcast_ref::<io::Error>()
        .unwrap();
    assert_eq!(source.raw_os_error(), Some(5));
    assert_eq!(fs::read(&output).unwrap(), b"new");
}
