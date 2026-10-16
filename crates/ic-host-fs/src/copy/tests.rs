use super::*;
use crate::{
    read::open_directory_no_follow,
    test_support::{Fixture, create_fifo},
};
use ic_host_artifacts::artifact::hash_reader;
use std::{
    ffi::OsStr,
    fs,
    io::{Seek, SeekFrom},
    os::unix::{
        ffi::OsStrExt,
        fs::{PermissionsExt, symlink},
    },
};

#[test]
fn regular_copy_preserves_offset_bytes_identity_and_private_create_new_output() {
    let fixture = Fixture::new();
    let input = fixture.root.join("input");
    fs::write(&input, b"skip exact payload").unwrap();
    let mut source = File::open(&input).unwrap();
    source.seek(SeekFrom::Start(5)).unwrap();
    let output = fixture.root.join("output");
    let identity = copy_file(&source, &output, 13).unwrap();
    assert_eq!(fs::read(&output).unwrap(), b"exact payload");
    assert_eq!(
        identity,
        hash_reader(b"exact payload".as_slice(), 13).unwrap()
    );
    assert_eq!(source.stream_position().unwrap(), 18);
    assert_eq!(
        fs::metadata(&output).unwrap().permissions().mode() & 0o077,
        0
    );
    assert_eq!(
        copy_file(&source, &output, 13).unwrap_err().kind(),
        io::ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read(&output).unwrap(), b"exact payload");
    let link = fixture.root.join("link");
    symlink(&input, &link).unwrap();
    assert_eq!(
        copy_file(&source, &link, 13).unwrap_err().kind(),
        io::ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read(&input).unwrap(), b"skip exact payload");
    let directory = open_directory_no_follow(&fixture.root).unwrap();
    assert_eq!(
        copy_file(&directory, &fixture.root.join("invalid"), 13)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    assert!(!fixture.root.join("invalid").exists());
}

#[test]
fn held_tree_copy_retains_empty_directories_and_exact_relative_file_identities() {
    let fixture = Fixture::new();
    let source = fixture.root.join("source");
    fs::create_dir_all(source.join("nested/empty")).unwrap();
    fs::write(source.join("a"), b"alpha").unwrap();
    fs::write(source.join("nested/b"), b"beta").unwrap();
    let held: OwnedFd = open_directory_no_follow(&source).unwrap().into();
    fs::rename(&source, fixture.root.join("retained-source")).unwrap();
    fs::create_dir(&source).unwrap();
    fs::write(source.join("a"), b"replacement").unwrap();
    let output = fixture.root.join("output");
    let mut files = copy_directory(&held, &output, &source, 5).unwrap();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        files,
        vec![
            (
                PathBuf::from("a"),
                hash_reader(b"alpha".as_slice(), 5).unwrap()
            ),
            (
                PathBuf::from("nested/b"),
                hash_reader(b"beta".as_slice(), 5).unwrap()
            ),
        ]
    );
    assert_eq!(fs::read(output.join("a")).unwrap(), b"alpha");
    assert_eq!(fs::read(output.join("nested/b")).unwrap(), b"beta");
    assert!(output.join("nested/empty").is_dir());
    for path in [
        &output,
        &output.join("nested"),
        &output.join("nested/empty"),
    ] {
        assert_eq!(fs::metadata(path).unwrap().permissions().mode() & 0o077, 0);
    }
    assert!(
        matches!(copy_directory(&held, &output, &source, 5), Err(TreeCopyError::Directory(DirectoryCreationError::BeforeCreation(error))) if error.kind() == io::ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(output.join("a")).unwrap(), b"alpha");
}

#[test]
fn rejected_entries_and_byte_limits_retain_output_without_following_or_cleanup() {
    for entry in ["link", "fifo", "non-utf8", "oversized"] {
        let fixture = Fixture::new();
        let source = fixture.root.join("source");
        fs::create_dir(&source).unwrap();
        let target = fixture.root.join("target");
        fs::write(&target, b"must remain").unwrap();
        match entry {
            "link" => symlink(&target, source.join("entry")).unwrap(),
            "fifo" => create_fifo(&source.join("entry"), None).unwrap(),
            "non-utf8" => fs::write(source.join(OsStr::from_bytes(b"\xff")), b"x").unwrap(),
            _ => fs::write(source.join("entry"), b"too large").unwrap(),
        }
        let held: OwnedFd = open_directory_no_follow(&source).unwrap().into();
        let output = fixture.root.join("output");
        let error = copy_directory(&held, &output, Path::new("display"), 1).unwrap_err();
        match entry {
            "link" | "fifo" => assert!(
                matches!(error, TreeCopyError::UnsupportedEntry { path, kind } if path == Path::new("display/entry") && kind == if entry == "link" { FileType::Symlink } else { FileType::Fifo })
            ),
            "non-utf8" => assert!(
                matches!(error, TreeCopyError::NonUtf8Path { path } if path == Path::new("display").join(OsStr::from_bytes(b"\xff")))
            ),
            _ => {
                let TreeCopyError::Io(error) = error else {
                    panic!("bounded copy error");
                };
                assert!(
                    error
                        .get_ref()
                        .unwrap()
                        .is::<ic_host_artifacts::artifact::CopyError>()
                );
                assert!(output.join("entry").is_file());
            }
        }
        assert!(output.is_dir());
        assert_eq!(fs::read(&target).unwrap(), b"must remain");
    }
}
