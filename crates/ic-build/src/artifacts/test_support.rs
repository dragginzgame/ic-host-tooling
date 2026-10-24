#[cfg(unix)]
use ic_host_process::testing::write_executable_script;

use crate::fs_fixture::unique_temp_directory;
use std::path::PathBuf;

/// Real Cargo metadata and identities, with a deterministic minimal Wasm producer.
#[cfg(unix)]
pub(super) fn fake_wasm_build_spec(label: &str) -> (PathBuf, super::wasm_cache::WasmBuildSpec) {
    let root = unique_temp_directory(label);
    crate::cargo_fixture::write_workspace(&root, &["fixture"]).expect("write workspace manifest");
    crate::cargo_fixture::write_fixture_package(
        &root.join("fixture"),
        "[lib]\ncrate-type = [\"cdylib\"]\n",
        "pub fn value() -> u8 { 1 }\n",
    )
    .expect("write fixture package");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let quoted_cargo = format!("'{}'", cargo.to_string_lossy().replace('\'', "'\"'\"'"));
    let wrapper = root.join("cargo.sh");
    let artifact_message = crate::cargo_fixture::wasm_artifact_message_script(&root, "target");
    write_executable_script(
        &wrapper,
        format!(
            r#"#!/bin/sh
set -eu
if [ "$1" = build ]; then
    target="$CARGO_TARGET_DIR"
    mkdir -p "$target/wasm32-unknown-unknown/debug"
    printf '\000asm\001\000\000\000' > "$target/wasm32-unknown-unknown/debug/fixture.wasm"
    {artifact_message}
    exit 0
fi
exec {quoted_cargo} "$@"
"#
        ),
    )
    .expect("write executable test script");
    let spec = super::wasm_cache::WasmBuildSpec::new(
        &root,
        &root.join("exact"),
        &["fixture"],
        crate::cargo_fixture::environment(),
        CONFIGURATION_LIMITS,
    )
    .with_isolated_builds()
    .with_cargo_program(wrapper);
    (root, spec)
}

// This caller admits small configuration trees; libraries do not select this policy.
const CONFIGURATION_LIMITS: crate::cargo::environment::ConfigurationLimits =
    crate::cargo::environment::ConfigurationLimits {
        file_bytes: 1024 * 1024,
        total_bytes: 16 * 1024 * 1024,
        files: 128,
        depth: 32,
    };
