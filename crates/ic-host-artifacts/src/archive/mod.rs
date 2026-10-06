//! Verify a gzip-compressed tar archive and select one admitted regular member.
//!
//! This boundary performs no filesystem writes, downloads, execution or
//! installation. Consumers supply both digests, the exact member and all limits.
//! GNU/PAX extension records, sparse files, links and special files are rejected;
//! only ordinary regular files and directories are accepted. Native distribution
//! compatibility must be qualified before replacing a consumer's installer.

#[cfg(test)]
mod tests;

use std::{collections::TryReserveError, fmt, io};

use crate::artifact::{self, ArtifactError, ArtifactIdentity, GzipError, Sha256Digest};

/// Independent input, inflation, traversal and selected-payload limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchiveLimits {
    /// Maximum compressed input bytes, checked before decompression.
    pub archive_bytes: usize,
    /// Maximum complete inflated tar bytes, including metadata and padding.
    pub decompressed_bytes: usize,
    /// Maximum raw tar records, including directories.
    pub members: u32,
    /// Maximum bytes in the selected regular-file payload.
    pub member_bytes: usize,
}

/// A resource whose caller-supplied bound was exceeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveResource {
    /// Number of raw tar records.
    Members,
    /// Selected regular-file payload size.
    MemberBytes,
}

/// Typed archive admission and decoding failures.
#[derive(Debug)]
pub enum ArchiveError {
    /// The requested member is not a canonical relative POSIX path.
    InvalidMember,
    /// Compressed bytes failed bounded digest verification.
    Archive(ArtifactError),
    /// Gzip decoding, its integrity check or its inflated byte bound failed.
    Decompression(ArtifactError),
    /// Bytes follow the single gzip stream, including another gzip stream.
    TrailingCompressedData,
    /// Tar header framing, checksum or numeric decoding failed.
    Tar(io::Error),
    /// A raw record exceeded its caller-owned resource bound.
    LimitExceeded {
        /// Exhausted resource.
        resource: ArchiveResource,
        /// Caller's bound.
        limit: u64,
    },
    /// Unsupported metadata, link, sparse or special-file record.
    UnsupportedEntry {
        /// One-based raw record number.
        entry: u32,
        /// Raw tar entry-type byte.
        kind: u8,
    },
    /// An entry's declared payload/padding extends beyond the inflated bytes.
    TruncatedEntry {
        /// One-based raw record number.
        entry: u32,
    },
    /// Tar end markers or the remaining zero padding are malformed.
    InvalidPadding,
    /// No regular member has the exact requested name.
    MissingMember,
    /// More than one record has the exact requested name.
    DuplicateMember,
    /// Selected bytes differ from the consumer's admitted executable digest.
    Member(ArtifactError),
    /// Fallible selected-payload allocation failed.
    Allocation(TryReserveError),
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMember => {
                f.write_str("archive member must be a canonical relative POSIX path")
            }
            Self::Archive(_) => f.write_str("compressed archive admission failed"),
            Self::Decompression(_) => f.write_str("bounded gzip decoding failed"),
            Self::TrailingCompressedData => f.write_str("bytes follow the single gzip stream"),
            Self::Tar(_) => f.write_str("tar decoding failed"),
            Self::LimitExceeded { resource, limit } => {
                write!(f, "archive {resource:?} limit {limit} exceeded")
            }
            Self::UnsupportedEntry { entry, kind } => {
                write!(f, "unsupported tar type {kind} at record {entry}")
            }
            Self::TruncatedEntry { entry } => write!(f, "truncated tar record {entry}"),
            Self::InvalidPadding => f.write_str("invalid tar end markers or padding"),
            Self::MissingMember => f.write_str("selected tar member is missing"),
            Self::DuplicateMember => f.write_str("selected tar member is duplicated"),
            Self::Member(_) => f.write_str("selected tar member admission failed"),
            Self::Allocation(_) => f.write_str("selected tar member allocation failed"),
        }
    }
}

impl std::error::Error for ArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Archive(source) | Self::Decompression(source) | Self::Member(source) => {
                Some(source)
            }
            Self::Tar(source) => Some(source),
            Self::Allocation(source) => Some(source),
            _ => None,
        }
    }
}

/// Admitted selected bytes with both observed raw identities.
///
/// Byte ownership does not admit a published file or verify an executable's
/// version; consumers must retain their staged `ic_host_process::tool` admission on Unix.
pub struct ExtractedMember {
    /// Selected regular-file payload, with no archive metadata applied.
    pub bytes: Vec<u8>,
    /// Complete compressed archive identity.
    pub archive_identity: ArtifactIdentity,
    /// Selected raw payload identity.
    pub member_identity: ArtifactIdentity,
}

impl fmt::Debug for ExtractedMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtractedMember")
            .field("archive_identity", &self.archive_identity)
            .field("member_identity", &self.member_identity)
            .finish_non_exhaustive()
    }
}

/// Verify compressed bytes before decoding and return one exact admitted member.
///
/// The input is immutable for this call. The complete inflated tar and selected
/// payload use bounded, fallible owned buffers; peak payload storage includes
/// both buffers plus caller-owned compressed bytes. One gzip stream is accepted,
/// including its CRC/length checks, with no compressed trailing data. The entire
/// tar is scanned to reject duplicate members and unsupported records; end markers
/// require at least two zero blocks followed only by whole zero blocks.
///
/// No member path is used as a filesystem destination. Unselected regular files
/// and directories need not have safe paths because they are never unpacked.
/// Requested paths are canonical and matched exactly, without normalization.
/// Archive permissions, ownership and timestamps are never applied.
///
/// # Errors
/// Returns typed digest, decoding, resource, unsupported-format, member-selection
/// and allocation failures. Caller input/evidence remains untouched on failure.
pub fn extract_tar_gz(
    bytes: &[u8],
    archive_sha256: Sha256Digest,
    member: &str,
    member_sha256: Sha256Digest,
    limits: ArchiveLimits,
) -> Result<ExtractedMember, ArchiveError> {
    if !valid_member(member) {
        return Err(ArchiveError::InvalidMember);
    }
    let archive_identity =
        artifact::verify_reader(bytes, limits.archive_bytes as u64, archive_sha256)
            .map_err(ArchiveError::Archive)?;
    let inflated = artifact::decode_gzip(bytes, limits.archive_bytes, limits.decompressed_bytes)
        .map_err(|source| match source {
            GzipError::InputLimit { limit, .. } => {
                ArchiveError::Archive(ArtifactError::LimitExceeded {
                    limit: limit as u64,
                })
            }
            GzipError::Decode(source) => ArchiveError::Decompression(source),
            GzipError::TrailingData => ArchiveError::TrailingCompressedData,
        })?;
    let payload = select_member(&inflated, member, limits)?;
    let member_identity =
        artifact::verify_reader(payload, limits.member_bytes as u64, member_sha256)
            .map_err(ArchiveError::Member)?;
    let mut selected = Vec::new();
    selected
        .try_reserve_exact(payload.len())
        .map_err(ArchiveError::Allocation)?;
    selected.extend_from_slice(payload);
    Ok(ExtractedMember {
        bytes: selected,
        archive_identity,
        member_identity,
    })
}

fn valid_member(member: &str) -> bool {
    !member.is_empty()
        && !member.bytes().any(|byte| matches!(byte, 0 | b'\\' | b':'))
        && member
            .split('/')
            .all(|component| !matches!(component, "" | "." | ".."))
}

fn select_member<'a>(
    bytes: &'a [u8],
    member: &str,
    limits: ArchiveLimits,
) -> Result<&'a [u8], ArchiveError> {
    let mut archive = tar::Archive::new(bytes);
    let entries = archive.entries().map_err(ArchiveError::Tar)?.raw(true);
    let mut next = 0_usize;
    let mut selected = None;
    for (index, entry) in entries.enumerate() {
        let entry = entry.map_err(ArchiveError::Tar)?;
        let count = u32::try_from(index)
            .ok()
            .and_then(|count| count.checked_add(1))
            .filter(|count| *count <= limits.members)
            .ok_or_else(|| ArchiveError::LimitExceeded {
                resource: ArchiveResource::Members,
                limit: u64::from(limits.members),
            })?;
        let kind = entry.header().entry_type();
        if !kind.is_file() && !kind.is_dir() {
            return Err(ArchiveError::UnsupportedEntry {
                entry: count,
                kind: kind.as_byte(),
            });
        }
        let start = usize::try_from(entry.raw_file_position())
            .map_err(|_| ArchiveError::TruncatedEntry { entry: count })?;
        let size = usize::try_from(entry.size())
            .map_err(|_| ArchiveError::TruncatedEntry { entry: count })?;
        let end = start
            .checked_add(size)
            .ok_or(ArchiveError::TruncatedEntry { entry: count })?;
        next = end
            .checked_add(511)
            .map(|value| value & !511)
            .filter(|value| *value <= bytes.len())
            .ok_or(ArchiveError::TruncatedEntry { entry: count })?;
        if header_matches(entry.header(), member.as_bytes()) {
            if selected.is_some() {
                return Err(ArchiveError::DuplicateMember);
            }
            if !kind.is_file() {
                return Err(ArchiveError::UnsupportedEntry {
                    entry: count,
                    kind: kind.as_byte(),
                });
            }
            if size > limits.member_bytes {
                return Err(ArchiveError::LimitExceeded {
                    resource: ArchiveResource::MemberBytes,
                    limit: limits.member_bytes as u64,
                });
            }
            selected = Some(
                bytes
                    .get(start..end)
                    .ok_or(ArchiveError::TruncatedEntry { entry: count })?,
            );
        }
    }
    let padding = &bytes[next..];
    if padding.len() < 1024
        || !padding.len().is_multiple_of(512)
        || padding.iter().any(|byte| *byte != 0)
    {
        return Err(ArchiveError::InvalidPadding);
    }
    selected.ok_or(ArchiveError::MissingMember)
}

fn header_matches(header: &tar::Header, member: &[u8]) -> bool {
    // Header::path_bytes normalizes USTAR backslashes. Compare exposed raw
    // fields instead so member selection never silently changes spelling.
    let name = header
        .as_old()
        .name
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or(&[]);
    if let Some(ustar) = header.as_ustar() {
        let prefix = ustar.prefix.split(|byte| *byte == 0).next().unwrap_or(&[]);
        if !prefix.is_empty() {
            return member
                .strip_prefix(prefix)
                .and_then(|rest| rest.strip_prefix(b"/"))
                == Some(name);
        }
    }
    member == name
}
