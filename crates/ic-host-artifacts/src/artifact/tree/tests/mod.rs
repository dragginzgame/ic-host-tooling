//! Regression framing extracted from IC Backup's public directory-checksum cases.
use super::*;

fn entry(name: &str, bytes: &[u8]) -> (PathBuf, Sha256Digest) {
    (name.into(), Sha256Digest::compute(bytes))
}

#[test]
fn declared_files_keep_original_golden_order_and_empty_framing() {
    let files = vec![entry("a.txt", b"a"), entry("nested/b.txt", b"b")];
    let original = checksum_relative_files(files.clone()).unwrap();
    assert_eq!(
        original.to_string(),
        "e4d330f138b8f1b3044e84b5dcbe4fd1cb7e043d0c20810c083d791b6de01266"
    );
    let mut reversed = files;
    reversed.reverse();
    assert_eq!(checksum_relative_files(reversed).unwrap(), original);
    assert_eq!(
        checksum_relative_files(Vec::new()).unwrap(),
        Sha256Digest::compute(&[])
    );
    assert_eq!(
        checksum_relative_files(vec![entry("a-file", b"b"), entry("a/leaf", b"a")])
            .unwrap()
            .to_string(),
        "8ebe1da155356d34952f1515a80ee0c223ee530c1f87f6d10efa48f875b7c9b0"
    );
}

#[cfg(unix)]
#[test]
fn exact_unicode_and_legal_filename_bytes_keep_canonical_digest_text() {
    let files = vec![
        entry("slash\\name", b"three"),
        entry("line\nname", b"two"),
        entry("café/雪", b"one"),
    ];
    assert_eq!(
        checksum_relative_files(files).unwrap().to_string(),
        "95a10c85036a57ecbae066721f0b5b47b69617f8c5d1788e694c22bbbd1b0dc0"
    );
    assert_ne!(
        checksum_relative_files(vec![entry("café", b"same")]).unwrap(),
        checksum_relative_files(vec![entry("cafe\u{301}", b"same")]).unwrap()
    );
}

#[test]
fn malformed_and_duplicate_paths_return_exact_typed_refusals() {
    for name in [
        "",
        "/absolute",
        ".",
        "..",
        "./a",
        "a/./b",
        "a/../b",
        "a//b",
        "a/",
        "a\0b",
    ] {
        assert_eq!(
            checksum_relative_files(vec![entry(name, b"a")]).unwrap_err(),
            DirectoryChecksumError::InvalidRelativePath { path: name.into() }
        );
    }
    for bytes in [b"a".as_slice(), b"different".as_slice()] {
        assert_eq!(
            checksum_relative_files(vec![entry("a", b"a"), entry("a", bytes)]).unwrap_err(),
            DirectoryChecksumError::DuplicatePath { path: "a".into() }
        );
    }
    let error = DirectoryChecksumError::DuplicatePath { path: "a".into() };
    assert_eq!(
        error.to_string(),
        "duplicate directory checksum path: \"a\""
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[cfg(unix)]
#[test]
fn non_utf8_identity_rejects_without_lossy_substitution() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let path = PathBuf::from(OsString::from_vec(vec![0xff]));
    assert_eq!(
        checksum_relative_files(vec![(path.clone(), Sha256Digest::compute(b"a"))]).unwrap_err(),
        DirectoryChecksumError::NonUtf8Path { path }
    );
    checksum_relative_files(vec![entry("\u{fffd}", b"a")]).unwrap();
}
