//! Actual CLI contracts extracted from Auth; caller policies stay outside Host.
#![cfg(all(unix, feature = "test-support"))]

use ic_host_fs::testing::FixtureDirectory;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn fixture() -> FixtureDirectory {
    FixtureDirectory::new(&std::env::temp_dir(), "ic-host-file-test").unwrap()
}

fn invoke(root: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ic-host-file"))
        .current_dir(root)
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn hash_stdout_is_exact_and_oversize_has_no_identity() {
    let fixture = fixture();
    let root = fixture.path();
    fs::write(root.join("input"), b"abc").unwrap();
    let accepted = invoke(root, &["hash-file", "input", "3"]);
    assert!(accepted.status.success());
    assert_eq!(
        accepted.stdout,
        b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n"
    );
    let rejected = invoke(root, &["hash-file", "input", "2"]);
    assert!(!rejected.status.success());
    assert_eq!(rejected.stdout, b"");
    assert!(!invoke(root, &["hash-file", ".", "3"]).status.success());
}

#[cfg(unix)]
fn batch(root: &Path, input: &[u8], maximum: &str) -> Output {
    use std::{io::Write, process::Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_ic-host-file"))
        .current_dir(root)
        .args(["hash-files0", maximum, "4194304", "4096"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[cfg(unix)]
#[test]
fn batch_hashing_preserves_raw_paths_order_duplicates_and_zero_limits() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
    let fixture = fixture();
    let root = fixture.path();
    let odd = b"space tab\tnewline\nutf8-\xc3\xa9";
    fs::write(root.join(OsStr::from_bytes(odd)), b"abc").unwrap();
    fs::write(root.join("plain"), b"abc").unwrap();
    let paths: Vec<&[u8]> = vec![odd.as_slice(), b"plain", odd.as_slice()];
    // macOS rejects invalid UTF-8 names before the hashing protocol runs.
    // Keep raw filesystem coverage on Linux and shared path/ordering coverage
    // on every supported Unix host.
    #[cfg(target_os = "linux")]
    let paths = {
        let non_utf8 = b"nonutf8\xff";
        fs::write(root.join(OsStr::from_bytes(non_utf8)), b"abc").unwrap();
        let mut paths = paths;
        paths.push(non_utf8);
        paths
    };
    let input: Vec<_> = paths
        .iter()
        .flat_map(|path| path.iter().copied().chain([0]))
        .collect();
    let result = batch(root, &input, "3");
    assert!(result.status.success(), "{result:?}");
    let mut expected = Vec::new();
    for path in paths {
        expected.extend_from_slice(
            b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\t",
        );
        expected.extend_from_slice(path);
        expected.push(0);
    }
    assert_eq!(result.stdout, expected);
    fs::write(root.join("empty"), []).unwrap();
    assert_eq!(
        batch(root, b"empty\0", "0").stdout,
        b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\tempty\0"
    );
    assert!(batch(root, b"", "0").status.success());
}

#[cfg(unix)]
#[test]
fn rejected_hash_batches_emit_no_partial_seal() {
    use std::os::unix::fs::symlink;
    let fixture = fixture();
    let root = fixture.path();
    fs::write(root.join("ok"), b"abc").unwrap();
    fs::write(root.join("big"), b"abcd").unwrap();
    symlink("ok", root.join("link")).unwrap();
    for input in [
        b"ok\0missing\0".to_vec(),
        b"ok\0missing\xff\0".to_vec(),
        b"ok\0big\0".to_vec(),
        b"ok\0link\0".to_vec(),
        b"ok\0.\0".to_vec(),
        b"ok\0\0".to_vec(),
        b"ok".to_vec(),
        b"ok\0".repeat(4097),
        vec![b'x'; 4 * 1024 * 1024 + 1],
    ] {
        let result = batch(root, &input, "3");
        assert!(!result.status.success());
        assert_eq!(result.stdout, b"");
    }
}

#[test]
fn empty_file_hash_accepts_zero_limit_without_admitting_nonempty_bytes() {
    let fixture = fixture();
    let root = fixture.path();
    fs::write(root.join("empty"), b"").unwrap();
    let accepted = invoke(root, &["hash-file", "empty", "0"]);
    assert!(accepted.status.success());
    assert_eq!(
        accepted.stdout,
        b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n"
    );
    fs::write(root.join("one-byte"), b"x").unwrap();
    let rejected = invoke(root, &["hash-file", "one-byte", "0"]);
    assert!(!rejected.status.success());
    assert_eq!(rejected.stdout, b"");
}

#[test]
fn intent_creation_never_replaces_saved_bytes() {
    let fixture = fixture();
    let root = fixture.path();
    fs::write(root.join("input"), b"original intent").unwrap();
    assert!(
        invoke(root, &["create-private", "input", "state/intent", "1024"])
            .status
            .success()
    );
    fs::write(root.join("input"), b"conflicting intent").unwrap();
    assert!(
        !invoke(root, &["create-private", "input", "state/intent", "1024"])
            .status
            .success()
    );
    assert_eq!(
        fs::read(root.join("state/intent")).unwrap(),
        b"original intent"
    );
}

#[test]
fn receipt_replacement_is_bounded_and_private() {
    let fixture = fixture();
    let root = fixture.path();
    fs::write(root.join("input"), b"validated receipt").unwrap();
    fs::write(root.join("receipt"), b"old").unwrap();
    assert!(
        !invoke(root, &["replace-private", "input", "receipt", "1"])
            .status
            .success()
    );
    assert_eq!(fs::read(root.join("receipt")).unwrap(), b"old");
    assert!(
        invoke(root, &["replace-private", "input", "receipt", "1024"])
            .status
            .success()
    );
    assert_eq!(
        fs::read(root.join("receipt")).unwrap(),
        b"validated receipt"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(root.join("receipt"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

#[test]
fn directory_required_targets_never_publish_or_replace_stripped_names() {
    let fixture = fixture();
    let root = fixture.path();
    fs::write(root.join("input"), b"new receipt").unwrap();
    for operation in ["create-private", "replace-private"] {
        for suffix in ["/", "/.", "//"] {
            fs::write(root.join("receipt"), b"saved receipt").unwrap();
            for destination in [format!("receipt{suffix}"), format!("new/receipt{suffix}")] {
                let refused = invoke(root, &[operation, "input", &destination, "1024"]);
                assert!(!refused.status.success(), "{operation} {destination}");
                assert_eq!(refused.stdout, b"");
                assert_eq!(fs::read(root.join("receipt")).unwrap(), b"saved receipt");
                assert!(!root.join("new").exists());
            }
        }
    }
}

#[cfg(unix)]
#[test]
fn symlink_inputs_and_existing_intent_links_are_refused() {
    use std::os::unix::fs::symlink;
    let fixture = fixture();
    let root = fixture.path();
    fs::write(root.join("input"), b"protected bytes").unwrap();
    symlink("input", root.join("link")).unwrap();
    assert!(
        !invoke(root, &["hash-file", "link", "1024"])
            .status
            .success()
    );
    assert!(
        !invoke(root, &["create-private", "link", "new", "1024"])
            .status
            .success()
    );
    assert!(!root.join("new").exists());
    assert!(
        !invoke(root, &["create-private", "input", "link", "1024"])
            .status
            .success()
    );
    assert!(
        fs::symlink_metadata(root.join("link"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read(root.join("input")).unwrap(), b"protected bytes");
}

#[test]
fn arguments_and_version_are_explicit_before_filesystem_effects() {
    let fixture = fixture();
    let root = fixture.path();
    let version = invoke(root, &["--version"]);
    assert!(version.status.success());
    assert_eq!(
        version.stdout,
        concat!("ic-host-file ", env!("CARGO_PKG_VERSION"), "\n").as_bytes()
    );
    let help = invoke(root, &["--help"]);
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    for command in [
        "hash-file",
        "hash-files0",
        "create-private",
        "replace-private",
    ] {
        assert!(help.contains(command));
    }
    for arguments in [
        vec![],
        vec!["unknown"],
        vec!["hash-file", "missing"],
        vec!["hash-file", "missing", "-1"],
        vec!["hash-file", "missing", "3", "extra"],
        vec!["hash-files0", "3"],
        vec!["hash-files0", "3", "-1", "1"],
        vec!["hash-files0", "3", "1", "-1"],
        vec!["create-private", "missing", "new/output", "bad"],
    ] {
        let refused = invoke(root, &arguments);
        assert!(!refused.status.success(), "{arguments:?}");
        assert_eq!(refused.stdout, b"");
        assert!(!root.join("new").exists());
    }
}

#[test]
fn batch_limits_are_caller_selected_and_exact() {
    use std::{io::Write as _, process::Stdio};
    let fixture = fixture();
    fs::write(fixture.path().join("a"), b"abc").unwrap();
    for (input, list_bytes, count, accepted) in [
        (&b""[..], "0", "0", true),
        (&b"a\0"[..], "2", "1", true),
        (&b"a\0"[..], "1", "1", false),
        (&b"a\0"[..], "2", "0", false),
        (&b"a\0a\0"[..], "4", "1", false),
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ic-host-file"))
            .current_dir(fixture.path())
            .args(["hash-files0", "3", list_bytes, count])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input).unwrap();
        let result = child.wait_with_output().unwrap();
        assert_eq!(result.status.success(), accepted, "{result:?}");
        if !accepted {
            assert_eq!(result.stdout, b"");
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn single_file_commands_preserve_non_utf8_paths() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt as _};
    let fixture = fixture();
    let source = OsStr::from_bytes(b"input\xff");
    let destination = OsStr::from_bytes(b"output\xfe");
    fs::write(fixture.path().join(source), b"abc").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ic-host-file"))
        .current_dir(fixture.path())
        .arg("create-private")
        .arg(source)
        .arg(destination)
        .arg("3")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(fs::read(fixture.path().join(destination)).unwrap(), b"abc");
    let output = Command::new(env!("CARGO_BIN_EXE_ic-host-file"))
        .current_dir(fixture.path())
        .arg("hash-file")
        .arg(source)
        .arg("3")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n"
    );
}
