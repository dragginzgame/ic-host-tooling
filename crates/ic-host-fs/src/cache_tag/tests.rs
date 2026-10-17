use super::*;
use crate::test_support::Fixture;
use std::fs;

const SUFFIX: &[u8] = b"\n# caller-selected cache\n";

#[test]
fn valid_tags_preserve_signature_only_crlf_and_large_trailers() {
    let fixture = Fixture::new();
    let tag = fixture.root.join("CACHEDIR.TAG");
    for contents in [
        SIGNATURE.to_owned(),
        format!("{SIGNATURE}\r\n# another application\r\n"),
        format!("{SIGNATURE}\n# {}\n", "comment".repeat(100_000)),
    ] {
        fs::write(&tag, &contents).unwrap();
        ensure(&fixture.root, SUFFIX, 0o600).unwrap();
        assert_eq!(fs::read_to_string(&tag).unwrap(), contents);
    }
}

#[test]
fn missing_and_invalid_tags_are_replaced_with_exact_caller_bytes() {
    let fixture = Fixture::new();
    let tag = fixture.root.join("CACHEDIR.TAG");
    for contents in [
        None,
        Some(""),
        Some(&SIGNATURE[..42]),
        Some("Signature: incorrect"),
    ] {
        if let Some(contents) = contents {
            fs::write(&tag, contents).unwrap();
        }
        ensure(&fixture.root, SUFFIX, 0o600).unwrap();
        assert_eq!(
            fs::read(&tag).unwrap(),
            [SIGNATURE.as_bytes(), SUFFIX].concat()
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            assert_eq!(
                fs::metadata(&tag).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}

#[test]
#[cfg(unix)]
fn symlink_tags_are_replaced_without_changing_referents() {
    let fixture = Fixture::new();
    let referent = fixture.root.join("other-application-tag");
    let contents = format!("{SIGNATURE}\n# Preserve this file\n");
    fs::write(&referent, &contents).unwrap();
    let tag = fixture.root.join("CACHEDIR.TAG");
    std::os::unix::fs::symlink(&referent, &tag).unwrap();
    ensure(&fixture.root, SUFFIX, 0o600).unwrap();
    assert!(fs::symlink_metadata(&tag).unwrap().file_type().is_file());
    assert_eq!(fs::read_to_string(&referent).unwrap(), contents);
    assert_eq!(
        fs::read(&tag).unwrap(),
        [SIGNATURE.as_bytes(), SUFFIX].concat()
    );
}

#[test]
fn directory_refusal_retains_the_original_publication_error() {
    let fixture = Fixture::new();
    let tag = fixture.root.join("CACHEDIR.TAG");
    fs::create_dir(&tag).unwrap();
    let error = ensure(&fixture.root, SUFFIX, 0o600).unwrap_err();
    assert!(matches!(
        error,
        NamedWriteError::BeforePublication {
            cleanup_error: None,
            ..
        }
    ));
    assert!(tag.is_dir());
}
