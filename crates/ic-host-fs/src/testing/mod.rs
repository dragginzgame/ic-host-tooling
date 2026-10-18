//! Opt-in filesystem fixtures with explicit allocation and panic-retaining cleanup.

use std::{
    io,
    path::{Path, PathBuf},
};

/// A fresh test directory removed on normal Drop and retained during panic.
///
/// Uses [`create_fixture_directory`] for allocation and
/// [`crate::owned_directory::OwnedDirectory`] for cleanup custody. Callers select
/// the parent/prefix and must exclude namespace replacement. Cleanup is best
/// effort; a failed cleanup leaves evidence at the original path. Panicking tests
/// retain the directory and print its path, without changing production cleanup.
#[derive(Debug)]
pub struct FixtureDirectory {
    directory: crate::owned_directory::OwnedDirectory,
}

impl FixtureDirectory {
    /// Allocate one fresh directory and take cleanup responsibility for it.
    ///
    /// # Errors
    /// Returns [`create_fixture_directory`]'s validation and allocation failures.
    pub fn new(parent: &Path, prefix: &str) -> io::Result<Self> {
        Ok(Self {
            directory: crate::owned_directory::OwnedDirectory::new(create_fixture_directory(
                parent, prefix,
            )?),
        })
    }

    /// Borrow the physical path allocated for this fixture.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.directory.path()
    }
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.directory.disarm();
            eprintln!(
                "Failed test fixture retained: {}",
                self.directory.path().display()
            );
        }
    }
}

/// Create one fresh fixture directory beneath the caller-selected existing parent.
///
/// The prefix must be one normal filename component without CR/LF. Parent aliases
/// are resolved before creation. Existing entries are skipped, never reclaimed.
/// Unix directories use mode 0700 subject to umask; other hosts use native defaults.
/// The path is anchored to the physical parent and has no automatic cleanup.
/// The caller owns retention,
/// removal and custody against concurrent renames or replacement of the namespace.
/// This is a fixture primitive, not hostile-directory confinement.
///
/// # Errors
/// Returns invalid-prefix, parent resolution and directory creation errors.
pub fn create_fixture_directory(parent: &Path, prefix: &str) -> io::Result<PathBuf> {
    // Preserve invalid-prefix admission before any parent lookup.
    crate::directory::validate_options(prefix, 0o700)?;
    let parent = parent.canonicalize()?;
    crate::directory::create_unique_directory(&parent, prefix, 0o700, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn fixture_guard_removes_successful_test_output() {
        let parent = crate::test_support::fixture();
        let fixture = FixtureDirectory::new(parent.path(), "successful").unwrap();
        let path = fixture.path().to_path_buf();
        fs::write(path.join("output"), b"completed output").unwrap();
        drop(fixture);
        assert!(!path.exists());
        assert!(parent.path().is_dir());
    }

    #[test]
    fn fixture_guard_retains_failed_test_evidence_during_unwind() {
        let parent = crate::test_support::fixture();
        let fixture = FixtureDirectory::new(parent.path(), "failed").unwrap();
        let path = fixture.path().to_path_buf();
        fs::write(path.join("evidence"), b"failure evidence").unwrap();
        let panic = std::panic::catch_unwind(move || {
            let _fixture = fixture;
            panic!("intentional fixture failure");
        });
        assert!(panic.is_err());
        assert_eq!(
            fs::read(path.join("evidence")).unwrap(),
            b"failure evidence"
        );
    }

    #[test]
    fn parallel_allocation_preserves_existing_entries_and_caller_retention() {
        let root = crate::test_support::fixture();
        let marker = root.path().join("retained");
        fs::write(&marker, b"original evidence").unwrap();
        #[expect(
            clippy::needless_collect,
            reason = "spawn every worker before joining to exercise concurrent allocation"
        )]
        let paths = std::thread::scope(|scope| {
            let workers = (0..32)
                .map(|_| scope.spawn(|| create_fixture_directory(root.path(), "parallel").unwrap()))
                .collect::<Vec<_>>();
            workers
                .into_iter()
                .map(|worker| worker.join().unwrap())
                .collect::<std::collections::BTreeSet<_>>()
        });
        assert_eq!(paths.len(), 32);
        assert_eq!(fs::read(&marker).unwrap(), b"original evidence");
        for path in paths {
            assert!(path.is_dir());
            assert_eq!(path.parent(), Some(root.path()));
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt as _;
                assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o077, 0);
            }
        }
    }

    #[test]
    fn invalid_prefix_and_missing_parent_preserve_the_namespace() {
        let root = crate::test_support::fixture();
        for prefix in [
            "",
            ".",
            "..",
            "../escape",
            "/absolute",
            "nested/child",
            "trailing/",
            "nested/.",
            "line\nfeed",
            "carriage\rreturn",
        ] {
            assert_eq!(
                create_fixture_directory(root.path(), prefix)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidInput
            );
        }
        assert_eq!(
            create_fixture_directory(&root.path().join("absent"), "valid")
                .unwrap_err()
                .kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
        assert_eq!(
            create_fixture_directory(&root.path().join("absent"), "../invalid")
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[cfg(unix)]
    #[test]
    fn parent_alias_returns_a_physical_path() {
        let root = crate::test_support::fixture();
        let parent = root.path().join("physical");
        fs::create_dir(&parent).unwrap();
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink(&parent, &alias).unwrap();
        let path = create_fixture_directory(&alias, "fixture").unwrap();
        assert_eq!(path.parent(), Some(parent.as_path()));
        assert_eq!(path, path.canonicalize().unwrap());
    }
}
