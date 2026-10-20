//! Validates the operator's workload selection before launching any tools.

use ic_host_process::tool::{OutputLimit, OutputLimits};
use std::{ffi::OsString, num::NonZeroUsize, path::PathBuf};

use crate::fixture_reuse_protocol::Mode;
use clap::{Arg, Command, error::ErrorKind, value_parser};

pub(super) const RSS_COMMAND_LIMITS: OutputLimits = OutputLimits {
    stdout: OutputLimit::Terminate(16 * 1024 * 1024),
    stderr: OutputLimit::Terminate(64 * 1024),
    timeout: None,
};

pub(super) struct Options {
    pub(super) server: PathBuf,
    pub(super) iterations: NonZeroUsize,
    pub(super) workers: NonZeroUsize,
    pub(super) repeats: NonZeroUsize,
    pub(super) modes: Vec<Mode>,
    pub(super) capacities: Vec<NonZeroUsize>,
    pub(super) state_bytes: u32,
    pub(super) profile: String,
    pub(super) output: Option<PathBuf>,
    pub(super) rss_limits: OutputLimits,
}

pub(super) struct Case {
    pub(super) label: String,
    pub(super) mode: Mode,
    pub(super) capacity: NonZeroUsize,
}

impl Options {
    pub(super) fn benchmark_cases(&self) -> Vec<Case> {
        self.modes
            .iter()
            .flat_map(|mode| match mode {
                Mode::Fresh => vec![Case {
                    label: "fresh".into(),
                    mode: *mode,
                    capacity: self.workers,
                }],
                Mode::Pooled => self
                    .capacities
                    .iter()
                    .map(|capacity| Case {
                        label: format!("pooled-{capacity}"),
                        mode: *mode,
                        capacity: *capacity,
                    })
                    .collect(),
            })
            .collect()
    }
}

fn cli() -> Command {
    Command::new("fixture_reuse_benchmark_driver")
        .about("Fixture reuse benchmark for Linux/macOS")
        .after_help(
            "Builds use the selected lockfile and prepared offline caches; no server download.",
        )
        .version(env!("CARGO_PKG_VERSION"))
        .no_binary_name(true)
        .args_override_self(true)
        .arg(
            Arg::new("server")
                .long("server")
                .required(true)
                .help("Exact PocketIC server binary")
                .value_parser(value_parser!(PathBuf))
                .allow_hyphen_values(true),
        )
        .args(
            [("iterations", "100"), ("workers", "2"), ("repeats", "3")].map(|(name, default)| {
                Arg::new(name)
                    .long(name)
                    .default_value(default)
                    .value_parser(value_parser!(NonZeroUsize))
            }),
        )
        .arg(
            Arg::new("modes")
                .long("modes")
                .num_args(1..)
                .default_values(["fresh", "pooled"])
                .value_parser(value_parser!(Mode)),
        )
        .arg(
            Arg::new("capacities")
                .long("capacities")
                .num_args(1..)
                .default_values(["1", "2"])
                .value_parser(value_parser!(NonZeroUsize)),
        )
        .arg(
            Arg::new("state-bytes")
                .long("state-bytes")
                .default_value("1048576")
                .value_parser(value_parser!(u32)),
        )
        .arg(
            Arg::new("profile")
                .long("profile")
                .default_value("release")
                .value_parser(["release", "dev"]),
        )
        .arg(
            Arg::new("output")
                .long("output")
                .value_parser(value_parser!(PathBuf))
                .allow_hyphen_values(true),
        )
        .args(
            [
                (
                    "rss-stdout-bytes",
                    RSS_COMMAND_LIMITS.stdout.retained_bytes(),
                ),
                (
                    "rss-stderr-bytes",
                    RSS_COMMAND_LIMITS.stderr.retained_bytes(),
                ),
            ]
            .map(|(name, default)| {
                Arg::new(name)
                    .long(name)
                    .default_value(default.to_string())
                    .value_parser(value_parser!(NonZeroUsize))
            }),
        )
}

pub(super) fn parse_arguments(
    args: impl IntoIterator<Item = OsString>,
) -> Result<Options, clap::Error> {
    let mut command = cli();
    let mut matches = command.try_get_matches_from_mut(args)?;
    let options = Options {
        server: matches
            .remove_one::<PathBuf>("server")
            .expect("server required"),
        iterations: matches
            .remove_one::<NonZeroUsize>("iterations")
            .expect("default iterations"),
        workers: matches
            .remove_one::<NonZeroUsize>("workers")
            .expect("default workers"),
        repeats: matches
            .remove_one::<NonZeroUsize>("repeats")
            .expect("default repeats"),
        modes: matches
            .remove_many::<Mode>("modes")
            .expect("default modes")
            .collect(),
        capacities: matches
            .remove_many::<NonZeroUsize>("capacities")
            .expect("default capacities")
            .collect(),
        state_bytes: matches
            .remove_one::<u32>("state-bytes")
            .expect("default state bytes"),
        profile: matches
            .remove_one::<String>("profile")
            .expect("default profile"),
        output: matches.remove_one::<PathBuf>("output"),
        rss_limits: OutputLimits {
            stdout: OutputLimit::Terminate(
                matches
                    .remove_one::<NonZeroUsize>("rss-stdout-bytes")
                    .expect("default stdout limit")
                    .get(),
            ),
            stderr: OutputLimit::Terminate(
                matches
                    .remove_one::<NonZeroUsize>("rss-stderr-bytes")
                    .expect("default stderr limit")
                    .get(),
            ),
            timeout: RSS_COMMAND_LIMITS.timeout,
        },
    };
    // Clap owns parsing and individual value constraints. These relationships
    // compare independently typed values and select a unique workload matrix.
    if options.workers > options.iterations {
        return Err(command.error(
            ErrorKind::ValueValidation,
            "workers must not exceed iterations",
        ));
    }
    if options
        .modes
        .iter()
        .enumerate()
        .any(|(index, value)| options.modes[..index].contains(value))
        || options
            .capacities
            .iter()
            .enumerate()
            .any(|(index, value)| options.capacities[..index].contains(value))
    {
        return Err(command.error(
            ErrorKind::ValueValidation,
            "each mode and capacity must be selected only once",
        ));
    }
    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::parse_arguments;

    #[test]
    fn help_is_owned_by_clap_before_workload_admission() {
        let error = parse_arguments(["--help"].map(Into::into))
            .err()
            .expect("display help");
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
        assert_eq!(error.exit_code(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn last_selection_and_native_path_bytes_are_preserved() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt as _};
        let server = OsString::from_vec(vec![b's', 255]);
        let output = OsString::from_vec(vec![b'o', 254]);
        let args = [
            OsString::from("--server"),
            server.clone(),
            "--output".into(),
            output.clone(),
            "--workers".into(),
            "3".into(),
            "--workers".into(),
            "2".into(),
        ];
        let options = parse_arguments(args).unwrap();
        assert_eq!(options.server.as_os_str(), &server);
        assert_eq!(options.output.unwrap().as_os_str(), &output);
        assert_eq!(options.workers.get(), 2);
    }

    #[test]
    fn capacity_sweep_preserves_order_and_one_fresh_control() {
        let options = parse_arguments(
            [
                "--server",
                "/unused/pocket-ic",
                "--iterations",
                "8",
                "--workers",
                "8",
                "--modes",
                "pooled",
                "fresh",
                "--capacities",
                "1",
                "2",
                "4",
                "8",
            ]
            .map(Into::into),
        )
        .unwrap();
        let cases = options
            .benchmark_cases()
            .into_iter()
            .map(|case| (case.label, case.mode.as_str(), case.capacity.get()))
            .collect::<Vec<_>>();
        assert_eq!(
            cases,
            [
                ("pooled-1".into(), "pooled", 1),
                ("pooled-2".into(), "pooled", 2),
                ("pooled-4".into(), "pooled", 4),
                ("pooled-8".into(), "pooled", 8),
                ("fresh".into(), "fresh", 8)
            ]
        );
    }

    #[test]
    fn default_and_fresh_only_selection() {
        let options = parse_arguments(["--server", "/unused/pocket-ic"].map(Into::into)).unwrap();
        assert_eq!(options.rss_limits, super::RSS_COMMAND_LIMITS);
        assert_eq!(
            options
                .benchmark_cases()
                .iter()
                .map(|case| case.label.as_str())
                .collect::<Vec<_>>(),
            ["fresh", "pooled-1", "pooled-2"]
        );
        let options = parse_arguments(
            [
                "--server",
                "/unused/pocket-ic",
                "--modes",
                "fresh",
                "--capacities",
                "8",
            ]
            .map(Into::into),
        )
        .unwrap();
        assert_eq!(options.benchmark_cases().len(), 1);
        assert_eq!(options.benchmark_cases()[0].capacity.get(), 2);
    }

    #[test]
    fn rss_limits_preserve_explicit_selection_without_a_deadline() {
        let options = parse_arguments(
            [
                "--server",
                "/unused/pocket-ic",
                "--rss-stdout-bytes",
                "1024",
                "--rss-stderr-bytes",
                "128",
            ]
            .map(Into::into),
        )
        .unwrap();
        assert_eq!(
            options.rss_limits.stdout,
            ic_host_process::tool::OutputLimit::Terminate(1024)
        );
        assert_eq!(
            options.rss_limits.stderr,
            ic_host_process::tool::OutputLimit::Terminate(128)
        );
        assert_eq!(options.rss_limits.timeout, None);
    }

    #[test]
    fn invalid_selections_fail_before_launching_tools() {
        for selection in [
            vec!["--capacities", "0"],
            vec!["--capacities", "-1"],
            vec!["--capacities", "2", "2"],
            vec!["--modes", "pooled", "pooled"],
            vec!["--workers", "101"],
            vec!["--state-bytes", "4294967296"],
            vec!["--profile", "custom"],
            vec!["--rss-stdout-bytes", "0"],
            vec!["--rss-stderr-bytes", "18446744073709551616"],
            vec!["--modes"],
            vec!["--unknown"],
        ] {
            let args = ["--server", "/unused/pocket-ic"]
                .into_iter()
                .chain(selection)
                .map(Into::into);
            let error = parse_arguments(args).err().expect("invalid selection");
            assert_eq!(error.exit_code(), 2);
        }
        assert!(
            parse_arguments(["--server", "/unused", "--state-bytes", "0"].map(Into::into)).is_ok()
        );
    }
}
