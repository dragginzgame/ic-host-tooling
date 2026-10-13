//! Exercise input admission and report publication through the actual executable.

use std::{
    fs, io,
    path::Path,
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

use ic_host_artifacts::artifact::Sha256Digest;

#[test]
fn input_admission_controls_exit_and_report_publication() {
    const MODULE: &[u8] = b"\0asm\x01\0\0\0";
    const STRUCTURED_MODULE: &[u8] = b"\0asm\x01\0\0\0\x01\x04\x01\x60\0\0\x03\x02\x01\0\x07\x09\x02\x01f\0\0\x01g\0\0\x0a\x04\x01\x02\0\x0b\0\x04\x01x\x07\x08";
    let identity = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/evidence/wasm-inspect-cli")
        .join(format!("{}-{}", process::id(), identity.as_nanos()));
    fs::create_dir_all(root.parent().unwrap()).unwrap();
    fs::create_dir(&root).unwrap();
    let input = root.join("input with spaces.wasm");
    fs::write(&input, MODULE).unwrap();
    let inspect = |path: &Path, budgets: [&str; 4]| {
        Command::new(env!("CARGO_BIN_EXE_ic-host-wasm-inspect"))
            .arg(path)
            .args(budgets)
            .output()
            .unwrap()
    };

    let accepted = inspect(&input, ["8", "0", "0", "0"]);
    assert!(accepted.status.success());
    assert_eq!(accepted.stderr.as_slice(), b"");
    let text = String::from_utf8(accepted.stdout).unwrap();
    let rows: Vec<_> = text.lines().collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[1],
        format!(
            "{}\t8\t0\t0\t0\t0\t0\t0\t0\t0\t0\t0",
            Sha256Digest::compute(MODULE)
        )
    );

    let malformed = root.join("malformed.wasm");
    fs::write(&malformed, b"not Wasm").unwrap();
    let missing = root.join("missing.wasm");
    for (path, max_bytes) in [
        (&input, "7"),
        (&malformed, "8"),
        (&missing, "8"),
        (&root, "8"),
    ] {
        let rejected = inspect(path, [max_bytes, "0", "0", "0"]);
        assert_eq!(rejected.status.code(), Some(1));
        assert_eq!(rejected.stdout.as_slice(), b"");
        assert_ne!(rejected.stderr.as_slice(), b"");
    }

    // Distinct nonzero limits expose argument swaps or omitted budget forwarding.
    let structured = root.join("structured.wasm");
    fs::write(&structured, STRUCTURED_MODULE).unwrap();
    let accepted = inspect(&structured, ["41", "5", "2", "1"]);
    assert!(accepted.status.success());
    assert_eq!(accepted.stderr.as_slice(), b"");
    let text = String::from_utf8(accepted.stdout).unwrap();
    assert_eq!(
        text.lines().nth(1).unwrap(),
        format!(
            "{}\t41\t4\t3\t0\t1\t0\t0\t0\t0\t2\t1",
            Sha256Digest::compute(STRUCTURED_MODULE)
        )
    );
    for budgets in [
        ["40", "5", "2", "1"],
        ["41", "4", "2", "1"],
        ["41", "5", "1", "1"],
        ["41", "5", "2", "0"],
    ] {
        let rejected = inspect(&structured, budgets);
        assert_eq!(rejected.status.code(), Some(1));
        assert_eq!(rejected.stdout.as_slice(), b"");
        assert_ne!(rejected.stderr.as_slice(), b"");
    }

    // Use a valid input so argument admission cannot pass via a file-read failure.
    for arguments in [
        vec![],
        vec!["8", "0", "0"],
        vec!["-1", "0", "0", "0"],
        vec!["8", "-1", "0", "0"],
        vec!["8", "0", "4294967296", "0"],
        vec!["8", "0", "0", "not-a-number"],
        vec!["8", "0", "0", "0", "extra"],
    ] {
        let rejected = Command::new(env!("CARGO_BIN_EXE_ic-host-wasm-inspect"))
            .arg(&input)
            .args(arguments)
            .output()
            .unwrap();
        assert_eq!(rejected.status.code(), Some(1));
        assert_eq!(rejected.stdout.as_slice(), b"");
        assert_ne!(rejected.stderr.as_slice(), b"");
    }

    // Close every reader before launch, making the output failure deterministic.
    let (reader, writer) = io::pipe().unwrap();
    drop(reader);
    let failed_output = Command::new(env!("CARGO_BIN_EXE_ic-host-wasm-inspect"))
        .arg(&input)
        .args(["8", "0", "0", "0"])
        .stdout(writer)
        .output()
        .unwrap();
    assert_eq!(failed_output.status.code(), Some(1));
    assert_ne!(failed_output.stderr.as_slice(), b"");

    // Admission and reporting must leave both accepted and refused inputs intact.
    assert_eq!(fs::read(&input).unwrap(), MODULE);
    assert_eq!(fs::read(&malformed).unwrap(), b"not Wasm");
    assert_eq!(fs::read(&structured).unwrap(), STRUCTURED_MODULE);
    assert!(!missing.exists());
    // Retain invocation-owned inputs under target/evidence on success or failure.
}
