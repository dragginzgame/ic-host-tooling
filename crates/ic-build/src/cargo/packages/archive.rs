//! Unix archive qualification over Host descriptor-based traversal.

use super::external_lock::admit_lock;
use super::source_inventory::sources;
use crate::cargo::observation::schema::{Metadata, Package};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_process::tool::{CompletedExecution, capture_command};
use ic_host_process::tool::{OutputLimits, ToolError};
#[cfg(test)]
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    path::Path,
    process::Command,
};
use std::{error::Error, fmt, io, path::PathBuf};

pub(super) type Cause = Box<dyn Error>;

/// One consumer-selected package and its expected archive identity.
pub struct PackageArchiveSpec<'a> {
    /// Cargo package name.
    pub name: &'a str,
    /// Exact expected Cargo version; coordinated versions remain caller policy.
    pub version: &'a str,
    /// Patch registry references to this extracted package during qualification only.
    pub patch_registry: bool,
}

/// Caller-selected bounds for process output and source admission.
#[derive(Clone, Copy)]
pub struct PackageQualificationLimits {
    /// Cargo/extractor output and optional per-command elapsed-time limit.
    pub process: OutputLimits,
    /// Maximum bytes in each admitted manifest, lock, archive or source file.
    pub file_bytes: usize,
    /// Maximum total directory and regular-file entries across extracted packages.
    pub source_entries: usize,
    /// Maximum directory depth beneath each extracted package root.
    pub source_depth: usize,
}

/// Command failure preserves execution separately from diagnostic-retention failures.
#[derive(Debug)]
pub struct PackageCommandError {
    /// Original process/capture failure, or `None` if only log retention failed.
    pub execution: Option<ToolError>,
    /// Independent failures writing the retained stdout/stderr records.
    pub retention: Vec<io::Error>,
}
impl fmt::Display for PackageCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(error) = &self.execution {
            error.fmt(f)
        } else {
            f.write_str("package command diagnostic retention failed")
        }
    }
}
impl Error for PackageCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.execution
            .as_ref()
            .map(|error| error as &dyn Error)
            .or_else(|| self.retention.first().map(|error| error as &dyn Error))
    }
}

/// Failed qualification retains its original cause and evidence directory.
#[derive(Debug)]
pub struct PackageQualificationError {
    /// The caller-selected directory, retained even on failure.
    pub evidence: PathBuf,
    /// Original I/O, encoding, process or admission failure.
    pub cause: Cause,
}
impl fmt::Display for PackageQualificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cause.fmt(f)
    }
}
impl Error for PackageQualificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.cause.as_ref())
    }
}

/// Package a selection and test its exact extracted graph with frozen external selections.
///
/// Available on Unix; the pure [`super::verify_external_lock_subset`] remains portable.
///
/// `evidence` must be an existing empty private directory with trusted ancestors.
/// It and all archives/logs are retained on success or failure. `archive_root`
/// must match the selected Cargo packaging target. Cargo and extraction commands
/// are supplied by the caller, including their environment and executable custody.
/// The extractor receives a Cargo-produced archive and the evidence destination;
/// it must safely extract that archive without selecting other inputs or effects.
/// This API does not admit untrusted third-party tar files or sandbox tools.
/// Exclude concurrent writers to original inputs, archives and extracted sources.
///
/// Each attempt uses its own `evidence/target`: fixed archive mtimes must never
/// permit another same-version payload's artifacts to satisfy this verification.
/// Only the temporary verification workspace receives path patches. Original
/// manifests/lock, archive bytes, extracted source bytes/modes and admitted
/// verification manifest/lock are compared after all-target and documentation tests.
/// Commands execute once, offline; resolution may update only the verification lock
/// before admission. Every external package/checksum must already be frozen in the
/// original lock. No install, publication, source mutation, cleanup or retry occurs.
///
/// # Errors
/// Retains exact process failure (including status/diagnostics), missing inputs,
/// bounded decoding, graph/lock refusals and final input drift in the evidence root.
pub fn qualify_package_archives(
    workspace: &Path,
    archive_root: &Path,
    evidence: &Path,
    packages: &[PackageArchiveSpec<'_>],
    limits: PackageQualificationLimits,
    mut cargo: impl FnMut() -> Command,
    mut extractor: impl FnMut(&Path, &Path) -> Command,
) -> Result<(), PackageQualificationError> {
    qualify(
        workspace,
        archive_root,
        evidence,
        packages,
        limits,
        &mut cargo,
        &mut extractor,
    )
    .map_err(|cause| PackageQualificationError {
        evidence: evidence.to_owned(),
        cause,
    })
}

#[expect(
    clippy::too_many_lines,
    reason = "ordered archive qualification retains one before/after custody boundary"
)]
fn qualify(
    workspace: &Path,
    archive_root: &Path,
    evidence: &Path,
    packages: &[PackageArchiveSpec<'_>],
    limits: PackageQualificationLimits,
    cargo: &mut impl FnMut() -> Command,
    extractor: &mut impl FnMut(&Path, &Path) -> Command,
) -> Result<(), Cause> {
    if packages.is_empty() || limits.file_bytes == 0 || limits.source_entries == 0 {
        return Err("empty package selection or admission budget".into());
    }
    let mut names = BTreeSet::new();
    for package in packages {
        if !identifier(package.name) || !identifier(package.version) || !names.insert(package.name)
        {
            return Err("invalid or duplicate package archive identity".into());
        }
    }
    let workspace = workspace.canonicalize()?;
    let archive_root = archive_root.canonicalize()?;
    let evidence = evidence.canonicalize()?;
    for path in [&workspace, &archive_root, &evidence] {
        if path
            .as_os_str()
            .as_encoded_bytes()
            .iter()
            .any(|b| matches!(b, b'\r' | b'\n'))
        {
            return Err("invalid directory namespace".into());
        }
    }
    if fs::read_dir(&evidence)?.next().is_some() {
        return Err("evidence directory is not empty".into());
    }
    let lock = workspace.join("Cargo.lock");
    let entry_lock = read(&lock, limits.file_bytes)?;
    fs::write(evidence.join("entry-Cargo.lock"), &entry_lock)?;
    let mut command = cargo();
    command.current_dir(&workspace).args([
        "metadata",
        "--offline",
        "--locked",
        "--no-deps",
        "--format-version",
        "1",
    ]);
    let metadata = execute(&mut command, &evidence, "entry-metadata", limits.process)?;
    let metadata = serde_json::from_slice(&metadata)?;
    let mut originals = BTreeMap::new();
    originals.insert(
        workspace.join("Cargo.toml"),
        read(&workspace.join("Cargo.toml"), limits.file_bytes)?,
    );
    for spec in packages {
        let selected = package(&metadata, spec.name)?;
        if selected.version.to_string() != spec.version {
            return Err("entry package version differs from selection".into());
        }
        let path = selected.manifest_path.clone().into_std_path_buf();
        originals.insert(path.clone(), read(&path, limits.file_bytes)?);
    }
    for (index, bytes) in originals.values().enumerate() {
        fs::write(evidence.join(format!("entry-manifest-{index}")), bytes)?;
    }
    let mut command = cargo();
    command.current_dir(&workspace).args([
        "package",
        "--offline",
        "--locked",
        "--allow-dirty",
        "--no-verify",
    ]);
    for spec in packages {
        command.args(["-p", spec.name]);
    }
    execute(&mut command, &evidence, "package", limits.process)?;
    let mut archives = BTreeMap::new();
    for (index, spec) in packages.iter().enumerate() {
        let path = archive_root.join(format!("{}-{}.crate", spec.name, spec.version));
        archives.insert(path.clone(), hash(&path, limits.file_bytes)?);
        execute(
            &mut extractor(&path, &evidence),
            &evidence,
            &format!("extract-{index}"),
            limits.process,
        )?;
    }
    let mut archive_hashes = String::new();
    for digest in archives.values() {
        writeln!(archive_hashes, "{digest}")?;
    }
    fs::write(evidence.join("archive.sha256"), archive_hashes)?;
    let sources = sources(&evidence, packages, limits)?;
    fs::write(
        evidence.join("entry-sources.json"),
        serde_json::to_vec(&sources)?,
    )?;
    let manifest = verification_manifest(packages)?;
    let manifest_path = evidence.join("Cargo.toml");
    let payload_lock = evidence.join("Cargo.lock");
    fs::write(&manifest_path, &manifest)?;
    fs::write(&payload_lock, &entry_lock)?;
    let mut command = cargo();
    command
        .current_dir(&evidence)
        .env("CARGO_TARGET_DIR", evidence.join("target"))
        .args([
            "metadata",
            "--offline",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(&manifest_path);
    let metadata = execute(&mut command, &evidence, "metadata", limits.process)?;
    admit_graph(&serde_json::from_slice(&metadata)?, &evidence, packages)?;
    let admitted_lock = read(&payload_lock, limits.file_bytes)?;
    admit_lock(&entry_lock, &admitted_lock)?;
    fs::write(evidence.join("admitted-Cargo.toml"), &manifest)?;
    fs::write(evidence.join("admitted-Cargo.lock"), &admitted_lock)?;
    for (label, selection) in [("test-targets", "--all-targets"), ("test-docs", "--doc")] {
        let mut command = cargo();
        command
            .current_dir(&evidence)
            .env("CARGO_TARGET_DIR", evidence.join("target"))
            .args(["test", "--offline", "--locked", "--manifest-path"])
            .arg(&manifest_path)
            .args(["--workspace", selection]);
        execute(&mut command, &evidence, label, limits.process)?;
    }
    if read(&lock, limits.file_bytes)? != entry_lock {
        return Err("original lock changed".into());
    }
    for (path, bytes) in originals {
        if read(&path, limits.file_bytes)? != bytes {
            return Err("original manifest changed".into());
        }
    }
    for (path, digest) in archives {
        if hash(&path, limits.file_bytes)? != digest {
            return Err("package archive changed".into());
        }
    }
    let final_sources = self::sources(&evidence, packages, limits)?;
    fs::write(
        evidence.join("final-sources.json"),
        serde_json::to_vec(&final_sources)?,
    )?;
    if final_sources != sources {
        return Err("extracted package sources changed".into());
    }
    if read(&manifest_path, limits.file_bytes)? != manifest.as_bytes()
        || read(&payload_lock, limits.file_bytes)? != admitted_lock
    {
        return Err("admitted verification manifest or lock changed".into());
    }
    Ok(())
}

fn verification_manifest(packages: &[PackageArchiveSpec<'_>]) -> Result<String, toml::ser::Error> {
    let members = packages
        .iter()
        .map(|p| format!("{}-{}", p.name, p.version))
        .collect::<Vec<_>>();
    let patches: toml::Table = packages
        .iter()
        .filter(|p| p.patch_registry)
        .map(|p| {
            let entry = toml::toml! { path = (format!("{}-{}", p.name, p.version)) };
            (p.name.to_owned(), toml::Value::Table(entry))
        })
        .collect();
    toml::to_string(&toml::toml! {
        [workspace]
        resolver = "3"
        members = (members)
        [patch]
        crates-io = (patches)
    })
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'+'))
}

fn read(path: &Path, limit: usize) -> Result<Vec<u8>, Cause> {
    Ok(ic_host_fs::read::read_file_no_follow(path, limit)?)
}
fn hash(path: &Path, limit: usize) -> Result<Sha256Digest, Cause> {
    Ok(ic_host_fs::read::hash_file_no_follow(path, u64::try_from(limit)?)?.sha256)
}

fn execute(
    command: &mut Command,
    evidence: &Path,
    label: &str,
    limits: OutputLimits,
) -> Result<Vec<u8>, Cause> {
    let result = capture_command(command, limits)
        .and_then(CompletedExecution::require_success)
        .and_then(CompletedExecution::require_complete);
    let (stdout, stderr) = match &result {
        Ok(output) => (&output.stdout, &output.stderr),
        Err(ToolError::Execution(error)) => (&error.evidence.stdout, &error.evidence.stderr),
        Err(_) => (&Vec::new(), &Vec::new()),
    };
    let retention = [
        fs::write(evidence.join(format!("{label}.json")), stdout),
        fs::write(evidence.join(format!("{label}.stderr")), stderr),
    ]
    .into_iter()
    .filter_map(Result::err)
    .collect::<Vec<_>>();
    match result {
        Ok(output) if retention.is_empty() => Ok(output.stdout),
        other => Err(Box::new(PackageCommandError {
            execution: other.err(),
            retention,
        })),
    }
}

fn package<'a>(metadata: &'a Metadata, name: &str) -> Result<&'a Package, Cause> {
    let mut matching = metadata.packages.iter().filter(|p| p.name == name);
    match (matching.next(), matching.next()) {
        (Some(package), None) => Ok(package),
        _ => Err(format!("expected one extracted package: {name}").into()),
    }
}

fn admit_graph(
    metadata: &Metadata,
    root: &Path,
    packages: &[PackageArchiveSpec<'_>],
) -> Result<(), Cause> {
    let members = &metadata.workspace_members;
    let mut selected = BTreeSet::new();
    for spec in packages {
        let package = package(metadata, spec.name)?;
        let manifest = root
            .join(format!("{}-{}/Cargo.toml", spec.name, spec.version))
            .canonicalize()?;
        if package.version.to_string() != spec.version
            || package.source.is_some()
            || package.manifest_path.as_std_path() != manifest
            || !members.contains(&package.id)
        {
            return Err(format!(
                "resolved package is not the exact extracted payload: {}",
                spec.name
            )
            .into());
        }
        selected.insert(&package.id);
    }
    if members.len() != packages.len() || selected.len() != members.len() {
        return Err("unexpected extracted workspace member".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
