#![cfg(unix)]
//! Real read-only command admission, saved identities and Cargo acceptance.

use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "build-release-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn run(&self, limit: usize, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ic-build-release-metadata"))
            .current_dir(&self.0)
            .args(["--max-input-bytes", &limit.to_string()])
            .args(arguments)
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("Release CLI fixture retained: {}", self.0.display());
        } else {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
}
fn write(path: impl AsRef<Path>, content: &str) {
    fs::write(path, content).unwrap();
}
const fn manifest() -> &'static str {
    "# unchanged é\n[workspace]\nmembers=['core']\n[workspace.package]\nversion=\"0.18.6\"\n[workspace.dependencies]\nrenamed={package='core',path='core',version='^0.18'}\nexternal='7.8'\n"
}
fn context(root: &Path) -> serde_json::Value {
    serde_json::json!({"source_root":root,"members":[{"name":"core","manifest_path":root.join("core/Cargo.toml")}],"manifest":{"workspace":{"dependencies":{"renamed":{"package":"core","path":"core","version":"^0.18"},"external":"7.8"}}}})
}

#[test]
fn saved_and_live_identity_select_the_same_edits_without_reopening_saved_source() {
    let fixture = Fixture::new();
    fs::create_dir_all(fixture.0.join("source/core")).unwrap();
    write(
        fixture.0.join("source/core/Cargo.toml"),
        "[package]\nname='core'\nversion='0.18.6'\n",
    );
    write(fixture.0.join("source/Cargo.toml"), manifest());
    let root = fixture.0.join("source").canonicalize().unwrap();
    let saved = context(&root);
    let mut live = saved.clone();
    live.as_object_mut().unwrap().remove("source_root");
    write(fixture.0.join("saved.json"), &saved.to_string());
    write(fixture.0.join("live.json"), &live.to_string());
    write(fixture.0.join("archive.toml"), manifest());
    let a = fixture.run(
        8192,
        &[
            "manifest",
            "source/Cargo.toml",
            "0.18.6",
            "0.19.0",
            "live.json",
        ],
    );
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    fs::rename(&root, fixture.0.join("retained")).unwrap();
    let b = fixture.run(
        8192,
        &["manifest", "archive.toml", "0.18.6", "0.19.0", "saved.json"],
    );
    assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));
    assert_eq!(a.stdout, b.stdout);
    assert_eq!(
        String::from_utf8(b.stdout).unwrap(),
        manifest()
            .replace("0.18.6", "0.19.0")
            .replace("version='^0.18'", "version=\"^0.19\"")
    );
    assert_eq!(
        fs::read_to_string(fixture.0.join("archive.toml")).unwrap(),
        manifest()
    );
    let missing = fixture.run(
        8192,
        &["manifest", "archive.toml", "0.18.6", "0.19.0", "live.json"],
    );
    assert_eq!(missing.status.code(), Some(2));
    assert_eq!(missing.stdout, [] as [u8; 0]);
}

#[test]
fn malformed_context_and_conflicting_ownership_never_publish_partial_candidates() {
    let fixture = Fixture::new();
    write(fixture.0.join("Cargo.toml"), manifest());
    let original = context(Path::new("/saved/é source"));
    let mut invalids = vec![serde_json::Value::Null];
    let mut duplicate = original.clone();
    duplicate["members"]
        .as_array_mut()
        .unwrap()
        .push(original["members"][0].clone());
    invalids.push(duplicate);
    for root in ["relative", "/saved\nroot", "/../escaped"] {
        let mut invalid = original.clone();
        invalid["source_root"] = root.into();
        invalids.push(invalid);
    }
    let mut mismatch = original.clone();
    mismatch["members"][0]["manifest_path"] = "/other/Cargo.toml".into();
    invalids.push(mismatch);
    let mut changed = original.clone();
    changed["manifest"]["workspace"]["dependencies"]["renamed"]["version"] = "^0.17".into();
    invalids.push(changed);
    for value in invalids {
        write(fixture.0.join("context.json"), &value.to_string());
        let output = fixture.run(
            8192,
            &["manifest", "Cargo.toml", "0.18.6", "0.19.0", "context.json"],
        );
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
    write(fixture.0.join("context.json"), &original.to_string());
    let valid = fixture.run(
        8192,
        &["manifest", "Cargo.toml", "0.18.6", "0.19.0", "context.json"],
    );
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    write(fixture.0.join("context.json"), &format!("{original} {{}}"));
    assert_eq!(
        fixture
            .run(
                8192,
                &["manifest", "Cargo.toml", "0.18.6", "0.19.0", "context.json"]
            )
            .stdout,
        [] as [u8; 0]
    );
}

#[test]
fn byte_limits_symlinks_directories_and_invalid_arguments_refuse_before_output() {
    let fixture = Fixture::new();
    write(fixture.0.join("Cargo.toml"), manifest());
    symlink("Cargo.toml", fixture.0.join("alias")).unwrap();
    for (limit, source) in [
        (0, "Cargo.toml"),
        (manifest().len() - 1, "Cargo.toml"),
        (8192, "alias"),
        (8192, "."),
        (8192, "missing"),
    ] {
        let output = fixture.run(limit, &["manifest", source, "0.18.6", "0.19.0"]);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
    for suffix in ['\r', '\n'] {
        let bad = fixture.0.join(format!("bad{suffix}"));
        fs::create_dir(&bad).unwrap();
        write(bad.join("Cargo.toml"), manifest());
        symlink(&bad, fixture.0.join("physical-alias")).unwrap();
        let output = fixture.run(
            8192,
            &["manifest", "physical-alias/Cargo.toml", "0.18.6", "0.19.0"],
        );
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
        fs::remove_file(fixture.0.join("physical-alias")).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_ic-build-release-metadata"))
            .current_dir(&bad)
            .args([
                "--max-input-bytes",
                "8192",
                "manifest",
                "../Cargo.toml",
                "0.18.6",
                "0.19.0",
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
    assert!(
        fixture
            .run(
                manifest().len(),
                &["manifest", "Cargo.toml", "0.18.6", "0.19.0"]
            )
            .status
            .success()
    );
    for arguments in [
        vec![],
        vec!["manifest"],
        vec!["lock", "Cargo.toml", "0.18.6", "0.19.0"],
        vec!["manifest", "Cargo.toml", "0.18.6", "bad"],
    ] {
        let output = fixture.run(8192, &arguments);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
}

#[test]
fn real_cargo_accepts_the_local_only_release_without_rewriting_the_lock() {
    let fixture = Fixture::new();
    for name in ["a", "b"] {
        fs::create_dir_all(fixture.0.join(name).join("src")).unwrap();
        write(fixture.0.join(name).join("src/lib.rs"), "");
        let dependency = if name == "a" {
            "[dependencies]\nb={path='../b'}\n"
        } else {
            ""
        };
        write(
            fixture.0.join(name).join("Cargo.toml"),
            &format!(
                "[package]\nname='{name}'\nversion.workspace=true\nedition='2024'\n{dependency}"
            ),
        );
    }
    write(
        fixture.0.join("Cargo.toml"),
        "[workspace]\nmembers=['a','b']\nresolver='3'\n[workspace.package]\nversion=\"0.1.0\"\n",
    );
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let initial = Command::new(&cargo)
        .current_dir(&fixture.0)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .args(["generate-lockfile", "--offline"])
        .output()
        .unwrap();
    assert!(
        initial.status.success(),
        "{}",
        String::from_utf8_lossy(&initial.stderr)
    );
    let rendered = fixture.run(8192, &["lock", "Cargo.lock", "0.1.0", "0.2.0", "a", "b"]);
    assert!(
        rendered.status.success(),
        "{}",
        String::from_utf8_lossy(&rendered.stderr)
    );
    fs::write(fixture.0.join("Cargo.lock"), &rendered.stdout).unwrap();
    let manifest = fixture.run(8192, &["manifest", "Cargo.toml", "0.1.0", "0.2.0"]);
    assert!(manifest.status.success());
    fs::write(fixture.0.join("Cargo.toml"), manifest.stdout).unwrap();
    let metadata = Command::new(cargo)
        .current_dir(&fixture.0)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .args(["metadata", "--locked", "--offline", "--format-version", "1"])
        .output()
        .unwrap();
    assert!(
        metadata.status.success(),
        "{}",
        String::from_utf8_lossy(&metadata.stderr)
    );
    assert_eq!(
        fs::read(fixture.0.join("Cargo.lock")).unwrap(),
        rendered.stdout
    );
}
