//! Compose bounded Wasm facts with explicitly selected IC reference comparisons.
//!
//! This example accepts trusted local regular files. Raw-byte admission and the
//! soft warning are caller policy; the three reference comparisons do not prove
//! instruction/type validity or installation on any deployed subnet.

use ic_host_artifacts::{
    artifact::read_reader,
    wasm::{InspectionLimits, WasmFacts, inspect},
};
use ic_host_tools::install_limits::{
    InstallReport, LimitUsage, REFERENCE_LIMITS, REFERENCE_REVISION,
};
use std::{error::Error, ffi::OsString, fs::File, io, process::ExitCode};

#[cfg(test)]
mod tests;

fn main() -> Result<ExitCode, Box<dyn Error>> {
    run(std::env::args_os().skip(1), &mut io::stdout().lock())
}

fn run(
    mut args: impl Iterator<Item = OsString>,
    output: &mut impl io::Write,
) -> Result<ExitCode, Box<dyn Error>> {
    let path = args.next().ok_or_else(|| invalid_argument(
        "usage: inspect_install_limits PATH MAX_BYTES MAX_SECTIONS MAX_EXPORTS MAX_CUSTOM_SECTIONS SOFT_RAW_BYTES [--json]",
    ))?;
    let mut number = || -> Result<usize, Box<dyn Error>> {
        Ok(args
            .next()
            .ok_or_else(|| invalid_argument("missing limit"))?
            .to_str()
            .ok_or_else(|| invalid_argument("limits must be UTF-8"))?
            .parse()?)
    };
    let limits = InspectionLimits {
        module_bytes: number()?,
        sections: number()?,
        exports: u32::try_from(number()?)?,
        custom_sections: number()?,
    };
    let soft_raw_bytes = number()?;
    let json = match args.next() {
        None => false,
        Some(value) if value == "--json" => true,
        Some(_) => return Err(invalid_argument("expected --json").into()),
    };
    if args.next().is_some() {
        return Err(invalid_argument("unexpected argument").into());
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(invalid_argument("expected a regular file").into());
    }
    let bytes = read_reader(file, limits.module_bytes)?;
    let facts = inspect(&bytes, limits)?;
    // Select the revision-bound reference explicitly, and inspect only once.
    let report = REFERENCE_LIMITS.report(&facts);
    let accepted = within_limits(report);
    render(output, &facts, report, soft_raw_bytes, accepted, json)?;
    Ok(if accepted {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn within_limits(report: InstallReport) -> bool {
    ![
        report.code_body_bytes,
        report.defined_functions,
        report.globals,
    ]
    .into_iter()
    .any(LimitUsage::exceeded)
}

fn render(
    output: &mut impl io::Write,
    facts: &WasmFacts<'_>,
    report: InstallReport,
    soft_raw_bytes: usize,
    accepted: bool,
    json: bool,
) -> io::Result<()> {
    let raw_warning = facts.raw_bytes > soft_raw_bytes;
    if json {
        serde_json::to_writer(
            &mut *output,
            &serde_json::json!({
                "reference_revision": REFERENCE_REVISION,
                "within_selected_limits": accepted,
                "raw_bytes": facts.raw_bytes,
                "soft_raw_bytes": soft_raw_bytes,
                "raw_warning": raw_warning,
                "code_section_bytes": facts.code_section_bytes,
                "code_body_bytes": { "observed": report.code_body_bytes.observed, "limit": report.code_body_bytes.limit },
                "defined_functions": { "observed": report.defined_functions.observed, "limit": report.defined_functions.limit },
                "imported_functions": facts.imported_functions,
                "defined_globals": facts.defined_globals,
                "imported_globals": facts.imported_globals,
                "globals": { "observed": report.globals.observed, "limit": report.globals.limit },
            }),
        )?;
        writeln!(output)
    } else {
        writeln!(
            output,
            "reference_revision={REFERENCE_REVISION} within_selected_limits={accepted}"
        )?;
        writeln!(
            output,
            "raw_bytes={} soft_raw_bytes={soft_raw_bytes} raw_warning={raw_warning}",
            facts.raw_bytes
        )?;
        writeln!(
            output,
            "code_section_bytes={} (diagnostic)",
            facts.code_section_bytes
        )?;
        writeln!(
            output,
            "code_body_bytes={}/{} defined_functions={}/{} imported_functions={}",
            report.code_body_bytes.observed,
            report.code_body_bytes.limit,
            report.defined_functions.observed,
            report.defined_functions.limit,
            facts.imported_functions
        )?;
        writeln!(
            output,
            "globals={}/{} defined_globals={} imported_globals={}",
            report.globals.observed,
            report.globals.limit,
            facts.defined_globals,
            facts.imported_globals
        )
    }
}

fn invalid_argument(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
