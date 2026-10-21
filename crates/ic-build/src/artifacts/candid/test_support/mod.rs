use ic_host_process::tool::{AdmittedTool, ExecutionContext, OutputLimits, VersionSpec};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

pub const LIMITS: OutputLimits = OutputLimits {
    stdout: ic_host_process::tool::OutputLimit::Terminate(256 * 1024),
    stderr: ic_host_process::tool::OutputLimit::Terminate(256 * 1024),
    timeout: Some(Duration::from_secs(5)),
};
pub const VERSION: &str = "test-host-tool version 1";

pub struct Fixture {
    pub root: ic_host_fs::testing::FixtureDirectory,
}

impl Fixture {
    pub fn new() -> Self {
        let root = ic_host_fs::testing::FixtureDirectory::new(
            &std::env::temp_dir(),
            "ic-build-candid-test",
        )
        .expect("create tools fixture");
        Self { root }
    }

    pub fn context<'a>(&'a self, environment: &'a [(OsString, OsString)]) -> ExecutionContext<'a> {
        ExecutionContext {
            current_dir: self.root.path(),
            environment,
        }
    }

    pub fn tool(&self) -> AdmittedTool {
        let path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/candid-extractor.sh"
        ))
        .canonicalize()
        .unwrap();
        AdmittedTool::admit_version(
            &VersionSpec {
                executable: &path,
                executable_bytes: 1024 * 1024,
                version_arguments: &[OsString::from("--version")],
            },
            VERSION,
            &self.context(&[]),
            LIMITS,
        )
        .unwrap()
    }

    pub fn source(&self) -> PathBuf {
        let source = self.root.path().join("source.wasm");
        fs::write(&source, b"\0asm\x01\0\0\0").unwrap();
        source
    }
}
