use super::*;
use serde_json::json;

fn spec() -> NativeExecutableSpec<'static> {
    NativeExecutableSpec {
        package: "fixture",
        target_name: "worker",
        target_kind: NativeTargetKind::Example,
        selection: CargoSelection::default(),
        max_message_bytes: 4096,
    }
}

fn metadata() -> schema::Metadata {
    serde_json::from_value(json!({
        "packages": [{
            "name": "fixture", "version": "0.0.0",
            "id": "path+file:///source#fixture@0.0.0",
            "dependencies": [], "features": {}, "manifest_path": "/source/Cargo.toml",
            "targets": [{"name": "worker", "kind": ["example"], "src_path": "/source/examples/worker.rs"}]
        }],
        "workspace_members": [], "workspace_root": "/source",
        "target_directory": "/selected-target", "version": 1
    }))
    .unwrap()
}

#[test]
fn metadata_selection_refuses_missing_ambiguous_and_relative_roots() {
    let spec = spec();
    let original = metadata();
    let (id, root) = selection(original.clone(), &spec).unwrap();
    assert_eq!(id, "path+file:///source#fixture@0.0.0");
    assert_eq!(root, PathBuf::from("/selected-target"));
    let mut ambiguous = original.clone();
    ambiguous.packages.push(original.packages[0].clone());
    assert!(selection(ambiguous, &spec).is_err());
    let mut missing = original.clone();
    missing.packages[0].targets.clear();
    assert!(selection(missing, &spec).is_err());
    let mut relative = original.clone();
    relative.target_directory = "relative-target".into();
    assert!(selection(relative, &spec).is_err());
    let mut wrong_kind = original;
    wrong_kind.packages[0].targets[0].kind = vec![schema::TargetKind::Lib];
    assert!(selection(wrong_kind, &spec).is_err());
    let mut missing_package = metadata();
    missing_package.packages.clear();
    assert!(selection(missing_package, &spec).is_err());
    let mut duplicate_target = metadata();
    let target = duplicate_target.packages[0].targets[0].clone();
    duplicate_target.packages[0].targets.push(target);
    assert!(selection(duplicate_target, &spec).is_err());
    let mut missing_identity = metadata();
    missing_identity.packages[0].id.repr.clear();
    assert_eq!(
        selection(missing_identity, &spec).unwrap_err().to_string(),
        "Cargo omitted package identity"
    );
}

#[test]
fn messages_keep_exact_package_target_diagnostics_and_ambiguity() {
    let spec = spec();
    let (id, _) = selection(metadata(), &spec).unwrap();
    let target = serde_json::to_value(&metadata().packages[0].targets[0]).unwrap();
    let artifact = json!({
        "reason": "compiler-artifact", "package_id": id, "target": target,
        "profile": {"opt_level": "0", "debug_assertions": true, "overflow_checks": true, "test": false},
        "features": [], "filenames": [], "fresh": false,
        "executable": "/custom-target/custom-profile/examples/worker"
    });
    let mut output = Vec::new();
    let mut selected = None;
    let mut unrelated = artifact.clone();
    unrelated["package_id"] = json!("path+file:///other#fixture@0.0.0");
    message(
        &serde_json::to_vec(&unrelated).unwrap(),
        &id,
        &spec,
        &mut selected,
        &mut output,
    )
    .unwrap();
    assert!(selected.is_none());
    let mut other_target = artifact.clone();
    other_target["target"]["name"] = json!("another-worker");
    message(
        &serde_json::to_vec(&other_target).unwrap(),
        &id,
        &spec,
        &mut selected,
        &mut output,
    )
    .unwrap();
    assert!(selected.is_none());
    let diagnostic = json!({
        "reason": "compiler-message", "package_id": id, "target": target,
        "message": {"message": "café", "level": "warning", "spans": [], "children": [], "rendered": "warning: café\n"}
    });
    message(
        &serde_json::to_vec(&diagnostic).unwrap(),
        &id,
        &spec,
        &mut selected,
        &mut output,
    )
    .unwrap();
    assert_eq!(output, "warning: café\n".as_bytes());
    let bytes = serde_json::to_vec(&artifact).unwrap();
    message(&bytes, &id, &spec, &mut selected, &mut output).unwrap();
    assert_eq!(
        selected,
        Some(PathBuf::from(
            "/custom-target/custom-profile/examples/worker"
        ))
    );
    assert!(message(&bytes, &id, &spec, &mut selected, &mut output).is_err());
    for invalid_path in [json!(null), json!("relative")] {
        let mut invalid = artifact.clone();
        invalid["executable"] = invalid_path;
        assert!(
            message(
                &serde_json::to_vec(&invalid).unwrap(),
                &id,
                &spec,
                &mut None,
                &mut output,
            )
            .is_err()
        );
    }
}

#[test]
fn message_allowance_and_protocol_refusals_preserve_boundaries() {
    let mut bytes = Vec::new();
    append(&mut bytes, b"123", 4).unwrap();
    append(&mut bytes, b"4", 4).unwrap();
    assert!(append(&mut bytes, b"5", 4).is_err());
    assert_eq!(bytes, b"1234");
    let spec = spec();
    for invalid in [b"not JSON".as_slice(), b"{}", br#"{"reason":"compiler-artifact","package_id":"selected","target":{"name":"worker","kind":["example"]},"executable":"relative"}"#] {
        assert!(message(invalid, "selected", &spec, &mut None, &mut Vec::new()).is_err());
    }
}

#[test]
fn invalid_selection_refuses_before_the_command_factory() {
    for invalid in [
        NativeExecutableSpec {
            package: "",
            ..spec()
        },
        NativeExecutableSpec {
            max_message_bytes: 0,
            ..spec()
        },
    ] {
        assert!(
            build_native_executable(
                &invalid,
                || panic!("invalid selection has no command effects"),
                Vec::new(),
                || Ok(()),
                OwnedChild::spawn_direct,
                || false,
            )
            .is_err()
        );
    }
    for profile in ["", "..", "nested/profile", "/absolute"] {
        let invalid = NativeExecutableSpec {
            selection: CargoSelection {
                profile: Some(profile.into()),
                ..Default::default()
            },
            ..spec()
        };
        assert!(
            build_native_executable(
                &invalid,
                || panic!("invalid profile has no command effects"),
                Vec::new(),
                || Ok(()),
                OwnedChild::spawn_direct,
                || false
            )
            .is_err()
        );
    }
}

fn message(
    bytes: &[u8],
    package: &str,
    spec: &NativeExecutableSpec<'_>,
    executable: &mut Option<PathBuf>,
    diagnostics: &mut impl Write,
) -> io::Result<()> {
    selected_message(
        serde_json::from_slice(bytes).map_err(io::Error::other)?,
        package,
        spec,
        executable,
        diagnostics,
    )
}

#[cfg(unix)]
#[test]
fn caller_spawn_and_cancellation_cover_metadata_and_build() {
    use ic_host_process::{
        child::CleanupPolicy,
        tool::{ExecutionFailure, ToolError},
    };
    use std::{
        cell::Cell,
        fs,
        os::unix::process::ExitStatusExt as _,
        time::{Duration, Instant},
    };

    let root = crate::fs_fixture::unique_temp_directory("native-cancellation");
    let metadata_path = root.join("metadata.json");
    fs::write(&metadata_path, serde_json::to_vec(&metadata()).unwrap()).unwrap();
    for phase in ["metadata", "build"] {
        let ready = root.join(phase);
        let spawned = Cell::new(0);
        let started = Instant::now();
        let error = build_native_executable(
            &spec(),
            || {
                let mut command = Command::new("/bin/sh");
                command
                    .args([
                        "-c",
                        r#"
if [ "$1" = "$CANCEL_PHASE" ]; then
    printf armed > "$READY"
    exec sleep 30
fi
cat "$METADATA"
"#,
                        "cargo-fixture",
                    ])
                    .env("CANCEL_PHASE", phase)
                    .env("READY", &ready)
                    .env("METADATA", &metadata_path)
                    .stderr(Stdio::piped());
                command
            },
            Vec::new(),
            || Ok(()),
            |command| {
                spawned.set(spawned.get() + 1);
                OwnedChild::spawn(command, CleanupPolicy::KillAndWait)
            },
            || ready.is_file() || started.elapsed() > Duration::from_secs(20),
        )
        .unwrap_err();
        assert_eq!(fs::read(&ready).unwrap(), b"armed");
        assert_eq!(spawned.get(), if phase == "metadata" { 1 } else { 2 });
        let execution = error
            .downcast_ref::<ToolError>()
            .unwrap()
            .execution_error()
            .unwrap();
        assert!(matches!(execution.failure, ExecutionFailure::Cancelled));
        assert!(execution.cleanup.is_none());
        assert_eq!(
            execution.evidence.status.unwrap().signal(),
            Some(libc::SIGKILL)
        );
    }
    fs::remove_dir_all(root).unwrap();
}
