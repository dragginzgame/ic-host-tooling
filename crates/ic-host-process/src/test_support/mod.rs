use crate::tool::{AdmittedTool, ExecutionContext, OutputLimits, VersionSpec};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    time::Duration,
};

pub const LIMITS: OutputLimits = OutputLimits {
    stdout: crate::tool::OutputLimit::Terminate(256 * 1024),
    stderr: crate::tool::OutputLimit::Terminate(256 * 1024),
    timeout: Some(Duration::from_secs(5)),
};
pub const VERSION: &str = "test-host-tool version 1";

pub struct Fixture {
    pub root: PathBuf,
    _directory: ic_host_fs::testing::FixtureDirectory,
}

impl Fixture {
    pub fn new() -> Self {
        let directory = ic_host_fs::testing::FixtureDirectory::new(
            &std::env::temp_dir(),
            "ic-host-process-test",
        )
        .expect("create test fixture");
        let root = directory.path().to_path_buf();
        Self {
            root,
            _directory: directory,
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
    AdmittedTool::admit_version(
        &VersionSpec {
            executable: path,
            executable_bytes: 1024 * 1024,
            version_arguments: &[OsString::from("--version")],
        },
        version,
        context,
        LIMITS,
    )
}
