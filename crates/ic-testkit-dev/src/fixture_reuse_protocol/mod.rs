//! Repository-local worker invocation; callers retain workload and report admission.

use std::{ffi::OsString, num::NonZeroUsize, path::PathBuf};

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Fresh,
    Pooled,
}

impl Mode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Pooled => "pooled",
        }
    }
}

#[derive(clap::Parser)]
#[command(
    name = "fixture_reuse_benchmark",
    about = "Run one fixture-reuse benchmark workload",
    version,
    no_binary_name = true
)]
pub struct WorkerArguments {
    #[arg(value_name = "mode")]
    pub mode: Mode,
    #[arg(value_name = "capacity")]
    pub capacity: NonZeroUsize,
    #[arg(value_name = "iterations")]
    pub iterations: NonZeroUsize,
    #[arg(value_name = "workers")]
    pub workers: NonZeroUsize,
    #[arg(value_name = "state_bytes")]
    pub state_bytes: u32,
    #[arg(value_name = "wasm", allow_hyphen_values = true)]
    pub wasm: PathBuf,
    #[arg(value_name = "server", allow_hyphen_values = true)]
    pub server: PathBuf,
}

impl TryFrom<Vec<OsString>> for WorkerArguments {
    type Error = clap::Error;

    fn try_from(args: Vec<OsString>) -> Result<Self, Self::Error> {
        use clap::{CommandFactory, Parser, error::ErrorKind};
        let result = Self::try_parse_from(args)?;
        if result.workers > result.iterations {
            return Err(Self::command().error(
                ErrorKind::ValueValidation,
                "workers must not exceed iterations",
            ));
        }
        Ok(result)
    }
}

impl From<WorkerArguments> for [OsString; 7] {
    fn from(args: WorkerArguments) -> Self {
        [
            args.mode.as_str().into(),
            args.capacity.to_string().into(),
            args.iterations.to_string().into(),
            args.workers.to_string().into(),
            args.state_bytes.to_string().into(),
            args.wasm.into_os_string(),
            args.server.into_os_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{Mode, WorkerArguments};
    use std::ffi::OsString;

    #[test]
    fn worker_invocation_admits_both_modes_and_zero_state() {
        for mode in [Mode::Fresh, Mode::Pooled] {
            let args = [
                mode.as_str(),
                "2",
                "4",
                "2",
                "0",
                "-probe wasm",
                "-server path",
            ]
            .map(OsString::from);
            let parsed = WorkerArguments::try_from(args.to_vec()).unwrap();
            assert_eq!(parsed.mode, mode);
            assert_eq!(parsed.capacity.get(), 2);
            assert_eq!(parsed.iterations.get(), 4);
            assert_eq!(parsed.workers.get(), 2);
            assert_eq!(parsed.state_bytes, 0);
            assert_eq!(<[OsString; 7]>::from(parsed), args);
        }
    }

    #[test]
    fn invalid_worker_invocations_refuse_before_work_starts() {
        for args in [
            vec!["fresh", "2", "4", "2", "0", "wasm"],
            vec!["fresh", "2", "4", "2", "0", "wasm", "server", "extra"],
            vec!["unknown", "2", "4", "2", "0", "wasm", "server"],
            vec!["fresh", "0", "4", "2", "0", "wasm", "server"],
            vec!["fresh", "2", "0", "2", "0", "wasm", "server"],
            vec!["fresh", "2", "4", "0", "0", "wasm", "server"],
            vec!["fresh", "2", "4", "5", "0", "wasm", "server"],
            vec!["fresh", "2", "4", "2", "4294967296", "wasm", "server"],
        ] {
            let error =
                WorkerArguments::try_from(args.into_iter().map(OsString::from).collect::<Vec<_>>())
                    .err()
                    .unwrap();
            assert_eq!(error.exit_code(), 2);
        }
    }

    #[cfg(unix)]
    #[test]
    fn worker_paths_preserve_native_non_utf8_bytes() {
        use std::os::unix::ffi::OsStringExt as _;
        let mut args = ["fresh", "1", "1", "1", "0", "wasm", "server"].map(OsString::from);
        args[5] = OsString::from_vec(vec![b'w', 255]);
        args[6] = OsString::from_vec(vec![b's', 254]);
        assert_eq!(
            <[OsString; 7]>::from(WorkerArguments::try_from(args.to_vec()).unwrap()),
            args
        );
    }
}
