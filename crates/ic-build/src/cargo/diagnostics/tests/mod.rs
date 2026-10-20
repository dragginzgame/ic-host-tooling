use super::*;
use crate::fs_fixture::unique_temp_directory;
use serde_json::json;

#[test]
fn exact_library_selection_rejects_incomplete_ambiguous_and_oversized_evidence() {
    let root = unique_temp_directory("diagnostic-artifacts");
    let path = root.join("fixture.rlib");
    fs::write(&path, []).unwrap();
    let artifact = json!({
        "reason":"compiler-artifact", "package_id":"path+file:///fixture#0.1.0",
        "manifest_path":"/fixture/Cargo.toml",
        "target":{"kind":["lib"],"crate_types":["lib"],"name":"fixture",
            "src_path":"/fixture/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},
        "profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":false},
        "features":[],"filenames":[path],"executable":null,"fresh":false
    }).to_string() + "\n";
    let finished = "{\"reason\":\"build-finished\",\"success\":true}\n";
    let stream = format!("{artifact}{finished}");
    assert_eq!(
        library_artifact(stream.as_bytes(), "fixture", &schema::TargetKind::Lib, 4096).unwrap(),
        path
    );
    for input in [
        artifact.clone(),
        format!("{artifact}{artifact}{finished}"),
        "bad json".to_string(),
    ] {
        assert!(
            library_artifact(input.as_bytes(), "fixture", &schema::TargetKind::Lib, 4096).is_err()
        );
    }
    assert!(
        library_artifact(stream.as_bytes(), "missing", &schema::TargetKind::Lib, 4096).is_err()
    );
    assert!(library_artifact(stream.as_bytes(), "fixture", &schema::TargetKind::Lib, 3).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn compiler_control_and_every_expected_diagnostic_are_required() {
    let root = unique_temp_directory("diagnostic-suite");
    let valid = root.join("valid.rs");
    let invalid = root.join("invalid.rs");
    fs::write(&valid, "pub fn valid() {}\n").unwrap();
    fs::write(&invalid, "compile_error!(\"selected diagnostic\");\n").unwrap();
    let command = |case: Option<usize>| {
        let mut command = Command::new("rustc");
        command
            .args(["--crate-type=lib", "--emit=metadata"])
            .arg(if case.is_none() { &valid } else { &invalid })
            .arg("-o")
            .arg(root.join("fixture.rmeta"));
        command
    };
    let expected = vec![vec!["selected diagnostic".to_string()]];
    check_suite(&expected, &root, 65536, command).unwrap();
    assert!(root.join("control.stderr").is_file());
    assert!(
        check_suite(
            &[vec![
                "selected diagnostic".into(),
                "absent diagnostic".into()
            ]],
            &root,
            65536,
            command
        )
        .is_err()
    );
    assert!(check_suite(&expected, &root, 65536, |_| command(None)).is_err());
    let mut invocations = 0;
    assert!(
        check_suite(&expected, &root, 65536, |_| {
            invocations += 1;
            command(Some(0))
        })
        .is_err()
    );
    assert_eq!(invocations, 1);
    assert!(check_suite(&expected, &root, 1, command).is_err());
    assert!(
        check_suite(&[vec![]], &root, 65536, |_| panic!(
            "must refuse before spawn"
        ))
        .is_err()
    );
    fs::remove_dir_all(root).unwrap();
}
