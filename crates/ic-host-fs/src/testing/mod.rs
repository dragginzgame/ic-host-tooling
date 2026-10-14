//! Opt-in filesystem fixture allocation without cleanup or application lifecycle.

use std::{
    fs, io,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

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
    let mut components = Path::new(prefix).components();
    if prefix.contains(['\r', '\n'])
        || !matches!(
            (components.next(), components.next()),
            (Some(Component::Normal(name)), None) if name == std::ffi::OsStr::new(prefix)
        )
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid fixture prefix",
        ));
    }
    let parent = parent.canonicalize()?;
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(0o700);
    }
    loop {
        let candidate = parent.join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed),
        ));
        match builder.create(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
