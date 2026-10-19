//! Explicit input and inspection budgets; there are no implicit artifact paths.

use ic_host_artifacts::artifact::Sha256Digest;
use std::path::PathBuf;

#[derive(clap::Parser)]
#[command(
    name = "ic-host-wasm-inspect",
    version,
    about = "Inspect raw Wasm without executing or transforming it"
)]
pub struct Command {
    pub path: PathBuf,
    #[arg(value_name = "MAX_BYTES")]
    pub module_bytes: usize,
    #[arg(value_name = "MAX_SECTIONS")]
    pub sections: usize,
    #[arg(value_name = "MAX_EXPORTS")]
    pub exports: u32,
    #[arg(value_name = "MAX_CUSTOM_SECTIONS")]
    pub custom_sections: usize,
    #[arg(long = "expect-sha256", value_name = "SHA256")]
    pub expected: Option<Sha256Digest>,
    #[arg(long)]
    pub details: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Parser as _, error::ErrorKind};
    use std::ffi::OsString;

    fn arguments(values: &[&str]) -> impl Iterator<Item = OsString> {
        std::iter::once(OsString::from("ic-host-wasm-inspect"))
            .chain(values.iter().map(OsString::from))
    }

    #[test]
    fn exact_budgets_and_input_are_required() {
        let command =
            Command::try_parse_from(arguments(&["input.wasm", "100", "20", "0", "0"])).unwrap();
        assert_eq!(command.path, PathBuf::from("input.wasm"));
        assert_eq!(command.module_bytes, 100);
        assert_eq!(command.exports, 0);
        assert_eq!(command.expected, None);
        assert!(!command.details);
        for values in [&[][..], &["input.wasm", "100", "20", "0"][..]] {
            assert_eq!(
                Command::try_parse_from(arguments(values))
                    .err()
                    .unwrap()
                    .kind(),
                ErrorKind::MissingRequiredArgument
            );
        }
    }

    #[test]
    fn invalid_or_surplus_limits_are_rejected() {
        for exports in ["-1", "4294967296", "not-a-number"] {
            assert!(
                Command::try_parse_from(arguments(&["input.wasm", "100", "20", exports, "0"]))
                    .is_err()
            );
        }
        assert_eq!(
            Command::try_parse_from(arguments(&["input.wasm", "100", "20", "0", "0", "extra"]))
                .err()
                .unwrap()
                .kind(),
            ErrorKind::UnknownArgument
        );
        assert_eq!(
            Command::try_parse_from(arguments(&["--help"]))
                .err()
                .unwrap()
                .kind(),
            ErrorKind::DisplayHelp
        );
        assert!(
            Command::try_parse_from(arguments(&[
                "input.wasm",
                "100",
                "20",
                "0",
                "0",
                "--details",
                "--details"
            ]))
            .is_err()
        );
    }
}
