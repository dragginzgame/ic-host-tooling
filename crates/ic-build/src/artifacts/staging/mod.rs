//! Invocation-owned artifact staging without cache publication or reuse.
//!
//! Callers own the parent directory, recipes and publication boundary. This
//! module allocates only its own child and removes it after the last owner is
//! dropped. Explicit retention transfers cleanup to the caller.
//! It never prunes existing artifacts or publishes a product release.

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use ic_host_artifacts::artifact::{ArtifactError, CopyError, copy_reader};
use ic_host_fs::{
    durable::{NamedWriteError, PublicationMode, WriteOptions, write_with},
    owned_directory::OwnedDirectory,
    read::open_regular_file,
};

mod produce;
pub use produce::produce_and_retain_artifact;

const STAGE_ATTEMPTS: usize = 64;
static STAGE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Temporary artifact directory retained until its owner is closed or dropped.
///
/// The caller keeps custody of the parent and allocated child against unrelated
/// writers. Drop performs best-effort cleanup of this invocation-owned child;
/// use [`Self::close`] when cleanup failures must reach the caller. No shared
/// cache entry is created, reused or removed.
#[derive(Debug)]
pub struct ArtifactStagingDirectory {
    directory: OwnedDirectory,
}

impl ArtifactStagingDirectory {
    /// Allocate a distinct staging child beneath an existing caller-owned directory.
    ///
    /// The parent is not created. Existing children are never reused or removed.
    ///
    /// # Errors
    /// Returns the filesystem error for a missing or unwritable parent, or
    /// `AlreadyExists` when every allocation attempt collides.
    pub fn new(parent: &Path) -> io::Result<Self> {
        for _ in 0..STAGE_ATTEMPTS {
            let sequence = STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let root = parent.join(format!(
                ".ic-build-artifact-stage-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&root) {
                Ok(()) => {
                    return Ok(Self {
                        directory: OwnedDirectory::new(root),
                    });
                }
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
                Err(source) => return Err(source),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "failed to allocate artifact staging beneath {}",
                parent.display()
            ),
        ))
    }

    /// Borrow the owned staging directory through artifact production and consumption.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.directory.path()
    }

    /// Remove this owned staging child, reporting cleanup failure without a hidden retry.
    ///
    /// On failure, the caller owns recovery of the remaining staging child.
    /// It does not remove the parent or any other artifact directory.
    ///
    /// # Errors
    /// Returns the filesystem error if removing the owned child fails.
    pub fn close(self) -> io::Result<()> {
        self.directory.cleanup()
    }
}

/// Owned copy of one completed producer output, independent of later source replacement.
///
/// Keep this record alive through every use of [`Self::path`]. The copy is
/// temporary rather than a reusable cache entry; ordinary Drop removes it.
/// [`Self::retain`] explicitly transfers cleanup to the caller.
#[derive(Debug)]
pub struct CapturedArtifact {
    staging: ArtifactStagingDirectory,
    path: PathBuf,
}

/// Finalization failed after capture custody was transferred to the caller.
///
/// The exact bytes remain at [`Self::retained_path`]. If rename succeeded but
/// cleanup failed, that path is the destination; otherwise it is the staging file.
#[derive(Debug)]
pub struct CaptureRetentionError {
    /// Filesystem operation that failed.
    pub source: io::Error,
    /// Actual retained file path after the failed operation.
    pub retained_path: PathBuf,
    /// Owned staging child still requiring caller disposition.
    pub staging_directory: PathBuf,
}

impl fmt::Display for CaptureRetentionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "capture retained at {} (staging {}): {}",
            self.retained_path.display(),
            self.staging_directory.display(),
            self.source
        )
    }
}

impl std::error::Error for CaptureRetentionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl CapturedArtifact {
    /// Capture an output synchronously before another producer can overwrite it.
    ///
    /// The caller keeps the source stable during capture and selects the maximum
    /// readable byte count. Read/publication failures retain their typed causes
    /// inside the returned I/O error. Existing public destinations are untouched.
    /// Bytes stream through bounded scratch rather than a complete in-memory copy.
    ///
    /// # Errors
    /// Returns the source read/byte-limit, staging allocation or durable-write
    /// error. An unsuccessful capture performs best-effort staging cleanup.
    pub fn capture(source: &Path, parent: &Path, maximum_bytes: usize) -> io::Result<Self> {
        let source = open_regular_file(source).map_err(io::Error::from)?;
        let limit = maximum_bytes as u64;
        if source.metadata()?.len() > limit {
            return Err(ArtifactError::LimitExceeded { limit }.into());
        }
        let staging = ArtifactStagingDirectory::new(parent)?;
        let path = staging.path().join("input.artifact");
        write_with(
            &path,
            WriteOptions {
                mode: PublicationMode::Replace,
                permissions: 0o666,
            },
            |file| copy_reader(source, file, limit).map(|_| ()),
        )
        .map_err(|error| match error {
            NamedWriteError::Producer {
                source: CopyError::Input(source),
                cleanup_error: None,
            } => io::Error::from(source),
            other => io::Error::other(other),
        })?;
        Ok(Self { staging, path })
    }

    /// Borrow the exact captured bytes while this record owns their lifetime.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Transfer the captured file and its staging child to caller-owned retention.
    ///
    /// Unlike close or Drop, this keeps the exact captured bytes after this owner
    /// is consumed. The caller owns later cleanup of the returned path's parent.
    /// Keep custody of those paths; this grants no cache or release admission.
    #[must_use]
    pub fn retain(mut self) -> PathBuf {
        self.staging.directory.disarm();
        self.path
    }

    /// Retain the exact capture at a caller-selected destination.
    ///
    /// Uses [`fs::rename`] semantics, including replacement of an existing
    /// destination when supported. The caller owns destination admission and
    /// later custody. Only the empty owned staging child is removed on success.
    ///
    /// # Errors
    /// Rename or cleanup failure reports the actual retained bytes and staging
    /// child. Neither failure deletes evidence or silently retries publication.
    pub fn retain_to(self, destination: &Path) -> Result<PathBuf, CaptureRetentionError> {
        let staging = self.staging.path().to_owned();
        let retained = self.retain();
        fs::rename(&retained, destination).map_err(|source| CaptureRetentionError {
            source,
            retained_path: retained.clone(),
            staging_directory: staging.clone(),
        })?;
        fs::remove_dir(&staging).map_err(|source| CaptureRetentionError {
            source,
            retained_path: destination.to_owned(),
            staging_directory: staging,
        })?;
        Ok(destination.to_owned())
    }

    /// Remove the captured output and its owned staging child, reporting cleanup failure.
    ///
    /// # Errors
    /// Returns the filesystem error if removing the owned child fails.
    pub fn close(self) -> io::Result<()> {
        self.staging.close()
    }
}

#[cfg(test)]
mod tests;
