use super::{communicate_cargo, schema};
use crate::{artifacts::CargoBuildFacts, cargo::selection::CargoSelection};
use ic_host_process::{
    child::{CleanupPolicy, OwnedChild},
    tool::{
        CompletedExecution, OutputLimit, OutputLimits, OutputStream, ToolError,
        capture_group_command_with_cancellation,
    },
};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    io,
    path::PathBuf,
    process::{Command, Stdio},
};

pub const TOOL_IDENTITY_BYTES: usize = 64 * 1024;

/// Supported Cargo recipes. Consumers retain profile overrides and compiler flags.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CargoRecipe {
    /// Build the selected packages' libraries.
    Library,
    /// Build one named example.
    Example(String),
    /// Compile a library with explicit arguments following Cargo's `--` separator.
    RustcLibrary(Vec<OsString>),
}

/// One concrete Cargo invocation with an explicit, complete child environment.
///
/// Graph arguments are rendered from `selection`, never reconstructed from argv.
/// Configurations and compiler arguments are caller-owned recipe policy. These
/// observations do not validate source custody or admit a release. Debug is not
/// implemented because environment/configuration values may contain secrets.
#[derive(Clone, Eq, PartialEq)]
pub struct CargoInvocation {
    /// Selected Cargo executable.
    pub cargo_program: OsString,
    /// Absolute Cargo working directory, also used by identity probes.
    pub working_directory: PathBuf,
    /// Complete child environment. No parent variables are inherited.
    /// Observed execution requires an explicit nonempty `RUSTC` entry, so Cargo
    /// cannot select a different compiler through unobserved configuration.
    pub environment: BTreeMap<OsString, OsString>,
    /// Compilation target, rendered into the command and retained in facts.
    pub target: String,
    /// Profile, features and network/lock policy, rendered exactly once.
    pub selection: CargoSelection,
    /// Package selectors; an empty list leaves Cargo's workspace selection intact.
    pub packages: Vec<String>,
    /// Optional explicit manifest.
    pub manifest_path: Option<PathBuf>,
    /// Selected library/example/compiler recipe.
    pub recipe: CargoRecipe,
    /// Explicit Cargo `--config` values, retaining their order.
    pub configurations: Vec<OsString>,
    /// Request Cargo's `json-render-diagnostics` instead of `json` messages.
    pub render_diagnostics: bool,
    /// Continue independent compilation jobs after a build error.
    pub keep_going: bool,
}

#[derive(Clone, Copy)]
pub enum CargoTool {
    Cargo,
    Rustc,
}

impl CargoInvocation {
    /// Render the selected invocation without spawning a process.
    ///
    /// This leaves stdio and process custody unconfigured. `execute_cargo` sets
    /// null stdin and piped output before invoking the caller's spawner, which
    /// may override stdio and custody. Changing executable, cwd, arguments or
    /// environment invalidates execution facts. Separate execution earns no receipt.
    ///
    /// # Errors
    /// Refuses relative working directories and invalid profile components.
    pub fn command(&self) -> io::Result<Command> {
        if !self.working_directory.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cargo invocation requires an absolute working directory",
            ));
        }
        self.selection
            .profile_target_dir()
            .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?;
        let mut command = Command::new(&self.cargo_program);
        command
            .current_dir(&self.working_directory)
            .env_clear()
            .envs(&self.environment);
        command.arg(if matches!(self.recipe, CargoRecipe::RustcLibrary(_)) {
            "rustc"
        } else {
            "build"
        });
        command.arg(if self.render_diagnostics {
            "--message-format=json-render-diagnostics"
        } else {
            "--message-format=json"
        });
        match &self.recipe {
            CargoRecipe::Example(name) => {
                command.args(["--example", name]);
            }
            _ => {
                command.arg("--lib");
            }
        }
        command
            .args(["--target", &self.target])
            .args(self.selection.build_arguments());
        if self.keep_going {
            command.arg("--keep-going");
        }
        for package in &self.packages {
            command.args(["-p", package]);
        }
        if let Some(manifest) = &self.manifest_path {
            command.arg("--manifest-path").arg(manifest);
        }
        for configuration in &self.configurations {
            command.arg("--config").arg(configuration);
        }
        if let CargoRecipe::RustcLibrary(arguments) = &self.recipe {
            command.arg("--").args(arguments);
        }
        Ok(command)
    }

    pub(crate) fn observe<E>(
        &self,
        rustc_fallback: &OsStr,
        mut capture: impl FnMut(CargoTool, &mut Command) -> Result<CompletedExecution, E>,
    ) -> Result<CargoBuildFacts, E> {
        let cargo_identity = command_identity(
            &self.cargo_program,
            &self.working_directory,
            &self.environment,
            &["--version", "--verbose"],
            |command| capture(CargoTool::Cargo, command),
        )?;
        let rustc = self
            .environment
            .get(OsStr::new("RUSTC"))
            .map_or(rustc_fallback, OsString::as_os_str);
        let rustc_identity = command_identity(
            rustc,
            &self.working_directory,
            &self.environment,
            &["-vV"],
            |command| capture(CargoTool::Rustc, command),
        )?;
        Ok(self.facts(cargo_identity, rustc_identity))
    }

    pub(crate) fn facts(
        &self,
        cargo_identity: Vec<u8>,
        rustc_identity: Vec<u8>,
    ) -> CargoBuildFacts {
        CargoBuildFacts {
            selection: self.selection.clone(),
            target: self.target.clone(),
            environment: self.environment.clone(),
            cargo_identity,
            rustc_identity,
        }
    }
}

pub fn command_identity<E>(
    program: &OsStr,
    directory: &std::path::Path,
    environment: &BTreeMap<OsString, OsString>,
    arguments: &[&str],
    capture: impl FnOnce(&mut Command) -> Result<CompletedExecution, E>,
) -> Result<Vec<u8>, E> {
    let mut command = Command::new(program);
    command
        .current_dir(directory)
        .env_clear()
        .envs(environment)
        .args(arguments);
    let output = capture(&mut command)?;
    let mut identity = output.stdout;
    identity.extend_from_slice(&output.stderr);
    Ok(identity)
}

/// Caller-owned output, decoding and probe cleanup policy. No default deadline.
#[derive(Clone, Copy)]
pub struct CargoExecutionPolicy {
    /// Host retention/deadline policy for the build; raw chunks still reach observers.
    pub output: OutputLimits,
    /// Maximum bytes in one Cargo protocol record.
    pub message_bytes: usize,
    /// Group cleanup policy for bounded, cancellable identity probes.
    pub probe_cleanup: CleanupPolicy,
}

/// Independent process/protocol evidence and observations bound to this execution.
///
/// Inspect process failure before protocol refusal. `facts` is `Ok(Some(_))` only
/// after successful execution, successful message delivery and matching before/
/// after tool observations. `Ok(None)` retains an unsuccessful execution's original
/// evidence; `Err` retains an observation or invocation-binding refusal. This is
/// not source validation, artifact custody or release admission.
#[must_use = "inspect execution, messages and facts before admitting artifacts"]
pub struct CargoExecution {
    /// Original bounded Host execution result and cleanup evidence.
    pub execution: Result<CompletedExecution, ToolError>,
    /// Original decoder or message-sink refusal; output is drained after refusal.
    pub messages: io::Result<()>,
    /// Successful execution observations, or a post-execution observation refusal.
    pub facts: io::Result<Option<CargoBuildFacts>>,
}

/// Observe tools, execute one rendered invocation, drain/decode it and recheck tools.
///
/// Source/configuration mutation guards and artifact lifetime remain caller-owned.
/// `spawn` selects process/group custody and may override stdio. It must execute
/// the supplied command without changing its recipe. Cancellation is also polled
/// during identity probes; probes have complete 64 KiB output bounds and no finite
/// default deadline. All raw build output reaches `output`, including failures.
///
/// # Errors
/// Returns pre-execution validation/probe failures or the original spawn error.
/// Once spawned, process, protocol and revalidation evidence is returned together.
pub fn execute_cargo(
    invocation: &CargoInvocation,
    policy: CargoExecutionPolicy,
    mut spawn: impl FnMut(&mut Command) -> io::Result<OwnedChild>,
    mut cancelled: impl FnMut() -> bool + Send,
    output: impl FnMut(OutputStream, &[u8]) -> io::Result<()>,
    message: impl FnMut(schema::Message) -> io::Result<()>,
) -> io::Result<CargoExecution> {
    let mut command = invocation.command()?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if invocation
        .environment
        .get(OsStr::new("RUSTC"))
        .is_none_or(|program| program.is_empty())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "observed Cargo execution requires an explicit RUSTC environment entry",
        ));
    }
    let observe = |cancelled: &mut (dyn FnMut() -> bool + Send)| {
        invocation.observe(OsStr::new("rustc"), |_, probe| {
            capture_group_command_with_cancellation(
                probe,
                OutputLimits {
                    stdout: OutputLimit::Terminate(TOOL_IDENTITY_BYTES),
                    stderr: OutputLimit::Terminate(TOOL_IDENTITY_BYTES),
                    timeout: None,
                },
                policy.probe_cleanup,
                None,
                &mut *cancelled,
            )
            .and_then(CompletedExecution::require_success)
            .and_then(CompletedExecution::require_complete)
            .map_err(io::Error::other)
        })
    };
    let before = observe(&mut cancelled)?;
    let mut child = spawn(&mut command)?;
    let expected = invocation.command()?;
    let unchanged = command.get_program() == expected.get_program()
        && command.get_current_dir() == expected.get_current_dir()
        && command.get_args().eq(expected.get_args())
        && command.get_envs().eq(expected.get_envs());
    let observed = communicate_cargo(
        &mut child,
        policy.output,
        policy.message_bytes,
        &mut cancelled,
        output,
        message,
    );
    let facts = if observed
        .execution
        .as_ref()
        .is_ok_and(|execution| execution.status.success())
        && observed.messages.is_ok()
    {
        if unchanged {
            observe(&mut cancelled).and_then(|after| {
                if before == after {
                    Ok(Some(before))
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Cargo/compiler identity changed during compilation",
                    ))
                }
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Cargo spawner changed the selected invocation",
            ))
        }
    } else {
        Ok(None)
    };
    Ok(CargoExecution {
        execution: observed.execution,
        messages: observed.messages,
        facts,
    })
}
