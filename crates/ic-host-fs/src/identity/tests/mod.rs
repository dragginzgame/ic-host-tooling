use super::*;
use crate::test_support::fixture;
use std::{fs, os::unix::fs::symlink};

#[test]
fn held_identity_distinguishes_replacement_and_preserves_file_kind_admission() {
    let fixture = fixture();
    let path = fixture.path().join("file");
    fs::write(&path, b"original").unwrap();
    let file = File::open(&path).unwrap();
    let identity = FileIdentity::from_file(&file).unwrap();
    assert_eq!(
        identity,
        FileIdentity::from_metadata(&file.metadata().unwrap())
    );
    let alias = fixture.path().join("alias");
    fs::hard_link(&path, &alias).unwrap();
    assert_eq!(
        identity,
        FileIdentity::from_metadata(&fs::metadata(&alias).unwrap())
    );
    fs::remove_file(&path).unwrap();
    fs::write(&path, b"replacement").unwrap();
    assert_ne!(
        identity,
        FileIdentity::from_metadata(&fs::metadata(&path).unwrap())
    );
    assert_eq!(identity, FileIdentity::from_file(&file).unwrap());

    let link = fixture.path().join("link");
    symlink(&path, &link).unwrap();
    assert_eq!(
        FileIdentity::from_metadata(&fs::metadata(&path).unwrap()),
        FileIdentity::from_metadata(&fs::metadata(&link).unwrap())
    );
    assert_ne!(
        FileIdentity::from_metadata(&fs::symlink_metadata(&link).unwrap()),
        FileIdentity::from_metadata(&fs::metadata(&link).unwrap())
    );
    let directory = File::open(fixture.path()).unwrap();
    assert_eq!(
        FileIdentity::from_file(&directory).unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(
        FileIdentity::from_metadata(&directory.metadata().unwrap()),
        FileIdentity::from_metadata(&fs::metadata(fixture.path()).unwrap())
    );
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        let stat = rustix::fs::fstat(&file).unwrap();
        assert_eq!(identity, FileIdentity::from_stat(&stat));
    }
}
