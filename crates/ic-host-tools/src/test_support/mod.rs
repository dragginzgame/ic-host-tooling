use ic_host_process::tool::{AdmittedTool, ExecutionContext, OutputLimits, VersionSpec};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
    time::Duration,
};

static PROCESS_FIXTURES: Mutex<()> = Mutex::new(());

pub const LIMITS: OutputLimits = OutputLimits {
    stdout: ic_host_process::tool::OutputLimit::Terminate(256 * 1024),
    stderr: ic_host_process::tool::OutputLimit::Terminate(256 * 1024),
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
        let root = ic_host_fs::testing::create_fixture_directory(
            &std::env::temp_dir(),
            "ic-host-tools-test",
        )
        .expect("create tools fixture");
        Self {
            root,
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
        let path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/tool.sh"
        ))
        .canonicalize()
        .unwrap();
        AdmittedTool::admit_version(
            &VersionSpec {
                executable: &path,
                executable_bytes: 1024 * 1024,
                version_arguments: &[OsString::from("--version")],
                version_identity: VERSION,
            },
            &self.context(&[]),
            LIMITS,
        )
        .unwrap()
    }

    pub fn source(&self) -> PathBuf {
        let source = self.root.join("source.wasm");
        fs::write(&source, b"\0asm\x01\0\0\0").unwrap();
        source
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("Failed tools fixture retained: {}", self.root.display());
            return;
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}
