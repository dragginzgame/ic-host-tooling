//! Borrowed structural facts and bounded custom-section framing for WebAssembly.
//!
//! Inspection delegates framing, section ordering, names and vector decoding
//! to `wasmparser`. Custom-section integers are encoded by `leb128` after admission.
//! Inspection checks the function/data/export structures it reports, but is not
//! instruction validation, type checking, feature admission, or IC install policy.
//! Facts borrow source bytes; no metadata or export names are copied.

use std::{collections::BTreeMap, fmt};
use wasmparser::{Encoding, ExternalKind, Parser, Payload, TypeRef};

#[cfg(test)]
mod tests;

/// Consumer-selected resource bounds; zero allows only an empty collection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InspectionLimits {
    /// Maximum complete module size in bytes.
    pub module_bytes: usize,
    /// Maximum number of top-level sections, including custom sections.
    pub sections: usize,
    /// Maximum number of export entries.
    pub exports: u32,
    /// Maximum number of custom sections retained as borrowed views.
    pub custom_sections: usize,
}

/// Which consumer-supplied bound rejected an artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectionResource {
    /// Complete source byte length.
    ModuleBytes,
    /// Number of top-level sections.
    Sections,
    /// Number of exported items.
    Exports,
    /// Number of custom metadata sections.
    CustomSections,
}

/// A malformed Wasm structure, independent of the parser dependency's API.
///
/// The parser and its diagnostic metadata remain private. Use [`Self::offset`]
/// for the source position and [`Self::message`] for a human-readable diagnostic.
#[derive(Debug)]
pub struct ParseError {
    source: wasmparser::BinaryReaderError,
}

impl ParseError {
    /// Byte offset in the original Wasm input where parsing failed.
    #[must_use]
    pub fn offset(&self) -> u64 {
        self.source.offset()
    }

    /// Human-readable parser diagnostic, without the formatted offset.
    ///
    /// Diagnostic wording may change; it is not a machine-readable category.
    #[must_use]
    pub fn message(&self) -> &str {
        self.source.message()
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.source, f)
    }
}

impl std::error::Error for ParseError {}

/// A structural inspection failed without yielding partial facts.
#[derive(Debug)]
pub enum InspectionError {
    /// A parser offset or extent cannot be represented by the host address space.
    UnrepresentableSize {
        /// Original parser value, without misclassifying it as a caller byte limit.
        value: u64,
    },
    /// Imports exceed the representable Wasm index-space count.
    CountOverflow {
        /// Index space being counted.
        kind: ExportKind,
    },
    /// A caller-supplied resource bound was exceeded.
    LimitExceeded {
        /// Resource being counted.
        resource: InspectionResource,
        /// Observed count or byte length.
        actual: u64,
        /// Caller-selected maximum.
        limit: u64,
    },
    /// A component was supplied instead of a core module.
    UnsupportedEncoding,
    /// Framing, section ordering, or an inspected vector is malformed.
    Parse(ParseError),
    /// Export names must be unique regardless of export kind.
    DuplicateExport {
        /// Offset of the duplicate entry in the original source.
        offset: usize,
    },
    /// An unknown core section cannot be interpreted as artifact evidence.
    UnknownSection {
        /// Binary section identifier.
        id: u8,
        /// Offset of the section payload.
        offset: usize,
    },
}

impl fmt::Display for InspectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnrepresentableSize { value } => write!(
                f,
                "Wasm offset or extent {value} exceeds the host address space"
            ),
            Self::CountOverflow { kind } => write!(f, "Wasm {kind:?} import count exceeds u32"),
            Self::LimitExceeded {
                resource,
                actual,
                limit,
            } => write!(f, "Wasm {resource:?} count {actual} exceeds {limit}"),
            Self::UnsupportedEncoding => {
                f.write_str("expected a core Wasm module, received a component")
            }
            Self::Parse(source) => write!(f, "malformed Wasm structure: {source}"),
            Self::DuplicateExport { offset } => write!(f, "duplicate Wasm export at byte {offset}"),
            Self::UnknownSection { id, offset } => {
                write!(f, "unknown Wasm section {id} at byte {offset}")
            }
        }
    }
}
impl std::error::Error for InspectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(source) => Some(source),
            _ => None,
        }
    }
}
const fn parse_error(source: wasmparser::BinaryReaderError) -> InspectionError {
    InspectionError::Parse(ParseError { source })
}

/// The exported item's core Wasm kind, independent of application method policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportKind {
    /// Function export.
    Function,
    /// Table export.
    Table,
    /// Linear memory export.
    Memory,
    /// Global export.
    Global,
    /// Exception tag export.
    Tag,
}

/// Export identity within its kind's index space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Export {
    /// Item kind.
    pub kind: ExportKind,
    /// Index within that kind's index space; not type-validated here.
    pub index: u32,
}

/// Borrowed custom metadata, preserving duplicates and encounter order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CustomSection<'a> {
    /// Decoded UTF-8 section name.
    pub name: &'a str,
    /// Exact bytes after the name, with no normalization.
    pub data: &'a [u8],
}

/// Custom-section encoding failed without returning partial output.
#[derive(Debug)]
pub enum CustomSectionEncodingError {
    /// A name/payload length exceeds Wasm's u32 domain or the host address space.
    LengthOverflow,
    /// The complete section framing exceeds the caller's byte allowance.
    LimitExceeded {
        /// Complete encoded section length, including its identifier and length.
        actual: usize,
        /// Caller-selected maximum encoded bytes.
        limit: usize,
    },
    /// Storage for the admitted encoded section could not be allocated.
    Allocation(std::collections::TryReserveError),
}

impl fmt::Display for CustomSectionEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow => f.write_str("Wasm custom-section length is unrepresentable"),
            Self::LimitExceeded { actual, limit } => {
                write!(
                    f,
                    "encoded Wasm custom section has {actual} bytes, limit {limit}"
                )
            }
            Self::Allocation(source) => {
                write!(f, "Wasm custom-section allocation failed: {source}")
            }
        }
    }
}

impl std::error::Error for CustomSectionEncodingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(source) => Some(source),
            Self::LengthOverflow | Self::LimitExceeded { .. } => None,
        }
    }
}

/// Encode one complete custom section using minimal unsigned LEB128 framing.
///
/// Includes the custom-section identifier, payload length and UTF-8 name length,
/// preserving the exact supplied name and arbitrary data bytes. Length admission
/// and the complete encoded-byte allowance are checked before allocation.
/// The allowance bounds output length; allocator overhead is outside that bound.
///
/// Callers select section names, metadata schemas, append order, duplicates and
/// aggregate module budgets. Module assembly and validation remain caller-owned;
/// this function encodes only the supplied section, without a Wasm header.
///
/// # Errors
/// Returns typed length, output-bound or allocation failures, with no partial
/// section returned. No module or caller-owned destination is modified.
#[expect(
    clippy::missing_panics_doc,
    reason = "the canonical integer writer uses Vec's infallible Write implementation after bounded allocation"
)]
pub fn encode_custom_section(
    section: CustomSection<'_>,
    max_bytes: usize,
) -> Result<Vec<u8>, CustomSectionEncodingError> {
    let (name_bytes, payload_bytes, encoded_bytes) =
        custom_section_layout(section.name.len(), section.data.len(), max_bytes)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(encoded_bytes)
        .map_err(CustomSectionEncodingError::Allocation)?;
    bytes.push(0);
    // The canonical integer writer cannot fail when its sink is an admitted Vec.
    leb128::write::unsigned(&mut bytes, u64::from(payload_bytes))
        .expect("preallocated Vec writes cannot fail");
    leb128::write::unsigned(&mut bytes, u64::from(name_bytes))
        .expect("preallocated Vec writes cannot fail");
    bytes.extend_from_slice(section.name.as_bytes());
    bytes.extend_from_slice(section.data);
    Ok(bytes)
}

fn custom_section_layout(
    name_bytes: usize,
    data_bytes: usize,
    max_bytes: usize,
) -> Result<(u32, u32, usize), CustomSectionEncodingError> {
    let overflow = || CustomSectionEncodingError::LengthOverflow;
    let name_bytes = u32::try_from(name_bytes).map_err(|_| overflow())?;
    let data_bytes = u32::try_from(data_bytes).map_err(|_| overflow())?;
    let payload_bytes = unsigned_width(name_bytes)
        .checked_add(name_bytes)
        .and_then(|bytes| bytes.checked_add(data_bytes))
        .ok_or_else(overflow)?;
    let encoded_bytes = 1 + u64::from(unsigned_width(payload_bytes)) + u64::from(payload_bytes);
    let encoded_bytes = usize::try_from(encoded_bytes).map_err(|_| overflow())?;
    if encoded_bytes > max_bytes {
        return Err(CustomSectionEncodingError::LimitExceeded {
            actual: encoded_bytes,
            limit: max_bytes,
        });
    }
    Ok((name_bytes, payload_bytes, encoded_bytes))
}

const fn unsigned_width(value: u32) -> u32 {
    if value == 0 {
        1
    } else {
        (u32::BITS - value.leading_zeros()).div_ceil(7)
    }
}

/// Core Wasm facts for consumer reports and transform-contract comparisons.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasmFacts<'a> {
    /// Complete raw artifact byte length.
    pub raw_bytes: usize,
    /// Code section payload bytes, including its vector count and body lengths.
    pub code_section_bytes: usize,
    /// Encoded function bodies and body lengths, excluding the code vector count.
    /// This is the parser's remaining code-section size, used by the IC reference.
    pub code_body_bytes: usize,
    /// Data section payload bytes, including its vector count and framing.
    pub data_section_bytes: usize,
    /// Defined functions, excluding imports; function/code counts must agree.
    pub defined_functions: u32,
    /// Imported functions, including exact-function imports.
    pub imported_functions: u32,
    /// Defined globals; initializer framing is parsed but types are not validated.
    pub defined_globals: u32,
    /// Imported globals, separate from definitions for index-space diagnostics.
    pub imported_globals: u32,
    /// Number of encoded data segments.
    pub data_segments: u32,
    /// All exports keyed by exact name; callers select IC/application methods.
    pub exports: BTreeMap<&'a str, Export>,
    /// All custom metadata; callers select Candid sections and acceptance rules.
    pub custom_sections: Vec<CustomSection<'a>>,
}

/// Inspect bounded core Wasm framing and the vectors used by artifact reports.
///
/// Preserves input bytes and borrows names/metadata. Storage is bounded by the
/// explicit export/custom-section limits; traversal is bounded by module bytes.
/// Unreported core section contents and function instructions are not validated.
/// Run the consumer's Wasm validator before admitting a deployable artifact.
///
/// # Errors
/// Rejects resource overflow, unsupported encoding, malformed framing or
/// inspected vectors, duplicate exports, unknown sections, and unequal function
/// and code counts. No partial facts are returned.
pub fn inspect(bytes: &[u8], limits: InspectionLimits) -> Result<WasmFacts<'_>, InspectionError> {
    enforce(
        InspectionResource::ModuleBytes,
        bytes.len() as u64,
        limits.module_bytes as u64,
    )?;
    let mut facts = WasmFacts {
        raw_bytes: bytes.len(),
        code_section_bytes: 0,
        code_body_bytes: 0,
        data_section_bytes: 0,
        defined_functions: 0,
        imported_functions: 0,
        defined_globals: 0,
        imported_globals: 0,
        data_segments: 0,
        exports: BTreeMap::new(),
        custom_sections: Vec::new(),
    };
    let mut sections = 0;
    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload.map_err(parse_error)?;
        if payload.as_section().is_some() {
            sections += 1;
            enforce(
                InspectionResource::Sections,
                sections,
                limits.sections as u64,
            )?;
        }
        match payload {
            Payload::Version { encoding, .. } if encoding != Encoding::Module => {
                return Err(InspectionError::UnsupportedEncoding);
            }
            Payload::ImportSection(reader) => inspect_imports(reader, &mut facts)?,
            Payload::GlobalSection(reader) => {
                facts.defined_globals = reader.count();
                for global in reader {
                    global.map_err(parse_error)?;
                }
            }
            Payload::FunctionSection(reader) => {
                facts.defined_functions = reader.count();
                // Exhaust the iterator to reject count/payload mismatch, rather
                // than trusting the count prefix as the hand-written readers did.
                for index in reader {
                    index.map_err(parse_error)?;
                }
            }
            Payload::CodeSectionStart { range, size, .. } => {
                facts.code_section_bytes = host_size(range.end - range.start)?;
                facts.code_body_bytes = host_size(u64::from(size))?;
            }
            Payload::DataSection(reader) => {
                let range = reader.range();
                facts.data_section_bytes = host_size(range.end - range.start)?;
                facts.data_segments = reader.count();
                for segment in reader {
                    segment.map_err(parse_error)?;
                }
            }
            Payload::ExportSection(reader) => inspect_exports(reader, &mut facts, limits.exports)?,
            Payload::CustomSection(reader) => {
                enforce(
                    InspectionResource::CustomSections,
                    facts.custom_sections.len() as u64 + 1,
                    limits.custom_sections as u64,
                )?;
                facts.custom_sections.push(CustomSection {
                    name: reader.name(),
                    data: reader.data(),
                });
            }
            Payload::UnknownSection { id, range, .. } => {
                return Err(InspectionError::UnknownSection {
                    id,
                    offset: host_size(range.start)?,
                });
            }
            _ => {}
        }
    }
    Ok(facts)
}

fn inspect_imports(
    reader: wasmparser::ImportSectionReader<'_>,
    facts: &mut WasmFacts<'_>,
) -> Result<(), InspectionError> {
    for import in reader.into_imports() {
        match import.map_err(parse_error)?.ty {
            TypeRef::Func(_) | TypeRef::FuncExact(_) => {
                facts.imported_functions = facts.imported_functions.checked_add(1).ok_or(
                    InspectionError::CountOverflow {
                        kind: ExportKind::Function,
                    },
                )?;
            }
            TypeRef::Global(_) => {
                facts.imported_globals = facts.imported_globals.checked_add(1).ok_or(
                    InspectionError::CountOverflow {
                        kind: ExportKind::Global,
                    },
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn inspect_exports<'a>(
    reader: wasmparser::ExportSectionReader<'a>,
    facts: &mut WasmFacts<'a>,
    limit: u32,
) -> Result<(), InspectionError> {
    enforce(
        InspectionResource::Exports,
        u64::from(reader.count()),
        u64::from(limit),
    )?;
    for entry in reader.into_iter_with_offsets() {
        let (offset, entry) = entry.map_err(parse_error)?;
        let kind = match entry.kind {
            ExternalKind::Func | ExternalKind::FuncExact => ExportKind::Function,
            ExternalKind::Table => ExportKind::Table,
            ExternalKind::Memory => ExportKind::Memory,
            ExternalKind::Global => ExportKind::Global,
            ExternalKind::Tag => ExportKind::Tag,
        };
        if facts
            .exports
            .insert(
                entry.name,
                Export {
                    kind,
                    index: entry.index,
                },
            )
            .is_some()
        {
            return Err(InspectionError::DuplicateExport {
                offset: host_size(offset)?,
            });
        }
    }
    Ok(())
}

// The parser carries u64 offsets; borrowed artifact facts use host-sized values.
// No offset or extent can require more than the host's entire addressable input.
fn host_size(value: u64) -> Result<usize, InspectionError> {
    usize::try_from(value).map_err(|_| InspectionError::UnrepresentableSize { value })
}

const fn enforce(
    resource: InspectionResource,
    actual: u64,
    limit: u64,
) -> Result<(), InspectionError> {
    if actual > limit {
        return Err(InspectionError::LimitExceeded {
            resource,
            actual,
            limit,
        });
    }
    Ok(())
}
