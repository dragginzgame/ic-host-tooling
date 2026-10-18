use std::{
    ffi::{OsStr, OsString},
    path::Path,
    process::Command,
};

use crate::icp::{
    CallArguments, CallMode, InstallationStatusError, StatusOutput, call_command,
    decode_installation_status, status_command,
};

#[test]
fn text_calls_keep_native_arguments_and_caller_context() {
    let mut base = Command::new("selected icp");
    base.arg("--identity-password-file")
        .arg("/private/password")
        .current_dir("/workspace")
        .env("CALLER_SETTING", "kept");
    let mut query = call_command(
        base,
        OsStr::new("actor"),
        OsStr::new("read"),
        CallArguments::Text(OsStr::new("(\"two words\")")),
        CallMode::Query,
    );
    query.args(["--output", "hex", "--environment", "chosen"]);
    assert_eq!(query.get_program(), OsStr::new("selected icp"));
    assert_eq!(query.get_current_dir(), Some(Path::new("/workspace")));
    assert!(
        query
            .get_envs()
            .any(|(key, value)| key == "CALLER_SETTING" && value == Some(OsStr::new("kept")))
    );
    let args: Vec<_> = query.get_args().collect();
    assert_eq!(
        args,
        [
            "--identity-password-file",
            "/private/password",
            "canister",
            "call",
            "actor",
            "read",
            "(\"two words\")",
            "--query",
            "--output",
            "hex",
            "--environment",
            "chosen"
        ]
        .map(OsStr::new)
    );
    let update = call_command(
        Command::new("icp"),
        OsStr::new("actor"),
        OsStr::new("write"),
        CallArguments::Text(OsStr::new("()")),
        CallMode::Update,
    );
    assert_eq!(
        update.get_args().collect::<Vec<_>>(),
        ["canister", "call", "actor", "write", "()"].map(OsStr::new)
    );
}

#[test]
fn binary_calls_and_status_select_only_the_requested_protocol() {
    for mode in [CallMode::Query, CallMode::Update] {
        let command = call_command(
            Command::new("icp"),
            OsStr::new("actor"),
            OsStr::new("method"),
            CallArguments::BinaryFile(Path::new("/args with spaces.bin")),
            mode,
        );
        let mut expected = vec![
            "canister",
            "call",
            "actor",
            "method",
            "--args-file",
            "/args with spaces.bin",
            "--args-format",
            "bin",
        ];
        if mode == CallMode::Query {
            expected.push("--query");
        }
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            expected.iter().map(OsStr::new).collect::<Vec<_>>()
        );
    }
    for (output, suffix) in [
        (StatusOutput::Text, None),
        (StatusOutput::Json, Some("--json")),
        (StatusOutput::IdOnly, Some("--id-only")),
    ] {
        let command = status_command(Command::new("icp"), OsStr::new("actor"), output);
        let mut expected = vec![
            OsString::from("canister"),
            OsString::from("status"),
            OsString::from("actor"),
        ];
        expected.extend(suffix.map(OsString::from));
        assert_eq!(
            command
                .get_args()
                .map(OsStr::to_os_string)
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[cfg(unix)]
#[test]
fn argument_bytes_are_not_lossily_converted() {
    use std::os::unix::ffi::OsStrExt;
    let argument = OsStr::from_bytes(b"(\"\xff\")");
    let command = call_command(
        Command::new("icp"),
        OsStr::new("actor"),
        OsStr::new("method"),
        CallArguments::Text(argument),
        CallMode::Update,
    );
    assert_eq!(command.get_args().last(), Some(argument));
}

#[test]
fn successful_status_null_and_string_are_the_only_installation_facts() {
    assert!(!decode_installation_status(br#"{"module_hash":null,"other":"ignored"}"#).unwrap());
    assert!(decode_installation_status(br#"{"module_hash":"hash"}"#).unwrap());
    assert!(decode_installation_status(br#"{"module_hash":""}"#).unwrap());
    assert!(matches!(
        decode_installation_status(b"{}"),
        Err(InstallationStatusError::MissingModuleHash)
    ));
    for input in [
        br#"{"module_hash":false}"#.as_slice(),
        br#"{"module_hash":23}"#,
        br#"{"module_hash":[]}"#,
        br#"{"module_hash":{}}"#,
    ] {
        assert!(matches!(
            decode_installation_status(input),
            Err(InstallationStatusError::InvalidModuleHash)
        ));
    }
    for input in [b"{".as_slice(), b"{} trailing", b"\xff"] {
        assert!(matches!(
            decode_installation_status(input),
            Err(InstallationStatusError::Json(_))
        ));
    }
    assert!(matches!(
        decode_installation_status(b"null"),
        Err(InstallationStatusError::MissingModuleHash)
    ));
}
