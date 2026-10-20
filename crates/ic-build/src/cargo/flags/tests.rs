//! Pure inherited-flag precedence and one real Cargo argument-boundary check.

use crate::cargo::flags::combined_encoded_rustflags;
use std::{env, fs, process::Command};

#[test]
fn inherited_flag_precedence_preserves_empty_values_and_argument_boundaries() {
    let additions = [String::new(), "--cfg".into(), "extra".into(), String::new()];
    for (encoded, ordinary, inherited) in [
        (
            Some("--cfg\x1fcaller_encoded"),
            Some("--cfg ignored"),
            "--cfg\x1fcaller_encoded",
        ),
        (Some(""), Some("--cfg ignored"), ""),
        (
            None,
            Some("--cfg caller_ordinary"),
            "--cfg\x1fcaller_ordinary",
        ),
        (
            None,
            Some("\t--cfg  caller_ordinary\n"),
            "--cfg\x1fcaller_ordinary",
        ),
        (None, None, ""),
    ] {
        let expected = if inherited.is_empty() {
            "--cfg\x1fextra\x1f".to_owned()
        } else {
            format!("{inherited}\x1f\x1f--cfg\x1fextra\x1f")
        };
        assert_eq!(
            combined_encoded_rustflags(encoded, ordinary, &additions),
            Some(expected)
        );
        assert_eq!(combined_encoded_rustflags(encoded, ordinary, &[]), None);
    }
}

#[test]
fn caller_flags_and_release_remaps_reach_cargo() {
    let root =
        ic_host_fs::testing::create_fixture_directory(&env::temp_dir(), "ic build flags explicit")
            .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"flags_probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n",
    )
    .unwrap();
    let source = root.join("src/absolute.rs");
    fs::write(&source, "const SOURCE: &str = file!();\n").unwrap();
    fs::write(
        root.join("src/main.rs"),
        format!(
            "#![allow(unexpected_cfgs)]\ninclude!({source:?});\nfn main() {{ println!(\"{{}}|{{}}|{{}}\", SOURCE, cfg!(caller_encoded), cfg!(caller_ordinary)); }}\n"
        ),
    ).unwrap();
    let additions = [format!(
        "--remap-path-prefix={}=/trimmed flags",
        root.display()
    )];
    let flags = combined_encoded_rustflags(
        Some("--cfg\x1fcaller_encoded"),
        Some("--cfg caller_ordinary"),
        &additions,
    )
    .unwrap();
    let output = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args(["run", "--offline", "--quiet"])
        .env("CARGO_ENCODED_RUSTFLAGS", flags)
        .env("RUSTFLAGS", "--cfg caller_ordinary")
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env_remove("CARGO_BUILD_TARGET")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "/trimmed flags/src/absolute.rs|true|false"
    );
    fs::remove_dir_all(&root).unwrap();
}
