use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use super::digest::read_stamp_with_limit;
use ic_host_fs::{
    durable::SharedFileLock, owned_directory::remove_path_if_present, read::directory_logical_size,
};

/// Resolve existing components through symlinks and normalize a missing suffix.
/// Parent traversal can return from a missing suffix to existing components.
pub(super) fn canonicalize_allow_missing(path: &Path) -> io::Result<PathBuf> {
    let base = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()?
    };
    ic_host_fs::path::canonicalize_allow_missing(path, &base, usize::MAX)
}

const CACHE_DIRECTORY_TAG_SUFFIX: &str = "\n\
# This file is a cache directory tag created by ic-build.\n\
# For information about cache directory tags see https://bford.info/cachedir/\n";
pub(super) const LAST_USED_FILE: &str = ".ic-testkit-last-used";
const LAST_MAINTENANCE_FILE: &str = ".ic-testkit-last-maintenance";
// Nanoseconds are written as decimal u128 values, which need at most 39 bytes.
const MAX_TIMESTAMP_BYTES: usize = 39;
pub(super) const RETENTION_LOCK_FILE: &str = ".ic-testkit-retention-v1";

/// Acquired under the producer/namespace lock before handing an entry to a
/// consumer. Clones share ownership; the OS releases locks on process exit.
#[derive(Clone, Debug)]
pub(super) struct RetainedCacheEntry {
    path: PathBuf,
    _lock: Arc<SharedFileLock>,
}

impl PartialEq for RetainedCacheEntry {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for RetainedCacheEntry {}

impl RetainedCacheEntry {
    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn acquire(path: &Path) -> Result<Self, CacheFsError> {
        let file = open_cache_lock_file(&path.join(RETENTION_LOCK_FILE))?;
        let lock = SharedFileLock::acquire(file).map_err(|source| CacheFsError {
            operation: "retain cache entry",
            path: path.to_owned(),
            source,
        })?;
        Ok(Self {
            path: path.to_owned(),
            _lock: Arc::new(lock),
        })
    }
}

/// The caller must hold the producer/namespace lock throughout this operation.
pub(super) fn remove_unretained_entry(path: &Path) -> Result<(), CacheFsError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(CacheFsError {
                operation: "inspect cache entry",
                path: path.to_owned(),
                source,
            });
        }
    };
    if !metadata.is_dir() {
        return remove_path_if_present(path).map_err(|source| CacheFsError {
            operation: "remove invalid cache entry",
            path: path.to_owned(),
            source,
        });
    }
    let _lock =
        try_lock_cache_file(&path.join(RETENTION_LOCK_FILE))?.ok_or_else(|| CacheFsError {
            operation: "replace retained cache entry",
            path: path.to_owned(),
            source: io::Error::new(
                io::ErrorKind::WouldBlock,
                "cache entry is retained by a consumer",
            ),
        })?;
    remove_path_if_present(path).map_err(|source| CacheFsError {
        operation: "remove cache entry",
        path: path.to_owned(),
        source,
    })
}

/// Caller-selected age and logical-size thresholds for cache maintenance.
///
/// Thresholds do not select a cleanup algorithm or schedule maintenance.
/// Exact-entry pruning removes expired and then least-recently-used entries,
/// skipping live retained records. Shared-target maintenance clears the entire
/// mutable Cargo state when either threshold is exceeded. It preserves only
/// the target root and IC Build's coordination metadata; do not colocate unrelated
/// data there. Without thresholds, no committed entry or shared compiler state
/// is removed. Explicit transaction maintenance may still clear abandoned staging.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CacheRetentionLimits {
    max_age: Option<Duration>,
    max_size_bytes: Option<u64>,
}

/// Summary of one lock-coordinated artifact-cache pruning pass.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArtifactCachePruneReport {
    entries_scanned: usize,
    entries_removed: usize,
    bytes_before: u64,
    bytes_removed: u64,
    uncommitted_directories_removed: usize,
    uncommitted_bytes_removed: u64,
}

/// Nonfatal retention attempted as part of a successful cache acquisition.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactCacheMaintenance {
    /// Configured retention completed under the cache lock.
    Pruned(ArtifactCachePruneReport),
    /// Configured retention failed after the requested artifacts were ready.
    PruneFailed {
        /// Cache error rendered without invalidating the successful acquisition.
        message: String,
    },
}

impl CacheRetentionLimits {
    /// Create explicit limits without an age or size threshold.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_age: None,
            max_size_bytes: None,
        }
    }

    /// Set the maximum time since recorded cache use.
    #[must_use]
    pub const fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Set the maximum logical cache size.
    #[must_use]
    pub const fn with_max_size_bytes(mut self, bytes: u64) -> Self {
        self.max_size_bytes = Some(bytes);
        self
    }

    /// Configured maximum entry age, if any.
    #[must_use]
    pub const fn max_age(self) -> Option<Duration> {
        self.max_age
    }

    /// Configured maximum logical cache size in bytes, if any.
    #[must_use]
    pub const fn max_size_bytes(self) -> Option<u64> {
        self.max_size_bytes
    }

    pub(super) fn maintenance_identity(self) -> String {
        format!(
            "age={:?};size={:?}",
            self.max_age.map(|duration| duration.as_nanos()),
            self.max_size_bytes
        )
    }
}

impl ArtifactCachePruneReport {
    /// Number of content-addressed directories considered for pruning.
    #[must_use]
    pub const fn entries_scanned(self) -> usize {
        self.entries_scanned
    }

    /// Number of content-addressed directories removed.
    #[must_use]
    pub const fn entries_removed(self) -> usize {
        self.entries_removed
    }

    /// Number of content-addressed directories retained.
    #[must_use]
    pub const fn entries_retained(self) -> usize {
        self.entries_scanned.saturating_sub(self.entries_removed)
    }

    /// Logical bytes occupied by scanned entries before pruning.
    #[must_use]
    pub const fn bytes_before(self) -> u64 {
        self.bytes_before
    }

    /// Logical bytes removed by pruning.
    #[must_use]
    pub const fn bytes_removed(self) -> u64 {
        self.bytes_removed
    }

    /// Logical bytes occupied by retained entries after pruning.
    #[must_use]
    pub const fn bytes_retained(self) -> u64 {
        self.bytes_before.saturating_sub(self.bytes_removed)
    }

    /// Abandoned transaction directories removed outside the committed-entry totals.
    #[must_use]
    pub const fn uncommitted_directories_removed(self) -> usize {
        self.uncommitted_directories_removed
    }

    /// Logical bytes removed from abandoned transaction directories.
    #[must_use]
    pub const fn uncommitted_bytes_removed(self) -> u64 {
        self.uncommitted_bytes_removed
    }

    pub(super) const fn record_uncommitted_removal(&mut self, bytes: u64) {
        self.uncommitted_directories_removed += 1;
        self.uncommitted_bytes_removed = self.uncommitted_bytes_removed.saturating_add(bytes);
    }
}

impl ArtifactCacheMaintenance {
    /// Successful pruning report, or `None` when maintenance failed.
    #[must_use]
    pub const fn prune_report(&self) -> Option<ArtifactCachePruneReport> {
        match self {
            Self::Pruned(report) => Some(*report),
            Self::PruneFailed { .. } => None,
        }
    }

    /// Rendered maintenance failure, or `None` when pruning succeeded.
    #[must_use]
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            Self::Pruned(_) => None,
            Self::PruneFailed { message } => Some(message),
        }
    }
}

#[derive(Debug)]
pub(super) struct CacheFsError {
    pub(super) operation: &'static str,
    pub(super) path: PathBuf,
    pub(super) source: io::Error,
}

pub(super) fn ensure_cache_directory_tag(cache_root: &Path) -> Result<(), CacheFsError> {
    ic_host_fs::cache_tag::ensure(cache_root, CACHE_DIRECTORY_TAG_SUFFIX.as_bytes(), 0o666).map_err(
        |source| CacheFsError {
            operation: "write cache directory tag",
            path: cache_root.join("CACHEDIR.TAG"),
            source: io::Error::other(source),
        },
    )
}

pub(super) fn lock_cache_file(path: &Path) -> Result<(File, Duration), CacheFsError> {
    let file = open_cache_lock_file(path)?;
    let started = Instant::now();
    ic_host_fs::durable::lock_exclusive(&file).map_err(|source| CacheFsError {
        operation: "lock cache",
        path: path.to_owned(),
        source,
    })?;
    Ok((file, started.elapsed()))
}

pub(super) fn lock_cache_file_with_wait_observer(
    path: &Path,
    poll_interval: Duration,
    mut observer: impl FnMut(Duration),
) -> Result<(File, Duration), CacheFsError> {
    let file = open_cache_lock_file(path)?;
    let wait = ic_host_fs::durable::lock_exclusive_with_wait(
        &file,
        poll_interval.min(Duration::from_millis(25)),
        |elapsed| {
            observer(elapsed);
            Ok(())
        },
    )
    .map_err(|source| CacheFsError {
        operation: "try lock cache",
        path: path.to_owned(),
        source,
    })?;
    Ok((file, wait))
}

pub(super) fn try_lock_cache_file(path: &Path) -> Result<Option<File>, CacheFsError> {
    let result = ic_host_fs::durable::open_regular_lock_file_with_parents(path)
        .map_err(io::Error::from)
        .and_then(|file| ic_host_fs::durable::try_lock_exclusive(&file).map(|()| file));
    match result {
        Ok(file) => Ok(Some(file)),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(None),
        Err(source) => Err(CacheFsError {
            operation: "try lock cache",
            path: path.to_owned(),
            source,
        }),
    }
}

fn open_cache_lock_file(path: &Path) -> Result<File, CacheFsError> {
    ic_host_fs::durable::open_regular_lock_file_with_parents(path)
        .map_err(io::Error::from)
        .map_err(|source| CacheFsError {
            operation: "open cache lock",
            path: path.to_owned(),
            source,
        })
}

pub(super) fn record_cache_entry_use(path: &Path) -> Result<(), CacheFsError> {
    write_last_used(path, SystemTime::now())
}

pub(super) fn cache_maintenance_due(
    path: &Path,
    minimum_interval: Duration,
    maintenance_identity: &str,
) -> Result<bool, CacheFsError> {
    if minimum_interval.is_zero() {
        return Ok(true);
    }
    let marker = path.join(LAST_MAINTENANCE_FILE);
    // Allow both LF and CRLF for the timestamp and policy-identity lines.
    let maximum_len = MAX_TIMESTAMP_BYTES + maintenance_identity.len() + 4;
    let contents = match read_stamp_with_limit(&marker, maximum_len) {
        Ok(Some(contents)) => contents,
        Ok(None) => return Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(true),
        Err(source) => {
            return Err(CacheFsError {
                operation: "read cache maintenance time",
                path: marker,
                source,
            });
        }
    };
    let mut lines = contents.lines();
    let Some(last_maintenance) = lines.next().and_then(decode_system_time) else {
        return Ok(true);
    };
    if lines.next() != Some(maintenance_identity) {
        return Ok(true);
    }
    Ok(SystemTime::now()
        .duration_since(last_maintenance)
        .map_or(true, |elapsed| elapsed >= minimum_interval))
}

pub(super) fn record_cache_maintenance(
    path: &Path,
    maintenance_identity: &str,
) -> Result<(), CacheFsError> {
    let marker = path.join(LAST_MAINTENANCE_FILE);
    let elapsed = encode_system_time(&marker, SystemTime::now())?;
    let contents = format!("{}\n{maintenance_identity}\n", elapsed.as_nanos());
    ic_host_fs::durable::write_bytes(
        &marker,
        ic_host_fs::durable::WriteOptions {
            mode: ic_host_fs::durable::PublicationMode::Replace,
            permissions: 0o666,
        },
        contents.as_bytes(),
    )
    .map_err(|source| CacheFsError {
        operation: "record cache maintenance time",
        path: marker,
        source: io::Error::other(source),
    })
}

pub(super) fn perform_scheduled_cache_maintenance(
    path: &Path,
    minimum_interval: Duration,
    maintenance_identity: &str,
    maintenance: impl FnOnce() -> Result<ArtifactCachePruneReport, String>,
) -> (Option<ArtifactCacheMaintenance>, Option<Duration>) {
    let started = Instant::now();
    match cache_maintenance_due(path, minimum_interval, maintenance_identity) {
        Ok(false) => return (None, Some(started.elapsed())),
        Ok(true) => {}
        Err(error) => {
            return (
                Some(ArtifactCacheMaintenance::PruneFailed {
                    message: error.to_string(),
                }),
                Some(started.elapsed()),
            );
        }
    }

    let result = maintenance();
    let marker = record_cache_maintenance(path, maintenance_identity);
    let outcome = match (result, marker) {
        (Ok(report), Ok(())) => ArtifactCacheMaintenance::Pruned(report),
        (Err(message), Ok(())) => ArtifactCacheMaintenance::PruneFailed { message },
        (Ok(_), Err(error)) => ArtifactCacheMaintenance::PruneFailed {
            message: error.to_string(),
        },
        (Err(message), Err(marker)) => ArtifactCacheMaintenance::PruneFailed {
            message: format!(
                "{message}; additionally failed to record the maintenance attempt: {marker}"
            ),
        },
    };
    (Some(outcome), Some(started.elapsed()))
}

pub(super) fn write_last_used(path: &Path, last_used: SystemTime) -> Result<(), CacheFsError> {
    let marker = path.join(LAST_USED_FILE);
    write_system_time(&marker, last_used, "record cache use time")
}

fn write_system_time(
    path: &Path,
    timestamp: SystemTime,
    operation: &'static str,
) -> Result<(), CacheFsError> {
    let elapsed = encode_system_time(path, timestamp)?;
    ic_host_fs::durable::write_bytes(
        path,
        ic_host_fs::durable::WriteOptions {
            mode: ic_host_fs::durable::PublicationMode::Replace,
            permissions: 0o666,
        },
        elapsed.as_nanos().to_string().as_bytes(),
    )
    .map_err(|source| CacheFsError {
        operation,
        path: path.to_owned(),
        source: io::Error::other(source),
    })
}

fn encode_system_time(path: &Path, timestamp: SystemTime) -> Result<Duration, CacheFsError> {
    timestamp
        .duration_since(UNIX_EPOCH)
        .map_err(|source| CacheFsError {
            operation: "encode cache time",
            path: path.to_owned(),
            source: io::Error::new(io::ErrorKind::InvalidInput, source),
        })
}

fn decode_system_time(contents: &str) -> Option<SystemTime> {
    let nanoseconds = contents.parse::<u128>().ok()?;
    let seconds = u64::try_from(nanoseconds / 1_000_000_000).ok()?;
    let subsecond_nanos = (nanoseconds % 1_000_000_000) as u32;
    UNIX_EPOCH.checked_add(Duration::new(seconds, subsecond_nanos))
}

pub(super) fn prune_direct_child_directories(
    cache_root: &Path,
    policy: CacheRetentionLimits,
    protected_entry: Option<&Path>,
    is_eligible: impl Fn(&Path) -> bool,
) -> Result<ArtifactCachePruneReport, CacheFsError> {
    let mut entries = cache_entries(cache_root, is_eligible)?;
    let bytes_before = entries
        .iter()
        .fold(0_u64, |total, entry| total.saturating_add(entry.bytes));
    let mut report = ArtifactCachePruneReport {
        entries_scanned: entries.len(),
        entries_removed: 0,
        bytes_before,
        bytes_removed: 0,
        uncommitted_directories_removed: 0,
        uncommitted_bytes_removed: 0,
    };
    let now = SystemTime::now();

    if let Some(max_age) = policy.max_age() {
        for entry in &mut entries {
            let age = now.duration_since(entry.last_used).unwrap_or_default();
            if protected_entry != Some(entry.path.as_path()) && age > max_age {
                remove_cache_entry(entry, &mut report)?;
            }
        }
    }

    if let Some(max_size_bytes) = policy.max_size_bytes()
        && report.bytes_retained() > max_size_bytes
    {
        entries.sort_by(|left, right| {
            left.last_used
                .cmp(&right.last_used)
                .then_with(|| left.path.cmp(&right.path))
        });
        for entry in &mut entries {
            if report.bytes_retained() <= max_size_bytes {
                break;
            }
            if protected_entry == Some(entry.path.as_path()) {
                continue;
            }
            remove_cache_entry(entry, &mut report)?;
        }
    }

    Ok(report)
}

pub(super) fn is_sha256_directory(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        let bytes = name.as_encoded_bytes();
        bytes.len() == 64 && bytes.iter().all(u8::is_ascii_hexdigit)
    })
}

struct CacheEntry {
    path: PathBuf,
    bytes: u64,
    last_used: SystemTime,
    removed: bool,
}

impl std::fmt::Display for CacheFsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "failed to {} at {}: {}",
            self.operation,
            self.path.display(),
            self.source
        )
    }
}

impl std::error::Error for CacheFsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

fn cache_entries(
    cache_root: &Path,
    is_eligible: impl Fn(&Path) -> bool,
) -> Result<Vec<CacheEntry>, CacheFsError> {
    let read_dir = match fs::read_dir(cache_root) {
        Ok(read_dir) => read_dir,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(CacheFsError {
                operation: "read cache directory",
                path: cache_root.to_owned(),
                source,
            });
        }
    };
    let mut entries = Vec::new();
    for directory_entry in read_dir {
        let directory_entry = directory_entry.map_err(|source| CacheFsError {
            operation: "read cache entry",
            path: cache_root.to_owned(),
            source,
        })?;
        let path = directory_entry.path();
        let file_type = directory_entry.file_type().map_err(|source| CacheFsError {
            operation: "inspect cache entry",
            path: path.clone(),
            source,
        })?;
        if !file_type.is_dir() || !is_eligible(&path) {
            continue;
        }
        let bytes = directory_logical_size(&path).map_err(|source| CacheFsError {
            operation: "measure cache entry",
            path: path.clone(),
            source,
        })?;
        let last_used = cache_entry_last_used(&path).map_err(|source| CacheFsError {
            operation: "read cache use time",
            path: path.clone(),
            source,
        })?;
        entries.push(CacheEntry {
            path,
            bytes,
            last_used,
            removed: false,
        });
    }
    Ok(entries)
}

pub(super) fn cache_entry_last_used(path: &Path) -> io::Result<SystemTime> {
    let marker = path.join(LAST_USED_FILE);
    if let Ok(Some(contents)) = read_stamp_with_limit(&marker, MAX_TIMESTAMP_BYTES)
        && let Some(timestamp) = decode_system_time(&contents)
    {
        return Ok(timestamp);
    }
    fs::metadata(path)?.modified()
}

fn remove_cache_entry(
    entry: &mut CacheEntry,
    report: &mut ArtifactCachePruneReport,
) -> Result<(), CacheFsError> {
    if entry.removed {
        return Ok(());
    }
    let Some(_retention_lock) = try_lock_cache_file(&entry.path.join(RETENTION_LOCK_FILE))? else {
        return Ok(());
    };
    remove_path_if_present(&entry.path).map_err(|source| CacheFsError {
        operation: "prune cache entry",
        path: entry.path.clone(),
        source,
    })?;
    entry.removed = true;
    report.entries_removed += 1;
    report.bytes_removed = report.bytes_removed.saturating_add(entry.bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::fs_fixture::unique_temp_directory;
    use std::{
        fs,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    #[cfg(unix)]
    #[test]
    fn cache_lock_admission_preserves_bytes_and_returns_an_unlocked_close_on_exec_file() {
        use std::os::fd::AsRawFd as _;

        let root = unique_temp_directory("cache-lock-admission");
        let path = root.join("nested/lock");
        let file = super::open_cache_lock_file(&path).unwrap();
        assert!(super::try_lock_cache_file(&path).unwrap().is_some());
        // SAFETY: inspect descriptor flags on the live file without changing them.
        #[expect(
            unsafe_code,
            reason = "inspect flags of a live descriptor without changing them"
        )]
        let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) };
        assert!(flags >= 0);
        assert_ne!(flags & libc::FD_CLOEXEC, 0);
        drop(file);
        fs::write(&path, b"retained lock bytes").unwrap();
        drop(super::lock_cache_file(&path).unwrap());
        assert_eq!(fs::read(&path).unwrap(), b"retained lock bytes");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn cache_lock_callers_refuse_redirected_final_entries() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_directory("cache-lock-redirected");
        let target = root.join("target");
        fs::write(&target, b"target bytes").unwrap();
        let redirected = root.join("redirected");
        symlink(&target, &redirected).unwrap();
        for result in [
            super::open_cache_lock_file(&redirected),
            super::lock_cache_file(&redirected).map(|(file, _)| file),
            super::lock_cache_file_with_wait_observer(
                &redirected,
                Duration::from_millis(5),
                |_| panic!("redirected lock reached acquisition"),
            )
            .map(|(file, _)| file),
        ] {
            let error = result.unwrap_err();
            assert_eq!(error.path, redirected);
            assert!(matches!(
                error.source.get_ref().and_then(|cause| cause
                    .downcast_ref::<ic_host_fs::durable::RegularFileLockError>(
                )),
                Some(ic_host_fs::durable::RegularFileLockError::NotRegular)
            ));
        }
        symlink(&target, root.join(super::RETENTION_LOCK_FILE)).unwrap();
        assert!(super::RetainedCacheEntry::acquire(&root).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"target bytes");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn nonblocking_cache_locks_preserve_contention_and_reject_redirected_files() {
        use std::{os::unix::fs::symlink, process::Command};
        const CHILD_ENV: &str = "IC_TESTKIT_NONBLOCKING_CACHE_LOCK_CHILD";

        if std::env::var_os(CHILD_ENV).is_none() {
            // Parallel tests can fork with the lock descriptor open. CLOEXEC
            // releases the child's copy only at exec, so isolate immediate
            // reacquisition after dropping the parent's descriptor.
            let thread = std::thread::current();
            let test_name = thread.name().expect("libtest names each test thread");
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", test_name, "--test-threads=1"])
                .env(CHILD_ENV, "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "isolated nonblocking cache lock test failed: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        let root = unique_temp_directory("nonblocking-cache-lock-admission");
        let lock_path = root.join("lock");
        fs::write(&lock_path, b"existing lock bytes").unwrap();
        let held = super::try_lock_cache_file(&lock_path).unwrap().unwrap();
        assert!(super::try_lock_cache_file(&lock_path).unwrap().is_none());
        assert_eq!(fs::read(&lock_path).unwrap(), b"existing lock bytes");
        drop(held);
        assert!(super::try_lock_cache_file(&lock_path).unwrap().is_some());

        let redirected = root.join("redirected-lock");
        symlink(&lock_path, &redirected).unwrap();
        let error = super::try_lock_cache_file(&redirected).unwrap_err();
        assert_eq!(error.path, redirected);
        assert!(matches!(
                error.source.get_ref().and_then(|cause| cause
                    .downcast_ref::<ic_host_fs::durable::RegularFileLockError>(
                )),
                Some(ic_host_fs::durable::RegularFileLockError::NotRegular)
            ));
        assert_eq!(fs::read(&lock_path).unwrap(), b"existing lock bytes");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn final_retention_owner_releases_lock_with_a_duplicate_descriptor_open() {
        let root = unique_temp_directory("retention-duplicate-descriptor");
        let retained = super::RetainedCacheEntry::acquire(&root).unwrap();
        // A process spawn can duplicate this descriptor before close-on-exec.
        #[expect(
            clippy::used_underscore_binding,
            reason = "inspect the RAII guard to verify inherited-descriptor retention"
        )]
        let inherited = retained._lock.file().try_clone().unwrap();
        let clone = retained.clone();
        let independently_retained = super::RetainedCacheEntry::acquire(&root).unwrap();
        let lock_path = root.join(super::RETENTION_LOCK_FILE);
        let available = || super::try_lock_cache_file(&lock_path).unwrap().is_some();
        drop(retained);
        assert!(!available(), "a record clone still retains the entry");
        drop(clone);
        assert!(
            !available(),
            "an independent acquisition still retains the entry"
        );
        drop(independently_retained);
        assert!(
            available(),
            "descriptor duplication must not extend record ownership"
        );
        drop(inherited);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn last_use_markers_preserve_timestamps_and_bounded_fallbacks() {
        let root = unique_temp_directory("bounded-last-use-marker");
        let marker = root.join(super::LAST_USED_FILE);
        let modified = || fs::metadata(&root).unwrap().modified().unwrap();
        assert_eq!(super::cache_entry_last_used(&root).unwrap(), modified());
        for timestamp in [
            UNIX_EPOCH + Duration::from_nanos(123),
            SystemTime::now() + Duration::from_secs(3600),
        ] {
            super::write_last_used(&root, timestamp).unwrap();
            assert_eq!(super::cache_entry_last_used(&root).unwrap(), timestamp);
        }
        let overflowing_timestamp = u128::MAX.to_string();
        for invalid in [
            b"invalid".as_slice(),
            overflowing_timestamp.as_bytes(),
            &[0xff],
        ] {
            fs::write(&marker, invalid).unwrap();
            assert_eq!(super::cache_entry_last_used(&root).unwrap(), modified());
        }
        fs::File::create(&marker)
            .unwrap()
            .set_len(1024 * 1024 * 1024)
            .unwrap();
        assert_eq!(super::cache_entry_last_used(&root).unwrap(), modified());
        fs::remove_file(&marker).unwrap();
        fs::create_dir(&marker).unwrap();
        assert_eq!(super::cache_entry_last_used(&root).unwrap(), modified());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn maintenance_markers_preserve_policy_intervals_and_read_errors() {
        let root = unique_temp_directory("bounded-maintenance-marker");
        let marker = root.join(super::LAST_MAINTENANCE_FILE);
        let identity = super::CacheRetentionLimits::new().maintenance_identity();
        let interval = Duration::from_secs(3600);
        let due = || super::cache_maintenance_due(&root, interval, &identity).unwrap();
        assert!(due());
        super::record_cache_maintenance(&root, &identity).unwrap();
        assert!(!due());
        assert!(super::cache_maintenance_due(&root, interval, "other-policy").unwrap());
        assert!(super::cache_maintenance_due(&root, Duration::ZERO, &identity).unwrap());

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        for suffix in ["", "\r\n"] {
            fs::write(&marker, format!("{}\r\n{identity}{suffix}", now.as_nanos())).unwrap();
            assert!(!due());
        }
        for timestamp in [
            "invalid".to_owned(),
            "0".to_owned(),
            (now + Duration::from_secs(3600)).as_nanos().to_string(),
        ] {
            fs::write(&marker, format!("{timestamp}\n{identity}\n")).unwrap();
            assert!(due());
        }
        super::record_cache_maintenance(&root, &identity).unwrap();
        fs::OpenOptions::new()
            .write(true)
            .open(&marker)
            .unwrap()
            .set_len(1024 * 1024 * 1024)
            .unwrap();
        assert!(due());

        fs::write(&marker, [0xff]).unwrap();
        assert!(super::cache_maintenance_due(&root, Duration::ZERO, &identity).unwrap());
        let error = super::cache_maintenance_due(&root, interval, &identity).unwrap_err();
        assert_eq!(error.operation, "read cache maintenance time");
        assert_eq!(error.path, marker);
        assert_eq!(error.source.kind(), std::io::ErrorKind::InvalidData);
        fs::remove_file(&marker).unwrap();
        fs::create_dir(&marker).unwrap();
        assert!(super::cache_maintenance_due(&root, interval, &identity).is_err());
        assert!(super::cache_maintenance_due(&root, Duration::ZERO, &identity).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cache_directory_tags_preserve_valid_standard_signatures() {
        let root = unique_temp_directory("cache-tag-signatures");
        let tag = root.join("CACHEDIR.TAG");
        let signature = ic_host_fs::cache_tag::SIGNATURE;
        for contents in [
            signature.to_owned(),
            format!("{signature}\r\n# Created by another application\r\n"),
            format!("{signature}\n# {}\n", "comment".repeat(100_000)),
        ] {
            fs::write(&tag, &contents).unwrap();
            super::ensure_cache_directory_tag(&root).unwrap();
            assert_eq!(fs::read_to_string(&tag).unwrap(), contents);
        }
        for invalid in ["", &signature[..42], "Signature: incorrect"] {
            fs::write(&tag, invalid).unwrap();
            super::ensure_cache_directory_tag(&root).unwrap();
            assert_eq!(
                fs::read_to_string(&tag).unwrap(),
                format!("{signature}{}", super::CACHE_DIRECTORY_TAG_SUFFIX)
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn cache_directory_tag_replaces_symlinks_without_changing_referents() {
        let root = unique_temp_directory("cache-tag-symlink");
        let referent = root.join("other-application-tag");
        let contents = "Signature: 8a477f597d28d172789f06886806bc55\n# Preserve this file\n";
        fs::write(&referent, contents).unwrap();
        let tag = root.join("CACHEDIR.TAG");
        std::os::unix::fs::symlink(&referent, &tag).unwrap();
        super::ensure_cache_directory_tag(&root).unwrap();
        assert!(fs::symlink_metadata(&tag).unwrap().file_type().is_file());
        assert_eq!(fs::read_to_string(&referent).unwrap(), contents);
        assert_eq!(
            fs::read_to_string(&tag).unwrap(),
            format!(
                "{}{}",
                ic_host_fs::cache_tag::SIGNATURE,
                super::CACHE_DIRECTORY_TAG_SUFFIX
            )
        );

        fs::remove_file(&tag).unwrap();
        fs::create_dir(&tag).unwrap();
        let error = super::ensure_cache_directory_tag(&root).unwrap_err();
        assert!(matches!(
            error
                .source
                .get_ref()
                .unwrap()
                .downcast_ref::<ic_host_fs::durable::NamedWriteError<std::io::Error>>()
                .unwrap(),
            ic_host_fs::durable::NamedWriteError::BeforePublication {
                cleanup_error: None,
                ..
            }
        ));
        assert_eq!(error.operation, "write cache directory tag");
        assert_eq!(error.path, tag);
        assert!(tag.is_dir());
        fs::remove_dir_all(root).unwrap();
    }
}
