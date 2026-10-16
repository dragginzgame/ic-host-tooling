//! Structural facts and exact input identity, without IC cost estimates.

use std::io::{self, Write};

use ic_host_artifacts::{
    artifact::{ArtifactIdentity, Sha256Digest},
    wasm::{InspectionLimits, WasmFacts, inspect},
};

use crate::Error;

pub fn write(
    bytes: &[u8],
    limits: InspectionLimits,
    expected: Option<Sha256Digest>,
    details: bool,
    output: &mut impl Write,
) -> Result<(), Error> {
    // Complete admission before publishing a header or an observed fact.
    let identity = ArtifactIdentity {
        bytes: bytes.len() as u64,
        sha256: Sha256Digest::compute(bytes),
    };
    if let Some(expected) = expected {
        identity.verify(expected).map_err(Error::Input)?;
    }
    let facts = inspect(bytes, limits).map_err(Error::Inspection)?;
    render(output, identity.sha256, facts, details).map_err(Error::Output)
}

fn render(
    output: &mut impl Write,
    digest: Sha256Digest,
    facts: WasmFacts<'_>,
    details: bool,
) -> io::Result<()> {
    writeln!(
        output,
        "sha256\traw_wasm_bytes\tcode_section_bytes\tcode_body_bytes\tdata_section_bytes\tdefined_functions\timported_functions\tdefined_globals\timported_globals\tdata_segments\texports\tcustom_sections"
    )?;
    writeln!(
        output,
        "{digest}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        facts.raw_bytes,
        facts.code_section_bytes,
        facts.code_body_bytes,
        facts.data_section_bytes,
        facts.defined_functions,
        facts.imported_functions,
        facts.defined_globals,
        facts.imported_globals,
        facts.data_segments,
        facts.exports.len(),
        facts.custom_sections.len()
    )?;
    if details {
        for (name, export) in facts.exports {
            writeln!(
                output,
                "export={name:?} kind={:?} index={}",
                export.kind, export.index
            )?;
        }
        for section in facts.custom_sections {
            writeln!(
                output,
                "custom_section={:?} bytes={}",
                section.name,
                section.data.len()
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_host_artifacts::wasm::{InspectionError, InspectionResource};
    use std::io;

    const MODULE: &[u8] = b"\0asm\x01\0\0\0\x01\x04\x01\x60\0\0\x03\x02\x01\0\x07\x05\x01\x01f\0\0\x0a\x04\x01\x02\0\x0b\0\x04\x01x\x07\x08";
    const LIMITS: InspectionLimits = InspectionLimits {
        module_bytes: 100,
        sections: 10,
        exports: 1,
        custom_sections: 1,
    };

    #[test]
    fn reports_raw_and_structural_units_separately() {
        let mut output = Vec::new();
        write(MODULE, LIMITS, None, false, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        let rows: Vec<_> = text.lines().collect();
        assert_eq!(rows.len(), 2);
        let fields: Vec<_> = rows[1].split('\t').collect();
        assert_eq!(fields.len(), 12);
        assert_eq!(
            &fields[1..],
            ["37", "4", "3", "0", "1", "0", "0", "0", "0", "1", "1"]
        );
    }

    #[test]
    fn malformed_and_over_budget_inputs_publish_no_report() {
        let mut output = Vec::new();
        assert!(matches!(
            write(
                MODULE,
                LIMITS,
                Some(Sha256Digest::compute(b"different")),
                true,
                &mut output
            ),
            Err(Error::Input(
                ic_host_artifacts::artifact::ArtifactError::DigestMismatch { .. }
            ))
        ));
        assert_eq!(output.as_slice(), b"");
        assert!(matches!(
            write(b"not Wasm", LIMITS, None, false, &mut output),
            Err(Error::Inspection(_))
        ));
        assert_eq!(output.as_slice(), b"");
        for limits in [
            InspectionLimits {
                module_bytes: 36,
                ..LIMITS
            },
            InspectionLimits {
                custom_sections: 0,
                ..LIMITS
            },
            InspectionLimits {
                exports: 0,
                ..LIMITS
            },
        ] {
            assert!(matches!(
                write(MODULE, limits, None, false, &mut output),
                Err(Error::Inspection(InspectionError::LimitExceeded {
                    resource: InspectionResource::ModuleBytes
                        | InspectionResource::CustomSections
                        | InspectionResource::Exports,
                    ..
                }))
            ));
            assert_eq!(output.as_slice(), b"");
        }
    }

    #[test]
    fn output_failures_are_typed() {
        struct FailedOutput;
        impl Write for FailedOutput {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "closed fixture output",
                ))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        assert!(
            matches!(write(MODULE, LIMITS, None, false, &mut FailedOutput), Err(Error::Output(error)) if error.kind() == io::ErrorKind::BrokenPipe)
        );
    }
}
