//! Parser-backed Candid service inventories without expanding argument types.
//!
//! The parser owns grammar and type checking. Consumers own source admission,
//! expected methods, payload annotations and publication. No imports are loaded,
//! diagnostics printed, files opened or processes run.

use crate::interface::{ExportedMethod, MethodMode};
use candid_parser::{IDLProg, TypeEnv, syntax::Dec, types::FuncMode};
use std::fmt;

#[cfg(test)]
mod tests;

/// Caller-selected input and result bounds; zero permits an empty result only.
///
/// The source bound applies before parsing. The method bound applies after type
/// checking and before projecting the inventory. These are not a parser heap,
/// stack or CPU quota: the selected parser allocates and traverses source types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceLimits {
    /// Maximum UTF-8 source bytes.
    pub source_bytes: usize,
    /// Maximum service methods returned.
    pub methods: usize,
}

/// One declared service method; arguments and return types are not expanded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceMethod {
    /// Decoded Candid method name, preserving spaces and escaped characters.
    pub name: String,
    /// Declared call mode, including oneway.
    pub mode: MethodMode,
}

impl ServiceMethod {
    /// Project this declaration to its Wasm name/mode for caller comparisons.
    /// This explicitly maps oneway to the update namespace.
    #[must_use]
    pub fn as_export(&self) -> ExportedMethod<'_> {
        ExportedMethod {
            name: &self.name,
            mode: self.mode.export_mode(),
        }
    }
}

/// Parser/type-check failure without exposing dependency types in the API.
#[derive(Debug)]
pub struct ParseError {
    source: candid_parser::Error,
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.source.fmt(formatter)
    }
}
impl std::error::Error for ParseError {}

/// Failure to produce a complete service inventory.
#[derive(Debug)]
pub enum ServiceError {
    /// Source exceeds the caller's byte allowance before parsing.
    InputLimit {
        /// Observed bytes.
        actual: usize,
        /// Caller-selected maximum.
        limit: usize,
    },
    /// Service exceeds the caller's method allowance before result allocation.
    MethodLimit {
        /// Observed methods.
        actual: usize,
        /// Caller-selected maximum.
        limit: usize,
    },
    /// Import declarations require an external resolution policy and are refused.
    ImportsUnsupported,
    /// The document has no service actor.
    MissingService,
    /// Grammar, duplicate definitions or type checking failed.
    Parse(ParseError),
    /// The result inventory could not be allocated.
    Allocation(std::collections::TryReserveError),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLimit { actual, limit } => write!(
                formatter,
                "Candid source has {actual} bytes, exceeding {limit}"
            ),
            Self::MethodLimit { actual, limit } => write!(
                formatter,
                "Candid service has {actual} methods, exceeding {limit}"
            ),
            Self::ImportsUnsupported => {
                formatter.write_str("Candid imports require caller resolution")
            }
            Self::MissingService => formatter.write_str("Candid document has no service"),
            Self::Parse(source) => source.fmt(formatter),
            Self::Allocation(_) => formatter.write_str("Candid inventory allocation failed"),
        }
    }
}

impl std::error::Error for ServiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(source) => Some(source),
            Self::Allocation(source) => Some(source),
            _ => None,
        }
    }
}

const fn parse_error(source: candid_parser::Error) -> ServiceError {
    ServiceError::Parse(ParseError { source })
}

/// Parse and type-check a self-contained document, then return its method facts.
///
/// Supports aliases, service constructors, quoted names and parser-supported
/// comments. Methods are ordered by name by the Candid service type checker.
/// Recursive/shared argument types remain in the parser's type environment and
/// are never expanded into a presentation tree. Normalization is not required.
///
/// # Errors
/// Returns input/method bounds, import refusal, missing service, parser/type
/// errors or result allocation failure. No partial inventory is returned.
pub fn inspect(source: &str, limits: ServiceLimits) -> Result<Vec<ServiceMethod>, ServiceError> {
    if source.len() > limits.source_bytes {
        return Err(ServiceError::InputLimit {
            actual: source.len(),
            limit: limits.source_bytes,
        });
    }
    let program = source.parse::<IDLProg>().map_err(parse_error)?;
    if program
        .decs
        .iter()
        .any(|declaration| matches!(declaration, Dec::ImportType(_) | Dec::ImportServ(_)))
    {
        return Err(ServiceError::ImportsUnsupported);
    }
    let mut environment = TypeEnv::new();
    let actor = candid_parser::check_prog(&mut environment, &program)
        .map_err(parse_error)?
        .ok_or(ServiceError::MissingService)?;
    let methods = environment
        .as_service(&actor)
        .map_err(|error| parse_error(error.into()))?;
    if methods.len() > limits.methods {
        return Err(ServiceError::MethodLimit {
            actual: methods.len(),
            limit: limits.methods,
        });
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(methods.len())
        .map_err(ServiceError::Allocation)?;
    for (name, ty) in methods {
        let function = environment
            .as_func(ty)
            .map_err(|error| parse_error(error.into()))?;
        let mode = match function.modes.first() {
            None => MethodMode::Update,
            Some(FuncMode::Query) => MethodMode::Query,
            Some(FuncMode::CompositeQuery) => MethodMode::CompositeQuery,
            Some(FuncMode::Oneway) => MethodMode::Oneway,
        };
        let mut owned_name = String::new();
        owned_name
            .try_reserve_exact(name.len())
            .map_err(ServiceError::Allocation)?;
        owned_name.push_str(name);
        result.push(ServiceMethod {
            name: owned_name,
            mode,
        });
    }
    Ok(result)
}
