//! Qualification budgets and byte/mode projection; Host owns Unix tree admission.

use super::archive::{Cause, PackageArchiveSpec, PackageQualificationLimits};
use ic_host_artifacts::artifact::Sha256Digest;
use serde::{Serialize, Serializer};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use ic_host_fs::read::tree::{TreeVisitor, walk_tree};

use rustix::{
    fd::OwnedFd,
    fs::{FileType, Mode, OFlags, open},
};

/// Typed source facts; JSON is built only while writing evidence.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct SourceFile {
    mode: u32,
    sha256: Sha256Digest,
}

impl Serialize for SourceFile {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Keep the former encoding, including serde_json's feature-selected order.
        serde_json::json!({"sha256": self.sha256.to_string(), "mode": self.mode})
            .serialize(serializer)
    }
}

pub(super) fn sources(
    root: &Path,
    packages: &[PackageArchiveSpec<'_>],
    limits: PackageQualificationLimits,
) -> Result<BTreeMap<PathBuf, SourceFile>, Cause> {
    let mut files = BTreeMap::new();
    let mut remaining = limits.source_entries;
    for spec in packages {
        let path = root.join(format!("{}-{}", spec.name, spec.version));

        let fd = open(
            &path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )?;
        let mut visitor = SourceInventory {
            prefix: path.strip_prefix(root)?,
            limits,
            remaining: &mut remaining,
            files: &mut files,
        };
        visitor.enter_directory(Path::new(""), &path)?;
        walk_tree(&fd, Path::new(""), &path, &mut visitor)?;
    }
    Ok(files)
}

struct SourceInventory<'a> {
    prefix: &'a Path,
    limits: PackageQualificationLimits,
    remaining: &'a mut usize,
    files: &'a mut BTreeMap<PathBuf, SourceFile>,
}

impl SourceInventory<'_> {
    fn admit(&mut self, relative: &Path) -> Result<(), Cause> {
        if relative.components().count() > self.limits.source_depth || *self.remaining == 0 {
            return Err("extracted source traversal budget exceeded".into());
        }
        *self.remaining -= 1;
        Ok(())
    }
}

impl TreeVisitor for SourceInventory<'_> {
    type Error = Cause;

    fn file(&mut self, fd: &OwnedFd, relative: &Path, _display: &Path) -> Result<(), Cause> {
        use std::os::unix::fs::MetadataExt as _;
        self.admit(relative)?;
        let file = std::fs::File::from(fd.try_clone()?);
        let mode = file.metadata()?.mode() & 0o777;
        let sha256 =
            ic_host_artifacts::artifact::hash_reader(file, self.limits.file_bytes as u64)?.sha256;
        self.files
            .insert(self.prefix.join(relative), SourceFile { mode, sha256 });
        Ok(())
    }

    fn enter_directory(&mut self, relative: &Path, _display: &Path) -> Result<(), Cause> {
        self.admit(relative)
    }

    fn leave_directory(
        &mut self,
        _fd: &OwnedFd,
        _relative: &Path,
        _display: &Path,
    ) -> Result<(), Cause> {
        Ok(())
    }

    fn unsupported(&self, display: &Path, _kind: FileType) -> Cause {
        format!("unsupported extracted source entry: {}", display.display()).into()
    }

    fn non_utf8(&self, display: &Path) -> Cause {
        format!("non-UTF-8 extracted source entry: {}", display.display()).into()
    }
}
