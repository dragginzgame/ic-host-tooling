use std::{
    ffi::OsString,
    path::PathBuf,
    process::{Command, ExitCode},
    sync::atomic::{AtomicI32, Ordering},
    time::Duration,
};

use clap::{
    Arg, Command as CliCommand, builder::OsStringValueParser, error::ErrorKind, value_parser,
};

use ic_host_process::{
    signal::{SignalActionError, SignalActions},
    status::shell_status_code,
};
use ic_testkit::pic::{PocketIcStartupConfig, server_assets};

static INTERRUPTED: AtomicI32 = AtomicI32::new(0);

enum Arguments {
    Setup(PathBuf),
    Check(PathBuf),
    Run(Box<RunArguments>),
}

struct RunArguments {
    directory: Option<PathBuf>,
    timeout: Duration,
    ttl: Option<Duration>,
    idle_ttl: Option<Duration>,
    output_files: Option<(PathBuf, PathBuf)>,
    command: Command,
}

extern "C" fn interrupted(signal: libc::c_int) {
    INTERRUPTED.store(signal, Ordering::Relaxed);
}

#[expect(
    unsafe_code,
    reason = "bounded native operation with the local SAFETY contract below"
)]
fn install_signals() -> Result<SignalActions, SignalActionError> {
    // SAFETY: this standalone CLI exclusively owns these dispositions through
    // guard drop. The static handler only stores a lock-free atomic value.
    unsafe { SignalActions::install(&[libc::SIGINT, libc::SIGTERM, libc::SIGHUP], interrupted) }
}

fn cli() -> CliCommand {
    let directory = Arg::new("directory")
        .long("directory")
        .value_name("DIRECTORY")
        .value_parser(value_parser!(PathBuf))
        .allow_hyphen_values(true);
    CliCommand::new("ic-testkit-server")
        .about("Explicit PocketIC setup, offline admission and command-scoped ownership")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands(["setup", "check"].map(|name| {
            CliCommand::new(name).arg(
                directory
                    .clone()
                    .default_value(server_assets::DEFAULT_DIRECTORY),
            )
        }))
        .subcommand(
            CliCommand::new("run")
                .arg(directory)
                .args(["ttl", "idle-ttl", "startup-timeout"].map(|name| {
                    let arg = Arg::new(name)
                        .long(name)
                        .value_name("SECONDS")
                        .value_parser(value_parser!(u64).range(1..));
                    if name == "startup-timeout" {
                        arg.default_value("30")
                    } else {
                        arg
                    }
                }))
                .arg(
                    Arg::new("server-stdout")
                        .long("server-stdout")
                        .value_name("NEW-FILE")
                        .value_parser(value_parser!(PathBuf))
                        .allow_hyphen_values(true)
                        .requires("server-stderr"),
                )
                .arg(
                    Arg::new("server-stderr")
                        .long("server-stderr")
                        .value_name("NEW-FILE")
                        .value_parser(value_parser!(PathBuf))
                        .allow_hyphen_values(true)
                        .requires("server-stdout"),
                )
                .arg(
                    Arg::new("command")
                        .last(true)
                        .required(true)
                        .num_args(1..)
                        .value_name("COMMAND")
                        .value_parser(OsStringValueParser::new()),
                ),
        )
}

fn parse(arguments: impl Iterator<Item = OsString>) -> Result<Arguments, clap::Error> {
    let mut cli = cli().no_binary_name(true);
    let mut matches = cli.try_get_matches_from_mut(arguments)?;
    let (name, mut matches) = matches.remove_subcommand().expect("subcommand required");
    let directory = matches.remove_one::<PathBuf>("directory");
    match name.as_str() {
        "setup" => Ok(Arguments::Setup(directory.expect("default directory"))),
        "check" => Ok(Arguments::Check(directory.expect("default directory"))),
        "run" => {
            let mut values = matches
                .remove_many::<OsString>("command")
                .expect("command required");
            let executable = values.next().expect("command has at least one value");
            // Clap captures one variadic argument so empty child arguments survive.
            // Only the executable, rather than every child argument, must be nonempty.
            if executable.is_empty() {
                return Err(cli.error(
                    ErrorKind::ValueValidation,
                    "command executable must not be empty",
                ));
            }
            let mut command = Command::new(executable);
            command.args(values);
            let stdout = matches.remove_one::<PathBuf>("server-stdout");
            let stderr = matches.remove_one::<PathBuf>("server-stderr");
            Ok(Arguments::Run(Box::new(RunArguments {
                directory,
                timeout: Duration::from_secs(
                    matches
                        .remove_one::<u64>("startup-timeout")
                        .expect("default timeout"),
                ),
                ttl: matches.remove_one::<u64>("ttl").map(Duration::from_secs),
                idle_ttl: matches
                    .remove_one::<u64>("idle-ttl")
                    .map(Duration::from_secs),
                output_files: stdout.zip(stderr),
                command,
            })))
        }
        _ => unreachable!("Clap admits only declared subcommands"),
    }
}

pub fn main() -> ExitCode {
    let arguments = match parse(std::env::args_os().skip(1)) {
        Ok(arguments) => arguments,
        Err(error) => {
            let _ = error.print();
            return ExitCode::from(u8::try_from(error.exit_code()).unwrap_or(2));
        }
    };
    let arguments = match arguments {
        Arguments::Setup(directory) => {
            return match install_signals() {
                Ok(_signals) => report_provisioning(server_assets::setup(&directory, || {
                    INTERRUPTED.load(Ordering::Relaxed) != 0
                })),
                Err(error) => report_provisioning(Err(error.into())),
            };
        }
        Arguments::Check(directory) => {
            return report_provisioning(server_assets::check(&directory));
        }
        Arguments::Run(arguments) => arguments,
    };
    let RunArguments {
        directory,
        timeout,
        ttl,
        idle_ttl,
        output_files,
        mut command,
    } = *arguments;
    let result = (|| {
        let _signals = install_signals()?;
        let environment_selected = std::env::var_os("IC_TESTKIT_POCKET_IC_URL").is_some()
            || std::env::var_os("POCKET_IC_BIN").is_some();
        let mut config = if directory.is_some() || !environment_selected {
            if environment_selected {
                return Err(
                    "--directory conflicts with an explicit server URL or POCKET_IC_BIN".into(),
                );
            }
            let binary = server_assets::check(
                &directory.unwrap_or_else(|| PathBuf::from(server_assets::DEFAULT_DIRECTORY)),
            )?;
            command.env("POCKET_IC_BIN", &binary);
            PocketIcStartupConfig::Spawn {
                server_binary: binary,
                timeout,
                server_hard_ttl: None,
                server_idle_ttl: None,
                server_output_files: None,
            }
        } else {
            PocketIcStartupConfig::from_env(timeout)?
        };
        match &mut config {
            PocketIcStartupConfig::Spawn {
                server_hard_ttl,
                server_idle_ttl,
                server_output_files,
                ..
            } => {
                *server_hard_ttl = ttl;
                *server_idle_ttl = idle_ttl;
                *server_output_files = output_files;
            }
            PocketIcStartupConfig::Connect { .. } => {
                if ttl.is_some() || idle_ttl.is_some() || output_files.is_some() {
                    return Err("--ttl, --idle-ttl and output files require an owned server".into());
                }
            }
        }
        Ok::<_, Box<dyn std::error::Error>>(
            config.run_command(&mut command, || INTERRUPTED.load(Ordering::Relaxed) != 0)?,
        )
    })();
    if let Err(error) = &result {
        eprintln!("{error}");
    }
    if let Some(status) = interruption_exit_code() {
        return status;
    }
    match result {
        Ok(status) => command_exit_code(status),
        Err(error) => error
            .downcast_ref::<ic_testkit::pic::PocketIcStartupError>()
            .map_or(ExitCode::FAILURE, |startup| {
                failure_exit_code(startup.failure())
            }),
    }
}

fn interruption_exit_code() -> Option<ExitCode> {
    let signal = INTERRUPTED.load(Ordering::Relaxed);
    (signal != 0).then(|| ExitCode::from(u8::try_from(128 + signal).unwrap_or(1)))
}

fn report_provisioning(result: Result<PathBuf, Box<dyn std::error::Error>>) -> ExitCode {
    use std::io::Write as _;
    use std::os::unix::ffi::OsStrExt as _;
    if let Err(error) = &result {
        eprintln!("{error}");
    }
    if let Some(status) = interruption_exit_code() {
        return status;
    }
    result.map_or(ExitCode::FAILURE, |path| {
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        if stdout
            .write_all(path.as_os_str().as_bytes())
            .and_then(|()| stdout.write_all(b"\n"))
            .is_err()
        {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    })
}

fn command_exit_code(status: std::process::ExitStatus) -> ExitCode {
    ExitCode::from(u8::try_from(shell_status_code(status).unwrap_or(129)).unwrap_or(1))
}

fn failure_exit_code(failure: &ic_testkit::pic::PocketIcStartupFailure) -> ExitCode {
    if let ic_testkit::pic::PocketIcStartupFailure::ServerCleanup { command_status } = failure
        && !command_status.success()
    {
        command_exit_code(*command_status)
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::{ExitCode, command_exit_code, failure_exit_code};
    use ic_testkit::pic::PocketIcStartupFailure;
    use std::os::unix::process::ExitStatusExt as _;

    #[test]
    fn command_arguments_preserve_native_paths_empty_values_and_limits() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt as _, time::Duration};
        let executable = OsString::from_vec(vec![b'p', 255]);
        let child_argument = OsString::from_vec(vec![b'a', 254]);
        let mut arguments = [
            "run",
            "--ttl",
            "90",
            "--idle-ttl",
            "10",
            "--startup-timeout",
            "7",
            "--server-stdout",
            "stdout path",
            "--server-stderr",
            "stderr path",
            "--",
        ]
        .map(OsString::from)
        .to_vec();
        arguments.extend([
            executable.clone(),
            "--help".into(),
            "".into(),
            child_argument.clone(),
        ]);
        let super::Arguments::Run(arguments) = super::parse(arguments.into_iter()).unwrap() else {
            panic!("run arguments required");
        };
        assert_eq!(arguments.command.get_program(), &executable);
        assert_eq!(
            arguments.command.get_args().collect::<Vec<_>>(),
            [
                std::ffi::OsStr::new("--help"),
                std::ffi::OsStr::new(""),
                child_argument.as_os_str()
            ]
        );
        assert_eq!(arguments.timeout, Duration::from_secs(7));
        assert_eq!(arguments.ttl, Some(Duration::from_secs(90)));
        assert_eq!(arguments.idle_ttl, Some(Duration::from_secs(10)));
        assert_eq!(
            arguments.output_files,
            Some(("stdout path".into(), "stderr path".into()))
        );
    }

    #[test]
    fn malformed_server_arguments_refuse_before_server_admission() {
        for arguments in [
            vec!["run", "--", ""],
            vec!["run", "command"],
            vec!["run", "--ttl", "0", "--", "command"],
            vec!["run", "--ttl", "1", "--ttl", "2", "--", "command"],
            vec!["run", "--server-stdout", "stdout", "--", "command"],
            vec!["run", "--server-stderr", "stderr", "--", "command"],
            vec!["check", "--directory", ""],
        ] {
            let error = super::parse(arguments.into_iter().map(Into::into))
                .err()
                .expect("invalid arguments");
            assert_eq!(error.exit_code(), 2);
        }
    }

    #[test]
    fn server_help_and_defaults_are_admitted_without_provisioning() {
        for arguments in [vec!["--help"], vec!["--version"], vec!["run", "--help"]] {
            let error = super::parse(arguments.into_iter().map(Into::into))
                .err()
                .expect("display request");
            assert_eq!(error.exit_code(), 0);
        }
        let super::Arguments::Check(directory) = super::parse(
            ["check", "--directory", "-store"]
                .map(Into::into)
                .into_iter(),
        )
        .unwrap() else {
            panic!("check arguments required");
        };
        assert_eq!(directory, std::path::Path::new("-store"));
        let super::Arguments::Check(directory) =
            super::parse(["check"].map(Into::into).into_iter()).unwrap()
        else {
            panic!("check arguments required");
        };
        assert_eq!(
            directory,
            std::path::Path::new(super::server_assets::DEFAULT_DIRECTORY)
        );
        let super::Arguments::Run(arguments) =
            super::parse(["run", "--", "command"].map(Into::into).into_iter()).unwrap()
        else {
            panic!("run arguments required");
        };
        assert_eq!(arguments.timeout, std::time::Duration::from_secs(30));
        assert!(
            arguments.directory.is_none()
                && arguments.ttl.is_none()
                && arguments.idle_ttl.is_none()
                && arguments.output_files.is_none()
        );
    }

    #[test]
    fn secondary_server_cleanup_preserves_failed_command_status() {
        for (raw_status, expected) in [
            (0, 1),
            (23 << 8, 23),
            (255 << 8, 255),
            (15, 143),
            ((19 << 8) | 0x7f, 129),
        ] {
            assert_eq!(
                command_exit_code(std::process::ExitStatus::from_raw(raw_status)),
                ExitCode::from(if raw_status == 0 { 0 } else { expected })
            );
            let failure = PocketIcStartupFailure::ServerCleanup {
                command_status: std::process::ExitStatus::from_raw(raw_status),
            };
            assert_eq!(failure_exit_code(&failure), ExitCode::from(expected));
        }
        assert_eq!(
            failure_exit_code(&PocketIcStartupFailure::BuilderDisconnected),
            ExitCode::FAILURE
        );
    }
}
