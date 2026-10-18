//! Explicit input and inspection budgets; there are no implicit artifact paths.

use std::{ffi::OsString, num::ParseIntError, path::PathBuf, str::FromStr};

use ic_host_artifacts::{artifact::Sha256Digest, wasm::InspectionLimits};

use crate::Error;

pub const USAGE: &str = "usage: ic-host-wasm-inspect PATH MAX_BYTES MAX_SECTIONS MAX_EXPORTS MAX_CUSTOM_SECTIONS [--expect-sha256 SHA256] [--details]";

pub enum Command {
    Help,
    Inspect {
        path: PathBuf,
        limits: InspectionLimits,
        expected: Option<Sha256Digest>,
        details: bool,
    },
}

pub fn parse(mut args: impl Iterator<Item = OsString>) -> Result<Command, Error> {
    let path = args.next().ok_or(Error::MissingArgument("PATH"))?;
    if path == "--help" {
        return if args.next().is_none() {
            Ok(Command::Help)
        } else {
            Err(Error::ExtraArgument)
        };
    }
    let limits = InspectionLimits {
        module_bytes: number(args.next(), "MAX_BYTES")?,
        sections: number(args.next(), "MAX_SECTIONS")?,
        exports: number(args.next(), "MAX_EXPORTS")?,
        custom_sections: number(args.next(), "MAX_CUSTOM_SECTIONS")?,
    };
    let mut expected = None;
    let mut details = false;
    while let Some(option) = args.next() {
        if option == "--details" && !details {
            details = true;
        } else if option == "--expect-sha256" && expected.is_none() {
            let digest = args.next().ok_or(Error::MissingArgument("SHA256"))?;
            expected = Some(
                digest
                    .to_str()
                    .ok_or(Error::NonUtf8Digest)?
                    .parse()
                    .map_err(Error::InvalidDigest)?,
            );
        } else {
            return Err(Error::ExtraArgument);
        }
    }
    Ok(Command::Inspect {
        path: path.into(),
        limits,
        expected,
        details,
    })
}

fn number<T: FromStr<Err = ParseIntError>>(
    value: Option<OsString>,
    name: &'static str,
) -> Result<T, Error> {
    let value = value.ok_or(Error::MissingArgument(name))?;
    let value = value.to_str().ok_or(Error::NonUtf8Number(name))?;
    value
        .parse()
        .map_err(|source| Error::InvalidNumber { name, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(values: &[&str]) -> impl Iterator<Item = OsString> {
        values.iter().map(OsString::from)
    }

    #[test]
    fn exact_budgets_and_input_are_required() {
        let Command::Inspect {
            path,
            limits,
            expected,
            details,
        } = parse(arguments(&["input.wasm", "100", "20", "0", "0"])).unwrap()
        else {
            panic!("expected inspection");
        };
        assert_eq!(path, PathBuf::from("input.wasm"));
        assert_eq!(limits.module_bytes, 100);
        assert_eq!(limits.exports, 0);
        assert_eq!(expected, None);
        assert!(!details);
        assert!(matches!(
            parse(arguments(&[])),
            Err(Error::MissingArgument("PATH"))
        ));
        assert!(matches!(
            parse(arguments(&["input.wasm", "100", "20", "0"])),
            Err(Error::MissingArgument("MAX_CUSTOM_SECTIONS"))
        ));
    }

    #[test]
    fn invalid_or_surplus_limits_are_rejected() {
        for exports in ["-1", "4294967296", "not-a-number"] {
            assert!(matches!(
                parse(arguments(&["input.wasm", "100", "20", exports, "0"])),
                Err(Error::InvalidNumber {
                    name: "MAX_EXPORTS",
                    ..
                })
            ));
        }
        assert!(matches!(
            parse(arguments(&["input.wasm", "100", "20", "0", "0", "extra"])),
            Err(Error::ExtraArgument)
        ));
        assert!(matches!(parse(arguments(&["--help"])), Ok(Command::Help)));
    }
}
