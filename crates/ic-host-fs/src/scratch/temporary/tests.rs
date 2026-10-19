use super::*;
use crate::test_support::fixture;
use std::{
    fs,
    io::Write as _,
    os::unix::fs::{PermissionsExt as _, symlink},
};

#[test]
fn scoped_files_are_private_unique_and_anchored_to_the_physical_parent() {
    let fixture = fixture();
    let parent = fixture.path().join("physical");
    fs::create_dir(&parent).unwrap();
    let alias = fixture.path().join("alias");
    symlink(&parent, &alias).unwrap();
    let mut file = TemporaryFile::new_in(&alias, "arguments").unwrap();
    let other = TemporaryFile::new_in(&alias, "arguments").unwrap();
    let bytes = [b"binary\0data".as_slice(), &vec![0xff; 128 * 1024]].concat();
    file.as_file_mut().write_all(&bytes).unwrap();
    assert_eq!(file.path().parent(), Some(parent.as_path()));
    assert!(file.path().is_absolute());
    assert_ne!(file.path(), other.path());
    assert_eq!(fs::read(file.path()).unwrap(), bytes);
    assert_eq!(
        file.as_file().metadata().unwrap().permissions().mode() & 0o177,
        0
    );
    let path = file.path().to_path_buf();
    drop(file);
    assert!(!path.exists());
    assert!(other.path().exists());
}

#[test]
fn child_can_read_binary_argument_file_while_the_guard_is_alive() {
    let fixture = fixture();
    let mut file = TemporaryFile::new_in(fixture.path(), "argument").unwrap();
    file.as_file_mut().write_all(b"raw\0\xffbytes").unwrap();
    let output = std::process::Command::new("/bin/cat")
        .arg(file.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"raw\0\xffbytes");
    let path = file.path().to_path_buf();
    file.close().unwrap();
    assert!(!path.exists());
}

#[test]
fn return_errors_and_unwinding_remove_only_the_scoped_file() {
    let fixture = fixture();
    let marker = fixture.path().join("retained");
    fs::write(&marker, b"evidence").unwrap();
    let mut path = None;
    let result: io::Result<()> = (|| {
        let mut file = TemporaryFile::new_in(fixture.path(), "failed").unwrap();
        path = Some(file.path().to_path_buf());
        file.as_file_mut().write_all(b"partial")?;
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    })();
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::BrokenPipe);
    assert!(!path.unwrap().exists());
    let file = TemporaryFile::new_in(fixture.path(), "unwind").unwrap();
    let path = file.path().to_path_buf();
    assert!(
        std::panic::catch_unwind(move || {
            assert!(file.path().exists());
            panic!("exercise scoped cleanup");
        })
        .is_err()
    );
    assert!(!path.exists());
    assert_eq!(fs::read(marker).unwrap(), b"evidence");
}

#[test]
fn explicit_close_preserves_original_removal_error() {
    let fixture = fixture();
    let file = TemporaryFile::new_in(fixture.path(), "missing").unwrap();
    fs::remove_file(file.path()).unwrap();
    assert_eq!(file.close().unwrap_err().kind(), io::ErrorKind::NotFound);
}

#[test]
fn invalid_prefixes_and_missing_parents_create_nothing() {
    let fixture = fixture();
    for prefix in ["", ".", "..", "a/b", "a/", "a\n", "a\r", "a\0"] {
        assert_eq!(
            TemporaryFile::new_in(&fixture.path().join("absent"), prefix)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }
    assert_eq!(
        TemporaryFile::new_in(&fixture.path().join("absent"), "valid")
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
    assert_eq!(
        TemporaryFile::new_in(&fixture.path().join("line\nfeed"), "valid")
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 0);
}

#[test]
fn physical_parent_with_forbidden_characters_is_refused_through_alias() {
    let fixture = fixture();
    let parent = fixture.path().join("invalid\rparent");
    fs::create_dir(&parent).unwrap();
    let alias = fixture.path().join("alias");
    symlink(&parent, &alias).unwrap();
    assert_eq!(
        TemporaryFile::new_in(&alias, "valid").unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(fs::read_dir(parent).unwrap().count(), 0);
}
