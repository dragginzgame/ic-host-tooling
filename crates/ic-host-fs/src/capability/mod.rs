//! Held Unix directory capabilities, adapted from Query's confined file backend.
//!
//! Paths are selected without following symlink components. Handles retain the
//! selected directories after renames; they do not prove current pathname
//! containment, immutable contents or exclusive namespace custody. Callers own
//! those obligations, permission settings, byte budgets, schemas and recovery.
//! The cap-* backend is private. Regular-file opening and publication compose
//! with the existing Host descriptor engines rather than introducing new ones.

mod entries;
mod error;

pub use entries::{DirectoryEntries, DirectoryEntry, EntryKind};
pub use error::{CapabilityError, Operation, Refusal, RemovalError};

use crate::read::{OpenEntryError, OpenedEntry, open_entry_beneath};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, DirBuilder, DirBuilderExt, OpenOptions, OpenOptionsExt, PermissionsExt};
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io,
    os::fd::{AsFd, BorrowedFd},
    path::{Component, Path, PathBuf},
};

/// Explicit directory creation and admission settings; there are no defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DirectoryOptions {
    /// `Some(mode)` permits missing directory creation, subject to umask.
    /// `None` observes absence without creating anything.
    pub create_permissions: Option<u32>,
    /// Existing and created selected directories must omit these low nine bits.
    /// Zero permits all ordinary permission bits. Existing modes are never repaired.
    pub forbidden_permissions: u32,
}

/// A held directory capability; dropping it never removes filesystem entries.
#[derive(Debug)]
pub struct Directory {
    inner: Dir,
    display: PathBuf,
}

impl Directory {
    /// Open a root without following any component, optionally creating parents.
    ///
    /// Relative inputs use the current directory through `std::path::absolute`.
    /// Parent traversal is refused before normalization. Existing ancestors above
    /// the selected root are not subjected to its permission mask; newly created
    /// ancestors and the final root are. Each creation re-admits the child after
    /// an `AlreadyExists` race and synchronizes its held parent after admission.
    ///
    /// # Errors
    /// Returns structural/permission refusals and original IO failures. Errors
    /// after creation can leave visible directories; callers own reconciliation.
    pub fn open_root(
        path: &Path,
        options: DirectoryOptions,
    ) -> Result<Option<Self>, CapabilityError> {
        validate_options(path, options)?;
        if path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::Prefix(_)))
        {
            return Err(refused(path, Refusal::InvalidPath));
        }
        let absolute =
            std::path::absolute(path).map_err(|error| failure(Operation::Open, path, error))?;
        let mut directory = Self {
            inner: Dir::open_ambient_dir(Path::new("/"), cap_std::ambient_authority())
                .map_err(|error| failure(Operation::Open, path, error))?,
            display: PathBuf::from("/"),
        };
        for component in absolute.components() {
            if let Component::Normal(name) = component {
                let Some(child) = directory.component(name, options)? else {
                    return Ok(None);
                };
                directory = child;
            }
        }
        directory.display = path.to_path_buf();
        directory.admit_permissions(options.forbidden_permissions)?;
        Ok(Some(directory))
    }

    /// Clone the held capability, without re-resolving its original pathname.
    ///
    /// # Errors
    /// Returns the original descriptor duplication failure.
    pub fn try_clone(&self) -> Result<Self, CapabilityError> {
        Ok(Self {
            inner: self
                .inner
                .try_clone()
                .map_err(|error| failure(Operation::Open, &self.display, error))?,
            display: self.display.clone(),
        })
    }

    /// Select a normal root-relative file name and retain its admitted parent.
    ///
    /// Every traversed directory is checked against the caller's mask. Missing
    /// parents are created only when explicitly selected. The final file itself
    /// is neither opened nor created. Cache path construction stays with callers.
    ///
    /// # Errors
    /// Refuses empty/absolute/parent paths, links, types and permission violations;
    /// native failures retain the selected operation and original cause.
    pub fn resolve_parent(
        &self,
        relative: &Path,
        options: DirectoryOptions,
    ) -> Result<Option<FileEntry>, CapabilityError> {
        validate_options(relative, options)?;
        let mut parts = relative.components().peekable();
        if parts.peek().is_none()
            || parts
                .clone()
                .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err(refused(relative, Refusal::InvalidPath));
        }
        let mut parent = self.try_clone()?;
        while let Some(Component::Normal(name)) = parts.next() {
            if parts.peek().is_none() {
                return Ok(Some(FileEntry {
                    parent,
                    name: name.to_os_string(),
                }));
            }
            let Some(child) = parent.component(name, options)? else {
                return Ok(None);
            };
            child.admit_permissions(options.forbidden_permissions)?;
            parent = child;
        }
        unreachable!("validated nonempty normal path")
    }

    /// Open/admit one directory component with explicit creation permissions.
    ///
    /// # Errors
    /// Returns the same structural, permission and native failures as traversal.
    pub fn open_directory(
        &self,
        name: &OsStr,
        options: DirectoryOptions,
    ) -> Result<Option<Self>, CapabilityError> {
        validate_name(name)?;
        validate_options(Path::new(name), options)?;
        let directory = self.component(name, options)?;
        if let Some(directory) = &directory {
            directory.admit_permissions(options.forbidden_permissions)?;
        }
        Ok(directory)
    }

    /// Observe low nine permission bits through the retained directory handle.
    ///
    /// # Errors
    /// Returns the original metadata failure.
    pub fn permissions(&self) -> Result<u32, CapabilityError> {
        self.inner
            .dir_metadata()
            .map(|metadata| metadata.permissions().mode() & 0o777)
            .map_err(|error| failure(Operation::Metadata, &self.display, error))
    }

    /// Open a readable, syncable descriptor relative to this same capability.
    ///
    /// Some cap-* handles use `O_PATH`. Use this result with existing Host durable
    /// writers rather than re-opening a display path or fsyncing an `O_PATH` handle.
    ///
    /// # Errors
    /// Returns the original opening failure.
    pub fn open_syncable(&self) -> Result<File, CapabilityError> {
        self.inner
            .open(Path::new("."))
            .map(cap_std::fs::File::into_std)
            .map_err(|error| failure(Operation::Open, &self.display, error))
    }

    /// Synchronize this held directory, without re-selecting its pathname.
    ///
    /// # Errors
    /// Returns the original opening or sync failure under the sync operation.
    pub fn sync(&self) -> Result<(), CapabilityError> {
        self.inner
            .open(Path::new("."))
            .and_then(|directory| directory.sync_all())
            .map_err(|error| failure(Operation::SyncDirectory, &self.display, error))
    }

    /// Lazily observe at most `maximum` entries, including nonmatching entries.
    ///
    /// The iterator reports another observed entry as `EntryLimit`, without
    /// interpreting candidate selection, sorting, recursion or truncation policy.
    ///
    /// # Errors
    /// Returns the original directory-open failure.
    pub fn entries(&self, maximum: usize) -> Result<DirectoryEntries, CapabilityError> {
        let entries = self
            .inner
            .entries()
            .map_err(|error| failure(Operation::ReadDirectory, &self.display, error))?;
        Ok(DirectoryEntries::new(
            entries,
            self.display.clone(),
            maximum,
        ))
    }

    fn component(
        &self,
        name: &OsStr,
        options: DirectoryOptions,
    ) -> Result<Option<Self>, CapabilityError> {
        if let Some(directory) = self.open_component(name)? {
            return Ok(Some(directory));
        }
        let Some(mode) = options.create_permissions else {
            return Ok(None);
        };
        self.create_component(name, mode, options.forbidden_permissions)
            .map(Some)
    }

    fn create_component(
        &self,
        name: &OsStr,
        mode: u32,
        forbidden: u32,
    ) -> Result<Self, CapabilityError> {
        let path = self.display.join(name);
        let mut builder = DirBuilder::new();
        builder.mode(mode);
        match self.inner.create_dir_with(name, &builder) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(failure(Operation::CreateDirectory, &path, error)),
        }
        let directory = self.open_component(name)?.ok_or_else(|| {
            failure(
                Operation::Open,
                &path,
                io::Error::new(io::ErrorKind::NotFound, "created directory disappeared"),
            )
        })?;
        directory.admit_permissions(forbidden)?;
        self.sync()?;
        Ok(directory)
    }

    fn open_component(&self, name: &OsStr) -> Result<Option<Self>, CapabilityError> {
        let path = self.display.join(name);
        match self.inner.open_dir_nofollow(name) {
            Ok(inner) => Ok(Some(Self {
                inner,
                display: path,
            })),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => match self.inner.symlink_metadata(name) {
                Ok(metadata) if metadata.is_symlink() => Err(refused(&path, Refusal::SymbolicLink)),
                Ok(metadata) if !metadata.is_dir() => Err(refused(&path, Refusal::NotDirectory)),
                _ => Err(failure(Operation::Open, &path, error)),
            },
        }
    }

    fn admit_permissions(&self, forbidden: u32) -> Result<(), CapabilityError> {
        let actual = self.permissions()?;
        if actual & forbidden != 0 {
            return Err(CapabilityError::Permissions {
                path: self.display.clone(),
                actual,
                forbidden,
            });
        }
        Ok(())
    }
}

impl AsFd for Directory {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

/// One normal file name and its retained parent; selection creates nothing.
#[derive(Debug)]
pub struct FileEntry {
    parent: Directory,
    name: OsString,
}

impl FileEntry {
    /// Borrow the selected parent capability.
    #[must_use]
    pub const fn parent(&self) -> &Directory {
        &self.parent
    }

    /// Borrow the final native name, without conversion to Unicode.
    #[must_use]
    pub fn file_name(&self) -> &OsStr {
        &self.name
    }

    /// Open a regular file without blocking on a raced-in FIFO.
    ///
    /// Missing files return None. Permission admission and bounded reading remain
    /// explicit caller steps over the returned standard file descriptor.
    ///
    /// # Errors
    /// Rejects observed links/special files and retains native opening failures.
    pub fn open_regular_file(&self) -> Result<Option<File>, CapabilityError> {
        let path = self.parent.display.join(&self.name);
        match self.parent.inner.symlink_metadata(&self.name) {
            Ok(metadata) if metadata.is_symlink() => {
                return Err(refused(&path, Refusal::SymbolicLink));
            }
            Ok(metadata) if !metadata.is_file() => {
                return Err(refused(&path, Refusal::NotRegularFile));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(failure(Operation::Metadata, &path, error)),
        }
        match open_entry_beneath(self.parent.as_fd(), Path::new(&self.name)) {
            Ok(OpenedEntry::RegularFile(file)) => Ok(Some(file)),
            Ok(OpenedEntry::Directory(_)) | Err(OpenEntryError::UnsupportedEntry { .. }) => {
                Err(refused(&path, Refusal::NotRegularFile))
            }
            Err(OpenEntryError::Io(error)) => Err(failure(Operation::Open, &path, error)),
        }
    }

    /// Exclusively create one read/write regular file with caller-selected mode.
    ///
    /// Existing files and links return the original `AlreadyExists` failure.
    /// No file or directory synchronization is implicit in this creation step.
    ///
    /// # Errors
    /// Returns invalid permissions or the original exclusive-create failure.
    pub fn create_new_file(&self, permissions: u32) -> Result<File, CapabilityError> {
        let path = self.parent.display.join(&self.name);
        validate_options(
            &path,
            DirectoryOptions {
                create_permissions: Some(permissions),
                forbidden_permissions: 0,
            },
        )?;
        let mut options = OpenOptions::new();
        options
            .read(true)
            .write(true)
            .create_new(true)
            .follow(FollowSymlinks::No)
            .mode(permissions);
        self.parent
            .inner
            .open_with(&self.name, &options)
            .map(cap_std::fs::File::into_std)
            .map_err(|error| failure(Operation::CreateFile, &path, error))
    }

    /// Unlink this name and synchronize the same held parent.
    ///
    /// Callers must first admit the permitted leaf type/mode and retain namespace
    /// custody. A replacement race is not an identity-conditional unlink. Links
    /// are never followed, but an intervening link can itself be unlinked. Missing
    /// entries return false; no recursive removal or recovery is performed.
    ///
    /// # Errors
    /// `removed` remains true if unlink completed before parent synchronization failed.
    pub fn remove_file_and_sync(&self) -> Result<bool, RemovalError> {
        self.remove_file_and_sync_with(Directory::sync)
    }

    fn remove_file_and_sync_with(
        &self,
        sync: impl FnOnce(&Directory) -> Result<(), CapabilityError>,
    ) -> Result<bool, RemovalError> {
        match self.parent.inner.remove_file(&self.name) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(RemovalError {
                    removed: false,
                    source: failure(
                        Operation::RemoveFile,
                        &self.parent.display.join(&self.name),
                        error,
                    ),
                });
            }
        }
        sync(&self.parent).map_err(|source| RemovalError {
            removed: true,
            source,
        })?;
        Ok(true)
    }
}

fn validate_name(name: &OsStr) -> Result<(), CapabilityError> {
    let mut parts = Path::new(name).components();
    if !matches!((parts.next(), parts.next()), (Some(Component::Normal(part)), None) if part == name)
    {
        return Err(refused(Path::new(name), Refusal::InvalidPath));
    }
    Ok(())
}

fn validate_options(path: &Path, options: DirectoryOptions) -> Result<(), CapabilityError> {
    if options.forbidden_permissions & !0o777 != 0
        || options
            .create_permissions
            .is_some_and(|mode| mode & !0o777 != 0)
    {
        return Err(failure(
            Operation::Open,
            path,
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "permissions must fit the low nine bits",
            ),
        ));
    }
    Ok(())
}

fn failure(operation: Operation, path: &Path, source: io::Error) -> CapabilityError {
    CapabilityError::Io {
        operation,
        path: path.to_path_buf(),
        source,
    }
}

fn refused(path: &Path, reason: Refusal) -> CapabilityError {
    CapabilityError::Refused {
        path: path.to_path_buf(),
        reason,
    }
}

#[cfg(test)]
mod tests;
