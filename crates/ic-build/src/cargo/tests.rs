use super::*;
use ic_host_artifacts::batch::BatchLabelError;
use ic_host_process::tool::OutputLimit;

#[test]
fn invalid_package_inputs_reject_before_command_effects() {
    let limits = OutputLimits {
        stdout: OutputLimit::Terminate(4096),
        stderr: OutputLimit::Terminate(4096),
        timeout: None,
    };
    for (packages, expected) in [
        (
            vec!["left", "", "right"],
            BatchLabelError::Empty { index: 1 },
        ),
        (
            vec!["left", "right", "left"],
            BatchLabelError::Duplicate {
                label: "left".into(),
                first_index: 0,
                duplicate_index: 2,
            },
        ),
    ] {
        let packages = packages.into_iter().map(str::to_string).collect::<Vec<_>>();
        let error = compatible_package_groups(
            Path::new("."),
            &packages,
            "wasm32-unknown-unknown",
            &selection::CargoSelection::default(),
            limits,
            || panic!("invalid package inputs must not invoke the command factory"),
        )
        .unwrap_err();
        assert_eq!(error.downcast_ref::<BatchLabelError>(), Some(&expected));
    }
    let groups = compatible_package_groups(
        Path::new("."),
        &[],
        "wasm32-unknown-unknown",
        &selection::CargoSelection::default(),
        limits,
        || panic!("an empty plan must not invoke the command factory"),
    )
    .unwrap();
    assert_eq!(groups, Vec::<Vec<String>>::new());
}

#[test]
#[cfg(unix)]
fn unsuccessful_tree_commands_preserve_status_and_bounded_diagnostics() {
    use ic_host_process::tool::{ExecutionFailure, ToolError};
    let limits = OutputLimits {
        stdout: OutputLimit::Truncate(12),
        stderr: OutputLimit::Terminate(4096),
        timeout: None,
    };
    for stdout in ["probe stdout", "probe stdout beyond the admitted limit"] {
        let error = compatible_package_groups(
            Path::new("."),
            &["role".to_string()],
            "wasm32-unknown-unknown",
            &selection::CargoSelection::default(),
            limits,
            || {
                let mut command = Command::new("/bin/sh");
                command.args([
                    "-c",
                    "printf '%s' \"$1\"; printf 'probe stderr' >&2; exit 23",
                    "probe",
                    stdout,
                ]);
                command
            },
        )
        .unwrap_err();
        let ToolError::Execution(execution) = error
            .downcast_ref::<ToolError>()
            .expect("typed Host failure")
        else {
            panic!("expected a completed execution failure")
        };
        assert!(matches!(execution.failure, ExecutionFailure::ExitStatus));
        assert_eq!(execution.evidence.status.unwrap().code(), Some(23));
        assert_eq!(execution.evidence.stdout, b"probe stdout");
        assert_eq!(execution.evidence.stderr, b"probe stderr");
        assert_eq!(execution.evidence.stdout_truncated, stdout.len() > 12);
        assert!(!execution.evidence.stderr_truncated);
    }
    let malformed = compatible_package_groups(
        Path::new("."),
        &["role".to_string()],
        "wasm32-unknown-unknown",
        &selection::CargoSelection::default(),
        limits,
        || {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", "printf '1dependency'"]);
            command
        },
    )
    .unwrap_err();
    assert!(malformed.downcast_ref::<ToolError>().is_none());
    assert_eq!(
        malformed.to_string(),
        "Cargo dependency has no selected root"
    );
}

#[test]
#[cfg(unix)]
fn incomplete_tree_capture_and_spawn_failures_preserve_typed_host_errors() {
    let root = crate::fs_fixture::unique_temp_directory("batch-capture-failure");
    let packages = ["role".to_string()];
    let limits = OutputLimits {
        stdout: OutputLimit::Truncate(20),
        stderr: OutputLimit::Terminate(4096),
        timeout: None,
    };
    let truncated = compatible_package_groups(
        &root,
        &packages,
        "wasm32-unknown-unknown",
        &selection::CargoSelection::default(),
        limits,
        || {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", "printf '0role v0.1.0\\t\\n1shared v0.1.0\\tbase\\n'"]);
            command
        },
    )
    .unwrap_err();
    assert!(
        truncated
            .downcast_ref::<ic_host_process::tool::ToolError>()
            .is_some()
    );
    let missing = compatible_package_groups(
        &root,
        &packages,
        "wasm32-unknown-unknown",
        &selection::CargoSelection::default(),
        limits,
        || Command::new(root.join("missing-cargo")),
    )
    .unwrap_err();
    assert!(
        missing
            .downcast_ref::<ic_host_process::tool::ToolError>()
            .is_some()
    );
    fs::remove_dir_all(root).unwrap();
}
use std::fs;

#[test]
fn real_cargo_resolution_batches_peers_and_splits_feature_growth() {
    let root = crate::fs_fixture::unique_temp_directory("build-feature-batches");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("Cargo.toml"), "[workspace]\nmembers=[\"shared\",\"exclusive\",\"left\",\"peer\",\"right\"]\nresolver=\"3\"\n").unwrap();
    for (name, extra) in [
        (
            "shared",
            "[features]\ndefault=[\"base\"]\nbase=[]\nextra=[]\n",
        ),
        ("exclusive", ""),
        (
            "left",
            "[features]\nextra=[\"shared/extra\"]\n[dependencies]\nshared={path=\"../shared\"}\n",
        ),
        (
            "peer",
            "[features]\nextra=[\"shared/extra\"]\n[dependencies]\nshared={path=\"../shared\"}\nexclusive={path=\"../exclusive\"}\n",
        ),
        (
            "right",
            "[dependencies]\nshared={path=\"../shared\",features=[\"extra\"]}\n",
        ),
    ] {
        let package = root.join(name);
        fs::create_dir_all(package.join("src")).unwrap();
        fs::write(
            package.join("Cargo.toml"),
            format!("[package]\nname=\"{name}\"\nversion=\"0.1.0\"\nedition=\"2024\"\n{extra}"),
        )
        .unwrap();
        fs::write(package.join("src/lib.rs"), "pub fn fixture() {}\n").unwrap();
    }
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    assert!(
        Command::new(&cargo)
            .current_dir(&root)
            .args(["generate-lockfile", "--offline"])
            .status()
            .unwrap()
            .success()
    );
    let limits = OutputLimits {
        stdout: OutputLimit::Terminate(1024 * 1024),
        stderr: OutputLimit::Terminate(64 * 1024),
        timeout: None,
    };
    let packages = ["left", "peer", "right"].map(str::to_string);
    for (all_features, expected) in [
        (false, vec![vec!["left", "peer"], vec!["right"]]),
        (true, vec![vec!["left", "peer", "right"]]),
    ] {
        let selection = selection::CargoSelection {
            locked: true,
            offline: true,
            features: selection::CargoFeatureSelection {
                all_features,
                ..Default::default()
            },
            ..Default::default()
        };
        let groups = compatible_package_groups(
            &root,
            &packages,
            "wasm32-unknown-unknown",
            &selection,
            limits,
            || {
                let mut command = Command::new(&cargo);
                command
                    .env("RUSTUP_AUTO_INSTALL", "0")
                    .env("CARGO_NET_OFFLINE", "true");
                command
            },
        )
        .unwrap();
        assert_eq!(groups, expected);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn shared_feature_growth_rejects_a_batch_but_disjoint_dependencies_do_not() {
    let left = "0left v1.0.0\t\n1shared v1.0.0\tbase\n";
    let right = "0right v1.0.0\t\n1shared v1.0.0\tbase\n1extra v1.0.0\tunique\n";
    let isolated = parse_trees(&format!("{left}\n{right}")).unwrap();
    let packages = ["left".to_string(), "right".to_string()];
    assert!(preserves_individual_trees(&isolated, &isolated, &packages));
    let changed = parse_trees(&format!(
        "{}\n{right}",
        left.replace("\tbase", "\tbase,extra")
    ))
    .unwrap();
    assert!(!preserves_individual_trees(&isolated, &changed, &packages));
    let omitted = parse_trees(left).unwrap();
    assert!(!preserves_individual_trees(&isolated, &omitted, &packages));
    assert!(parse_trees(&format!("{left}{left}")).is_err());
    assert!(parse_trees("1shared v1.0.0\tbase\n").is_err());
}

#[test]
fn feature_assignment_order_is_retained_for_host_and_wasm_instances() {
    let isolated =
        parse_trees("0role v1.0.0\t\n1shared v1.0.0\twasm\n1shared v1.0.0\thost\n").unwrap();
    let swapped =
        parse_trees("0role v1.0.0\t\n1shared v1.0.0\thost\n1shared v1.0.0\twasm\n").unwrap();
    assert!(!preserves_individual_trees(
        &isolated,
        &swapped,
        &["role".to_string()]
    ));
}

#[test]
#[cfg(unix)]
fn grouping_forwards_recipe_graph_choices_to_every_observation() {
    use selection::{CargoFeatureSelection, CargoSelection};
    let root = crate::fs_fixture::unique_temp_directory("grouping-selection");
    let selection = CargoSelection {
        profile: Some("release".into()),
        features: CargoFeatureSelection {
            default_features: false,
            features: ["selected".into()].into(),
            all_features: false,
        },
        locked: true,
        offline: true,
    };
    let limits = OutputLimits {
        stdout: OutputLimit::Terminate(4096),
        stderr: OutputLimit::Terminate(4096),
        timeout: None,
    };
    let mut observations = 0;
    let groups = compatible_package_groups(
        &root,
        &["left".into(), "right".into()],
        "wasm32-unknown-unknown",
        &selection,
        limits,
        || {
            observations += 1;
            let mut command = Command::new("/bin/sh");
            command.args([
                "-c",
                r#"
locked=0 offline=0 defaults=0 features=0
for arg in "$@"; do
    case "$arg" in
        --locked) locked=1;;
        --offline) offline=1;;
        --no-default-features) defaults=1;;
        --features=selected) features=1;;
        --release|--profile=*) exit 24;;
    esac
done
[ "$locked$offline$defaults$features" = 1111 ] || exit 23
while [ "$#" -gt 0 ]; do
    if [ "$1" = --package ]; then
        shift
        printf '0%s v0.1.0\t\n' "$1"
    fi
    shift
done
"#,
                "tree-fixture",
            ]);
            command
        },
    )
    .unwrap();
    assert_eq!(groups, [vec!["left", "right"]]);
    assert_eq!(observations, 3);
    fs::remove_dir_all(root).unwrap();
}
