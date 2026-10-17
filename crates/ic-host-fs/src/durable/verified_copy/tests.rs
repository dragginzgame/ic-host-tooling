use super::*;
use ic_host_artifacts::artifact::ArtifactError;
use std::fs;

#[test]
fn verified_copy_preserves_prior_bytes_on_digest_limit_and_source_refusal() {
    let fixture = crate::test_support::Fixture::new();
    let source = fixture.root.join("source");
    let output = fixture.root.join("output");
    fs::write(&source, b"complete").unwrap();
    fs::write(&output, b"prior").unwrap();
    let options = WriteOptions {
        mode: super::super::PublicationMode::Replace,
        permissions: 0o600,
    };
    assert!(matches!(
        copy_verified(
            &source,
            &output,
            Sha256Digest::compute(b"wrong"),
            8,
            options
        ),
        Err(NamedWriteError::Producer {
            source: CopyError::Input(ArtifactError::DigestMismatch { .. }),
            cleanup_error: None
        })
    ));
    assert_eq!(fs::read(&output).unwrap(), b"prior");
    assert!(
        copy_verified(
            &source,
            &output,
            Sha256Digest::compute(b"complete"),
            7,
            options
        )
        .is_err()
    );
    assert_eq!(fs::read(&output).unwrap(), b"prior");
    assert!(
        copy_verified(
            &fixture.root,
            &output,
            Sha256Digest::compute(b""),
            8,
            options
        )
        .is_err()
    );
    assert_eq!(fs::read(&output).unwrap(), b"prior");
    let identity = copy_verified(
        &source,
        &output,
        Sha256Digest::compute(b"complete"),
        8,
        options,
    )
    .unwrap();
    assert_eq!(identity.bytes, 8);
    assert_eq!(fs::read(&output).unwrap(), b"complete");
    let create = WriteOptions {
        mode: super::super::PublicationMode::CreateNew,
        ..options
    };
    assert!(matches!(
        copy_verified(&source, &output, identity.sha256, 8, create),
        Err(NamedWriteError::BeforePublication { .. })
    ));
    fs::write(&source, b"").unwrap();
    assert_eq!(
        copy_verified(&source, &output, Sha256Digest::compute(b""), 0, options)
            .unwrap()
            .bytes,
        0
    );
    assert_eq!(fs::read(&output).unwrap(), [] as [u8; 0]);
    assert_eq!(fs::read_dir(&fixture.root).unwrap().count(), 2);
}

#[cfg(unix)]
#[test]
fn verified_copy_refuses_fifo_without_a_writer_and_follows_selected_symlinks() {
    use std::os::unix::fs::symlink;
    let fixture = crate::test_support::Fixture::new();
    let source = fixture.root.join("source");
    let alias = fixture.root.join("alias");
    let fifo = fixture.root.join("fifo");
    let output = fixture.root.join("output");
    fs::write(&source, b"bytes").unwrap();
    symlink(&source, &alias).unwrap();
    crate::test_support::create_fifo(&fifo, None).unwrap();
    let options = WriteOptions {
        mode: super::super::PublicationMode::Replace,
        permissions: 0o600,
    };
    assert!(matches!(
        copy_verified(&fifo, &output, Sha256Digest::compute(b"bytes"), 5, options),
        Err(NamedWriteError::Producer {
            source: CopyError::Input(ArtifactError::NotRegularFile),
            ..
        })
    ));
    assert!(!output.exists());
    copy_verified(&alias, &output, Sha256Digest::compute(b"bytes"), 5, options).unwrap();
    assert_eq!(fs::read(&output).unwrap(), b"bytes");
}
