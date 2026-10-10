use ic_host_fs::read::hash_file;

use crate::tool::{AdmittedTool, ExecutionContext, OutputLimits, ToolSpec};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

static PROCESS_FIXTURES: Mutex<()> = Mutex::new(());
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub const LIMITS: OutputLimits = OutputLimits {
    stdout: crate::tool::OutputLimit::Terminate(256 * 1024),
    stderr: crate::tool::OutputLimit::Terminate(256 * 1024),
    timeout: Some(Duration::from_secs(5)),
};
pub const VERSION: &str = "test-host-tool version 1";

pub struct Fixture {
    pub root: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl Fixture {
    pub fn new() -> Self {
        // Serialize mutable executable-fixture writes against subprocess fork
        // within this test binary; no production executable-busy retry is added.
        let guard = PROCESS_FIXTURES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let root = std::env::temp_dir().join(format!(
            "ic-host-tools-test-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self {
            root: root.canonicalize().unwrap(),
            _guard: guard,
        }
    }

    pub fn context<'a>(&'a self, environment: &'a [(OsString, OsString)]) -> ExecutionContext<'a> {
        ExecutionContext {
            current_dir: &self.root,
            environment,
        }
    }

    pub fn tool(&self) -> AdmittedTool {
        admit(&tool_path(), &self.context(&[]), VERSION).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub fn tool_path() -> PathBuf {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tool.sh"
    ))
    .canonicalize()
    .unwrap()
}

pub fn admit(
    path: &Path,
    context: &ExecutionContext<'_>,
    version: &str,
) -> Result<AdmittedTool, crate::tool::ToolError> {
    let identity = hash_file(path, 1024 * 1024).unwrap();
    AdmittedTool::admit(
        &ToolSpec {
            executable: path,
            sha256: identity.sha256,
            executable_bytes: 1024 * 1024,
            version_arguments: &[OsString::from("--version")],
            version_identity: version,
        },
        context,
        LIMITS,
    )
}
