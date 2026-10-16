//! Inherited descriptor custody and exact existing-sidecar quiescence.
//!
//! Callers own sidecar names, retained identities, command admission, environment
//! names, waiting policy and durable intent. This module never serializes records,
//! removes sidecars, kills descendants or grants retry or remote-effect authority.

use command_fds::CommandFdExt;
use ic_host_fs::durable::{self, RegularFileLockError};
use std::{
    fmt,
    fs::{self, File, Metadata},
    io,
    os::{fd::AsRawFd, unix::fs::MetadataExt},
    path::{Path, PathBuf},
    process::{Child, Command},
    thread,
    time::{Duration, Instant},
};

/// Native regular-file identity; callers own persistence and namespace custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileIdentity {
    /// Filesystem device number.
    pub device: u64,
    /// Inode number on that device.
    pub inode: u64,
}

impl FileIdentity {
    /// Observe a caller-owned regular descriptor without reopening its path.
    ///
    /// # Errors
    /// Returns metadata failures or rejects a non-regular descriptor.
    pub fn from_file(file: &File) -> io::Result<Self> {
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        Ok(Self::from_metadata(&metadata))
    }

    fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }

    fn require(self, metadata: &Metadata, path: &Path) -> Result<(), CustodyError> {
        if !metadata.is_file() || Self::from_metadata(metadata) != self {
            return Err(CustodyError::IdentityChanged {
                path: path.to_path_buf(),
            });
        }
        Ok(())
    }
}

/// One command's inheritable sidecar custody, acquired without waiting.
///
/// Dropping closes the owner descriptor without unlocking inherited copies.
/// Descendants must preserve that descriptor for their entire relevant lifetime.
#[derive(Debug)]
pub struct CommandCustody {
    file: File,
    path: PathBuf,
    identity: FileIdentity,
    dispatched: bool,
}

impl CommandCustody {
    /// Open/create a private regular sidecar in an existing parent and try its lock.
    ///
    /// Existing bytes and permissions are preserved. Creation is not synchronized.
    /// Only the final component is no-follow; callers trust and stabilize parents.
    /// # Errors
    /// Returns entry, lock contention and native I/O failures, retaining the sidecar.
    pub fn acquire(path: &Path) -> Result<Self, CustodyError> {
        let file = durable::open_private_regular_lock_file(path).map_err(CustodyError::Lock)?;
        try_lock(&file, path)?;
        Ok(Self {
            identity: FileIdentity::from_file(&file)?,
            file,
            path: path.to_path_buf(),
            dispatched: false,
        })
    }

    /// Read the observed identity for caller-owned retention before dispatch.
    #[must_use]
    pub const fn identity(&self) -> FileIdentity {
        self.identity
    }

    /// Read the caller-selected sidecar path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Attempt one command with a duplicated descriptor named by its environment.
    ///
    /// Recheck the sidecar identity before consuming the single spawn allowance.
    /// Duplication and spawn failures consume that allowance too. The owner stays
    /// close-on-exec; the command's parent-side copy is closed before returning.
    /// Callers own argv/environment admission, output bounds, deadlines and reaping.
    /// # Errors
    /// Returns replacement, repeated-dispatch, descriptor or spawn failures.
    pub fn spawn(
        &mut self,
        mut command: Command,
        descriptor_environment: &str,
    ) -> Result<Child, CustodyError> {
        if self.dispatched {
            return Err(CustodyError::AlreadyDispatched {
                path: self.path.clone(),
            });
        }
        self.identity
            .require(&fs::symlink_metadata(&self.path)?, &self.path)?;
        self.dispatched = true;
        let descriptor = rustix::io::fcntl_dupfd_cloexec(&self.file, 3).map_err(io::Error::from)?;
        command.env(descriptor_environment, descriptor.as_raw_fd().to_string());
        command.preserved_fds(vec![descriptor]);
        let child = command.spawn();
        drop(command);
        child.map_err(CustodyError::Io)
    }

    /// Close owner custody and probe exact quiescence within caller-selected grace.
    ///
    /// Retry only contention. A zero grace performs one probe; polling must be
    /// nonzero. Filesystem operations have no wall-clock deadline. Failure retains
    /// the original sidecar and establishes no remote outcome or retry authority.
    /// # Errors
    /// Returns invalid timing, active inherited holders, replacement or I/O failure.
    pub fn finish(
        self,
        grace: Duration,
        poll_interval: Duration,
    ) -> Result<QuiescenceGuard, CustodyError> {
        if poll_interval.is_zero() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput).into());
        }
        let deadline = Instant::now()
            .checked_add(grace)
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
        let Self {
            file,
            path,
            identity,
            ..
        } = self;
        drop(file);
        loop {
            match QuiescenceGuard::acquire(&path, identity) {
                Err(CustodyError::InFlight { .. }) if Instant::now() < deadline => {
                    thread::sleep(
                        poll_interval.min(deadline.saturating_duration_since(Instant::now())),
                    );
                }
                result => return result,
            }
        }
    }
}

/// Fresh non-spawning exclusive custody over an exact existing regular sidecar.
///
/// Drop explicitly unlocks, including transient duplicated/inherited copies of
/// this guard. Unlike command custody, this guard cannot intentionally dispatch.
#[derive(Debug)]
pub struct QuiescenceGuard {
    file: File,
    identity: FileIdentity,
}

impl QuiescenceGuard {
    /// Admit the retained identity before locking, then recheck the opened descriptor.
    ///
    /// Missing sidecars are never created. A different regular inode rejects before
    /// lock contention can obscure replacement. Special entries use Host FS admission.
    /// # Errors
    /// Returns active custody, replacement, missing/unsafe entries or native I/O errors.
    pub fn acquire(path: &Path, expected: FileIdentity) -> Result<Self, CustodyError> {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.is_file() {
            expected.require(&metadata, path)?;
        }
        let file = durable::open_existing_regular_lock_file(path).map_err(CustodyError::Lock)?;
        try_lock(&file, path)?;
        expected.require(&file.metadata()?, path)?;
        Ok(Self {
            file,
            identity: expected,
        })
    }

    /// Read the exact admitted identity.
    #[must_use]
    pub const fn identity(&self) -> FileIdentity {
        self.identity
    }
}

impl Drop for QuiescenceGuard {
    fn drop(&mut self) {
        let _ = durable::unlock(&self.file);
    }
}

fn try_lock(file: &File, path: &Path) -> Result<(), CustodyError> {
    durable::try_lock_exclusive(file).map_err(|source| {
        if source.kind() == io::ErrorKind::WouldBlock {
            CustodyError::InFlight {
                path: path.to_path_buf(),
            }
        } else {
            CustodyError::Io(source)
        }
    })
}

/// Typed local custody refusal; no variant authorizes reissue or cleanup.
#[derive(Debug)]
pub enum CustodyError {
    /// An owner or inherited holder still has the exclusive lock.
    InFlight {
        /// Caller-selected sidecar.
        path: PathBuf,
    },
    /// Retained regular-file identity no longer matches.
    IdentityChanged {
        /// Caller-selected sidecar.
        path: PathBuf,
    },
    /// This owner already attempted its one spawn.
    AlreadyDispatched {
        /// Caller-selected sidecar.
        path: PathBuf,
    },
    /// Host FS sidecar admission failed, preserving the original typed cause.
    Lock(RegularFileLockError),
    /// Native filesystem, descriptor, timing or spawn failure.
    Io(io::Error),
}

impl fmt::Display for CustodyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InFlight { .. } => "command custody remains in flight",
            Self::IdentityChanged { .. } => "command custody identity changed",
            Self::AlreadyDispatched { .. } => "command spawn already attempted",
            Self::Lock(_) => "command sidecar admission failed",
            Self::Io(_) => "command custody I/O failed",
        })
    }
}

impl std::error::Error for CustodyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Lock(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for CustodyError {
    fn from(source: io::Error) -> Self {
        Self::Io(source)
    }
}

#[cfg(test)]
mod tests;
