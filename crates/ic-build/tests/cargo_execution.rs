//! Real Cargo messages bind exact outputs and build-script exports before capture.
mod support;

use ic_build::{
    artifacts::CapturedArtifact,
    cargo::{
        execution::{
            CargoExecutionPolicy, CargoInvocation, CargoRecipe, execute_cargo, select_cdylib_wasm,
        },
        observation::schema::Message,
        selection::CargoSelection,
    },
};
use ic_host_process::{
    child::{CleanupPolicy, OwnedChild},
    tool::{OutputLimit, OutputLimits},
};
use std::{fs, path::Path, process::Command};

#[test]
fn reported_custom_libraries_and_exports_survive_shared_target_collisions() {
    let root = support::fs_fixture::unique_temp_directory("cargo-execution");
    fs::create_dir_all(root.join("captures")).unwrap();
    let target = root.join("target");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut retained = Vec::new();
    for (workspace, marker) in [("first", 1), ("second", 2)] {
        let directory = root.join(workspace);
        let manifest = write_library_fixture(&directory, marker, &cargo);
        let mut environment = support::cargo::environment();
        environment
            .entry("RUSTC".into())
            .or_insert_with(|| "rustc".into());
        environment.insert("CARGO_TARGET_DIR".into(), target.clone().into_os_string());
        let invocation = CargoInvocation {
            cargo_program: cargo.clone(),
            working_directory: directory.clone(),
            environment,
            target: "wasm32-unknown-unknown".into(),
            selection: CargoSelection {
                locked: true,
                offline: true,
                ..CargoSelection::default()
            },
            packages: Vec::new(),
            manifest_path: Some(manifest.clone()),
            recipe: CargoRecipe::Library,
            configurations: Vec::new(),
            render_diagnostics: true,
            keep_going: false,
        };
        let mut messages = Vec::new();
        let observed = execute_cargo(
            &invocation,
            CargoExecutionPolicy {
                output: OutputLimits {
                    stdout: OutputLimit::Truncate(0),
                    stderr: OutputLimit::Truncate(1024 * 1024),
                    timeout: None,
                },
                message_bytes: 1024 * 1024,
                probe_cleanup: CleanupPolicy::KillAndWait,
            },
            OwnedChild::spawn_direct,
            || false,
            |_, _| Ok(()),
            |message| {
                messages.push(message);
                Ok(())
            },
        )
        .unwrap();
        let evidence = observed.execution.unwrap();
        assert!(
            evidence.status.success(),
            "{}",
            String::from_utf8_lossy(&evidence.stderr)
        );
        observed.messages.unwrap();
        let facts = observed.facts.unwrap().unwrap();
        assert_eq!(facts.environment, invocation.environment);
        assert_eq!(facts.selection, invocation.selection);
        assert_eq!(facts.target, invocation.target);
        let script = messages
            .iter()
            .find_map(|message| {
                if let Message::BuildScriptExecuted(script) = message {
                    Some(script)
                } else {
                    None
                }
            })
            .unwrap();
        assert!(
            script
                .env
                .iter()
                .any(|(name, value)| name == "CONSUMER_INPUT" && Path::new(value) == directory)
        );
        let path = select_cdylib_wasm(&messages, &manifest).unwrap();
        assert_eq!(path.file_name().unwrap(), "custom_library.wasm");
        let bytes = fs::read(&path).unwrap();
        let capture =
            CapturedArtifact::capture(&path, &root.join("captures"), 128 * 1024 * 1024).unwrap();
        retained.push((path, bytes, capture));
    }
    assert_eq!(retained[0].0, retained[1].0);
    assert_ne!(retained[0].1, retained[1].1);
    for (_, bytes, capture) in &retained {
        assert_eq!(&fs::read(capture.path()).unwrap(), bytes);
    }
    assert_eq!(fs::read(&retained[0].0).unwrap(), retained[1].1);
    drop(retained);
    fs::remove_dir_all(root).unwrap();
}

fn write_library_fixture(
    directory: &Path,
    marker: u32,
    cargo: &std::ffi::OsStr,
) -> std::path::PathBuf {
    fs::create_dir_all(directory.join("src")).unwrap();
    let manifest = directory.join("Cargo.toml");
    fs::write(&manifest, "[workspace]\n[package]\nname='collision'\nversion='0.1.0'\nedition='2024'\n[lib]\nname='custom_library'\ncrate-type=['cdylib']\n").unwrap();
    fs::write(
        directory.join("src/lib.rs"),
        format!("#[unsafe(no_mangle)] pub extern \"C\" fn marker() -> u32 {{ {marker} }}\n"),
    )
    .unwrap();
    fs::write(directory.join("build.rs"), "fn main() { println!(\"cargo:rustc-env=CONSUMER_INPUT={}\", std::env::var(\"CARGO_MANIFEST_DIR\").unwrap()); }\n").unwrap();
    assert!(
        Command::new(cargo)
            .current_dir(directory)
            .args(["generate-lockfile", "--offline"])
            .status()
            .unwrap()
            .success()
    );
    manifest
}
