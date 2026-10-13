use super::*;
use crate::test_support::Fixture;
use std::{
    fs,
    io::Write as _,
    os::{
        fd::AsFd as _,
        unix::fs::{PermissionsExt as _, symlink},
    },
};

#[test]
fn dropped_stage_is_retained_and_verified_stage_publishes_under_held_parent() {
    let fixture = Fixture::new();
    let parent_path = fixture.root.join("parent");
    fs::create_dir(&parent_path).unwrap();
    let parent = File::open(&parent_path).unwrap();
    let mut failed = create_retained_at(parent.as_fd(), OsStr::new("failed.part"), 0o600).unwrap();
    failed.write_all(b"partial evidence").unwrap();
    drop(failed);
    assert_eq!(
        fs::read(parent_path.join("failed.part")).unwrap(),
        b"partial evidence"
    );
    assert!(create_retained_at(parent.as_fd(), OsStr::new("failed.part"), 0o600).is_err());
    let mut body = create_retained_at(parent.as_fd(), OsStr::new("body.part"), 0o600).unwrap();
    body.write_all(b"verified").unwrap();
    assert_eq!(body.metadata().unwrap().permissions().mode() & 0o777, 0o600);
    let moved = fixture.root.join("moved");
    fs::rename(&parent_path, &moved).unwrap();
    fs::create_dir(&parent_path).unwrap();
    publish_retained_at(
        parent.as_fd(),
        OsStr::new("body.part"),
        OsStr::new("body.bin"),
        body,
    )
    .unwrap();
    assert_eq!(fs::read(moved.join("body.bin")).unwrap(), b"verified");
    assert!(!moved.join("body.part").exists());
    assert!(!parent_path.join("body.bin").exists());
}

#[test]
fn occupied_destination_and_substituted_stage_preserve_evidence() {
    let fixture = Fixture::new();
    let parent = File::open(&fixture.root).unwrap();
    for name in ["occupied", "substituted", "linked"] {
        let mut file = create_retained_at(parent.as_fd(), OsStr::new(name), 0o600).unwrap();
        file.write_all(b"original").unwrap();
        let target = format!("{name}.target");
        match name {
            "occupied" => fs::write(fixture.root.join(&target), b"prior").unwrap(),
            "substituted" => {
                fs::rename(fixture.root.join(name), fixture.root.join("original")).unwrap();
                symlink("original", fixture.root.join(name)).unwrap();
            }
            "linked" => fs::hard_link(fixture.root.join(name), fixture.root.join("extra")).unwrap(),
            _ => unreachable!(),
        }
        assert!(matches!(
            publish_retained_at(parent.as_fd(), OsStr::new(name), OsStr::new(&target), file),
            Err(NamedWriteError::BeforePublication { .. })
        ));
        assert_eq!(fs::read(fixture.root.join(name)).unwrap(), b"original");
        if name == "occupied" {
            assert_eq!(fs::read(fixture.root.join(&target)).unwrap(), b"prior");
        } else {
            assert!(!fixture.root.join(&target).exists());
        }
    }
}

#[test]
fn invalid_names_permissions_and_non_directory_refuse_without_creation() {
    let fixture = Fixture::new();
    let parent = File::open(&fixture.root).unwrap();
    for name in ["", ".", "..", "../escape", "a/b", "nul\0name"] {
        assert!(create_retained_at(parent.as_fd(), OsStr::new(name), 0o600).is_err());
    }
    assert!(create_retained_at(parent.as_fd(), OsStr::new("bad-mode"), 0o1600).is_err());
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 0);
    let file = create_retained_at(parent.as_fd(), OsStr::new("file"), 0o600).unwrap();
    assert!(create_retained_at(file.as_fd(), OsStr::new("child"), 0o600).is_err());
}
