use super::*;
use crate::fs_fixture::unique_temp_directory;
use sha2::{Digest as _, Sha256};

#[test]
fn discovery_and_environment_classification_keep_caller_selection() {
    let root = unique_temp_directory("cargo-environment-directories");
    let invocation = root.join("workspace/nested");
    let directories = configuration_directories(
        &invocation,
        Some(OsStr::new("relative-home")),
        Some(OsStr::new("ignored")),
    )
    .unwrap();
    assert_eq!(directories[0], invocation.join(".cargo"));
    assert_eq!(
        directories.last().unwrap(),
        &invocation.join("relative-home")
    );
    let default = configuration_directories(&invocation, None, Some(root.as_os_str())).unwrap();
    assert_eq!(default.last().unwrap(), &root.join(".cargo"));
    assert!(configuration_directories(Path::new("relative"), None, None).is_err());
    for name in [
        "RUSTC",
        "RUSTDOC",
        "CARGO_BUILD_RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
    ] {
        assert!(is_compiler_override(name));
    }
    for name in [
        "RUSTDOCFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER",
        "CARGO_PROFILE_RELEASE_LTO",
    ] {
        assert!(is_build_variable(name));
    }
    for name in ["HOME", "CARGO_HOME", "CARGO_TARGET_DIR", "GITHUB_TOKEN"] {
        assert!(!is_build_variable(name));
        assert!(!is_compiler_override(name));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn observations_bind_raw_bytes_overrides_and_include_trees() {
    let root = unique_temp_directory("cargo-environment-observe");
    let cargo = root.join(".cargo");
    fs::create_dir_all(&cargo).unwrap();
    let config = cargo.join("config");
    let shadowed = cargo.join("config.toml");
    let included = cargo.join("included.toml");
    let source = "include = ['included.toml', {path = 'missing.toml', optional = true}]\n[build]\nrustc = 'caller-compiler'\nrustdoc = 'caller-doc'\n";
    fs::write(&config, source).unwrap();
    fs::write(&shadowed, "[build]\nrustc-wrapper = 'shadowed'\n").unwrap();
    fs::write(
        &included,
        "include = 'config'\n[build]\nrustc-workspace-wrapper = 'included'\n",
    )
    .unwrap();
    assert_eq!(preferred_configuration(&cargo), Some(config.clone()));
    let observed = observe_configurations(&[
        config.clone(),
        included.clone(),
        config.clone(),
        shadowed.clone(),
    ])
    .unwrap();
    assert_eq!(
        observed.iter().map(|file| &file.path).collect::<Vec<_>>(),
        vec![&config, &included, &shadowed]
    );
    assert_eq!(
        *observed[0].sha256.as_bytes(),
        <[u8; 32]>::from(Sha256::digest(source.as_bytes()))
    );
    assert_eq!(observed[0].compiler_overrides, ["rustc", "rustdoc"]);
    assert_eq!(observed[1].compiler_overrides, ["rustc-workspace-wrapper"]);
    assert_eq!(observed[2].compiler_overrides, ["rustc-wrapper"]);
    fs::write(&config, source.replace("caller-doc", "changed-doc")).unwrap();
    assert_ne!(
        observe_configurations(std::slice::from_ref(&config)).unwrap()[0].sha256,
        observed[0].sha256
    );
    let absolute = Value::String(included.to_str().unwrap().to_owned());
    fs::write(
        &config,
        format!("include = {{ path = {absolute}, optional = false }}\n"),
    )
    .unwrap();
    let observed = observe_configurations(std::slice::from_ref(&config)).unwrap();
    assert_eq!(
        observed.iter().map(|file| &file.path).collect::<Vec<_>>(),
        [&config, &included]
    );
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn repeated_configuration_roots_keep_distinct_alias_include_trees() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_directory("cargo-environment-aliases");
    let shared = root.join("shared.toml");
    fs::write(&shared, "include = 'local.toml'\n").unwrap();
    let mut roots = Vec::new();
    let mut expected = Vec::new();
    for name in ["left", "right"] {
        let directory = root.join(name);
        fs::create_dir(&directory).unwrap();
        let alias = directory.join("config.toml");
        let local = directory.join("local.toml");
        symlink(&shared, &alias).unwrap();
        fs::write(
            &local,
            format!("include = 'config.toml'\n[build]\nrustc = '{name}'\n"),
        )
        .unwrap();
        roots.extend([alias.clone(), local.clone(), alias.clone()]);
        expected.extend([alias, local]);
    }
    let observed = observe_configurations(&roots).unwrap();
    assert_eq!(
        observed.iter().map(|file| &file.path).collect::<Vec<_>>(),
        expected.iter().collect::<Vec<_>>()
    );
    assert_eq!(observed[0].sha256, observed[2].sha256);
    assert_ne!(observed[1].sha256, observed[3].sha256);
    assert_eq!(observed[1].compiler_overrides, ["rustc"]);
    assert_eq!(observed[3].compiler_overrides, ["rustc"]);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn required_and_malformed_configuration_fail_without_source_text_in_diagnostics() {
    let root = unique_temp_directory("cargo-environment-refusal");
    fs::create_dir_all(&root).unwrap();
    let config = root.join("config.toml");
    for contents in [
        "include = 'missing.toml'\n",
        "include = 1\n",
        "include = {path = 1}\n",
        "include = {path = 'nested.toml', optional = 1}\n",
        "secret = 'private-value'\n[broken\n",
    ] {
        fs::write(&config, contents).unwrap();
        let Err(error) = observe_configurations(std::slice::from_ref(&config)) else {
            panic!("invalid configuration must fail");
        };
        assert!(!format!("{error:?}").contains("private-value"));
        assert!(!error.to_string().contains("private-value"));
    }
    assert!(observe_configurations(&[PathBuf::from("relative.toml")]).is_err());
    // Validate all include entries before attempting the first recursive read.
    fs::write(&config, "include = ['missing.toml', {path = 1}]\n").unwrap();
    assert!(matches!(
        observe_configurations(std::slice::from_ref(&config)),
        Err(ConfigurationError::Invalid { message, .. })
            if message == "Cargo configuration include table requires a string `path`"
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn configuration_error_preserves_io_source_and_path_context() {
    use std::error::Error as _;
    let source = std::io::Error::from_raw_os_error(2);
    let expected = format!("read Cargo configuration /selected/config.toml: {source}");
    let error = ConfigurationError::Io {
        operation: "read Cargo configuration",
        path: PathBuf::from("/selected/config.toml"),
        source,
    };
    assert_eq!(error.to_string(), expected);
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .raw_os_error(),
        Some(2)
    );
    let invalid = ConfigurationError::Invalid {
        path: PathBuf::from("/selected/config.toml"),
        message: "invalid include structure".into(),
    };
    assert_eq!(
        invalid.to_string(),
        "invalid Cargo configuration /selected/config.toml: invalid include structure"
    );
    assert!(invalid.source().is_none());
}
