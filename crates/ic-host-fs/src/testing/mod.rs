//! Opt-in filesystem fixture allocation without cleanup or application lifecycle.

use std::{
    io,
    path::{Path, PathBuf},
};

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
    let parent = parent.canonicalize()?;
    crate::directory::create_unique_directory(&parent, prefix, 0o700, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parallel_allocation_preserves_existing_entries_and_caller_retention() {
        let root = crate::test_support::Fixture::new();
        let marker = root.root.join("retained");
        fs::write(&marker, b"original evidence").unwrap();
        #[expect(
            clippy::needless_collect,
            reason = "spawn every worker before joining to exercise concurrent allocation"
        )]
        let paths = std::thread::scope(|scope| {
            let workers = (0..32)
                .map(|_| scope.spawn(|| create_fixture_directory(&root.root, "parallel").unwrap()))
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
            assert_eq!(path.parent(), Some(root.root.as_path()));
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt as _;
                assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o077, 0);
            }
        }
    }

    #[test]
    fn invalid_prefix_and_missing_parent_preserve_the_namespace() {
        let root = crate::test_support::Fixture::new();
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
                create_fixture_directory(&root.root, prefix)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidInput
            );
        }
        assert_eq!(
            create_fixture_directory(&root.root.join("absent"), "valid")
                .unwrap_err()
                .kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(fs::read_dir(&root.root).unwrap().count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn parent_alias_returns_a_physical_path() {
        let root = crate::test_support::Fixture::new();
        let parent = root.root.join("physical");
        fs::create_dir(&parent).unwrap();
        let alias = root.root.join("alias");
        std::os::unix::fs::symlink(&parent, &alias).unwrap();
        let path = create_fixture_directory(&alias, "fixture").unwrap();
        assert_eq!(path.parent(), Some(parent.as_path()));
        assert_eq!(path, path.canonicalize().unwrap());
    }
}
