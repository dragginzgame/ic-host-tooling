//! Repository-local worker invocation; callers retain workload and report admission.

use std::{ffi::OsString, num::NonZeroUsize, path::PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
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

impl clap::ValueEnum for Mode {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Fresh, Self::Pooled]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(self.as_str()))
    }
}

pub struct WorkerArguments {
    pub mode: Mode,
    pub capacity: NonZeroUsize,
    pub iterations: NonZeroUsize,
    pub workers: NonZeroUsize,
    pub state_bytes: u32,
    pub wasm: PathBuf,
    pub server: PathBuf,
}

impl TryFrom<Vec<OsString>> for WorkerArguments {
    type Error = clap::Error;

    fn try_from(args: Vec<OsString>) -> Result<Self, Self::Error> {
        use clap::{Arg, Command, error::ErrorKind, value_parser};
        let mut command = Command::new("fixture_reuse_benchmark")
            .about("Run one fixture-reuse benchmark workload")
            .version(env!("CARGO_PKG_VERSION"))
            .no_binary_name(true)
            .arg(
                Arg::new("mode")
                    .required(true)
                    .value_parser(value_parser!(Mode)),
            )
            .args(["capacity", "iterations", "workers"].map(|name| {
                Arg::new(name)
                    .required(true)
                    .value_parser(value_parser!(NonZeroUsize))
            }))
            .arg(
                Arg::new("state_bytes")
                    .required(true)
                    .value_parser(value_parser!(u32)),
            )
            .args(["wasm", "server"].map(|name| {
                Arg::new(name)
                    .required(true)
                    .value_parser(value_parser!(PathBuf))
                    .allow_hyphen_values(true)
            }));
        let mut matches = command.try_get_matches_from_mut(args)?;
        let result = Self {
            mode: matches.remove_one::<Mode>("mode").expect("mode required"),
            capacity: matches
                .remove_one::<NonZeroUsize>("capacity")
                .expect("capacity required"),
            iterations: matches
                .remove_one::<NonZeroUsize>("iterations")
                .expect("iterations required"),
            workers: matches
                .remove_one::<NonZeroUsize>("workers")
                .expect("workers required"),
            state_bytes: matches
                .remove_one::<u32>("state_bytes")
                .expect("state bytes required"),
            wasm: matches
                .remove_one::<PathBuf>("wasm")
                .expect("Wasm required"),
            server: matches
                .remove_one::<PathBuf>("server")
                .expect("server required"),
        };
        if result.workers > result.iterations {
            return Err(command.error(
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
