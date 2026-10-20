//! Native optimizer qualification through retained artifact transactions.

use ic_build::artifacts::{ArtifactCachePreparation, ArtifactCacheSpec, prepare_artifact_cache};
use std::{fs, path::PathBuf, process::Command};

#[test]
#[ignore = "requires explicitly admitted wasm-opt, Node and a retained evidence directory"]
fn optimizer_preserves_wasm_and_reuses_artifacts() {
    let evidence =
        PathBuf::from(std::env::var_os("IC_BUILD_WASM_EVIDENCE").expect("evidence root"));
    let optimizer =
        PathBuf::from(std::env::var_os("IC_BUILD_WASM_OPT").expect("admitted optimizer"));
    let node = PathBuf::from(std::env::var_os("IC_BUILD_WASM_NODE").expect("prepared Node"));
    let input = evidence.join("input.wasm");
    let cache = evidence.join("cache");
    let mut retained = Vec::new();
    for flag in ["O3", "Os", "Oz"] {
        let argument = format!("-{flag}");
        let output = evidence.join(format!("{flag}.wasm"));
        let spec = ArtifactCacheSpec::new(&cache, "wasm-optimizer", "native-qualification/v1")
            .with_input("wasm", &input)
            .with_tool("wasm-opt", &optimizer)
            .with_arguments([argument.as_str()])
            .with_output("optimized", &output);
        let ArtifactCachePreparation::Build(transaction) =
            prepare_artifact_cache(&spec).expect("prepare fresh transform")
        else {
            panic!("fresh qualification unexpectedly reused an artifact");
        };
        let log = fs::File::create(evidence.join(format!("{flag}.log"))).expect("optimizer log");
        let status = Command::new(&optimizer)
            .arg(&input)
            .arg(&argument)
            .arg("-o")
            .arg(transaction.output_path("optimized").expect("staged output"))
            .stdout(log.try_clone().expect("clone log"))
            .stderr(log)
            .status()
            .expect("run admitted optimizer");
        assert!(status.success(), "optimizer {flag} failed: {status}");
        let built = transaction.commit().expect("publish optimized artifact");
        assert!(!built.is_reused());
        let bytes = fs::read(&output).expect("published Wasm");
        retained.push(built.record().clone());
        fs::remove_file(&output).expect("remove materialization before reuse");
        let ArtifactCachePreparation::Reused(record) =
            prepare_artifact_cache(&spec).expect("reuse retained transform")
        else {
            panic!("unchanged transform was rebuilt");
        };
        assert_eq!(fs::read(&output).expect("repaired materialization"), bytes);
        retained.push(record);
    }
    let log = fs::File::create(evidence.join("execution.log")).expect("execution log");
    let status = Command::new(node)
        .arg(evidence.join("check.cjs"))
        .args(["input.wasm", "O3.wasm", "Os.wasm", "Oz.wasm"].map(|name| evidence.join(name)))
        .stdout(log.try_clone().expect("clone execution log"))
        .stderr(log)
        .status()
        .expect("execute Wasm oracle");
    assert!(status.success(), "Wasm execution failed: {status}");
    // Hold cache records through the oracle, including repaired materializations.
    drop(retained);
}
