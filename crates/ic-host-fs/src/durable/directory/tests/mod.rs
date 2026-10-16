use super::*;
use crate::test_support::Fixture;
use std::os::unix::fs::PermissionsExt as _;

#[test]
fn claim_is_private_exclusive_and_retained_after_drop() {
    let fixture = Fixture::new();
    let path = fixture.root.join("claim");
    let directory = create_private_directory(&path).unwrap();
    assert_eq!(
        directory.metadata().unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert!(
        matches!(create_private_directory(&path), Err(DirectoryCreationError::BeforeCreation(error)) if error.kind() == io::ErrorKind::AlreadyExists)
    );
    admit_directory(&path, &directory).unwrap();
    drop(directory);
    assert!(path.is_dir());
}

#[test]
fn replaced_removed_and_symlink_names_do_not_admit_original_descriptor() {
    let fixture = Fixture::new();
    let path = fixture.root.join("claim");
    let moved = fixture.root.join("moved");
    let directory = create_private_directory(&path).unwrap();
    std::fs::rename(&path, &moved).unwrap();
    assert!(admit_directory(&path, &directory).is_err());
    std::fs::create_dir(&path).unwrap();
    assert!(admit_directory(&path, &directory).is_err());
    std::fs::remove_dir(&path).unwrap();
    std::os::unix::fs::symlink(&moved, &path).unwrap();
    assert!(admit_directory(&path, &directory).is_err());
    assert!(create_private_directory(&path).is_err());
    admit_directory(&moved, &directory).unwrap();
    let file = File::create(fixture.root.join("regular")).unwrap();
    assert!(admit_directory(&moved, &file).is_err());
}

#[test]
fn missing_parent_and_forbidden_namespace_leave_no_claim() {
    let fixture = Fixture::new();
    for path in [
        fixture.root.join("missing/claim"),
        fixture.root.join("bad\nclaim"),
    ] {
        assert!(matches!(
            create_private_directory(&path),
            Err(DirectoryCreationError::BeforeCreation(_))
        ));
        assert!(!path.exists());
    }
}

#[test]
fn directory_commit_reconciles_a_lost_reply_without_replacing_evidence() {
    let fixture = Fixture::new();
    let staging = fixture.root.join("staging");
    let canonical = fixture.root.join("artifact");
    std::fs::create_dir(&staging).unwrap();
    std::fs::write(staging.join("member"), b"original").unwrap();
    let mut prepared = 0;
    let first = commit_existing_directory_with(
        &staging,
        &canonical,
        |held, selected| -> io::Result<()> {
            prepared += 1;
            assert_eq!(selected, staging);
            assert!(held.metadata()?.is_dir());
            assert_eq!(std::fs::read(selected.join("member"))?, b"original");
            std::fs::File::open(selected.join("member"))?.sync_all()?;
            held.sync_all()
        },
        |step, _| {
            if step == DirectoryCommitStep::BeforeParentSync {
                Err(io::Error::other("lost reply after publication"))
            } else {
                Ok(())
            }
        },
    );
    assert!(matches!(
        first,
        Err(DirectoryCommitError::AfterPublication(_))
    ));
    assert_eq!(prepared, 1);
    assert!(!staging.exists());
    assert_eq!(
        std::fs::read(canonical.join("member")).unwrap(),
        b"original"
    );

    let recovered = commit_existing_directory_with(
        &staging,
        &canonical,
        |held, selected| -> io::Result<()> {
            assert_eq!(selected, canonical);
            assert_eq!(std::fs::read(selected.join("member"))?, b"original");
            held.sync_all()
        },
        |_, _| Ok(()),
    )
    .unwrap();
    assert_eq!(recovered, DirectoryCommitOutcome::Recovered);
    assert_eq!(
        std::fs::read(canonical.join("member")).unwrap(),
        b"original"
    );
}

#[test]
fn directory_commit_rejects_changed_conflicting_and_unsafe_trees() {
    let fixture = Fixture::new();
    let staging = fixture.root.join("staging");
    let canonical = fixture.root.join("artifact");
    assert!(matches!(
        commit_existing_directory_with(
            &staging,
            &canonical,
            |_, _| Ok::<(), io::Error>(()),
            |_, _| unreachable!(),
        ),
        Err(DirectoryCommitError::Missing)
    ));
    assert!(matches!(
        commit_existing_directory_with(
            &staging,
            &staging,
            |_, _| Ok::<(), io::Error>(()),
            |_, _| unreachable!(),
        ),
        Err(DirectoryCommitError::InvalidPaths)
    ));
    std::fs::create_dir(&staging).unwrap();
    std::fs::write(staging.join("member"), b"changed").unwrap();
    let rejected = commit_existing_directory_with(
        &staging,
        &canonical,
        |_, selected| {
            assert_eq!(selected, staging);
            Err::<(), _>(io::Error::other("checksum mismatch"))
        },
        |_, _| panic!("rejected bytes must not reach publication"),
    );
    assert!(matches!(rejected, Err(DirectoryCommitError::Prepare(_))));
    assert!(staging.exists());
    assert!(!canonical.exists());

    std::fs::create_dir(&canonical).unwrap();
    let conflicted = commit_existing_directory_with(
        &staging,
        &canonical,
        |_, _| Ok::<(), io::Error>(()),
        |_, _| panic!("conflict must not publish"),
    );
    assert!(matches!(conflicted, Err(DirectoryCommitError::Conflict)));
    std::fs::remove_dir(&canonical).unwrap();
    std::fs::remove_dir_all(&staging).unwrap();
    std::os::unix::fs::symlink(&fixture.root, &canonical).unwrap();
    let unsafe_name = commit_existing_directory_with(
        &staging,
        &canonical,
        |_, _| Ok::<(), io::Error>(()),
        |_, _| panic!("symlink must not publish"),
    );
    assert!(matches!(
        unsafe_name,
        Err(DirectoryCommitError::BeforePublication(_))
    ));
    assert!(
        std::fs::symlink_metadata(&canonical)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}
