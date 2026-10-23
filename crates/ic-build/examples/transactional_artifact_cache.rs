//! Caller-selected, verified external transform composed with a retained cache transaction.

use clap::{Arg, Command, value_parser};
use ic_build::artifacts::{
    ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCacheSpec, CacheRetentionLimits,
    prepare_artifact_cache,
};
use std::{path::PathBuf, time::Duration};
use {
    ic_host_artifacts::artifact::Sha256Digest,
    ic_host_process::tool::{
        AdmittedTool, ExecutionContext, OutputLimit, OutputLimits, VersionSpec, resolve_executable,
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = Command::new("transactional_artifact_cache")
        .about("Run an admitted external transform with a retained artifact cache")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("tool-path")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("sha256")
                .required(true)
                .value_parser(value_parser!(Sha256Digest)),
        )
        .arg(Arg::new("expected-version").required(true))
        .args(["input", "output", "cache-root"].map(|name| {
            Arg::new(name)
                .required(true)
                .value_parser(value_parser!(PathBuf))
        }))
        .get_matches();
    let tool = arguments.get_one::<PathBuf>("tool-path").unwrap();
    let sha256 = *arguments.get_one::<Sha256Digest>("sha256").unwrap();
    let version = arguments.get_one::<String>("expected-version").unwrap();
    let input = arguments.get_one::<PathBuf>("input").unwrap();
    let output = arguments.get_one::<PathBuf>("output").unwrap();
    let cache_root = arguments.get_one::<PathBuf>("cache-root").unwrap();
    let current_dir = std::env::current_dir()?;
    let tool = resolve_executable(&current_dir.join(tool), &current_dir, &[])?;
    let context = ExecutionContext {
        current_dir: &current_dir,
        environment: &[],
    };
    let limits = OutputLimits {
        stdout: OutputLimit::Terminate(64 * 1024),
        stderr: OutputLimit::Terminate(64 * 1024),
        timeout: Some(Duration::from_secs(60)),
    };
    // Admission uses caller authority, never a digest computed from untrusted
    // selected bytes. Transform tools must work with this empty environment.
    let admitted = AdmittedTool::admit(
        &VersionSpec {
            executable: &tool,
            executable_bytes: 256 * 1024 * 1024,
            version_arguments: &["--version".into()],
        },
        sha256,
        version,
        &context,
        limits,
    )?;
    let input = current_dir.join(input);
    let output = current_dir.join(output);
    let cache_root = current_dir.join(cache_root);
    let spec = ArtifactCacheSpec::new(
        &cache_root,
        "example-transform",
        "example/admitted-transform/v1",
    )
    .with_input("source", &input)
    .with_tool("transformer", &tool)
    .with_arguments(["<input>", "<output>"])
    .with_output("result", &output)
    .with_prune_policy(
        CacheRetentionLimits::new()
            .with_max_age(Duration::from_hours(7 * 24))
            .with_max_size_bytes(2 * 1024 * 1024 * 1024),
    );

    let outcome = match prepare_artifact_cache(&spec)? {
        ArtifactCachePreparation::Reused(record) => ArtifactCacheOutcome::Reused(record),
        ArtifactCachePreparation::Build(transaction) => {
            let staged_output = transaction.output_path("result")?;
            admitted
                .run(
                    &[input.into_os_string(), staged_output.into_os_string()],
                    &context,
                    limits,
                )?
                .require_success()?;
            transaction.commit()?
        }
    };

    let disposition = if outcome.is_reused() {
        "reused"
    } else {
        "built"
    };
    println!("{disposition} artifact set {}", outcome.record().key());
    Ok(())
}
