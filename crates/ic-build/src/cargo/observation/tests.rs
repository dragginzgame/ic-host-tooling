use super::*;
use crate::fs_fixture::unique_temp_directory;
use ic_host_process::tool::{ExecutionFailure, OutputLimit, ToolError};
use std::{collections::BTreeSet, fs, path::PathBuf};

fn limits() -> OutputLimits {
    OutputLimits {
        stdout: OutputLimit::Terminate(1024 * 1024),
        stderr: OutputLimit::Terminate(64 * 1024),
        timeout: None,
    }
}

fn selected_cargo() -> Command {
    Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
}

fn fixture() -> PathBuf {
    let root = unique_temp_directory("cargo-observation");
    for directory in [root.join("src"), root.join("domain/src")] {
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("lib.rs"), "pub fn value() {}\n").unwrap();
    }
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = 'observer-app'\nversion = '0.1.0'\nedition = '2024'\n\
         [workspace]\nmembers = ['domain']\nresolver = '3'\n\
         [dependencies]\nrenamed = { package = 'observer-domain', path = 'domain', optional = true }\n\
         [features]\ndefault = ['on']\non = ['dep:renamed', 'renamed/special']\n",
    ).unwrap();
    fs::write(
        root.join("domain/Cargo.toml"),
        "[package]\nname = 'observer-domain'\nversion = '0.1.0'\nedition = '2024'\n\
         [features]\nspecial = []\n",
    )
    .unwrap();
    let mut command = selected_cargo();
    command
        .current_dir(&root)
        .args(["generate-lockfile", "--offline"]);
    capture(&mut command, limits()).unwrap();
    root
}

#[test]
fn actual_catalog_and_resolved_graph_preserve_ids_aliases_and_feature_selection() {
    let root = fixture();
    let manifest = root.join("Cargo.toml");
    let features = CargoFeatureSelection {
        features: BTreeSet::new(),
        default_features: false,
        all_features: false,
    };
    let mut spec = CargoMetadataSpec {
        manifest_path: &manifest,
        include_dependencies: false,
        filter_platform: Some("wasm32-unknown-unknown"),
        locked: true,
        offline: true,
        features: &features,
    };
    let catalog = observe_metadata(&spec, limits(), selected_cargo()).unwrap();
    assert!(catalog.resolve.is_none());
    let app = catalog
        .packages
        .iter()
        .find(|p| p.name == "observer-app")
        .unwrap();
    assert_eq!(app.dependencies[0].rename.as_deref(), Some("renamed"));
    assert!(app.dependencies[0].optional);
    let id = app.id.clone();

    spec.include_dependencies = true;
    let disabled = observe_metadata(&spec, limits(), selected_cargo()).unwrap();
    let app_node = disabled
        .resolve
        .unwrap()
        .nodes
        .into_iter()
        .find(|n| n.id == id)
        .unwrap();
    assert_eq!(app_node.deps, []);

    let mut explicit = features.clone();
    explicit.features.insert("on".into());
    let all = CargoFeatureSelection {
        all_features: true,
        ..features.clone()
    };
    for enabled_features in [explicit, all] {
        let selected = CargoMetadataSpec {
            features: &enabled_features,
            ..spec
        };
        let enabled = observe_metadata(&selected, limits(), selected_cargo()).unwrap();
        let graph = enabled.resolve.unwrap();
        let app_node = graph.nodes.iter().find(|n| n.id == id).unwrap();
        assert_eq!(app_node.deps[0].name, "renamed");
        let domain_node = graph
            .nodes
            .iter()
            .find(|n| n.id == app_node.deps[0].pkg)
            .unwrap();
        assert!(domain_node.features.iter().any(|f| f.as_str() == "special"));
        assert_eq!(
            domain_node.id,
            catalog
                .packages
                .iter()
                .find(|p| p.name == "observer-domain")
                .unwrap()
                .id
        );

        let tree = observe_tree(
            &CargoTreeSpec {
                manifest_path: &manifest,
                packages: &[&id.repr],
                target: "wasm32-unknown-unknown",
                edges: "normal",
                format: "{p}\t{lib}\t{f}",
                locked: true,
                offline: true,
                features: &enabled_features,
            },
            limits(),
            selected_cargo(),
        )
        .unwrap();
        assert!(tree.lines().next().unwrap().starts_with("0observer-app "));
        assert!(
            tree.lines()
                .any(|line| line.starts_with("1observer-domain ") && line.ends_with("\tspecial"))
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn actual_build_script_messages_distinguish_same_name_workspaces_and_cached_exports() {
    let first = fixture();
    let second = fixture();
    let target = unique_temp_directory("shared-script-target");
    for root in [&first, &second] {
        fs::write(root.join("external.txt"), root.to_str().unwrap()).unwrap();
        fs::write(
            root.join("build.rs"),
            format!(
                "fn main() {{ let root = {root:?}; \
             println!(\"cargo::rustc-env=OBSERVED_EXTERNAL={{}}/external.txt\", root); \
             println!(\"cargo::rerun-if-changed=build.rs\"); }}\n"
            ),
        )
        .unwrap();
    }
    let build = |root: &Path| {
        let mut command = selected_cargo();
        command
            .current_dir(root)
            .args([
                "build",
                "--locked",
                "--offline",
                "--no-default-features",
                "--message-format=json",
                "--target-dir",
            ])
            .arg(&target);
        let output = capture(&mut command, limits()).unwrap();
        schema::Message::parse_stream(std::io::Cursor::new(output.stdout))
            .map(Result::unwrap)
            .filter_map(|message| match message {
                schema::Message::BuildScriptExecuted(script) => Some(script),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let first_scripts = build(&first);
    let second_scripts = build(&second);
    assert_eq!(first_scripts.len(), 1);
    assert_eq!(second_scripts.len(), 1);
    assert_ne!(first_scripts[0].package_id, second_scripts[0].package_id);
    // Same-name workspaces may share an uplifted output directory. The package
    // IDs and exact observed export bytes, not directory names, establish custody.
    let foreign_record = target.join("debug/build/foreign-unselected/output");
    fs::create_dir_all(foreign_record.parent().unwrap()).unwrap();
    fs::write(foreign_record, [0xff]).unwrap();
    let dependency_record = target.join("debug/libobserver_app.d");
    fs::remove_file(dependency_record).unwrap();
    // Cargo emits the selected package's exact ID despite an unrelated malformed
    // retained output and no uplifted dependency record.
    let cached = build(&second);
    assert_eq!(cached[0].package_id, second_scripts[0].package_id);
    assert_eq!(cached[0].out_dir, second_scripts[0].out_dir);
    assert_eq!(cached[0].env, second_scripts[0].env);
    let retained_output =
        fs::read_to_string(cached[0].out_dir.parent().unwrap().join("output")).unwrap();
    let exported = cached[0]
        .env
        .iter()
        .find(|(name, _)| name == "OBSERVED_EXTERNAL")
        .unwrap();
    assert!(
        retained_output
            .lines()
            .any(|line| line == format!("cargo::rustc-env={}={}", exported.0, exported.1))
    );
    for root in [first, second, target] {
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(unix)]
fn shell(script: &str) -> Command {
    let mut command = Command::new("/bin/sh");
    command.args(["-c", script, "cargo-observation"]);
    command
}

#[test]
#[cfg(unix)]
fn caller_command_and_environment_are_preserved() {
    for (names, expected) in [
        (vec![], vec!["--no-default-features"]),
        (
            vec!["z", "a", "a"],
            vec!["--no-default-features", "--features=a,z"],
        ),
        (
            vec!["a", ""],
            vec!["--no-default-features", "--features=,a"],
        ),
        (vec![""], vec!["--no-default-features", "--features="]),
    ] {
        let selection = CargoFeatureSelection {
            features: names.into_iter().map(str::to_owned).collect(),
            default_features: false,
            all_features: false,
        };
        let mut command = Command::new("unused");
        command.args(selection.arguments());
        assert_eq!(command.get_args().collect::<Vec<_>>(), expected);
    }
    let root = fixture();
    let manifest = root.join("Cargo.toml");
    let args = root.join("observed-args");
    let mut command = shell(
        "printf '%s\\n' \"$OBSERVATION_ENV\" \"$PWD\" \"$@\" > \"$OBSERVATION_ARGS\"; exec \"$SELECTED_CARGO\" \"$@\"",
    );
    command
        .env("OBSERVATION_ENV", "selected environment")
        .env("OBSERVATION_ARGS", &args)
        .env("SELECTED_CARGO", selected_cargo().get_program());
    let features = CargoFeatureSelection::default();
    let metadata = observe_metadata(
        &CargoMetadataSpec {
            manifest_path: &manifest,
            include_dependencies: false,
            filter_platform: None,
            locked: true,
            offline: true,
            features: &features,
        },
        limits(),
        command,
    )
    .unwrap();
    assert_eq!(metadata.workspace_root.as_std_path(), root);
    let recorded = fs::read_to_string(args).unwrap();
    let recorded = recorded.lines().collect::<Vec<_>>();
    assert_eq!(
        &recorded[..3],
        ["selected environment", root.to_str().unwrap(), "metadata"]
    );
    for argument in ["--format-version=1", "--locked", "--offline", "--no-deps"] {
        assert!(recorded.contains(&argument));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn incomplete_output_and_failed_exit_keep_typed_host_evidence() {
    let manifest = Path::new("Cargo.toml");
    let features = CargoFeatureSelection::default();
    let spec = CargoMetadataSpec {
        manifest_path: manifest,
        include_dependencies: true,
        filter_platform: None,
        locked: true,
        offline: true,
        features: &features,
    };
    let bounded = OutputLimits {
        stdout: OutputLimit::Truncate(5),
        ..limits()
    };
    let error = observe_metadata(
        &spec,
        bounded,
        shell("printf 'large stdout'; printf 'cause' >&2; exit 23"),
    )
    .unwrap_err();
    let ToolError::Execution(error) = error.downcast_ref::<ToolError>().unwrap() else {
        panic!("typed execution failure")
    };
    assert!(matches!(error.failure, ExecutionFailure::ExitStatus));
    assert_eq!(error.evidence.status.unwrap().code(), Some(23));
    assert_eq!(error.evidence.stdout, b"large");
    assert!(error.evidence.stdout_truncated);
    assert_eq!(error.evidence.stderr, b"cause");

    let error = observe_metadata(&spec, bounded, shell("printf 'large stdout'")).unwrap_err();
    assert!(
        matches!(error.downcast_ref::<ToolError>(), Some(ToolError::Execution(error))
        if error.evidence.stdout_truncated)
    );
    let error =
        observe_metadata(&spec, limits(), Command::new("/absent/cargo-observation")).unwrap_err();
    assert!(
        matches!(error.downcast_ref::<ToolError>(), Some(ToolError::Execution(error))
        if matches!(error.failure, ExecutionFailure::Io { .. }))
    );

    let terminated = OutputLimits {
        stdout: OutputLimit::Terminate(5),
        ..limits()
    };
    let error = observe_metadata(&spec, terminated, shell("printf 'large stdout'")).unwrap_err();
    assert!(error.downcast_ref::<ToolError>().is_some());
}

#[test]
#[cfg(unix)]
fn malformed_metadata_missing_fields_and_invalid_tree_text_are_rejected() {
    let features = CargoFeatureSelection::default();
    let spec = CargoMetadataSpec {
        manifest_path: Path::new("Cargo.toml"),
        include_dependencies: true,
        filter_platform: None,
        locked: false,
        offline: false,
        features: &features,
    };
    for script in ["printf 'not json'", "printf '{}'", "printf ''"] {
        let error = observe_metadata(&spec, limits(), shell(script)).unwrap_err();
        assert!(error.downcast_ref::<serde_json::Error>().is_some());
    }
    let mut tree = CargoTreeSpec {
        manifest_path: spec.manifest_path,
        packages: &["app"],
        target: "wasm32-unknown-unknown",
        edges: "normal",
        format: "{p}",
        locked: false,
        offline: false,
        features: &features,
    };
    let error = observe_tree(&tree, limits(), shell("printf '\\377'")).unwrap_err();
    assert!(error.downcast_ref::<std::string::FromUtf8Error>().is_some());
    tree.packages = &[];
    assert!(observe_tree(&tree, limits(), Command::new("/absent/cargo-observation")).is_err());
}
