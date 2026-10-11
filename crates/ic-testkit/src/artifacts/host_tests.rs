use super::test_support::{fake_wasm_build_spec, unique_temp_directory, write_executable_script};
use super::{ArtifactCachePreparation, ArtifactCacheSpec, prepare_artifact_cache};
use crate::{
    ic_host_artifacts::artifact::{ArtifactError, Sha256Digest},
    ic_host_process::tool::{
        AdmittedTool, ExecutionContext, OutputLimit, OutputLimits, ToolError, ToolSpec,
        resolve_executable,
    },
};
use std::{ffi::OsString, fs, path::Path, time::Duration};

#[test]
fn admitted_shared_tool_output_is_published_and_invalidated_by_tool_bytes() {
    let root = unique_temp_directory("shared-admitted-tool")
        .canonicalize()
        .unwrap();
    let script = b"#!/bin/sh\nif [ \"$1\" = --version ]; then printf 'fixture-tool 1.0\\n'; else printf 'compiled output'; fi\n";
    let executable = root.join("producer");
    write_executable_script(&executable, script);
    let executable = resolve_executable(Path::new("./producer"), &root, &[]).unwrap();
    let context = ExecutionContext {
        current_dir: &root,
        environment: &[],
    };
    let limits = OutputLimits {
        stdout: OutputLimit::Terminate(128),
        stderr: OutputLimit::Terminate(128),
        timeout: Some(Duration::from_secs(5)),
    };
    let tool = AdmittedTool::admit(
        &ToolSpec {
            executable: &executable,
            sha256: Sha256Digest::compute(script),
            executable_bytes: script.len() as u64,
            version_arguments: &[OsString::from("--version")],
            version_identity: "fixture-tool 1.0",
        },
        &context,
        limits,
    )
    .unwrap();
    let destination = root.join("output");
    let spec = ArtifactCacheSpec::new(&root.join("cache"), "shared-tool", "compile/v1")
        .with_tool("producer", &executable)
        .with_output("compiled", &destination);
    let ArtifactCachePreparation::Build(transaction) = prepare_artifact_cache(&spec).unwrap()
    else {
        panic!("first acquisition must build");
    };
    let evidence = tool
        .run(&[OsString::from("compile")], &context, limits)
        .unwrap()
        .require_success()
        .unwrap();
    fs::write(
        transaction.output_path("compiled").unwrap(),
        &evidence.stdout,
    )
    .unwrap();
    let record = transaction.commit().unwrap();
    assert_eq!(fs::read(&destination).unwrap(), b"compiled output");
    let ArtifactCachePreparation::Reused(reused) = prepare_artifact_cache(&spec).unwrap() else {
        panic!("same admitted producer must reuse its output");
    };
    write_executable_script(&executable, b"#!/bin/sh\nexit 1\n");
    assert!(matches!(
        tool.run(&[OsString::from("compile")], &context, limits),
        Err(ToolError::Artifact(ArtifactError::DigestMismatch { .. }))
    ));
    assert!(matches!(
        prepare_artifact_cache(&spec).unwrap(),
        ArtifactCachePreparation::Build(_)
    ));
    drop(reused);
    drop(record);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn shim_wasm_and_post_link_caches_reuse_and_invalidate_independently() {
    use super::{
        ArtifactCacheOutcome, WasmBuildOutcome, build_wasm_canisters_cached,
        resolve_cargo_build_inputs,
    };
    use crate::ic_host_artifacts::artifact::hash_reader;

    let (root, spec) = fake_wasm_build_spec("post-link-composition");
    let root = root.canonicalize().unwrap();
    let compiler = root.join("selected-rustc");
    write_executable_script(&compiler, "#!/bin/sh\nprintf 'compiler-A\\n'\n");
    let spec = spec.with_rustc_program(compiler.as_os_str());
    let executable = root.join("optimizer");
    let script = b"#!/bin/sh\nif [ \"$1\" = --version ]; then printf 'optimizer 1\\n'; else /bin/cp \"$1\" \"$3\"; printf 'run\\n' >> \"$0.calls\"; fi\n";
    write_executable_script(&executable, script);
    let context = ExecutionContext {
        current_dir: &root,
        environment: &[],
    };
    let limits = OutputLimits {
        stdout: OutputLimit::Terminate(128),
        stderr: OutputLimit::Terminate(128),
        timeout: Some(Duration::from_secs(5)),
    };
    let admit = |bytes: &[u8]| {
        AdmittedTool::admit(
            &ToolSpec {
                executable: &executable,
                sha256: Sha256Digest::compute(bytes),
                executable_bytes: bytes.len() as u64,
                version_arguments: &[OsString::from("--version")],
                version_identity: "optimizer 1",
            },
            &context,
            limits,
        )
        .unwrap()
    };
    let acquire = |optimizer: &AdmittedTool| -> Result<_, Box<dyn std::error::Error>> {
        let resolved = resolve_cargo_build_inputs(&spec)?;
        let wasm = build_wasm_canisters_cached(&spec)?;
        let input = &wasm.record().artifacts()[0];
        let post_spec = ArtifactCacheSpec::new(&root.join("post-cache"), "deploy", "optimizer/v1")
            .with_cargo_build_inputs("cargo", &spec, &resolved)
            .with_input("wasm", input)
            .with_tool("optimizer", optimizer.path())
            .with_arguments(["<input>", "-o", "<output>"])
            .with_environment(context.environment.iter().cloned())
            .with_identity_bytes(
                "optimizer-cwd",
                context.current_dir.as_os_str().as_encoded_bytes(),
            )
            .with_output("deploy", &root.join("deploy.wasm"));
        let deploy = match prepare_artifact_cache(&post_spec)? {
            ArtifactCachePreparation::Reused(record) => ArtifactCacheOutcome::Reused(record),
            ArtifactCachePreparation::Build(transaction) => {
                let output = transaction.output_path("deploy")?;
                optimizer
                    .run(
                        &[input.into(), "-o".into(), output.clone().into()],
                        &context,
                        limits,
                    )?
                    .require_success()?;
                hash_reader(fs::File::open(&output)?, 1024)?;
                transaction.commit()?
            }
        };
        assert_eq!(
            fs::read(deploy.record().artifacts()[0].path()).unwrap(),
            b"\0asm\x01\0\0\0"
        );
        Ok((
            matches!(wasm, WasmBuildOutcome::Reused { .. }),
            matches!(deploy, ArtifactCacheOutcome::Reused(_)),
        ))
    };
    let optimizer = admit(script);
    assert_eq!(acquire(&optimizer).unwrap(), (false, false));
    assert_eq!(acquire(&optimizer).unwrap(), (true, true));
    assert_eq!(
        fs::read_to_string(root.join("optimizer.calls")).unwrap(),
        "run\n"
    );
    write_executable_script(&compiler, "#!/bin/sh\nprintf 'compiler-B\\n'\n");
    assert_eq!(acquire(&optimizer).unwrap(), (false, false));
    let changed_script = [script.as_slice(), b"# changed optimizer bytes\n"].concat();
    write_executable_script(&executable, &changed_script);
    let optimizer = admit(&changed_script);
    assert_eq!(acquire(&optimizer).unwrap(), (true, false));
    assert_eq!(acquire(&optimizer).unwrap(), (true, true));
    assert_eq!(
        fs::read_to_string(root.join("optimizer.calls")).unwrap(),
        "run\nrun\nrun\n"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_admitted_producers_cannot_publish_or_reuse_staged_output() {
    use crate::ic_host_process::tool::ExecutionFailure;
    use std::os::unix::process::ExitStatusExt as _;

    let root = unique_temp_directory("failed-admitted-producer")
        .canonicalize()
        .unwrap();
    let executable = root.join("producer");
    let destination = root.join("deploy.wasm");
    let published = b"previous public artifact";
    fs::write(&destination, published).unwrap();
    let context = ExecutionContext {
        current_dir: &root,
        environment: &[],
    };
    let limits = OutputLimits {
        stdout: OutputLimit::Terminate(128),
        stderr: OutputLimit::Terminate(128),
        timeout: Some(Duration::from_secs(5)),
    };
    for ending in ["exit 23", "kill -TERM \"$$\""] {
        let failing_script = format!(
            "#!/bin/sh\nif [ \"$1\" = --version ]; then printf 'producer 1\\n'; else printf '\\000asm\\001\\000\\000\\000' > \"$1\"; printf 'run\\n' >> \"$0.calls\"; {ending}; fi\n"
        );
        write_executable_script(&executable, &failing_script);
        let producer = AdmittedTool::admit(
            &ToolSpec {
                executable: &executable,
                sha256: Sha256Digest::compute(failing_script.as_bytes()),
                executable_bytes: failing_script.len() as u64,
                version_arguments: &[OsString::from("--version")],
                version_identity: "producer 1",
            },
            &context,
            limits,
        )
        .unwrap();
        let spec = ArtifactCacheSpec::new(&root.join("cache"), "deploy", "producer/v1")
            .with_tool("producer", &executable)
            .with_output("deploy", &destination);
        let acquire = || -> Result<_, Box<dyn std::error::Error>> {
            match prepare_artifact_cache(&spec)? {
                ArtifactCachePreparation::Reused(_) => Ok(()),
                ArtifactCachePreparation::Build(transaction) => {
                    let output = transaction.output_path("deploy")?;
                    let completed = producer.run(&[output.clone().into()], &context, limits)?;
                    assert_eq!(fs::read(&output)?, b"\0asm\x01\0\0\0");
                    completed.require_success()?;
                    transaction.commit()?;
                    Ok(())
                }
            }
        };
        for _ in 0..2 {
            let error = acquire().unwrap_err();
            let failure = error
                .downcast_ref::<ToolError>()
                .unwrap()
                .execution_error()
                .unwrap();
            assert!(matches!(failure.failure, ExecutionFailure::ExitStatus));
            let status = failure.evidence.status.unwrap();
            if ending == "exit 23" {
                assert_eq!(status.code(), Some(23));
            } else {
                assert_eq!(status.signal(), Some(libc::SIGTERM));
            }
            assert_eq!(fs::read(root.join("deploy.wasm")).unwrap(), published);
        }
    }
    assert_eq!(
        fs::read_to_string(root.join("producer.calls"))
            .unwrap()
            .lines()
            .count(),
        4
    );
    fs::remove_dir_all(root).unwrap();
}
