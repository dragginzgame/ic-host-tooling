use super::*;
use flate2::{Compression, GzBuilder};
use std::io::Write;
use tar::{Builder, EntryType, Header};

const MEMBER: &str = "binaryen-version_132/bin/wasm-opt";
const PAYLOAD: &[u8] = b"substitute executable bytes\n";
const LIMITS: ArchiveLimits = ArchiveLimits {
    archive_bytes: 1_000_000,
    decompressed_bytes: 1_000_000,
    members: 100,
    member_bytes: 100_000,
};

fn tar_bytes(entries: &[(&str, &[u8], EntryType)]) -> Vec<u8> {
    let mut archive = Builder::new(Vec::new());
    for (path, bytes, kind) in entries {
        let mut header = Header::new_gnu();
        header.set_entry_type(*kind);
        header.set_mode(0o755);
        header.set_size(bytes.len() as u64);
        archive.append_data(&mut header, path, *bytes).unwrap();
    }
    archive.into_inner().unwrap()
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut gzip = GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), Compression::fast());
    gzip.write_all(bytes).unwrap();
    gzip.finish().unwrap()
}

fn extract(bytes: &[u8], limits: ArchiveLimits) -> Result<ExtractedMember, ArchiveError> {
    extract_tar_gz(
        bytes,
        Sha256Digest::compute(bytes),
        MEMBER,
        Sha256Digest::compute(PAYLOAD),
        limits,
    )
}

fn fixture() -> (Vec<u8>, Vec<u8>) {
    let tar = tar_bytes(&[
        ("binaryen-version_132/", b"", EntryType::Directory),
        ("binaryen-version_132/README", b"unused", EntryType::Regular),
        (MEMBER, PAYLOAD, EntryType::Regular),
    ]);
    (gzip(&tar), tar)
}

#[test]
fn exact_member_has_both_identities_without_disclosing_payload_in_debug() {
    let (bytes, tar) = fixture();
    let limits = ArchiveLimits {
        archive_bytes: bytes.len(),
        decompressed_bytes: tar.len(),
        members: 3,
        member_bytes: PAYLOAD.len(),
    };
    let original = bytes.clone();
    let member = extract(&bytes, limits).unwrap();
    assert_eq!(member.bytes, PAYLOAD);
    assert_eq!(member.archive_identity.bytes, bytes.len() as u64);
    assert_eq!(
        member.archive_identity.sha256,
        Sha256Digest::compute(&bytes)
    );
    assert_eq!(member.member_identity.bytes, PAYLOAD.len() as u64);
    assert_eq!(
        member.member_identity.sha256,
        Sha256Digest::compute(PAYLOAD)
    );
    assert!(!format!("{member:?}").contains("substitute"));
    assert_eq!(bytes, original);
}

#[test]
fn archive_admission_precedes_decoding_and_member_mismatch_retains_identity() {
    let wrong = Sha256Digest::compute(b"wrong");
    assert!(matches!(
        extract_tar_gz(b"not gzip", wrong, MEMBER, wrong, LIMITS),
        Err(ArchiveError::Archive(ArtifactError::DigestMismatch { .. }))
    ));
    let (bytes, _) = fixture();
    assert!(
        matches!(extract_tar_gz(&bytes, Sha256Digest::compute(&bytes), MEMBER, wrong, LIMITS),
        Err(ArchiveError::Member(ArtifactError::DigestMismatch { actual, .. }))
            if actual.sha256 == Sha256Digest::compute(PAYLOAD) && actual.bytes == PAYLOAD.len() as u64)
    );
}

#[test]
fn all_four_limits_reject_one_below_the_inclusive_boundary() {
    let (bytes, tar) = fixture();
    assert!(matches!(
        extract(
            &bytes,
            ArchiveLimits {
                archive_bytes: bytes.len() - 1,
                ..LIMITS
            }
        ),
        Err(ArchiveError::Archive(ArtifactError::LimitExceeded { .. }))
    ));
    assert!(matches!(
        extract(
            &bytes,
            ArchiveLimits {
                decompressed_bytes: tar.len() - 1,
                ..LIMITS
            }
        ),
        Err(ArchiveError::Decompression(
            ArtifactError::LimitExceeded { .. }
        ))
    ));
    assert!(matches!(
        extract(
            &bytes,
            ArchiveLimits {
                members: 2,
                ..LIMITS
            }
        ),
        Err(ArchiveError::LimitExceeded {
            resource: ArchiveResource::Members,
            limit: 2
        })
    ));
    assert!(matches!(
        extract(
            &bytes,
            ArchiveLimits {
                member_bytes: PAYLOAD.len() - 1,
                ..LIMITS
            }
        ),
        Err(ArchiveError::LimitExceeded {
            resource: ArchiveResource::MemberBytes,
            ..
        })
    ));
    assert!(matches!(
        extract(
            &bytes,
            ArchiveLimits {
                members: 0,
                ..LIMITS
            }
        ),
        Err(ArchiveError::LimitExceeded {
            resource: ArchiveResource::Members,
            limit: 0
        })
    ));
}

#[test]
fn requested_paths_are_canonical_and_names_are_literal() {
    for invalid in [
        "",
        "/tool",
        "./tool",
        "a/../tool",
        "a//tool",
        "a/",
        "a\\tool",
        "C:tool",
        "tool\0",
    ] {
        assert!(matches!(
            extract_tar_gz(
                b"",
                Sha256Digest::compute(b""),
                invalid,
                Sha256Digest::compute(PAYLOAD),
                LIMITS
            ),
            Err(ArchiveError::InvalidMember)
        ));
    }
    for name in ["bin/工具", "bin/space tool", "bin/$(touch forbidden)"] {
        let bytes = gzip(&tar_bytes(&[(name, PAYLOAD, EntryType::Regular)]));
        let result = extract_tar_gz(
            &bytes,
            Sha256Digest::compute(&bytes),
            name,
            Sha256Digest::compute(PAYLOAD),
            LIMITS,
        )
        .unwrap();
        assert_eq!(result.bytes, PAYLOAD);
    }
    // append_data/set_path normalize dot components. Supply the raw field to
    // exercise exact member spelling rather than the builder's normalization.
    let name = b"./binaryen-version_132/bin/wasm-opt";
    let mut header = Header::new_gnu();
    header.set_size(PAYLOAD.len() as u64);
    header.as_old_mut().name[..name.len()].copy_from_slice(name);
    header.set_cksum();
    let mut tar = Builder::new(Vec::new());
    tar.append(&header, PAYLOAD).unwrap();
    let bytes = gzip(&tar.into_inner().unwrap());
    assert!(matches!(
        extract(&bytes, LIMITS),
        Err(ArchiveError::MissingMember)
    ));
}

#[test]
fn ustar_prefixes_match_exactly_and_backslashes_are_not_normalized() {
    let prefix = "a".repeat(120);
    let name = format!("{prefix}/wasm-opt");
    let mut header = Header::new_ustar();
    header.set_size(PAYLOAD.len() as u64);
    header.set_path(&name).unwrap();
    header.set_cksum();
    let mut tar = Builder::new(Vec::new());
    tar.append(&header, PAYLOAD).unwrap();
    let bytes = gzip(&tar.into_inner().unwrap());
    assert_eq!(
        extract_tar_gz(
            &bytes,
            Sha256Digest::compute(&bytes),
            &name,
            Sha256Digest::compute(PAYLOAD),
            LIMITS
        )
        .unwrap()
        .bytes,
        PAYLOAD
    );

    let mut header = Header::new_ustar();
    header.set_size(PAYLOAD.len() as u64);
    header.as_ustar_mut().unwrap().name[..8].copy_from_slice(b"bin\\tool");
    header.set_cksum();
    let mut tar = Builder::new(Vec::new());
    tar.append(&header, PAYLOAD).unwrap();
    let bytes = gzip(&tar.into_inner().unwrap());
    assert!(matches!(
        extract_tar_gz(
            &bytes,
            Sha256Digest::compute(&bytes),
            "bin/tool",
            Sha256Digest::compute(PAYLOAD),
            LIMITS
        ),
        Err(ArchiveError::MissingMember)
    ));
}

#[test]
fn duplicates_missing_members_and_links_after_selected_bytes_are_rejected() {
    let bytes = gzip(&tar_bytes(&[
        (MEMBER, PAYLOAD, EntryType::Regular),
        (MEMBER, PAYLOAD, EntryType::Regular),
    ]));
    assert!(matches!(
        extract(&bytes, LIMITS),
        Err(ArchiveError::DuplicateMember)
    ));
    let bytes = gzip(&tar_bytes(&[]));
    assert!(matches!(
        extract(&bytes, LIMITS),
        Err(ArchiveError::MissingMember)
    ));
    let bytes = gzip(&tar_bytes(&[(MEMBER, b"", EntryType::Directory)]));
    assert!(matches!(
        extract(&bytes, LIMITS),
        Err(ArchiveError::UnsupportedEntry { .. })
    ));
    for kind in [
        EntryType::Symlink,
        EntryType::Link,
        EntryType::Fifo,
        EntryType::GNUSparse,
        EntryType::GNULongName,
        EntryType::GNULongLink,
        EntryType::XHeader,
        EntryType::XGlobalHeader,
    ] {
        let bytes = gzip(&tar_bytes(&[
            (MEMBER, PAYLOAD, EntryType::Regular),
            ("unselected", b"", kind),
        ]));
        assert!(matches!(
            extract(&bytes, LIMITS),
            Err(ArchiveError::UnsupportedEntry { entry: 2, .. })
        ));
    }
}

#[test]
fn corrupt_truncated_and_concatenated_gzip_streams_are_rejected() {
    let (bytes, _) = fixture();
    let mut corrupt = bytes.clone();
    let crc_offset = corrupt.len() - 8;
    corrupt[crc_offset] ^= 1;
    assert!(matches!(
        extract(&corrupt, LIMITS),
        Err(ArchiveError::Decompression(ArtifactError::Io(_)))
    ));
    for cut in [0, 1, 10, bytes.len() - 1] {
        assert!(matches!(
            extract(&bytes[..cut], LIMITS),
            Err(ArchiveError::Decompression(ArtifactError::Io(_)))
        ));
    }
    for suffix in [b"trailing".to_vec(), gzip(b"another stream")] {
        let mut trailing = bytes.clone();
        trailing.extend(suffix);
        assert!(matches!(
            extract(&trailing, LIMITS),
            Err(ArchiveError::TrailingCompressedData)
        ));
    }
}

#[test]
fn tar_checksum_payload_truncation_and_end_padding_are_validated() {
    let tar = tar_bytes(&[(MEMBER, PAYLOAD, EntryType::Regular)]);
    let mut corrupt = tar.clone();
    corrupt[0] ^= 1;
    assert!(matches!(
        extract(&gzip(&corrupt), LIMITS),
        Err(ArchiveError::Tar(_))
    ));
    assert!(matches!(
        extract(&gzip(&tar[..513]), LIMITS),
        Err(ArchiveError::TruncatedEntry { entry: 1 })
    ));
    // The member ends at block 2. One zero block does not meet the two-block end contract.
    assert!(matches!(
        extract(&gzip(&tar[..1536]), LIMITS),
        Err(ArchiveError::InvalidPadding)
    ));
    assert!(matches!(
        extract(&gzip(&tar[..1024]), LIMITS),
        Err(ArchiveError::InvalidPadding)
    ));
    let mut trailing = tar.clone();
    trailing.extend_from_slice(&[0; 512]);
    assert_eq!(extract(&gzip(&trailing), LIMITS).unwrap().bytes, PAYLOAD);
    trailing.push(0);
    assert!(matches!(
        extract(&gzip(&trailing), LIMITS),
        Err(ArchiveError::InvalidPadding)
    ));
    let mut hidden = tar.clone();
    hidden.extend_from_slice(&tar);
    assert!(matches!(
        extract(&gzip(&hidden), LIMITS),
        Err(ArchiveError::InvalidPadding)
    ));
}

#[test]
fn oversized_declared_payload_is_rejected_without_allocating_or_inflating_it() {
    let mut header = Header::new_gnu();
    header.set_path(MEMBER).unwrap();
    header.set_size(u64::MAX);
    header.set_cksum();
    let mut tar = header.as_bytes().to_vec();
    tar.extend_from_slice(&[0; 1024]);
    assert!(extract(&gzip(&tar), LIMITS).is_err());
}
