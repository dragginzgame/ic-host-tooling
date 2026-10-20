use super::*;

const REGISTRY: &str = concat!(
    "[[package]]\nname='external'\nversion='1.0.0'\nsource='registry+test'\nchecksum='",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef'\n",
);
const GIT: &str = "[[package]]\nname='git'\nversion='0.1.0'\nsource='git+test#1234'\n";

#[test]
fn subsets_allow_unused_originals_local_policy_and_empty_external_graphs() {
    let original = format!("{REGISTRY}{GIT}");
    for candidate in [REGISTRY, GIT, "package=[]", "[[package]]\nname='local'\n"] {
        verify_external_lock_subset(original.as_bytes(), candidate.as_bytes(), original.len())
            .unwrap();
    }
    verify_external_lock_subset(b"package=[]", b"package=[]", 10).unwrap();
    // Identical name/version pairs from distinct sources remain distinct keys.
    let other = GIT.replace("git+test#1234", "git+other#5678");
    verify_external_lock_subset(format!("{GIT}{other}").as_bytes(), other.as_bytes(), 4096)
        .unwrap();
    // These are optional fields for non-registry selections, compared verbatim.
    let checksum = format!("{GIT}checksum='opaque'\n");
    verify_external_lock_subset(checksum.as_bytes(), checksum.as_bytes(), 4096).unwrap();
}

#[test]
fn substitutions_and_duplicates_refuse_without_weakening_checksum_identity() {
    for changed in [
        REGISTRY.replace("external", "new"),
        REGISTRY.replace("1.0.0", "1.0.1"),
        REGISTRY.replace("registry+test", "registry+other"),
        REGISTRY.replace("0123456789abcdef", "fedcba9876543210"),
        GIT.to_owned(),
    ] {
        assert!(matches!(
            verify_external_lock_subset(REGISTRY.as_bytes(), changed.as_bytes(), 4096),
            Err(ExternalLockError::SelectionChanged { .. })
        ));
    }
    for (original, candidate) in [
        (GIT.to_owned(), format!("{GIT}checksum=''\n")),
        (
            format!("{GIT}checksum='one'\n"),
            format!("{GIT}checksum='two'\n"),
        ),
        (format!("{GIT}checksum='one'\n"), GIT.to_owned()),
    ] {
        assert!(matches!(
            verify_external_lock_subset(original.as_bytes(), candidate.as_bytes(), 4096),
            Err(ExternalLockError::SelectionChanged { .. })
        ));
    }
    for input in ["original", "candidate"] {
        let duplicate = format!("{REGISTRY}{REGISTRY}");
        let (original, candidate) = if input == "original" {
            (duplicate.as_bytes(), REGISTRY.as_bytes())
        } else {
            (REGISTRY.as_bytes(), duplicate.as_bytes())
        };
        assert!(matches!(
            verify_external_lock_subset(original, candidate, 4096),
            Err(ExternalLockError::DuplicateSelection { input: role, .. }) if role == input
        ));
    }
}

#[test]
fn bounds_and_malformed_inputs_preserve_role_and_original_error_sources() {
    for (original, candidate, expected) in [
        (
            b"invalid too large".as_slice(),
            b"package=[]".as_slice(),
            "original",
        ),
        (
            b"package=[]".as_slice(),
            b"invalid too large".as_slice(),
            "candidate",
        ),
    ] {
        assert!(matches!(
            verify_external_lock_subset(original, candidate, 10),
            Err(ExternalLockError::LimitExceeded { input, limit: 10 }) if input == expected
        ));
    }
    let utf8 = verify_external_lock_subset(&[0xff], b"package=[]", 4096).unwrap_err();
    assert!(matches!(
        utf8,
        ExternalLockError::Utf8 {
            input: "original",
            ..
        }
    ));
    assert!(utf8.source().is_some());
    let parse = verify_external_lock_subset(b"package=[]", b"[", 4096).unwrap_err();
    assert!(matches!(
        parse,
        ExternalLockError::Parse {
            input: "candidate",
            ..
        }
    ));
    assert!(parse.source().is_some());
    assert!(matches!(
        verify_external_lock_subset(b"version=4", b"package=[]", 4096),
        Err(ExternalLockError::MissingPackages { input: "original" })
    ));
    for invalid in [
        "package=[1]".to_owned(),
        "[[package]]\nsource=1\n".to_owned(),
        REGISTRY.replace("name='external'", "name=1"),
        REGISTRY.replace("version='1.0.0'", "version=1"),
        REGISTRY.replace("checksum=", "absent="),
        REGISTRY.replace("0123456789abcdef", "0123456789abcdeF"),
        REGISTRY.replace("0123456789abcdef", "short"),
        format!("{GIT}checksum=1\n"),
    ] {
        assert!(
            matches!(
                verify_external_lock_subset(REGISTRY.as_bytes(), invalid.as_bytes(), 4096),
                Err(ExternalLockError::InvalidPackage {
                    input: "candidate",
                    index: 0,
                    ..
                })
            ),
            "{invalid}"
        );
    }
}

#[test]
fn archive_admission_keeps_its_nonempty_rule_on_the_shared_comparison() {
    verify_external_lock_subset(REGISTRY.as_bytes(), b"package=[]", 4096).unwrap();
    let empty = admit_lock(REGISTRY.as_bytes(), b"package=[]").unwrap_err();
    assert_eq!(empty.to_string(), "empty package external graph");
    admit_lock(REGISTRY.as_bytes(), REGISTRY.as_bytes()).unwrap();
    let changed = REGISTRY.replace("1.0.0", "1.0.1");
    let changed = admit_lock(REGISTRY.as_bytes(), changed.as_bytes()).unwrap_err();
    assert!(matches!(
        changed.downcast_ref::<ExternalLockError>(),
        Some(ExternalLockError::SelectionChanged { .. })
    ));
    assert!(admit_lock(REGISTRY.as_bytes(), b"package=[false]").is_err());
}
